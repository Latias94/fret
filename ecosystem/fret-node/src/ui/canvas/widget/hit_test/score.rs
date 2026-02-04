use super::super::EdgeEndpoint;
use crate::core::{EdgeId, PortId};

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
    best: Option<(PortId, u32)>,
}

impl BestPortByNodeRank {
    pub fn new() -> Self {
        Self { best: None }
    }

    pub fn consider(&mut self, port_id: PortId, node_rank: u32) {
        match self.best {
            Some((best_id, best_rank)) => {
                if node_rank > best_rank || (node_rank == best_rank && port_id < best_id) {
                    self.best = Some((port_id, node_rank));
                }
            }
            None => self.best = Some((port_id, node_rank)),
        }
    }

    pub fn into_port_id(self) -> Option<PortId> {
        self.best.map(|(id, _)| id)
    }
}

pub(super) struct BestLoosePort {
    eps: f32,
    best: Option<(PortId, f32, bool, u32)>,
}

impl BestLoosePort {
    pub fn new(zoom: f32) -> Self {
        Self {
            eps: super::zoom_eps(zoom),
            best: None,
        }
    }

    pub fn consider(&mut self, port_id: PortId, d2: f32, preferred: bool, rank: u32) {
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

    pub fn into_port_id(self) -> Option<PortId> {
        self.best.map(|(id, _d2, _preferred, _rank)| id)
    }
}

pub(super) struct BestEdgeByDistance {
    inner: BestByDistance<EdgeId, EdgeId>,
}

impl BestEdgeByDistance {
    pub fn new(zoom: f32) -> Self {
        Self {
            inner: BestByDistance::new(zoom),
        }
    }

    pub fn consider(&mut self, edge_id: EdgeId, d2: f32) {
        self.inner.consider(edge_id, edge_id, d2);
    }

    pub fn into_edge_id(self) -> Option<EdgeId> {
        self.inner.into_value()
    }
}

pub(super) struct BestEdgeFocusAnchorByDistance {
    inner: BestByDistance<(EdgeId, u8, PortId), (EdgeId, EdgeEndpoint, PortId)>,
}

impl BestEdgeFocusAnchorByDistance {
    pub fn new(zoom: f32) -> Self {
        let z = zoom.max(1.0e-6);
        Self {
            inner: BestByDistance::with_eps(super::zoom_eps(z)),
        }
    }

    pub fn consider(&mut self, edge_id: EdgeId, endpoint: EdgeEndpoint, fixed: PortId, d2: f32) {
        let endpoint_order = match endpoint {
            EdgeEndpoint::From => 0u8,
            EdgeEndpoint::To => 1u8,
        };
        self.inner.consider(
            (edge_id, endpoint_order, fixed),
            (edge_id, endpoint, fixed),
            d2,
        );
    }

    pub fn into_value(self) -> Option<(EdgeId, EdgeEndpoint, PortId)> {
        self.inner.into_value()
    }
}
