fn compute_layout_map(
    graph: &DockGraph,
    root: DockNodeId,
    bounds: Rect,
) -> std::collections::HashMap<DockNodeId, Rect> {
    let mut layout = std::collections::HashMap::new();
    compute_layout_map_impl(graph, root, bounds, &mut layout);
    layout
}

fn compute_layout_map_impl(
    graph: &DockGraph,
    node: DockNodeId,
    bounds: Rect,
    out: &mut std::collections::HashMap<DockNodeId, Rect>,
) {
    let Some(n) = graph.node(node) else {
        return;
    };

    out.insert(node, bounds);
    match n {
        DockNode::Tabs { .. } => {}
        DockNode::Split {
            axis,
            children,
            fractions,
        } => {
            let count = children.len().min(fractions.len());
            if count == 0 {
                return;
            }

            let total: f32 = fractions.iter().take(count).sum();
            let total = if total <= 0.0 { 1.0 } else { total };

            let axis_len = match axis {
                fret_core::Axis::Horizontal => bounds.size.width.0,
                fret_core::Axis::Vertical => bounds.size.height.0,
            };
            if !axis_len.is_finite() || axis_len <= 0.0 {
                return;
            }

            let gaps = count.saturating_sub(1) as f32;
            let mut gap = DOCK_SPLIT_HANDLE_GAP.0;
            if gaps == 0.0 || axis_len <= gap * gaps {
                gap = 0.0;
            }

            let available = axis_len - gap * gaps;
            if !available.is_finite() || available <= 0.0 {
                return;
            }

            let mut cursor = 0.0;
            for i in 0..count {
                let f = (fractions[i] / total).max(0.0);
                let (child_axis_len, next_cursor) = if i + 1 == count {
                    let remaining = (available - cursor).max(0.0);
                    (remaining, available)
                } else {
                    let len = available * f;
                    (len, cursor + len)
                };

                let origin_axis = cursor + gap * (i as f32);
                let child_rect = match axis {
                    fret_core::Axis::Horizontal => Rect {
                        origin: Point::new(Px(bounds.origin.x.0 + origin_axis), bounds.origin.y),
                        size: Size::new(Px(child_axis_len), bounds.size.height),
                    },
                    fret_core::Axis::Vertical => Rect {
                        origin: Point::new(bounds.origin.x, Px(bounds.origin.y.0 + origin_axis)),
                        size: Size::new(bounds.size.width, Px(child_axis_len)),
                    },
                };

                cursor = next_cursor;
                compute_layout_map_impl(graph, children[i], child_rect, out);
            }
        }
    }
}

fn hidden_bounds(size: Size) -> Rect {
    Rect {
        origin: Point::new(Px(-1_000_000.0), Px(-1_000_000.0)),
        size,
    }
}

fn active_panel_content_bounds(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
) -> std::collections::HashMap<PanelKey, Rect> {
    let mut out: std::collections::HashMap<PanelKey, Rect> = std::collections::HashMap::new();

    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let (_tab_bar, content) = split_tab_bar(rect);
        if let Some(panel) = tabs.get(*active) {
            out.insert(panel.clone(), content);
        }
    }

    out
}

struct PaintDockParams<'a> {
    window: fret_core::AppWindowId,
    layout: &'a std::collections::HashMap<DockNodeId, Rect>,
    tab_titles: &'a HashMap<PanelKey, PreparedTabTitle>,
    hovered_tab: Option<(DockNodeId, usize)>,
    hovered_tab_close: bool,
    pressed_tab_close: Option<(DockNodeId, usize)>,
    tab_scroll: &'a HashMap<DockNodeId, Px>,
    tab_close_glyph: Option<PreparedTabTitle>,
}

