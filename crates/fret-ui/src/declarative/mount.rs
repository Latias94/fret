use super::frame::layout_style_for_instance;
use super::frame::{
    DismissibleLayerProps, ElementFrame, ElementInstance, ElementRecord, WindowFrame,
};
use super::host_widget::ElementHostWidget;
use super::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::tree::{UiDebugInvalidationDetail, UiDebugInvalidationSource};

pub struct RenderRootContext<'a, H: UiHost> {
    pub ui: &'a mut UiTree<H>,
    pub app: &'a mut H,
    pub services: &'a mut dyn fret_core::UiServices,
    pub window: AppWindowId,
    pub bounds: Rect,
}

impl<'a, H: UiHost + 'static> RenderRootContext<'a, H> {
    pub fn new(
        ui: &'a mut UiTree<H>,
        app: &'a mut H,
        services: &'a mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
    ) -> Self {
        Self {
            ui,
            app,
            services,
            window,
            bounds,
        }
    }

    pub fn render_root<I>(
        self,
        root_name: &str,
        render: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> NodeId
    where
        I: IntoIterator<Item = AnyElement>,
    {
        crate::declarative::render_root(
            self.ui,
            self.app,
            self.services,
            self.window,
            self.bounds,
            root_name,
            render,
        )
    }

    pub fn render_dismissible_root_with_hooks<I>(
        self,
        root_name: &str,
        render: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> NodeId
    where
        I: IntoIterator<Item = AnyElement>,
    {
        crate::declarative::render_dismissible_root_with_hooks(
            self.ui,
            self.app,
            self.services,
            self.window,
            self.bounds,
            root_name,
            render,
        )
    }
}

pub(crate) fn with_window_frame<H: UiHost, R>(
    app: &mut H,
    window: AppWindowId,
    f: impl FnOnce(Option<&WindowFrame>) -> R,
) -> R {
    app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
        f(frame.windows.get(&window))
    })
}

pub(crate) fn node_for_element_in_window_frame<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<NodeId> {
    with_window_frame(app, window, |window_frame| {
        let window_frame = window_frame?;
        window_frame
            .instances
            .iter()
            .find_map(|(node, record)| (record.element == element).then_some(node))
    })
}

#[derive(Clone, Copy)]
struct StaleNodeRecord {
    node: NodeId,
    element: GlobalElementId,
    #[cfg(feature = "diagnostics")]
    element_root: GlobalElementId,
}

fn prepare_window_frame_for_frame(window_frame: &mut WindowFrame, frame_id: FrameId) {
    if window_frame.frame_id != frame_id {
        window_frame.frame_id = frame_id;
    }
}

fn set_window_frame_children(
    window_frame: &mut WindowFrame,
    parent: NodeId,
    children: Vec<NodeId>,
) {
    if let Some(prev) = window_frame.children.get(parent)
        && prev.as_ref() == children.as_slice()
    {
        return;
    }
    window_frame
        .children
        .insert(parent, Arc::<[NodeId]>::from(children));
}

pub(crate) fn children_for_node_in_window_frame<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    node: NodeId,
) -> Vec<NodeId> {
    with_window_frame(app, window, |window_frame| {
        window_frame
            .and_then(|w| w.children.get(node))
            .map(|children| children.as_ref().to_vec())
            .unwrap_or_default()
    })
}

