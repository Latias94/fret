use super::*;
use crate::ui::canvas::state::DrawOrderFingerprint;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn repair_focused_edge_after_graph_change<H: UiHost>(
        &mut self,
        host: &mut H,
        preferred: Option<EdgeId>,
    ) {
        if preferred.is_none() && self.interaction.focused_edge.is_none() {
            return;
        }

        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.edges_focusable && !snapshot.interaction.edges_reconnectable {
            self.interaction.focused_edge = None;
            return;
        }

        let (edges, current_valid) = self
            .graph
            .read_ref(host, |g| {
                let mut edges: Vec<EdgeId> = g.edges.keys().copied().collect();
                edges.sort_unstable();
                let current = self.interaction.focused_edge;
                let current_valid = current.is_some_and(|id| g.edges.contains_key(&id));
                (edges, current_valid)
            })
            .ok()
            .unwrap_or_default();

        if edges.is_empty() {
            self.interaction.focused_edge = None;
            return;
        }

        if current_valid {
            return;
        }

        let base = preferred.or(self.interaction.focused_edge);
        let next = match base {
            Some(id) => match edges.binary_search(&id) {
                Ok(ix) => edges.get(ix).copied(),
                Err(ix) => edges.get(ix).copied().or_else(|| edges.first().copied()),
            },
            None => edges.first().copied(),
        };
        self.interaction.focused_edge = next;
    }

    pub(super) fn draw_order_fingerprint(ids: &[GraphNodeId]) -> DrawOrderFingerprint {
        fn mix64(mut x: u64) -> u64 {
            x ^= x >> 33;
            x = x.wrapping_mul(0xff51afd7ed558ccd);
            x ^= x >> 33;
            x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
            x ^= x >> 33;
            x
        }

        let mut lo: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut hi: u64 = 0xc2b2_ae3d_27d4_eb4f;
        let len = ids.len() as u64;
        lo = lo ^ mix64(len);
        hi = hi ^ mix64(len.wrapping_add(0x1656_67b1_9e37_79f9));

        for id in ids {
            let u = id.0.as_u128();
            let a = u as u64;
            let b = (u >> 64) as u64;

            lo = lo.wrapping_add(mix64(a ^ 0x243f_6a88_85a3_08d3));
            lo = lo.rotate_left(27) ^ hi;
            lo = lo.wrapping_mul(0x9e37_79b9_7f4a_7c15);

            hi = hi.wrapping_add(mix64(b ^ 0x1319_8a2e_0370_7344));
            hi = hi.rotate_left(31) ^ lo;
            hi = hi.wrapping_mul(0xc2b2_ae3d_27d4_eb4f);
        }

        DrawOrderFingerprint {
            lo: mix64(lo),
            hi: mix64(hi),
        }
    }

    pub(super) fn focus_port_direction<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        dir: PortNavDir,
    ) -> bool {
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        if self.interaction.focused_port.is_none() {
            return self.focus_next_port(host, true);
        }

        let from_port = self.interaction.focused_port;
        let Some(from_port) = from_port else {
            return false;
        };

        let Some(from_center) = self.port_center_canvas(host, snapshot, from_port) else {
            return false;
        };

        let required_dir = self.interaction.wire_drag.as_ref().and_then(|w| {
            let from_port = match &w.kind {
                WireDragKind::New { from, .. } => Some(*from),
                WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
                WireDragKind::ReconnectMany { edges } => edges.first().map(|e| e.2),
            }?;
            let dir = self
                .graph
                .read_ref(host, |g| g.ports.get(&from_port).map(|p| p.dir))
                .ok()
                .flatten()?;
            Some(match dir {
                PortDirection::In => PortDirection::Out,
                PortDirection::Out => PortDirection::In,
            })
        });

        let (geom, _) = self.canvas_derived(host, snapshot);
        let required_dir = required_dir;

        let best = self
            .graph
            .read_ref(host, |graph| {
                #[derive(Clone, Copy)]
                struct Rank {
                    angle: f32,
                    parallel: f32,
                    dist2: f32,
                    port: PortId,
                }

                let from = from_center;
                let mut best: Option<Rank> = None;

                for (&port, handle) in &geom.ports {
                    if port == from_port {
                        continue;
                    }

                    let Some(p) = graph.ports.get(&port) else {
                        continue;
                    };

                    if let Some(required_dir) = required_dir {
                        if p.dir != required_dir {
                            continue;
                        }
                    }

                    let dx = handle.center.x.0 - from.x;
                    let dy = handle.center.y.0 - from.y;
                    let (parallel, perp) = match dir {
                        PortNavDir::Left => (-dx, dy.abs()),
                        PortNavDir::Right => (dx, dy.abs()),
                        PortNavDir::Up => (-dy, dx.abs()),
                        PortNavDir::Down => (dy, dx.abs()),
                    };
                    if !parallel.is_finite() || !perp.is_finite() || parallel <= 1.0e-6 {
                        continue;
                    }

                    let angle = (perp / parallel).abs();
                    let dist2 = dx * dx + dy * dy;
                    if !angle.is_finite() || !dist2.is_finite() {
                        continue;
                    }

                    let cand = Rank {
                        angle,
                        parallel,
                        dist2,
                        port,
                    };

                    let better = match best {
                        None => true,
                        Some(best) => {
                            let by_angle = angle.total_cmp(&best.angle);
                            if by_angle != std::cmp::Ordering::Equal {
                                by_angle == std::cmp::Ordering::Less
                            } else {
                                let by_parallel = parallel.total_cmp(&best.parallel);
                                if by_parallel != std::cmp::Ordering::Equal {
                                    by_parallel == std::cmp::Ordering::Less
                                } else {
                                    let by_dist = dist2.total_cmp(&best.dist2);
                                    if by_dist != std::cmp::Ordering::Equal {
                                        by_dist == std::cmp::Ordering::Less
                                    } else {
                                        port < best.port
                                    }
                                }
                            }
                        }
                    };

                    if better {
                        best = Some(cand);
                    }
                }

                best.map(|r| r.port)
            })
            .ok()
            .flatten();

        let Some(next) = best else {
            return false;
        };

        let owner = self
            .graph
            .read_ref(host, |g| g.ports.get(&next).map(|p| p.node))
            .ok()
            .flatten();

        let Some(owner) = owner else {
            return false;
        };

        self.interaction.focused_node = Some(owner);
        self.interaction.focused_edge = None;
        self.interaction.focused_port = Some(next);
        self.refresh_focused_port_hints(host);
        self.update_view_state(host, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_nodes.clear();
            s.selected_nodes.push(owner);
            s.draw_order.retain(|id| *id != owner);
            s.draw_order.push(owner);
        });

        let snapshot = self.sync_view_state(host);
        if let Some(center) = self.port_center_canvas(host, &snapshot, next) {
            self.ensure_canvas_point_visible(host, &snapshot, center);
        }

        true
    }
}
