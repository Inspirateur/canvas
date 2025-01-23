use eframe::egui;
use eframe::egui::*;
use eframe::App;

use crate::brush::round_brush;
use crate::brush::Brush;
use crate::easings::*;
use crate::image::CanvasImage;

/// Shrinks the given rect as little as possible to fit the given aspect ratio (width/height)
pub fn shrink_to_aspect_ratio(rect: Rect, aspect_ratio: f32) -> Rect {
    let width = rect.width().min(rect.height()*aspect_ratio);
    let height = rect.height().min(rect.width()/aspect_ratio);
    Rect { min: rect.min, max: rect.min + Vec2::new(width, height) }
}

pub struct CanvasApp {
    image: CanvasImage,
    render_texture: TextureHandle,
    brush: Brush,
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
            stroke_width: 4,
            stroke_color: Color32::from_rgb(25, 200, 100),
        }
    }

    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label("Brush width:");
            if ui.add(Slider::new(&mut self.stroke_width, 1..=100)).changed() {
                self.brush = round_brush(self.stroke_width as usize, &exponential_easing);
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
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let mut canvas_pos = from_screen * pointer_pos;
            canvas_pos.x *= self.image.width() as f32;
            canvas_pos.y *= self.image.height() as f32;
            self.render_texture.set(
                self.image.preview(&self.brush, &glam::IVec2 { x: canvas_pos.x as i32, y: canvas_pos.y as i32 }, self.stroke_color), 
                TextureOptions::NEAREST
            );
        } else {
            self.image.apply_preview(self.stroke_color);
        }
        Image::from_texture((self.render_texture.id(), self.image.dims()))
            .paint_at(&ui, shrink_to_aspect_ratio(painter.clip_rect(), self.image.aspect_ratio()));
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
