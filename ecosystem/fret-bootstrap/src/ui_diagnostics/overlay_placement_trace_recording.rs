fn ui_rect_from_rect(rect: Rect) -> UiRectV1 {
    UiRectV1 {
        x_px: rect.origin.x.0,
        y_px: rect.origin.y.0,
        w_px: rect.size.width.0,
        h_px: rect.size.height.0,
    }
}

fn ui_size_from_size(size: fret_core::Size) -> UiSizeV1 {
    UiSizeV1 {
        w_px: size.width.0,
        h_px: size.height.0,
    }
}

fn ui_edges_from_edges(edges: fret_core::Edges) -> UiEdgesV1 {
    UiEdgesV1 {
        top_px: edges.top.0,
        right_px: edges.right.0,
        bottom_px: edges.bottom.0,
        left_px: edges.left.0,
    }
}

fn ui_layout_direction_from_dir(
    dir: fret_ui::overlay_placement::LayoutDirection,
) -> UiLayoutDirectionV1 {
    match dir {
        fret_ui::overlay_placement::LayoutDirection::Ltr => UiLayoutDirectionV1::Ltr,
        fret_ui::overlay_placement::LayoutDirection::Rtl => UiLayoutDirectionV1::Rtl,
    }
}

fn ui_overlay_side_from_side(side: fret_ui::overlay_placement::Side) -> UiOverlaySideV1 {
    match side {
        fret_ui::overlay_placement::Side::Top => UiOverlaySideV1::Top,
        fret_ui::overlay_placement::Side::Bottom => UiOverlaySideV1::Bottom,
        fret_ui::overlay_placement::Side::Left => UiOverlaySideV1::Left,
        fret_ui::overlay_placement::Side::Right => UiOverlaySideV1::Right,
    }
}

fn ui_overlay_align_from_align(align: fret_ui::overlay_placement::Align) -> UiOverlayAlignV1 {
    match align {
        fret_ui::overlay_placement::Align::Start => UiOverlayAlignV1::Start,
        fret_ui::overlay_placement::Align::Center => UiOverlayAlignV1::Center,
        fret_ui::overlay_placement::Align::End => UiOverlayAlignV1::End,
    }
}

fn ui_overlay_sticky_from_sticky(
    sticky: fret_ui::overlay_placement::StickyMode,
) -> UiOverlayStickyModeV1 {
    match sticky {
        fret_ui::overlay_placement::StickyMode::Partial => UiOverlayStickyModeV1::Partial,
        fret_ui::overlay_placement::StickyMode::Always => UiOverlayStickyModeV1::Always,
    }
}

fn test_id_for_element(
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    window: AppWindowId,
    element: fret_ui::elements::GlobalElementId,
) -> Option<String> {
    let (Some(rt), Some(snapshot)) = (element_runtime, semantics_snapshot) else {
        return None;
    };
    let node_id = rt.node_for_element(window, element)?;
    let node = snapshot
        .nodes
        .iter()
        .find(|n| n.id.data().as_ffi() == node_id.data().as_ffi())?;
    node.test_id.clone()
}

