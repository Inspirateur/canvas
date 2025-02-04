use std::ops::Deref;
use std::path::PathBuf;

use eframe::egui;
use eframe::egui::*;
use eframe::App;
use glam::IVec2;
use image::ExtendedColorType;

use crate::brush::round_brush;
use crate::brush::Brush;
use crate::brush_stroke::BrushStroke;
use crate::canvas_image::CanvasImage;

/// Shrinks the given size as little as possible to fit the given aspect ratio (width/height)
fn shrink_to_aspect_ratio(size: Vec2, aspect_ratio: f32) -> Vec2 {
    Vec2::new(
        size.x.min(size.y*aspect_ratio), 
        size.y.min(size.x/aspect_ratio)
    )
}

fn to_ivec(pos: Pos2) -> IVec2 {
    IVec2 { x: pos.x as i32, y: pos.y as i32 }
}

#[derive(PartialEq, Eq)]
enum Tool {
    Brush,
    Fill,
    Selection,
}

pub struct CanvasApp {
    image: CanvasImage,
    render_texture: TextureHandle,
    tool: Tool,
    brush: Brush,
    brush_stroke: BrushStroke,
    stroke_width: u32,
    stroke_color: Color32,
    dragging: bool,
    saving_path: Option<PathBuf>,
    unsaved_changes: bool,
}

impl CanvasApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let width = 256;
        let height = 256;
        Self {
            image: CanvasImage::new(width, height),
            render_texture: _cc.egui_ctx.load_texture(
                "render",
                ColorImage::new([width, height], Color32::TRANSPARENT),
                Default::default()
            ),
            brush: round_brush(4),
            brush_stroke: BrushStroke::new(),
            tool: Tool::Brush,
            stroke_width: 3,
            stroke_color: Color32::from_rgb(25, 200, 100),
            dragging: false,
            saving_path: None,
            unsaved_changes: true,
        }
    }

    fn save(&mut self) {
        let path = match &self.saving_path {
            Some(path) => path,
            None => if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.saving_path = Some(path);
                self.saving_path.as_ref().unwrap()
            } else {
                println!("Couldn't get a path to save the image ...");
                return;
            },
        };
        let render = self.image.render();
        if let Err(err) = image::save_buffer(
            path, 
            render.as_raw(), 
            render.width() as u32, 
            render.height() as u32, 
            ExtendedColorType::Rgba8
        ) {
            println!("Couldn't save image to path, reason: {:?}", err);
        }
    }

    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.input(|i| {
            for event in &i.raw.events {
                let Event::Key { 
                    key, physical_key: _, pressed: _, repeat: _, modifiers 
                } = event else {
                    continue;
                };
                if key == &Key::S && modifiers.command && self.unsaved_changes {
                    self.save();
                    self.unsaved_changes = false;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tool, Tool::Selection, "Selection");
            ui.selectable_value(&mut self.tool, Tool::Fill, "Fill");
            ui.selectable_value(&mut self.tool, Tool::Brush, "Brush");
            ui.add_enabled(self.tool == Tool::Brush, Label::new("Size:"));
            if ui.add_enabled(
                self.tool == Tool::Brush, 
                Slider::new(&mut self.stroke_width, 1..=100).step_by(2.).logarithmic(true)
            ).changed() {
                self.brush = round_brush(self.stroke_width as usize+1);
            }
            let mut rgba = Rgba::from(self.stroke_color);
            if color_picker::color_edit_button_rgba(ui, &mut rgba, color_picker::Alpha::OnlyBlend).changed() {
                let srgba = rgba.to_srgba_unmultiplied();
                self.stroke_color = Color32::from_rgba_unmultiplied(srgba[0], srgba[1], srgba[2], srgba[3]);
            }
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.image = CanvasImage::new(self.image.width(), self.image.height());
                self.render_texture.set(
                    ColorImage::new([self.image.width(), self.image.height()], Color32::TRANSPARENT), 
                    TextureOptions::LINEAR
                );
            }
        })
        .response
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        // TODO: use this https://docs.rs/egui/latest/egui/viewport/enum.ViewportCommand.html
        // to change window title depending on save path and unsaved changes (like "my_drawing.png*")
        let response = ui
            .allocate_response(
                shrink_to_aspect_ratio(
                    ui.available_size_before_wrap(), 
                    self.image.aspect_ratio()
                ), 
                Sense::drag()
            ).on_hover_cursor(match self.tool {
                Tool::Brush => egui::CursorIcon::Crosshair,
                Tool::Fill => egui::CursorIcon::Cell,
                Tool::Selection => egui::CursorIcon::Copy,
            });
        let clip_rect = ui.clip_rect().intersect(response.rect); // Make sure we don't paint out of bounds
        let painter = ui.painter().with_clip_rect(clip_rect);

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let mut canvas_pos = from_screen * pointer_pos;
            canvas_pos.x *= self.image.width() as f32;
            canvas_pos.y *= self.image.height() as f32;
            match self.tool {
                Tool::Brush => {
                    for brush_pos in self.brush_stroke.update_stroke(canvas_pos, self.brush.spacing) {
                        self.image.add_stroke(&self.brush, &to_ivec(brush_pos));
                    }
                    self.render_texture.set(
                        self.image.preview_stroke(self.stroke_color), 
                        TextureOptions::NEAREST
                    )
                },
                Tool::Fill => {
                    if !self.dragging {
                        self.render_texture.set(
                            self.image.fill(&to_ivec(canvas_pos), self.stroke_color), 
                            TextureOptions::NEAREST
                        );
                    }
                },
                _ => {}
            }
            self.dragging = true;
            self.unsaved_changes = true;
        }
        if response.drag_stopped() {
            self.image.apply_preview(self.stroke_color);
            self.brush_stroke.clear_stroke();
            self.dragging = false;
        }
        Image::from_texture((self.render_texture.id(), self.image.dims()))
            .paint_at(&ui, painter.clip_rect());
        response
    }
}

impl App for CanvasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.ui_control(ui);
            self.ui_content(ui);
        });
    }
}