/// Render a declarative element tree into an existing `UiTree` root.
///
/// Call this once per frame *before* `layout_all`/`paint_all`, for the relevant window.
pub fn render_root<H: UiHost, I>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> NodeId
where
    H: 'static,
    I: IntoIterator<Item = AnyElement>,
{
    let frame_id = app.frame_id();
    let focused = ui.focus();
    ui.begin_debug_frame_if_needed(frame_id);

    // Prepare per-window element runtime state up-front so render code can observe last-frame
    // geometry via `ElementContext::last_bounds_for_element` (e.g. measured-height motion).
    app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, _app| {
        runtime.prepare_window_for_frame(window, frame_id);
    });

    // Out-of-band scroll handle mutations (e.g. deferred scroll-to-item) must be visible to view
    // caching decisions. Apply scroll-handle-driven invalidations before running the declarative
    // render closure so cache-hit frames cannot replay stale virtual-surface output.
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        app,
        crate::layout_pass::LayoutPassKind::Final,
    );

    let ui_ref: &UiTree<H> = &*ui;
    let children: Vec<AnyElement> =
        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, app| {
            runtime.prepare_window_for_frame(window, frame_id);
            let mut should_reuse_view_cache =
                |node: NodeId| ui_ref.should_reuse_view_cache_node(node);
            let mut cx = crate::elements::ElementContext::new_for_root_name(
                app, runtime, window, bounds, root_name,
            );
            cx.set_view_cache_should_reuse(&mut should_reuse_view_cache);
            cx.sync_focused_element_from_focused_node(focused);
            cx.dismissible_clear_on_dismiss_request();
            cx.dismissible_clear_on_pointer_move();
            render(&mut cx).into_iter().collect()
        });

    app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let lag = runtime.gc_lag_frames();
        let cutoff = frame_id.0.saturating_sub(lag);

        let window_state = runtime.for_window_mut(window);
        let root_id = crate::elements::global_root(window, root_name);
        let mut scroll_bindings: Vec<crate::declarative::frame::ScrollHandleBinding> = Vec::new();

        let root_node = window_state
            .node_entry(root_id)
            .map(|e| e.node)
            .filter(|&node| ui.node_exists(node))
            .unwrap_or_else(|| {
                let node = ui.create_node(ElementHostWidget::new(root_id));
                ui.set_node_element(node, Some(root_id));
                window_state.set_node_entry(
                    root_id,
                    NodeEntry {
                        node,
                        last_seen_frame: frame_id,
                        root: root_id,
                    },
                );
                node
            });
        ui.set_node_element(root_node, Some(root_id));

        window_state.set_node_entry(
            root_id,
            NodeEntry {
                node: root_node,
                last_seen_frame: frame_id,
                root: root_id,
            },
        );

        // Declarative GC uses `UiTree::node_layer` to detect whether nodes are detached from any UI
        // layer. On the first frame, the base layer is typically registered by the app after
        // `render_root` returns (e.g. `ui.set_root(root_node)`), which is too late for the GC pass
        // below.
        //
        // However, `render_root` is also used for non-base roots (e.g. overlay helper roots). Do not
        // steal the base layer if it is already installed.
        if ui.node_layer(root_node).is_none() && ui.base_root().is_none() {
            ui.set_root(root_node);
        }

        let mut pending_invalidations: HashMap<NodeId, u8> = HashMap::new();
        app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
            let window_frame = frame.windows.entry(window).or_default();
            prepare_window_frame_for_frame(window_frame, frame_id);

            let inserted = window_frame
                .instances
                .insert(
                    root_node,
                    ElementRecord {
                        element: root_id,
                        instance: ElementInstance::Stack(StackProps::default()),
                    },
                )
                .is_none();

            let mut mounted_children: Vec<NodeId> = Vec::with_capacity(children.len());
            for child in children {
                mounted_children.push(mount_element(
                    ui,
                    window,
                    root_id,
                    frame_id,
                    window_state,
                    window_frame,
                    child,
                    &mut scroll_bindings,
                    &mut pending_invalidations,
                ));
            }
            ui.set_children(root_node, mounted_children.clone());
            set_window_frame_children(window_frame, root_node, mounted_children);
            if inserted {
                window_frame.revision = window_frame.revision.saturating_add(1);
            }

            let retained_virtual_lists = window_state.take_retained_virtual_list_reconciles();
            if !retained_virtual_lists.is_empty() {
                reconcile_retained_virtual_list_hosts(
                    ui,
                    _app,
                    window,
                    bounds,
                    root_id,
                    frame_id,
                    window_state,
                    window_frame,
                    &mut scroll_bindings,
                    &mut pending_invalidations,
                    retained_virtual_lists,
                );
            }
        });

        // View-cache experiments rely on explicit liveness bookkeeping (layer roots + view-cache
        // reuse roots + subtree membership lists; ADR 0191). Parent pointers are still required
        // for cache-root discovery and `node_layer` detachment checks, so repair any reachable
        // inconsistencies before applying invalidations that may need to propagate across cache-root
        // boundaries.
        if ui.view_cache_enabled() {
            let _ = ui.repair_parent_pointers_from_layer_roots();
        }

        apply_pending_invalidations(ui, pending_invalidations);

        if ui.view_cache_enabled() {
            ui.propagate_auto_sized_view_cache_root_invalidations();
        }

        for element in window_state.take_notify_for_animation_frame() {
            if let Some(node) = window_state.node_entry(element).map(|e| e.node) {
                ui.invalidate_with_source_and_detail(
                    node,
                    Invalidation::Paint,
                    UiDebugInvalidationSource::Notify,
                    UiDebugInvalidationDetail::AnimationFrameRequest,
                );
            }
        }

        crate::declarative::frame::register_scroll_handle_bindings_batch(
            app,
            window,
            frame_id,
            scroll_bindings,
        );

        // Record the root's coordinate space for placement/collision logic (anchored overlays).
        window_state.set_root_bounds(root_id, bounds);

        let keep_alive_view_cache_elements: HashSet<GlobalElementId> = {
            let mut keep_alive: HashSet<GlobalElementId> = HashSet::new();
            let mut visited_roots: HashSet<GlobalElementId> = HashSet::new();
            let mut stack: Vec<GlobalElementId> = window_state.view_cache_reuse_roots().collect();

            while let Some(root) = stack.pop() {
                if !visited_roots.insert(root) {
                    continue;
                }
                let Some(elements) = window_state.view_cache_elements_for_root(root) else {
                    continue;
                };
                for &element in elements {
                    keep_alive.insert(element);
                    if !visited_roots.contains(&element)
                        && window_state.view_cache_elements_for_root(element).is_some()
                    {
                        stack.push(element);
                    }
                }
            }

            keep_alive
        };

        // If any cache root transitions into reuse this frame, proactively touch the entire
        // retained subtree from the window root. This avoids GC sweeping still-live nodes in the
        // transition frame when the producer subtree starts skipping renders.
        if window_state
            .view_cache_transitioned_reuse_roots()
            .next()
            .is_some()
        {
            with_window_frame(app, window, |window_frame| {
                touch_existing_declarative_subtree_seen(
                    ui,
                    window_state,
                    window_frame,
                    root_id,
                    frame_id,
                    root_node,
                );
            });
        }

        // Node GC is keyed off `last_seen_frame`. Cache-hit frames can legitimately skip
        // re-mounting cached subtrees, so cache roots must keep the retained subtree alive.
        //
        // We only sweep nodes that are both stale and detached from any UI layer.
        //
        // Note: `UiTree::node_layer` relies on parent pointers. If a retained subtree becomes
        // inconsistent (children edges still attached, but parent pointers broken), `node_layer`
        // can return `None` even though the node is still reachable from the layer root. In that
        // case, do not sweep: treat reachability from the layer roots as authoritative for
        // liveness.
        let liveness_roots = ui.all_layer_roots();
        let mut stale: Vec<StaleNodeRecord> = Vec::new();
        let mut reachable_from_layers: Option<HashSet<NodeId>> = None;
        let view_cache_has_reuse_roots = window_state.view_cache_reuse_roots().next().is_some();
        let reachable_from_view_cache_roots: Option<HashSet<NodeId>> = if view_cache_has_reuse_roots
        {
            let view_cache_reuse_roots: Vec<GlobalElementId> =
                window_state.view_cache_reuse_roots().collect();
            let view_cache_reuse_root_nodes: Vec<NodeId> = view_cache_reuse_roots
                .iter()
                .filter_map(|root| window_state.node_entry(*root).map(|e| e.node))
                .collect();

            let mut reachable: HashSet<NodeId> = HashSet::new();

            if !view_cache_reuse_root_nodes.is_empty() {
                reachable.extend(with_window_frame(app, window, |window_frame| {
                    collect_reachable_nodes_for_gc(
                        ui,
                        window_frame,
                        view_cache_reuse_root_nodes.iter().copied(),
                    )
                }));
            }

            // Also treat recorded view-cache subtree memberships as authoritative reachability,
            // so cache hits can keep subtrees alive even when child edges are temporarily
            // incomplete (ADR 0191).
            for root in view_cache_reuse_roots {
                if let Some(elements) = window_state.view_cache_elements_for_root(root) {
                    for &element in elements {
                        if let Some(entry) = window_state.node_entry(element) {
                            reachable.insert(entry.node);
                        }
                    }
                }
            }

            Some(reachable)
        } else {
            None
        };
        window_state.retain_nodes(|id, entry| {
            if *id == root_id {
                return true;
            }
            if entry.root != root_id {
                return true;
            }
            if !keep_alive_view_cache_elements.is_empty()
                && keep_alive_view_cache_elements.contains(id)
            {
                entry.last_seen_frame = frame_id;
                return true;
            }
            if entry.last_seen_frame.0 >= cutoff {
                return true;
            }
            if ui.node_layer(entry.node).is_some() {
                return true;
            }
            let reachable = reachable_from_layers.get_or_insert_with(|| {
                with_window_frame(app, window, |window_frame| {
                    if liveness_roots.is_empty() {
                        collect_reachable_nodes_for_gc(ui, window_frame, std::iter::once(root_node))
                    } else {
                        collect_reachable_nodes_for_gc(
                            ui,
                            window_frame,
                            liveness_roots.iter().copied(),
                        )
                    }
                })
            });
            if reachable.contains(&entry.node) {
                return true;
            }
            if let Some(reachable) = reachable_from_view_cache_roots.as_ref()
                && reachable.contains(&entry.node)
            {
                return true;
            }
            stale.push(StaleNodeRecord {
                node: entry.node,
                element: *id,
                #[cfg(feature = "diagnostics")]
                element_root: entry.root,
            });
            false
        });

        for record in &stale {
            window_state.forget_view_cache_subtree_elements(record.element);
        }

        for record in stale {
            let node = record.node;
            #[cfg(feature = "diagnostics")]
            if let Some(ctx) = with_window_frame(app, window, |window_frame| {
                let window_frame = window_frame?;
                let parent = ui.node_parent(node);
                let parent_frame_children = parent.and_then(|p| window_frame.children.get(p));
                let root_reachable_from_view_cache_roots = reachable_from_view_cache_roots
                    .as_ref()
                    .map(|reachable| reachable.contains(&node));
                let view_cache_reuse_roots: Vec<GlobalElementId> =
                    window_state.view_cache_reuse_roots().collect();
                let liveness_layer_roots_len = liveness_roots.len().min(u32::MAX as usize) as u32;
                let view_cache_reuse_roots_len =
                    view_cache_reuse_roots.len().min(u32::MAX as usize) as u32;
                let view_cache_reuse_root_nodes_len = view_cache_reuse_roots
                    .iter()
                    .filter(|root| window_state.node_entry(**root).is_some())
                    .count()
                    .min(u32::MAX as usize)
                    as u32;
                let mut path_edge_frame_contains_child: [u8; 16] = [2u8; 16];
                let mut path_edge_len: u8 = 0;
                let mut current = Some(node);
                while let Some(child) = current {
                    let Some(parent) = ui.node_parent(child) else {
                        break;
                    };
                    if (path_edge_len as usize) >= path_edge_frame_contains_child.len() {
                        break;
                    }
                    let contains = window_frame
                        .children
                        .get(parent)
                        .map(|children| children.contains(&child));
                    path_edge_frame_contains_child[path_edge_len as usize] = match contains {
                        Some(true) => 1,
                        Some(false) => 0,
                        None => 2,
                    };
                    path_edge_len = path_edge_len.saturating_add(1);
                    current = Some(parent);
                }
                Some(crate::tree::UiDebugRemoveSubtreeFrameContext {
                    parent_frame_children_len: parent_frame_children
                        .map(|v| v.len().min(u32::MAX as usize) as u32),
                    parent_frame_children_contains_root: parent_frame_children
                        .map(|v| v.contains(&node)),
                    root_frame_instance_present: window_frame.instances.contains_key(node),
                    root_frame_children_len: window_frame
                        .children
                        .get(node)
                        .map(|v| v.len().min(u32::MAX as usize) as u32),
                    root_reachable_from_view_cache_roots,
                    liveness_layer_roots_len,
                    view_cache_reuse_roots_len,
                    view_cache_reuse_root_nodes_len,
                    trigger_element: Some(record.element),
                    trigger_element_root: Some(record.element_root),
                    trigger_element_in_view_cache_keep_alive: Some(
                        keep_alive_view_cache_elements.contains(&record.element),
                    ),
                    trigger_element_listed_under_reuse_root: window_state
                        .view_cache_reuse_roots()
                        .find(|&root| {
                            window_state
                                .view_cache_elements_for_root(root)
                                .is_some_and(|elements| elements.contains(&record.element))
                        }),
                    path_edge_len,
                    path_edge_frame_contains_child,
                })
            }) {
                ui.debug_set_remove_subtree_frame_context(node, ctx);
            }

            let removed = ui.remove_subtree(services, node);
            app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
                let window_frame = frame.windows.entry(window).or_default();
                let any_removed = !removed.is_empty();
                for removed in removed {
                    window_frame.instances.remove(removed);
                    window_frame.children.remove(removed);
                }
                if any_removed {
                    window_frame.revision = window_frame.revision.saturating_add(1);
                }
            });
        }

        if window_state.wants_continuous_frames() {
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        root_node
    })
}

