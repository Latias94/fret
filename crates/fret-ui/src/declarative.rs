use crate::UiHost;
use crate::element::{
    AnyElement, ContainerProps, CrossAlign, ElementKind, FlexProps, HoverCardAlign, HoverCardProps,
    LayoutStyle, Length, MainAlign, Overflow, PressableProps, SpacerProps, SpinnerProps,
    StackProps, TextProps,
};
use crate::elements::{ElementCx, GlobalElementId, NodeEntry};
use crate::primitives::BoundTextInput;
use crate::tree::UiTree;
use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget};
use fret_core::{
    AppWindowId, Color, CursorIcon, DrawOrder, Edges, Event, FontId, FrameId, MouseButton, NodeId,
    Point, Px, Rect, SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow,
    TextStyle,
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

fn scrollbar_track_rect(bounds: Rect, scrollbar_w: Px) -> Option<Rect> {
    let w = Px(scrollbar_w.0.max(0.0).min(bounds.size.width.0.max(0.0)));
    if w.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
        return None;
    }
    Some(Rect::new(
        fret_core::Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0 - w.0),
            bounds.origin.y,
        ),
        Size::new(w, bounds.size.height),
    ))
}

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

fn paint_children_clipped_if<H: UiHost>(cx: &mut PaintCx<'_, H>, clip: bool) {
    if clip {
        cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
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

#[derive(Default)]
pub(crate) struct WindowFrame {
    pub(crate) instances: HashMap<NodeId, ElementRecord>,
}

#[derive(Debug, Clone)]
pub(crate) enum ElementInstance {
    Container(ContainerProps),
    Pressable(PressableProps),
    Stack(StackProps),
    Spacer(SpacerProps),
    Text(TextProps),
    TextInput(crate::element::TextInputProps),
    VirtualList(crate::element::VirtualListProps),
    Flex(FlexProps),
    Grid(crate::element::GridProps),
    Image(crate::element::ImageProps),
    Spinner(SpinnerProps),
    HoverCard(HoverCardProps),
    Scroll(crate::element::ScrollProps),
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

fn layout_style_for_node<H: UiHost>(app: &mut H, window: AppWindowId, node: NodeId) -> LayoutStyle {
    element_record_for_node(app, window, node)
        .map(|r| match r.instance {
            ElementInstance::Container(p) => p.layout,
            ElementInstance::Pressable(p) => p.layout,
            ElementInstance::Stack(p) => p.layout,
            ElementInstance::Spacer(p) => p.layout,
            ElementInstance::Text(p) => p.layout,
            ElementInstance::TextInput(p) => p.layout,
            ElementInstance::VirtualList(p) => p.layout,
            ElementInstance::Flex(p) => p.layout,
            ElementInstance::Grid(p) => p.layout,
            ElementInstance::Image(p) => p.layout,
            ElementInstance::Spinner(p) => p.layout,
            ElementInstance::HoverCard(p) => p.layout,
            ElementInstance::Scroll(p) => p.layout,
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

fn taffy_dimension(length: Length) -> Dimension {
    match length {
        Length::Auto => Dimension::auto(),
        Length::Fill => Dimension::percent(1.0),
        Length::Px(px) => Dimension::length(px.0),
    }
}

fn taffy_position(position: crate::element::PositionStyle) -> TaffyPosition {
    match position {
        crate::element::PositionStyle::Static | crate::element::PositionStyle::Relative => {
            TaffyPosition::Relative
        }
        crate::element::PositionStyle::Absolute => TaffyPosition::Absolute,
    }
}

fn taffy_lpa(px: Option<Px>) -> LengthPercentageAuto {
    match px {
        Some(px) => LengthPercentageAuto::length(px.0),
        None => LengthPercentageAuto::auto(),
    }
}

fn taffy_rect_lpa_from_inset(
    position: crate::element::PositionStyle,
    inset: crate::element::InsetStyle,
) -> TaffyRect<LengthPercentageAuto> {
    if position == crate::element::PositionStyle::Static {
        return TaffyRect {
            left: LengthPercentageAuto::auto(),
            right: LengthPercentageAuto::auto(),
            top: LengthPercentageAuto::auto(),
            bottom: LengthPercentageAuto::auto(),
        };
    }
    TaffyRect {
        left: taffy_lpa(inset.left),
        right: taffy_lpa(inset.right),
        top: taffy_lpa(inset.top),
        bottom: taffy_lpa(inset.bottom),
    }
}

fn taffy_rect_lpa_from_edges(edges: Edges) -> TaffyRect<LengthPercentageAuto> {
    TaffyRect {
        left: LengthPercentageAuto::length(edges.left.0),
        right: LengthPercentageAuto::length(edges.right.0),
        top: LengthPercentageAuto::length(edges.top.0),
        bottom: LengthPercentageAuto::length(edges.bottom.0),
    }
}

fn taffy_grid_line(line: crate::element::GridLine) -> TaffyLine<GridPlacement> {
    let start = line
        .start
        .map(|s| taffy::style_helpers::line::<GridPlacement>(s))
        .unwrap_or(GridPlacement::Auto);
    let end = line
        .span
        .map(GridPlacement::Span)
        .unwrap_or(GridPlacement::Auto);
    TaffyLine { start, end }
}

fn taffy_align_items(align: CrossAlign) -> TaffyAlignItems {
    match align {
        CrossAlign::Start => TaffyAlignItems::FlexStart,
        CrossAlign::Center => TaffyAlignItems::Center,
        CrossAlign::End => TaffyAlignItems::FlexEnd,
        CrossAlign::Stretch => TaffyAlignItems::Stretch,
    }
}

fn taffy_align_self(align: CrossAlign) -> TaffyAlignSelf {
    match align {
        CrossAlign::Start => TaffyAlignSelf::FlexStart,
        CrossAlign::Center => TaffyAlignSelf::Center,
        CrossAlign::End => TaffyAlignSelf::FlexEnd,
        CrossAlign::Stretch => TaffyAlignSelf::Stretch,
    }
}

fn taffy_justify(justify: MainAlign) -> JustifyContent {
    match justify {
        MainAlign::Start => JustifyContent::FlexStart,
        MainAlign::Center => JustifyContent::Center,
        MainAlign::End => JustifyContent::FlexEnd,
        MainAlign::SpaceBetween => JustifyContent::SpaceBetween,
        MainAlign::SpaceAround => JustifyContent::SpaceAround,
        MainAlign::SpaceEvenly => JustifyContent::SpaceEvenly,
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

#[derive(Debug, Default, Clone)]
struct TextCache {
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<TextMetrics>,
    prepared_scale_factor_bits: Option<u32>,
    last_text: Option<std::sync::Arc<str>>,
    last_style: Option<TextStyle>,
    last_wrap: Option<fret_core::TextWrap>,
    last_overflow: Option<TextOverflow>,
    last_width: Option<Px>,
    last_theme_revision: Option<u64>,
}

#[derive(Debug, Default, Clone, Copy)]
struct HoverCardOpenState {
    open: bool,
    hover_start: Option<FrameId>,
    leave_start: Option<FrameId>,
}

struct ElementHostWidget {
    element: GlobalElementId,
    text_cache: TextCache,
    hit_testable: bool,
    hit_test_children: bool,
    is_focusable: bool,
    is_text_input: bool,
    clips_hit_test: bool,
    scrollbar_hit_rect: Option<Rect>,
    text_input: Option<BoundTextInput>,
    hover_card_open: bool,
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
    fn clips_hit_test(&self, _bounds: Rect) -> bool {
        self.clips_hit_test
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        self.hit_testable
    }

    fn hit_test_children(&self, _bounds: Rect, position: Point) -> bool {
        if !self.hit_test_children {
            return false;
        }
        if let Some(rect) = self.scrollbar_hit_rect
            && rect.contains(position)
        {
            return false;
        }
        true
    }

    fn is_focusable(&self) -> bool {
        self.is_focusable
    }

    fn is_text_input(&self) -> bool {
        self.is_text_input
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };

        match instance {
            ElementInstance::TextInput(props) => {
                if self.text_input.is_none() {
                    self.text_input = Some(BoundTextInput::new(props.model));
                }
                let input = self.text_input.as_mut().expect("text input");
                if input.model_id() != props.model.id() {
                    input.set_model(props.model);
                }
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);
                input.event(cx, event);
            }
            ElementInstance::VirtualList(props) => {
                let Event::Pointer(pe) = event else {
                    return;
                };
                match pe {
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
                }
            }
            ElementInstance::Scroll(props) => {
                let Event::Pointer(pe) = event else {
                    return;
                };
                match pe {
                    fret_core::PointerEvent::Wheel { delta, .. } => {
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollState::default,
                            |state| {
                                let viewport_h = Px(state.viewport_h.0.max(0.0));
                                let content_h = Px(state.content_h.0.max(0.0));
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
                    fret_core::PointerEvent::Move { position, .. } => {
                        let mut needs_layout = false;
                        let mut needs_paint = false;
                        let mut should_stop = false;

                        let scrollbar_w = props.show_scrollbar.then(|| {
                            let theme = cx.theme();
                            theme
                                .metric_by_key("metric.scrollbar.width")
                                .unwrap_or(theme.metrics.scrollbar_width)
                        });
                        let bounds = cx.bounds;
                        let position = *position;

                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollState::default,
                            |state| {
                                let viewport_h = Px(state.viewport_h.0.max(0.0));
                                let content_h = Px(state.content_h.0.max(0.0));
                                let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));

                                let track =
                                    scrollbar_w.and_then(|w| scrollbar_track_rect(bounds, w));
                                let hovered = track.is_some_and(|t| t.contains(position));

                                if state.hovered_scrollbar != hovered && !state.dragging_thumb {
                                    state.hovered_scrollbar = hovered;
                                    needs_paint = true;
                                }

                                if state.dragging_thumb && max_offset.0 > 0.0 {
                                    if let Some(track) = track
                                        && let Some(thumb) = scrollbar_thumb_rect(
                                            track,
                                            viewport_h,
                                            content_h,
                                            state.drag_start_offset_y,
                                        )
                                    {
                                        let max_thumb_y =
                                            (track.size.height.0 - thumb.size.height.0).max(0.0);
                                        if max_thumb_y > 0.0 {
                                            let delta_y =
                                                position.y.0 - state.drag_start_pointer_y.0;
                                            let scale = max_offset.0 / max_thumb_y;
                                            let next = Px((state.drag_start_offset_y.0
                                                + delta_y * scale)
                                                .max(0.0));
                                            let next = Px(next.0.min(max_offset.0));
                                            if state.offset_y != next {
                                                state.offset_y = next;
                                                needs_layout = true;
                                                needs_paint = true;
                                            }
                                            state.hovered_scrollbar = true;
                                            should_stop = true;
                                        }
                                    }
                                } else if hovered {
                                    should_stop = true;
                                }
                            },
                        );

                        if needs_layout {
                            cx.invalidate_self(Invalidation::Layout);
                        }
                        if needs_paint {
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                        if should_stop {
                            cx.stop_propagation();
                        }
                    }
                    fret_core::PointerEvent::Down {
                        position, button, ..
                    } => {
                        if *button != MouseButton::Left {
                            return;
                        }
                        cx.request_focus(cx.node);

                        if !props.show_scrollbar {
                            return;
                        }

                        let mut did_handle = false;
                        let mut did_start_drag = false;

                        let scrollbar_w = {
                            let theme = cx.theme();
                            theme
                                .metric_by_key("metric.scrollbar.width")
                                .unwrap_or(theme.metrics.scrollbar_width)
                        };
                        let bounds = cx.bounds;
                        let position = *position;

                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollState::default,
                            |state| {
                                let viewport_h = Px(state.viewport_h.0.max(0.0));
                                let content_h = Px(state.content_h.0.max(0.0));
                                let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
                                if max_offset.0 <= 0.0 {
                                    return;
                                }

                                let Some(track) = scrollbar_track_rect(bounds, scrollbar_w) else {
                                    return;
                                };
                                if !track.contains(position) {
                                    return;
                                }

                                let Some(thumb) = scrollbar_thumb_rect(
                                    track,
                                    viewport_h,
                                    content_h,
                                    state.offset_y,
                                ) else {
                                    return;
                                };

                                did_handle = true;
                                state.hovered_scrollbar = true;

                                if thumb.contains(position) {
                                    state.dragging_thumb = true;
                                    state.drag_start_pointer_y = position.y;
                                    state.drag_start_offset_y = state.offset_y;
                                    did_start_drag = true;
                                } else {
                                    // Page to the click position (center the thumb on the pointer).
                                    let max_thumb_y =
                                        (track.size.height.0 - thumb.size.height.0).max(0.0);
                                    if max_thumb_y > 0.0 {
                                        let click_y = (position.y.0 - track.origin.y.0)
                                            .clamp(0.0, track.size.height.0);
                                        let thumb_top = (click_y - thumb.size.height.0 * 0.5)
                                            .clamp(0.0, max_thumb_y);
                                        let t = thumb_top / max_thumb_y;
                                        state.offset_y =
                                            Px((max_offset.0 * t).clamp(0.0, max_offset.0));
                                    }
                                }
                            },
                        );

                        if did_handle {
                            if did_start_drag {
                                cx.capture_pointer(cx.node);
                            }
                            cx.invalidate_self(Invalidation::Layout);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                    fret_core::PointerEvent::Up { button, .. } => {
                        if *button != MouseButton::Left {
                            return;
                        }

                        let mut did_handle = false;
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollState::default,
                            |state| {
                                if state.dragging_thumb {
                                    did_handle = true;
                                    state.dragging_thumb = false;
                                }
                            },
                        );
                        if did_handle {
                            cx.release_pointer_capture();
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                }
            }
            ElementInstance::Pressable(props) => {
                if !props.enabled {
                    return;
                }
                match event {
                    Event::Pointer(pe) => match pe {
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
                    },
                    Event::KeyDown { key, repeat, .. } => {
                        if *repeat {
                            return;
                        }
                        if cx.focus != Some(cx.node) {
                            return;
                        }
                        if !matches!(
                            key,
                            fret_core::KeyCode::Enter
                                | fret_core::KeyCode::NumpadEnter
                                | fret_core::KeyCode::Space
                        ) {
                            return;
                        }
                        crate::elements::set_pressed_pressable(
                            &mut *cx.app,
                            window,
                            Some(self.element),
                        );
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    Event::KeyUp { key, .. } => {
                        if cx.focus != Some(cx.node) {
                            return;
                        }
                        if !matches!(
                            key,
                            fret_core::KeyCode::Enter
                                | fret_core::KeyCode::NumpadEnter
                                | fret_core::KeyCode::Space
                        ) {
                            return;
                        }
                        let pressed = crate::elements::is_pressed_pressable(
                            &mut *cx.app,
                            window,
                            self.element,
                        );
                        if !pressed {
                            return;
                        }
                        crate::elements::set_pressed_pressable(&mut *cx.app, window, None);
                        if let Some(command) = props.on_click.clone() {
                            cx.dispatch_command(command);
                        }
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        if let Some(blob) = self.text_cache.blob.take() {
            services.text().release(blob);
        }
        self.text_cache.prepared_scale_factor_bits = None;
        self.text_cache.metrics = None;
        if let Some(input) = self.text_input.as_mut() {
            input.cleanup_resources(services);
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };
        match instance {
            ElementInstance::Text(_) => {
                cx.set_role(SemanticsRole::Text);
            }
            ElementInstance::TextInput(props) => {
                if self.text_input.is_none() {
                    self.text_input = Some(BoundTextInput::new(props.model));
                }
                let input = self.text_input.as_mut().expect("text input");
                if input.model_id() != props.model.id() {
                    input.set_model(props.model);
                }
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);
                input.semantics(cx);
            }
            ElementInstance::Pressable(props) => {
                cx.set_role(SemanticsRole::Button);
                cx.set_disabled(!props.enabled);
            }
            ElementInstance::VirtualList(_) => {
                cx.set_role(SemanticsRole::List);
            }
            ElementInstance::Flex(_) | ElementInstance::Grid(_) => {
                // Flex/Grid are layout containers; they do not imply semantics beyond their children.
            }
            ElementInstance::Image(_)
            | ElementInstance::HoverCard(_)
            | ElementInstance::Spinner(_)
            | ElementInstance::Scroll(_) => {
                cx.set_role(SemanticsRole::Generic);
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return Size::new(Px(0.0), Px(0.0));
        };

        for (model, invalidation) in
            crate::elements::observed_models_for_element(cx.app, window, self.element)
        {
            (cx.observe_model)(model, invalidation);
        }

        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return Size::new(Px(0.0), Px(0.0));
        };

        self.hit_testable = match &instance {
            ElementInstance::Pressable(p) => p.enabled,
            ElementInstance::Spinner(_) => false,
            _ => true,
        };
        self.hit_test_children = match &instance {
            ElementInstance::Pressable(p) => p.enabled,
            ElementInstance::Spinner(_) => false,
            _ => true,
        };
        self.is_text_input = matches!(&instance, ElementInstance::TextInput(_));
        self.is_focusable = match &instance {
            ElementInstance::TextInput(_) => true,
            ElementInstance::Pressable(p) => p.enabled,
            _ => false,
        };
        self.clips_hit_test = match &instance {
            ElementInstance::Container(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Pressable(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Stack(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Flex(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Grid(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::TextInput(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Scroll(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::HoverCard(p) => matches!(p.layout.overflow, Overflow::Clip),
            // These primitives are always hit-test clipped by their own bounds (they are not
            // intended as overflow-visible containers).
            ElementInstance::VirtualList(_)
            | ElementInstance::Image(_)
            | ElementInstance::Spinner(_)
            | ElementInstance::Text(_) => true,
            ElementInstance::Spacer(_) => true,
        };
        self.scrollbar_hit_rect = None;

        match instance {
            ElementInstance::Container(props) => {
                let pad_left = props.padding.left.0.max(0.0);
                let pad_right = props.padding.right.0.max(0.0);
                let pad_top = props.padding.top.0.max(0.0);
                let pad_bottom = props.padding.bottom.0.max(0.0);
                let pad_w = pad_left + pad_right;
                let pad_h = pad_top + pad_bottom;

                let inner_avail = Size::new(
                    Px((cx.available.width.0 - pad_w).max(0.0)),
                    Px((cx.available.height.0 - pad_h).max(0.0)),
                );

                let probe_bounds =
                    Rect::new(cx.bounds.origin, Size::new(inner_avail.width, Px(1.0e9)));
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = Size::new(
                    Px((max_child.width.0 + pad_w).max(0.0)),
                    Px((max_child.height.0 + pad_h).max(0.0)),
                );
                let desired = clamp_to_constraints(desired, props.layout, cx.available);

                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_left),
                    Px(cx.bounds.origin.y.0 + pad_top),
                );
                let inner_size = Size::new(
                    Px((desired.width.0 - pad_w).max(0.0)),
                    Px((desired.height.0 - pad_h).max(0.0)),
                );
                let inner_bounds = Rect::new(inner_origin, inner_size);

                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(
                        cx,
                        child,
                        inner_bounds,
                        positioned_layout_style(layout_style),
                    );
                }

                desired
            }
            ElementInstance::Pressable(props) => {
                let probe_bounds =
                    Rect::new(cx.bounds.origin, Size::new(cx.available.width, Px(1.0e9)));
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::Stack(props) => {
                let probe_bounds =
                    Rect::new(cx.bounds.origin, Size::new(cx.available.width, Px(1.0e9)));
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::Spacer(props) => {
                clamp_to_constraints(Size::new(Px(0.0), Px(0.0)), props.layout, cx.available)
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
                    line_height: Some(
                        cx.theme()
                            .metric_by_key("font.line_height")
                            .unwrap_or(cx.theme().metrics.font_line_height),
                    ),
                    ..Default::default()
                });
                let mut measure_width = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.max(0.0)),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                if let Some(max_w) = props.layout.size.max_width {
                    measure_width = Px(measure_width.0.min(max_w.0.max(0.0)));
                }
                measure_width = Px(measure_width.0.max(0.0).min(cx.available.width.0.max(0.0)));
                let constraints = TextConstraints {
                    max_width: Some(measure_width),
                    wrap: props.wrap,
                    overflow: props.overflow,
                    scale_factor: cx.scale_factor,
                };
                let metrics = cx.services.text().measure(&props.text, style, constraints);

                self.text_cache.metrics = Some(metrics);
                self.text_cache.last_text = Some(props.text.clone());
                self.text_cache.last_style = Some(style);
                self.text_cache.last_wrap = Some(props.wrap);
                self.text_cache.last_overflow = Some(props.overflow);
                self.text_cache.last_width = Some(measure_width);
                self.text_cache.last_theme_revision = Some(theme_revision);

                clamp_to_constraints(metrics.size, props.layout, cx.available)
            }
            ElementInstance::TextInput(props) => {
                if self.text_input.is_none() {
                    self.text_input = Some(BoundTextInput::new(props.model));
                }
                let input = self.text_input.as_mut().expect("text input");
                if input.model_id() != props.model.id() {
                    input.set_model(props.model);
                }
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);

                let desired = input.layout(cx);
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::VirtualList(props) => {
                let row_h = Px(props.row_height.0.max(0.0));
                let content_h = Px(row_h.0 * props.len as f32);

                let desired_w = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.max(0.0)),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                let desired_h = match props.layout.size.height {
                    Length::Px(px) => Px(px.0.max(0.0)),
                    Length::Fill => cx.available.height,
                    Length::Auto => Px(content_h.0.min(cx.available.height.0.max(0.0))),
                };

                let size = clamp_to_constraints(
                    Size::new(desired_w, desired_h),
                    props.layout,
                    cx.available,
                );
                let mut needs_redraw = false;
                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::VirtualListState::default,
                    |state| {
                        let prev_viewport_h = state.viewport_h;
                        let prev_offset_y = state.offset_y;

                        let viewport_h = Px(size.height.0.max(0.0));
                        if state.viewport_h != viewport_h {
                            state.viewport_h = viewport_h;
                        }

                        let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
                        state.offset_y = Px(state.offset_y.0.max(0.0).min(max_offset.0));

                        if let Some(target) = props.scroll_to_index
                            && viewport_h.0 > 0.0
                            && row_h.0 > 0.0
                            && props.len > 0
                        {
                            let target = target.min(props.len.saturating_sub(1));
                            let row_top = row_h.0 * target as f32;
                            let row_bottom = row_top + row_h.0;
                            let view_top = state.offset_y.0;
                            let view_bottom = state.offset_y.0 + viewport_h.0;

                            if row_top < view_top {
                                state.offset_y = Px(row_top);
                            } else if row_bottom > view_bottom {
                                state.offset_y = Px(row_bottom - viewport_h.0);
                            }

                            state.offset_y = Px(state.offset_y.0.max(0.0).min(max_offset.0));
                        }

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
                        Size::new(size.width, row_h),
                    );
                    let _ = cx.layout_in(child, child_bounds);
                }

                size
            }
            ElementInstance::Flex(props) => {
                let pad_left = props.padding.left.0.max(0.0);
                let pad_right = props.padding.right.0.max(0.0);
                let pad_top = props.padding.top.0.max(0.0);
                let pad_bottom = props.padding.bottom.0.max(0.0);
                let pad_w = pad_left + pad_right;
                let pad_h = pad_top + pad_bottom;
                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_left),
                    Px(cx.bounds.origin.y.0 + pad_top),
                );
                let outer_avail_w = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.min(cx.available.width.0.max(0.0))),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                let outer_avail_h = match props.layout.size.height {
                    Length::Px(px) => Px(px.0.min(cx.available.height.0.max(0.0))),
                    Length::Fill | Length::Auto => cx.available.height,
                };

                let inner_avail = Size::new(
                    Px((outer_avail_w.0 - pad_w).max(0.0)),
                    Px((outer_avail_h.0 - pad_h).max(0.0)),
                );

                let mut taffy: TaffyTree<Option<NodeId>> = TaffyTree::new();

                let root_style = TaffyStyle {
                    display: Display::Flex,
                    flex_direction: match props.direction {
                        fret_core::Axis::Horizontal => FlexDirection::Row,
                        fret_core::Axis::Vertical => FlexDirection::Column,
                    },
                    flex_wrap: if props.wrap {
                        FlexWrap::Wrap
                    } else {
                        FlexWrap::NoWrap
                    },
                    justify_content: Some(taffy_justify(props.justify)),
                    align_items: Some(taffy_align_items(props.align)),
                    gap: TaffySize {
                        width: LengthPercentage::length(props.gap.0.max(0.0)),
                        height: LengthPercentage::length(props.gap.0.max(0.0)),
                    },
                    size: TaffySize {
                        width: match props.layout.size.width {
                            Length::Px(px) => Dimension::length((px.0 - pad_w).max(0.0)),
                            other => taffy_dimension(other),
                        },
                        height: match props.layout.size.height {
                            Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0)),
                            other => taffy_dimension(other),
                        },
                    },
                    max_size: TaffySize {
                        width: Dimension::length(inner_avail.width.0.max(0.0)),
                        height: Dimension::length(inner_avail.height.0.max(0.0)),
                    },
                    ..Default::default()
                };

                let mut child_nodes: Vec<TaffyNodeId> = Vec::new();
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    let spacer_min = element_record_for_node(cx.app, window, child).and_then(|r| {
                        if let ElementInstance::Spacer(p) = r.instance {
                            Some(p.min)
                        } else {
                            None
                        }
                    });

                    let mut min_w = layout_style.size.min_width.map(|p| p.0);
                    let mut min_h = layout_style.size.min_height.map(|p| p.0);
                    if let Some(min) = spacer_min {
                        let min = min.0.max(0.0);
                        match props.direction {
                            fret_core::Axis::Horizontal => {
                                min_w = Some(min_w.unwrap_or(0.0).max(min));
                            }
                            fret_core::Axis::Vertical => {
                                min_h = Some(min_h.unwrap_or(0.0).max(min));
                            }
                        }
                    }

                    let child_style = TaffyStyle {
                        display: Display::Block,
                        position: taffy_position(layout_style.position),
                        inset: taffy_rect_lpa_from_inset(layout_style.position, layout_style.inset),
                        size: TaffySize {
                            width: taffy_dimension(layout_style.size.width),
                            height: taffy_dimension(layout_style.size.height),
                        },
                        aspect_ratio: layout_style.aspect_ratio,
                        min_size: TaffySize {
                            width: min_w.map(Dimension::length).unwrap_or_else(Dimension::auto),
                            height: min_h.map(Dimension::length).unwrap_or_else(Dimension::auto),
                        },
                        max_size: TaffySize {
                            width: layout_style
                                .size
                                .max_width
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                            height: layout_style
                                .size
                                .max_height
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                        },
                        margin: taffy_rect_lpa_from_edges(layout_style.margin),
                        flex_grow: layout_style.flex.grow.max(0.0),
                        flex_shrink: layout_style.flex.shrink.max(0.0),
                        flex_basis: taffy_dimension(layout_style.flex.basis),
                        align_self: layout_style.flex.align_self.map(taffy_align_self),
                        ..Default::default()
                    };

                    let id = taffy
                        .new_leaf_with_context(child_style, Some(child))
                        .expect("taffy leaf");
                    child_nodes.push(id);
                }

                let root = if child_nodes.is_empty() {
                    taffy.new_leaf(root_style).expect("taffy root")
                } else {
                    taffy
                        .new_with_children(root_style, &child_nodes)
                        .expect("taffy root")
                };

                let available = taffy::geometry::Size {
                    width: TaffyAvailableSpace::Definite(inner_avail.width.0),
                    height: TaffyAvailableSpace::Definite(inner_avail.height.0),
                };

                taffy
                    .compute_layout_with_measure(
                        root,
                        available,
                        |known, avail, _id, ctx, _style| {
                            let Some(child) = ctx.and_then(|c| *c) else {
                                return taffy::geometry::Size::default();
                            };

                            let max_w = match avail.width {
                                TaffyAvailableSpace::Definite(w) => Px(w),
                                _ => Px(1.0e9),
                            };
                            let max_h = match avail.height {
                                TaffyAvailableSpace::Definite(h) => Px(h),
                                _ => Px(1.0e9),
                            };

                            let known_w = known.width.map(Px);
                            let known_h = known.height.map(Px);

                            let w = known_w.unwrap_or(max_w);
                            let h = known_h.unwrap_or(max_h);

                            let probe = Rect::new(inner_origin, Size::new(w, h));
                            let s = cx.layout_in(child, probe);
                            taffy::geometry::Size {
                                width: s.width.0,
                                height: s.height.0,
                            }
                        },
                    )
                    .expect("taffy compute");

                for child_node in child_nodes {
                    let layout = taffy.layout(child_node).expect("taffy layout");
                    let Some(child) = taffy.get_node_context(child_node).and_then(|c| *c) else {
                        continue;
                    };
                    let rect = Rect::new(
                        fret_core::Point::new(
                            Px(inner_origin.x.0 + layout.location.x),
                            Px(inner_origin.y.0 + layout.location.y),
                        ),
                        Size::new(Px(layout.size.width), Px(layout.size.height)),
                    );
                    let _ = cx.layout_in(child, rect);
                }

                let layout = taffy.layout(root).expect("taffy root layout");
                let inner_size = Size::new(
                    Px(layout.size.width.max(0.0)),
                    Px(layout.size.height.max(0.0)),
                );

                let desired = Size::new(
                    Px((inner_size.width.0 + pad_w).max(0.0)),
                    Px((inner_size.height.0 + pad_h).max(0.0)),
                );
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::Grid(props) => {
                let pad_left = props.padding.left.0.max(0.0);
                let pad_right = props.padding.right.0.max(0.0);
                let pad_top = props.padding.top.0.max(0.0);
                let pad_bottom = props.padding.bottom.0.max(0.0);
                let pad_w = pad_left + pad_right;
                let pad_h = pad_top + pad_bottom;
                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_left),
                    Px(cx.bounds.origin.y.0 + pad_top),
                );

                let outer_avail_w = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.min(cx.available.width.0.max(0.0))),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                let outer_avail_h = match props.layout.size.height {
                    Length::Px(px) => Px(px.0.min(cx.available.height.0.max(0.0))),
                    Length::Fill | Length::Auto => cx.available.height,
                };

                let inner_avail = Size::new(
                    Px((outer_avail_w.0 - pad_w).max(0.0)),
                    Px((outer_avail_h.0 - pad_h).max(0.0)),
                );

                let mut taffy: TaffyTree<Option<NodeId>> = TaffyTree::new();

                let root_style = TaffyStyle {
                    display: Display::Grid,
                    justify_content: Some(taffy_justify(props.justify)),
                    align_items: Some(taffy_align_items(props.align)),
                    gap: TaffySize {
                        width: LengthPercentage::length(props.gap.0.max(0.0)),
                        height: LengthPercentage::length(props.gap.0.max(0.0)),
                    },
                    size: TaffySize {
                        width: match props.layout.size.width {
                            Length::Px(px) => Dimension::length((px.0 - pad_w).max(0.0)),
                            other => taffy_dimension(other),
                        },
                        height: match props.layout.size.height {
                            Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0)),
                            other => taffy_dimension(other),
                        },
                    },
                    max_size: TaffySize {
                        width: Dimension::length(inner_avail.width.0.max(0.0)),
                        height: Dimension::length(inner_avail.height.0.max(0.0)),
                    },
                    grid_template_columns: taffy::style_helpers::evenly_sized_tracks(props.cols),
                    grid_template_rows: props
                        .rows
                        .map(taffy::style_helpers::evenly_sized_tracks)
                        .unwrap_or_default(),
                    ..Default::default()
                };

                let mut child_nodes: Vec<TaffyNodeId> = Vec::new();
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);

                    let child_style = TaffyStyle {
                        display: Display::Block,
                        position: taffy_position(layout_style.position),
                        inset: taffy_rect_lpa_from_inset(layout_style.position, layout_style.inset),
                        size: TaffySize {
                            width: taffy_dimension(layout_style.size.width),
                            height: taffy_dimension(layout_style.size.height),
                        },
                        aspect_ratio: layout_style.aspect_ratio,
                        min_size: TaffySize {
                            width: layout_style
                                .size
                                .min_width
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                            height: layout_style
                                .size
                                .min_height
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                        },
                        max_size: TaffySize {
                            width: layout_style
                                .size
                                .max_width
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                            height: layout_style
                                .size
                                .max_height
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                        },
                        margin: taffy_rect_lpa_from_edges(layout_style.margin),
                        grid_column: taffy_grid_line(layout_style.grid.column),
                        grid_row: taffy_grid_line(layout_style.grid.row),
                        ..Default::default()
                    };

                    let id = taffy
                        .new_leaf_with_context(child_style, Some(child))
                        .expect("taffy leaf");
                    child_nodes.push(id);
                }

                let root = if child_nodes.is_empty() {
                    taffy.new_leaf(root_style).expect("taffy root")
                } else {
                    taffy
                        .new_with_children(root_style, &child_nodes)
                        .expect("taffy root")
                };

                let available = taffy::geometry::Size {
                    width: TaffyAvailableSpace::Definite(inner_avail.width.0),
                    height: TaffyAvailableSpace::Definite(inner_avail.height.0),
                };

                taffy
                    .compute_layout_with_measure(
                        root,
                        available,
                        |known, avail, _id, ctx, _style| {
                            let Some(child) = ctx.and_then(|c| *c) else {
                                return taffy::geometry::Size::default();
                            };

                            let max_w = match avail.width {
                                TaffyAvailableSpace::Definite(w) => Px(w),
                                _ => Px(1.0e9),
                            };
                            let max_h = match avail.height {
                                TaffyAvailableSpace::Definite(h) => Px(h),
                                _ => Px(1.0e9),
                            };

                            let known_w = known.width.map(Px);
                            let known_h = known.height.map(Px);

                            let w = known_w.unwrap_or(max_w);
                            let h = known_h.unwrap_or(max_h);

                            let probe = Rect::new(inner_origin, Size::new(w, h));
                            let s = cx.layout_in(child, probe);
                            taffy::geometry::Size {
                                width: s.width.0,
                                height: s.height.0,
                            }
                        },
                    )
                    .expect("taffy compute");

                for child_node in child_nodes {
                    let layout = taffy.layout(child_node).expect("taffy layout");
                    let Some(child) = taffy.get_node_context(child_node).and_then(|c| *c) else {
                        continue;
                    };
                    let rect = Rect::new(
                        fret_core::Point::new(
                            Px(inner_origin.x.0 + layout.location.x),
                            Px(inner_origin.y.0 + layout.location.y),
                        ),
                        Size::new(Px(layout.size.width), Px(layout.size.height)),
                    );
                    let _ = cx.layout_in(child, rect);
                }

                let layout = taffy.layout(root).expect("taffy root layout");
                let inner_size = Size::new(
                    Px(layout.size.width.max(0.0)),
                    Px(layout.size.height.max(0.0)),
                );

                let desired = Size::new(
                    Px((inner_size.width.0 + pad_w).max(0.0)),
                    Px((inner_size.height.0 + pad_h).max(0.0)),
                );
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::Image(props) => {
                let desired = clamp_to_constraints(cx.available, props.layout, cx.available);
                desired
            }
            ElementInstance::Spinner(props) => {
                let desired =
                    clamp_to_constraints(Size::new(Px(16.0), Px(16.0)), props.layout, cx.available);
                desired
            }
            ElementInstance::HoverCard(props) => {
                let hovered =
                    crate::elements::is_hovered_hover_card(&mut *cx.app, window, self.element);

                let frame = cx.app.frame_id();
                let open_delay = props.open_delay_frames as u64;
                let close_delay = props.close_delay_frames as u64;

                let open = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    HoverCardOpenState::default,
                    |state| {
                        if hovered {
                            state.leave_start = None;
                            if !state.open {
                                let start = state.hover_start.get_or_insert(frame);
                                let elapsed = frame.0.saturating_sub(start.0);
                                if elapsed >= open_delay {
                                    state.open = true;
                                    state.hover_start = None;
                                }
                            }
                        } else {
                            state.hover_start = None;
                            if state.open {
                                let start = state.leave_start.get_or_insert(frame);
                                let elapsed = frame.0.saturating_sub(start.0);
                                if elapsed >= close_delay {
                                    state.open = false;
                                    state.leave_start = None;
                                }
                            } else {
                                state.leave_start = None;
                            }
                        }
                        state.open
                    },
                );
                self.hover_card_open = open;

                if let Some(window) = cx.window
                    && ((hovered && !open && open_delay > 0)
                        || (!hovered && open && close_delay > 0))
                {
                    cx.app.push_effect(Effect::RequestAnimationFrame(window));
                    cx.app.push_effect(Effect::UiInvalidateLayout { window });
                    cx.app.request_redraw(window);
                }

                let Some(&trigger) = cx.children.first() else {
                    self.hover_card_open = false;
                    return Size::new(Px(0.0), Px(0.0));
                };

                let trigger_probe =
                    Rect::new(cx.bounds.origin, Size::new(cx.available.width, Px(1.0e9)));
                let trigger_size = cx.layout_in(trigger, trigger_probe);
                let trigger_bounds = Rect::new(cx.bounds.origin, trigger_size);
                let _ = cx.layout_in(trigger, trigger_bounds);

                if let Some(&content) = cx.children.get(1) {
                    if open {
                        let probe = Rect::new(
                            fret_core::Point::new(Px(0.0), Px(0.0)),
                            Size::new(Px(1.0e9), Px(1.0e9)),
                        );
                        let content_size = cx.layout_in(content, probe);

                        let x = match props.align {
                            HoverCardAlign::Start => trigger_bounds.origin.x.0,
                            HoverCardAlign::Center => {
                                trigger_bounds.origin.x.0
                                    + (trigger_bounds.size.width.0 - content_size.width.0) * 0.5
                            }
                            HoverCardAlign::End => {
                                trigger_bounds.origin.x.0
                                    + (trigger_bounds.size.width.0 - content_size.width.0)
                            }
                        };
                        let y = trigger_bounds.origin.y.0
                            + trigger_bounds.size.height.0
                            + props.side_offset.0;

                        let bounds = Rect::new(
                            fret_core::Point::new(Px(x), Px(y)),
                            Size::new(content_size.width, content_size.height),
                        );
                        let _ = cx.layout_in(content, bounds);
                    } else {
                        let _ = cx.layout_in(
                            content,
                            Rect::new(cx.bounds.origin, Size::new(Px(0.0), Px(0.0))),
                        );
                    }
                }

                clamp_to_constraints(trigger_size, props.layout, cx.available)
            }
            ElementInstance::Scroll(props) => {
                let probe_bounds =
                    Rect::new(cx.bounds.origin, Size::new(cx.available.width, Px(1.0e9)));

                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let viewport_h = Px(desired.height.0.max(0.0));
                let content_h = Px(max_child.height.0.max(0.0));
                let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));

                let offset_y = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::ScrollState::default,
                    |state| {
                        state.viewport_h = viewport_h;
                        state.content_h = content_h;
                        state.offset_y = Px(state.offset_y.0.max(0.0).min(max_offset.0));
                        state.offset_y
                    },
                );

                if props.show_scrollbar && max_offset.0 > 0.0 {
                    let theme = cx.theme();
                    let scrollbar_w = theme
                        .metric_by_key("metric.scrollbar.width")
                        .unwrap_or(theme.metrics.scrollbar_width);
                    let w = Px(scrollbar_w.0.max(0.0).min(desired.width.0.max(0.0)));
                    if w.0 > 0.0 {
                        let track = Rect::new(
                            fret_core::Point::new(
                                Px(cx.bounds.origin.x.0 + desired.width.0.max(0.0) - w.0),
                                cx.bounds.origin.y,
                            ),
                            Size::new(w, desired.height),
                        );
                        self.scrollbar_hit_rect = Some(track);
                    }
                }

                let shifted = Rect::new(
                    fret_core::Point::new(
                        cx.bounds.origin.x,
                        Px(cx.bounds.origin.y.0 - offset_y.0),
                    ),
                    Size::new(desired.width, content_h),
                );
                for &child in cx.children {
                    let _ = cx.layout_in(child, shifted);
                }

                desired
            }
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return;
        };

        for (model, invalidation) in
            crate::elements::observed_models_for_element(cx.app, window, self.element)
        {
            (cx.observe_model)(model, invalidation);
        }

        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };

        match instance {
            ElementInstance::Container(props) => {
                let should_draw = props.shadow.is_some()
                    || props.background.is_some()
                    || props.border_color.is_some()
                    || props.border != Edges::all(Px(0.0));

                if should_draw {
                    if let Some(shadow) = props.shadow {
                        crate::paint::paint_shadow(cx.scene, DrawOrder(0), cx.bounds, shadow);
                    }
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: cx.bounds,
                        background: props.background.unwrap_or(Color::TRANSPARENT),
                        border: props.border,
                        border_color: props.border_color.unwrap_or(Color::TRANSPARENT),
                        corner_radii: props.corner_radii,
                    });
                }

                paint_children_clipped_if(cx, matches!(props.layout.overflow, Overflow::Clip));
            }
            ElementInstance::Stack(props) => {
                paint_children_clipped_if(cx, matches!(props.layout.overflow, Overflow::Clip));
            }
            ElementInstance::Flex(props) => {
                paint_children_clipped_if(cx, matches!(props.layout.overflow, Overflow::Clip));
            }
            ElementInstance::Grid(props) => {
                paint_children_clipped_if(cx, matches!(props.layout.overflow, Overflow::Clip));
            }
            ElementInstance::Spacer(_props) => {}
            ElementInstance::Pressable(props) => {
                paint_children_clipped_if(cx, matches!(props.layout.overflow, Overflow::Clip));

                if props.enabled
                    && cx.focus == Some(cx.node)
                    && crate::focus_visible::is_focus_visible(cx.app, cx.window)
                    && let Some(ring) = props.focus_ring
                {
                    crate::paint::paint_focus_ring(cx.scene, DrawOrder(0), cx.bounds, ring);
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
                    line_height: Some(
                        cx.theme()
                            .metric_by_key("font.line_height")
                            .unwrap_or(cx.theme().metrics.font_line_height),
                    ),
                    ..Default::default()
                });
                let color = props
                    .color
                    .or_else(|| cx.theme().color_by_key("foreground"))
                    .unwrap_or(cx.theme().colors.text_primary);
                let constraints = TextConstraints {
                    max_width: Some(cx.bounds.size.width),
                    wrap: props.wrap,
                    overflow: props.overflow,
                    scale_factor: cx.scale_factor,
                };

                let scale_bits = cx.scale_factor.to_bits();
                let needs_prepare = self.text_cache.blob.is_none()
                    || self.text_cache.prepared_scale_factor_bits != Some(scale_bits)
                    || self.text_cache.last_text.as_ref() != Some(&props.text)
                    || self.text_cache.last_style.as_ref() != Some(&style)
                    || self.text_cache.last_wrap != Some(props.wrap)
                    || self.text_cache.last_overflow != Some(props.overflow)
                    || self.text_cache.last_width != Some(cx.bounds.size.width)
                    || self.text_cache.last_theme_revision != Some(theme_revision);

                if needs_prepare {
                    if let Some(blob) = self.text_cache.blob.take() {
                        cx.services.text().release(blob);
                    }
                    let (blob, metrics) =
                        cx.services.text().prepare(&props.text, style, constraints);
                    self.text_cache.blob = Some(blob);
                    self.text_cache.metrics = Some(metrics);
                    self.text_cache.prepared_scale_factor_bits = Some(scale_bits);
                    self.text_cache.last_text = Some(props.text.clone());
                    self.text_cache.last_style = Some(style);
                    self.text_cache.last_wrap = Some(props.wrap);
                    self.text_cache.last_overflow = Some(props.overflow);
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
            ElementInstance::TextInput(props) => {
                if self.text_input.is_none() {
                    self.text_input = Some(BoundTextInput::new(props.model));
                }
                let input = self.text_input.as_mut().expect("text input");
                if input.model_id() != props.model.id() {
                    input.set_model(props.model);
                }
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);
                input.paint(cx);
            }
            ElementInstance::VirtualList(props) => {
                cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });

                let offset_y = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::VirtualListState::default,
                    |state| Px(state.offset_y.0.max(0.0)),
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

                    cx.scene.push(SceneOp::PushClipRect { rect: child_bounds });
                    cx.paint(child, child_bounds);
                    cx.scene.push(SceneOp::PopClip);
                }

                cx.scene.push(SceneOp::PopClip);
            }
            ElementInstance::Image(props) => {
                let opacity = props.opacity.clamp(0.0, 1.0);
                if let Some(uv) = props.uv {
                    cx.scene.push(SceneOp::ImageRegion {
                        order: DrawOrder(0),
                        rect: cx.bounds,
                        image: props.image,
                        uv,
                        opacity,
                    });
                } else {
                    cx.scene.push(SceneOp::Image {
                        order: DrawOrder(0),
                        rect: cx.bounds,
                        image: props.image,
                        opacity,
                    });
                }
            }
            ElementInstance::Spinner(props) => {
                let theme = cx.theme();
                let base = props
                    .color
                    .or_else(|| theme.color_by_key("muted-foreground"))
                    .unwrap_or(theme.colors.text_muted);

                let n = (props.dot_count.max(1).min(32)) as usize;

                let w = cx.bounds.size.width.0.max(0.0);
                let h = cx.bounds.size.height.0.max(0.0);
                let min_dim = w.min(h);
                if min_dim <= 0.0 {
                    return;
                }

                let dot = (min_dim * 0.18).clamp(2.0, (min_dim * 0.25).max(2.0));
                let radius = (min_dim * 0.5 - dot * 0.5).max(0.0);

                let cx0 = cx.bounds.origin.x.0 + w * 0.5;
                let cy0 = cx.bounds.origin.y.0 + h * 0.5;

                let speed = props.speed.max(0.0);
                if speed > 0.0 {
                    cx.app.push_effect(Effect::RequestAnimationFrame(window));
                }

                let phase = cx.app.frame_id().0 as f32 * speed;
                let active = (phase.floor() as i32).rem_euclid(n as i32) as usize;
                let tail_len = (n.min(5)).saturating_sub(1);

                for i in 0..n {
                    let dist = ((i + n) - active) % n;
                    let t = if tail_len == 0 {
                        if dist == 0 { 1.0 } else { 0.25 }
                    } else if dist == 0 {
                        1.0
                    } else if dist <= tail_len {
                        1.0 - dist as f32 / (tail_len as f32 + 1.0)
                    } else {
                        0.25
                    };

                    let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
                    let x = cx0 + radius * angle.cos() - dot * 0.5;
                    let y = cy0 + radius * angle.sin() - dot * 0.5;

                    let mut color = base;
                    color.a = (color.a * t).clamp(0.0, 1.0);

                    let rect = Rect::new(
                        fret_core::Point::new(Px(x), Px(y)),
                        Size::new(Px(dot), Px(dot)),
                    );
                    let r = Px(dot * 0.5);
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect,
                        background: color,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(r),
                    });
                }
            }
            ElementInstance::HoverCard(props) => {
                let clip = matches!(props.layout.overflow, Overflow::Clip);
                if clip {
                    cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
                }

                if let Some(&trigger) = cx.children.first() {
                    let bounds = cx.child_bounds(trigger).unwrap_or(cx.bounds);
                    cx.paint(trigger, bounds);
                }

                if self.hover_card_open
                    && let Some(&content) = cx.children.get(1)
                {
                    let bounds = cx.child_bounds(content).unwrap_or(cx.bounds);
                    cx.paint(content, bounds);
                }

                if clip {
                    cx.scene.push(SceneOp::PopClip);
                }
            }
            ElementInstance::Scroll(props) => {
                let clip = matches!(props.layout.overflow, Overflow::Clip);
                if clip {
                    cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
                }

                for &child in cx.children {
                    let bounds = cx.child_bounds(child).unwrap_or(cx.bounds);
                    cx.paint(child, bounds);
                }

                if clip {
                    cx.scene.push(SceneOp::PopClip);
                }

                if props.show_scrollbar {
                    let (offset_y, viewport_h, content_h, hovered, dragging) =
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollState::default,
                            |state| {
                                (
                                    state.offset_y,
                                    state.viewport_h,
                                    state.content_h,
                                    state.hovered_scrollbar,
                                    state.dragging_thumb,
                                )
                            },
                        );
                    let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
                    if max_offset.0 > 0.0 {
                        let theme = cx.theme();
                        let scrollbar_w = theme
                            .metric_by_key("metric.scrollbar.width")
                            .unwrap_or(theme.metrics.scrollbar_width);
                        if let Some(track) = scrollbar_track_rect(cx.bounds, scrollbar_w)
                            && let Some(thumb) =
                                scrollbar_thumb_rect(track, viewport_h, content_h, offset_y)
                        {
                            let thumb_color = if hovered || dragging {
                                theme
                                    .color_by_key("scrollbar.thumb.hover.background")
                                    .unwrap_or(
                                        theme
                                            .color_by_key("scrollbar.thumb.background")
                                            .unwrap_or(theme.colors.scrollbar_thumb_hover),
                                    )
                            } else {
                                theme
                                    .color_by_key("scrollbar.thumb.background")
                                    .unwrap_or(theme.colors.scrollbar_thumb)
                            };
                            let mut bg = thumb_color;
                            if !(hovered || dragging) {
                                bg.a *= 0.65;
                            }

                            let inset = 1.0f32.min(thumb.size.width.0 * 0.25);
                            let rect = Rect::new(
                                fret_core::Point::new(Px(thumb.origin.x.0 + inset), thumb.origin.y),
                                Size::new(
                                    Px((thumb.size.width.0 - inset * 2.0).max(0.0)),
                                    thumb.size.height,
                                ),
                            );

                            cx.scene.push(SceneOp::Quad {
                                order: DrawOrder(20_000),
                                rect,
                                background: bg,
                                border: Edges::all(Px(0.0)),
                                border_color: Color::TRANSPARENT,
                                corner_radii: fret_core::Corners::all(Px(999.0)),
                            });
                        }
                    }
                }
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
    services: &mut dyn fret_core::UiServices,
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
                    hit_testable: true,
                    hit_test_children: true,
                    is_focusable: false,
                    is_text_input: false,
                    clips_hit_test: true,
                    scrollbar_hit_rect: None,
                    text_input: None,
                    hover_card_open: false,
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
            let _ = ui.remove_subtree(services, node);
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
                hit_testable: true,
                hit_test_children: true,
                is_focusable: false,
                is_text_input: false,
                clips_hit_test: true,
                scrollbar_hit_rect: None,
                text_input: None,
                hover_card_open: false,
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
        ElementKind::VirtualList(p) => ElementInstance::VirtualList(p),
        ElementKind::Flex(p) => ElementInstance::Flex(p),
        ElementKind::Grid(p) => ElementInstance::Grid(p),
        ElementKind::Image(p) => ElementInstance::Image(p),
        ElementKind::Spinner(p) => ElementInstance::Spinner(p),
        ElementKind::HoverCard(p) => ElementInstance::HoverCard(p),
        ElementKind::Scroll(p) => ElementInstance::Scroll(p),
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
    use crate::element::{AnyElement, CrossAlign, MainAlign};
    use crate::elements::ElementCx;
    use crate::test_host::TestHost;
    use crate::tree::UiTree;
    use crate::widget::Invalidation;
    use fret_core::{
        AppWindowId, Color, Modifiers, MouseButton, MouseButtons, NodeId, Point, Px, Rect, Scene,
        SceneOp, Size, TextConstraints, TextMetrics, TextService, TextStyle,
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

    impl fret_core::PathService for FakeTextService {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    #[test]
    fn keyed_elements_reuse_node_ids_across_reorder() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

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
            let list = cx.virtual_list(100, Px(10.0), 0, None, |cx, range| {
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
    fn virtual_list_scroll_to_index_keeps_target_visible() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(30.0)),
        );
        let mut text = FakeTextService::default();

        // Frame 0: establish viewport height.
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-scroll-to",
            |cx| {
                vec![cx.virtual_list(100, Px(10.0), 0, None, |cx, range| {
                    range.map(|i| cx.keyed(i, |cx| cx.text("row"))).collect()
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        app.advance_frame();

        // Frame 1: request scroll-to on a row below the viewport.
        let target = 6usize; // row_top=60, viewport=30 => needs offset ~= 40..60
        let mut list_element_id: Option<crate::elements::GlobalElementId> = None;
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-scroll-to",
            |cx| {
                let list = cx.virtual_list(100, Px(10.0), 0, Some(target), |cx, range| {
                    range.map(|i| cx.keyed(i, |cx| cx.text("row"))).collect()
                });
                list_element_id = Some(list.id);
                vec![list]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let state = crate::elements::with_element_state(
            &mut app,
            window,
            list_element_id.expect("list element id"),
            crate::element::VirtualListState::default,
            |s| *s,
        );

        assert!(
            (state.offset_y.0 - 40.0).abs() < 0.01,
            "offset_y={:?}",
            state.offset_y
        );
    }

    #[test]
    fn virtual_list_paint_clips_each_visible_row() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(50.0)),
        );
        let mut text = FakeTextService::default();

        fn build_list<H: UiHost>(cx: &mut ElementCx<'_, H>) -> AnyElement {
            cx.virtual_list(100, Px(10.0), 0, None, |cx, range| {
                range.map(|i| cx.keyed(i, |cx| cx.text("row"))).collect()
            })
        }

        // Frame 0: record viewport height (no visible children yet).
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-clip",
            |cx| vec![build_list(cx)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        app.advance_frame();

        // Frame 1: mount visible children based on the recorded viewport height.
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-clip",
            |cx| vec![build_list(cx)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

        // One clip for the list viewport + one clip per visible row child.
        let pushes = scene
            .ops()
            .iter()
            .filter(|op| matches!(op, SceneOp::PushClipRect { .. }))
            .count();
        assert_eq!(pushes, 1 + 5);
    }

    #[test]
    fn virtual_list_keyed_reuses_node_ids_across_reorder() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(30.0)),
        );
        let mut text = FakeTextService::default();

        let mut items: Vec<u64> = vec![10, 20, 30];
        let mut ids: Vec<(u64, crate::elements::GlobalElementId)> = Vec::new();

        fn build_list<H: UiHost>(
            cx: &mut ElementCx<'_, H>,
            items: &[u64],
            mut ids: Option<&mut Vec<(u64, crate::elements::GlobalElementId)>>,
        ) -> AnyElement {
            cx.virtual_list_keyed(
                items.len(),
                Px(10.0),
                0,
                None,
                |i| items[i],
                |cx, i| {
                    let row = cx.text("row");
                    if let Some(ids) = ids.as_deref_mut() {
                        ids.push((items[i], row.id));
                    }
                    row
                },
            )
        }

        // Frame 0: record viewport height (no visible children yet).
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-keyed",
            |cx| vec![build_list(cx, &items, None)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        app.advance_frame();

        let mut prev: std::collections::HashMap<u64, (crate::elements::GlobalElementId, NodeId)> =
            std::collections::HashMap::new();

        for pass in 0..2 {
            ids.clear();
            let root = render_root(
                &mut ui,
                &mut app,
                &mut text,
                window,
                bounds,
                "mvp50-vlist-keyed",
                |cx| vec![build_list(cx, &items, Some(&mut ids))],
            );
            ui.set_root(root);
            ui.layout_all(&mut app, &mut text, bounds, 1.0);

            let cur: std::collections::HashMap<u64, (crate::elements::GlobalElementId, NodeId)> =
                app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
                    runtime.prepare_window_for_frame(window, app.frame_id());
                    let st = runtime.for_window_mut(window);
                    ids.iter()
                        .map(|(item, id)| (*item, (*id, st.node_entry(*id).unwrap().node)))
                        .collect()
                });

            if pass == 1 {
                for item in [10u64, 20u64, 30u64] {
                    let (prev_id, prev_node) = prev.get(&item).copied().unwrap();
                    let (cur_id, cur_node) = cur.get(&item).copied().unwrap();
                    assert_eq!(prev_id, cur_id, "element id should be stable");
                    assert_eq!(prev_node, cur_node, "node id should be stable");
                }
            }

            prev = cur;
            items.reverse();
            app.advance_frame();
        }
    }

    #[test]
    fn row_justify_center_and_align_end_positions_children() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(20.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "row-align",
            |cx| {
                let mut props = crate::element::RowProps {
                    gap: Px(5.0),
                    justify: MainAlign::Center,
                    align: CrossAlign::End,
                    ..Default::default()
                };
                props.layout.size.width = crate::element::Length::Fill;
                props.layout.size.height = crate::element::Length::Fill;
                vec![cx.row(props, |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")])]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let row_node = ui.children(root)[0];
        let children = ui.children(row_node);
        assert_eq!(children.len(), 3);

        let b0 = ui.debug_node_bounds(children[0]).expect("child0 bounds");
        let b1 = ui.debug_node_bounds(children[1]).expect("child1 bounds");
        let b2 = ui.debug_node_bounds(children[2]).expect("child2 bounds");

        // Each text measures to 10x10. With gap=5 and width=100:
        // content_w = 3*10 + 2*5 = 40; remaining=60; center => start_offset=30.
        assert!((b0.origin.x.0 - 30.0).abs() < 0.01, "x0={:?}", b0.origin.x);
        assert!((b1.origin.x.0 - 45.0).abs() < 0.01, "x1={:?}", b1.origin.x);
        assert!((b2.origin.x.0 - 60.0).abs() < 0.01, "x2={:?}", b2.origin.x);

        // align-end with row height 20 => y = 0 + (20-10)=10.
        assert!((b0.origin.y.0 - 10.0).abs() < 0.01, "y0={:?}", b0.origin.y);
        assert!((b1.origin.y.0 - 10.0).abs() < 0.01, "y1={:?}", b1.origin.y);
        assert!((b2.origin.y.0 - 10.0).abs() < 0.01, "y2={:?}", b2.origin.y);
    }

    #[test]
    fn pressable_keyboard_activation_dispatches_click_command() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let cmd = CommandId::new("test.pressable.click");

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(30.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "pressable-keyboard",
            |cx| {
                vec![cx.pressable(
                    crate::element::PressableProps {
                        on_click: Some(cmd.clone()),
                        ..Default::default()
                    },
                    |cx, _state| vec![cx.text("ok")],
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let pressable_node = ui.children(root)[0];
        ui.set_focus(Some(pressable_node));
        assert_eq!(ui.focus(), Some(pressable_node));

        let _ = app.take_effects();
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );
        let effects = app.take_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::Command { command, .. } if *command == cmd)),
            "expected click command effect"
        );
    }

    #[test]
    fn pressable_disabled_is_not_focusable() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(30.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "pressable-disabled-focus",
            |cx| {
                vec![cx.pressable(
                    crate::element::PressableProps {
                        enabled: false,
                        ..Default::default()
                    },
                    |cx, _state| vec![cx.text("disabled")],
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        assert_eq!(ui.first_focusable_descendant(root), None);
    }

    #[test]
    fn image_paints_image_op() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );
        let mut text = FakeTextService::default();

        let img = fret_core::ImageId::default();
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp-image",
            |cx| {
                let mut p = crate::element::ImageProps::new(img);
                p.layout.size.width = crate::element::Length::Px(Px(160.0));
                p.layout.size.height = crate::element::Length::Px(Px(80.0));
                vec![cx.image_props(p)]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

        assert!(
            scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::Image { image, .. } if *image == img)),
            "expected an Image op for the declarative image element"
        );
    }

    #[test]
    fn overflow_clip_pushes_clip_rect_for_children() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(60.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp-overflow-clip",
            |cx| {
                let mut props = crate::element::ContainerProps::default();
                props.layout.overflow = crate::element::Overflow::Clip;
                vec![cx.container(props, |cx| vec![cx.text("child")])]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

        let pushes = scene
            .ops()
            .iter()
            .filter(|op| matches!(op, SceneOp::PushClipRect { .. }))
            .count();
        let pops = scene
            .ops()
            .iter()
            .filter(|op| matches!(op, SceneOp::PopClip))
            .count();

        assert!(
            pushes >= 1,
            "expected container overflow clip to push a clip rect"
        );
        assert!(
            pops >= 1,
            "expected container overflow clip to pop a clip rect"
        );
    }

    #[test]
    fn overflow_visible_does_not_push_clip_rect() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(60.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp-overflow-visible",
            |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("child")])],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

        assert!(
            !scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::PushClipRect { .. })),
            "expected no clip ops by default"
        );
    }

    #[test]
    fn scroll_wheel_updates_offset_and_shifts_child_bounds() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(40.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp-scroll-wheel",
            |cx| {
                let mut p = crate::element::ScrollProps::default();
                p.layout.size.width = crate::element::Length::Fill;
                p.layout.size.height = crate::element::Length::Px(Px(20.0));
                vec![cx.scroll(p, |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0),
                            ..Default::default()
                        },
                        |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                    )]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let scroll_node = ui.children(root)[0];
        let column_node = ui.children(scroll_node)[0];
        let before = ui.debug_node_bounds(column_node).expect("column bounds");

        let wheel_pos = fret_core::Point::new(Px(5.0), Px(5.0));
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
                position: wheel_pos,
                delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let after = ui
            .debug_node_bounds(column_node)
            .expect("column bounds after scroll");

        assert!(
            after.origin.y.0 < before.origin.y.0,
            "expected content to move up after wheel scroll: before={:?} after={:?}",
            before.origin.y,
            after.origin.y
        );
    }

    #[test]
    fn scroll_thumb_drag_updates_offset() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(40.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp-scrollbar-drag",
            |cx| {
                let mut p = crate::element::ScrollProps::default();
                p.layout.size.height = crate::element::Length::Px(Px(20.0));
                vec![cx.scroll(p, |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0),
                            ..Default::default()
                        },
                        |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                    )]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let scroll_node = ui.children(root)[0];
        let column_node = ui.children(scroll_node)[0];
        let before = ui.debug_node_bounds(column_node).expect("column bounds");

        // Click/drag the scrollbar thumb down (thumb starts at the top at offset=0).
        let scroll_bounds = ui.debug_node_bounds(scroll_node).expect("scroll bounds");
        let down_pos = fret_core::Point::new(
            Px(scroll_bounds.origin.x.0 + scroll_bounds.size.width.0 - 1.0),
            Px(scroll_bounds.origin.y.0 + 2.0),
        );
        let move_pos = fret_core::Point::new(down_pos.x, Px(down_pos.y.0 + 8.0));
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: down_pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: move_pos,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: move_pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let after = ui
            .debug_node_bounds(column_node)
            .expect("column bounds after drag");

        assert!(
            after.origin.y.0 < before.origin.y.0,
            "expected content to move up after thumb drag: before={:?} after={:?}",
            before.origin.y,
            after.origin.y
        );
    }

    #[test]
    fn fill_respects_max_width_constraint() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(100.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp-scroll-max-width",
            |cx| {
                vec![cx.container(
                    crate::element::ContainerProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: crate::element::Length::Fill,
                                max_width: Some(Px(100.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |_cx| vec![],
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let container_node = ui.children(root)[0];
        let rect = ui
            .debug_node_bounds(container_node)
            .expect("container bounds");
        assert_eq!(rect.size.width, Px(100.0));
    }

    #[test]
    fn flex_child_margin_affects_layout() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(40.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp58-flex-margin",
            |cx| {
                vec![cx.flex(
                    crate::element::FlexProps {
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| {
                        let mut a = crate::element::ContainerProps::default();
                        a.layout.size.width = crate::element::Length::Px(Px(10.0));
                        a.layout.size.height = crate::element::Length::Px(Px(10.0));

                        let mut b = crate::element::ContainerProps::default();
                        b.layout.size.width = crate::element::Length::Px(Px(10.0));
                        b.layout.size.height = crate::element::Length::Px(Px(10.0));
                        b.layout.margin.left = Px(5.0);

                        vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let flex_node = ui.children(root)[0];
        let children = ui.children(flex_node);
        assert_eq!(children.len(), 2);
        let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
        let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

        assert_eq!(a_bounds.origin.x, Px(0.0));
        assert_eq!(b_bounds.origin.x, Px(15.0));
    }

    #[test]
    fn container_absolute_inset_positions_child() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(200.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp58-stack-absolute",
            |cx| {
                vec![
                    cx.container(crate::element::ContainerProps::default(), |cx| {
                        let mut base = crate::element::ContainerProps::default();
                        base.layout.size.width = crate::element::Length::Px(Px(100.0));
                        base.layout.size.height = crate::element::Length::Px(Px(80.0));

                        let mut badge = crate::element::ContainerProps::default();
                        badge.layout.size.width = crate::element::Length::Px(Px(10.0));
                        badge.layout.size.height = crate::element::Length::Px(Px(10.0));
                        badge.layout.position = crate::element::PositionStyle::Absolute;
                        badge.layout.inset.top = Some(Px(0.0));
                        badge.layout.inset.right = Some(Px(0.0));

                        vec![
                            cx.container(base, |_cx| vec![]),
                            cx.container(badge, |_cx| vec![]),
                        ]
                    }),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let container_node = ui.children(root)[0];
        let container_bounds = ui
            .debug_node_bounds(container_node)
            .expect("container bounds");
        assert_eq!(container_bounds.size.width, Px(100.0));
        assert_eq!(container_bounds.size.height, Px(80.0));

        let children = ui.children(container_node);
        assert_eq!(children.len(), 2);
        let badge_bounds = ui.debug_node_bounds(children[1]).expect("badge bounds");
        assert_eq!(badge_bounds.origin.x, Px(90.0));
        assert_eq!(badge_bounds.origin.y, Px(0.0));
    }

    #[test]
    fn grid_places_children_in_columns() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp58-grid",
            |cx| {
                vec![cx.grid(
                    crate::element::GridProps {
                        layout: {
                            let mut l = crate::element::LayoutStyle::default();
                            l.size.width = crate::element::Length::Fill;
                            l.size.height = crate::element::Length::Fill;
                            l
                        },
                        cols: 2,
                        ..Default::default()
                    },
                    |cx| {
                        let mut a = crate::element::ContainerProps::default();
                        a.layout.size.width = crate::element::Length::Fill;
                        a.layout.size.height = crate::element::Length::Fill;

                        let mut b = crate::element::ContainerProps::default();
                        b.layout.size.width = crate::element::Length::Fill;
                        b.layout.size.height = crate::element::Length::Fill;

                        vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let grid_node = ui.children(root)[0];
        let children = ui.children(grid_node);
        assert_eq!(children.len(), 2);
        let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
        let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

        assert_eq!(a_bounds.origin.x, Px(0.0));
        assert_eq!(b_bounds.origin.x, Px(100.0));
        assert_eq!(a_bounds.size.width, Px(100.0));
        assert_eq!(b_bounds.size.width, Px(100.0));
    }

    #[test]
    fn focus_ring_is_focus_visible_only() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(64.0), Px(32.0)),
        );
        let mut text = FakeTextService::default();

        let ring = crate::element::RingStyle {
            placement: crate::element::RingPlacement::Outset,
            width: Px(2.0),
            offset: Px(2.0),
            color: Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            offset_color: None,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        };

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp-focus-visible",
            |cx| {
                vec![cx.pressable(
                    crate::element::PressableProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: crate::element::Length::Fill,
                                height: crate::element::Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        focus_ring: Some(ring),
                        ..Default::default()
                    },
                    |_cx, _st| vec![],
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let pressable_node = ui.children(root)[0];

        // Focus the pressable via pointer: should *not* show focus-visible ring.
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: fret_core::Point::new(Px(4.0), Px(4.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(
            ui.focus(),
            Some(pressable_node),
            "expected pressable to be focused after pointer down"
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
        assert_eq!(
            scene.ops().len(),
            0,
            "expected no ring ops for mouse-focused control"
        );

        // Enable focus-visible via keyboard navigation: ring should appear for focused control.
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Tab,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        assert_eq!(
            ui.focus(),
            Some(pressable_node),
            "expected focus to remain on pressable after keydown"
        );
        assert!(
            crate::focus_visible::is_focus_visible(&mut app, Some(window)),
            "expected focus-visible to be enabled after Tab keydown"
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
        assert!(
            !scene.ops().is_empty(),
            "expected ring ops for keyboard navigation focus-visible"
        );
    }

    #[test]
    fn declarative_elements_can_observe_models_for_invalidation() {
        let mut app = TestHost::new();
        let model = app.models_mut().insert(0u32);

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(40.0)),
        );
        let mut text = FakeTextService::default();

        let root_name = "mvp50-observe-model";

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            root_name,
            |cx| {
                vec![cx.container(Default::default(), |cx| {
                    cx.observe_model(model, Invalidation::Layout);
                    let v = cx.app.models().get(model).copied().unwrap_or_default();
                    vec![cx.text(format!("Value {v}"))]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let stats0 = ui.debug_stats();
        assert!(
            stats0.layout_nodes_visited > 0,
            "expected layout traversal: visited={} performed={}",
            stats0.layout_nodes_visited,
            stats0.layout_nodes_performed
        );
        let performed0 = stats0.layout_nodes_performed;
        assert!(performed0 > 0, "expected initial layout work");

        // A second layout pass with no changes and no re-render should perform no node layouts.
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let performed1 = ui.debug_stats().layout_nodes_performed;
        assert_eq!(performed1, 0, "expected no layout work when clean");

        let _ = model.update(&mut app, |v, _cx| *v += 1);
        let changed = app.take_changed_models();
        ui.propagate_model_changes(&mut app, &changed);

        // The observed model change should invalidate the declarative host, enabling layout work.
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let performed2 = ui.debug_stats().layout_nodes_performed;
        assert!(performed2 > 0, "expected model change to trigger relayout");
    }

    #[test]
    fn model_observation_requires_rerender_after_frame_advance() {
        let mut app = TestHost::new();
        let model = app.models_mut().insert(0u32);

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(40.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-observe-contract-frame-advance",
            |cx| {
                vec![cx.container(Default::default(), |cx| {
                    cx.observe_model(model, Invalidation::Layout);
                    let v = cx.app.models().get(model).copied().unwrap_or_default();
                    vec![cx.text(format!("Value {v}"))]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        // Advance the frame but intentionally skip the render pass.
        app.advance_frame();

        // The first model change still invalidates because UiTree retains the previous observation
        // index until the next layout/paint pass records observations again.
        let _ = model.update(&mut app, |v, _cx| *v += 1);
        let changed = app.take_changed_models();
        assert!(
            ui.propagate_model_changes(&mut app, &changed),
            "expected invalidation from the last recorded observation index"
        );

        // Layout now runs on the advanced frame. Without a new render pass, the declarative layer
        // has no per-frame observation data to re-register, so the observation index is cleared.
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        // A second model change no longer invalidates: this encodes the ADR 0028 execution contract
        // that `render_root(...)` must be called each frame before layout/paint.
        let _ = model.update(&mut app, |v, _cx| *v += 1);
        let changed = app.take_changed_models();
        assert!(
            !ui.propagate_model_changes(&mut app, &changed),
            "expected no invalidation without re-rendering after a frame advance"
        );
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
                        padding: fret_core::Edges::symmetric(Px(4.0), Px(6.0)),
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
        let container_bounds = ui
            .debug_node_bounds(container_node)
            .expect("container bounds");
        let text_bounds = ui.debug_node_bounds(text_node).expect("text bounds");
        assert_eq!(text_bounds.origin.x, Px(4.0));
        assert_eq!(text_bounds.origin.y, Px(6.0));
        assert_eq!(text_bounds.size.width, Px(10.0));
        assert_eq!(text_bounds.size.height, Px(10.0));

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

        assert_eq!(scene.ops_len(), 2);
        match scene.ops()[0] {
            SceneOp::Quad {
                rect, background, ..
            } => {
                assert_eq!(rect, container_bounds);
                assert_eq!(background.a, 1.0);
            }
            _ => panic!("expected quad op"),
        }
    }

    #[test]
    fn container_paints_shadow_before_background() {
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
            "mvp60-shadow",
            |cx| {
                vec![cx.container(
                    crate::element::ContainerProps {
                        background: Some(Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        shadow: Some(crate::element::ShadowStyle {
                            color: Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.5,
                            },
                            offset_x: Px(2.0),
                            offset_y: Px(3.0),
                            spread: Px(1.0),
                            softness: 0,
                            corner_radii: fret_core::Corners::all(Px(4.0)),
                        }),
                        corner_radii: fret_core::Corners::all(Px(4.0)),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("hi")],
                )]
            },
        );
        ui.set_root(root);

        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let container_node = ui.children(root)[0];
        let container_bounds = ui
            .debug_node_bounds(container_node)
            .expect("container bounds");

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

        assert_eq!(scene.ops_len(), 3);

        let shadow_bounds = match scene.ops()[0] {
            SceneOp::Quad { rect, .. } => rect,
            _ => panic!("expected shadow quad first"),
        };
        match scene.ops()[1] {
            SceneOp::Quad {
                rect, background, ..
            } => {
                assert_eq!(rect, container_bounds);
                assert_eq!(background.a, 1.0);
            }
            _ => panic!("expected background quad second"),
        }

        assert!(shadow_bounds.origin.x.0 > container_bounds.origin.x.0);
        assert!(shadow_bounds.origin.y.0 > container_bounds.origin.y.0);
        assert!(shadow_bounds.size.width.0 > container_bounds.size.width.0);
        assert!(shadow_bounds.size.height.0 > container_bounds.size.height.0);
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
                        ..Default::default()
                    },
                    |cx, _state| {
                        vec![cx.container(
                            crate::element::ContainerProps {
                                padding: fret_core::Edges::all(Px(4.0)),
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

    #[test]
    fn flex_defaults_to_fit_content_under_constraints() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "flex-fit",
            |cx| {
                vec![cx.flex(
                    crate::element::FlexProps {
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(5.0),
                        padding: fret_core::Edges::symmetric(Px(4.0), Px(6.0)),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a"), cx.text("b")],
                )]
            },
        );
        ui.set_root(root);

        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let flex_node = ui.children(root)[0];
        let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");

        // FakeTextService measures each text to 10x10. With gap=5 and padding (4,6):
        // inner_w = 10 + 5 + 10 = 25, outer_w = 25 + 8 = 33
        // inner_h = 10, outer_h = 10 + 12 = 22
        assert!(
            (flex_bounds.size.width.0 - 33.0).abs() < 0.01,
            "w={:?}",
            flex_bounds.size.width
        );
        assert!(
            (flex_bounds.size.height.0 - 22.0).abs() < 0.01,
            "h={:?}",
            flex_bounds.size.height
        );
    }
}
