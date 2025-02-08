use glam::IVec2;
use grid::Grid;

#[derive(Clone)]
pub struct Raster(pub Grid<u8>);

impl Raster {
    pub fn new(dims: &[usize; 2]) -> Self {
        Self(Grid::new(dims[0], dims[1]))
    }

    pub fn set_max(&mut self, other: &Grid<u8>, pos: &IVec2) {
        for ((x, y), val) in other.indexed_iter() {
            let pos = pos + IVec2::new(x as i32, y as i32);
            if pos.cmplt(IVec2::ZERO).any()
                || pos.x >= self.0.size().0 as i32
                || pos.y >= self.0.size().1 as i32
            {
                continue;
            }
            let xy = (pos.x as usize, pos.y as usize);
            self.0[xy] = self.0[xy].max(val.clone());
        }
    }
}
