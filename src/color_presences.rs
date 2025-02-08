use std::{collections::BTreeMap, u8};
use eframe::egui::{Color32, ColorImage};
use glam::IVec2;
use grid::Grid;

use crate::{raster::Raster, vec_map::VecMap};

#[derive(Clone)]
pub struct ColorPresences {
    // [u8; 4] is rgba
    data: VecMap<[u8; 4], Raster>,
    dims: [usize; 2],
}

impl ColorPresences {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: VecMap(Vec::new()),
            dims: [width, height],
        }
    }

    pub fn add_image(&mut self, pos: (usize, usize), pixel_data: &[u8], width: usize) {
        let mut color_idx = BTreeMap::new();
        let mut _x = 0;
        let mut _y = 0;
        // TODO: translate and add pixel data to color presences
        for ((x, y), color) in pixel_data.chunks(4).map(|c| {
            let res = ((_x, _y), c);
            _x += 1;
            if _x >= width {
                _x = 0;
                _y += 1;
            }
            res
        }) {
            let xy = (pos.0+x, pos.1+y);
            if xy.0 >= self.dims[0] || xy.1 >= self.dims[1] {
                continue;
            }
            let color: [u8; 4] = color.try_into().unwrap();
            let i: usize = *color_idx.entry(color).or_insert_with(|| 
                self.data.position(&color).unwrap_or_else(|| {
                    self.data.0.push((color.clone(), Raster::new(&self.dims)));
                    self.data.0.len()-1
                })
            );
            self.data[i].0[xy] = u8::MAX;

        }
    }

    fn apply_presence(&mut self, pos: (usize, usize), raster_idx: usize, presence: u8) {
        // Check what will be left for the other color after the new color is applied (could be 0)
        let spare_presence = u8::MAX - presence;
        for (_, other_presence) in self.data.0.iter_mut() {
            // Previous color presences are affected by the new color
            // For example if the new color has 70% presence, all previous color shrink by 70% 
            // (this is true even if the total wasn't at 100%)
            if spare_presence == 0 {
                other_presence.0[pos] = 0;
            } else {
                other_presence.0[pos] = (other_presence.0[pos] as f32 * spare_presence as f32/u8::MAX as f32) as u8;
            }
        }
        // Add the presence of the new color
        self.data.0[raster_idx].1.0[pos] += presence;
    }

    pub fn apply(&mut self, brush: &Grid<u8>, pos: &IVec2, color: Color32) {
        let rgba = [color.r(), color.g(), color.b(), color.a()];
        if !self.data.contains_key(&rgba) {
            self.data.0.push((rgba, Raster::new(&self.dims)));
        }
        let presence_idx = self.data.0.iter().position(|(c, _)| *c == rgba).unwrap();
        for ((x, y), &new_val) in brush.indexed_iter() {
            let xy = (x + pos.x as usize, y + pos.y as usize);
            // amount of color to be applied
            let new_val = (new_val as f32 * color.a() as f32 / u8::MAX as f32) as u8;
            self.apply_presence(xy, presence_idx, new_val);
        }
    }

    pub fn render(&self) -> ColorImage {
        let mut pixels = vec![Color32::TRANSPARENT; self.dims[0]*self.dims[1]];
        for x in 0..self.dims[0] {
            for y in 0..self.dims[1] {
                let xy = (x, y);
                // Render the colors
                let mut r = 0.;
                let mut g = 0.;
                let mut b = 0.;
                let mut a = 0;
                for ([cr, cg, cb, _], raster) in self.data.0.iter() {
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
        }
        ColorImage { size: self.dims, pixels }
    }

    fn colors_at(&self, xy: (usize, usize)) -> Vec<[u8; 4]> {
        self.data.0.iter()
            .filter_map(|(rgba, raster)| {
                let val = raster.0[xy];
                if val > 0 {
                    Some(rgba.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn fill(&mut self, pos: &IVec2, color: Color32) {
        let rgba = [color.r(), color.g(), color.b(), color.a()];
        if !self.data.contains_key(&rgba) {
            self.data.0.push((rgba, Raster::new(&self.dims)));
        }
        let start = (pos.x as usize, pos.y as usize);
        let start_colors = self.colors_at(start);
        if start_colors.len() == 0 {
            let raster_idx = self.data.position(&rgba).unwrap();
            self.fill_space(start, |obj, pos| {
                let mut current_presence = 0;
                for i in 0..obj.data.0.len() {
                    if i == raster_idx {
                        continue;
                    }
                    current_presence += obj.data[i].0[pos];
                }
                if current_presence >= u8::MAX {
                    return false;
                }
                let spare_presence = u8::MAX-current_presence;
                if obj.data[raster_idx].0[pos] == spare_presence {
                    return false;
                }
                obj.data[raster_idx].0[pos] = spare_presence;
                true
            });
        } else {
            let raster_idx = self.data.position(&rgba).unwrap();
            let min_alpha = start_colors.iter().map(|c| c[3]).min().unwrap() as f32/u8::MAX as f32;
            let scaling = (rgba[3] as f32/u8::MAX as f32)/min_alpha;
            let source_rasters = start_colors.into_iter().map(|c| self.data.position(&c).unwrap()).collect::<Vec<_>>();
            if rgba[3] < u8::MAX || scaling < 1. {
                self.fill_space(start, |obj, pos| {
                    let min_presence = source_rasters.iter().map(|&idx| obj.data[idx].0[pos]).min().unwrap();
                    if min_presence == 0 {
                        return false;
                    }
                    obj.apply_presence(pos, raster_idx, (min_presence as f32*scaling) as u8);
                    true        
                });
            } else {
                self.fill_space(start, |obj, pos| {
                    let (min_presence, min_idx) = source_rasters.iter().map(|&idx| (obj.data[idx].0[pos], idx)).min().unwrap();
                    if min_presence == 0 {
                        return false;
                    }
                    obj.data[min_idx].0[pos] = 0;
                    let mut current_presence = 0;
                    for i in 0..obj.data.0.len() {
                        if i == raster_idx {
                            continue;
                        }
                        current_presence += obj.data[i].0[pos];
                    }
                    let spare_presence = u8::MAX-current_presence;
                    obj.data[raster_idx].0[pos] += spare_presence.min((min_presence as f32*scaling) as u8);
                    true
                });
            }
        }
    }

    /// Fills the biggest horizontal span from seed within [min, max[, returning [left, right[ edges of span
    /// if seed cannot be filled then (seed.x, seed.x) is returned, 
    /// so left < right can be used to check if operation was succesful
    fn fill_span<Func>(&mut self, seed: (usize, usize), min: usize, max: usize, pixel_fill: &mut Func) -> (usize, usize)
        where Func: FnMut(&mut Self, (usize, usize)) -> bool
    {
        if !pixel_fill(self, seed) {
            return (seed.0, seed.0);
        }
        let mut left = seed.0;
        loop {
            if left > min {
                left -= 1;
            } else {
                break;
            }
            if !pixel_fill(self, (left, seed.1)) {
                left += 1;
                break;
            }
        }
        let mut right = seed.0+1;
        while right < max && pixel_fill(self, (right, seed.1)) {
            right += 1;
        }
        (left, right)
    }

    fn fill_space<Func>(&mut self, start: (usize, usize), mut pixel_fill: Func) 
        where Func: FnMut(&mut Self, (usize, usize)) -> bool
    {
        // special case for the first span
        let start_span = self.fill_span(start, 0, self.dims[0], &mut pixel_fill);
        let mut spans = Vec::new();
        self.checked_span_add(&mut spans, start_span, start.1, From::Above);
        self.checked_span_add(&mut spans, start_span, start.1, From::Below);
        while let Some(span) = spans.pop() {
            let child_span = self.fill_span((span.left, span.y), 0, self.dims[1], &mut pixel_fill);
            self.checked_span_add(&mut spans, child_span, span.y, span.from);
            let left = child_span.0;
            let mut right = child_span.1+1;
            while right < span.right {
                let child_span = self.fill_span((right, span.y), span.left, self.dims[1], &mut pixel_fill);
                self.checked_span_add(&mut spans, child_span, span.y, span.from);
                right = child_span.1+1;
            }
            right -= 1;
            self.checked_span_add(&mut spans, (left, span.left - if span.left > 0 { 1 } else { 0 }), span.y, span.from.opposite());
            self.checked_span_add(&mut spans, (span.right+1, right), span.y, span.from.opposite());
        }
    }

    fn checked_span_add(&self, spans: &mut Vec<FillSpan>, span: (usize, usize), y: usize, dir: From) {
        if span.0 >= span.1 {
            return;
        }
        if span.0 >= self.dims[0] {
            return;
        }
        match dir {
            From::Above => {
                if y == 0 {
                    return;
                }
                spans.push(
                    FillSpan {
                        left: span.0,
                        right: span.1,
                        y: y-1,
                        from: dir
                    }
                );
            },
            From::Below => {
                if y+1 >= self.dims[1] {
                    return;
                }
                spans.push(
                    FillSpan {
                        left: span.0,
                        right: span.1,
                        y: y+1,
                        from: dir
                    }
                );
            },
        }
    }
}

#[derive(Clone, Copy)]
enum From {
    Above,
    Below
}

impl From {
    pub fn opposite(&self) -> Self {
        match self {
            From::Above => From::Below,
            From::Below => From::Above,
        }
    }
}

struct FillSpan {
    left: usize,
    right: usize,
    y: usize,
    from: From
}