fn paint_dock(
    theme: fret_ui::ThemeSnapshot,
    dock: &DockManager,
    params: PaintDockParams<'_>,
    overlay_hooks: Option<&dyn DockViewportOverlayHooks>,
    scene: &mut Scene,
) {
    let PaintDockParams {
        window,
        layout,
        tab_titles,
        hovered_tab,
        hovered_tab_close,
        pressed_tab_close,
        tab_scroll,
        tab_close_glyph,
    } = params;
    let graph = &dock.graph;
    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let (tab_bar, content) = split_tab_bar(rect);

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(0),
            rect,
            background: theme.colors.panel_background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(1),
            rect: tab_bar,
            background: theme.colors.surface_background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let scroll = tab_scroll_for_node(tab_scroll, node_id);
        scene.push(SceneOp::PushClipRect { rect: tab_bar });

        for (i, panel) in tabs.iter().enumerate() {
            let tab_rect = tab_rect_for_index(tab_bar, i, scroll);
            if tab_rect.origin.x.0 + tab_rect.size.width.0 < tab_bar.origin.x.0
                || tab_rect.origin.x.0 > tab_bar.origin.x.0 + tab_bar.size.width.0
            {
                continue;
            }

            let is_active = i == *active;
            let is_hovered = hovered_tab == Some((node_id, i));
            let bg = if is_active {
                theme.colors.panel_background
            } else if is_hovered {
                theme.colors.hover_background
            } else {
                Color {
                    a: 0.0,
                    ..theme.colors.panel_background
                }
            };

            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(2),
                rect: tab_rect,
                background: bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });

            if is_active {
                let underline_h = Px(2.0);
                let underline = Rect {
                    origin: Point::new(
                        tab_rect.origin.x,
                        Px(tab_rect.origin.y.0 + tab_rect.size.height.0 - underline_h.0),
                    ),
                    size: Size::new(tab_rect.size.width, underline_h),
                };
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(3),
                    rect: underline,
                    background: theme.colors.accent,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            if let Some(title) = tab_titles.get(panel) {
                let pad_x = theme.metrics.padding_md;
                let text_x = Px(tab_rect.origin.x.0 + pad_x.0);
                let inner_y = tab_rect.origin.y.0
                    + ((tab_rect.size.height.0 - title.metrics.size.height.0) * 0.5);
                let text_y = Px(inner_y + title.metrics.baseline.0);
                let text_color = if is_active || is_hovered {
                    theme.colors.text_primary
                } else {
                    theme.colors.text_muted
                };

                scene.push(SceneOp::PushClipRect { rect: tab_rect });
                scene.push(SceneOp::Text {
                    order: fret_core::DrawOrder(4),
                    origin: Point::new(text_x, text_y),
                    text: title.blob,
                    color: text_color,
                });
                scene.push(SceneOp::PopClip);
            }

            if (is_active || is_hovered) && tab_close_glyph.is_some() {
                let close_rect = tab_close_rect(theme, tab_rect);
                let close_hovered = is_hovered && hovered_tab_close;
                let close_pressed = pressed_tab_close == Some((node_id, i));

                if close_pressed || close_hovered {
                    scene.push(SceneOp::Quad {
                        order: fret_core::DrawOrder(5),
                        rect: close_rect,
                        background: theme.colors.hover_background,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(theme.metrics.radius_sm),
                    });
                }

                if let Some(glyph) = tab_close_glyph {
                    let text_x = Px(close_rect.origin.x.0
                        + (close_rect.size.width.0 - glyph.metrics.size.width.0) * 0.5);
                    let inner_y = close_rect.origin.y.0
                        + ((close_rect.size.height.0 - glyph.metrics.size.height.0) * 0.5);
                    let text_y = Px(inner_y + glyph.metrics.baseline.0);
                    let color = if close_pressed || close_hovered {
                        theme.colors.text_primary
                    } else {
                        theme.colors.text_muted
                    };
                    scene.push(SceneOp::Text {
                        order: fret_core::DrawOrder(6),
                        origin: Point::new(text_x, text_y),
                        text: glyph.blob,
                        color,
                    });
                }
            }
        }

        scene.push(SceneOp::PopClip);

        let active_panel = tabs.get(*active);
        if let Some(panel) = active_panel.and_then(|p| dock.panel(p)) {
            if let Some(vp) = panel.viewport {
                let mapping = ViewportMapping {
                    content_rect: content,
                    target_px_size: vp.target_px_size,
                    fit: vp.fit,
                };
                let draw_rect = mapping.map().draw_rect;

                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(3),
                    rect: content,
                    background: panel.color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(theme.metrics.radius_sm),
                });

                scene.push(SceneOp::PushClipRect { rect: content });
                scene.push(SceneOp::ViewportSurface {
                    order: fret_core::DrawOrder(4),
                    rect: draw_rect,
                    target: vp.target,
                    opacity: 1.0,
                });
                if let Some(hooks) = overlay_hooks
                    && let Some(panel_key) = active_panel
                {
                    hooks.paint(theme, window, panel_key, vp, mapping, draw_rect, scene);
                }
                scene.push(SceneOp::PopClip);
            } else {
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(3),
                    rect: content,
                    background: panel.color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(theme.metrics.radius_sm),
                });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ViewportHit {
    panel: PanelKey,
    viewport: ViewportPanel,
    content: Rect,
    draw_rect: Rect,
}

#[derive(Debug, Clone, PartialEq)]
struct ViewportCaptureState {
    hit: ViewportHit,
    button: fret_core::MouseButton,
    start: Point,
    moved: bool,
}

