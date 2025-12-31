use crate::UiHost;
use crate::element::{
    AnyElement, ContainerProps, CrossAlign, ElementKind, FlexProps, HoverRegionProps, LayoutStyle,
    Length, MainAlign, Overflow, PointerRegionProps, PressableProps, SpacerProps, SpinnerProps,
    StackProps, TextProps, VisualTransformProps,
};
use crate::elements::{ElementCx, GlobalElementId, NodeEntry};
use crate::text_input::BoundTextInput;
use crate::tree::UiTree;
use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget};
use crate::{
    action,
    action::{ActivateReason, DismissReason, KeyDownCx},
};
use fret_core::{
    AppWindowId, Color, CursorIcon, DrawOrder, Edges, Event, FontId, FrameId, MouseButton, NodeId,
    Point, Px, Rect, SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow,
    TextStyle, Transform2D,
};
use fret_runtime::Effect;
use std::collections::HashMap;
use taffy::{
    TaffyTree,
    geometry::{Line as TaffyLine, Rect as TaffyRect, Size as TaffySize},
    style::{
        AlignItems as TaffyAlignItems, AlignSelf as TaffyAlignSelf,
        AvailableSpace as TaffyAvailableSpace, Dimension, Display, FlexDirection, FlexWrap,
        GridPlacement, JustifyContent, LengthPercentage, LengthPercentageAuto,
        Position as TaffyPosition, Style as TaffyStyle,
    },
    tree::NodeId as TaffyNodeId,
};

mod host_widget;
use host_widget::ElementHostWidget;
mod taffy_layout;
use taffy_layout::*;

fn scrollbar_thumb_rect(track: Rect, viewport_h: Px, content_h: Px, offset_y: Px) -> Option<Rect> {
    let viewport_h = Px(viewport_h.0.max(0.0));
    let content_h = Px(content_h.0.max(0.0));
    let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
    if max_offset.0 <= 0.0 || track.size.height.0 <= 0.0 {
        return None;
    }

    let track_h = track.size.height.0;
    let min_thumb_h = 16.0f32.min(track_h);
    let ratio = (viewport_h.0 / content_h.0).clamp(0.0, 1.0);
    let thumb_h = (track_h * ratio).max(min_thumb_h).min(track_h);
    let max_thumb_y = (track_h - thumb_h).max(0.0);

    let t = (offset_y.0.max(0.0).min(max_offset.0)) / max_offset.0;
    let y = track.origin.y.0 + max_thumb_y * t;

    Some(Rect::new(
        fret_core::Point::new(track.origin.x, Px(y)),
        Size::new(track.size.width, Px(thumb_h)),
    ))
}

fn paint_children_clipped_if<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    clip: bool,
    corner_radii: Option<fret_core::Corners>,
) {
    if clip {
        if let Some(radii) = corner_radii
            && (radii.top_left.0 > 0.0
                || radii.top_right.0 > 0.0
                || radii.bottom_right.0 > 0.0
                || radii.bottom_left.0 > 0.0)
        {
            cx.scene.push(SceneOp::PushClipRRect {
                rect: cx.bounds,
                corner_radii: radii,
            });
        } else {
            cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
        }
    }

    for &child in cx.children {
        if let Some(bounds) = cx.child_bounds(child) {
            cx.paint(child, bounds);
        } else {
            cx.paint(child, cx.bounds);
        }
    }

    if clip {
        cx.scene.push(SceneOp::PopClip);
    }
}

#[derive(Debug, Clone, Copy)]
enum PositionedLayoutStyle {
    Static,
    Relative(crate::element::InsetStyle),
    Absolute(crate::element::InsetStyle),
}

fn positioned_layout_style(layout: LayoutStyle) -> PositionedLayoutStyle {
    match layout.position {
        crate::element::PositionStyle::Static => PositionedLayoutStyle::Static,
        crate::element::PositionStyle::Relative => PositionedLayoutStyle::Relative(layout.inset),
        crate::element::PositionStyle::Absolute => PositionedLayoutStyle::Absolute(layout.inset),
    }
}

