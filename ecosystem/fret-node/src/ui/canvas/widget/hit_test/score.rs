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

pub(super) struct BestPortByNodeRank {
    best: Option<(crate::core::PortId, u32)>,
}

impl BestPortByNodeRank {
    pub fn new() -> Self {
        Self { best: None }
    }

    pub fn consider(&mut self, port_id: crate::core::PortId, node_rank: u32) {
        match self.best {
            Some((best_id, best_rank)) => {
                if node_rank > best_rank || (node_rank == best_rank && port_id < best_id) {
                    self.best = Some((port_id, node_rank));
                }
            }
            None => self.best = Some((port_id, node_rank)),
        }
    }

    pub fn into_port_id(self) -> Option<crate::core::PortId> {
        self.best.map(|(id, _)| id)
    }
}

pub(super) struct BestLoosePort {
    eps: f32,
    best: Option<(crate::core::PortId, f32, bool, u32)>,
}

impl BestLoosePort {
    pub fn new(zoom: f32) -> Self {
        Self {
            eps: super::zoom_eps(zoom),
            best: None,
        }
    }

    pub fn consider(&mut self, port_id: crate::core::PortId, d2: f32, preferred: bool, rank: u32) {
        let better = match self.best {
            Some((best_id, best_d2, best_preferred, best_rank)) => {
                if d2 + self.eps < best_d2 {
                    true
                } else if (d2 - best_d2).abs() <= self.eps {
                    if preferred != best_preferred {
                        preferred
                    } else if rank > best_rank {
                        true
                    } else {
                        rank == best_rank && port_id < best_id
                    }
                } else {
                    false
                }
            }
            None => true,
        };

        if better {
            self.best = Some((port_id, d2, preferred, rank));
        }
    }

    pub fn into_port_id(self) -> Option<crate::core::PortId> {
        self.best.map(|(id, _d2, _preferred, _rank)| id)
    }
}
