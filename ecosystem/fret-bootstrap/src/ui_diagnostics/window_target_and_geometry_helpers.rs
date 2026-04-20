fn resolve_window_target_from_known_windows(
    current_window: AppWindowId,
    known_windows: &[AppWindowId],
    target: UiWindowTargetV1,
) -> Option<AppWindowId> {
    let first_seen = known_windows
        .iter()
        .copied()
        .min_by_key(|w| w.data().as_ffi());
    let last_seen = known_windows
        .iter()
        .copied()
        .max_by_key(|w| w.data().as_ffi());
    match target {
        UiWindowTargetV1::Current => Some(current_window),
        UiWindowTargetV1::FirstSeen => first_seen,
        UiWindowTargetV1::FirstSeenOther => known_windows
            .iter()
            .copied()
            .filter(|w| *w != current_window)
            .min_by_key(|w| w.data().as_ffi()),
        UiWindowTargetV1::LastSeen => last_seen,
        UiWindowTargetV1::LastSeenOther => known_windows
            .iter()
            .copied()
            .filter(|w| *w != current_window)
            .max_by_key(|w| w.data().as_ffi()),
        UiWindowTargetV1::WindowFfi { window } => {
            let want = AppWindowId::from(KeyData::from_ffi(window));
            known_windows.contains(&want).then_some(want)
        }
    }
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = ax0 + a.size.width.0.max(0.0);
    let ay1 = ay0 + a.size.height.0.max(0.0);

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = bx0 + b.size.width.0.max(0.0);
    let by1 = by0 + b.size.height.0.max(0.0);

    ax1 > bx0 && bx1 > ax0 && ay1 > by0 && by1 > ay0
}

fn center_of_rect(rect: Rect) -> Point {
    let x = rect.origin.x + rect.size.width * 0.5;
    let y = rect.origin.y + rect.size.height * 0.5;
    Point::new(x, y)
}

fn interaction_bounds_for_ui_node(
    element_runtime: Option<&ElementRuntime>,
    ui: &UiTree<App>,
    window: AppWindowId,
    node: fret_core::NodeId,
) -> Option<Rect> {
    let element_visual_bounds = |node| {
        ui.debug_node_element(node).and_then(|element| {
            element_runtime.and_then(|runtime| runtime.visual_bounds_for_element(window, element))
        })
    };
    let element_layout_bounds = |node| {
        ui.debug_node_element(node).and_then(|element| {
            element_runtime.and_then(|runtime| runtime.layout_bounds_for_element(window, element))
        })
    };

    let descendant_visual_bounds = || {
        let mut stack = ui.debug_node_children(node);
        while let Some(current) = stack.pop() {
            if let Some(bounds) =
                element_visual_bounds(current).or_else(|| ui.debug_node_visual_bounds(current))
            {
                return Some(bounds);
            }
            stack.extend(ui.debug_node_children(current));
        }
        None
    };
    let descendant_layout_bounds = || {
        let mut stack = ui.debug_node_children(node);
        while let Some(current) = stack.pop() {
            if let Some(bounds) =
                element_layout_bounds(current).or_else(|| ui.debug_node_bounds(current))
            {
                return Some(bounds);
            }
            stack.extend(ui.debug_node_children(current));
        }
        None
    };
    let ancestor_visual_bounds = || {
        ui.debug_node_path(node)
            .into_iter()
            .rev()
            .find_map(|current| {
                element_visual_bounds(current).or_else(|| ui.debug_node_visual_bounds(current))
            })
    };
    let ancestor_layout_bounds = || {
        ui.debug_node_path(node)
            .into_iter()
            .rev()
            .find_map(|current| {
                element_layout_bounds(current).or_else(|| ui.debug_node_bounds(current))
            })
    };

    element_visual_bounds(node)
        .or_else(|| ui.debug_node_visual_bounds(node))
        .or_else(descendant_visual_bounds)
        .or_else(ancestor_visual_bounds)
        .or_else(|| element_layout_bounds(node))
        .or_else(descendant_layout_bounds)
        .or_else(ancestor_layout_bounds)
        .or_else(|| ui.debug_node_bounds(node))
}

