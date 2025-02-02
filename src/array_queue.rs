use std::ops::Index;

pub struct ArrayQueue<T: Copy, const N: usize>(Vec<T>);

impl<T: Copy, const N: usize> ArrayQueue<T, N> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push(&mut self, val: T) {
        if self.0.len() < N {
            self.0.insert(0, val);
        } else {
            self.0.copy_within(0..(N-1), 1);
            self.0[0] = val;
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Copy, const N: usize> Index<usize> for ArrayQueue<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}