fn viewport_input_from_hit(
    window: fret_core::AppWindowId,
    hit: ViewportHit,
    position: Point,
    kind: ViewportInputKind,
) -> Option<ViewportInputEvent> {
    let mapping = ViewportMapping {
        content_rect: hit.content,
        target_px_size: hit.viewport.target_px_size,
        fit: hit.viewport.fit,
    };
    let uv = mapping.window_point_to_uv(position)?;
    let target_px = mapping.window_point_to_target_px(position)?;
    Some(ViewportInputEvent {
        window,
        target: hit.viewport.target,
        uv,
        target_px,
        kind,
    })
}

fn viewport_input_from_hit_clamped(
    window: fret_core::AppWindowId,
    hit: ViewportHit,
    position: Point,
    kind: ViewportInputKind,
) -> ViewportInputEvent {
    let mapping = ViewportMapping {
        content_rect: hit.content,
        target_px_size: hit.viewport.target_px_size,
        fit: hit.viewport.fit,
    };
    let uv = mapping.window_point_to_uv_clamped(position);
    let target_px = mapping.window_point_to_target_px_clamped(position);
    ViewportInputEvent {
        window,
        target: hit.viewport.target,
        uv,
        target_px,
        kind,
    }
}

fn hit_test_active_viewport_panel(
    graph: &DockGraph,
    panels: &HashMap<PanelKey, DockPanel>,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    position: Point,
) -> Option<ViewportHit> {
    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let Some(panel_key) = tabs.get(*active).cloned() else {
            continue;
        };
        let Some(panel) = panels.get(&panel_key) else {
            continue;
        };
        let Some(viewport) = panel.viewport else {
            continue;
        };

        let (_tab_bar, content) = split_tab_bar(rect);
        let mapping = ViewportMapping {
            content_rect: content,
            target_px_size: viewport.target_px_size,
            fit: viewport.fit,
        };
        let draw_rect = mapping.map().draw_rect;
        if draw_rect.contains(position) {
            return Some(ViewportHit {
                panel: panel_key,
                viewport,
                content,
                draw_rect,
            });
        }
    }
    None
}

fn split_tab_bar(rect: Rect) -> (Rect, Rect) {
    let tab_bar = Rect {
        origin: rect.origin,
        size: Size::new(rect.size.width, Px(DOCK_TAB_H.0.min(rect.size.height.0))),
    };
    let content = Rect {
        origin: Point::new(rect.origin.x, Px(rect.origin.y.0 + tab_bar.size.height.0)),
        size: Size::new(
            rect.size.width,
            Px((rect.size.height.0 - tab_bar.size.height.0).max(0.0)),
        ),
    };
    (tab_bar, content)
}

fn dock_drop_edge_thickness(rect: Rect) -> Px {
    let min_dim = rect.size.width.0.min(rect.size.height.0);
    // Keep split zones usable on large panels, but avoid making "center tab" drops difficult.
    // Also keep the thickness sane on small panels.
    // ImGui-style: edge splits should be easy to hit even on big panels; we still cap it so the
    // center/tab drop remains a first-class target.
    let base = (min_dim * 0.30).clamp(20.0, 120.0);
    let cap = (min_dim * 0.44).clamp(20.0, 120.0);
    Px(base.min(cap))
}

fn drop_zone_rect(rect: Rect, zone: DropZone) -> Rect {
    if zone == DropZone::Center {
        return rect;
    }
    let thickness = dock_drop_edge_thickness(rect).0;
    match zone {
        DropZone::Left => Rect {
            origin: rect.origin,
            size: Size::new(Px(thickness), rect.size.height),
        },
        DropZone::Right => Rect {
            origin: Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 - thickness),
                rect.origin.y,
            ),
            size: Size::new(Px(thickness), rect.size.height),
        },
        DropZone::Top => Rect {
            origin: rect.origin,
            size: Size::new(rect.size.width, Px(thickness)),
        },
        DropZone::Bottom => Rect {
            origin: Point::new(
                rect.origin.x,
                Px(rect.origin.y.0 + rect.size.height.0 - thickness),
            ),
            size: Size::new(rect.size.width, Px(thickness)),
        },
        DropZone::Center => rect,
    }
}

fn float_zone(bounds: Rect) -> Rect {
    let size = Px(34.0);
    Rect {
        origin: Point::new(Px(bounds.origin.x.0 + 8.0), Px(bounds.origin.y.0 + 8.0)),
        size: Size::new(size, size),
    }
}