fn interaction_bounds_for_semantics_node(
    element_runtime: Option<&ElementRuntime>,
    ui: Option<&UiTree<App>>,
    window: AppWindowId,
    node: &fret_core::SemanticsNode,
) -> Rect {
    if let Some(ui) = ui
        && let Some(bounds) = interaction_bounds_for_ui_node(element_runtime, ui, window, node.id)
    {
        return bounds;
    }

    element_runtime
        .and_then(|runtime| runtime.interaction_bounds_for_node(window, node.id))
        .unwrap_or(node.bounds)
}

fn rect_area(rect: Rect) -> f32 {
    rect.size.width.0.max(0.0) * rect.size.height.0.max(0.0)
}

#[derive(Debug, Clone, Copy)]
struct ResolvedInteractionBounds {
    bounds: Rect,
    source: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct PointerTargetResolution {
    position: Point,
    bounds: Rect,
    bounds_source: &'static str,
    used_window_scan: bool,
    clamped_outside_window: bool,
}

fn live_test_id_instance_priority(kind: &str) -> u8 {
    match kind {
        "Pressable" => 0,
        "TextInput" | "TextArea" => 1,
        "PointerRegion" | "Scroll" | "Scrollbar" => 2,
        _ => 10,
    }
}

fn interaction_bounds_for_live_test_id(
    app: &App,
    element_runtime: Option<&ElementRuntime>,
    ui: Option<&UiTree<App>>,
    window: AppWindowId,
    test_id: &str,
) -> Option<ResolvedInteractionBounds> {
    let mut best: Option<(u8, u8, f32, u64, ResolvedInteractionBounds)> = None;
    for candidate in fret_ui::declarative::live_test_id_matches_for_window(app, window, test_id) {
        let resolved = element_runtime
            .and_then(|runtime| runtime.visual_bounds_for_element(window, candidate.element))
            .map(|bounds| {
                (
                    0u8,
                    ResolvedInteractionBounds {
                        bounds,
                        source: "live.element_visual",
                    },
                )
            })
            .or_else(|| {
                ui.and_then(|ui| {
                    interaction_bounds_for_ui_node(element_runtime, ui, window, candidate.node).map(
                        |bounds| {
                            (
                                1u8,
                                ResolvedInteractionBounds {
                                    bounds,
                                    source: "live.ui_node_interaction",
                                },
                            )
                        },
                    )
                })
            })
            .or_else(|| {
                element_runtime
                    .and_then(|runtime| {
                        runtime.layout_bounds_for_element(window, candidate.element)
                    })
                    .map(|bounds| {
                        (
                            2u8,
                            ResolvedInteractionBounds {
                                bounds,
                                source: "live.element_layout",
                            },
                        )
                    })
            })
            .or_else(|| {
                ui.and_then(|ui| {
                    ui.debug_node_bounds(candidate.node).map(|bounds| {
                        (
                            3u8,
                            ResolvedInteractionBounds {
                                bounds,
                                source: "live.node_layout",
                            },
                        )
                    })
                })
            });
        let Some((source_priority, resolved_bounds)) = resolved else {
            continue;
        };

        let area = rect_area(resolved_bounds.bounds);
        if !area.is_finite() || area <= 0.0 {
            continue;
        }

        let kind_priority = live_test_id_instance_priority(candidate.instance_kind);
        let node_id = candidate.node.data().as_ffi();
        let better = match best {
            None => true,
            Some((best_source, best_kind, best_area, best_node, _)) => {
                source_priority < best_source
                    || (source_priority == best_source && kind_priority < best_kind)
                    || (source_priority == best_source
                        && kind_priority == best_kind
                        && (area < best_area || (area == best_area && node_id < best_node)))
            }
        };
        if better {
            best = Some((
                source_priority,
                kind_priority,
                area,
                node_id,
                resolved_bounds,
            ));
        }
    }

    best.map(|(_, _, _, _, resolved_bounds)| resolved_bounds)
}

fn center_of_rect_clamped_to_rect(rect: Rect, clamp: Rect) -> Point {
    let cx0 = clamp.origin.x.0;
    let cy0 = clamp.origin.y.0;
    let cx1 = cx0 + clamp.size.width.0.max(0.0);
    let cy1 = cy0 + clamp.size.height.0.max(0.0);
    let center = center_of_rect(rect);
    if cx1 <= cx0 || cy1 <= cy0 {
        return center;
    }

    Point::new(
        fret_core::Px(center.x.0.clamp(cx0, cx1)),
        fret_core::Px(center.y.0.clamp(cy0, cy1)),
    )
}

fn semantics_hit_matches_intended(
    index: &SemanticsIndex<'_>,
    hit: &fret_core::SemanticsNode,
    intended: &fret_core::SemanticsNode,
) -> bool {
    if let (Some(hit_test_id), Some(intended_test_id)) =
        (hit.test_id.as_deref(), intended.test_id.as_deref())
        && hit_test_id == intended_test_id
    {
        return true;
    }

    let intended_id = intended.id.data().as_ffi();
    let hit_id = hit.id.data().as_ffi();
    let controls_intended = hit
        .controls
        .iter()
        .any(|id| id.data().as_ffi() == intended_id);
    controls_intended || index.is_descendant_of_or_self(hit_id, intended_id)
}

fn scan_window_for_intended_hit(
    snapshot: &fret_core::SemanticsSnapshot,
    ui: &UiTree<App>,
    intended: &fret_core::SemanticsNode,
    window_bounds: Rect,
    probe_x: f32,
    probe_y: f32,
) -> Option<Point> {
    let index = SemanticsIndex::new(snapshot);
    let wx0 = window_bounds.origin.x.0;
    let wy0 = window_bounds.origin.y.0;
    let wx1 = wx0 + window_bounds.size.width.0.max(0.0);
    let wy1 = wy0 + window_bounds.size.height.0.max(0.0);
    if wx1 <= wx0 || wy1 <= wy0 {
        return None;
    }

    let step = 24.0f32;
    let inset = 8.0f32;
    let x_offsets = [0.0f32, -32.0, 32.0, -64.0, 64.0];
    let y_offsets = [0.0f32, -32.0, 32.0, -64.0, 64.0];

    let mut y = wy0 + inset;
    while y <= wy1 - inset {
        for dx in x_offsets {
            let pos = Point::new(
                fret_core::Px((probe_x + dx).clamp(wx0, wx1)),
                fret_core::Px(y),
            );
            if let Some(hit) = pick_semantics_node_at(snapshot, ui, pos)
                && semantics_hit_matches_intended(&index, hit, intended)
            {
                return Some(pos);
            }
        }
        y += step;
    }

    let mut x = wx0 + inset;
    while x <= wx1 - inset {
        for dy in y_offsets {
            let pos = Point::new(
                fret_core::Px(x),
                fret_core::Px((probe_y + dy).clamp(wy0, wy1)),
            );
            if let Some(hit) = pick_semantics_node_at(snapshot, ui, pos)
                && semantics_hit_matches_intended(&index, hit, intended)
            {
                return Some(pos);
            }
        }
        x += step;
    }

    None
}

fn pointer_target_resolution_prefer_intended_hit(
    app: &App,
    snapshot: &fret_core::SemanticsSnapshot,
    element_runtime: Option<&ElementRuntime>,
    ui: &UiTree<App>,
    window: AppWindowId,
    intended: &fret_core::SemanticsNode,
    window_bounds: Rect,
) -> PointerTargetResolution {
    let resolved_bounds = intended
        .test_id
        .as_deref()
        .and_then(|test_id| {
            interaction_bounds_for_live_test_id(app, element_runtime, Some(ui), window, test_id)
        })
        .unwrap_or_else(|| ResolvedInteractionBounds {
            bounds: interaction_bounds_for_semantics_node(
                element_runtime,
                Some(ui),
                window,
                intended,
            ),
            source: "semantics.node_fallback",
        });
    let intended_bounds = resolved_bounds.bounds;
    let intended_center = center_of_rect_clamped_to_rect(intended_bounds, window_bounds);
    let rx0 = intended_bounds.origin.x.0;
    let ry0 = intended_bounds.origin.y.0;
    let rx1 = rx0 + intended_bounds.size.width.0.max(0.0);
    let ry1 = ry0 + intended_bounds.size.height.0.max(0.0);

    let wx0 = window_bounds.origin.x.0;
    let wy0 = window_bounds.origin.y.0;
    let wx1 = wx0 + window_bounds.size.width.0.max(0.0);
    let wy1 = wy0 + window_bounds.size.height.0.max(0.0);

    let ix0 = rx0.max(wx0);
    let iy0 = ry0.max(wy0);
    let ix1 = rx1.min(wx1);
    let iy1 = ry1.min(wy1);

    if ix1 <= ix0 || iy1 <= iy0 {
        if let Some(pos) = scan_window_for_intended_hit(
            snapshot,
            ui,
            intended,
            window_bounds,
            intended_center.x.0,
            intended_center.y.0,
        ) {
            return PointerTargetResolution {
                position: pos,
                bounds: intended_bounds,
                bounds_source: resolved_bounds.source,
                used_window_scan: true,
                clamped_outside_window: true,
            };
        }
        return PointerTargetResolution {
            position: center_of_rect_clamped_to_rect(intended_bounds, window_bounds),
            bounds: intended_bounds,
            bounds_source: resolved_bounds.source,
            used_window_scan: false,
            clamped_outside_window: true,
        };
    }

    let w = (ix1 - ix0).max(0.0);
    let h = (iy1 - iy0).max(0.0);
    let pad_x = 8.0f32.min(w * 0.5);
    let pad_y = 8.0f32.min(h * 0.5);

    let x_mid = (ix0 + ix1) * 0.5;
    let y_mid = (iy0 + iy1) * 0.5;

    let candidates = [
        Point::new(fret_core::Px(x_mid), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(x_mid), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(x_mid), fret_core::Px(iy1 - pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(iy1 - pad_y)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(iy1 - pad_y)),
    ];

    let index = SemanticsIndex::new(snapshot);
    for pos in candidates {
        let Some(hit) = pick_semantics_node_at(snapshot, ui, pos) else {
            continue;
        };
        if semantics_hit_matches_intended(&index, hit, intended) {
            return PointerTargetResolution {
                position: pos,
                bounds: intended_bounds,
                bounds_source: resolved_bounds.source,
                used_window_scan: false,
                clamped_outside_window: false,
            };
        }
    }

    if let Some(pos) = scan_window_for_intended_hit(
        snapshot,
        ui,
        intended,
        window_bounds,
        intended_center.x.0,
        intended_center.y.0,
    ) {
        return PointerTargetResolution {
            position: pos,
            bounds: intended_bounds,
            bounds_source: resolved_bounds.source,
            used_window_scan: true,
            clamped_outside_window: false,
        };
    }

    PointerTargetResolution {
        position: candidates[0],
        bounds: intended_bounds,
        bounds_source: resolved_bounds.source,
        used_window_scan: false,
        clamped_outside_window: false,
    }
}

fn pointer_position_prefer_intended_hit(
    app: &App,
    snapshot: &fret_core::SemanticsSnapshot,
    element_runtime: Option<&ElementRuntime>,
    ui: &UiTree<App>,
    window: AppWindowId,
    intended: &fret_core::SemanticsNode,
    window_bounds: Rect,
) -> Point {
    pointer_target_resolution_prefer_intended_hit(
        app,
        snapshot,
        element_runtime,
        ui,
        window,
        intended,
        window_bounds,
    )
    .position
}

fn pointer_target_resolution_trace_note(
    action: &str,
    resolution: PointerTargetResolution,
) -> String {
    format!(
        "{action} bounds_source={} bounds=({:.1},{:.1},{:.1},{:.1}) window_scan={} clamped_outside_window={}",
        resolution.bounds_source,
        resolution.bounds.origin.x.0,
        resolution.bounds.origin.y.0,
        resolution.bounds.size.width.0,
        resolution.bounds.size.height.0,
        resolution.used_window_scan,
        resolution.clamped_outside_window,
    )
}

fn wheel_position_prefer_intended_hit(
    snapshot: &fret_core::SemanticsSnapshot,
    _element_runtime: Option<&ElementRuntime>,
    ui: &UiTree<App>,
    _window: AppWindowId,
    intended: &fret_core::SemanticsNode,
    container_bounds: Rect,
    window_bounds: Rect,
) -> Point {
    let cx0 = window_bounds.origin.x.0;
    let cy0 = window_bounds.origin.y.0;
    let cx1 = cx0 + window_bounds.size.width.0.max(0.0);
    let cy1 = cy0 + window_bounds.size.height.0.max(0.0);

    let bx0 = container_bounds.origin.x.0;
    let by0 = container_bounds.origin.y.0;
    let bx1 = bx0 + container_bounds.size.width.0.max(0.0);
    let by1 = by0 + container_bounds.size.height.0.max(0.0);

    let ix0 = bx0.max(cx0);
    let iy0 = by0.max(cy0);
    let ix1 = bx1.min(cx1);
    let iy1 = by1.min(cy1);

    if ix1 <= ix0 || iy1 <= iy0 {
        return center_of_rect(container_bounds);
    }

    let w = (ix1 - ix0).max(0.0);
    let h = (iy1 - iy0).max(0.0);
    let pad_x = 8.0f32.min(w * 0.5);
    let pad_y = 8.0f32.min(h * 0.5);

    let x_mid = (ix0 + ix1) * 0.5;
    let y_mid = (iy0 + iy1) * 0.5;

    let candidates = [
        Point::new(fret_core::Px(x_mid), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(x_mid), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(x_mid), fret_core::Px(iy1 - pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(iy1 - pad_y)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(iy1 - pad_y)),
    ];

    for pos in candidates {
        if let Some(hit) = pick_semantics_node_at(snapshot, ui, pos)
            && (hit.id.data().as_ffi() == intended.id.data().as_ffi()
                || hit
                    .controls
                    .iter()
                    .any(|id| id.data().as_ffi() == intended.id.data().as_ffi()))
        {
            return pos;
        }
    }

    candidates[0]
}

#[cfg(test)]
mod window_target_and_geometry_helper_tests {
    use super::*;

    #[test]
    fn center_of_rect_clamped_to_rect_clamps_disjoint_rects_into_window() {
        let rect = Rect::new(
            Point::new(Px(512.0), Px(1178.0)),
            fret_core::Size::new(Px(59.5), Px(34.0)),
        );
        let clamp = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(1360.0), Px(900.0)),
        );

        let center = center_of_rect_clamped_to_rect(rect, clamp);

        assert_eq!(center.x, Px(541.75));
        assert_eq!(center.y, Px(900.0));
    }

    #[test]
    fn center_of_rect_clamped_to_rect_keeps_intersecting_rect_centers() {
        let rect = Rect::new(
            Point::new(Px(512.0), Px(708.0)),
            fret_core::Size::new(Px(59.5), Px(34.0)),
        );
        let clamp = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(1360.0), Px(900.0)),
        );

        let center = center_of_rect_clamped_to_rect(rect, clamp);

        assert_eq!(center.x, Px(541.75));
        assert_eq!(center.y, Px(725.0));
    }
}

fn parse_semantics_numeric_value(value: &str) -> Option<f32> {
    let s = value.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(raw) = s.strip_suffix('%') {
        return raw.trim().parse::<f32>().ok();
    }
    if let Ok(v) = s.parse::<f32>() {
        return Some(v);
    }

    // Best-effort: extract the first float-ish token from the string.
    let mut token = String::new();
    let mut started = false;
    for ch in s.chars() {
        let keep = ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+');
        if keep {
            token.push(ch);
            started = true;
        } else if started {
            break;
        }
    }
    if token.is_empty() {
        return None;
    }
    token.parse::<f32>().ok()
}
