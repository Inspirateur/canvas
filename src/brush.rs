use eframe::egui::Color32;
use grid::Grid;

pub struct Brush {
    pub texture: Grid<u8>,
    pub spacing: f32,
}