fn tab_scroll_for_node(tab_scroll: &HashMap<DockNodeId, Px>, node: DockNodeId) -> Px {
    tab_scroll.get(&node).copied().unwrap_or(Px(0.0))
}

fn tab_rect_for_index(tab_bar: Rect, index: usize, scroll: Px) -> Rect {
    Rect {
        origin: Point::new(
            Px(tab_bar.origin.x.0 + DOCK_TAB_W.0 * index as f32 - scroll.0),
            tab_bar.origin.y,
        ),
        size: Size::new(DOCK_TAB_W, tab_bar.size.height),
    }
}

fn tab_close_rect(theme: fret_ui::ThemeSnapshot, tab_rect: Rect) -> Rect {
    let pad = theme.metrics.padding_sm.0.max(0.0);
    let x = tab_rect.origin.x.0 + tab_rect.size.width.0 - pad - DOCK_TAB_CLOSE_SIZE.0;
    let y = tab_rect.origin.y.0 + (tab_rect.size.height.0 - DOCK_TAB_CLOSE_SIZE.0) * 0.5;
    Rect::new(
        Point::new(Px(x), Px(y)),
        Size::new(DOCK_TAB_CLOSE_SIZE, DOCK_TAB_CLOSE_SIZE),
    )
}

fn hit_test_tab(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    theme: fret_ui::ThemeSnapshot,
    position: Point,
) -> Option<(DockNodeId, usize, PanelKey, bool)> {
    for (&node, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let (tab_bar, _content) = split_tab_bar(rect);
        if !tab_bar.contains(position) {
            continue;
        }
        let scroll = tab_scroll_for_node(tab_scroll, node);
        let rel_x = position.x.0 - tab_bar.origin.x.0 + scroll.0;
        let idx = (rel_x / DOCK_TAB_W.0).floor() as isize;
        if idx < 0 {
            continue;
        }
        let idx = idx as usize;
        let panel = tabs.get(idx)?.clone();
        let tab_rect = tab_rect_for_index(tab_bar, idx, scroll);
        let close = tab_close_rect(theme, tab_rect).contains(position);
        return Some((node, idx, panel, close));
    }
    None
}

fn hit_test_drop_target(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    position: Point,
) -> Option<HoverTarget> {
    for (&node, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
            continue;
        };
        if !rect.contains(position) {
            continue;
        }

        let (tab_bar, _content) = split_tab_bar(rect);
        if tab_bar.contains(position) {
            let scroll = tab_scroll_for_node(tab_scroll, node);
            let insert_index = compute_tab_insert_index(tab_bar, scroll, tabs.len(), position);
            return Some(HoverTarget {
                tabs: node,
                zone: DropZone::Center,
                insert_index: Some(insert_index),
            });
        }

        // ImGui-style direction-pad hit targets near the center of the hovered dock node.
        // This makes split docking discoverable and avoids requiring the cursor to be near edges.
        for (zone, hint_rect) in dock_hint_rects(rect) {
            if hint_rect.contains(position) {
                return Some(HoverTarget {
                    tabs: node,
                    zone,
                    insert_index: None,
                });
            }
        }

        let thickness = dock_drop_edge_thickness(rect).0;
        let left = position.x.0 - rect.origin.x.0;
        let right = rect.origin.x.0 + rect.size.width.0 - position.x.0;
        let top = position.y.0 - rect.origin.y.0;
        let bottom = rect.origin.y.0 + rect.size.height.0 - position.y.0;

        let mut zone = DropZone::Center;
        let mut best = thickness;
        for (candidate, dist) in [
            (DropZone::Left, left),
            (DropZone::Right, right),
            (DropZone::Top, top),
            (DropZone::Bottom, bottom),
        ] {
            if dist < best {
                best = dist;
                zone = candidate;
            }
        }

        return Some(HoverTarget {
            tabs: node,
            zone,
            insert_index: None,
        });
    }
    None
}

fn compute_tab_insert_index(tab_bar: Rect, scroll: Px, tab_count: usize, position: Point) -> usize {
    let rel_x = position.x.0 - tab_bar.origin.x.0 + scroll.0;
    let raw = (rel_x / DOCK_TAB_W.0) + 0.5;
    let idx = raw.floor() as isize;
    idx.clamp(0, tab_count as isize) as usize
}

fn split_children_two(graph: &DockGraph, split: DockNodeId) -> Option<(DockNodeId, DockNodeId)> {
    let Some(DockNode::Split { children, .. }) = graph.node(split) else {
        return None;
    };
    if children.len() != 2 {
        return None;
    }
    Some((children[0], children[1]))
}