fn layout_positioned_child<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    child: NodeId,
    base: Rect,
    style: PositionedLayoutStyle,
) {
    match style {
        PositionedLayoutStyle::Static => {
            let _ = cx.layout_in(child, base);
        }
        PositionedLayoutStyle::Relative(inset) => {
            let dx = inset.left.unwrap_or(Px(0.0)).0 - inset.right.unwrap_or(Px(0.0)).0;
            let dy = inset.top.unwrap_or(Px(0.0)).0 - inset.bottom.unwrap_or(Px(0.0)).0;
            let origin = fret_core::Point::new(Px(base.origin.x.0 + dx), Px(base.origin.y.0 + dy));
            let _ = cx.layout_in(child, Rect::new(origin, base.size));
        }
        PositionedLayoutStyle::Absolute(inset) => {
            let measured = cx.layout_in(child, base);

            let left = inset.left.unwrap_or(Px(0.0));
            let right = inset.right.unwrap_or(Px(0.0));
            let top = inset.top.unwrap_or(Px(0.0));
            let bottom = inset.bottom.unwrap_or(Px(0.0));

            let w = if inset.left.is_some() && inset.right.is_some() {
                Px((base.size.width.0 - left.0 - right.0).max(0.0))
            } else {
                Px(measured.width.0.min(base.size.width.0.max(0.0)).max(0.0))
            };
            let h = if inset.top.is_some() && inset.bottom.is_some() {
                Px((base.size.height.0 - top.0 - bottom.0).max(0.0))
            } else {
                Px(measured.height.0.min(base.size.height.0.max(0.0)).max(0.0))
            };

            let x = if inset.left.is_some() {
                left
            } else if inset.right.is_some() {
                Px((base.size.width.0 - right.0 - w.0).max(0.0))
            } else {
                Px(0.0)
            };
            let y = if inset.top.is_some() {
                top
            } else if inset.bottom.is_some() {
                Px((base.size.height.0 - bottom.0 - h.0).max(0.0))
            } else {
                Px(0.0)
            };

            let origin =
                fret_core::Point::new(Px(base.origin.x.0 + x.0), Px(base.origin.y.0 + y.0));
            let _ = cx.layout_in(child, Rect::new(origin, Size::new(w, h)));
        }
    }
}

#[derive(Default)]
pub(crate) struct ElementFrame {
    windows: HashMap<AppWindowId, WindowFrame>,
}

pub(crate) struct WindowFrame {
    frame_id: FrameId,
    pub(crate) instances: HashMap<NodeId, ElementRecord>,
}

impl Default for WindowFrame {
    fn default() -> Self {
        Self {
            frame_id: FrameId(0),
            instances: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ElementInstance {
    Container(ContainerProps),
    Semantics(crate::element::SemanticsProps),
    Opacity(crate::element::OpacityProps),
    VisualTransform(VisualTransformProps),
    Pressable(PressableProps),
    PointerRegion(PointerRegionProps),
    DismissibleLayer(DismissibleLayerProps),
    RovingFlex(crate::element::RovingFlexProps),
    Stack(StackProps),
    Spacer(SpacerProps),
    Text(TextProps),
    TextInput(crate::element::TextInputProps),
    TextArea(crate::element::TextAreaProps),
    ResizablePanelGroup(crate::element::ResizablePanelGroupProps),
    VirtualList(crate::element::VirtualListProps),
    Flex(FlexProps),
    Grid(crate::element::GridProps),
    Image(crate::element::ImageProps),
    SvgIcon(crate::element::SvgIconProps),
    Spinner(SpinnerProps),
    HoverRegion(HoverRegionProps),
    Scroll(crate::element::ScrollProps),
    Scrollbar(crate::element::ScrollbarProps),
}

#[derive(Debug, Clone)]
pub(crate) struct ElementRecord {
    pub element: GlobalElementId,
    pub instance: ElementInstance,
}

#[derive(Clone)]
pub(crate) struct DismissibleLayerProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
}

impl std::fmt::Debug for DismissibleLayerProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = f.debug_struct("DismissibleLayerProps");
        out.field("layout", &self.layout)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl Default for DismissibleLayerProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        Self {
            layout,
            enabled: true,
        }
    }
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

fn layout_style_for_node<H: UiHost>(app: &mut H, window: AppWindowId, node: NodeId) -> LayoutStyle {
    element_record_for_node(app, window, node)
        .map(|r| match r.instance {
            ElementInstance::Container(p) => p.layout,
            ElementInstance::Semantics(p) => p.layout,
            ElementInstance::Opacity(p) => p.layout,
            ElementInstance::VisualTransform(p) => p.layout,
            ElementInstance::Pressable(p) => p.layout,
            ElementInstance::PointerRegion(p) => p.layout,
            ElementInstance::DismissibleLayer(p) => p.layout,
            ElementInstance::RovingFlex(p) => p.flex.layout,
            ElementInstance::Stack(p) => p.layout,
            ElementInstance::Spacer(p) => p.layout,
            ElementInstance::Text(p) => p.layout,
            ElementInstance::TextInput(p) => p.layout,
            ElementInstance::TextArea(p) => p.layout,
            ElementInstance::ResizablePanelGroup(p) => p.layout,
            ElementInstance::VirtualList(p) => p.layout,
            ElementInstance::Flex(p) => p.layout,
            ElementInstance::Grid(p) => p.layout,
            ElementInstance::Image(p) => p.layout,
            ElementInstance::SvgIcon(p) => p.layout,
            ElementInstance::Spinner(p) => p.layout,
            ElementInstance::HoverRegion(p) => p.layout,
            ElementInstance::Scroll(p) => p.layout,
            ElementInstance::Scrollbar(p) => p.layout,
        })
        .unwrap_or_default()
}

fn clamp_to_constraints(mut size: Size, style: LayoutStyle, available: Size) -> Size {
    let width_auto = matches!(style.size.width, Length::Auto);
    let height_auto = matches!(style.size.height, Length::Auto);

    match style.size.width {
        Length::Px(px) => size.width = Px(px.0.max(0.0)),
        Length::Fill => size.width = available.width,
        Length::Auto => {}
    }
    match style.size.height {
        Length::Px(px) => size.height = Px(px.0.max(0.0)),
        Length::Fill => size.height = available.height,
        Length::Auto => {}
    }

    if let Some(min_w) = style.size.min_width {
        size.width = Px(size.width.0.max(min_w.0.max(0.0)));
    }
    if let Some(min_h) = style.size.min_height {
        size.height = Px(size.height.0.max(min_h.0.max(0.0)));
    }
    if let Some(max_w) = style.size.max_width {
        size.width = Px(size.width.0.min(max_w.0.max(0.0)));
    }
    if let Some(max_h) = style.size.max_height {
        size.height = Px(size.height.0.min(max_h.0.max(0.0)));
    }

    size.width = Px(size.width.0.max(0.0).min(available.width.0.max(0.0)));
    size.height = Px(size.height.0.max(0.0).min(available.height.0.max(0.0)));

    if let Some(ratio) = style.aspect_ratio
        && ratio.is_finite()
        && ratio > 0.0
    {
        if height_auto && !width_auto {
            size.height = Px((size.width.0 / ratio).max(0.0));
        } else if width_auto && !height_auto {
            size.width = Px((size.height.0 * ratio).max(0.0));
        }

        if let Some(min_w) = style.size.min_width {
            size.width = Px(size.width.0.max(min_w.0.max(0.0)));
        }
        if let Some(min_h) = style.size.min_height {
            size.height = Px(size.height.0.max(min_h.0.max(0.0)));
        }
        if let Some(max_w) = style.size.max_width {
            size.width = Px(size.width.0.min(max_w.0.max(0.0)));
        }
        if let Some(max_h) = style.size.max_height {
            size.height = Px(size.height.0.min(max_h.0.max(0.0)));
        }

        size.width = Px(size.width.0.max(0.0).min(available.width.0.max(0.0)));
        size.height = Px(size.height.0.max(0.0).min(available.height.0.max(0.0)));
    }
    size
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

fn node_for_element_in_window_frame<H: UiHost>(
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
    render: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> NodeId {
    let frame_id = app.frame_id();

    let children = crate::elements::with_element_cx(app, window, bounds, root_name, |cx| {
        cx.dismissible_clear_on_dismiss_request();
        render(cx)
    });

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
                let node = ui.create_node(ElementHostWidget::new(root_id));
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
                ));
            }
            ui.set_children(root_node, mounted_children);
        });

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
    render: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> NodeId {
    render_dismissible_root_impl(ui, app, services, window, bounds, root_name, render)
}

