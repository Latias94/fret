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
mod mount;
pub(crate) use mount::{node_for_element_in_window_frame, with_window_frame};
pub use mount::{render_dismissible_root_with_hooks, render_root};

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

#[cfg(test)]
mod tests;