fn hit_test_split_handle(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    position: Point,
) -> Option<DividerDragState> {
    for (&node, &bounds) in layout.iter() {
        let Some(DockNode::Split {
            axis,
            children,
            fractions,
        }) = graph.node(node)
        else {
            continue;
        };
        if children.len() != 2 {
            continue;
        }
        if !bounds.contains(position) {
            continue;
        }

        let Some(left) = layout.get(&children[0]).copied() else {
            continue;
        };
        let Some(right) = layout.get(&children[1]).copied() else {
            continue;
        };

        let handle = split_handle_rect(*axis, bounds, left, right, DOCK_SPLIT_HANDLE_HIT_THICKNESS);

        if handle.contains(position) {
            let total = fractions.iter().take(2).sum::<f32>();
            let total = if total <= 0.0 { 1.0 } else { total };
            let f0 = fractions.first().copied().unwrap_or(0.5) / total;
            let center = split_handle_center(*axis, left, right);
            let grab_offset = match axis {
                fret_core::Axis::Horizontal => position.x.0 - center,
                fret_core::Axis::Vertical => position.y.0 - center,
            };
            return Some(DividerDragState {
                split: node,
                axis: *axis,
                bounds,
                fraction: f0,
                grab_offset,
            });
        }
    }

    None
}

fn split_gap(axis: fret_core::Axis, first: Rect, second: Rect) -> f32 {
    let gap = match axis {
        fret_core::Axis::Horizontal => second.origin.x.0 - (first.origin.x.0 + first.size.width.0),
        fret_core::Axis::Vertical => second.origin.y.0 - (first.origin.y.0 + first.size.height.0),
    };
    if gap.is_finite() { gap.max(0.0) } else { 0.0 }
}

fn split_handle_center(axis: fret_core::Axis, first: Rect, second: Rect) -> f32 {
    let gap = split_gap(axis, first, second);
    match axis {
        fret_core::Axis::Horizontal => {
            let start = first.origin.x.0 + first.size.width.0;
            if gap > 0.0 { start + gap * 0.5 } else { start }
        }
        fret_core::Axis::Vertical => {
            let start = first.origin.y.0 + first.size.height.0;
            if gap > 0.0 { start + gap * 0.5 } else { start }
        }
    }
}

fn split_handle_rect(
    axis: fret_core::Axis,
    bounds: Rect,
    first: Rect,
    second: Rect,
    thickness: Px,
) -> Rect {
    let gap = split_gap(axis, first, second);
    if gap > 0.0 {
        match axis {
            fret_core::Axis::Horizontal => Rect {
                origin: Point::new(Px(first.origin.x.0 + first.size.width.0), bounds.origin.y),
                size: Size::new(Px(gap), bounds.size.height),
            },
            fret_core::Axis::Vertical => Rect {
                origin: Point::new(bounds.origin.x, Px(first.origin.y.0 + first.size.height.0)),
                size: Size::new(bounds.size.width, Px(gap)),
            },
        }
    } else {
        let center = split_handle_center(axis, first, second);
        match axis {
            fret_core::Axis::Horizontal => Rect {
                origin: Point::new(Px(center - thickness.0 * 0.5), bounds.origin.y),
                size: Size::new(thickness, bounds.size.height),
            },
            fret_core::Axis::Vertical => Rect {
                origin: Point::new(bounds.origin.x, Px(center - thickness.0 * 0.5)),
                size: Size::new(bounds.size.width, thickness),
            },
        }
    }
}

fn compute_split_fraction(
    axis: fret_core::Axis,
    bounds: Rect,
    first: Rect,
    second: Rect,
    grab_offset: f32,
    position: Point,
) -> Option<f32> {
    let min_px = 120.0;
    match axis {
        fret_core::Axis::Horizontal => {
            let w = bounds.size.width.0;
            if !w.is_finite() {
                return None;
            }
            let gap = split_gap(axis, first, second);
            let avail = w - gap;
            if !avail.is_finite() || avail <= min_px * 2.0 {
                return None;
            }
            let max_x = (avail - min_px).max(min_px);
            let anchor = position.x.0 - grab_offset - bounds.origin.x.0;
            let x = (anchor - gap * 0.5).clamp(min_px, max_x);
            Some(x / avail)
        }
        fret_core::Axis::Vertical => {
            let h = bounds.size.height.0;
            if !h.is_finite() {
                return None;
            }
            let gap = split_gap(axis, first, second);
            let avail = h - gap;
            if !avail.is_finite() || avail <= min_px * 2.0 {
                return None;
            }
            let max_y = (avail - min_px).max(min_px);
            let anchor = position.y.0 - grab_offset - bounds.origin.y.0;
            let y = (anchor - gap * 0.5).clamp(min_px, max_y);
            Some(y / avail)
        }
    }
}

