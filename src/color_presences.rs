use std::u8;
use eframe::egui::{Color32, ColorImage};
use glam::IVec2;
use grid::Grid;

use crate::{raster::Raster, vec_map::VecMap};

#[derive(Clone)]
pub struct ColorPresences {
    // [u8; 3] is rgb
    data: VecMap<[u8; 3], Raster>,
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
        let rgb = [color.r(), color.g(), color.b()];
        if !self.data.contains_key(&rgb) {
            self.data.0.push((rgb, Raster(Grid::new(self.dims[0], self.dims[1]))));
        }
        let presence_idx = self.data.0.iter().position(|(c, _)| *c == rgb).unwrap();
        for ((x, y), &new_val) in brush.indexed_iter() {
            let xy = (x + pos.x as usize, y + pos.y as usize);
            // amount of color to be applied
            let new_val = (new_val as f32 * color.a() as f32 / u8::MAX as f32) as u8;
            // Check what will be left for the other color after the new color is applied (could be 0)
            let spare_presence = u8::MAX - new_val;
            for (_, presence) in self.data.0.iter_mut() {
                // Previous color presences are affected by the new color
                // For example if the new color has 70% presence, all previous color shrink by 70% 
                // (this is true even if the total wasn't at 100%)
                if spare_presence == 0 {
                    presence.0[xy] = 0;
                } else {
                    presence.0[xy] = (presence.0[xy] as f32 * spare_presence as f32/u8::MAX as f32) as u8;
                }
            }
            // Add the presence of the new color
            self.data.0[presence_idx].1.0[xy] += new_val;
            // Render the colors
            let mut r = 0.;
            let mut g = 0.;
            let mut b = 0.;
            let mut a = 0;
            for ([cr, cg, cb], raster) in self.data.0.iter() {
                if raster.0[xy] == 0 {
                    continue;
                }
                let presence = raster.0[xy] as f32/u8::MAX as f32;
                r += *cr as f32*presence;
                g += *cg as f32*presence;
                b += *cb as f32*presence; 
                a += raster.0[xy];
            }
            pixels[xy.0+xy.1*self.dims[0]] = Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a);
        }
        ColorImage { size: self.dims, pixels }
    }
}