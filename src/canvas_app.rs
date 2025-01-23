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
    color: Color32,
    brush: Brush,
    render_texture: TextureHandle,
}

impl CanvasApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            image: CanvasImage::new(128, 128),
            color: Color32::from_rgb(25, 200, 100),
            brush: round_brush(5, &exponential_easing),
            render_texture: _cc.egui_ctx.load_texture(
                "render",
                egui::ColorImage::example(),
                Default::default()
            )
        }
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
            canvas_pos.x *= self.image.width();
            canvas_pos.y *= self.image.height();
            self.render_texture.set(
                self.image.preview(&self.brush, &glam::IVec2 { x: canvas_pos.x as i32, y: canvas_pos.y as i32 }, self.color), 
                TextureOptions::NEAREST
            );
        } else {
            self.image.apply_preview(self.color);
        }
        Image::from_texture((self.render_texture.id(), self.image.dims()))
            .paint_at(&ui, shrink_to_aspect_ratio(painter.clip_rect(), self.image.aspect_ratio()));
        response
    }
}

impl App for CanvasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.ui_content(ui);
        });
    }
}
