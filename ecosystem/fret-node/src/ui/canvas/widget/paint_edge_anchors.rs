use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_edge_focus_anchors<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        edge_anchor_target_id: Option<EdgeId>,
        edge_anchor_target: Option<(EdgeRouteKind, Point, Point, Color)>,
        zoom: f32,
    ) {
        let Some((route, from, to, color)) = edge_anchor_target else {
            return;
        };

        let (a0, a1) = Self::edge_focus_anchor_centers(route, from, to, zoom);
        let target_edge_id = edge_anchor_target_id;
        let (allow_from, allow_to) = target_edge_id
            .and_then(|edge_id| {
                self.graph
                    .read_ref(cx.app, |g| {
                        let edge = g.edges.get(&edge_id)?;
                        Some(Self::edge_reconnectable_flags(edge, &snapshot.interaction))
                    })
                    .ok()
                    .flatten()
            })
            .unwrap_or((false, false));

        let z = zoom.max(1.0e-6);
        let border_base = Px(Self::EDGE_FOCUS_ANCHOR_BORDER_SCREEN / z);
        let anchor_color = Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: 0.95,
        };
        let fill_color = Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: 0.15,
        };

        for (endpoint, center) in [(EdgeEndpoint::From, a0), (EdgeEndpoint::To, a1)] {
            if (endpoint == EdgeEndpoint::From && !allow_from)
                || (endpoint == EdgeEndpoint::To && !allow_to)
            {
                continue;
            }
            let rect = Self::edge_focus_anchor_rect(center, zoom);
            let r = Px(0.5 * rect.size.width.0);
            let hovered = self
                .interaction
                .hover_edge_anchor
                .is_some_and(|(edge, ep)| Some(edge) == target_edge_id && ep == endpoint);
            let active = self
                .interaction
                .wire_drag
                .as_ref()
                .is_some_and(|w| match &w.kind {
                    WireDragKind::Reconnect {
                        edge, endpoint: ep, ..
                    } => Some(*edge) == target_edge_id && *ep == endpoint,
                    _ => false,
                });

            let border = if active {
                Px((Self::EDGE_FOCUS_ANCHOR_BORDER_SCREEN + 1.0) / z)
            } else if hovered {
                Px((Self::EDGE_FOCUS_ANCHOR_BORDER_SCREEN + 0.5) / z)
            } else {
                border_base
            };

            let background = if active {
                Color {
                    a: (fill_color.a + 0.20).min(1.0),
                    ..fill_color
                }
            } else if hovered {
                Color {
                    a: (fill_color.a + 0.10).min(1.0),
                    ..fill_color
                }
            } else {
                fill_color
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(6),
                rect,
                background: fret_core::Paint::Solid(background),
                border: Edges::all(border),
                border_paint: fret_core::Paint::Solid(anchor_color),

                corner_radii: Corners::all(r),
            });
        }
    }
}