fn paint_split_handles(
    theme: fret_ui::ThemeSnapshot,
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    active: Option<DockNodeId>,
    scale_factor: f32,
    scene: &mut Scene,
) {
    for (&node, &bounds) in layout.iter() {
        let Some(DockNode::Split { axis, children, .. }) = graph.node(node) else {
            continue;
        };
        if children.len() != 2 {
            continue;
        }
        let Some(first) = layout.get(&children[0]).copied() else {
            continue;
        };
        let Some(second) = layout.get(&children[1]).copied() else {
            continue;
        };

        let center = split_handle_center(*axis, first, second);

        let background = if active == Some(node) {
            theme.colors.focus_ring
        } else {
            theme.colors.panel_border
        };

        ResizeHandle {
            axis: *axis,
            hit_thickness: DOCK_SPLIT_HANDLE_HIT_THICKNESS,
            paint_device_px: 1.0,
        }
        .paint(
            scene,
            // Keep split handle under component focus rings (typically DrawOrder(1)),
            // while still painting above panel backgrounds (DrawOrder(0)).
            fret_core::DrawOrder(0),
            bounds,
            center,
            scale_factor,
            background,
        );
    }
}

fn paint_drop_overlay(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    window: fret_core::AppWindowId,
    bounds: Rect,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    scene: &mut Scene,
) {
    let Some(target) = target else {
        return;
    };

    match target {
        DockDropTarget::Float { window: w } => {
            if w != window {
                return;
            }
            let zone = bounds;
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: zone,
                background: Color {
                    a: 0.10,
                    ..theme.colors.accent
                },
                border: Edges::all(Px(3.0)),
                border_color: Color {
                    a: 0.85,
                    ..theme.colors.accent
                },
                corner_radii: fret_core::Corners::all(Px(theme.metrics.radius_md.0.max(6.0))),
            });
        }
        DockDropTarget::Dock(target) => {
            let Some(rect) = layout.get(&target.tabs).copied() else {
                return;
            };

            if target.zone == DropZone::Center {
                let (tab_bar, _content) = split_tab_bar(rect);
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(9_990),
                    rect: tab_bar,
                    background: Color {
                        a: 0.14,
                        ..theme.colors.accent
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Color {
                        a: 0.45,
                        ..theme.colors.accent
                    },
                    corner_radii: fret_core::Corners::all(Px(theme.metrics.radius_sm.0.max(4.0))),
                });
                if let Some(i) = target.insert_index {
                    let scroll = tab_scroll_for_node(tab_scroll, target.tabs);
                    let x = tab_bar.origin.x.0 + DOCK_TAB_W.0 * i as f32 - scroll.0;
                    let marker = Rect::new(
                        Point::new(Px(x - 3.0), Px(tab_bar.origin.y.0 + 3.0)),
                        Size::new(Px(6.0), Px((tab_bar.size.height.0 - 6.0).max(0.0))),
                    );
                    scene.push(SceneOp::Quad {
                        order: fret_core::DrawOrder(10_000),
                        rect: marker,
                        background: Color {
                            a: 0.85,
                            ..theme.colors.accent
                        },
                        border: Edges::all(Px(1.0)),
                        border_color: Color {
                            a: 1.0,
                            ..theme.colors.accent
                        },
                        corner_radii: fret_core::Corners::all(Px(3.0)),
                    });

                    let cap_w = Px(14.0);
                    let cap_h = Px(3.0);
                    let cap_x = Px(x - cap_w.0 * 0.5);
                    let cap_top =
                        Rect::new(Point::new(cap_x, marker.origin.y), Size::new(cap_w, cap_h));
                    let cap_bottom = Rect::new(
                        Point::new(
                            cap_x,
                            Px(marker.origin.y.0 + marker.size.height.0 - cap_h.0),
                        ),
                        Size::new(cap_w, cap_h),
                    );
                    for cap in [cap_top, cap_bottom] {
                        scene.push(SceneOp::Quad {
                            order: fret_core::DrawOrder(10_001),
                            rect: cap,
                            background: Color {
                                a: 0.92,
                                ..theme.colors.accent
                            },
                            border: Edges::all(Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: fret_core::Corners::all(Px(2.0)),
                        });
                    }
                }
                return;
            }

            let overlay = drop_zone_rect(rect, target.zone);
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: overlay,
                background: Color {
                    a: 0.16,
                    ..theme.colors.accent
                },
                border: Edges::all(Px(2.0)),
                border_color: Color {
                    a: 0.85,
                    ..theme.colors.accent
                },
                corner_radii: fret_core::Corners::all(Px(theme.metrics.radius_sm.0.max(4.0))),
            });
        }
    }
}

