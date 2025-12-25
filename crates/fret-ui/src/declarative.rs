use crate::UiHost;
use crate::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, ElementKind, FlexProps, LayoutStyle,
    Length, MainAlign, PressableProps, RowProps, SpacerProps, StackProps, TextProps,
};
use crate::elements::{ElementCx, GlobalElementId, NodeEntry};
use crate::tree::UiTree;
use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget};
use fret_core::{
    AppWindowId, Color, CursorIcon, DrawOrder, Edges, Event, FontId, MouseButton, NodeId, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextStyle,
};
use std::collections::HashMap;
use taffy::{
    TaffyTree,
    geometry::Size as TaffySize,
    style::{
        AlignItems as TaffyAlignItems, AlignSelf as TaffyAlignSelf,
        AvailableSpace as TaffyAvailableSpace, Dimension, Display, FlexDirection, FlexWrap,
        JustifyContent, LengthPercentage, Style as TaffyStyle,
    },
    tree::NodeId as TaffyNodeId,
};

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
    Spacer(SpacerProps),
    Text(TextProps),
    VirtualList(crate::element::VirtualListProps),
    Flex(FlexProps),
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

fn clamp_to_constraints(mut size: Size, style: LayoutStyle, available: Size) -> Size {
    if let Some(min_w) = style.size.min_width {
        size.width = Px(size.width.0.max(min_w.0));
    }
    if let Some(min_h) = style.size.min_height {
        size.height = Px(size.height.0.max(min_h.0));
    }
    if let Some(max_w) = style.size.max_width {
        size.width = Px(size.width.0.min(max_w.0));
    }
    if let Some(max_h) = style.size.max_height {
        size.height = Px(size.height.0.min(max_h.0));
    }

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

    size.width = Px(size.width.0.max(0.0).min(available.width.0.max(0.0)));
    size.height = Px(size.height.0.max(0.0).min(available.height.0.max(0.0)));
    size
}

