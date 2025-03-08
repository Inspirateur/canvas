use std::borrow::Cow;
use std::path::PathBuf;
use arboard::Clipboard;
use arboard::ImageData;
use eframe::egui;
use eframe::egui::*;
use eframe::App;
use image::ExtendedColorType;

use crate::brush::round_brush;
use crate::brush::Brush;
use crate::brush_stroke::BrushStroke;
use crate::canvas_image::CanvasImage;

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
    last_title: String,
    clipboard: Clipboard,
    camera: Rect,
}

impl CanvasApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let width = 1280;
        let height = 720;
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
            last_title: String::new(),
            clipboard: Clipboard::new().unwrap(),
            camera: Rect::ZERO,
        }
    }

    fn title(&self) -> String {
        let Some(path) = &self.saving_path else {
            return "New drawing.png*".to_string();
        };
        format!("{}{}", path.file_name().unwrap().to_str().unwrap(), if self.unsaved_changes {
            "*"
        } else { "" })
    }

    fn save(&mut self) {
        let path = match &self.saving_path {
            Some(path) => path,
            None => if let Some(path) = rfd::FileDialog::new().save_file() {
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

    fn paste(&mut self) {
        let Ok(img) = self.clipboard.get_image() else {
            return;
        };
        self.image.add_image((0, 0), &img.bytes, img.width);
    }

    fn copy(&mut self) {
        if let Err(err) = self.clipboard.set_image(ImageData {
            width: self.image.width(),
            height: self.image.height(),
            bytes: Cow::Borrowed(self.image.render().as_raw())
        }) {
            println!("Couldn't copy the image, reason: {:?}", err);
        }
    }

    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
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
        });
    }

    pub fn ui_content(&mut self, ui: &mut Ui) {
        // TODO: 
        // 1. remove the jitter when clamping the camera pos
        // 2. make Scene panning controlled by middle click or CTRL Click (for tablet pen)
        // 3. make Scene panning work when cursor is inside canvas (currently doesn't work because allocate_response eats the event ?)
        self.camera.set_center(self.image.rect().clamp(self.camera.center()));
        Scene::new().zoom_range(0.1..=8.0).show(ui, &mut self.camera, |ui| {
            let response = ui
                .allocate_response(self.image.dims(), Sense::drag())
                .on_hover_cursor(match self.tool {
                    Tool::Brush => egui::CursorIcon::Crosshair,
                    Tool::Fill => egui::CursorIcon::Cell,
                    Tool::Selection => egui::CursorIcon::Copy,
                });
            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
                response.rect,
            );
            let from_screen = to_screen.inverse();
            if response.dragged_by(PointerButton::Primary) {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let mut canvas_pos = from_screen * pointer_pos;
                    let min_axis = self.image.width().min(self.image.height()) as f32;
                    canvas_pos.x *= min_axis;
                    canvas_pos.y *= min_axis;
                    match self.tool {
                        Tool::Brush => {
                            self.render_texture.set(
                                self.image.preview_with(
                                    &self.brush, 
                                    self.stroke_color, 
                                    self.brush_stroke.update_stroke(canvas_pos, self.brush.spacing)
                                ).clone(), 
                                TextureOptions::NEAREST
                            )
                        },
                        Tool::Fill => {
                            if !self.dragging {
                                self.render_texture.set(
                                    self.image.fill(canvas_pos, self.stroke_color).clone(), 
                                    TextureOptions::NEAREST
                                );
                            }
                        },
                        _ => {}
                    }
                    self.dragging = true;
                    self.unsaved_changes = true;
                }    
            }
            if response.drag_stopped() {
                self.image.apply_preview(self.stroke_color);
                self.brush_stroke.clear_stroke();
                self.dragging = false;
            }
            Image::from_texture((self.render_texture.id(), self.image.dims()))
                .bg_fill(Color32::WHITE)
                .paint_at(&ui, self.image.rect());
            response
        });
    }
}

impl App for CanvasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.input(|i| {
                for event in &i.raw.events {
                    let Event::Key { 
                        key, physical_key: _, pressed: _, repeat: _, modifiers 
                    } = event else {
                        continue;
                    };
                    if !modifiers.command {
                        continue;
                    }
                    if key == &Key::S && self.unsaved_changes {
                        self.unsaved_changes = false;
                        self.save();
                    } else if key == &Key::V {
                        // Serious performance issues (kinda expected), need to chunk the color presences
                        self.paste();
                        self.render_texture.set(self.image.render(), TextureOptions::NEAREST);
                    } else if key == &Key::C {
                        self.copy();
                    }
                }
            });
            self.ui_control(ui);
            self.ui_content(ui);
        });
        let title = self.title();
        if title != self.last_title {
            ctx.send_viewport_cmd(ViewportCommand::Title(self.title()));
            self.last_title = title;
        }
    }
}