fn paint_drop_hints(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    _window: fret_core::AppWindowId,
    _bounds: Rect,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    scene: &mut Scene,
) {
    let Some(target) = target else {
        return;
    };

    let DockDropTarget::Dock(target) = target else {
        return;
    };

    let Some(rect) = layout.get(&target.tabs).copied() else {
        return;
    };

    let hint_rects = dock_hint_rects(rect);

    let inactive_bg = Color {
        a: 0.64,
        ..theme.colors.panel_background
    };
    let inactive_border = Color {
        a: 0.95,
        ..theme.colors.panel_border
    };
    let active_bg = Color {
        a: 0.92,
        ..theme.colors.accent
    };
    let active_border = Color {
        a: 1.0,
        ..theme.colors.accent
    };

    let order = fret_core::DrawOrder(9_500);
    let border = Edges::all(Px(2.0));
    let corner_radii = fret_core::Corners::all(Px(theme.metrics.radius_sm.0.max(4.0)));

    // Draw a plate behind the 5-way pad, closer to ImGui/Godot affordances.
    let pad = Px(theme.metrics.padding_sm.0.max(6.0));
    let mut min_x: f32 = f32::INFINITY;
    let mut min_y: f32 = f32::INFINITY;
    let mut max_x: f32 = f32::NEG_INFINITY;
    let mut max_y: f32 = f32::NEG_INFINITY;
    for &(_zone, r) in hint_rects.iter() {
        min_x = min_x.min(r.origin.x.0);
        min_y = min_y.min(r.origin.y.0);
        max_x = max_x.max(r.origin.x.0 + r.size.width.0);
        max_y = max_y.max(r.origin.y.0 + r.size.height.0);
    }
    if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
        let plate = Rect::new(
            Point::new(Px(min_x - pad.0), Px(min_y - pad.0)),
            Size::new(
                Px((max_x - min_x + pad.0 * 2.0).max(0.0)),
                Px((max_y - min_y + pad.0 * 2.0).max(0.0)),
            ),
        );
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(order.0 - 2),
            rect: plate,
            background: Color {
                a: 0.70,
                ..theme.colors.surface_background
            },
            border: Edges::all(Px(1.0)),
            border_color: Color {
                a: 0.70,
                ..theme.colors.panel_border
            },
            corner_radii: fret_core::Corners::all(Px(theme.metrics.radius_md.0.max(6.0))),
        });
    }

    for &(zone, hint_rect) in hint_rects.iter() {
        let is_active = zone == target.zone;
        let bg = if is_active { active_bg } else { inactive_bg };
        let stroke = if is_active {
            active_border
        } else {
            inactive_border
        };

        scene.push(SceneOp::Quad {
            order,
            rect: hint_rect,
            background: bg,
            border,
            border_color: stroke,
            corner_radii,
        });
        paint_drop_hint_icon(theme, zone, hint_rect, is_active, scene, order.0 + 1);
    }
}

