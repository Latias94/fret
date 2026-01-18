use super::frame::layout_style_for_instance;
use super::frame::{
    DismissibleLayerProps, ElementFrame, ElementInstance, ElementRecord, WindowFrame,
};
use super::host_widget::ElementHostWidget;
use super::prelude::*;
use std::collections::HashMap;

pub struct RenderRootContext<'a, H: UiHost> {
    pub ui: &'a mut UiTree<H>,
    pub app: &'a mut H,
    pub services: &'a mut dyn fret_core::UiServices,
    pub window: AppWindowId,
    pub bounds: Rect,
}

impl<'a, H: UiHost> RenderRootContext<'a, H> {
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

    pub fn render_root(
        self,
        root_name: &str,
        render: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> NodeId {
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

    pub fn render_dismissible_root_with_hooks(
        self,
        root_name: &str,
        render: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> NodeId {
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
            .find_map(|(&node, record)| (record.element == element).then_some(node))
    })
}

fn prepare_window_frame_for_frame(window_frame: &mut WindowFrame, frame_id: FrameId) {
    if window_frame.frame_id != frame_id {
        window_frame.frame_id = frame_id;
    }
}

pub(crate) fn children_for_node_in_window_frame<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    node: NodeId,
) -> Vec<NodeId> {
    with_window_frame(app, window, |window_frame| {
        window_frame
            .and_then(|w| w.children.get(&node))
            .cloned()
            .unwrap_or_default()
    })
}

/// Render a declarative element tree into an existing `UiTree` root.
///
/// Call this once per frame *before* `layout_all`/`paint_all`, for the relevant window.
pub fn render_root<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> NodeId {
    let frame_id = app.frame_id();
    let focused = ui.focus();
    ui.begin_debug_frame_if_needed(frame_id);

    let ui_ref: &UiTree<H> = &*ui;
    let children =
        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, app| {
            let mut should_reuse_view_cache =
                |node: NodeId| ui_ref.should_reuse_view_cache_node(node);
            let mut cx = crate::elements::ElementContext::new_for_root_name(
                app, runtime, window, bounds, root_name,
            );
            cx.set_view_cache_should_reuse(&mut should_reuse_view_cache);
            cx.sync_focused_element_from_focused_node(focused);
            cx.dismissible_clear_on_dismiss_request();
            cx.dismissible_clear_on_pointer_move();
            render(&mut cx)
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

        app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
            let window_frame = frame.windows.entry(window).or_default();
            prepare_window_frame_for_frame(window_frame, frame_id);
            let mut pending_invalidations: HashMap<NodeId, u8> = HashMap::new();

            window_frame.instances.insert(
                root_node,
                ElementRecord {
                    element: root_id,
                    instance: ElementInstance::Stack(StackProps::default()),
                },
            );

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
            window_frame.children.insert(root_node, mounted_children);

            apply_pending_invalidations(ui, pending_invalidations);
        });

        crate::declarative::frame::register_scroll_handle_bindings_batch(
            app,
            window,
            frame_id,
            scroll_bindings,
        );

        // Record the root's coordinate space for placement/collision logic (anchored overlays).
        window_state.set_root_bounds(root_id, bounds);

        // Sweep nodes that are not seen for `gc_lag_frames`.
        let mut stale_nodes: Vec<NodeId> = Vec::new();
        window_state.retain_nodes(|id, entry| {
            if *id == root_id {
                return true;
            }
            if entry.root != root_id {
                return true;
            }
            if entry.last_seen_frame.0 >= cutoff {
                return true;
            }
            stale_nodes.push(entry.node);
            false
        });

