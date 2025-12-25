use crate::UiHost;
use crate::element::{
    AnyElement, ColumnProps, ContainerProps, ElementKind, PressableProps, RowProps, StackProps,
    TextProps,
};
use crate::elements::{ElementCx, GlobalElementId, NodeEntry};
use crate::tree::UiTree;
use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget};
use fret_core::{
    AppWindowId, Color, CursorIcon, DrawOrder, Edges, Event, FontId, MouseButton, NodeId, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextStyle,
};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ElementFrame {
    windows: HashMap<AppWindowId, WindowFrame>,
}

#[derive(Default)]
pub(crate) struct WindowFrame {
    pub(crate) instances: HashMap<NodeId, ElementRecord>,
}

#[derive(Debug, Clone)]
pub(crate) enum ElementInstance {
    Container(ContainerProps),
    Pressable(PressableProps),
    Stack(StackProps),
    Column(ColumnProps),
    Row(RowProps),
    Text(TextProps),
    VirtualList(crate::element::VirtualListProps),
}

#[derive(Debug, Clone)]
pub(crate) struct ElementRecord {
    pub element: GlobalElementId,
    pub instance: ElementInstance,
}

pub(crate) fn element_record_for_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    node: NodeId,
) -> Option<ElementRecord> {
    app.with_global_mut(ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&node))
            .cloned()
    })
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

#[derive(Debug, Default, Clone)]
struct TextCache {
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<TextMetrics>,
    prepared_scale_factor_bits: Option<u32>,
    last_text: Option<std::sync::Arc<str>>,
    last_style: Option<TextStyle>,
    last_wrap: Option<fret_core::TextWrap>,
    last_width: Option<Px>,
    last_theme_revision: Option<u64>,
}

#[derive(Debug, Clone)]
struct ElementHostWidget {
    element: GlobalElementId,
    text_cache: TextCache,
}

impl ElementHostWidget {
    fn instance<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> Option<ElementInstance> {
        element_record_for_node(app, window, node).map(|r| r.instance)
    }
}

