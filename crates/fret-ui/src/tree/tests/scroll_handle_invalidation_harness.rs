use std::collections::HashMap;

use super::*;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

const SCROLL_HANDLE_INVALIDATION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tree/tests/fixtures/scroll_handle_invalidation_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ScrollHandleInvalidationScenario {
    Tree {
        root: String,
        nodes: Vec<TreeNodeSpec>,
        #[serde(default)]
        handles: Vec<ScrollHandleSpec>,
        steps: Vec<TreeStep>,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct TreeNodeSpec {
    id: String,
    #[serde(default)]
    element_id: Option<u64>,
    #[serde(default)]
    children: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ScrollHandleSpec {
    id: String,
    kind: ScrollHandleKind,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ScrollHandleKind {
    Scroll,
    VirtualList,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum TreeStep {
    BumpScrollRevision {
        handle: String,
    },
    CaptureMetrics {
        label: String,
    },
    ClearAllInvalidations,
    InvalidateScrollHandles {
        #[serde(default = "default_consume_deferred_scroll_to_item")]
        consume_deferred_scroll_to_item: bool,
        #[serde(default = "default_commit_scroll_handle_baselines")]
        commit_scroll_handle_baselines: bool,
    },
    RegisterScrollSurface {
        node: String,
        element_id: u64,
        handle: String,
        surface: ScrollSurfaceSpec,
    },
    SetBoundsAll {
        width: f32,
        height: f32,
    },
    SetDebugBaseline,
    SetScrollOffset {
        handle: String,
        x: f32,
        y: f32,
        #[serde(default)]
        mode: ScrollOffsetMode,
    },
    SetViewCacheFlags {
        node: String,
        enabled: bool,
        layout_contained_when_bounds_known: bool,
        layout_definite: bool,
    },
}

fn default_consume_deferred_scroll_to_item() -> bool {
    false
}

fn default_commit_scroll_handle_baselines() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ScrollSurfaceSpec {
    Scroll {
        #[serde(default)]
        windowed_paint: bool,
    },
    VirtualList {
        len: usize,
        overscan: usize,
        viewport: f32,
        estimate_row_height: f32,
        #[serde(default)]
        initial_offset: f32,
        #[serde(default)]
        render_start: Option<usize>,
        #[serde(default)]
        render_end: Option<usize>,
    },
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ScrollOffsetMode {
    #[default]
    External,
    Internal,
}

#[derive(Clone)]
enum HarnessScrollHandle {
    Scroll(crate::scroll::ScrollHandle),
    VirtualList(crate::scroll::VirtualListScrollHandle),
}

impl HarnessScrollHandle {
    fn kind(&self) -> ScrollHandleKind {
        match self {
            Self::Scroll(_) => ScrollHandleKind::Scroll,
            Self::VirtualList(_) => ScrollHandleKind::VirtualList,
        }
    }

    fn base_handle(&self) -> crate::scroll::ScrollHandle {
        match self {
            Self::Scroll(handle) => handle.clone(),
            Self::VirtualList(handle) => handle.base_handle().clone(),
        }
    }

    fn binding_key(&self) -> usize {
        self.base_handle().binding_key()
    }

    fn set_offset(&self, offset: Point, mode: ScrollOffsetMode) {
        let handle = self.base_handle();
        match mode {
            ScrollOffsetMode::External => handle.set_offset(offset),
            ScrollOffsetMode::Internal => handle.set_offset_internal(offset),
        }
    }

    fn bump_revision(&self) {
        self.base_handle().bump_revision();
    }

    fn as_virtual_list(
        &self,
        handle_id: &str,
    ) -> Result<crate::scroll::VirtualListScrollHandle, ScenarioObserveError> {
        match self {
            Self::VirtualList(handle) => Ok(handle.clone()),
            Self::Scroll(_) => Err(ScenarioObserveError::new(format!(
                "scroll handle {handle_id:?} is not a virtual list handle"
            ))),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct DebugBaseline {
    scroll_handle_changes: usize,
    invalidation_walks: usize,
}

#[test]
fn mechanism_harness_scroll_handle_invalidation_matches_oracles() {
    let suite: MechanismSuite<ScrollHandleInvalidationScenario> =
        MechanismSuite::from_json_str(SCROLL_HANDLE_INVALIDATION)
            .expect("scroll handle invalidation fixture suite");

    let mut observer: fn(
        &MechanismCase<ScrollHandleInvalidationScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<ScrollHandleInvalidationScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        ScrollHandleInvalidationScenario::Tree {
            root,
            nodes,
            handles,
            steps,
        } => observe_tree_case(root, nodes, handles, steps),
    }
}

fn observe_tree_case(
    root: &str,
    nodes: &[TreeNodeSpec],
    handles: &[ScrollHandleSpec],
    steps: &[TreeStep],
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    ui.set_view_cache_enabled(true);

    let mut ids = HashMap::new();
    for node in nodes {
        let ui_node = if let Some(element_id) = node.element_id {
            ui.create_node_for_element(crate::elements::GlobalElementId(element_id), TestStack)
        } else {
            ui.create_node(TestStack)
        };
        if ids.insert(node.id.clone(), ui_node).is_some() {
            return Err(ScenarioObserveError::new(format!(
                "duplicate node id {:?}",
                node.id
            )));
        }
    }

    let root_id = lookup_node(&ids, root)?;
    ui.set_root(root_id);
    for node in nodes {
        let parent = lookup_node(&ids, &node.id)?;
        let children = node
            .children
            .iter()
            .map(|child| lookup_node(&ids, child))
            .collect::<Result<Vec<_>, _>>()?;
        ui.set_children(parent, children);
    }

    let mut scroll_handles = HashMap::new();
    for handle in handles {
        let scroll_handle = match handle.kind {
            ScrollHandleKind::Scroll => {
                HarnessScrollHandle::Scroll(crate::scroll::ScrollHandle::default())
            }
            ScrollHandleKind::VirtualList => {
                HarnessScrollHandle::VirtualList(crate::scroll::VirtualListScrollHandle::new())
            }
        };
        if scroll_handles
            .insert(handle.id.clone(), scroll_handle)
            .is_some()
        {
            return Err(ScenarioObserveError::new(format!(
                "duplicate scroll handle id {:?}",
                handle.id
            )));
        }
    }

    let mut observed = ObservedTree::new(Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(1.0), Px(1.0)),
    ));
    let mut debug_baseline = DebugBaseline::default();
    for step in steps {
        apply_step(
            &mut app,
            &mut ui,
            window,
            &ids,
            nodes,
            &scroll_handles,
            step,
            &mut observed,
            &mut debug_baseline,
        )?;
    }

    append_metrics(&ui, &ids, nodes, &mut observed, None, debug_baseline);
    Ok(observed)
}

#[allow(clippy::too_many_arguments)]
fn apply_step(
    app: &mut crate::test_host::TestHost,
    ui: &mut UiTree<crate::test_host::TestHost>,
    window: AppWindowId,
    ids: &HashMap<String, fret_core::NodeId>,
    nodes: &[TreeNodeSpec],
    scroll_handles: &HashMap<String, HarnessScrollHandle>,
    step: &TreeStep,
    observed: &mut ObservedTree,
    debug_baseline: &mut DebugBaseline,
) -> Result<(), ScenarioObserveError> {
    match step {
        TreeStep::BumpScrollRevision { handle } => {
            lookup_handle(scroll_handles, handle)?.bump_revision();
        }
        TreeStep::CaptureMetrics { label } => {
            append_metrics(ui, ids, nodes, observed, Some(label), *debug_baseline);
        }
        TreeStep::ClearAllInvalidations => {
            for node in nodes {
                let id = lookup_node(ids, &node.id)?;
                if ui.nodes.contains_key(id) {
                    ui.test_clear_node_invalidations(id);
                    ui.nodes[id].view_cache_needs_rerender = false;
                }
            }
        }
        TreeStep::InvalidateScrollHandles {
            consume_deferred_scroll_to_item,
            commit_scroll_handle_baselines,
        } => {
            ui.invalidate_scroll_handle_bindings_for_changed_handles(
                app,
                crate::layout_pass::LayoutPassKind::Final,
                *consume_deferred_scroll_to_item,
                *commit_scroll_handle_baselines,
            );
        }
        TreeStep::RegisterScrollSurface {
            node,
            element_id,
            handle,
            surface,
        } => {
            register_scroll_surface(
                app,
                window,
                lookup_existing_node(ui, ids, node)?,
                crate::elements::GlobalElementId(*element_id),
                lookup_handle(scroll_handles, handle)?,
                handle,
                surface,
            )?;
        }
        TreeStep::SetBoundsAll { width, height } => {
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(*width), Px(*height)),
            );
            for node in nodes {
                let id = lookup_node(ids, &node.id)?;
                if ui.nodes.contains_key(id) {
                    ui.nodes[id].bounds = bounds;
                    ui.nodes[id].measured_size = bounds.size;
                }
            }
        }
        TreeStep::SetDebugBaseline => {
            *debug_baseline = DebugBaseline {
                scroll_handle_changes: ui.debug_scroll_handle_changes().len(),
                invalidation_walks: ui.debug_invalidation_walks().len(),
            };
        }
        TreeStep::SetScrollOffset { handle, x, y, mode } => {
            lookup_handle(scroll_handles, handle)?.set_offset(Point::new(Px(*x), Px(*y)), *mode);
        }
        TreeStep::SetViewCacheFlags {
            node,
            enabled,
            layout_contained_when_bounds_known,
            layout_definite,
        } => {
            ui.set_node_view_cache_flags(
                lookup_existing_node(ui, ids, node)?,
                *enabled,
                *layout_contained_when_bounds_known,
                *layout_definite,
            );
        }
    }
    Ok(())
}

fn register_scroll_surface(
    app: &mut crate::test_host::TestHost,
    window: AppWindowId,
    node: fret_core::NodeId,
    element: crate::elements::GlobalElementId,
    handle: &HarnessScrollHandle,
    handle_id: &str,
    surface: &ScrollSurfaceSpec,
) -> Result<(), ScenarioObserveError> {
    let instance = match surface {
        ScrollSurfaceSpec::Scroll { windowed_paint } => {
            if handle.kind() != ScrollHandleKind::Scroll {
                return Err(ScenarioObserveError::new(format!(
                    "scroll surface requires plain scroll handle {handle_id:?}"
                )));
            }
            crate::declarative::frame::ElementInstance::Scroll(crate::element::ScrollProps {
                layout: crate::element::LayoutStyle::default(),
                axis: crate::element::ScrollAxis::Y,
                scroll_handle: Some(handle.base_handle()),
                known_content_size: None,
                intrinsic_measure_mode: crate::element::ScrollIntrinsicMeasureMode::Content,
                windowed_paint: *windowed_paint,
                probe_unbounded: true,
            })
        }
        ScrollSurfaceSpec::VirtualList {
            len,
            overscan,
            viewport,
            estimate_row_height,
            initial_offset,
            render_start,
            render_end,
        } => {
            let virtual_handle = handle.as_virtual_list(handle_id)?;
            seed_virtual_list_state(
                app,
                window,
                element,
                *len,
                *overscan,
                Px(*viewport),
                Px(*estimate_row_height),
                Px(*initial_offset),
                *render_start,
                *render_end,
            )?;
            crate::declarative::frame::ElementInstance::VirtualList(
                crate::element::VirtualListProps {
                    layout: crate::element::LayoutStyle::default(),
                    axis: fret_core::Axis::Vertical,
                    len: *len,
                    items_revision: 0,
                    estimate_row_height: Px(*estimate_row_height),
                    measure_mode: crate::element::VirtualListMeasureMode::Fixed,
                    key_cache: crate::element::VirtualListKeyCacheMode::AllKeys,
                    overscan: *overscan,
                    effective_overscan: *overscan,
                    keep_alive: 0,
                    scroll_margin: Px(0.0),
                    gap: Px(0.0),
                    scroll_handle: virtual_handle,
                    visible_items: Vec::new(),
                },
            )
        }
    };

    crate::declarative::frame::with_window_frame_mut(app, window, |window_frame| {
        window_frame.instances.insert(
            node,
            crate::declarative::frame::ElementRecord {
                element,
                instance,
                inherited_foreground: None,
                inherited_text_style: None,
                semantics_decoration: None,
                key_context: None,
                layout_direction: fret_core::LayoutDirection::default(),
            },
        );
    });

    crate::declarative::frame::register_scroll_handle_bindings_batch(
        app,
        window,
        app.frame_id(),
        [crate::declarative::frame::ScrollHandleBinding {
            handle_key: handle.binding_key(),
            element,
            handle: handle.base_handle(),
        }],
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn seed_virtual_list_state(
    app: &mut crate::test_host::TestHost,
    window: AppWindowId,
    element: crate::elements::GlobalElementId,
    len: usize,
    overscan: usize,
    viewport: Px,
    estimate_row_height: Px,
    initial_offset: Px,
    render_start: Option<usize>,
    render_end: Option<usize>,
) -> Result<(), ScenarioObserveError> {
    let mut metrics = crate::virtual_list::VirtualListMetrics::default();
    metrics.ensure_with_mode(
        crate::element::VirtualListMeasureMode::Fixed,
        len,
        estimate_row_height,
        Px(0.0),
        Px(0.0),
    );

    let render_window = match (render_start, render_end) {
        (Some(start_index), Some(end_index)) => Some(crate::virtual_list::VirtualRange {
            start_index,
            end_index,
            overscan,
            count: len,
        }),
        (None, None) => metrics.visible_range(initial_offset, viewport, overscan),
        _ => {
            return Err(ScenarioObserveError::new(
                "virtual list render_start and render_end must be specified together",
            ));
        }
    };

    crate::elements::with_element_state(
        app,
        window,
        element,
        crate::element::VirtualListState::default,
        |state| {
            state.viewport_h = viewport;
            state.metrics = metrics;
            state.render_window_range = render_window;
        },
    );

    Ok(())
}

fn append_metrics(
    ui: &UiTree<crate::test_host::TestHost>,
    ids: &HashMap<String, fret_core::NodeId>,
    nodes: &[TreeNodeSpec],
    observed: &mut ObservedTree,
    prefix: Option<&str>,
    debug_baseline: DebugBaseline,
) {
    for node in nodes {
        let id = ids[&node.id];
        let exists = ui.nodes.contains_key(id);
        set_metric(
            observed,
            prefix,
            format!("node.{}.exists", node.id),
            bool_metric(exists),
        );
        if exists {
            let entry = &ui.nodes[id];
            set_metric(
                observed,
                prefix,
                format!("node.{}.layout_dirty", node.id),
                bool_metric(entry.invalidation.layout),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.paint_dirty", node.id),
                bool_metric(entry.invalidation.paint),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.hit_test_dirty", node.id),
                bool_metric(entry.invalidation.hit_test),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.view_cache_needs_rerender", node.id),
                bool_metric(entry.view_cache_needs_rerender),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.dirty_cache_root", node.id),
                bool_metric(ui.boundary_layout_dirty(id)),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.dirty_boundary", node.id),
                bool_metric(ui.boundary_layout_dirty(id)),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.should_reuse_view_cache", node.id),
                bool_metric(ui.should_reuse_view_cache_node(id)),
            );
        }
    }

    append_scroll_handle_change_metrics(ui, observed, prefix, debug_baseline);
    append_invalidation_walk_metrics(ui, observed, prefix, debug_baseline);
}

fn append_scroll_handle_change_metrics(
    ui: &UiTree<crate::test_host::TestHost>,
    observed: &mut ObservedTree,
    prefix: Option<&str>,
    baseline: DebugBaseline,
) {
    let changes = ui
        .debug_scroll_handle_changes()
        .get(baseline.scroll_handle_changes..)
        .unwrap_or_default();
    set_metric(
        observed,
        prefix,
        "debug.scroll_handle_changes_since_baseline.count",
        changes.len() as f32,
    );
    for metric in [
        "debug.scroll_handle_changes_since_baseline.layout_count",
        "debug.scroll_handle_changes_since_baseline.hit_test_only_count",
        "debug.scroll_handle_changes_since_baseline.offset_changed_count",
        "debug.scroll_handle_changes_since_baseline.viewport_changed_count",
        "debug.scroll_handle_changes_since_baseline.content_changed_count",
        "debug.scroll_handle_changes_since_baseline.bound_nodes_total",
        "debug.scroll_handle_changes_since_baseline.upgraded_to_layout_bindings_total",
    ] {
        set_metric(observed, prefix, metric, 0.0);
    }

    for change in changes {
        match change.kind {
            crate::tree::UiDebugScrollHandleChangeKind::Layout => set_metric_add(
                observed,
                prefix,
                "debug.scroll_handle_changes_since_baseline.layout_count",
                1.0,
            ),
            crate::tree::UiDebugScrollHandleChangeKind::HitTestOnly => set_metric_add(
                observed,
                prefix,
                "debug.scroll_handle_changes_since_baseline.hit_test_only_count",
                1.0,
            ),
        }
        if change.offset_changed {
            set_metric_add(
                observed,
                prefix,
                "debug.scroll_handle_changes_since_baseline.offset_changed_count",
                1.0,
            );
        }
        if change.viewport_changed {
            set_metric_add(
                observed,
                prefix,
                "debug.scroll_handle_changes_since_baseline.viewport_changed_count",
                1.0,
            );
        }
        if change.content_changed {
            set_metric_add(
                observed,
                prefix,
                "debug.scroll_handle_changes_since_baseline.content_changed_count",
                1.0,
            );
        }
        set_metric_add(
            observed,
            prefix,
            "debug.scroll_handle_changes_since_baseline.bound_nodes_total",
            change.bound_nodes_sample.len() as f32,
        );
        set_metric_add(
            observed,
            prefix,
            "debug.scroll_handle_changes_since_baseline.upgraded_to_layout_bindings_total",
            change.upgraded_to_layout_bindings as f32,
        );
    }
}

fn append_invalidation_walk_metrics(
    ui: &UiTree<crate::test_host::TestHost>,
    observed: &mut ObservedTree,
    prefix: Option<&str>,
    baseline: DebugBaseline,
) {
    let walks = ui
        .debug_invalidation_walks()
        .get(baseline.invalidation_walks..)
        .unwrap_or_default();
    set_metric(
        observed,
        prefix,
        "debug.invalidation_walks_since_baseline.count",
        walks.len() as f32,
    );
    for metric in [
        "debug.invalidation_walks_since_baseline.inv.layout_count",
        "debug.invalidation_walks_since_baseline.inv.paint_count",
        "debug.invalidation_walks_since_baseline.inv.hit_test_count",
        "debug.invalidation_walks_since_baseline.inv.hit_test_only_count",
        "debug.invalidation_walks_since_baseline.detail.scroll_handle_hit_test_only_count",
        "debug.invalidation_walks_since_baseline.detail.scroll_handle_layout_count",
        "debug.invalidation_walks_since_baseline.detail.scroll_handle_window_update_count",
    ] {
        set_metric(observed, prefix, metric, 0.0);
    }

    for walk in walks {
        set_metric_add(
            observed,
            prefix,
            format!(
                "debug.invalidation_walks_since_baseline.inv.{}_count",
                invalidation_label(walk.inv)
            ),
            1.0,
        );
        if let Some(detail) = walk.detail.as_str() {
            set_metric_add(
                observed,
                prefix,
                format!("debug.invalidation_walks_since_baseline.detail.{detail}_count"),
                1.0,
            );
        }
    }
}

fn invalidation_label(invalidation: Invalidation) -> &'static str {
    match invalidation {
        Invalidation::Layout => "layout",
        Invalidation::Paint => "paint",
        Invalidation::HitTest => "hit_test",
        Invalidation::HitTestOnly => "hit_test_only",
    }
}

fn set_metric(
    observed: &mut ObservedTree,
    prefix: Option<&str>,
    id: impl Into<String>,
    value: f32,
) {
    observed.set_metric(metric_id(prefix, id.into()), value);
}

fn set_metric_add(
    observed: &mut ObservedTree,
    prefix: Option<&str>,
    id: impl Into<String>,
    value: f32,
) {
    let id = metric_id(prefix, id.into());
    let next = observed.metric_value(&id).unwrap_or(0.0) + value;
    observed.set_metric(id, next);
}

fn metric_id(prefix: Option<&str>, id: String) -> String {
    if let Some(prefix) = prefix {
        format!("capture.{prefix}.{id}")
    } else {
        id
    }
}

fn lookup_node(
    ids: &HashMap<String, fret_core::NodeId>,
    node: &str,
) -> Result<fret_core::NodeId, ScenarioObserveError> {
    ids.get(node)
        .copied()
        .ok_or_else(|| ScenarioObserveError::new(format!("unknown node id {node:?}")))
}

fn lookup_existing_node(
    ui: &UiTree<crate::test_host::TestHost>,
    ids: &HashMap<String, fret_core::NodeId>,
    node: &str,
) -> Result<fret_core::NodeId, ScenarioObserveError> {
    let id = lookup_node(ids, node)?;
    if ui.nodes.contains_key(id) {
        Ok(id)
    } else {
        Err(ScenarioObserveError::new(format!(
            "node id {node:?} no longer exists"
        )))
    }
}

fn lookup_handle<'a>(
    handles: &'a HashMap<String, HarnessScrollHandle>,
    handle: &str,
) -> Result<&'a HarnessScrollHandle, ScenarioObserveError> {
    handles
        .get(handle)
        .ok_or_else(|| ScenarioObserveError::new(format!("unknown scroll handle id {handle:?}")))
}

fn bool_metric(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}
