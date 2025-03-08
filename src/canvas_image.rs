use std::{collections::{BTreeMap, BTreeSet, HashSet}, u8};

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Vec2};
use glam::IVec2;
use grid::Grid;

use crate::{brush::Brush, raster::Raster, vec_map::VecMap};

pub struct CanvasImage {
    colors: VecMap<[u8; 4], Raster>,
    cached_render: ColorImage,
    current_stroke: Raster,
    dims: [usize; 2],
}

impl CanvasImage {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            colors: VecMap(Vec::new()),
            current_stroke: Raster(Grid::new(width, height)),
            dims: [width, height],
            cached_render: ColorImage::new([width, height], Color32::TRANSPARENT),
        }
    }

    pub fn render(&self) -> ColorImage {
        self.cached_render.clone()
    }

    fn raster_idx(&mut self, color: Color32) -> usize {
        let rgba = [color.r(), color.g(), color.b(), color.a()];
        match self.colors.position(&rgba) {
            Some(idx) => idx,
            None => {
                self.colors.0.push((rgba, Raster::new(&self.dims)));
                self.colors.0.len()-1
            }
        }
    }

    fn update_render(&mut self) {
        for x in 0..self.dims[0] {
            for y in 0..self.dims[1] {
                let xy = (x, y);
                // Render the colors
                let mut r = 0.;
                let mut g = 0.;
                let mut b = 0.;
                let mut a = 0;
                for ([cr, cg, cb, _], raster) in self.colors.0.iter() {
                    if raster.0[xy] == 0 {
                        continue;
                    }
                    let presence = raster.0[xy] as f32/u8::MAX as f32;
                    r += *cr as f32*presence;
                    g += *cg as f32*presence;
                    b += *cb as f32*presence; 
                    a += raster.0[xy];
                }
                self.cached_render[xy] = Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a);
            }
        }
    }

    fn update_stroke(&mut self, brush: &Brush, poses: Vec<Pos2>) -> HashSet<(usize, usize)> {
        let half_brush = IVec2::new(brush.width() as i32/2, brush.height() as i32/2);
        let mut updated_pixels = HashSet::new();
        for pos in poses.into_iter().map(|p| to_ivec(p)-half_brush) {
            for ((x, y), &val) in brush.texture.indexed_iter() {
                if val == 0 {
                    continue;
                }
                let pos = pos + IVec2::new(x as i32, y as i32);
                // early return if out of bounds
                if pos.cmplt(IVec2::ZERO).any()
                    || pos.x >= self.dims[0] as i32
                    || pos.y >= self.dims[1] as i32
                {
                    continue;
                }
                let xy = (pos.x as usize, pos.y as usize);
                // update current stroke
                self.current_stroke.0[xy] = self.current_stroke.0[xy].max(val.clone());
                updated_pixels.insert(xy);
            }
        }
        updated_pixels
    }

    pub fn preview_with(&mut self, brush: &Brush, color: Color32, poses: Vec<Pos2>) -> &ColorImage {
        let raster_i = self.raster_idx(color);
        let ca = color.a() as f32/u8::MAX as f32;
        // For each unique newly affected pixels
        for xy in self.update_stroke(brush, poses) {
            // Update the render without modifying the color presences
            let presence = self.current_stroke.0[xy] as f32*ca/u8::MAX as f32;
            let pres_mult = (1. - presence)/u8::MAX as f32;
            let mut r = 0.;
            let mut g = 0.;
            let mut b = 0.;
            let mut a = 0.;
            for (i, ([cr, cg, cb, _], raster)) in self.colors.0.iter().enumerate() {
                let r_presence = if pres_mult == 0. {
                    0.
                } else {
                    raster.0[xy] as f32 * pres_mult
                } + if i == raster_i { presence } else { 0. };
                if r_presence == 0. {
                    continue;
                }
                r += *cr as f32*r_presence;
                g += *cg as f32*r_presence;
                b += *cb as f32*r_presence; 
                a += r_presence;
            }
            self.cached_render[xy] = Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, (a*u8::MAX as f32) as u8);
        }
        &self.cached_render
    }

    fn apply_presence(&mut self, pos: (usize, usize), raster_idx: usize, presence: u8) {
        // Check what will be left for the other color after the new color is applied (could be 0)
        let spare_presence = u8::MAX - presence;
        for (_, other_presence) in self.colors.0.iter_mut() {
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
        self.colors.0[raster_idx].1.0[pos] += presence;
    }

    pub fn apply_preview(&mut self, color: Color32) {
        let raster_i = self.raster_idx(color);
        let ca = color.a() as f32/u8::MAX as f32;
        for (xy, &presence) in self.current_stroke.0.indexed_iter() {
            if presence == 0 {
                continue;
            }
            let presence = presence as f32*ca;
            let spare_presence = 1. - presence/u8::MAX as f32;
            for (_, other_presence) in self.colors.0.iter_mut() {
                // Previous color presences are affected by the new color
                // For example if the new color has 70% presence, all previous color shrink by 70% 
                // (this is true even if the total wasn't at 100%)
                if spare_presence == 0. {
                    other_presence.0[xy] = 0;
                } else {
                    other_presence.0[xy] = (other_presence.0[xy] as f32 * spare_presence) as u8;
                }
            }
            // Add the presence of the new color
            self.colors.0[raster_i].1.0[xy] += presence as u8;
    
        }
        self.current_stroke = Raster(Grid::new(self.dims[0], self.dims[1]));
    }

    pub fn add_image(&mut self, pos: (usize, usize), pixel_data: &[u8], width: usize) {
        let mut color_idx = BTreeMap::new();
        let mut _x = 0;
        let mut _y = 0;
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
                self.colors.position(&color).unwrap_or_else(|| {
                    self.colors.0.push((color.clone(), Raster::new(&self.dims)));
                    self.colors.0.len()-1
                })
            );
            self.colors[i].0[xy] = u8::MAX;
        }
        self.update_render();
    }

    fn colors_at(&self, xy: (usize, usize)) -> Vec<[u8; 4]> {
        self.colors.0.iter()
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

    pub fn fill(&mut self, pos: Pos2, color: Color32) -> &ColorImage {
        let rgba = [color.r(), color.g(), color.b(), color.a()];
        if !self.colors.contains_key(&rgba) {
            self.colors.0.push((rgba, Raster::new(&self.dims)));
        }
        let start = (pos.x as usize, pos.y as usize);
        let start_colors = self.colors_at(start);
        if start_colors.len() == 0 {
            let raster_idx = self.colors.position(&rgba).unwrap();
            self.fill_space(start, |obj, pos| {
                let mut current_presence = 0;
                for i in 0..obj.colors.0.len() {
                    if i == raster_idx {
                        continue;
                    }
                    current_presence += obj.colors[i].0[pos];
                }
                if current_presence >= u8::MAX {
                    return false;
                }
                let spare_presence = u8::MAX-current_presence;
                if obj.colors[raster_idx].0[pos] == spare_presence {
                    return false;
                }
                obj.colors[raster_idx].0[pos] = spare_presence;
                true
            });
        } else {
            let raster_idx = self.colors.position(&rgba).unwrap();
            let min_alpha = start_colors.iter().map(|c| c[3]).min().unwrap() as f32/u8::MAX as f32;
            let scaling = (rgba[3] as f32/u8::MAX as f32)/min_alpha;
            let source_rasters = start_colors.into_iter().map(|c| self.colors.position(&c).unwrap()).collect::<Vec<_>>();
            if rgba[3] < u8::MAX || scaling < 1. {
                self.fill_space(start, |obj, pos| {
                    let min_presence = source_rasters.iter().map(|&idx| obj.colors[idx].0[pos]).min().unwrap();
                    if min_presence == 0 {
                        return false;
                    }
                    obj.apply_presence(pos, raster_idx, (min_presence as f32*scaling) as u8);
                    true        
                });
            } else {
                self.fill_space(start, |obj, pos| {
                    let (min_presence, min_idx) = source_rasters.iter().map(|&idx| (obj.colors[idx].0[pos], idx)).min().unwrap();
                    if min_presence == 0 {
                        return false;
                    }
                    obj.colors[min_idx].0[pos] = 0;
                    let mut current_presence = 0;
                    for i in 0..obj.colors.0.len() {
                        if i == raster_idx {
                            continue;
                        }
                        current_presence += obj.colors[i].0[pos];
                    }
                    let spare_presence = u8::MAX-current_presence;
                    obj.colors[raster_idx].0[pos] += spare_presence.min((min_presence as f32*scaling) as u8);
                    true
                });
            }
        }
        self.update_render();
        &self.cached_render
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

    pub fn dims(&self) -> egui::Vec2 {
        Vec2::new(self.dims[0] as f32, self.dims[1] as f32)
    }

    pub fn width(&self) -> usize {
        self.dims[0]
    }

    pub fn height(&self) -> usize {
        self.dims[1]
    }

    pub fn rect(&self) -> egui::Rect {
        Rect { min: Pos2::ZERO, max: Pos2::new(self.dims[0] as f32, self.dims[1] as f32) }
    }
}

fn to_ivec(pos: Pos2) -> IVec2 {
    IVec2 { x: pos.x as i32, y: pos.y as i32 }
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