/// Render a declarative element tree into a full-window, input-transparent overlay root.
///
/// The root handles:
/// - Escape dismissal (bubbling from any focused descendant).
/// - Outside-press dismissal via the runtime outside-press observer pass (ADR 0069).
#[allow(clippy::too_many_arguments)]
pub fn render_dismissible_root_with_hooks<H: UiHost, I>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> NodeId
where
    H: 'static,
    I: IntoIterator<Item = AnyElement>,
{
    render_dismissible_root_impl(ui, app, services, window, bounds, root_name, render)
}

#[allow(clippy::too_many_arguments)]
fn render_dismissible_root_impl<H: UiHost, F, I>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: F,
) -> NodeId
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    let frame_id = app.frame_id();
    let focused = ui.focus();
    ui.begin_debug_frame_if_needed(frame_id);

    // Match `render_root`: apply out-of-band scroll handle invalidations before render so view
    // caching can make a correct reuse decision.
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        app,
        crate::layout_pass::LayoutPassKind::Final,
    );

    let ui_ref: &UiTree<H> = &*ui;
    let children: Vec<AnyElement> =
        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, app| {
            runtime.prepare_window_for_frame(window, frame_id);
            let mut should_reuse_view_cache =
                |node: NodeId| ui_ref.should_reuse_view_cache_node(node);
            let mut cx = crate::elements::ElementContext::new_for_root_name(
                app, runtime, window, bounds, root_name,
            );
            cx.set_view_cache_should_reuse(&mut should_reuse_view_cache);
            cx.sync_focused_element_from_focused_node(focused);
            cx.dismissible_clear_on_dismiss_request();
            cx.dismissible_clear_on_pointer_move();
            render(&mut cx).into_iter().collect()
        });

    app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let lag = runtime.gc_lag_frames();
        let cutoff = frame_id.0.saturating_sub(lag);

        let window_state = runtime.for_window_mut(window);
        let root_id = crate::elements::global_root(window, root_name);
        let mut scroll_bindings: Vec<crate::declarative::frame::ScrollHandleBinding> = Vec::new();

        let root_node = window_state
            .node_entry(root_id)
            .map(|e| e.node)
            .filter(|&node| ui.node_exists(node))
            .unwrap_or_else(|| {
                let node = ui.create_node(ElementHostWidget::new(root_id));
                ui.set_node_element(node, Some(root_id));
                window_state.set_node_entry(
                    root_id,
                    NodeEntry {
                        node,
                        last_seen_frame: frame_id,
                        root: root_id,
                    },
                );
                node
            });
        ui.set_node_element(root_node, Some(root_id));

        window_state.set_node_entry(
            root_id,
            NodeEntry {
                node: root_node,
                last_seen_frame: frame_id,
                root: root_id,
            },
        );

        let mut pending_invalidations: HashMap<NodeId, u8> = HashMap::new();
        app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
            let window_frame = frame.windows.entry(window).or_default();
            prepare_window_frame_for_frame(window_frame, frame_id);

            let inserted = window_frame
                .instances
                .insert(
                    root_node,
                    ElementRecord {
                        element: root_id,
                        instance: ElementInstance::DismissibleLayer(
                            DismissibleLayerProps::default(),
                        ),
                    },
                )
                .is_none();
            if inserted {
                window_frame.revision = window_frame.revision.saturating_add(1);
            }

            let mut mounted_children: Vec<NodeId> = Vec::with_capacity(children.len());
            for child in children {
                mounted_children.push(mount_element(
                    ui,
                    window,
                    root_id,
                    frame_id,
                    window_state,
                    window_frame,
                    child,
                    &mut scroll_bindings,
                    &mut pending_invalidations,
                ));
            }
            ui.set_children(root_node, mounted_children);
        });

        if ui.view_cache_enabled() {
            let _ = ui.repair_parent_pointers_from_layer_roots();
        }

        apply_pending_invalidations(ui, pending_invalidations);

        crate::declarative::frame::register_scroll_handle_bindings_batch(
            app,
            window,
            frame_id,
            scroll_bindings,
        );

        // Record the root's coordinate space for placement/collision logic (anchored overlays).
        window_state.set_root_bounds(root_id, bounds);

        let keep_alive_view_cache_elements: HashSet<GlobalElementId> = {
            let mut keep_alive: HashSet<GlobalElementId> = HashSet::new();
            let mut visited_roots: HashSet<GlobalElementId> = HashSet::new();
            let mut stack: Vec<GlobalElementId> = window_state.view_cache_reuse_roots().collect();

            while let Some(root) = stack.pop() {
                if !visited_roots.insert(root) {
                    continue;
                }
                let Some(elements) = window_state.view_cache_elements_for_root(root) else {
                    continue;
                };
                for &element in elements {
                    keep_alive.insert(element);
                    if !visited_roots.contains(&element)
                        && window_state.view_cache_elements_for_root(element).is_some()
                    {
                        stack.push(element);
                    }
                }
            }

            keep_alive
        };

        // See `render_root`: on the first cache-hit frame for a previously dirty root, ensure the
        // overlay subtree stays alive even if it won't rerender this frame.
        if window_state
            .view_cache_transitioned_reuse_roots()
            .next()
            .is_some()
        {
            with_window_frame(app, window, |window_frame| {
                touch_existing_declarative_subtree_seen(
                    ui,
                    window_state,
                    window_frame,
                    root_id,
                    frame_id,
                    root_node,
                );
            });
        }

        // See `render_root`: cache-hit frames can skip re-mounting cached subtrees, so we sweep
        // only detached nodes that have been stale beyond the configured lag window.
        let liveness_roots = ui.all_layer_roots();
        let mut stale: Vec<StaleNodeRecord> = Vec::new();
        let mut reachable_from_layers: Option<HashSet<NodeId>> = None;
        let view_cache_has_reuse_roots = window_state.view_cache_reuse_roots().next().is_some();
        let reachable_from_view_cache_roots: Option<HashSet<NodeId>> = if view_cache_has_reuse_roots
        {
            let view_cache_reuse_roots: Vec<GlobalElementId> =
                window_state.view_cache_reuse_roots().collect();
            let view_cache_reuse_root_nodes: Vec<NodeId> = view_cache_reuse_roots
                .iter()
                .filter_map(|root| window_state.node_entry(*root).map(|e| e.node))
                .collect();

            let mut reachable: HashSet<NodeId> = HashSet::new();

            if !view_cache_reuse_root_nodes.is_empty() {
                reachable.extend(with_window_frame(app, window, |window_frame| {
                    collect_reachable_nodes_for_gc(
                        ui,
                        window_frame,
                        view_cache_reuse_root_nodes.iter().copied(),
                    )
                }));
            }

            for root in view_cache_reuse_roots {
                if let Some(elements) = window_state.view_cache_elements_for_root(root) {
                    for &element in elements {
                        if let Some(entry) = window_state.node_entry(element) {
                            reachable.insert(entry.node);
                        }
                    }
                }
            }

            Some(reachable)
        } else {
            None
        };
        window_state.retain_nodes(|id, entry| {
            if *id == root_id {
                return true;
            }
            if entry.root != root_id {
                return true;
            }

            if !keep_alive_view_cache_elements.is_empty()
                && keep_alive_view_cache_elements.contains(id)
            {
                entry.last_seen_frame = frame_id;
                return true;
            }

            if entry.last_seen_frame.0 >= cutoff {
                return true;
            }
            if ui.node_layer(entry.node).is_some() {
                return true;
            }
            let reachable = reachable_from_layers.get_or_insert_with(|| {
                with_window_frame(app, window, |window_frame| {
                    if liveness_roots.is_empty() {
                        collect_reachable_nodes_for_gc(ui, window_frame, std::iter::once(root_node))
                    } else {
                        collect_reachable_nodes_for_gc(
                            ui,
                            window_frame,
                            liveness_roots.iter().copied(),
                        )
                    }
                })
            });
            if reachable.contains(&entry.node) {
                return true;
            }
            if let Some(reachable) = reachable_from_view_cache_roots.as_ref()
                && reachable.contains(&entry.node)
            {
                return true;
            }
            stale.push(StaleNodeRecord {
                node: entry.node,
                element: *id,
                #[cfg(feature = "diagnostics")]
                element_root: entry.root,
            });
            false
        });

        for record in &stale {
            window_state.forget_view_cache_subtree_elements(record.element);
        }

        for record in stale {
            let node = record.node;
            #[cfg(feature = "diagnostics")]
            if let Some(ctx) = with_window_frame(app, window, |window_frame| {
                let window_frame = window_frame?;
                let parent = ui.node_parent(node);
                let parent_frame_children = parent.and_then(|p| window_frame.children.get(p));
                let root_reachable_from_view_cache_roots = reachable_from_view_cache_roots
                    .as_ref()
                    .map(|reachable| reachable.contains(&node));
                let view_cache_reuse_roots: Vec<GlobalElementId> =
                    window_state.view_cache_reuse_roots().collect();
                let liveness_layer_roots_len = liveness_roots.len().min(u32::MAX as usize) as u32;
                let view_cache_reuse_roots_len =
                    view_cache_reuse_roots.len().min(u32::MAX as usize) as u32;
                let view_cache_reuse_root_nodes_len = view_cache_reuse_roots
                    .iter()
                    .filter(|root| window_state.node_entry(**root).is_some())
                    .count()
                    .min(u32::MAX as usize)
                    as u32;
                let mut path_edge_frame_contains_child: [u8; 16] = [2u8; 16];
                let mut path_edge_len: u8 = 0;
                let mut current = Some(node);
                while let Some(child) = current {
                    let Some(parent) = ui.node_parent(child) else {
                        break;
                    };
                    if (path_edge_len as usize) >= path_edge_frame_contains_child.len() {
                        break;
                    }
                    let contains = window_frame
                        .children
                        .get(parent)
                        .map(|children| children.contains(&child));
                    path_edge_frame_contains_child[path_edge_len as usize] = match contains {
                        Some(true) => 1,
                        Some(false) => 0,
                        None => 2,
                    };
                    path_edge_len = path_edge_len.saturating_add(1);
                    current = Some(parent);
                }
                Some(crate::tree::UiDebugRemoveSubtreeFrameContext {
                    parent_frame_children_len: parent_frame_children
                        .map(|v| v.len().min(u32::MAX as usize) as u32),
                    parent_frame_children_contains_root: parent_frame_children
                        .map(|v| v.contains(&node)),
                    root_frame_instance_present: window_frame.instances.contains_key(node),
                    root_frame_children_len: window_frame
                        .children
                        .get(node)
                        .map(|v| v.len().min(u32::MAX as usize) as u32),
                    root_reachable_from_view_cache_roots,
                    liveness_layer_roots_len,
                    view_cache_reuse_roots_len,
                    view_cache_reuse_root_nodes_len,
                    trigger_element: Some(record.element),
                    trigger_element_root: Some(record.element_root),
                    trigger_element_in_view_cache_keep_alive: Some(
                        keep_alive_view_cache_elements.contains(&record.element),
                    ),
                    trigger_element_listed_under_reuse_root: window_state
                        .view_cache_reuse_roots()
                        .find(|&root| {
                            window_state
                                .view_cache_elements_for_root(root)
                                .is_some_and(|elements| elements.contains(&record.element))
                        }),
                    path_edge_len,
                    path_edge_frame_contains_child,
                })
            }) {
                ui.debug_set_remove_subtree_frame_context(node, ctx);
            }

            let removed = ui.remove_subtree(services, node);
            app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
                let window_frame = frame.windows.entry(window).or_default();
                let any_removed = !removed.is_empty();
                for removed in removed {
                    window_frame.instances.remove(removed);
                    window_frame.children.remove(removed);
                }
                if any_removed {
                    window_frame.revision = window_frame.revision.saturating_add(1);
                }
            });
        }

        if window_state.wants_continuous_frames() {
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        root_node
    })
}

