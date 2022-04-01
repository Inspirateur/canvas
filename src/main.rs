mod canvas;
use canvas::Painting;
use eframe::*;

fn main() {
    run_native(Box::new(Painting::default()), NativeOptions::default());
}
