use eframe::egui;
use eframe::egui::*;
use eframe::App;

use crate::brush::round_brush;
use crate::brush::Brush;
use crate::brush_stroke::BrushStroke;
use crate::easings::*;
use crate::image::CanvasImage;

/// Shrinks the given size as little as possible to fit the given aspect ratio (width/height)
pub fn shrink_to_aspect_ratio(size: Vec2, aspect_ratio: f32) -> Vec2 {
    Vec2::new(
        size.x.min(size.y*aspect_ratio), 
        size.y.min(size.x/aspect_ratio)
    )
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
}

impl CanvasApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let width = 128;
        let height = 128;
        Self {
            image: CanvasImage::new(width, height),
            render_texture: _cc.egui_ctx.load_texture(
                "render",
                ColorImage::new([width, height], Color32::TRANSPARENT),
                Default::default()
            ),
            brush: round_brush(5, &exponential_easing),
            brush_stroke: BrushStroke::new(),
            tool: Tool::Brush,
            stroke_width: 4,
            stroke_color: Color32::from_rgb(25, 200, 100),
        }
    }

    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tool, Tool::Selection, "Selection");
            ui.selectable_value(&mut self.tool, Tool::Fill, "Fill");
            ui.selectable_value(&mut self.tool, Tool::Brush, "Brush");
            ui.add_enabled(self.tool == Tool::Brush, Label::new("Size:"));
            if ui.add_enabled(
                self.tool == Tool::Brush, 
                Slider::new(&mut self.stroke_width, 1..=100).logarithmic(true)
            ).changed() {
                self.brush = round_brush(self.stroke_width as usize+1, &exponential_easing);
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
                        self.image.add_stroke(&self.brush, &glam::IVec2 { x: brush_pos.x as i32, y: brush_pos.y as i32 });
                    }
                    self.render_texture.set(
                        self.image.preview_stroke(self.stroke_color), 
                        TextureOptions::NEAREST
                    )
                },
                _ => {}
            }
        }
        if response.drag_stopped() {
            self.image.apply_preview(self.stroke_color);
            self.brush_stroke.clear_stroke();
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