#[allow(clippy::too_many_arguments)]
fn mount_element<H: UiHost>(
    ui: &mut UiTree<H>,
    _window: AppWindowId,
    root_id: GlobalElementId,
    frame_id: fret_runtime::FrameId,
    window_state: &mut crate::elements::WindowElementState,
    window_frame: &mut WindowFrame,
    element: AnyElement,
    scroll_bindings: &mut Vec<crate::declarative::frame::ScrollHandleBinding>,
    pending_invalidations: &mut HashMap<NodeId, u8>,
) -> NodeId {
    let id = element.id;
    let existing_node_entry = window_state.node_entry(id);
    let had_existing_node_entry = existing_node_entry.is_some();
    let had_existing_node = existing_node_entry
        .map(|e| ui.node_exists(e.node))
        .unwrap_or(false);
    let view_cache_props = match &element.kind {
        ElementKind::ViewCache(props) => Some(*props),
        _ => None,
    };
    let reuse_view_cache =
        view_cache_props.is_some() && window_state.should_reuse_view_cache_root(id);

    let span = if view_cache_props.is_some() && tracing::enabled!(tracing::Level::TRACE) {
        tracing::trace_span!(
            "ui.cache_root.mount",
            element = ?id,
            node = tracing::field::Empty,
            cache_hit = reuse_view_cache,
            contained_layout = view_cache_props
                .map(|p| p.contained_layout)
                .unwrap_or(false),
            frame_id = frame_id.0,
        )
    } else {
        tracing::Span::none()
    };
    let _span_guard = span.enter();

    let node = window_state
        .node_entry(id)
        .map(|e| e.node)
        .filter(|&node| ui.node_exists(node))
        .unwrap_or_else(|| {
            let node = ui.create_node(ElementHostWidget::new(id));
            ui.set_node_element(node, Some(id));
            window_state.set_node_entry(
                id,
                NodeEntry {
                    node,
                    last_seen_frame: frame_id,
                    root: root_id,
                },
            );
            node
        });
    ui.set_node_element(node, Some(id));

    window_state.set_node_entry(
        id,
        NodeEntry {
            node,
            last_seen_frame: frame_id,
            root: root_id,
        },
    );

    if reuse_view_cache
        && view_cache_root_needs_layout_for_deferred_scroll_requests(ui, window_frame, node)
    {
        // A deferred scroll request means we must run a contained relayout for this cache root,
        // even when the child render closure was skipped (cache hit).
        //
        // Importantly, do *not* disable reuse here: when the render closure is skipped the
        // declarative element list is intentionally empty, and treating that as authoritative would
        // detach the retained subtree (breaking semantics + scripted interactions).
        ui.invalidate(node, Invalidation::Layout);
    }

    if view_cache_props.is_some() && tracing::enabled!(tracing::Level::TRACE) {
        span.record("node", tracing::field::debug(node));
    }

    match &element.kind {
        ElementKind::ViewCache(props) => {
            let layout_definite = !matches!(props.layout.size.width, crate::element::Length::Auto)
                && !matches!(props.layout.size.height, crate::element::Length::Auto);
            ui.set_node_view_cache_flags(node, true, props.contained_layout, layout_definite);
            if !reuse_view_cache {
                ui.set_node_view_cache_needs_rerender(node, false);
            }
            let reuse_reason = if !had_existing_node_entry {
                crate::tree::UiDebugCacheRootReuseReason::FirstMount
            } else if !had_existing_node {
                crate::tree::UiDebugCacheRootReuseReason::NodeRecreated
            } else if reuse_view_cache {
                crate::tree::UiDebugCacheRootReuseReason::MarkedReuseRoot
            } else if window_state.view_cache_key_mismatch(id) {
                crate::tree::UiDebugCacheRootReuseReason::CacheKeyMismatch
            } else {
                crate::tree::UiDebugCacheRootReuseReason::NotMarkedReuseRoot
            };
            ui.debug_record_view_cache_root(
                node,
                reuse_view_cache,
                props.contained_layout,
                reuse_reason,
            );
        }
        _ => {
            ui.set_node_view_cache_flags(node, false, false, false);
        }
    }

    match &element.kind {
        ElementKind::TextInputRegion(props) => {
            ui.set_node_text_boundary_mode_override(node, props.text_boundary_mode_override);
        }
        _ => {
            ui.set_node_text_boundary_mode_override(node, None);
        }
    }

    let instance = match element.kind {
        ElementKind::Container(p) => ElementInstance::Container(p),
        ElementKind::Semantics(p) => ElementInstance::Semantics(p),
        ElementKind::SemanticFlex(p) => ElementInstance::SemanticFlex(p),
        ElementKind::FocusScope(p) => ElementInstance::FocusScope(p),
        ElementKind::InteractivityGate(p) => ElementInstance::InteractivityGate(p),
        ElementKind::Opacity(p) => ElementInstance::Opacity(p),
        ElementKind::EffectLayer(p) => ElementInstance::EffectLayer(p),
        ElementKind::ViewCache(p) => ElementInstance::ViewCache(p),
        ElementKind::VisualTransform(p) => ElementInstance::VisualTransform(p),
        ElementKind::RenderTransform(p) => ElementInstance::RenderTransform(p),
        ElementKind::FractionalRenderTransform(p) => ElementInstance::FractionalRenderTransform(p),
        ElementKind::Anchored(p) => ElementInstance::Anchored(p),
        ElementKind::Pressable(p) => ElementInstance::Pressable(p),
        ElementKind::PointerRegion(p) => ElementInstance::PointerRegion(p),
        ElementKind::TextInputRegion(p) => ElementInstance::TextInputRegion(p),
        ElementKind::InternalDragRegion(p) => ElementInstance::InternalDragRegion(p),
        ElementKind::RovingFlex(p) => ElementInstance::RovingFlex(p),
        ElementKind::Stack(p) => ElementInstance::Stack(p),
        ElementKind::Column(p) => ElementInstance::Flex(FlexProps {
            layout: p.layout,
            direction: fret_core::Axis::Vertical,
            gap: p.gap,
            padding: p.padding,
            justify: p.justify,
            align: p.align,
            wrap: false,
        }),
        ElementKind::Row(p) => ElementInstance::Flex(FlexProps {
            layout: p.layout,
            direction: fret_core::Axis::Horizontal,
            gap: p.gap,
            padding: p.padding,
            justify: p.justify,
            align: p.align,
            wrap: false,
        }),
        ElementKind::Spacer(p) => ElementInstance::Spacer(p),
        ElementKind::Text(p) => ElementInstance::Text(p),
        ElementKind::StyledText(p) => ElementInstance::StyledText(p),
        ElementKind::SelectableText(p) => ElementInstance::SelectableText(p),
        ElementKind::TextInput(p) => ElementInstance::TextInput(p),
        ElementKind::TextArea(p) => ElementInstance::TextArea(p),
        ElementKind::ResizablePanelGroup(p) => ElementInstance::ResizablePanelGroup(p),
        ElementKind::VirtualList(p) => ElementInstance::VirtualList(p),
        ElementKind::Flex(p) => ElementInstance::Flex(p),
        ElementKind::Grid(p) => ElementInstance::Grid(p),
        ElementKind::Image(p) => ElementInstance::Image(p),
        ElementKind::Canvas(p) => ElementInstance::Canvas(p),
        ElementKind::ViewportSurface(p) => ElementInstance::ViewportSurface(p),
        ElementKind::SvgIcon(p) => ElementInstance::SvgIcon(p),
        ElementKind::Spinner(p) => ElementInstance::Spinner(p),
        ElementKind::HoverRegion(p) => ElementInstance::HoverRegion(p),
        ElementKind::WheelRegion(p) => ElementInstance::WheelRegion(p),
        ElementKind::Scroll(p) => ElementInstance::Scroll(p),
        ElementKind::Scrollbar(p) => ElementInstance::Scrollbar(p),
    };

    collect_scroll_handle_bindings(id, &instance, scroll_bindings);
    let interactivity_gate_state = match &instance {
        ElementInstance::InteractivityGate(p) => Some((p.present, p.interactive)),
        _ => None,
    };
    let use_barrier_set_children = matches!(
        &instance,
        ElementInstance::VirtualList(props) if virtual_list_can_be_layout_barrier(props)
    );

    let previous_instance = window_frame.instances.get(node).map(|r| &r.instance);
    if !reuse_view_cache {
        let mask = declarative_instance_change_mask(previous_instance, &instance);
        if mask != 0 {
            ui.debug_record_hover_declarative_invalidation(
                node,
                (mask & INVALIDATION_HIT_TEST) != 0,
                (mask & INVALIDATION_LAYOUT) != 0,
                (mask & INVALIDATION_PAINT) != 0,
            );
            pending_invalidations
                .entry(node)
                .and_modify(|m| *m |= mask)
                .or_insert(mask);
        }
    }

    if let Some((present, interactive)) = interactivity_gate_state {
        ui.sync_interactivity_gate_widget(node, present, interactive);
    }
    let inserted = window_frame
        .instances
        .insert(
            node,
            ElementRecord {
                element: id,
                instance,
            },
        )
        .is_none();
    if inserted {
        window_frame.revision = window_frame.revision.saturating_add(1);
    }

    if reuse_view_cache {
        let reuse_span = if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace_span!(
                "ui.cache_root.reuse",
                element = ?id,
                node = ?node,
                cache_hit = true,
                contained_layout = view_cache_props
                    .map(|p| p.contained_layout)
                    .unwrap_or(false),
                frame_id = frame_id.0,
                reason = "marked_reuse_root",
            )
        } else {
            tracing::Span::none()
        };
        let _reuse_guard = reuse_span.enter();

        if window_frame.children.get(node).is_none() {
            window_frame
                .children
                .insert(node, Arc::<[NodeId]>::from(ui.children(node)));
        }

        let transitioned_into_reuse = window_state.record_view_cache_reuse_frame(id, frame_id);
        let touched =
            window_state.touch_view_cache_subtree_elements_if_recorded(id, frame_id, root_id);
        if transitioned_into_reuse && !touched {
            // If a cache root transitions into reuse without having a recorded subtree list yet,
            // fall back to walking the retained subtree so GC liveness bookkeeping remains
            // correct on the first cache-hit frame.
            mark_existing_declarative_subtree_seen(
                ui,
                window_state,
                window_frame,
                root_id,
                frame_id,
                node,
            );
            window_state.record_view_cache_subtree_elements(
                id,
                collect_declarative_elements_for_existing_subtree(
                    ui,
                    window_state,
                    window_frame,
                    node,
                ),
            );
        } else if !touched {
            mark_existing_declarative_subtree_seen(
                ui,
                window_state,
                window_frame,
                root_id,
                frame_id,
                node,
            );
            window_state.record_view_cache_subtree_elements(
                id,
                collect_declarative_elements_for_existing_subtree(
                    ui,
                    window_state,
                    window_frame,
                    node,
                ),
            );
        }
        inherit_observations_for_existing_subtree(ui, window_state, window_frame, node);
        collect_scroll_handle_bindings_for_existing_subtree(
            ui,
            window_frame,
            scroll_bindings,
            node,
        );
        return node;
    }

    if view_cache_props.is_some() {
        let reuse_span = if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace_span!(
                "ui.cache_root.reuse",
                element = ?id,
                node = ?node,
                cache_hit = false,
                contained_layout = view_cache_props
                    .map(|p| p.contained_layout)
                    .unwrap_or(false),
                frame_id = frame_id.0,
                reason = "not_marked_reuse_root",
            )
        } else {
            tracing::Span::none()
        };
        let _reuse_guard = reuse_span.enter();

        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(element.children.len());
        for child in element.children {
            child_nodes.push(mount_element(
                ui,
                _window,
                root_id,
                frame_id,
                window_state,
                window_frame,
                child,
                scroll_bindings,
                pending_invalidations,
            ));
        }
        if use_barrier_set_children {
            ui.set_children_barrier(node, child_nodes.clone());
        } else if had_existing_node {
            ui.set_children(node, child_nodes.clone());
        } else {
            ui.set_children_in_mount(node, child_nodes.clone());
        }
        set_window_frame_children(window_frame, node, child_nodes);

        // Keep a complete retained-subtree element list for this cache root so cache-hit frames
        // can refresh liveness without re-running the render closure.
        window_state.record_view_cache_subtree_elements(
            id,
            collect_declarative_elements_for_existing_subtree(ui, window_state, window_frame, node),
        );
    } else {
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(element.children.len());
        for child in element.children {
            child_nodes.push(mount_element(
                ui,
                _window,
                root_id,
                frame_id,
                window_state,
                window_frame,
                child,
                scroll_bindings,
                pending_invalidations,
            ));
        }
        if use_barrier_set_children {
            ui.set_children_barrier(node, child_nodes.clone());
        } else if had_existing_node {
            ui.set_children(node, child_nodes.clone());
        } else {
            ui.set_children_in_mount(node, child_nodes.clone());
        }
        set_window_frame_children(window_frame, node, child_nodes);
    }

    node
}