impl<H: UiHost> Widget<H> for ElementHostWidget {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };

        let Event::Pointer(pe) = event else {
            return;
        };

        match instance {
            ElementInstance::VirtualList(props) => match pe {
                fret_core::PointerEvent::Wheel { delta, .. } => {
                    crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::element::VirtualListState::default,
                        |state| {
                            let viewport_h = Px(state.viewport_h.0.max(0.0));
                            let row_h = Px(props.row_height.0.max(0.0));
                            let content_h = Px(row_h.0 * props.len as f32);
                            let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));

                            let next = Px((state.offset_y.0 - delta.y.0).max(0.0));
                            state.offset_y = Px(next.0.min(max_offset.0));
                        },
                    );
                    cx.invalidate_self(Invalidation::Layout);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Down { button, .. } => {
                    if *button == MouseButton::Left {
                        cx.request_focus(cx.node);
                    }
                }
                _ => {}
            },
            ElementInstance::Pressable(props) => {
                if !props.enabled {
                    return;
                }
                match pe {
                    fret_core::PointerEvent::Move { .. } => {
                        cx.set_cursor_icon(CursorIcon::Pointer);
                    }
                    fret_core::PointerEvent::Down { button, .. } => {
                        if *button != MouseButton::Left {
                            return;
                        }
                        cx.request_focus(cx.node);
                        cx.capture_pointer(cx.node);
                        crate::elements::set_pressed_pressable(
                            &mut *cx.app,
                            window,
                            Some(self.element),
                        );
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    fret_core::PointerEvent::Up { button, .. } => {
                        if *button != MouseButton::Left {
                            return;
                        }
                        cx.release_pointer_capture();
                        crate::elements::set_pressed_pressable(&mut *cx.app, window, None);

                        let hovered = crate::elements::is_hovered_pressable(
                            &mut *cx.app,
                            window,
                            self.element,
                        );

                        if hovered {
                            if let Some(command) = props.on_click.clone() {
                                cx.dispatch_command(command);
                            }
                        }
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        if let Some(blob) = self.text_cache.blob.take() {
            text.release(blob);
        }
        self.text_cache.prepared_scale_factor_bits = None;
        self.text_cache.metrics = None;
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };
        if matches!(instance, ElementInstance::Text(_)) {
            cx.set_role(SemanticsRole::Text);
        }
        if matches!(instance, ElementInstance::Pressable(_)) {
            cx.set_role(SemanticsRole::Button);
        }
        if matches!(instance, ElementInstance::VirtualList(_)) {
            cx.set_role(SemanticsRole::List);
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return Size::new(Px(0.0), Px(0.0));
        };

        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return Size::new(Px(0.0), Px(0.0));
        };

        match instance {
            ElementInstance::Container(props) => {
                let pad_x = props.padding_x.0.max(0.0);
                let pad_y = props.padding_y.0.max(0.0);
                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_x),
                    Px(cx.bounds.origin.y.0 + pad_y),
                );
                let inner_size = Size::new(
                    Px((cx.bounds.size.width.0 - pad_x * 2.0).max(0.0)),
                    Px((cx.bounds.size.height.0 - pad_y * 2.0).max(0.0)),
                );
                let inner_bounds = Rect::new(inner_origin, inner_size);

                for &child in cx.children {
                    let _ = cx.layout_in(child, inner_bounds);
                }
                cx.available
            }
            ElementInstance::Pressable(_) | ElementInstance::Stack(_) => {
                for &child in cx.children {
                    let _ = cx.layout_in(child, cx.bounds);
                }
                cx.available
            }
            ElementInstance::Column(props) => {
                let pad = props.padding.0.max(0.0);
                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad),
                    Px(cx.bounds.origin.y.0 + pad),
                );
                let inner_width = Px((cx.available.width.0 - pad * 2.0).max(0.0));

                let mut y = inner_origin.y;
                let mut content_height = Px(0.0);

                for (i, &child) in cx.children.iter().enumerate() {
                    if i > 0 {
                        y = Px(y.0 + props.spacing.0.max(0.0));
                        content_height = Px(content_height.0 + props.spacing.0.max(0.0));
                    }

                    let child_origin = fret_core::Point::new(inner_origin.x, y);
                    let child_bounds = Rect::new(child_origin, Size::new(inner_width, Px(1.0e9)));
                    let child_size = cx.layout_in(child, child_bounds);
                    let child_h = child_size.height;

                    let final_bounds = Rect::new(child_origin, Size::new(inner_width, child_h));
                    let _ = cx.layout_in(child, final_bounds);

                    y = Px(y.0 + child_h.0);
                    content_height = Px(content_height.0 + child_h.0);
                }

                let total_h = Px(content_height.0 + pad * 2.0);
                Size::new(cx.available.width, total_h)
            }
            ElementInstance::Row(props) => {
                let pad = props.padding.0.max(0.0);
                let spacing = props.spacing.0.max(0.0);

                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad),
                    Px(cx.bounds.origin.y.0 + pad),
                );
                let inner_width = Px((cx.available.width.0 - pad * 2.0).max(0.0));

                let mut remaining_w = inner_width.0;
                let mut max_h = 0.0f32;

                let mut placements: Vec<(NodeId, fret_core::Point, Size)> = Vec::new();
                let mut x = inner_origin.x.0;

                for (i, &child) in cx.children.iter().enumerate() {
                    if i > 0 {
                        x += spacing;
                        remaining_w = (remaining_w - spacing).max(0.0);
                    }

                    let is_last = i + 1 == cx.children.len();
                    let probe_bounds = Rect::new(
                        fret_core::Point::new(Px(x), inner_origin.y),
                        Size::new(Px(remaining_w), Px(1.0e9)),
                    );
                    let child_size = cx.layout_in(child, probe_bounds);

                    let w = if is_last {
                        Px(remaining_w)
                    } else {
                        Px(child_size.width.0.min(remaining_w))
                    };
                    let size = Size::new(w, child_size.height);
                    placements.push((child, fret_core::Point::new(Px(x), inner_origin.y), size));

                    x += w.0;
                    remaining_w = (remaining_w - w.0).max(0.0);
                    max_h = max_h.max(child_size.height.0);
                }

                for (child, origin, size) in placements {
                    let dy = (max_h - size.height.0).max(0.0) * 0.5;
                    let child_origin = fret_core::Point::new(origin.x, Px(origin.y.0 + dy));
                    let bounds = Rect::new(child_origin, Size::new(size.width, Px(max_h)));
                    let _ = cx.layout_in(child, bounds);
                }

                let total_h = if cx.children.is_empty() {
                    Px(0.0)
                } else {
                    Px(max_h + pad * 2.0)
                };
                let total_w = if cx.children.is_empty() {
                    Px(0.0)
                } else {
                    Px(inner_width.0 + pad * 2.0)
                };

                Size::new(Px(total_w.0.min(cx.available.width.0)), total_h)
            }
            ElementInstance::Text(props) => {
                let theme_revision = cx.theme().revision();
                let font_size = cx
                    .theme()
                    .metric_by_key("font.size")
                    .unwrap_or(cx.theme().metrics.font_size);
                let style = props.style.unwrap_or(TextStyle {
                    font: FontId::default(),
                    size: font_size,
                });
                let constraints = TextConstraints {
                    max_width: Some(cx.available.width),
                    wrap: props.wrap,
                    scale_factor: cx.scale_factor,
                };
                let metrics = cx.text.measure(&props.text, style, constraints);

                self.text_cache.metrics = Some(metrics);
                self.text_cache.last_text = Some(props.text.clone());
                self.text_cache.last_style = Some(style);
                self.text_cache.last_wrap = Some(props.wrap);
                self.text_cache.last_width = Some(cx.available.width);
                self.text_cache.last_theme_revision = Some(theme_revision);

                metrics.size
            }
            ElementInstance::VirtualList(props) => {
                let mut needs_redraw = false;
                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::VirtualListState::default,
                    |state| {
                        let prev_viewport_h = state.viewport_h;
                        let prev_offset_y = state.offset_y;

                        let viewport_h = Px(cx.bounds.size.height.0.max(0.0));
                        if state.viewport_h != viewport_h {
                            state.viewport_h = viewport_h;
                        }

                        let row_h = Px(props.row_height.0.max(0.0));
                        let content_h = Px(row_h.0 * props.len as f32);
                        let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
                        state.offset_y = Px(state.offset_y.0.max(0.0).min(max_offset.0));

                        needs_redraw =
                            state.viewport_h != prev_viewport_h || state.offset_y != prev_offset_y;
                    },
                );
                if needs_redraw && let Some(window) = cx.window {
                    cx.app.request_redraw(window);
                }

                let offset_y = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::VirtualListState::default,
                    |state| state.offset_y,
                );

                let start = props.visible_start;
                let row_h = Px(props.row_height.0.max(0.0));

                for (i, &child) in cx.children.iter().enumerate() {
                    let idx = start + i;
                    let y = cx.bounds.origin.y.0 + row_h.0 * idx as f32 - offset_y.0;
                    let child_bounds = Rect::new(
                        fret_core::Point::new(cx.bounds.origin.x, Px(y)),
                        Size::new(cx.bounds.size.width, row_h),
                    );
                    let _ = cx.layout_in(child, child_bounds);
                }

                cx.available
            }
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return;
        };

        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };

        match instance {
            ElementInstance::Container(props) => {
                let should_draw = props.background.is_some()
                    || props.border_color.is_some()
                    || props.border != Edges::all(Px(0.0));

                if should_draw {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: cx.bounds,
                        background: props.background.unwrap_or(Color::TRANSPARENT),
                        border: props.border,
                        border_color: props.border_color.unwrap_or(Color::TRANSPARENT),
                        corner_radii: props.corner_radii,
                    });
                }

                for &child in cx.children {
                    if let Some(bounds) = cx.child_bounds(child) {
                        cx.paint(child, bounds);
                    } else {
                        cx.paint(child, cx.bounds);
                    }
                }
            }
            ElementInstance::Pressable(_)
            | ElementInstance::Stack(_)
            | ElementInstance::Column(_)
            | ElementInstance::Row(_) => {
                for &child in cx.children {
                    if let Some(bounds) = cx.child_bounds(child) {
                        cx.paint(child, bounds);
                    } else {
                        cx.paint(child, cx.bounds);
                    }
                }
            }
            ElementInstance::Text(props) => {
                let theme_revision = cx.theme().revision();
                let font_size = cx
                    .theme()
                    .metric_by_key("font.size")
                    .unwrap_or(cx.theme().metrics.font_size);
                let style = props.style.unwrap_or(TextStyle {
                    font: FontId::default(),
                    size: font_size,
                });
                let color = props
                    .color
                    .or_else(|| cx.theme().color_by_key("foreground"))
                    .unwrap_or(cx.theme().colors.text_primary);
                let constraints = TextConstraints {
                    max_width: Some(cx.bounds.size.width),
                    wrap: props.wrap,
                    scale_factor: cx.scale_factor,
                };

                let scale_bits = cx.scale_factor.to_bits();
                let needs_prepare = self.text_cache.blob.is_none()
                    || self.text_cache.prepared_scale_factor_bits != Some(scale_bits)
                    || self.text_cache.last_text.as_ref() != Some(&props.text)
                    || self.text_cache.last_style.as_ref() != Some(&style)
                    || self.text_cache.last_wrap != Some(props.wrap)
                    || self.text_cache.last_width != Some(cx.bounds.size.width)
                    || self.text_cache.last_theme_revision != Some(theme_revision);

                if needs_prepare {
                    if let Some(blob) = self.text_cache.blob.take() {
                        cx.text.release(blob);
                    }
                    let (blob, metrics) = cx.text.prepare(&props.text, style, constraints);
                    self.text_cache.blob = Some(blob);
                    self.text_cache.metrics = Some(metrics);
                    self.text_cache.prepared_scale_factor_bits = Some(scale_bits);
                    self.text_cache.last_text = Some(props.text.clone());
                    self.text_cache.last_style = Some(style);
                    self.text_cache.last_wrap = Some(props.wrap);
                    self.text_cache.last_width = Some(cx.bounds.size.width);
                    self.text_cache.last_theme_revision = Some(theme_revision);
                }

                let Some(blob) = self.text_cache.blob else {
                    return;
                };
                let Some(metrics) = self.text_cache.metrics else {
                    return;
                };

                let origin = fret_core::Point::new(
                    cx.bounds.origin.x,
                    cx.bounds.origin.y + metrics.baseline,
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin,
                    text: blob,
                    color,
                });
            }
            ElementInstance::VirtualList(_) => {
                cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
                for &child in cx.children {
                    if let Some(bounds) = cx.child_bounds(child) {
                        cx.paint(child, bounds);
                    }
                }
                cx.scene.push(SceneOp::PopClip);
            }
        }
    }
}

