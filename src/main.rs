mod array_queue;
mod brush_stroke;
mod vec_map;
mod brush;
mod canvas_image;
mod raster;
mod canvas_app;
mod packed_u8;
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