fn taffy_dimension(length: Length) -> Dimension {
    match length {
        Length::Auto => Dimension::auto(),
        Length::Fill => Dimension::percent(1.0),
        Length::Px(px) => Dimension::length(px.0),
    }
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
        if matches!(instance, ElementInstance::Flex(_)) {
            // Flex is a layout container; it does not imply semantics beyond its children.
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

        match instance {
            ElementInstance::Container(props) => {
                let pad_x = props.padding_x.0.max(0.0);
                let pad_y = props.padding_y.0.max(0.0);

                let inner_avail = Size::new(
                    Px((cx.available.width.0 - pad_x * 2.0).max(0.0)),
                    Px((cx.available.height.0 - pad_y * 2.0).max(0.0)),
                );

                let probe_bounds =
                    Rect::new(cx.bounds.origin, Size::new(inner_avail.width, Px(1.0e9)));
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = Size::new(
                    Px((max_child.width.0 + pad_x * 2.0).max(0.0)),
                    Px((max_child.height.0 + pad_y * 2.0).max(0.0)),
                );
                let desired = clamp_to_constraints(desired, props.layout, cx.available);

                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_x),
                    Px(cx.bounds.origin.y.0 + pad_y),
                );
                let inner_size = Size::new(
                    Px((desired.width.0 - pad_x * 2.0).max(0.0)),
                    Px((desired.height.0 - pad_y * 2.0).max(0.0)),
                );
                let inner_bounds = Rect::new(inner_origin, inner_size);

                for &child in cx.children {
                    let _ = cx.layout_in(child, inner_bounds);
                }

                desired
            }
            ElementInstance::Pressable(props) => {
                let probe_bounds =
                    Rect::new(cx.bounds.origin, Size::new(cx.available.width, Px(1.0e9)));
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                for &child in cx.children {
                    let _ = cx.layout_in(child, Rect::new(cx.bounds.origin, desired));
                }
                desired
            }
            ElementInstance::Stack(props) => {
                let probe_bounds =
                    Rect::new(cx.bounds.origin, Size::new(cx.available.width, Px(1.0e9)));
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                for &child in cx.children {
                    let _ = cx.layout_in(child, Rect::new(cx.bounds.origin, desired));
                }
                desired
            }
            ElementInstance::Column(props) => {
                let pad_x = props.padding_x.0.max(0.0);
                let pad_y = props.padding_y.0.max(0.0);
                let gap = props.gap.0.max(0.0);
                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_x),
                    Px(cx.bounds.origin.y.0 + pad_y),
                );
                let inner_width = Px((cx.available.width.0 - pad_x * 2.0).max(0.0));
                let inner_height = Px((cx.available.height.0 - pad_y * 2.0).max(0.0));

                let mut rows: Vec<(NodeId, Size, bool)> = Vec::new();
                let mut fixed_h = 0.0f32;
                let mut spacer_count = 0usize;

                for &child in cx.children {
                    let is_spacer = matches!(
                        element_record_for_node(cx.app, window, child).map(|r| r.instance),
                        Some(ElementInstance::Spacer(_))
                    );
                    if is_spacer {
                        spacer_count = spacer_count.saturating_add(1);
                        rows.push((child, Size::new(Px(0.0), Px(0.0)), true));
                        continue;
                    }

                    let probe_bounds = Rect::new(
                        fret_core::Point::new(inner_origin.x, inner_origin.y),
                        Size::new(inner_width, Px(1.0e9)),
                    );
                    let child_size = cx.layout_in(child, probe_bounds);
                    let w = Px(child_size.width.0.max(0.0).min(inner_width.0));
                    let h = Px(child_size.height.0.max(0.0));
                    fixed_h += h.0;
                    rows.push((child, Size::new(w, h), false));
                }

                let gaps = if cx.children.len() > 1 {
                    gap * (cx.children.len() as f32 - 1.0)
                } else {
                    0.0
                };
                let fixed_total = fixed_h + gaps;
                let remaining = (inner_height.0 - fixed_total).max(0.0);

                let mut extra_gap = 0.0f32;
                let mut start_offset = 0.0f32;

                if spacer_count == 0 {
                    match props.justify {
                        MainAlign::Start => {}
                        MainAlign::Center => start_offset = remaining * 0.5,
                        MainAlign::End => start_offset = remaining,
                        MainAlign::SpaceBetween => {
                            if cx.children.len() > 1 {
                                extra_gap = remaining / (cx.children.len() as f32 - 1.0);
                            }
                        }
                        MainAlign::SpaceAround => {
                            if !cx.children.is_empty() {
                                extra_gap = remaining / (cx.children.len() as f32);
                                start_offset = extra_gap * 0.5;
                            }
                        }
                        MainAlign::SpaceEvenly => {
                            if !cx.children.is_empty() {
                                extra_gap = remaining / (cx.children.len() as f32 + 1.0);
                                start_offset = extra_gap;
                            }
                        }
                    }
                }

                let gap_used = gap + extra_gap;
                let spacer_h = if spacer_count > 0 {
                    remaining / spacer_count as f32
                } else {
                    0.0
                };

                let mut y = Px(inner_origin.y.0 + start_offset);
                for (i, (child, measured, is_spacer)) in rows.into_iter().enumerate() {
                    if i > 0 {
                        y = Px(y.0 + gap_used);
                    }

                    let child_h = if is_spacer {
                        let min = element_record_for_node(cx.app, window, child)
                            .and_then(|r| match r.instance {
                                ElementInstance::Spacer(p) => Some(p.min),
                                _ => None,
                            })
                            .unwrap_or(Px(0.0));
                        Px(spacer_h.max(min.0).max(0.0))
                    } else {
                        measured.height
                    };

                    let child_w = match props.align {
                        CrossAlign::Stretch => inner_width,
                        _ => Px(measured.width.0.max(0.0).min(inner_width.0)),
                    };

                    let x = match props.align {
                        CrossAlign::Start | CrossAlign::Stretch => inner_origin.x,
                        CrossAlign::Center => {
                            Px(inner_origin.x.0 + (inner_width.0 - child_w.0).max(0.0) * 0.5)
                        }
                        CrossAlign::End => {
                            Px(inner_origin.x.0 + (inner_width.0 - child_w.0).max(0.0))
                        }
                    };

                    let bounds =
                        Rect::new(fret_core::Point::new(x, y), Size::new(child_w, child_h));
                    let _ = cx.layout_in(child, bounds);
                    y = Px(y.0 + child_h.0);
                }

                let total_h = Px((fixed_total + pad_y * 2.0).max(0.0));
                Size::new(cx.available.width, Px(total_h.0.min(cx.available.height.0)))
            }
            ElementInstance::Row(props) => {
                let pad_x = props.padding_x.0.max(0.0);
                let pad_y = props.padding_y.0.max(0.0);
                let gap = props.gap.0.max(0.0);

                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_x),
                    Px(cx.bounds.origin.y.0 + pad_y),
                );
                let inner_width = Px((cx.available.width.0 - pad_x * 2.0).max(0.0));

                let mut cols: Vec<(NodeId, Size, bool)> = Vec::new();
                let mut fixed_w = 0.0f32;
                let mut max_h = 0.0f32;
                let mut spacer_count = 0usize;

                for &child in cx.children {
                    let is_spacer = matches!(
                        element_record_for_node(cx.app, window, child).map(|r| r.instance),
                        Some(ElementInstance::Spacer(_))
                    );
                    if is_spacer {
                        spacer_count = spacer_count.saturating_add(1);
                        cols.push((child, Size::new(Px(0.0), Px(0.0)), true));
                        continue;
                    }

                    let probe_bounds = Rect::new(
                        fret_core::Point::new(inner_origin.x, inner_origin.y),
                        Size::new(inner_width, Px(1.0e9)),
                    );
                    let child_size = cx.layout_in(child, probe_bounds);
                    let w = Px(child_size.width.0.max(0.0).min(inner_width.0));
                    let h = Px(child_size.height.0.max(0.0));
                    fixed_w += w.0;
                    max_h = max_h.max(h.0);
                    cols.push((child, Size::new(w, h), false));
                }

                let gaps = if cx.children.len() > 1 {
                    gap * (cx.children.len() as f32 - 1.0)
                } else {
                    0.0
                };
                let fixed_total = fixed_w + gaps;
                let remaining = (inner_width.0 - fixed_total).max(0.0);

                let mut extra_gap = 0.0f32;
                let mut start_offset = 0.0f32;

                if spacer_count == 0 {
                    match props.justify {
                        MainAlign::Start => {}
                        MainAlign::Center => start_offset = remaining * 0.5,
                        MainAlign::End => start_offset = remaining,
                        MainAlign::SpaceBetween => {
                            if cx.children.len() > 1 {
                                extra_gap = remaining / (cx.children.len() as f32 - 1.0);
                            }
                        }
                        MainAlign::SpaceAround => {
                            if !cx.children.is_empty() {
                                extra_gap = remaining / (cx.children.len() as f32);
                                start_offset = extra_gap * 0.5;
                            }
                        }
                        MainAlign::SpaceEvenly => {
                            if !cx.children.is_empty() {
                                extra_gap = remaining / (cx.children.len() as f32 + 1.0);
                                start_offset = extra_gap;
                            }
                        }
                    }
                }

                let gap_used = gap + extra_gap;
                let spacer_w = if spacer_count > 0 {
                    remaining / spacer_count as f32
                } else {
                    0.0
                };

                let mut x = Px(inner_origin.x.0 + start_offset);
                for (i, (child, measured, is_spacer)) in cols.into_iter().enumerate() {
                    if i > 0 {
                        x = Px(x.0 + gap_used);
                    }

                    let child_w = if is_spacer {
                        let min = element_record_for_node(cx.app, window, child)
                            .and_then(|r| match r.instance {
                                ElementInstance::Spacer(p) => Some(p.min),
                                _ => None,
                            })
                            .unwrap_or(Px(0.0));
                        Px(spacer_w.max(min.0).max(0.0))
                    } else {
                        measured.width
                    };

                    let (child_h, dy) = match props.align {
                        CrossAlign::Stretch => (Px(max_h), 0.0),
                        CrossAlign::Start => (Px(measured.height.0.max(0.0)), 0.0),
                        CrossAlign::Center => (
                            Px(measured.height.0.max(0.0)),
                            (max_h - measured.height.0).max(0.0) * 0.5,
                        ),
                        CrossAlign::End => (
                            Px(measured.height.0.max(0.0)),
                            (max_h - measured.height.0).max(0.0),
                        ),
                    };

                    let y = Px(inner_origin.y.0 + dy);
                    let bounds =
                        Rect::new(fret_core::Point::new(x, y), Size::new(child_w, child_h));
                    let _ = cx.layout_in(child, bounds);
                    x = Px(x.0 + child_w.0);
                }

                let total_h = Px((max_h + pad_y * 2.0).max(0.0));
                Size::new(cx.available.width, Px(total_h.0.min(cx.available.height.0)))
            }
            ElementInstance::Spacer(_) => cx.available,
            ElementInstance::Text(props) => {
                let theme_revision = cx.theme().revision();
                let font_size = cx
                    .theme()
                    .metric_by_key("font.size")
                    .unwrap_or(cx.theme().metrics.font_size);
                let style = props.style.unwrap_or(TextStyle {
                    font: FontId::default(),
                    size: font_size,
                    ..Default::default()
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
                        Size::new(cx.bounds.size.width, row_h),
                    );
                    let _ = cx.layout_in(child, child_bounds);
                }

                cx.available
            }
            ElementInstance::Flex(props) => {
                let pad_x = props.padding_x.0.max(0.0);
                let pad_y = props.padding_y.0.max(0.0);
                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_x),
                    Px(cx.bounds.origin.y.0 + pad_y),
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
                    Px((outer_avail_w.0 - pad_x * 2.0).max(0.0)),
                    Px((outer_avail_h.0 - pad_y * 2.0).max(0.0)),
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
                            Length::Px(px) => Dimension::length((px.0 - pad_x * 2.0).max(0.0)),
                            other => taffy_dimension(other),
                        },
                        height: match props.layout.size.height {
                            Length::Px(px) => Dimension::length((px.0 - pad_y * 2.0).max(0.0)),
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
                    let layout_style = element_record_for_node(cx.app, window, child)
                        .map(|r| match r.instance {
                            ElementInstance::Container(p) => p.layout,
                            ElementInstance::Pressable(p) => p.layout,
                            ElementInstance::Stack(p) => p.layout,
                            ElementInstance::Column(p) => p.layout,
                            ElementInstance::Row(p) => p.layout,
                            ElementInstance::Spacer(p) => p.layout,
                            ElementInstance::Text(p) => p.layout,
                            ElementInstance::VirtualList(p) => p.layout,
                            ElementInstance::Flex(p) => p.layout,
                        })
                        .unwrap_or_default();

                    let child_style = TaffyStyle {
                        display: Display::Block,
                        size: TaffySize {
                            width: taffy_dimension(layout_style.size.width),
                            height: taffy_dimension(layout_style.size.height),
                        },
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
                    Px((inner_size.width.0 + pad_x * 2.0).max(0.0)),
                    Px((inner_size.height.0 + pad_y * 2.0).max(0.0)),
                );
                clamp_to_constraints(desired, props.layout, cx.available)
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
            | ElementInstance::Row(_)
            | ElementInstance::Flex(_)
            | ElementInstance::Spacer(_) => {
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
                    ..Default::default()
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
        ElementKind::Spacer(p) => ElementInstance::Spacer(p),
        ElementKind::Text(p) => ElementInstance::Text(p),
        ElementKind::VirtualList(p) => ElementInstance::VirtualList(p),
        ElementKind::Flex(p) => ElementInstance::Flex(p),
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
                vec![cx.row(
                    crate::element::RowProps {
                        gap: Px(5.0),
                        justify: MainAlign::Center,
                        align: CrossAlign::End,
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                )]
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

        // align-end with row height 10 => y = 0 + (10-10)=0, so still 0.
        assert!((b0.origin.y.0 - 0.0).abs() < 0.01, "y0={:?}", b0.origin.y);
        assert!((b1.origin.y.0 - 0.0).abs() < 0.01, "y1={:?}", b1.origin.y);
        assert!((b2.origin.y.0 - 0.0).abs() < 0.01, "y2={:?}", b2.origin.y);
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
                        padding_x: Px(4.0),
                        padding_y: Px(6.0),
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
