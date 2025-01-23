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

pub fn round_brush(diameter: usize, opacity_easing: &dyn Fn(f32) -> f32) -> Brush {
    let mut grid = Grid::new(diameter, diameter);
    let r = diameter as f32/2.;
    for ((x, y), pixel) in grid.indexed_iter_mut() {
        let dist_to_center = ((x as f32-r).powi(2) + (y as f32 - r).powi(2)).sqrt();
        // the value we feed to the easing function needs to be between 0 and 1
        // with 0 being the "start" of the animation (here the edge of the brush)
        // and 1 being the "end" (here the center of brush)
        let t = 1. - dist_to_center.min(r)/r;
        *pixel = (opacity_easing(t)*u8::MAX as f32) as u8;
    }
    Brush { texture: grid, spacing: 1. }
}