#[allow(clippy::too_many_arguments)]
fn reconcile_retained_virtual_list_hosts<H: UiHost + 'static>(
    ui: &mut UiTree<H>,
    app: &mut H,
    window: AppWindowId,
    bounds: Rect,
    root_id: GlobalElementId,
    frame_id: FrameId,
    window_state: &mut crate::elements::WindowElementState,
    window_frame: &mut WindowFrame,
    scroll_bindings: &mut Vec<crate::declarative::frame::ScrollHandleBinding>,
    pending_invalidations: &mut HashMap<NodeId, u8>,
    elements: Vec<GlobalElementId>,
) {
    if elements.is_empty() {
        return;
    }

    for element in elements {
        let Some(node) = window_state.node_entry(element).map(|e| e.node) else {
            continue;
        };

        let Some(record) = window_frame.instances.get(node) else {
            continue;
        };
        let ElementInstance::VirtualList(props) = &record.instance else {
            continue;
        };
        if !virtual_list_can_be_layout_barrier(props) {
            continue;
        }
        let props = props.clone();

        let Some((key_at, row, range_extractor)) = window_state
            .try_with_state_mut::<crate::windowed_surface_host::RetainedVirtualListHostCallbacks<H>, _>(
                element,
                |st| (Arc::clone(&st.key_at), Arc::clone(&st.row), st.range_extractor),
            )
        else {
            continue;
        };

        let desired_items: Option<Vec<crate::virtual_list::VirtualItem>> = window_state
            .with_state_mut(
                element,
                crate::element::VirtualListState::default,
                |state| {
                    state.metrics.ensure_with_mode(
                        props.measure_mode,
                        props.len,
                        props.estimate_row_height,
                        props.gap,
                        props.scroll_margin,
                    );

                    let viewport = match props.axis {
                        fret_core::Axis::Vertical => Px(state.viewport_h.0.max(0.0)),
                        fret_core::Axis::Horizontal => Px(state.viewport_w.0.max(0.0)),
                    };
                    if viewport.0 <= 0.0 || props.len == 0 {
                        return None;
                    }

                    let offset_point = props.scroll_handle.offset();
                    let offset_axis = match props.axis {
                        fret_core::Axis::Vertical => offset_point.y,
                        fret_core::Axis::Horizontal => offset_point.x,
                    };
                    let offset_axis = state.metrics.clamp_offset(offset_axis, viewport);

                    let range = state.window_range.or_else(|| {
                        state
                            .metrics
                            .visible_range(offset_axis, viewport, props.overscan)
                    });
                    state.window_range = range;
                    state.render_window_range = range;
                    let range = range?;

                    let mut indices = (range_extractor)(range)
                        .into_iter()
                        .filter(|&idx| idx < props.len)
                        .collect::<Vec<_>>();
                    indices.sort_unstable();
                    indices.dedup();

                    let items = indices
                        .iter()
                        .copied()
                        .map(|idx| {
                            let key = (key_at)(idx);
                            state.metrics.virtual_item(idx, key)
                        })
                        .collect::<Vec<_>>();
                    Some(items)
                },
            );

        let Some(desired_items) = desired_items else {
            continue;
        };

        let prev_items_len = props.visible_items.len();

        let mut existing_by_key: HashMap<crate::ItemKey, NodeId> = HashMap::new();
        {
            let current_children = ui.children(node);
            for (&child, item) in current_children.iter().zip(props.visible_items.iter()) {
                existing_by_key.insert(item.key, child);
            }
        }

        let mut preserved: u32 = 0;
        let mut attached: u32 = 0;
        let mut next_children: Vec<NodeId> = Vec::with_capacity(desired_items.len());
        for item in &desired_items {
            if let Some(existing) = existing_by_key.get(&item.key).copied() {
                next_children.push(existing);
                preserved = preserved.saturating_add(1);
                continue;
            }

            attached = attached.saturating_add(1);
            let child_element = {
                let mut cx = crate::elements::ElementContext::new_for_existing_window_state(
                    app,
                    window,
                    bounds,
                    element,
                    window_state,
                );
                let ui_ref: &UiTree<H> = &*ui;
                let mut should_reuse_view_cache =
                    |node: NodeId| ui_ref.should_reuse_view_cache_node(node);
                cx.set_view_cache_should_reuse(&mut should_reuse_view_cache);
                cx.retained_virtual_list_row_any_element(item.key, item.index, &row)
            };

            let child_node = mount_element(
                ui,
                window,
                root_id,
                frame_id,
                window_state,
                window_frame,
                child_element,
                scroll_bindings,
                pending_invalidations,
            );
            next_children.push(child_node);
        }

        let detached =
            (prev_items_len.saturating_sub(preserved as usize)).min(u32::MAX as usize) as u32;
        ui.debug_record_retained_virtual_list_reconcile(
            crate::tree::UiDebugRetainedVirtualListReconcile {
                node,
                element,
                prev_items: prev_items_len.min(u32::MAX as usize) as u32,
                next_items: desired_items.len().min(u32::MAX as usize) as u32,
                preserved_items: preserved,
                attached_items: attached,
                detached_items: detached,
            },
        );

        ui.set_children_barrier(node, next_children.clone());
        set_window_frame_children(window_frame, node, next_children);

        if let Some(record) = window_frame.instances.get_mut(node) {
            if let ElementInstance::VirtualList(props) = &mut record.instance {
                props.visible_items = desired_items;
            }
        }
    }
}