/// Render a declarative element tree into an existing `UiTree` root.
///
/// Call this once per frame *before* `layout_all`/`paint_all`, for the relevant window.
pub fn render_root<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    text: &mut dyn fret_core::TextService,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> NodeId {
    let frame_id = app.frame_id();

    let children = crate::elements::with_element_cx(app, window, bounds, root_name, render);

    app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let lag = runtime.gc_lag_frames();
        let cutoff = frame_id.0.saturating_sub(lag);

        let window_state = runtime.for_window_mut(window);
        let root_id = crate::elements::global_root(window, root_name);

        let root_node = window_state
            .node_entry(root_id)
            .map(|e| e.node)
            .unwrap_or_else(|| {
                let node = ui.create_node(ElementHostWidget {
                    element: root_id,
                    text_cache: TextCache::default(),
                });
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

        window_state.set_node_entry(
            root_id,
            NodeEntry {
                node: root_node,
                last_seen_frame: frame_id,
                root: root_id,
            },
        );

        app.with_global_mut(ElementFrame::default, |frame, _app| {
            let w = frame.windows.entry(window).or_default();
            w.instances.clear();
            w.instances.insert(
                root_node,
                ElementRecord {
                    element: root_id,
                    instance: ElementInstance::Stack(StackProps::default()),
                },
            );
        });

        let mut mounted_children: Vec<NodeId> = Vec::with_capacity(children.len());
        for child in children {
            mounted_children.push(mount_element(
                ui,
                app,
                window,
                root_id,
                frame_id,
                window_state,
                child,
            ));
        }
        ui.set_children(root_node, mounted_children);

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
            let _ = ui.remove_subtree(text, node);
        }

        root_node
    })
}

