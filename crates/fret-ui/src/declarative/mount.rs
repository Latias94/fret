use super::frame::{
    DismissibleLayerProps, ElementFrame, ElementInstance, ElementRecord, WindowFrame,
};
use super::host_widget::ElementHostWidget;
use super::prelude::*;

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
    app.with_global_mut(ElementFrame::default, |frame, _app| {
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
        window_frame.instances.clear();
    }
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

    let children = crate::elements::with_element_cx(app, window, bounds, root_name, |cx| {
        cx.sync_focused_element_from_focused_node(focused);
        cx.dismissible_clear_on_dismiss_request();
        cx.dismissible_clear_on_pointer_move();
        render(cx)
    });

    app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let lag = runtime.gc_lag_frames();
        let cutoff = frame_id.0.saturating_sub(lag);

        let window_state = runtime.for_window_mut(window);
        let root_id = crate::elements::global_root(window, root_name);
        let mut scroll_bindings: Vec<(usize, GlobalElementId)> = Vec::new();

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

        app.with_global_mut(ElementFrame::default, |frame, _app| {
            let window_frame = frame.windows.entry(window).or_default();
            prepare_window_frame_for_frame(window_frame, frame_id);

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
                ));
            }
            ui.set_children(root_node, mounted_children);
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
            let _ = ui.remove_subtree(services, node);
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

    let children = crate::elements::with_element_cx(app, window, bounds, root_name, |cx| {
        cx.sync_focused_element_from_focused_node(focused);
        cx.dismissible_clear_on_dismiss_request();
        cx.dismissible_clear_on_pointer_move();
        render(cx)
    });

    app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let lag = runtime.gc_lag_frames();
        let cutoff = frame_id.0.saturating_sub(lag);

        let window_state = runtime.for_window_mut(window);
        let root_id = crate::elements::global_root(window, root_name);
        let mut scroll_bindings: Vec<(usize, GlobalElementId)> = Vec::new();

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

        app.with_global_mut(ElementFrame::default, |frame, _app| {
            let window_frame = frame.windows.entry(window).or_default();
            prepare_window_frame_for_frame(window_frame, frame_id);

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
                ));
            }
            ui.set_children(root_node, mounted_children);
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
            let _ = ui.remove_subtree(services, node);
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
    scroll_bindings: &mut Vec<(usize, GlobalElementId)>,
) -> NodeId {
    let id = element.id;
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

    window_state.set_node_entry(
        id,
        NodeEntry {
            node,
            last_seen_frame: frame_id,
            root: root_id,
        },
    );

    let instance = match element.kind {
        ElementKind::Container(p) => ElementInstance::Container(p),
        ElementKind::Semantics(p) => ElementInstance::Semantics(p),
        ElementKind::FocusScope(p) => ElementInstance::FocusScope(p),
        ElementKind::InteractivityGate(p) => ElementInstance::InteractivityGate(p),
        ElementKind::Opacity(p) => ElementInstance::Opacity(p),
        ElementKind::EffectLayer(p) => ElementInstance::EffectLayer(p),
        ElementKind::VisualTransform(p) => ElementInstance::VisualTransform(p),
        ElementKind::RenderTransform(p) => ElementInstance::RenderTransform(p),
        ElementKind::Anchored(p) => ElementInstance::Anchored(p),
        ElementKind::Pressable(p) => ElementInstance::Pressable(p),
        ElementKind::PointerRegion(p) => ElementInstance::PointerRegion(p),
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
        ElementKind::ViewportSurface(p) => ElementInstance::ViewportSurface(p),
        ElementKind::SvgIcon(p) => ElementInstance::SvgIcon(p),
        ElementKind::Spinner(p) => ElementInstance::Spinner(p),
        ElementKind::HoverRegion(p) => ElementInstance::HoverRegion(p),
        ElementKind::WheelRegion(p) => ElementInstance::WheelRegion(p),
        ElementKind::Scroll(p) => ElementInstance::Scroll(p),
        ElementKind::Scrollbar(p) => ElementInstance::Scrollbar(p),
    };

    collect_scroll_handle_bindings(id, &instance, scroll_bindings);

    window_frame.instances.insert(
        node,
        ElementRecord {
            element: id,
            instance,
        },
    );

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
        ));
    }
    ui.set_children(node, child_nodes);

    node
}

fn collect_scroll_handle_bindings(
    element: GlobalElementId,
    instance: &ElementInstance,
    out: &mut Vec<(usize, GlobalElementId)>,
) {
    match instance {
        ElementInstance::VirtualList(props) => {
            out.push((props.scroll_handle.base_handle().binding_key(), element));
        }
        ElementInstance::Scroll(props) => {
            if let Some(handle) = props.scroll_handle.as_ref() {
                out.push((handle.binding_key(), element));
            }
        }
        ElementInstance::WheelRegion(props) => {
            out.push((props.scroll_handle.binding_key(), element));
        }
        ElementInstance::Scrollbar(props) => {
            out.push((props.scroll_handle.binding_key(), element));
        }
        _ => {}
    }
}