const INVALIDATION_HIT_TEST: u8 = 1 << 0;
const INVALIDATION_LAYOUT: u8 = 1 << 1;
const INVALIDATION_PAINT: u8 = 1 << 2;

fn declarative_instance_change_mask(
    previous: Option<&ElementInstance>,
    next: &ElementInstance,
) -> u8 {
    let Some(previous) = previous else {
        // Newly mounted nodes already start invalidated (layout/paint/hit-test) and structural
        // changes are handled via parent `set_children` updates. Avoid redundant invalidation
        // propagation in large rerender frames (e.g. VirtualList window jumps).
        return 0;
    };

    if std::mem::discriminant(previous) != std::mem::discriminant(next) {
        return INVALIDATION_HIT_TEST | INVALIDATION_LAYOUT | INVALIDATION_PAINT;
    }

    let mut layout_changed = layout_style_for_instance(previous) != layout_style_for_instance(next);
    let mut paint_changed = false;

    match (previous, next) {
        (ElementInstance::InteractivityGate(a), ElementInstance::InteractivityGate(b)) => {
            // Presence/interactivity gates affect layout participation, hit-testing, focus traversal,
            // and semantics inclusion. Even when the wrapper layout is unchanged, we need a layout
            // refresh so the host widget can recompute its derived flags.
            if a.present != b.present || a.interactive != b.interactive {
                layout_changed = true;
                paint_changed = true;
            }
        }
        (ElementInstance::Opacity(a), ElementInstance::Opacity(b)) => {
            if a.opacity != b.opacity {
                paint_changed = true;
            }
        }
        (ElementInstance::VisualTransform(a), ElementInstance::VisualTransform(b)) => {
            if a.transform != b.transform {
                paint_changed = true;
            }
        }
        (ElementInstance::RenderTransform(a), ElementInstance::RenderTransform(b)) => {
            // Render transforms affect paint and hit-testing. We treat them as a layout refresh so
            // the retained tree updates its per-node transform stack for hit-test/debug queries.
            if a.transform != b.transform {
                layout_changed = true;
                paint_changed = true;
            }
        }
        (
            ElementInstance::FractionalRenderTransform(a),
            ElementInstance::FractionalRenderTransform(b),
        ) => {
            // Fractional transforms are resolved during layout (dependent on bounds), but any input
            // change requires a layout refresh so we can recompute the pixel transform.
            if a.translate_x_fraction != b.translate_x_fraction
                || a.translate_y_fraction != b.translate_y_fraction
            {
                layout_changed = true;
                paint_changed = true;
            }
        }
        (ElementInstance::Anchored(a), ElementInstance::Anchored(b)) => {
            // Anchored placement is resolved during layout and affects the render transform stack.
            // Treat any meaningful input change as requiring a layout refresh.
            if a.outer_margin != b.outer_margin || a.anchor != b.anchor || a.options != b.options {
                layout_changed = true;
                paint_changed = true;
            }
        }
        (ElementInstance::Text(a), ElementInstance::Text(b)) => {
            if a.text != b.text
                || a.style != b.style
                || a.color != b.color
                || a.wrap != b.wrap
                || a.overflow != b.overflow
            {
                layout_changed = true;
                paint_changed = true;
            }
        }
        (ElementInstance::StyledText(a), ElementInstance::StyledText(b)) => {
            if a.rich != b.rich
                || a.style != b.style
                || a.color != b.color
                || a.wrap != b.wrap
                || a.overflow != b.overflow
            {
                layout_changed = true;
                paint_changed = true;
            }
        }
        (ElementInstance::SelectableText(a), ElementInstance::SelectableText(b)) => {
            if a.rich != b.rich
                || a.style != b.style
                || a.color != b.color
                || a.wrap != b.wrap
                || a.overflow != b.overflow
            {
                layout_changed = true;
                paint_changed = true;
            }
        }
        _ => {}
    }

    if layout_changed {
        return INVALIDATION_HIT_TEST | INVALIDATION_LAYOUT | INVALIDATION_PAINT;
    }
    if paint_changed {
        return INVALIDATION_PAINT;
    }
    0
}

