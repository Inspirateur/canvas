mod array_queue;
mod brush_stroke;
mod vec_map;
mod brush;
mod color_presences;
mod image;
mod raster;
mod canvas_app;
use canvas_app::CanvasApp;
use eframe::Result;

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Canvas",
        native_options,
        Box::new(|cc| Ok(Box::new(CanvasApp::new(cc)))),
    )?;
    Ok(())
}