        for node in stale_nodes {
            let removed = ui.remove_subtree(services, node);
            app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
                let window_frame = frame.windows.entry(window).or_default();
                for removed in removed {
                    window_frame.instances.remove(&removed);
                    window_frame.children.remove(&removed);
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
pub fn render_dismissible_root_with_hooks<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> NodeId {
    render_dismissible_root_impl(ui, app, services, window, bounds, root_name, render)
}

#[allow(clippy::too_many_arguments)]
fn render_dismissible_root_impl<
    H: UiHost,
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: F,
) -> NodeId {
    let frame_id = app.frame_id();
    let focused = ui.focus();
    ui.begin_debug_frame_if_needed(frame_id);

    let ui_ref: &UiTree<H> = &*ui;
    let children =
        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, app| {
            let mut should_reuse_view_cache =
                |node: NodeId| ui_ref.should_reuse_view_cache_node(node);
            let mut cx = crate::elements::ElementContext::new_for_root_name(
                app, runtime, window, bounds, root_name,
            );
            cx.set_view_cache_should_reuse(&mut should_reuse_view_cache);
            cx.sync_focused_element_from_focused_node(focused);
            cx.dismissible_clear_on_dismiss_request();
            cx.dismissible_clear_on_pointer_move();
            render(&mut cx)
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

        app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
            let window_frame = frame.windows.entry(window).or_default();
            prepare_window_frame_for_frame(window_frame, frame_id);
            let mut pending_invalidations: HashMap<NodeId, u8> = HashMap::new();

            window_frame.instances.insert(
                root_node,
                ElementRecord {
                    element: root_id,
                    instance: ElementInstance::DismissibleLayer(DismissibleLayerProps::default()),
                },
            );

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

            apply_pending_invalidations(ui, pending_invalidations);
        });

        crate::declarative::frame::register_scroll_handle_bindings_batch(
            app,
            window,
            frame_id,
            scroll_bindings,
        );

        // Record the root's coordinate space for placement/collision logic (anchored overlays).
        window_state.set_root_bounds(root_id, bounds);

        // Sweep nodes that are not seen for `gc_lag_frames`.
        let mut stale_nodes: Vec<NodeId> = Vec::new();
        window_state.retain_nodes(|id, entry| {
            if *id == root_id {
                return true;
            }
            if entry.root != root_id {
                return true;
            }
            if entry.last_seen_frame.0 >= cutoff {
                return true;
            }
            stale_nodes.push(entry.node);
            false
        });

        for node in stale_nodes {
            let removed = ui.remove_subtree(services, node);
            app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
                let window_frame = frame.windows.entry(window).or_default();
                for removed in removed {
                    window_frame.instances.remove(&removed);
                    window_frame.children.remove(&removed);
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
    let mut reuse_view_cache =
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
        && view_cache_root_needs_layout_for_deferred_scroll_requests(window_frame, node)
    {
        reuse_view_cache = false;
        ui.invalidate(node, Invalidation::Layout);
    }

    if view_cache_props.is_some() && tracing::enabled!(tracing::Level::TRACE) {
        span.record("node", tracing::field::debug(node));
    }

    match &element.kind {
        ElementKind::ViewCache(props) => {
            ui.set_node_view_cache_flags(node, true, props.contained_layout);
            if !reuse_view_cache {
                ui.set_node_view_cache_needs_rerender(node, false);
            }
            let reuse_reason = if !had_existing_node_entry {
                crate::tree::UiDebugCacheRootReuseReason::FirstMount
            } else if !had_existing_node {
                crate::tree::UiDebugCacheRootReuseReason::NodeRecreated
            } else if reuse_view_cache {
                crate::tree::UiDebugCacheRootReuseReason::MarkedReuseRoot
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
            ui.set_node_view_cache_flags(node, false, false);
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
        ElementKind::Anchored(p) => ElementInstance::Anchored(p),
        ElementKind::Pressable(p) => ElementInstance::Pressable(p),
        ElementKind::PointerRegion(p) => ElementInstance::PointerRegion(p),
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

    let previous_instance = window_frame.instances.get(&node).map(|r| &r.instance);
    if !reuse_view_cache {
        let mask = declarative_instance_change_mask(previous_instance, &instance);
        if mask != 0 {
            pending_invalidations
                .entry(node)
                .and_modify(|m| *m |= mask)
                .or_insert(mask);
        }
    }

    window_frame.instances.insert(
        node,
        ElementRecord {
            element: id,
            instance,
        },
    );

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

        let children = ui.children(node);
        window_frame.children.insert(node, children);

        mark_existing_declarative_subtree_seen(ui, window_state, root_id, frame_id, node);
        inherit_observations_for_existing_subtree(window_state, window_frame, node);
        collect_scroll_handle_bindings_for_existing_subtree(window_frame, scroll_bindings, node);
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
        ui.set_children(node, child_nodes.clone());
        window_frame.children.insert(node, child_nodes);
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
        ui.set_children(node, child_nodes.clone());
        window_frame.children.insert(node, child_nodes);
    }

    node
}

const INVALIDATION_HIT_TEST: u8 = 1 << 0;
const INVALIDATION_LAYOUT: u8 = 1 << 1;
const INVALIDATION_PAINT: u8 = 1 << 2;

fn declarative_instance_change_mask(
    previous: Option<&ElementInstance>,
    next: &ElementInstance,
) -> u8 {
    let Some(previous) = previous else {
        return INVALIDATION_HIT_TEST | INVALIDATION_LAYOUT | INVALIDATION_PAINT;
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
    root_id: GlobalElementId,
    frame_id: FrameId,
    root: NodeId,
) {
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(element) = ui.node_element(node) {
            window_state.set_node_entry(
                element,
                NodeEntry {
                    node,
                    last_seen_frame: frame_id,
                    root: root_id,
                },
            );

            #[cfg(feature = "diagnostics")]
            window_state.touch_debug_identity_for_element(frame_id, element);
        }

        for child in ui.children(node) {
            stack.push(child);
        }
    }
}

fn collect_scroll_handle_bindings_for_existing_subtree(
    window_frame: &WindowFrame,
    out: &mut Vec<crate::declarative::frame::ScrollHandleBinding>,
    root: NodeId,
) {
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(record) = window_frame.instances.get(&node) {
            collect_scroll_handle_bindings(record.element, &record.instance, out);
        }

        if let Some(children) = window_frame.children.get(&node) {
            for &child in children {
                stack.push(child);
            }
        }
    }
}

fn view_cache_root_needs_layout_for_deferred_scroll_requests(
    window_frame: &WindowFrame,
    root: NodeId,
) -> bool {
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(record) = window_frame.instances.get(&node)
            && let ElementInstance::VirtualList(props) = &record.instance
            && props.scroll_handle.deferred_scroll_to_item().is_some()
        {
            return true;
        }

        if let Some(children) = window_frame.children.get(&node) {
            for &child in children {
                stack.push(child);
            }
        }
    }
    false
}

fn inherit_observations_for_existing_subtree(
    window_state: &mut crate::elements::WindowElementState,
    window_frame: &WindowFrame,
    root: NodeId,
) {
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(record) = window_frame.instances.get(&node) {
            let element = record.element;
            window_state.touch_observed_models_for_element_if_recorded(element);
            window_state.touch_observed_globals_for_element_if_recorded(element);
        }

        if let Some(children) = window_frame.children.get(&node) {
            for &child in children {
                stack.push(child);
            }
        }
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