fn mount_element<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    window: AppWindowId,
    root_id: GlobalElementId,
    frame_id: fret_core::FrameId,
    window_state: &mut crate::elements::WindowElementState,
    element: AnyElement,
) -> NodeId {
    let id = element.id;
    let node = window_state
        .node_entry(id)
        .map(|e| e.node)
        .unwrap_or_else(|| {
            let node = ui.create_node(ElementHostWidget {
                element: id,
                text_cache: TextCache::default(),
            });
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
        ElementKind::Pressable(p) => ElementInstance::Pressable(p),
        ElementKind::Stack(p) => ElementInstance::Stack(p),
        ElementKind::Column(p) => ElementInstance::Column(p),
        ElementKind::Row(p) => ElementInstance::Row(p),
        ElementKind::Text(p) => ElementInstance::Text(p),
        ElementKind::VirtualList(p) => ElementInstance::VirtualList(p),
    };

    app.with_global_mut(ElementFrame::default, |frame, _app| {
        frame.windows.entry(window).or_default().instances.insert(
            node,
            ElementRecord {
                element: id,
                instance,
            },
        );
    });

    let mut child_nodes: Vec<NodeId> = Vec::with_capacity(element.children.len());
    for child in element.children {
        child_nodes.push(mount_element(
            ui,
            app,
            window,
            root_id,
            frame_id,
            window_state,
            child,
        ));
    }
    ui.set_children(node, child_nodes);

    node
}

#[cfg(test)]
mod tests {
    use super::render_root;
    use crate::UiHost;
    use crate::elements::ElementCx;
    use crate::test_host::TestHost;
    use crate::tree::UiTree;
    use fret_core::{
        AppWindowId, Color, Modifiers, MouseButton, MouseButtons, Point, Px, Rect, Scene, SceneOp,
        Size, TextConstraints, TextMetrics, TextService, TextStyle,
    };
    use fret_runtime::{CommandId, Effect};

    #[derive(Default)]
    struct FakeTextService;

    impl TextService for FakeTextService {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    #[test]
    fn keyed_elements_reuse_node_ids_across_reorder() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(500.0)),
        );
        let mut text = FakeTextService::default();

        let mut items: Vec<u64> = vec![1, 2, 3];
        let mut ids: Vec<(u64, crate::elements::GlobalElementId)> = Vec::new();

        let mut prev: std::collections::HashMap<
            u64,
            (crate::elements::GlobalElementId, fret_core::NodeId),
        > = std::collections::HashMap::new();

        let mut root: Option<fret_core::NodeId> = None;

        for pass in 0..2 {
            ids.clear();
            let r = render_root(
                &mut ui,
                &mut app,
                &mut text,
                window,
                bounds,
                "mvp49",
                |cx| build_keyed_rows(cx, &items, &mut ids),
            );
            root.get_or_insert(r);

            let cur: std::collections::HashMap<
                u64,
                (crate::elements::GlobalElementId, fret_core::NodeId),
            > = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
                runtime.prepare_window_for_frame(window, app.frame_id());
                let st = runtime.for_window_mut(window);
                ids.iter()
                    .map(|(item, id)| (*item, (*id, st.node_entry(*id).unwrap().node)))
                    .collect()
            });

            if pass == 1 {
                for item in [1u64, 2u64, 3u64] {
                    let (prev_id, prev_node) = prev.get(&item).copied().unwrap();
                    let (cur_id, cur_node) = cur.get(&item).copied().unwrap();
                    assert_eq!(
                        prev_id, cur_id,
                        "element id should be stable for item {item}"
                    );
                    assert_eq!(
                        prev_node, cur_node,
                        "node id should be stable for item {item}"
                    );
                }
            }

            prev = cur;
            items.reverse();
            app.advance_frame();
        }

        assert_eq!(ui.children(root.unwrap()).len(), 3);
    }

    #[test]
    fn stale_nodes_are_swept_after_gc_lag() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(500.0)),
        );
        let mut text = FakeTextService::default();

        let mut ids: Vec<(u64, crate::elements::GlobalElementId)> = Vec::new();
        let _root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp49-sweep",
            |cx| build_keyed_rows(cx, &[1u64, 2u64], &mut ids),
        );

        let node_to_remove =
            app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
                runtime.prepare_window_for_frame(window, app.frame_id());
                let st = runtime.for_window_mut(window);
                st.node_entry(ids[1].1).unwrap().node
            });

        // Remove item 2 from the render output, but it should not be swept immediately.
        app.advance_frame();
        let _ = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp49-sweep",
            |cx| build_keyed_rows(cx, &[1u64], &mut Vec::new()),
        );
        assert!(ui.debug_node_bounds(node_to_remove).is_some());

        // Advance frames until the GC lag is exceeded, then render again to trigger the sweep.
        app.advance_frame();
        let _ = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp49-sweep",
            |cx| build_keyed_rows(cx, &[1u64], &mut Vec::new()),
        );
        assert!(ui.debug_node_bounds(node_to_remove).is_some());

        app.advance_frame();
        let _ = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp49-sweep",
            |cx| build_keyed_rows(cx, &[1u64], &mut Vec::new()),
        );
        assert!(ui.debug_node_bounds(node_to_remove).is_none());
    }

    #[track_caller]
    fn build_keyed_rows(
        cx: &mut ElementCx<'_, TestHost>,
        items: &[u64],
        ids: &mut Vec<(u64, crate::elements::GlobalElementId)>,
    ) -> Vec<crate::element::AnyElement> {
        let mut out = Vec::new();
        for &item in items {
            let el = cx.keyed(item, |cx| cx.text("row"));
            ids.push((item, el.id));
            out.push(el);
        }
        out
    }

    #[test]
    fn virtual_list_computes_visible_range_after_first_layout() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(50.0)),
        );
        let mut text = FakeTextService::default();
        let mut list_element_id: Option<crate::elements::GlobalElementId> = None;

        fn build_list(
            cx: &mut ElementCx<'_, TestHost>,
            list_element_id: &mut Option<crate::elements::GlobalElementId>,
        ) -> crate::element::AnyElement {
            let list = cx.virtual_list(100, Px(10.0), 0, |cx, range| {
                range.map(|i| cx.keyed(i, |cx| cx.text("row"))).collect()
            });
            *list_element_id = Some(list.id);
            list
        }

        // Frame 0: no viewport height is known yet (it is written during layout), so the list
        // renders with an empty visible range.
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist",
            |cx| vec![build_list(cx, &mut list_element_id)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let list_node = ui.children(root)[0];
        assert_eq!(ui.children(list_node).len(), 0);
        let state = crate::elements::with_element_state(
            &mut app,
            window,
            list_element_id.unwrap(),
            crate::element::VirtualListState::default,
            |s| *s,
        );
        assert_eq!(state.viewport_h, Px(50.0));

        // Frame 1: the list has recorded its viewport height during layout, so the authoring layer
        // can compute a visible range and mount only the visible children.
        app.advance_frame();
        let prev_list_element_id = list_element_id;
        let mut list_element_id: Option<crate::elements::GlobalElementId> = None;
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist",
            |cx| vec![build_list(cx, &mut list_element_id)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        assert_eq!(
            prev_list_element_id, list_element_id,
            "virtual list element id should be stable across frames"
        );

        let list_node = ui.children(root)[0];
        let props = app.with_global_mut(super::ElementFrame::default, |frame, _app| {
            frame
                .windows
                .get(&window)
                .and_then(|w| w.instances.get(&list_node))
                .cloned()
        });
        let super::ElementInstance::VirtualList(props) =
            props.expect("list instance exists").instance
        else {
            panic!("expected VirtualList instance");
        };
        assert_eq!((props.visible_start, props.visible_end), (0, 5));
        assert_eq!(ui.children(list_node).len(), 5);
    }

    #[test]
    fn container_applies_padding_and_paints_background() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-container",
            |cx| {
                vec![cx.container(
                    crate::element::ContainerProps {
                        padding_x: Px(4.0),
                        padding_y: Px(6.0),
                        background: Some(Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("hi")],
                )]
            },
        );
        ui.set_root(root);

        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let container_node = ui.children(root)[0];
        let text_node = ui.children(container_node)[0];
        let text_bounds = ui.debug_node_bounds(text_node).expect("text bounds");
        assert_eq!(text_bounds.origin.x, Px(4.0));
        assert_eq!(text_bounds.origin.y, Px(6.0));
        assert_eq!(text_bounds.size.width, Px(92.0));
        assert_eq!(text_bounds.size.height, Px(28.0));

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

        assert_eq!(scene.ops_len(), 2);
        match scene.ops()[0] {
            SceneOp::Quad {
                rect, background, ..
            } => {
                assert_eq!(rect, bounds);
                assert_eq!(background.a, 1.0);
            }
            _ => panic!("expected quad op"),
        }
    }

    #[test]
    fn pressable_dispatches_click_command_when_released_over_self() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
        let mut text = FakeTextService::default();

        let command = CommandId::from("test.click");

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-pressable",
            |cx| {
                vec![cx.pressable(
                    crate::element::PressableProps {
                        enabled: true,
                        on_click: Some(command.clone()),
                    },
                    |cx, _state| {
                        vec![cx.container(
                            crate::element::ContainerProps {
                                padding_x: Px(4.0),
                                padding_y: Px(4.0),
                                ..Default::default()
                            },
                            |cx| vec![cx.text("hi")],
                        )]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let pressable_node = ui.children(root)[0];
        let pressable_bounds = ui
            .debug_node_bounds(pressable_node)
            .expect("pressable bounds");
        let position = Point::new(
            Px(pressable_bounds.origin.x.0 + 10.0),
            Px(pressable_bounds.origin.y.0 + 10.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        let effects = app.take_effects();
        assert!(
            effects.iter().any(
                |e| matches!(e, Effect::Command { command: c, .. } if c.as_str() == "test.click")
            ),
            "expected Effect::Command(test.click), got {effects:?}"
        );

        // Sanity: move outside should clear hover state for future interactions.
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(200.0), Px(200.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );
    }
}