fn paint_drop_hint_icon(
    theme: fret_ui::ThemeSnapshot,
    zone: DropZone,
    hint_rect: Rect,
    is_active: bool,
    scene: &mut Scene,
    order: u32,
) {
    fn inset(rect: Rect, inset: Px) -> Rect {
        let w = (rect.size.width.0 - inset.0 * 2.0).max(0.0);
        let h = (rect.size.height.0 - inset.0 * 2.0).max(0.0);
        Rect::new(
            Point::new(Px(rect.origin.x.0 + inset.0), Px(rect.origin.y.0 + inset.0)),
            Size::new(Px(w), Px(h)),
        )
    }

    let min_dim = hint_rect.size.width.0.min(hint_rect.size.height.0);
    let pad = Px((min_dim * 0.18).clamp(6.0, 10.0));
    let frame = inset(hint_rect, pad);
    let inner = inset(frame, Px((min_dim * 0.08).clamp(2.0, 4.0)));

    let stroke = Color {
        a: if is_active { 0.92 } else { 0.80 },
        ..theme.colors.text_primary
    };
    let base = Color {
        a: if is_active { 0.16 } else { 0.12 },
        ..theme.colors.text_primary
    };
    let fill = Color {
        a: if is_active { 0.90 } else { 0.72 },
        ..theme.colors.text_primary
    };

    let frame_radius = Px(theme.metrics.radius_sm.0.clamp(2.0, 4.0));
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(order),
        rect: frame,
        background: Color::TRANSPARENT,
        border: Edges::all(Px(2.0)),
        border_color: stroke,
        corner_radii: fret_core::Corners::all(frame_radius),
    });

    // Base fill so the highlighted region reads as "target placement" (ImGui-like).
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(order + 1),
        rect: inner,
        background: base,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let split_ratio = 0.42_f32;
    let tab_ratio = 0.24_f32;
    let line_thickness = Px((min_dim * 0.04).clamp(1.5, 2.5));

    match zone {
        DropZone::Center => {
            let tab_h = Px((inner.size.height.0 * tab_ratio).max(0.0));
            let tab = Rect::new(inner.origin, Size::new(inner.size.width, tab_h));
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: tab,
                background: fill,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
        DropZone::Left | DropZone::Right => {
            let w = Px((inner.size.width.0 * split_ratio).max(0.0));
            let (highlight, line_x) = if zone == DropZone::Left {
                (
                    Rect::new(inner.origin, Size::new(w, inner.size.height)),
                    Px(inner.origin.x.0 + w.0),
                )
            } else {
                (
                    Rect::new(
                        Point::new(Px(inner.origin.x.0 + inner.size.width.0 - w.0), inner.origin.y),
                        Size::new(w, inner.size.height),
                    ),
                    Px(inner.origin.x.0 + inner.size.width.0 - w.0),
                )
            };
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: highlight,
                background: fill,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
            let line = Rect::new(
                Point::new(Px(line_x.0 - line_thickness.0 * 0.5), inner.origin.y),
                Size::new(line_thickness, inner.size.height),
            );
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 3),
                rect: line,
                background: stroke,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
        DropZone::Top | DropZone::Bottom => {
            let h = Px((inner.size.height.0 * split_ratio).max(0.0));
            let (highlight, line_y) = if zone == DropZone::Top {
                (
                    Rect::new(inner.origin, Size::new(inner.size.width, h)),
                    Px(inner.origin.y.0 + h.0),
                )
            } else {
                (
                    Rect::new(
                        Point::new(inner.origin.x, Px(inner.origin.y.0 + inner.size.height.0 - h.0)),
                        Size::new(inner.size.width, h),
                    ),
                    Px(inner.origin.y.0 + inner.size.height.0 - h.0),
                )
            };
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: highlight,
                background: fill,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
            let line = Rect::new(
                Point::new(inner.origin.x, Px(line_y.0 - line_thickness.0 * 0.5)),
                Size::new(inner.size.width, line_thickness),
            );
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 3),
                rect: line,
                background: stroke,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
    }
}

fn dock_hint_rects(rect: Rect) -> [(DropZone, Rect); 5] {
    // Match the mental model of ImGui docking: an explicit 5-way “direction pad” near the
    // center of the hovered dock node. Hit-testing uses the same rects.
    let cx = rect.origin.x.0 + rect.size.width.0 * 0.5;
    let cy = rect.origin.y.0 + rect.size.height.0 * 0.5;

    let min_dim = rect.size.width.0.min(rect.size.height.0);
    // Scale targets up on larger panels to make split docking feel effortless (Unity/ImGui-like),
    // while keeping it usable on small panels.
    let size = Px((min_dim * 0.095).clamp(34.0, 56.0));
    let gap = Px((size.0 * 0.35).clamp(10.0, 16.0));
    let step = Px(size.0 + gap.0);

    let mk = |dx: f32, dy: f32| -> Rect {
        Rect::new(
            Point::new(Px(cx + dx - size.0 * 0.5), Px(cy + dy - size.0 * 0.5)),
            Size::new(size, size),
        )
    };

    [
        (DropZone::Center, mk(0.0, 0.0)),
        (DropZone::Left, mk(-step.0, 0.0)),
        (DropZone::Right, mk(step.0, 0.0)),
        (DropZone::Top, mk(0.0, -step.0)),
        (DropZone::Bottom, mk(0.0, step.0)),
    ]
}

fn dock_space_regions(bounds: Rect) -> (Rect, Rect) {
    let chrome_h = Px(0.0);
    let chrome = Rect {
        origin: bounds.origin,
        size: Size::new(bounds.size.width, Px(chrome_h.0.min(bounds.size.height.0))),
    };
    let dock = Rect {
        origin: Point::new(
            bounds.origin.x,
            Px(bounds.origin.y.0 + chrome.size.height.0),
        ),
        size: Size::new(
            bounds.size.width,
            Px((bounds.size.height.0 - chrome.size.height.0).max(0.0)),
        ),
    };
    (chrome, dock)
}
