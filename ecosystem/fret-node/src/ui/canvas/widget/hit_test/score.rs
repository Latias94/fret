pub(super) struct BestByDistance<K: Ord + Copy, V: Copy> {
    eps: f32,
    best: Option<(K, V, f32)>,
}

impl<K: Ord + Copy, V: Copy> BestByDistance<K, V> {
    pub fn new(zoom: f32) -> Self {
        Self::with_eps(super::zoom_eps(zoom))
    }

    pub fn with_eps(eps: f32) -> Self {
        Self { eps, best: None }
    }

    pub fn consider(&mut self, key: K, value: V, d2: f32) {
        match self.best {
            Some((best_key, _best_value, best_d2)) => {
                let better = if d2 + self.eps < best_d2 {
                    true
                } else if (d2 - best_d2).abs() <= self.eps {
                    key < best_key
                } else {
                    false
                };
                if better {
                    self.best = Some((key, value, d2));
                }
            }
            None => self.best = Some((key, value, d2)),
        }
    }

    pub fn into_value(self) -> Option<V> {
        self.best.map(|(_, v, _)| v)
    }
}