#[allow(clippy::too_many_arguments)]
fn render_dismissible_root_impl<H: UiHost, F: FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: F,
) -> NodeId {
    let frame_id = app.frame_id();

    let children = crate::elements::with_element_cx(app, window, bounds, root_name, |cx| {
        cx.dismissible_clear_on_dismiss_request();
        render(cx)
    });

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
                let node = ui.create_node(ElementHostWidget::new(root_id));
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
                ));
            }
            ui.set_children(root_node, mounted_children);
        });

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

fn mount_element<H: UiHost>(
    ui: &mut UiTree<H>,
    _window: AppWindowId,
    root_id: GlobalElementId,
    frame_id: fret_core::FrameId,
    window_state: &mut crate::elements::WindowElementState,
    window_frame: &mut WindowFrame,
    element: AnyElement,
) -> NodeId {
    let id = element.id;
    let node = window_state
        .node_entry(id)
        .map(|e| e.node)
        .unwrap_or_else(|| {
            let node = ui.create_node(ElementHostWidget::new(id));
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
        ElementKind::Opacity(p) => ElementInstance::Opacity(p),
        ElementKind::VisualTransform(p) => ElementInstance::VisualTransform(p),
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
        ElementKind::TextInput(p) => ElementInstance::TextInput(p),
        ElementKind::TextArea(p) => ElementInstance::TextArea(p),
        ElementKind::ResizablePanelGroup(p) => ElementInstance::ResizablePanelGroup(p),
        ElementKind::VirtualList(p) => ElementInstance::VirtualList(p),
        ElementKind::Flex(p) => ElementInstance::Flex(p),
        ElementKind::Grid(p) => ElementInstance::Grid(p),
        ElementKind::Image(p) => ElementInstance::Image(p),
        ElementKind::SvgIcon(p) => ElementInstance::SvgIcon(p),
        ElementKind::Spinner(p) => ElementInstance::Spinner(p),
        ElementKind::HoverRegion(p) => ElementInstance::HoverRegion(p),
        ElementKind::Scroll(p) => ElementInstance::Scroll(p),
        ElementKind::Scrollbar(p) => ElementInstance::Scrollbar(p),
    };

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
        ));
    }
    ui.set_children(node, child_nodes);

    node
}

#[cfg(test)]
mod tests;
