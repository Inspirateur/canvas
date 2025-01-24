use eframe::egui::Pos2;

use crate::array_queue::ArrayQueue;

pub struct BrushStroke {
    // can store as many past positions as we need 
    // depending on the stroke correction algorithm
    pos_buffer: ArrayQueue<Pos2, 3>,
}

impl BrushStroke {
    pub fn new() -> Self {
        Self { pos_buffer: ArrayQueue::new() }
    }
    /// Takes a new brush position as well as a brush spacing value, 
    /// and outputs all the position the brush needs to be applied to
    pub fn update_stroke(&mut self, new_pos: Pos2, spacing: f32) -> Vec<Pos2> {
        self.pos_buffer.push(new_pos);
        if self.pos_buffer.len() == 1 {
            return vec![new_pos];
        }
        // for now we just trace a line between the new pos and the previous pos
        spaced_lerp(self.pos_buffer[1], self.pos_buffer[0], spacing)
    }

    pub fn clear_stroke(&mut self) {
        self.pos_buffer.clear();
    }
}

fn spaced_lerp(start: Pos2, end: Pos2, spacing: f32) -> Vec<Pos2> {
    let diff = start-end;
    let dist = diff.length();
    let step = diff/dist;
    let mut i = 0.;
    let mut res = Vec::new();
    while i < dist {
        res.push(end+step*i);
        i += spacing;
    }
    res
}