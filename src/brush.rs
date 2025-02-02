use grid::Grid;

pub struct Brush {
    pub texture: Grid<u8>,
    pub spacing: f32,
}

impl Brush {
    pub fn width(&self) -> usize {
        self.texture.cols()
    }

    pub fn height(&self) -> usize {
        self.texture.rows()
    }
}

pub fn round_brush(diameter: usize) -> Brush {
    let mut grid = Grid::new(diameter, diameter);
    let r = diameter as f32/2.;
    for ((x, y), pixel) in grid.indexed_iter_mut() {
        let dist_to_center = ((x as f32-r).powi(2) + (y as f32 - r).powi(2)).sqrt();
        if dist_to_center == 0. {
            *pixel = u8::MAX;
        } else {
            let dist_to_edge = r - dist_to_center.min(r);
            *pixel = (dist_to_edge*(u8::MAX-1) as f32) as u8;
        }
    }
    Brush { texture: grid, spacing: 1. }
}