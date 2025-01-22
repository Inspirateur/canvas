use glam::IVec2;
use grid::Grid;

#[derive(Clone)]
pub struct Raster<T>(pub Grid<T>);

impl<T: Clone> Raster<T> {
    pub fn set(&mut self, other: &Grid<T>, pos: &IVec2) {
        for ((x, y), val) in other.indexed_iter() {
            let pos = pos + IVec2::new(x as i32, y as i32);
            if pos.cmplt(IVec2::ZERO).any()
                || pos.x >= self.0.size().0 as i32
                || pos.y >= self.0.size().1 as i32
            {
                continue;
            }
            self.0[(pos.x as usize, pos.y as usize)] = val.clone();
        }
    }
}
