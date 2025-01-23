use eframe::egui::{self, Color32, ColorImage, Vec2};
use glam::IVec2;
use grid::Grid;

use crate::{brush::Brush, color_presences::ColorPresences, raster::Raster};

pub struct CanvasImage {
    colors: ColorPresences,
    current_stroke: Raster,
    dims: [usize; 2],
}

impl CanvasImage {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            colors: ColorPresences::new(width, height),
            current_stroke: Raster(Grid::new(width, height)),
            dims: [width, height],
        }
    }

    pub fn preview(&mut self, brush: &Brush, pos: &IVec2, color: Color32) -> ColorImage {
        // adjust pos so that it is at the center of the brush
        let pos = pos - IVec2::new(brush.width() as i32/2, brush.height() as i32/2);
        // Accumulate the new brush stroke in the ongoing stroke
        self.current_stroke.set_max(&brush.texture, &pos);
        // Render a new preview with the updated stroke
        let mut preview = self.colors.clone();
        preview.apply(&self.current_stroke.0, &IVec2::ZERO, color)
    }

    pub fn apply_preview(&mut self, color: Color32) {
        self.colors.apply(&self.current_stroke.0, &IVec2::ZERO, color);
        self.current_stroke = Raster(Grid::new(self.dims[0], self.dims[1]));
    }

    pub fn dims(&self) -> egui::Vec2 {
        Vec2::new(self.dims[0] as f32, self.dims[1] as f32)
    }

    pub fn width(&self) -> usize {
        self.dims[0]
    }

    pub fn height(&self) -> usize {
        self.dims[1]
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.dims[0] as f32/self.dims[1] as f32
    }
}