fn virtual_list_can_be_layout_barrier(props: &crate::element::VirtualListProps) -> bool {
    match props.axis {
        fret_core::Axis::Vertical => {
            !matches!(props.layout.size.height, crate::element::Length::Auto)
        }
        fret_core::Axis::Horizontal => {
            !matches!(props.layout.size.width, crate::element::Length::Auto)
        }
    }
}

fn apply_pending_invalidations<H: UiHost>(ui: &mut UiTree<H>, pending: HashMap<NodeId, u8>) {
    for (node, mask) in pending {
        if (mask & INVALIDATION_HIT_TEST) != 0 {
            ui.invalidate(node, Invalidation::HitTest);
        }
        if (mask & INVALIDATION_LAYOUT) != 0 {
            ui.invalidate(node, Invalidation::Layout);
        }
        if (mask & INVALIDATION_PAINT) != 0 {
            ui.invalidate(node, Invalidation::Paint);
        }
    }
}

fn mark_existing_declarative_subtree_seen<H: UiHost>(
    ui: &UiTree<H>,
    window_state: &mut crate::elements::WindowElementState,
    window_frame: &WindowFrame,
    root_id: GlobalElementId,
    frame_id: FrameId,
    root: NodeId,
) {
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if !ui.node_exists(node) {
            continue;
        }
        if let Some(element) = window_frame
            .instances
            .get(node)
            .map(|r| r.element)
            .or_else(|| ui.node_element(node))
            .or_else(|| window_state.element_for_node(node))
        {
            let root = window_state
                .node_entry(element)
                .map(|e| e.root)
                .unwrap_or(root_id);
            window_state.set_node_entry(
                element,
                NodeEntry {
                    node,
                    last_seen_frame: frame_id,
                    root,
                },
            );

            #[cfg(feature = "diagnostics")]
            window_state.touch_debug_identity_for_element(frame_id, element);
        }

        push_existing_subtree_children(ui, window_frame, node, &mut stack);
    }
}

fn touch_existing_declarative_subtree_seen<H: UiHost>(
    ui: &UiTree<H>,
    window_state: &mut crate::elements::WindowElementState,
    window_frame: Option<&WindowFrame>,
    root_id: GlobalElementId,
    frame_id: FrameId,
    root: NodeId,
) {
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if !ui.node_exists(node) {
            continue;
        }
        if let Some(element) = window_frame
            .and_then(|window_frame| window_frame.instances.get(node).map(|r| r.element))
            .or_else(|| ui.node_element(node))
            .or_else(|| window_state.element_for_node(node))
        {
            let root = window_state
                .node_entry(element)
                .map(|e| e.root)
                .unwrap_or(root_id);
            window_state.set_node_entry(
                element,
                NodeEntry {
                    node,
                    last_seen_frame: frame_id,
                    root,
                },
            );

            #[cfg(feature = "diagnostics")]
            window_state.touch_debug_identity_for_element(frame_id, element);
        }

        if let Some(window_frame) = window_frame {
            push_existing_subtree_children(ui, window_frame, node, &mut stack);
        } else {
            for child in ui.children(node) {
                stack.push(child);
            }
        }
    }
}

