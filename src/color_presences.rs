use std::u8;
use eframe::egui::{Color32, ColorImage};
use glam::IVec2;
use grid::Grid;

use crate::{raster::Raster, vec_map::VecMap};

#[derive(Clone)]
pub struct ColorPresences {
    data: VecMap<Color32, Raster>,
    dims: [usize; 2],
}

impl ColorPresences {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: VecMap(Vec::new()),
            dims: [width, height],
        }
    }

    pub fn apply(&mut self, brush: &Grid<u8>, pos: &IVec2, color: Color32) -> ColorImage {
        let mut pixels = vec![Color32::TRANSPARENT; self.dims[0]*self.dims[1]];
        if !self.data.contains_key(&color) {
            self.data.0.push((color, Raster(Grid::new(self.dims[0], self.dims[1]))));
        }
        let presence_idx = self.data.0.iter().position(|(c, _)| *c == color).unwrap();
        for ((x, y), new_val) in brush.indexed_iter() {
            let xy = (x + pos.x as usize, y + pos.y as usize);
            // Compute the portion of the (x, y) pixel that is already occupied by color
            let mut curr_total = 0;
            for (_, presence) in self.data.0.iter() {
                curr_total += presence.0[xy];
            }
            // Check what will be left for the other color after the new color is applied (could be 0)
            let spare_presence = u8::MAX - new_val;
            if spare_presence < curr_total {
                for (_, presence) in self.data.0.iter_mut() {
                    // the previous colors will share what space is left
                    // sharing of an integer value (the presence) normally requires something like https://en.wikipedia.org/wiki/Largest_remainder_method
                    // but here we don't care if some presence is lost
                    if spare_presence == 0 {
                        presence.0[xy] = 0;
                    } else {
                        presence.0[xy] = (presence.0[xy] as u32*spare_presence as u32/curr_total as u32) as u8;
                    }
                }
            }
            // Add the presence of the new color
            self.data.0[presence_idx].1.0[xy] += *new_val;
            // Render the colors
            let mut r = 0;
            let mut g = 0;
            let mut b = 0;
            let mut a = 0;
            for (color, raster) in self.data.0.iter() {
                let t = color.a() as u32*raster.0[xy] as u32;
                let u8maxsq = u8::MAX as u32*u8::MAX as u32;
                r += color.r() as u32*t/u8maxsq;
                g += color.g() as u32*t/u8maxsq;
                b += color.b() as u32*t/u8maxsq;
                a += color.a() as u32*raster.0[xy] as u32/u8::MAX as u32;
            }
            pixels[xy.0+xy.1*self.dims[0]] = Color32::from_rgba_premultiplied(r as u8, g as u8, b as u8, a as u8);
        }
        ColorImage { size: self.dims, pixels }
    }
}