fn record_overlay_placement_trace(
    trace: &mut Vec<UiOverlayPlacementTraceEntryV1>,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    window: AppWindowId,
    step_index: u32,
    note: &str,
) {
    let snapshot = element_runtime.and_then(|rt| rt.diagnostics_snapshot(window));
    let Some(snapshot) = snapshot else {
        return;
    };

    for rec in snapshot.overlay_placement.iter() {
        match rec {
            fret_ui::elements::OverlayPlacementDiagnosticsRecord::AnchoredPanel(r) => {
                let anchor_test_id = r.anchor_element.and_then(|el| {
                    test_id_for_element(element_runtime, semantics_snapshot, window, el)
                });
                let content_test_id = r.content_element.and_then(|el| {
                    test_id_for_element(element_runtime, semantics_snapshot, window, el)
                });
                let t = r.trace;
                let options = t.options;

                let arrow = t.layout.arrow.map(|a| UiOverlayArrowLayoutV1 {
                    side: ui_overlay_side_from_side(a.side),
                    offset_px: a.offset.0,
                    alignment_offset_px: a.alignment_offset.0,
                    center_offset_px: a.center_offset.0,
                });

                push_overlay_placement_trace(
                    trace,
                    UiOverlayPlacementTraceEntryV1::AnchoredPanel {
                        step_index,
                        note: Some(note.to_string()),
                        frame_id: r.frame_id.0,
                        overlay_root_name: r.overlay_root_name.as_deref().map(|s| s.to_string()),
                        anchor_element: r.anchor_element.map(|id| id.0),
                        anchor_test_id,
                        content_element: r.content_element.map(|id| id.0),
                        content_test_id,
                        outer_input: ui_rect_from_rect(t.outer_input),
                        outer_collision: ui_rect_from_rect(t.outer_collision),
                        anchor: ui_rect_from_rect(t.anchor),
                        desired: ui_size_from_size(t.desired),
                        side_offset_px: t.side_offset.0,
                        preferred_side: ui_overlay_side_from_side(t.preferred_side),
                        align: ui_overlay_align_from_align(t.align),
                        direction: ui_layout_direction_from_dir(options.direction),
                        sticky: ui_overlay_sticky_from_sticky(options.sticky),
                        offset: UiOverlayOffsetV1 {
                            main_axis_px: options.offset.main_axis.0,
                            cross_axis_px: options.offset.cross_axis.0,
                            alignment_axis_px: options.offset.alignment_axis.map(|v| v.0),
                        },
                        shift: UiOverlayShiftV1 {
                            main_axis: options.shift.main_axis,
                            cross_axis: options.shift.cross_axis,
                        },
                        collision_padding: ui_edges_from_edges(options.collision.padding),
                        collision_boundary: options.collision.boundary.map(ui_rect_from_rect),
                        gap_px: t.gap.0,
                        preferred_rect: ui_rect_from_rect(t.preferred_rect),
                        flipped_rect: ui_rect_from_rect(t.flipped_rect),
                        preferred_fits_without_main_clamp: t.preferred_fits_without_main_clamp,
                        flipped_fits_without_main_clamp: t.flipped_fits_without_main_clamp,
                        preferred_available_main_px: t.preferred_available_main_px,
                        flipped_available_main_px: t.flipped_available_main_px,
                        chosen_side: ui_overlay_side_from_side(t.chosen_side),
                        chosen_rect: ui_rect_from_rect(t.chosen_rect),
                        rect_after_shift: ui_rect_from_rect(t.rect_after_shift),
                        shift_delta: UiPointV1 {
                            x_px: t.shift_delta.x.0,
                            y_px: t.shift_delta.y.0,
                        },
                        final_rect: ui_rect_from_rect(t.layout.rect),
                        arrow,
                    },
                );
            }
            fret_ui::elements::OverlayPlacementDiagnosticsRecord::PlacedRect(r) => {
                let anchor_test_id = r.anchor_element.and_then(|el| {
                    test_id_for_element(element_runtime, semantics_snapshot, window, el)
                });
                let content_test_id = r.content_element.and_then(|el| {
                    test_id_for_element(element_runtime, semantics_snapshot, window, el)
                });
                push_overlay_placement_trace(
                    trace,
                    UiOverlayPlacementTraceEntryV1::PlacedRect {
                        step_index,
                        note: Some(note.to_string()),
                        frame_id: r.frame_id.0,
                        overlay_root_name: r.overlay_root_name.as_deref().map(|s| s.to_string()),
                        anchor_element: r.anchor_element.map(|id| id.0),
                        anchor_test_id,
                        content_element: r.content_element.map(|id| id.0),
                        content_test_id,
                        outer: ui_rect_from_rect(r.outer),
                        anchor: ui_rect_from_rect(r.anchor),
                        placed: ui_rect_from_rect(r.placed),
                        side: r.side.map(ui_overlay_side_from_side),
                    },
                );
            }
        }
    }
}
