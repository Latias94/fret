use super::*;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

const VIRTUAL_LIST_WINDOW_UPDATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tree/prepaint/tests/fixtures/virtual_list_window_update_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PrepaintScenario {
    VirtualListPrepaint {
        len: usize,
        items_revision: u64,
        state_items_revision: u64,
        viewport_height: f32,
        state_viewport_height: f32,
        row_height: f32,
        overscan: usize,
        scroll_offset_y: f32,
        state_offset_y: f32,
        #[serde(default)]
        scroll_to_item: Option<usize>,
        render_window: VirtualRangeSpec,
    },
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct VirtualRangeSpec {
    start_index: usize,
    end_index: usize,
    #[serde(default)]
    count: Option<usize>,
}

#[test]
fn mechanism_harness_prepaint_virtual_list_window_update_matches_oracles() {
    let suite: MechanismSuite<PrepaintScenario> =
        MechanismSuite::from_json_str(VIRTUAL_LIST_WINDOW_UPDATE)
            .expect("prepaint virtual-list window-update fixture suite");

    let mut observer: fn(
        &MechanismCase<PrepaintScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<PrepaintScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        PrepaintScenario::VirtualListPrepaint {
            len,
            items_revision,
            state_items_revision,
            viewport_height,
            state_viewport_height,
            row_height,
            overscan,
            scroll_offset_y,
            state_offset_y,
            scroll_to_item,
            render_window,
        } => observe_virtual_list_prepaint_case(
            *len,
            *items_revision,
            *state_items_revision,
            *viewport_height,
            *state_viewport_height,
            *row_height,
            *overscan,
            *scroll_offset_y,
            *state_offset_y,
            *scroll_to_item,
            *render_window,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn observe_virtual_list_prepaint_case(
    len: usize,
    items_revision: u64,
    state_items_revision: u64,
    viewport_height: f32,
    state_viewport_height: f32,
    row_height: f32,
    overscan: usize,
    scroll_offset_y: f32,
    state_offset_y: f32,
    scroll_to_item: Option<usize>,
    render_window: VirtualRangeSpec,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let cache_root = ui.create_node(NoopWidget);
    ui.nodes[cache_root].view_cache.enabled = true;
    ui.set_root(cache_root);

    let element = GlobalElementId(1);
    let vlist_node = ui.create_node_for_element(element, NoopWidget);
    ui.add_child(cache_root, vlist_node);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(viewport_height)),
    );
    ui.nodes[vlist_node].bounds = bounds;

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    if let Some(index) = scroll_to_item {
        scroll_handle.scroll_to_item(index, crate::scroll::ScrollStrategy::Nearest);
    }
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(scroll_offset_y)));

    crate::declarative::frame::with_window_frame_mut(&mut app, window, |frame| {
        frame.instances.insert(
            vlist_node,
            crate::declarative::frame::ElementRecord {
                element,
                instance: crate::declarative::frame::ElementInstance::VirtualList(
                    crate::element::VirtualListProps {
                        layout: crate::element::LayoutStyle::default(),
                        axis: fret_core::Axis::Vertical,
                        len,
                        items_revision,
                        estimate_row_height: Px(row_height),
                        measure_mode: crate::element::VirtualListMeasureMode::Fixed,
                        key_cache: crate::element::VirtualListKeyCacheMode::VisibleOnly,
                        overscan,
                        effective_overscan: overscan,
                        keep_alive: 0,
                        scroll_margin: Px(0.0),
                        gap: Px(0.0),
                        scroll_handle: scroll_handle.clone(),
                        visible_items: Vec::new(),
                    },
                ),
                inherited_foreground: None,
                inherited_text_style: None,
                semantics_decoration: None,
                key_context: None,
                layout_direction: fret_core::LayoutDirection::default(),
            },
        );
    });

    crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::VirtualListState::default,
        |state| {
            state.metrics.ensure_with_mode(
                crate::element::VirtualListMeasureMode::Fixed,
                len,
                Px(row_height),
                Px(0.0),
                Px(0.0),
            );
            state.items_len = len;
            state.items_revision = state_items_revision;
            state.offset_y = Px(state_offset_y);
            state.viewport_h = Px(state_viewport_height);
            state.render_window_range = Some(crate::virtual_list::VirtualRange {
                start_index: render_window.start_index,
                end_index: render_window.end_index,
                overscan,
                count: render_window.count.unwrap_or(len),
            });
        },
    );

    let record = InteractionRecord {
        node: vlist_node,
        bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: true,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: true,
    };

    ui.prepaint_virtual_list_window_from_interaction_record(&mut app, &record);

    let mut observed = ObservedTree::new(bounds);
    observed.set_metric(
        "cache_root.view_cache_needs_rerender",
        bool_metric(ui.nodes[cache_root].view_cache_needs_rerender),
    );
    observed.set_metric(
        "cache_root.dirty_cache_root",
        bool_metric(ui.boundary_layout_dirty(cache_root)),
    );

    let last = ui
        .debug_virtual_list_windows()
        .last()
        .ok_or_else(|| ScenarioObserveError::new("missing prepaint virtual-list debug record"))?;
    set_prepaint_window_metrics(&mut observed, last);
    set_cache_root_dirty_reason_metrics(&mut observed, &ui, cache_root);
    Ok(observed)
}

