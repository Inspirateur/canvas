pub struct VecMap<K, V>(pub Vec<(K, V)>);

impl<K: Eq, V> VecMap<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.0.iter().any(|(k, _)| k == key)
    }
    
    pub fn get(&self, key: &K) -> Option<&V> {
        for (k, v) in &self.0 {
            if k == key {
                return Some(v)
            }
        }
        None
    }
}

impl<K: Clone, V: Clone> Clone for VecMap<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}