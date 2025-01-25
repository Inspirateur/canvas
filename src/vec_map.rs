use std::ops::{Index, IndexMut};

pub struct VecMap<K, V>(pub Vec<(K, V)>);

impl<K: Eq, V> VecMap<K, V> {
    pub fn contains_key(&self, key: &K) -> bool {
        self.0.iter().any(|(k, _)| k == key)
    }

    pub fn position(&self, key: &K) -> Option<usize> {
        let mut i = 0;
        for (k, _) in &self.0 {
            if k == key {
                return Some(i);
            }
            i += 1;
        }
        None
    }
}

impl<K: Eq, V> Index<usize> for VecMap<K, V> {
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index].1
    }
}

impl<K: Eq, V> IndexMut<usize> for VecMap<K, V> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index].1
    }
}

impl<K: Clone, V: Clone> Clone for VecMap<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}