fn set_prepaint_window_metrics(
    observed: &mut ObservedTree,
    record: &crate::tree::UiDebugVirtualListWindow,
) {
    observed.set_metric(
        "prepaint.window_mismatch",
        bool_metric(record.window_mismatch),
    );
    observed.set_metric("prepaint.items_len", record.items_len as f32);
    observed.set_metric("prepaint.offset_px", record.offset.0);
    observed.set_metric("prepaint.content_extent_px", record.content_extent.0);
    match record.window_shift_kind {
        crate::tree::UiDebugVirtualListWindowShiftKind::None => {
            observed.set_metric("prepaint.window_shift.kind.none", 1.0);
        }
        crate::tree::UiDebugVirtualListWindowShiftKind::Prefetch => {
            observed.set_metric("prepaint.window_shift.kind.prefetch", 1.0);
        }
        crate::tree::UiDebugVirtualListWindowShiftKind::Escape => {
            observed.set_metric("prepaint.window_shift.kind.escape", 1.0);
        }
    }

    if let Some(reason) = record.window_shift_reason {
        observed.set_metric(
            format!(
                "prepaint.window_shift.reason.{}",
                window_shift_reason_label(reason)
            ),
            1.0,
        );
    }
    if let Some(mode) = record.window_shift_apply_mode {
        observed.set_metric(
            format!(
                "prepaint.window_shift.apply_mode.{}",
                apply_mode_label(mode)
            ),
            1.0,
        );
    }
    if let Some(detail) = record
        .window_shift_invalidation_detail
        .and_then(|d| d.as_str())
    {
        observed.set_metric(
            format!("prepaint.window_shift.invalidation_detail.{detail}"),
            1.0,
        );
    }
    if let Some(range) = record.window_range {
        observed.set_metric(
            "prepaint.window_range.start_index",
            range.start_index as f32,
        );
        observed.set_metric("prepaint.window_range.end_index", range.end_index as f32);
    }
}

fn set_cache_root_dirty_reason_metrics(
    observed: &mut ObservedTree,
    ui: &UiTree<crate::test_host::TestHost>,
    cache_root: NodeId,
) {
    if let Some((_source, detail)) = ui.boundary_layout_dirty_reason(cache_root)
        && let Some(label) = detail.as_str()
    {
        observed.set_metric(format!("cache_root.dirty_reason.{label}"), 1.0);
    }
}

fn window_shift_reason_label(
    reason: crate::tree::UiDebugVirtualListWindowShiftReason,
) -> &'static str {
    match reason {
        crate::tree::UiDebugVirtualListWindowShiftReason::ScrollOffset => "scroll_offset",
        crate::tree::UiDebugVirtualListWindowShiftReason::ViewportResize => "viewport_resize",
        crate::tree::UiDebugVirtualListWindowShiftReason::ItemsRevision => "items_revision",
        crate::tree::UiDebugVirtualListWindowShiftReason::ScrollToItem => "scroll_to_item",
        crate::tree::UiDebugVirtualListWindowShiftReason::InputsChange => "inputs_change",
        crate::tree::UiDebugVirtualListWindowShiftReason::Unknown => "unknown",
    }
}

fn apply_mode_label(mode: crate::tree::UiDebugVirtualListWindowShiftApplyMode) -> &'static str {
    match mode {
        crate::tree::UiDebugVirtualListWindowShiftApplyMode::RetainedReconcile => {
            "retained_reconcile"
        }
        crate::tree::UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender => {
            "non_retained_rerender"
        }
    }
}

fn bool_metric(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}