fn collect_declarative_elements_for_existing_subtree<H: UiHost>(
    ui: &UiTree<H>,
    window_state: &crate::elements::WindowElementState,
    window_frame: &WindowFrame,
    root: NodeId,
) -> Vec<GlobalElementId> {
    let mut out: Vec<GlobalElementId> = Vec::new();
    let mut seen: HashSet<GlobalElementId> = HashSet::new();
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if !ui.node_exists(node) {
            continue;
        }
        if let Some(element) = window_frame
            .instances
            .get(node)
            .map(|r| r.element)
            .or_else(|| ui.node_element(node))
            .or_else(|| window_state.element_for_node(node))
        {
            if seen.insert(element) {
                out.push(element);
            }
        }

        push_existing_subtree_children(ui, window_frame, node, &mut stack);
    }
    out
}

fn collect_reachable_nodes_for_gc<H: UiHost>(
    ui: &UiTree<H>,
    window_frame: Option<&WindowFrame>,
    roots: impl IntoIterator<Item = NodeId>,
) -> HashSet<NodeId> {
    let mut out: HashSet<NodeId> = HashSet::new();
    let mut stack: Vec<NodeId> = roots.into_iter().collect();
    while let Some(node) = stack.pop() {
        if !out.insert(node) {
            continue;
        }
        if let Some(window_frame) = window_frame {
            push_existing_subtree_children(ui, window_frame, node, &mut stack);
        } else {
            stack.extend(ui.children(node));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gc_reachability_unions_ui_and_window_frame_children() {
        use crate::UiHost;
        use crate::declarative::frame::WindowFrame;
        use crate::tree::UiTree;
        use crate::widget::{LayoutCx, PaintCx, Widget};
        use fret_runtime::FrameId;

        #[derive(Default)]
        struct TestWidget;

        impl<H: UiHost> Widget<H> for TestWidget {
            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                for &child in cx.children {
                    let _ = cx.layout_in(child, cx.bounds);
                }
                cx.available
            }

            fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
        }

        let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
        ui.set_window(AppWindowId::default());
        ui.set_debug_enabled(true);
        ui.begin_debug_frame_if_needed(FrameId(1));

        let root = ui.create_node(TestWidget::default());
        let ui_child = ui.create_node(TestWidget::default());
        let frame_child = ui.create_node(TestWidget::default());

        ui.set_root(root);
        ui.set_children(root, vec![ui_child]);

        let mut window_frame = WindowFrame::default();
        window_frame
            .children
            .insert(root, Arc::<[NodeId]>::from(vec![ui_child, frame_child]));

        let reachable = collect_reachable_nodes_for_gc(&ui, Some(&window_frame), [root]);
        assert!(reachable.contains(&root));
        assert!(reachable.contains(&ui_child));
        assert!(reachable.contains(&frame_child));
    }

    #[test]
    fn touch_existing_subtree_can_walk_window_frame_children() {
        use crate::UiHost;
        use crate::declarative::frame::WindowFrame;
        use crate::tree::UiTree;
        use crate::widget::{LayoutCx, PaintCx, Widget};
        use fret_runtime::FrameId;

        #[derive(Default)]
        struct TestWidget;

        impl<H: UiHost> Widget<H> for TestWidget {
            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                for &child in cx.children {
                    let _ = cx.layout_in(child, cx.bounds);
                }
                cx.available
            }

            fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
        }

        let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
        ui.set_window(AppWindowId::default());

        let root_node = ui.create_node(TestWidget::default());
        let child_node = ui.create_node(TestWidget::default());

        let root_element = GlobalElementId(1);
        let child_element = GlobalElementId(2);
        let root_id = GlobalElementId(999);

        ui.set_node_element(root_node, Some(root_element));
        ui.set_node_element(child_node, Some(child_element));

        // Intentionally omit `ui.set_children(root_node, ..)` so `UiTree` has no child edges.
        let mut window_frame = WindowFrame::default();
        window_frame
            .children
            .insert(root_node, Arc::<[NodeId]>::from(vec![child_node]));

        let mut window_state = crate::elements::WindowElementState::default();

        touch_existing_declarative_subtree_seen(
            &ui,
            &mut window_state,
            Some(&window_frame),
            root_id,
            FrameId(1),
            root_node,
        );

        let entry = window_state
            .node_entry(child_element)
            .expect("child touched");
        assert_eq!(entry.node, child_node);
        assert_eq!(entry.last_seen_frame, FrameId(1));
        assert_eq!(entry.root, root_id);
    }
}
fn collect_scroll_handle_bindings_for_existing_subtree<H: UiHost>(
    ui: &UiTree<H>,
    window_frame: &WindowFrame,
    out: &mut Vec<crate::declarative::frame::ScrollHandleBinding>,
    root: NodeId,
) {
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(record) = window_frame.instances.get(node) {
            collect_scroll_handle_bindings(record.element, &record.instance, out);
        }

        push_existing_subtree_children(ui, window_frame, node, &mut stack);
    }
}

fn view_cache_root_needs_layout_for_deferred_scroll_requests<H: UiHost>(
    ui: &UiTree<H>,
    window_frame: &WindowFrame,
    root: NodeId,
) -> bool {
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(record) = window_frame.instances.get(node)
            && let ElementInstance::VirtualList(props) = &record.instance
            && props.scroll_handle.deferred_scroll_to_item().is_some()
        {
            return true;
        }

        push_existing_subtree_children(ui, window_frame, node, &mut stack);
    }
    false
}

fn push_existing_subtree_children<H: UiHost>(
    ui: &UiTree<H>,
    window_frame: &WindowFrame,
    node: NodeId,
    stack: &mut Vec<NodeId>,
) {
    // GC reachability should be conservative: a retained subtree can temporarily have incomplete
    // `UiTree` child edges (eg. during view-cache reuse) while the `WindowFrame` still retains the
    // authoritative element-tree edges. Prefer the union of both sources so we don't misclassify
    // a still-live subtree as detached.
    let ui_children = ui.children(node);
    if !ui_children.is_empty() {
        stack.extend(ui_children.iter().copied());
    }
    if let Some(frame_children) = window_frame.children.get(node) {
        if ui_children.is_empty() {
            stack.extend(frame_children.iter().copied());
        } else {
            for &child in frame_children.iter() {
                if !ui_children.contains(&child) {
                    stack.push(child);
                }
            }
        }
    }
}

fn inherit_observations_for_existing_subtree<H: UiHost>(
    ui: &UiTree<H>,
    window_state: &mut crate::elements::WindowElementState,
    window_frame: &WindowFrame,
    root: NodeId,
) {
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(record) = window_frame.instances.get(node) {
            let element = record.element;
            window_state.touch_observed_models_for_element_if_recorded(element);
            window_state.touch_observed_globals_for_element_if_recorded(element);
            if matches!(record.instance, ElementInstance::ViewCache(_)) {
                window_state.touch_view_cache_state_keys_if_recorded(element);
            }
        }

        push_existing_subtree_children(ui, window_frame, node, &mut stack);
    }
}

fn collect_scroll_handle_bindings(
    element: GlobalElementId,
    instance: &ElementInstance,
    out: &mut Vec<crate::declarative::frame::ScrollHandleBinding>,
) {
    match instance {
        ElementInstance::VirtualList(props) => {
            let handle = props.scroll_handle.base_handle();
            out.push(crate::declarative::frame::ScrollHandleBinding {
                handle_key: handle.binding_key(),
                element,
                handle: handle.clone(),
            });
        }
        ElementInstance::Scroll(props) => {
            if let Some(handle) = props.scroll_handle.as_ref() {
                out.push(crate::declarative::frame::ScrollHandleBinding {
                    handle_key: handle.binding_key(),
                    element,
                    handle: handle.clone(),
                });
            }
        }
        ElementInstance::WheelRegion(props) => {
            out.push(crate::declarative::frame::ScrollHandleBinding {
                handle_key: props.scroll_handle.binding_key(),
                element,
                handle: props.scroll_handle.clone(),
            });
        }
        ElementInstance::Scrollbar(props) => {
            out.push(crate::declarative::frame::ScrollHandleBinding {
                handle_key: props.scroll_handle.binding_key(),
                element,
                handle: props.scroll_handle.clone(),
            });
        }
        _ => {}
    }
}
