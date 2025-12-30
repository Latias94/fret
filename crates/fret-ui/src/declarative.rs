use crate::UiHost;
use crate::element::{
    AnyElement, ContainerProps, CrossAlign, ElementKind, FlexProps, HoverRegionProps, LayoutStyle,
    Length, MainAlign, Overflow, PointerRegionProps, PressableProps, SpacerProps, SpinnerProps,
    StackProps, TextProps,
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
    Pressable(PressableProps),
    PointerRegion(PointerRegionProps),
    DismissibleLayer(DismissibleLayerProps),
    RovingFlex(crate::element::RovingFlexProps),
    Stack(StackProps),
    Spacer(SpacerProps),
    Text(TextProps),
    TextInput(crate::element::TextInputProps),
    TextArea(crate::element::TextAreaProps),
    Slider(crate::element::SliderProps),
    ResizablePanelGroup(crate::element::ResizablePanelGroupProps),
    VirtualList(crate::element::VirtualListProps),
    Flex(FlexProps),
    Grid(crate::element::GridProps),
    Image(crate::element::ImageProps),
    SvgIcon(crate::element::SvgIconProps),
    Spinner(SpinnerProps),
    HoverRegion(HoverRegionProps),
    Scroll(crate::element::ScrollProps),
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
            ElementInstance::Pressable(p) => p.layout,
            ElementInstance::PointerRegion(p) => p.layout,
            ElementInstance::DismissibleLayer(p) => p.layout,
            ElementInstance::RovingFlex(p) => p.flex.layout,
            ElementInstance::Stack(p) => p.layout,
            ElementInstance::Spacer(p) => p.layout,
            ElementInstance::Text(p) => p.layout,
            ElementInstance::TextInput(p) => p.layout,
            ElementInstance::TextArea(p) => p.layout,
            ElementInstance::Slider(p) => p.layout,
            ElementInstance::ResizablePanelGroup(p) => p.layout,
            ElementInstance::VirtualList(p) => p.layout,
            ElementInstance::Flex(p) => p.layout,
            ElementInstance::Grid(p) => p.layout,
            ElementInstance::Image(p) => p.layout,
            ElementInstance::SvgIcon(p) => p.layout,
            ElementInstance::Spinner(p) => p.layout,
            ElementInstance::HoverRegion(p) => p.layout,
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

fn taffy_lpa_margin_edge(edge: crate::element::MarginEdge) -> LengthPercentageAuto {
    match edge {
        crate::element::MarginEdge::Px(px) => LengthPercentageAuto::length(px.0),
        crate::element::MarginEdge::Auto => LengthPercentageAuto::auto(),
    }
}

fn taffy_rect_lpa_from_margin_edges(
    margin: crate::element::MarginEdges,
) -> TaffyRect<LengthPercentageAuto> {
    TaffyRect {
        left: taffy_lpa_margin_edge(margin.left),
        right: taffy_lpa_margin_edge(margin.right),
        top: taffy_lpa_margin_edge(margin.top),
        bottom: taffy_lpa_margin_edge(margin.bottom),
    }
}

fn taffy_grid_line(line: crate::element::GridLine) -> TaffyLine<GridPlacement> {
    let start = line
        .start
        .map(taffy::style_helpers::line::<GridPlacement>)
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

fn prepare_window_frame_for_frame(window_frame: &mut WindowFrame, frame_id: FrameId) {
    if window_frame.frame_id != frame_id {
        window_frame.frame_id = frame_id;
        window_frame.instances.clear();
    }
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

struct TaffyContainerCache {
    children: Vec<NodeId>,
    taffy: TaffyTree<Option<NodeId>>,
    root: TaffyNodeId,
    child_nodes: Vec<TaffyNodeId>,
    node_by_child: HashMap<NodeId, TaffyNodeId>,
    root_style: Option<TaffyStyle>,
    child_styles: HashMap<NodeId, TaffyStyle>,
    measure_cache: std::collections::HashMap<TaffyMeasureKey, taffy::geometry::Size<f32>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TaffyMeasureKey {
    child: NodeId,
    known_w: Option<u32>,
    known_h: Option<u32>,
    avail_w: (u8, u32),
    avail_h: (u8, u32),
}

fn taffy_available_space_key(avail: TaffyAvailableSpace) -> (u8, u32) {
    match avail {
        TaffyAvailableSpace::Definite(v) => (0, v.to_bits()),
        TaffyAvailableSpace::MinContent => (1, 0),
        TaffyAvailableSpace::MaxContent => (2, 0),
    }
}

impl Default for TaffyContainerCache {
    fn default() -> Self {
        // Root stays stable across frames; children are updated incrementally.
        let mut taffy: TaffyTree<Option<NodeId>> = TaffyTree::new();
        let root = taffy.new_leaf(TaffyStyle::default()).expect("taffy root");
        Self {
            children: Vec::new(),
            taffy,
            root,
            child_nodes: Vec::new(),
            node_by_child: HashMap::new(),
            root_style: None,
            child_styles: HashMap::new(),
            measure_cache: std::collections::HashMap::new(),
        }
    }
}

impl TaffyContainerCache {
    fn sync_root_style(&mut self, root_style: TaffyStyle) {
        if self.root_style.as_ref() == Some(&root_style) {
            return;
        }
        self.taffy
            .set_style(self.root, root_style.clone())
            .expect("taffy root style");
        self.root_style = Some(root_style);
    }

    fn sync_children(
        &mut self,
        children: &[NodeId],
        mut style_for_child: impl FnMut(NodeId) -> TaffyStyle,
    ) {
        let children_changed = self.children != children;

        if children_changed {
            let keep: std::collections::HashSet<NodeId> = children.iter().copied().collect();
            let removed: Vec<NodeId> = self
                .node_by_child
                .keys()
                .copied()
                .filter(|child| !keep.contains(child))
                .collect();

            for child in removed {
                let Some(node) = self.node_by_child.remove(&child) else {
                    continue;
                };
                self.child_styles.remove(&child);
                self.taffy.remove(node).expect("taffy remove");
            }

            self.children = children.to_vec();
        }

        self.child_nodes.clear();
        self.child_nodes.reserve(children.len());
        for &child in children {
            let node = if let Some(&node) = self.node_by_child.get(&child) {
                node
            } else {
                let node = self
                    .taffy
                    .new_leaf_with_context(TaffyStyle::default(), Some(child))
                    .expect("taffy leaf");
                self.node_by_child.insert(child, node);
                node
            };
            self.child_nodes.push(node);

            let style = style_for_child(child);
            let style_changed = self.child_styles.get(&child) != Some(&style);
            if style_changed {
                self.taffy
                    .set_style(node, style.clone())
                    .expect("taffy child style");
                self.child_styles.insert(child, style);
            }
        }

        if children_changed {
            self.taffy
                .set_children(self.root, &self.child_nodes)
                .expect("taffy set children");
        }
    }
}

struct ElementHostWidget {
    element: GlobalElementId,
    text_cache: TextCache,
    hit_testable: bool,
    hit_test_children: bool,
    is_focusable: bool,
    is_text_input: bool,
    clips_hit_test: bool,
    clip_hit_test_corner_radii: Option<fret_core::Corners>,
    scrollbar_hit_rect: Option<Rect>,
    text_input: Option<BoundTextInput>,
    text_area: Option<crate::text_area::BoundTextArea>,
    slider: Option<crate::slider::BoundSlider>,
    resizable_panel_group: Option<crate::resizable_panel_group::BoundResizablePanelGroup>,
    flex_cache: Option<TaffyContainerCache>,
    grid_cache: Option<TaffyContainerCache>,
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

    fn layout_flex_container<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: FlexProps,
    ) -> Size {
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
                    Length::Fill => Dimension::length(inner_avail.width.0.max(0.0)),
                    Length::Auto => Dimension::auto(),
                },
                height: match props.layout.size.height {
                    Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0)),
                    Length::Fill => Dimension::length(inner_avail.height.0.max(0.0)),
                    Length::Auto => Dimension::auto(),
                },
            },
            max_size: TaffySize {
                width: Dimension::length(inner_avail.width.0.max(0.0)),
                height: Dimension::length(inner_avail.height.0.max(0.0)),
            },
            ..Default::default()
        };

        let cache = self
            .flex_cache
            .get_or_insert_with(TaffyContainerCache::default);

        cache.sync_root_style(root_style);
        cache.sync_children(cx.children, |child| {
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

            TaffyStyle {
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
                margin: taffy_rect_lpa_from_margin_edges(layout_style.margin),
                flex_grow: layout_style.flex.grow.max(0.0),
                flex_shrink: layout_style.flex.shrink.max(0.0),
                flex_basis: taffy_dimension(layout_style.flex.basis),
                align_self: layout_style.flex.align_self.map(taffy_align_self),
                ..Default::default()
            }
        });

        cache
            .taffy
            .mark_dirty(cache.root)
            .expect("taffy mark dirty");

        cache.measure_cache.clear();
        let root = cache.root;
        {
            let measure_cache = &mut cache.measure_cache;
            let taffy = &mut cache.taffy;

            let available = taffy::geometry::Size {
                width: TaffyAvailableSpace::Definite(inner_avail.width.0),
                height: TaffyAvailableSpace::Definite(inner_avail.height.0),
            };

            taffy
                .compute_layout_with_measure(root, available, |known, avail, _id, ctx, _style| {
                    let Some(child) = ctx.and_then(|c| *c) else {
                        return taffy::geometry::Size::default();
                    };

                    let key = TaffyMeasureKey {
                        child,
                        known_w: known.width.map(|v| v.to_bits()),
                        known_h: known.height.map(|v| v.to_bits()),
                        avail_w: taffy_available_space_key(avail.width),
                        avail_h: taffy_available_space_key(avail.height),
                    };
                    if let Some(size) = measure_cache.get(&key) {
                        return *size;
                    }

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
                    let out = taffy::geometry::Size {
                        width: s.width.0,
                        height: s.height.0,
                    };
                    measure_cache.insert(key, out);
                    out
                })
                .expect("taffy compute");
        }

        let taffy = &cache.taffy;
        let root_layout = taffy.layout(root).expect("taffy root layout");
        let container_inner_size = Size::new(
            Px(root_layout.size.width.max(0.0)),
            Px(root_layout.size.height.max(0.0)),
        );
        let auto_margin_inner_size = Size::new(
            match props.layout.size.width {
                Length::Fill => inner_avail.width,
                _ => container_inner_size.width,
            },
            match props.layout.size.height {
                Length::Fill => inner_avail.height,
                _ => container_inner_size.height,
            },
        );

        for &child_node in &cache.child_nodes {
            let layout = taffy.layout(child_node).expect("taffy layout");
            let Some(child) = taffy.get_node_context(child_node).and_then(|c| *c) else {
                continue;
            };
            let child_style = layout_style_for_node(cx.app, window, child);
            let single_child = cx.children.len() == 1;

            let mut x = layout.location.x;
            let mut y = layout.location.y;

            let margin_left_auto =
                matches!(child_style.margin.left, crate::element::MarginEdge::Auto);
            let margin_right_auto =
                matches!(child_style.margin.right, crate::element::MarginEdge::Auto);
            let margin_top_auto =
                matches!(child_style.margin.top, crate::element::MarginEdge::Auto);
            let margin_bottom_auto =
                matches!(child_style.margin.bottom, crate::element::MarginEdge::Auto);

            let margin_px = |edge: crate::element::MarginEdge| match edge {
                crate::element::MarginEdge::Px(px) => px.0,
                crate::element::MarginEdge::Auto => 0.0,
            };

            match props.direction {
                fret_core::Axis::Horizontal => {
                    if single_child && (margin_left_auto || margin_right_auto) {
                        let left = if margin_left_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.left)
                        };
                        let right = if margin_right_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.right)
                        };
                        let free =
                            auto_margin_inner_size.width.0 - layout.size.width - left - right;
                        if margin_left_auto && margin_right_auto {
                            x = (left + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_left_auto {
                            x = (left + free.max(0.0)).max(0.0);
                        } else if margin_right_auto {
                            x = left.max(0.0);
                        }
                    }

                    if margin_top_auto || margin_bottom_auto {
                        let top = if margin_top_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.top)
                        };
                        let bottom = if margin_bottom_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.bottom)
                        };
                        let free =
                            auto_margin_inner_size.height.0 - layout.size.height - top - bottom;
                        if margin_top_auto && margin_bottom_auto {
                            y = (top + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_top_auto {
                            y = (top + free.max(0.0)).max(0.0);
                        } else if margin_bottom_auto {
                            y = top.max(0.0);
                        }
                    }
                }
                fret_core::Axis::Vertical => {
                    if single_child && (margin_top_auto || margin_bottom_auto) {
                        let top = if margin_top_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.top)
                        };
                        let bottom = if margin_bottom_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.bottom)
                        };
                        let free =
                            auto_margin_inner_size.height.0 - layout.size.height - top - bottom;
                        if margin_top_auto && margin_bottom_auto {
                            y = (top + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_top_auto {
                            y = (top + free.max(0.0)).max(0.0);
                        } else if margin_bottom_auto {
                            y = top.max(0.0);
                        }
                    }

                    if margin_left_auto || margin_right_auto {
                        let left = if margin_left_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.left)
                        };
                        let right = if margin_right_auto {
                            0.0
                        } else {
                            margin_px(child_style.margin.right)
                        };
                        let free =
                            auto_margin_inner_size.width.0 - layout.size.width - left - right;
                        if margin_left_auto && margin_right_auto {
                            x = (left + (free.max(0.0) / 2.0)).max(0.0);
                        } else if margin_left_auto {
                            x = (left + free.max(0.0)).max(0.0);
                        } else if margin_right_auto {
                            x = left.max(0.0);
                        }
                    }
                }
            }
            let rect = Rect::new(
                fret_core::Point::new(Px(inner_origin.x.0 + x), Px(inner_origin.y.0 + y)),
                Size::new(Px(layout.size.width), Px(layout.size.height)),
            );
            let _ = cx.layout_in(child, rect);
        }

        let desired = Size::new(
            Px((container_inner_size.width.0 + pad_w).max(0.0)),
            Px((container_inner_size.height.0 + pad_h).max(0.0)),
        );
        clamp_to_constraints(desired, props.layout, cx.available)
    }
}

impl<H: UiHost> Widget<H> for ElementHostWidget {
    fn clips_hit_test(&self, _bounds: Rect) -> bool {
        self.clips_hit_test
    }

    fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<fret_core::Corners> {
        self.clip_hit_test_corner_radii
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

        let is_text_input = matches!(
            instance,
            ElementInstance::TextInput(_) | ElementInstance::TextArea(_)
        );

        if let Event::Timer { token } = event {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                crate::action::TimerActionHooks::default,
                |hooks| hooks.on_timer.clone(),
            );

            if let Some(h) = hook {
                let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                let handled = h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: self.element,
                    },
                    *token,
                );
                if handled {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }
        }

        let try_key_hook = |cx: &mut EventCx<'_, H>,
                            key: fret_core::KeyCode,
                            modifiers: fret_core::Modifiers,
                            repeat: bool| {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                crate::action::KeyActionHooks::default,
                |hooks| hooks.on_key_down.clone(),
            );

            if let Some(h) = hook {
                let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                let handled = h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: self.element,
                    },
                    KeyDownCx {
                        key,
                        modifiers,
                        repeat,
                    },
                );
                if handled {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return true;
                }
            }
            false
        };

        if let Event::KeyDown {
            key,
            modifiers,
            repeat,
        } = event
            && cx.focus == Some(cx.node)
            && !is_text_input
            && try_key_hook(cx, *key, *modifiers, *repeat)
        {
            return;
        }

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
            ElementInstance::TextArea(props) => {
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(props.model));
                }
                let area = self.text_area.as_mut().expect("text area");
                if area.model_id() != props.model.id() {
                    area.set_model(props.model);
                }
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);
                area.event(cx, event);
            }
            ElementInstance::Slider(props) => {
                if self.slider.is_none() {
                    self.slider = Some(crate::slider::BoundSlider::new(props.model));
                }
                let slider = self.slider.as_mut().expect("slider");
                if slider.model_id() != props.model.id() {
                    slider.set_model(props.model);
                }
                slider.set_range(props.min, props.max);
                slider.set_step(props.step);
                slider.set_enabled(props.enabled);
                slider.set_style(props.chrome);
                slider.event(cx, event);
            }
            ElementInstance::ResizablePanelGroup(props) => {
                if self.resizable_panel_group.is_none() {
                    self.resizable_panel_group =
                        Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                            props.axis,
                            props.model,
                        ));
                }
                let group = self
                    .resizable_panel_group
                    .as_mut()
                    .expect("resizable panel group");
                if group.model_id() != props.model.id() {
                    group.set_model(props.model);
                }
                group.set_axis(props.axis);
                group.set_enabled(props.enabled);
                group.set_min_px(props.min_px.clone());
                group.set_style(props.chrome.clone());
                group.event(cx, event);
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
                                state.metrics.ensure(
                                    props.len,
                                    props.estimate_row_height,
                                    props.gap,
                                    props.scroll_margin,
                                );
                                let viewport_h = Px(state.viewport_h.0.max(0.0));

                                let prev = props.scroll_handle.offset();
                                let offset_y = state.metrics.clamp_offset(prev.y, viewport_h);

                                let next = state
                                    .metrics
                                    .clamp_offset(Px(offset_y.0 - delta.y.0), viewport_h);
                                if (prev.y.0 - next.0).abs() > 0.01 {
                                    props
                                        .scroll_handle
                                        .set_offset(fret_core::Point::new(prev.x, next));
                                }
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
                let external_handle = props.scroll_handle.clone();
                match pe {
                    fret_core::PointerEvent::Wheel { delta, .. } => {
                        if let Some(handle) = external_handle.as_ref() {
                            let prev = handle.offset();
                            handle.set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));
                        } else {
                            crate::elements::with_element_state(
                                &mut *cx.app,
                                window,
                                self.element,
                                crate::element::ScrollState::default,
                                |state| {
                                    let prev = state.scroll_handle.offset();
                                    state
                                        .scroll_handle
                                        .set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));
                                },
                            );
                        }
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
                                let handle =
                                    external_handle.as_ref().unwrap_or(&state.scroll_handle);
                                let viewport_h = Px(handle.viewport_size().height.0.max(0.0));
                                let content_h = Px(handle.content_size().height.0.max(0.0));
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
                                            if (handle.offset().y.0 - next.0).abs() > 0.01 {
                                                let prev = handle.offset();
                                                handle.set_offset(Point::new(prev.x, next));
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
                                let handle =
                                    external_handle.as_ref().unwrap_or(&state.scroll_handle);
                                let viewport_h = Px(handle.viewport_size().height.0.max(0.0));
                                let content_h = Px(handle.content_size().height.0.max(0.0));
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
                                    handle.offset().y,
                                ) else {
                                    return;
                                };

                                did_handle = true;
                                state.hovered_scrollbar = true;

                                if thumb.contains(position) {
                                    state.dragging_thumb = true;
                                    state.drag_start_pointer_y = position.y;
                                    state.drag_start_offset_y = handle.offset().y;
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
                                        let next = Px((max_offset.0 * t).clamp(0.0, max_offset.0));
                                        let prev = handle.offset();
                                        handle.set_offset(Point::new(prev.x, next));
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
            ElementInstance::DismissibleLayer(props) => {
                if !props.enabled {
                    return;
                }

                match event {
                    Event::KeyDown {
                        key: fret_core::KeyCode::Escape,
                        repeat: false,
                        ..
                    } => {
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::DismissibleActionHooks::default,
                            |hooks| hooks.on_dismiss_request.clone(),
                        );

                        if let Some(h) = hook {
                            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                            h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
                                },
                                DismissReason::Escape,
                            );
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                    Event::Pointer(fret_core::PointerEvent::Down { .. }) => {
                        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Observer
                        {
                            return;
                        }
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::DismissibleActionHooks::default,
                            |hooks| hooks.on_dismiss_request.clone(),
                        );

                        if let Some(h) = hook {
                            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                            h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
                                },
                                DismissReason::OutsidePress,
                            );
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                    }
                    _ => {}
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
                            if props.focusable {
                                cx.request_focus(cx.node);
                            }
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
                                let hook = crate::elements::with_element_state(
                                    &mut *cx.app,
                                    window,
                                    self.element,
                                    crate::action::PressableActionHooks::default,
                                    |hooks| hooks.on_activate.clone(),
                                );

                                if let Some(h) = hook {
                                    let mut host =
                                        action::UiActionHostAdapter { app: &mut *cx.app };
                                    h(
                                        &mut host,
                                        action::ActionCx {
                                            window,
                                            target: self.element,
                                        },
                                        ActivateReason::Pointer,
                                    );
                                }
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
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PressableActionHooks::default,
                            |hooks| hooks.on_activate.clone(),
                        );

                        if let Some(h) = hook {
                            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                            h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
                                },
                                ActivateReason::Keyboard,
                            );
                        }
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
            ElementInstance::PointerRegion(props) => {
                if !props.enabled {
                    return;
                }

                struct PointerHookHost<'a, H: UiHost> {
                    app: &'a mut H,
                    node: NodeId,
                    input_ctx: &'a fret_runtime::InputContext,
                    requested_capture: &'a mut Option<Option<NodeId>>,
                    requested_cursor: &'a mut Option<fret_core::CursorIcon>,
                }

                impl<H: UiHost> action::UiActionHost for PointerHookHost<'_, H> {
                    fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
                        self.app.models_mut()
                    }

                    fn push_effect(&mut self, effect: Effect) {
                        self.app.push_effect(effect);
                    }

                    fn request_redraw(&mut self, window: AppWindowId) {
                        self.app.request_redraw(window);
                    }

                    fn next_timer_token(&mut self) -> fret_core::TimerToken {
                        self.app.next_timer_token()
                    }
                }

                impl<H: UiHost> action::UiPointerActionHost for PointerHookHost<'_, H> {
                    fn capture_pointer(&mut self) {
                        *self.requested_capture = Some(Some(self.node));
                    }

                    fn release_pointer_capture(&mut self) {
                        *self.requested_capture = Some(None);
                    }

                    fn set_cursor_icon(&mut self, icon: fret_core::CursorIcon) {
                        if !self.input_ctx.caps.ui.cursor_icons {
                            return;
                        }
                        *self.requested_cursor = Some(icon);
                    }
                }

                match event {
                    Event::Pointer(fret_core::PointerEvent::Down {
                        position,
                        button,
                        modifiers,
                    }) => {
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PointerActionHooks::default,
                            |hooks| hooks.on_pointer_down.clone(),
                        );

                        let Some(h) = hook else {
                            return;
                        };

                        let down = action::PointerDownCx {
                            position: *position,
                            button: *button,
                            modifiers: *modifiers,
                        };

                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::PointerRegionState::default,
                            |state| {
                                state.last_down = Some(down);
                            },
                        );

                        let mut host = PointerHookHost {
                            app: &mut *cx.app,
                            node: cx.node,
                            input_ctx: &cx.input_ctx,
                            requested_capture: &mut cx.requested_capture,
                            requested_cursor: &mut cx.requested_cursor,
                        };
                        let handled = h(
                            &mut host,
                            action::ActionCx {
                                window,
                                target: self.element,
                            },
                            down,
                        );

                        if handled {
                            cx.stop_propagation();
                        }
                    }
                    Event::Pointer(fret_core::PointerEvent::Move {
                        position,
                        buttons,
                        modifiers,
                    }) => {
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PointerActionHooks::default,
                            |hooks| hooks.on_pointer_move.clone(),
                        );

                        let Some(h) = hook else {
                            return;
                        };

                        let mv = action::PointerMoveCx {
                            position: *position,
                            buttons: *buttons,
                            modifiers: *modifiers,
                        };

                        let mut host = PointerHookHost {
                            app: &mut *cx.app,
                            node: cx.node,
                            input_ctx: &cx.input_ctx,
                            requested_capture: &mut cx.requested_capture,
                            requested_cursor: &mut cx.requested_cursor,
                        };
                        let handled = h(
                            &mut host,
                            action::ActionCx {
                                window,
                                target: self.element,
                            },
                            mv,
                        );

                        if handled {
                            cx.stop_propagation();
                        }
                    }
                    Event::Pointer(fret_core::PointerEvent::Up {
                        position,
                        button,
                        modifiers,
                    }) => {
                        let was_captured = cx.captured == Some(cx.node);

                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PointerActionHooks::default,
                            |hooks| hooks.on_pointer_up.clone(),
                        );

                        let up = action::PointerUpCx {
                            position: *position,
                            button: *button,
                            modifiers: *modifiers,
                        };

                        if let Some(h) = hook {
                            let mut host = PointerHookHost {
                                app: &mut *cx.app,
                                node: cx.node,
                                input_ctx: &cx.input_ctx,
                                requested_capture: &mut cx.requested_capture,
                                requested_cursor: &mut cx.requested_cursor,
                            };
                            let handled = h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
                                },
                                up,
                            );

                            if handled {
                                cx.stop_propagation();
                            }
                        }

                        if was_captured {
                            cx.release_pointer_capture();
                        }
                    }
                    _ => {}
                }
            }
            ElementInstance::RovingFlex(props) => {
                if !props.roving.enabled {
                    return;
                }

                let Event::KeyDown { key, repeat, .. } = event else {
                    return;
                };
                if *repeat {
                    return;
                }

                enum Nav {
                    Prev,
                    Next,
                    Home,
                    End,
                }

                let nav = match (props.flex.direction, key) {
                    (_, fret_core::KeyCode::Home) => Some(Nav::Home),
                    (_, fret_core::KeyCode::End) => Some(Nav::End),
                    (fret_core::Axis::Vertical, fret_core::KeyCode::ArrowUp) => Some(Nav::Prev),
                    (fret_core::Axis::Vertical, fret_core::KeyCode::ArrowDown) => Some(Nav::Next),
                    (fret_core::Axis::Horizontal, fret_core::KeyCode::ArrowLeft) => Some(Nav::Prev),
                    (fret_core::Axis::Horizontal, fret_core::KeyCode::ArrowRight) => {
                        Some(Nav::Next)
                    }
                    _ => None,
                };
                let len = cx.children.len();
                if len == 0 {
                    return;
                }

                let current = cx
                    .focus
                    .and_then(|focus| cx.children.iter().position(|n| *n == focus));

                let is_disabled = |idx: usize| -> bool {
                    props.roving.disabled.get(idx).copied().unwrap_or(false)
                };

                let mut target: Option<usize> = None;
                match nav {
                    Some(Nav::Home) => {
                        target = (0..len).find(|&i| !is_disabled(i));
                    }
                    Some(Nav::End) => {
                        target = (0..len).rev().find(|&i| !is_disabled(i));
                    }
                    Some(Nav::Next) if props.roving.wrap => {
                        let Some(current) = current else {
                            return;
                        };
                        for step in 1..=len {
                            let idx = (current + step) % len;
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    }
                    Some(Nav::Prev) if props.roving.wrap => {
                        let Some(current) = current else {
                            return;
                        };
                        for step in 1..=len {
                            let idx = (current + len - (step % len)) % len;
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    }
                    Some(Nav::Next) => {
                        let Some(current) = current else {
                            return;
                        };
                        target = ((current + 1)..len).find(|&i| !is_disabled(i));
                    }
                    Some(Nav::Prev) => {
                        let Some(current) = current else {
                            return;
                        };
                        if current > 0 {
                            target = (0..current).rev().find(|&i| !is_disabled(i));
                        }
                    }
                    None => {}
                }

                let key_to_ascii = |key: fret_core::KeyCode| -> Option<char> {
                    use fret_core::KeyCode;
                    Some(match key {
                        KeyCode::KeyA => 'a',
                        KeyCode::KeyB => 'b',
                        KeyCode::KeyC => 'c',
                        KeyCode::KeyD => 'd',
                        KeyCode::KeyE => 'e',
                        KeyCode::KeyF => 'f',
                        KeyCode::KeyG => 'g',
                        KeyCode::KeyH => 'h',
                        KeyCode::KeyI => 'i',
                        KeyCode::KeyJ => 'j',
                        KeyCode::KeyK => 'k',
                        KeyCode::KeyL => 'l',
                        KeyCode::KeyM => 'm',
                        KeyCode::KeyN => 'n',
                        KeyCode::KeyO => 'o',
                        KeyCode::KeyP => 'p',
                        KeyCode::KeyQ => 'q',
                        KeyCode::KeyR => 'r',
                        KeyCode::KeyS => 's',
                        KeyCode::KeyT => 't',
                        KeyCode::KeyU => 'u',
                        KeyCode::KeyV => 'v',
                        KeyCode::KeyW => 'w',
                        KeyCode::KeyX => 'x',
                        KeyCode::KeyY => 'y',
                        KeyCode::KeyZ => 'z',
                        KeyCode::Digit0 => '0',
                        KeyCode::Digit1 => '1',
                        KeyCode::Digit2 => '2',
                        KeyCode::Digit3 => '3',
                        KeyCode::Digit4 => '4',
                        KeyCode::Digit5 => '5',
                        KeyCode::Digit6 => '6',
                        KeyCode::Digit7 => '7',
                        KeyCode::Digit8 => '8',
                        KeyCode::Digit9 => '9',
                        _ => return None,
                    })
                };

                if target.is_none()
                    && let Some(ch) = key_to_ascii(*key)
                {
                    let hook = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::action::RovingActionHooks::default,
                        |hooks| hooks.on_typeahead.clone(),
                    );

                    if let Some(h) = hook {
                        let tick = cx.app.tick_id().0;
                        let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                        target = h(
                            &mut host,
                            action::ActionCx {
                                window,
                                target: self.element,
                            },
                            crate::action::RovingTypeaheadCx {
                                input: ch,
                                current,
                                len,
                                disabled: props.roving.disabled.clone(),
                                wrap: props.roving.wrap,
                                tick,
                            },
                        );
                    }
                }

                let Some(target) = target else {
                    return;
                };
                if current.is_some_and(|current| target == current) {
                    return;
                }

                cx.request_focus(cx.children[target]);

                let hook = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::action::RovingActionHooks::default,
                    |hooks| hooks.on_active_change.clone(),
                );

                if let Some(h) = hook {
                    let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                    h(
                        &mut host,
                        action::ActionCx {
                            window,
                            target: self.element,
                        },
                        target,
                    );
                }

                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }

        if is_text_input
            && !cx.stop_propagation
            && let Event::KeyDown {
                key,
                modifiers,
                repeat,
            } = event
            && cx.focus == Some(cx.node)
            && try_key_hook(cx, *key, *modifiers, *repeat)
        {}
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
        if let Some(area) = self.text_area.as_mut() {
            area.cleanup_resources(services);
        }
        if let Some(slider) = self.slider.as_mut() {
            slider.cleanup_resources(services);
        }
        if let Some(group) = self.resizable_panel_group.as_mut() {
            group.cleanup_resources(services);
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
            ElementInstance::Text(props) => {
                cx.set_role(SemanticsRole::Text);
                cx.set_label(props.text.as_ref().to_string());
            }
            ElementInstance::Semantics(props) => {
                cx.set_role(props.role);
                if let Some(label) = props.label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                if props.disabled {
                    cx.set_disabled(true);
                }
                if props.selected {
                    cx.set_selected(true);
                }
                if let Some(expanded) = props.expanded {
                    cx.set_expanded(expanded);
                }
                if props.checked.is_some() {
                    cx.set_checked(props.checked);
                }
                if props.active_descendant.is_some() {
                    cx.set_active_descendant(props.active_descendant);
                }
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
                if let Some(label) = props.a11y_label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                cx.set_active_descendant(props.active_descendant);
                input.semantics(cx);
            }
            ElementInstance::TextArea(props) => {
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(props.model));
                }
                let area = self.text_area.as_mut().expect("text area");
                if area.model_id() != props.model.id() {
                    area.set_model(props.model);
                }
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);
                if let Some(label) = props.a11y_label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                area.semantics(cx);
            }
            ElementInstance::Slider(props) => {
                if self.slider.is_none() {
                    self.slider = Some(crate::slider::BoundSlider::new(props.model));
                }
                let slider = self.slider.as_mut().expect("slider");
                if slider.model_id() != props.model.id() {
                    slider.set_model(props.model);
                }
                slider.set_range(props.min, props.max);
                slider.set_step(props.step);
                slider.set_enabled(props.enabled);
                slider.set_style(props.chrome);
                if let Some(label) = props.a11y_label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                slider.semantics(cx);
            }
            ElementInstance::ResizablePanelGroup(props) => {
                if self.resizable_panel_group.is_none() {
                    self.resizable_panel_group =
                        Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                            props.axis,
                            props.model,
                        ));
                }
                let group = self
                    .resizable_panel_group
                    .as_mut()
                    .expect("resizable panel group");
                if group.model_id() != props.model.id() {
                    group.set_model(props.model);
                }
                group.set_axis(props.axis);
                group.set_enabled(props.enabled);
                group.set_min_px(props.min_px.clone());
                group.set_style(props.chrome.clone());
                group.semantics(cx);
            }
            ElementInstance::Pressable(props) => {
                cx.set_role(props.a11y.role.unwrap_or(SemanticsRole::Button));
                if let Some(label) = props.a11y.label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                if props.a11y.selected {
                    cx.set_selected(true);
                }
                if let Some(expanded) = props.a11y.expanded {
                    cx.set_expanded(expanded);
                }
                if props.a11y.checked.is_some() {
                    cx.set_checked(props.a11y.checked);
                }
                cx.set_disabled(!props.enabled);
                cx.set_focusable(props.enabled);
                cx.set_invokable(props.enabled);
                cx.set_collection_position(props.a11y.pos_in_set, props.a11y.set_size);
            }
            ElementInstance::VirtualList(_) => {
                cx.set_role(SemanticsRole::List);
            }
            ElementInstance::Flex(_)
            | ElementInstance::DismissibleLayer(_)
            | ElementInstance::RovingFlex(_)
            | ElementInstance::Grid(_) => {
                // Flex/Grid are layout containers; they do not imply semantics beyond their children.
            }
            ElementInstance::Image(_)
            | ElementInstance::PointerRegion(_)
            | ElementInstance::HoverRegion(_)
            | ElementInstance::Spinner(_)
            | ElementInstance::Opacity(_)
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

        crate::elements::record_bounds_for_element(&mut *cx.app, window, self.element, cx.bounds);

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
            ElementInstance::PointerRegion(p) => p.enabled,
            ElementInstance::Slider(p) => p.enabled,
            ElementInstance::Semantics(_) => false,
            ElementInstance::DismissibleLayer(_) => false,
            ElementInstance::Opacity(_) => false,
            ElementInstance::Spinner(_) => false,
            _ => true,
        };
        self.hit_test_children = match &instance {
            ElementInstance::Pressable(p) => p.enabled,
            ElementInstance::PointerRegion(_) => true,
            ElementInstance::Semantics(_) => true,
            ElementInstance::DismissibleLayer(_) => true,
            ElementInstance::Spinner(_) => false,
            _ => true,
        };
        self.is_text_input = matches!(
            &instance,
            ElementInstance::TextInput(_) | ElementInstance::TextArea(_)
        );
        self.is_focusable = match &instance {
            ElementInstance::TextInput(_) | ElementInstance::TextArea(_) => true,
            ElementInstance::Pressable(p) => p.enabled && p.focusable,
            ElementInstance::Slider(p) => p.enabled,
            _ => false,
        };
        self.clips_hit_test = match &instance {
            ElementInstance::Container(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Semantics(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Opacity(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Pressable(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::PointerRegion(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::DismissibleLayer(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Stack(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Flex(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::RovingFlex(p) => matches!(p.flex.layout.overflow, Overflow::Clip),
            ElementInstance::Grid(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::TextInput(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::TextArea(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Slider(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::ResizablePanelGroup(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Scroll(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::HoverRegion(p) => matches!(p.layout.overflow, Overflow::Clip),
            // These primitives are always hit-test clipped by their own bounds (they are not
            // intended as overflow-visible containers).
            ElementInstance::VirtualList(_)
            | ElementInstance::Image(_)
            | ElementInstance::SvgIcon(_)
            | ElementInstance::Spinner(_)
            | ElementInstance::Text(_) => true,
            ElementInstance::Spacer(_) => true,
        };
        self.clip_hit_test_corner_radii = match &instance {
            ElementInstance::Container(p) if matches!(p.layout.overflow, Overflow::Clip) => {
                if p.corner_radii.top_left.0 > 0.0
                    || p.corner_radii.top_right.0 > 0.0
                    || p.corner_radii.bottom_right.0 > 0.0
                    || p.corner_radii.bottom_left.0 > 0.0
                {
                    Some(p.corner_radii)
                } else {
                    None
                }
            }
            _ => None,
        };
        self.scrollbar_hit_rect = None;

        let is_flex = matches!(&instance, ElementInstance::Flex(_));
        let is_roving_flex = matches!(&instance, ElementInstance::RovingFlex(_));
        let is_grid = matches!(&instance, ElementInstance::Grid(_));
        if !is_flex && !is_roving_flex {
            self.flex_cache = None;
        }
        if !is_grid {
            self.grid_cache = None;
        }

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

                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, inner_avail);
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
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
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
            ElementInstance::Semantics(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
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
            ElementInstance::Opacity(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
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
            ElementInstance::DismissibleLayer(props) => {
                let desired = clamp_to_constraints(cx.available, props.layout, cx.available);
                let base = cx.bounds;
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::Stack(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
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
            ElementInstance::TextArea(props) => {
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(props.model));
                }
                let area = self.text_area.as_mut().expect("text area");
                if area.model_id() != props.model.id() {
                    area.set_model(props.model);
                }
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);

                let desired = area.layout(cx);
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::Slider(props) => {
                if self.slider.is_none() {
                    self.slider = Some(crate::slider::BoundSlider::new(props.model));
                }
                let slider = self.slider.as_mut().expect("slider");
                if slider.model_id() != props.model.id() {
                    slider.set_model(props.model);
                }
                slider.set_range(props.min, props.max);
                slider.set_step(props.step);
                slider.set_enabled(props.enabled);
                slider.set_style(props.chrome.clone());

                let desired = slider.layout(cx);
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::ResizablePanelGroup(props) => {
                if self.resizable_panel_group.is_none() {
                    self.resizable_panel_group =
                        Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                            props.axis,
                            props.model,
                        ));
                }
                let group = self
                    .resizable_panel_group
                    .as_mut()
                    .expect("resizable panel group");
                if group.model_id() != props.model.id() {
                    group.set_model(props.model);
                }
                group.set_axis(props.axis);
                group.set_enabled(props.enabled);
                group.set_min_px(props.min_px.clone());
                group.set_style(props.chrome.clone());

                let desired = group.layout(cx);
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::VirtualList(props) => {
                let mut metrics = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::VirtualListState::default,
                    |state| {
                        state.metrics.ensure(
                            props.len,
                            props.estimate_row_height,
                            props.gap,
                            props.scroll_margin,
                        );
                        state.metrics.clone()
                    },
                );
                let content_h = metrics.total_height();

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
                let viewport_h = Px(size.height.0.max(0.0));
                let mut needs_redraw = false;

                props.scroll_handle.set_items_count(props.len);

                let prev_offset = props.scroll_handle.offset();
                let mut offset_y = metrics.clamp_offset(prev_offset.y, viewport_h);

                // Avoid consuming deferred scroll requests during "probe" layout passes that use
                // an effectively-unbounded available height (e.g. Stack/Pressable measuring with
                // `Px(1.0e9)`). Those passes are not the final viewport constraints and would
                // otherwise clear the request before the real layout happens.
                let is_probe_layout = cx.available.height.0 >= 1.0e8;

                if !is_probe_layout
                    && viewport_h.0 > 0.0
                    && props.len > 0
                    && let Some((index, strategy)) = props.scroll_handle.deferred_scroll_to_item()
                {
                    offset_y =
                        metrics.scroll_offset_for_item(index, viewport_h, offset_y, strategy);
                    props.scroll_handle.clear_deferred_scroll_to_item();
                }

                offset_y = metrics.clamp_offset(offset_y, viewport_h);

                if (prev_offset.y.0 - offset_y.0).abs() > 0.01 {
                    needs_redraw = true;
                }
                props
                    .scroll_handle
                    .set_offset(fret_core::Point::new(prev_offset.x, offset_y));

                props
                    .scroll_handle
                    .set_viewport_size(Size::new(size.width, size.height));
                props
                    .scroll_handle
                    .set_content_size(Size::new(size.width, content_h));

                let mut measured_updates: Vec<(usize, crate::ItemKey, Px)> =
                    Vec::with_capacity(cx.children.len());

                for (&child, item) in cx.children.iter().zip(props.visible_items.iter()) {
                    let idx = item.index;
                    let key = item.key;
                    let y = cx.bounds.origin.y.0 + metrics.offset_for_index(idx).0 - offset_y.0;
                    let origin = fret_core::Point::new(cx.bounds.origin.x, Px(y));

                    let measure_bounds = Rect::new(origin, Size::new(size.width, Px(1.0e9)));
                    let measured = cx.layout_in(child, measure_bounds);
                    let measured_h = Px(measured.height.0.max(0.0));

                    measured_updates.push((idx, key, measured_h));
                    if metrics.set_measured_height(idx, measured_h) {
                        needs_redraw = true;
                    }

                    let child_bounds = Rect::new(origin, Size::new(size.width, measured_h));
                    let _ = cx.layout_in(child, child_bounds);
                }

                let content_h = metrics.total_height();
                props
                    .scroll_handle
                    .set_viewport_size(Size::new(size.width, viewport_h));
                props
                    .scroll_handle
                    .set_content_size(Size::new(size.width, content_h));

                let prev_offset = props.scroll_handle.offset();
                let clamped = metrics.clamp_offset(prev_offset.y, viewport_h);
                if (clamped.0 - prev_offset.y.0).abs() > 0.01 {
                    needs_redraw = true;
                }
                props
                    .scroll_handle
                    .set_offset(fret_core::Point::new(prev_offset.x, clamped));
                offset_y = clamped;

                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::VirtualListState::default,
                    |state| {
                        for (idx, key, h) in &measured_updates {
                            state.size_cache.insert(*key, *h);
                            if let Some(slot) = state.keys.get_mut(*idx) {
                                *slot = *key;
                            }
                        }
                        state.offset_y = offset_y;
                        if state.viewport_h != viewport_h {
                            state.viewport_h = viewport_h;
                            needs_redraw = true;
                        }
                        state.items_revision = props.items_revision;
                        state.metrics = metrics;
                    },
                );

                if needs_redraw && let Some(window) = cx.window {
                    cx.app.request_redraw(window);
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
                            Length::Fill => Dimension::length(inner_avail.width.0.max(0.0)),
                            Length::Auto => Dimension::auto(),
                        },
                        height: match props.layout.size.height {
                            Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0)),
                            Length::Fill => Dimension::length(inner_avail.height.0.max(0.0)),
                            Length::Auto => Dimension::auto(),
                        },
                    },
                    max_size: TaffySize {
                        width: Dimension::length(inner_avail.width.0.max(0.0)),
                        height: Dimension::length(inner_avail.height.0.max(0.0)),
                    },
                    ..Default::default()
                };

                let cache = self
                    .flex_cache
                    .get_or_insert_with(TaffyContainerCache::default);

                cache.sync_root_style(root_style);
                cache.sync_children(cx.children, |child| {
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

                    TaffyStyle {
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
                        margin: taffy_rect_lpa_from_margin_edges(layout_style.margin),
                        flex_grow: layout_style.flex.grow.max(0.0),
                        flex_shrink: layout_style.flex.shrink.max(0.0),
                        flex_basis: taffy_dimension(layout_style.flex.basis),
                        align_self: layout_style.flex.align_self.map(taffy_align_self),
                        ..Default::default()
                    }
                });

                cache
                    .taffy
                    .mark_dirty(cache.root)
                    .expect("taffy mark dirty");

                cache.measure_cache.clear();
                let root = cache.root;

                {
                    let measure_cache = &mut cache.measure_cache;
                    let taffy = &mut cache.taffy;

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

                                let key = TaffyMeasureKey {
                                    child,
                                    known_w: known.width.map(|v| v.to_bits()),
                                    known_h: known.height.map(|v| v.to_bits()),
                                    avail_w: taffy_available_space_key(avail.width),
                                    avail_h: taffy_available_space_key(avail.height),
                                };
                                if let Some(size) = measure_cache.get(&key) {
                                    return *size;
                                }

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
                                let out = taffy::geometry::Size {
                                    width: s.width.0,
                                    height: s.height.0,
                                };
                                measure_cache.insert(key, out);
                                out
                            },
                        )
                        .expect("taffy compute");
                }

                let taffy = &cache.taffy;
                let root_layout = taffy.layout(root).expect("taffy root layout");
                let container_inner_size = Size::new(
                    Px(root_layout.size.width.max(0.0)),
                    Px(root_layout.size.height.max(0.0)),
                );
                let auto_margin_inner_size = Size::new(
                    match props.layout.size.width {
                        Length::Fill => inner_avail.width,
                        _ => container_inner_size.width,
                    },
                    match props.layout.size.height {
                        Length::Fill => inner_avail.height,
                        _ => container_inner_size.height,
                    },
                );

                for &child_node in &cache.child_nodes {
                    let layout = taffy.layout(child_node).expect("taffy layout");
                    let Some(child) = taffy.get_node_context(child_node).and_then(|c| *c) else {
                        continue;
                    };
                    let child_style = layout_style_for_node(cx.app, window, child);
                    let single_child = cx.children.len() == 1;

                    let mut x = layout.location.x;
                    let mut y = layout.location.y;

                    let margin_left_auto =
                        matches!(child_style.margin.left, crate::element::MarginEdge::Auto);
                    let margin_right_auto =
                        matches!(child_style.margin.right, crate::element::MarginEdge::Auto);
                    let margin_top_auto =
                        matches!(child_style.margin.top, crate::element::MarginEdge::Auto);
                    let margin_bottom_auto =
                        matches!(child_style.margin.bottom, crate::element::MarginEdge::Auto);

                    let margin_px = |edge: crate::element::MarginEdge| match edge {
                        crate::element::MarginEdge::Px(px) => px.0,
                        crate::element::MarginEdge::Auto => 0.0,
                    };

                    match props.direction {
                        fret_core::Axis::Horizontal => {
                            if single_child && (margin_left_auto || margin_right_auto) {
                                let left = if margin_left_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.left)
                                };
                                let right = if margin_right_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.right)
                                };
                                let free = auto_margin_inner_size.width.0
                                    - layout.size.width
                                    - left
                                    - right;
                                if margin_left_auto && margin_right_auto {
                                    x = (left + (free.max(0.0) / 2.0)).max(0.0);
                                } else if margin_left_auto {
                                    x = (left + free.max(0.0)).max(0.0);
                                } else if margin_right_auto {
                                    x = left.max(0.0);
                                }
                            }

                            if margin_top_auto || margin_bottom_auto {
                                let top = if margin_top_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.top)
                                };
                                let bottom = if margin_bottom_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.bottom)
                                };
                                let free = auto_margin_inner_size.height.0
                                    - layout.size.height
                                    - top
                                    - bottom;
                                if margin_top_auto && margin_bottom_auto {
                                    y = (top + (free.max(0.0) / 2.0)).max(0.0);
                                } else if margin_top_auto {
                                    y = (top + free.max(0.0)).max(0.0);
                                } else if margin_bottom_auto {
                                    y = top.max(0.0);
                                }
                            }
                        }
                        fret_core::Axis::Vertical => {
                            if single_child && (margin_top_auto || margin_bottom_auto) {
                                let top = if margin_top_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.top)
                                };
                                let bottom = if margin_bottom_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.bottom)
                                };
                                let free = auto_margin_inner_size.height.0
                                    - layout.size.height
                                    - top
                                    - bottom;
                                if margin_top_auto && margin_bottom_auto {
                                    y = (top + (free.max(0.0) / 2.0)).max(0.0);
                                } else if margin_top_auto {
                                    y = (top + free.max(0.0)).max(0.0);
                                } else if margin_bottom_auto {
                                    y = top.max(0.0);
                                }
                            }

                            if margin_left_auto || margin_right_auto {
                                let left = if margin_left_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.left)
                                };
                                let right = if margin_right_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.right)
                                };
                                let free = auto_margin_inner_size.width.0
                                    - layout.size.width
                                    - left
                                    - right;
                                if margin_left_auto && margin_right_auto {
                                    x = (left + (free.max(0.0) / 2.0)).max(0.0);
                                } else if margin_left_auto {
                                    x = (left + free.max(0.0)).max(0.0);
                                } else if margin_right_auto {
                                    x = left.max(0.0);
                                }
                            }
                        }
                    }
                    let rect = Rect::new(
                        fret_core::Point::new(Px(inner_origin.x.0 + x), Px(inner_origin.y.0 + y)),
                        Size::new(Px(layout.size.width), Px(layout.size.height)),
                    );
                    let _ = cx.layout_in(child, rect);
                }

                let desired = Size::new(
                    Px((container_inner_size.width.0 + pad_w).max(0.0)),
                    Px((container_inner_size.height.0 + pad_h).max(0.0)),
                );
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::RovingFlex(props) => {
                self.layout_flex_container(cx, window, props.flex)
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
                            Length::Fill => Dimension::length(inner_avail.width.0.max(0.0)),
                            Length::Auto => Dimension::auto(),
                        },
                        height: match props.layout.size.height {
                            Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0)),
                            Length::Fill => Dimension::length(inner_avail.height.0.max(0.0)),
                            Length::Auto => Dimension::auto(),
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

                let cache = self
                    .grid_cache
                    .get_or_insert_with(TaffyContainerCache::default);

                cache.sync_root_style(root_style);
                cache.sync_children(cx.children, |child| {
                    let layout_style = layout_style_for_node(cx.app, window, child);

                    TaffyStyle {
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
                        margin: taffy_rect_lpa_from_margin_edges(layout_style.margin),
                        grid_column: taffy_grid_line(layout_style.grid.column),
                        grid_row: taffy_grid_line(layout_style.grid.row),
                        ..Default::default()
                    }
                });

                cache
                    .taffy
                    .mark_dirty(cache.root)
                    .expect("taffy mark dirty");

                cache.measure_cache.clear();
                let root = cache.root;

                {
                    let measure_cache = &mut cache.measure_cache;
                    let taffy = &mut cache.taffy;

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

                                let key = TaffyMeasureKey {
                                    child,
                                    known_w: known.width.map(|v| v.to_bits()),
                                    known_h: known.height.map(|v| v.to_bits()),
                                    avail_w: taffy_available_space_key(avail.width),
                                    avail_h: taffy_available_space_key(avail.height),
                                };
                                if let Some(size) = measure_cache.get(&key) {
                                    return *size;
                                }

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
                                let out = taffy::geometry::Size {
                                    width: s.width.0,
                                    height: s.height.0,
                                };
                                measure_cache.insert(key, out);
                                out
                            },
                        )
                        .expect("taffy compute");
                }

                let taffy = &cache.taffy;

                for &child_node in &cache.child_nodes {
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
                clamp_to_constraints(cx.available, props.layout, cx.available)
            }
            ElementInstance::SvgIcon(props) => {
                clamp_to_constraints(cx.available, props.layout, cx.available)
            }
            ElementInstance::Spinner(props) => {
                clamp_to_constraints(Size::new(Px(16.0), Px(16.0)), props.layout, cx.available)
            }
            ElementInstance::PointerRegion(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
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
            ElementInstance::HoverRegion(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
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

                // Avoid mutating the imperative handle during "probe" layout passes that use an
                // effectively-unbounded available height (e.g. Stack/Pressable measuring with
                // `Px(1.0e9)`), otherwise scroll position can be clamped to zero prematurely.
                let is_probe_layout = cx.available.height.0 >= 1.0e8;
                let external_handle = props.scroll_handle.clone();
                let offset_y = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::ScrollState::default,
                    |state| {
                        let handle = external_handle.as_ref().unwrap_or(&state.scroll_handle);
                        if !is_probe_layout {
                            handle.set_viewport_size(desired);
                            handle.set_content_size(Size::new(max_child.width, content_h));
                            let prev = handle.offset();
                            handle.set_offset(prev);
                        }
                        handle.offset().y
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

                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    Some(props.corner_radii),
                );
            }
            ElementInstance::Semantics(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Opacity(props) => {
                let opacity = props.opacity.clamp(0.0, 1.0);
                if opacity <= 0.0 {
                    return;
                }

                if opacity < 1.0 {
                    cx.scene.push(SceneOp::PushOpacity { opacity });
                }

                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );

                if opacity < 1.0 {
                    cx.scene.push(SceneOp::PopOpacity);
                }
            }
            ElementInstance::DismissibleLayer(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Stack(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Flex(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::RovingFlex(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.flex.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Grid(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Spacer(_props) => {}
            ElementInstance::Pressable(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );

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
            ElementInstance::TextArea(props) => {
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(props.model));
                }
                let area = self.text_area.as_mut().expect("text area");
                if area.model_id() != props.model.id() {
                    area.set_model(props.model);
                }
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);
                area.paint(cx);
            }
            ElementInstance::Slider(props) => {
                if self.slider.is_none() {
                    self.slider = Some(crate::slider::BoundSlider::new(props.model));
                }
                let slider = self.slider.as_mut().expect("slider");
                if slider.model_id() != props.model.id() {
                    slider.set_model(props.model);
                }
                slider.set_range(props.min, props.max);
                slider.set_step(props.step);
                slider.set_enabled(props.enabled);
                slider.set_style(props.chrome.clone());
                slider.paint(cx);
            }
            ElementInstance::ResizablePanelGroup(props) => {
                if self.resizable_panel_group.is_none() {
                    self.resizable_panel_group =
                        Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                            props.axis,
                            props.model,
                        ));
                }
                let group = self
                    .resizable_panel_group
                    .as_mut()
                    .expect("resizable panel group");
                if group.model_id() != props.model.id() {
                    group.set_model(props.model);
                }
                group.set_axis(props.axis);
                group.set_enabled(props.enabled);
                group.set_min_px(props.min_px.clone());
                group.set_style(props.chrome.clone());
                group.paint(cx);
            }
            ElementInstance::VirtualList(props) => {
                cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });

                let offset_y = props.scroll_handle.offset().y;
                let metrics = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::VirtualListState::default,
                    |state| {
                        state.metrics.ensure(
                            props.len,
                            props.estimate_row_height,
                            props.gap,
                            props.scroll_margin,
                        );
                        state.metrics.clone()
                    },
                );

                for (&child, item) in cx.children.iter().zip(props.visible_items.iter()) {
                    let idx = item.index;
                    let y = cx.bounds.origin.y.0 + metrics.offset_for_index(idx).0 - offset_y.0;
                    let row_h = metrics.height_at(idx);
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
            ElementInstance::SvgIcon(props) => {
                let opacity = props.opacity.clamp(0.0, 1.0);
                if opacity <= 0.0 || props.color.a <= 0.0 {
                    return;
                }

                let svg = props.svg.resolve(cx.services);
                cx.scene.push(SceneOp::SvgMaskIcon {
                    order: DrawOrder(0),
                    rect: cx.bounds,
                    svg,
                    fit: props.fit,
                    color: props.color,
                    opacity,
                });
            }
            ElementInstance::Spinner(props) => {
                let theme = cx.theme();
                let base = props
                    .color
                    .or_else(|| theme.color_by_key("muted-foreground"))
                    .unwrap_or(theme.colors.text_muted);

                let n = props.dot_count.clamp(1, 32) as usize;

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
            ElementInstance::HoverRegion(props) => {
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
            }
            ElementInstance::PointerRegion(props) => {
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
                    let external_handle = props.scroll_handle.clone();
                    let (internal_handle, hovered, dragging) = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::element::ScrollState::default,
                        |state| {
                            (
                                state.scroll_handle.clone(),
                                state.hovered_scrollbar,
                                state.dragging_thumb,
                            )
                        },
                    );
                    let handle = external_handle.as_ref().unwrap_or(&internal_handle);
                    let offset_y = handle.offset().y;
                    let viewport_h = handle.viewport_size().height;
                    let content_h = handle.content_size().height;
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
                let node = ui.create_node(ElementHostWidget {
                    element: root_id,
                    text_cache: TextCache::default(),
                    hit_testable: true,
                    hit_test_children: true,
                    is_focusable: false,
                    is_text_input: false,
                    clips_hit_test: true,
                    clip_hit_test_corner_radii: None,
                    scrollbar_hit_rect: None,
                    text_input: None,
                    text_area: None,
                    slider: None,
                    resizable_panel_group: None,
                    flex_cache: None,
                    grid_cache: None,
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
                let node = ui.create_node(ElementHostWidget {
                    element: root_id,
                    text_cache: TextCache::default(),
                    hit_testable: true,
                    hit_test_children: true,
                    is_focusable: false,
                    is_text_input: false,
                    clips_hit_test: true,
                    clip_hit_test_corner_radii: None,
                    scrollbar_hit_rect: None,
                    text_input: None,
                    text_area: None,
                    slider: None,
                    resizable_panel_group: None,
                    flex_cache: None,
                    grid_cache: None,
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
            let node = ui.create_node(ElementHostWidget {
                element: id,
                text_cache: TextCache::default(),
                hit_testable: true,
                hit_test_children: true,
                is_focusable: false,
                is_text_input: false,
                clips_hit_test: true,
                clip_hit_test_corner_radii: None,
                scrollbar_hit_rect: None,
                text_input: None,
                text_area: None,
                slider: None,
                resizable_panel_group: None,
                flex_cache: None,
                grid_cache: None,
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
        ElementKind::Semantics(p) => ElementInstance::Semantics(p),
        ElementKind::Opacity(p) => ElementInstance::Opacity(p),
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
        ElementKind::Slider(p) => ElementInstance::Slider(p),
        ElementKind::ResizablePanelGroup(p) => ElementInstance::ResizablePanelGroup(p),
        ElementKind::VirtualList(p) => ElementInstance::VirtualList(p),
        ElementKind::Flex(p) => ElementInstance::Flex(p),
        ElementKind::Grid(p) => ElementInstance::Grid(p),
        ElementKind::Image(p) => ElementInstance::Image(p),
        ElementKind::SvgIcon(p) => ElementInstance::SvgIcon(p),
        ElementKind::Spinner(p) => ElementInstance::Spinner(p),
        ElementKind::HoverRegion(p) => ElementInstance::HoverRegion(p),
        ElementKind::Scroll(p) => ElementInstance::Scroll(p),
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
mod tests {
    use std::sync::Arc;

    use super::render_root;
    use crate::UiHost;
    use crate::action::{ActivateReason, DismissReason};
    use crate::element::{AnyElement, CrossAlign, Length, MainAlign, TextInputProps};
    use crate::elements::{ContinuousFrames, ElementCx};
    use crate::test_host::TestHost;
    use crate::tree::UiTree;
    use crate::widget::Invalidation;
    use crate::widget::{LayoutCx, PaintCx, Widget};
    use fret_core::{
        AppWindowId, Color, Modifiers, MouseButton, MouseButtons, NodeId, Point, Px, Rect, Scene,
        SceneOp, Size, TextConstraints, TextMetrics, TextService, TextStyle,
    };
    use fret_runtime::{CommandId, Effect};

    #[derive(Default)]
    struct FakeTextService {}

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

    impl fret_core::SvgService for FakeTextService {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    #[derive(Default)]
    struct FillStack;

    impl<H: UiHost> Widget<H> for FillStack {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in cx.children {
                let _ = cx.layout(child, cx.available);
            }
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            for &child in cx.children {
                cx.paint(child, cx.bounds);
            }
        }
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
    fn opacity_element_emits_opacity_stack_ops() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(80.0)),
        );
        let mut services = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "opacity-element-emits-ops",
            |cx| {
                vec![cx.opacity(0.5, |cx| {
                    let mut props = crate::element::ContainerProps::default();
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Fill;
                    props.background = Some(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    });
                    vec![cx.container(props, |_| Vec::new())]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        assert_eq!(scene.ops_len(), 3);
        assert!(matches!(
            scene.ops()[0],
            SceneOp::PushOpacity { opacity } if (opacity - 0.5).abs() < 1e-6
        ));
        assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
        assert!(matches!(scene.ops()[2], SceneOp::PopOpacity));
    }

    #[test]
    fn key_hook_runs_for_focused_text_input() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let value = app.models_mut().insert(String::new());
        let invoked = app.models_mut().insert(0u32);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(80.0)),
        );
        let mut text = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "key-hook-text-input",
            |cx| {
                let mut props = TextInputProps::new(value);
                props.layout.size.width = Length::Px(Px(160.0));
                props.layout.size.height = Length::Px(Px(32.0));
                let input = cx.text_input(props);

                cx.key_on_key_down_for(
                    input.id,
                    Arc::new(move |host, _cx, down| {
                        if down.repeat {
                            return false;
                        }
                        if down.key != fret_core::KeyCode::ArrowDown {
                            return false;
                        }
                        let _ = host.models_mut().update(invoked, |v| *v += 1);
                        true
                    }),
                );

                vec![input]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let input_node = ui.children(root)[0];
        let input_bounds = ui.debug_node_bounds(input_node).expect("input bounds");
        let pos = Point::new(
            Px(input_bounds.origin.x.0 + 2.0),
            Px(input_bounds.origin.y.0 + 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        assert_eq!(app.models().get(invoked).copied().unwrap_or_default(), 1);
    }

    #[test]
    fn continuous_frames_lease_requests_animation_frames_while_held() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(500.0)),
        );
        let mut services = FakeTextService::default();

        let mut lease: Option<ContinuousFrames> = None;

        let _root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "root",
            |cx| {
                lease = Some(cx.begin_continuous_frames());
                Vec::<AnyElement>::new()
            },
        );

        let effects = app.take_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
            "expected RequestAnimationFrame while beginning a continuous frames lease"
        );

        app.advance_frame();
        let _root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "root",
            |_cx| Vec::<AnyElement>::new(),
        );

        let effects = app.take_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
            "expected RequestAnimationFrame while continuous frames lease is held"
        );

        drop(lease.take());
        app.advance_frame();
        let _root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "root",
            |_cx| Vec::<AnyElement>::new(),
        );

        let effects = app.take_effects();
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
            "did not expect RequestAnimationFrame after dropping the last continuous frames lease"
        );
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

    #[test]
    fn declarative_text_sets_semantics_label() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(60.0)),
        );
        let mut services = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "a11y-text",
            |cx| vec![cx.text("Hello declarative")],
        );
        ui.set_root(root);

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        // Root is a host widget, so text is in a descendant; ensure at least one Text node carries
        // the label payload.
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == fret_core::SemanticsRole::Text
                    && n.label.as_deref() == Some("Hello declarative")),
            "expected a Text semantics node with label"
        );
    }

    #[test]
    fn declarative_text_input_sets_semantics_label() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(60.0)),
        );
        let mut services = FakeTextService::default();

        let model = app.models_mut().insert("hello".to_string());
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "a11y-text-input-label",
            |cx| {
                let mut props = crate::element::TextInputProps::new(model);
                props.a11y_label = Some("Search".into());
                vec![cx.text_input(props)]
            },
        );
        ui.set_root(root);

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == fret_core::SemanticsRole::TextField
                    && n.label.as_deref() == Some("Search")),
            "expected a TextField semantics node with label"
        );
    }

    #[test]
    fn declarative_text_area_updates_model_on_text_input() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );
        let mut services = FakeTextService::default();

        let model = app.models_mut().insert(String::new());
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "text-area-text-input",
            |cx| {
                let mut props = crate::element::TextAreaProps::new(model);
                props.min_height = Px(80.0);
                vec![cx.text_area(props)]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text area");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::TextInput("hello\nworld".to_string()),
        );
        assert_eq!(
            app.models().get(model).map(|s| s.as_str()),
            Some("hello\nworld")
        );
    }

    #[test]
    fn declarative_slider_updates_model_on_pointer_down() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(40.0)),
        );
        let mut services = FakeTextService::default();

        let model = app.models_mut().insert(vec![0.0]);
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "slider-pointer-down",
            |cx| {
                let mut props = crate::element::SliderProps::new(model);
                props.min = 0.0;
                props.max = 100.0;
                props.step = 1.0;
                vec![cx.slider(props)]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let slider_node = ui.children(root)[0];
        let slider_bounds = ui.debug_node_bounds(slider_node).expect("slider bounds");
        let position = Point::new(
            Px(slider_bounds.origin.x.0 + slider_bounds.size.width.0 - 1.0),
            Px(slider_bounds.origin.y.0 + slider_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        let v = app
            .models()
            .get(model)
            .and_then(|values| values.first())
            .copied()
            .unwrap_or(f32::NAN);
        assert!((v - 100.0).abs() < 0.01, "expected slider=100, got {v}");
    }

    #[test]
    fn declarative_pointer_region_can_capture_and_receive_move_up() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(60.0)),
        );
        let mut services = FakeTextService::default();

        let counter = app.models_mut().insert(0u32);
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "pointer-region-capture-move-up",
            |cx| {
                let counter_down = counter;
                let counter_move = counter;
                let counter_up = counter;

                let on_down = Arc::new(
                    move |host: &mut dyn crate::action::UiPointerActionHost,
                          cx: crate::action::ActionCx,
                          down: crate::action::PointerDownCx| {
                        if down.button != MouseButton::Left {
                            return false;
                        }
                        host.capture_pointer();
                        let _ = host
                            .models_mut()
                            .update(counter_down, |v| *v = v.saturating_add(1));
                        host.request_redraw(cx.window);
                        true
                    },
                );

                let on_move = Arc::new(
                    move |host: &mut dyn crate::action::UiPointerActionHost,
                          _cx: crate::action::ActionCx,
                          _mv: crate::action::PointerMoveCx| {
                        let _ = host
                            .models_mut()
                            .update(counter_move, |v| *v = v.saturating_add(10));
                        true
                    },
                );

                let on_up = Arc::new(
                    move |host: &mut dyn crate::action::UiPointerActionHost,
                          cx: crate::action::ActionCx,
                          up: crate::action::PointerUpCx| {
                        if up.button == MouseButton::Left {
                            host.release_pointer_capture();
                        }
                        let _ = host
                            .models_mut()
                            .update(counter_up, |v| *v = v.saturating_add(100));
                        host.request_redraw(cx.window);
                        true
                    },
                );

                let mut props = crate::element::PointerRegionProps::default();
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;

                vec![cx.pointer_region(props, |cx| {
                    cx.pointer_region_on_pointer_down(on_down);
                    cx.pointer_region_on_pointer_move(on_move);
                    cx.pointer_region_on_pointer_up(on_up);
                    Vec::new()
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let region = ui.children(root)[0];
        let region_bounds = ui.debug_node_bounds(region).expect("pointer region bounds");

        let inside = Point::new(
            Px(region_bounds.origin.x.0 + 5.0),
            Px(region_bounds.origin.y.0 + 5.0),
        );
        let outside = Point::new(Px(region_bounds.origin.x.0 + 250.0), inside.y);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: inside,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: outside,
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: Modifiers::default(),
            }),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: outside,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        assert_eq!(app.models().get(counter).copied(), Some(111));
    }

    #[test]
    fn declarative_resizable_panel_group_updates_model_on_drag() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(40.0)),
        );
        let mut services = FakeTextService::default();

        let model = app.models_mut().insert(vec![0.33, 0.34, 0.33]);
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "resizable-panel-group-drag",
            |cx| {
                let mut props = crate::element::ResizablePanelGroupProps::new(
                    fret_core::Axis::Horizontal,
                    model,
                );
                props.min_px = vec![Px(10.0)];
                props.chrome = crate::ResizablePanelGroupStyle {
                    hit_thickness: Px(10.0),
                    ..Default::default()
                };
                vec![cx.resizable_panel_group(props, |cx| {
                    vec![
                        cx.spacer(crate::element::SpacerProps::default()),
                        cx.spacer(crate::element::SpacerProps::default()),
                        cx.spacer(crate::element::SpacerProps::default()),
                    ]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let fractions_now = app.models().get(model).cloned().unwrap_or_default();
        let layout = crate::resizable_panel_group::compute_resizable_panel_group_layout(
            fret_core::Axis::Horizontal,
            bounds,
            3,
            fractions_now,
            Px(0.0),
            Px(10.0),
            &[Px(10.0)],
        );
        let down_x = layout.handle_centers.first().copied().unwrap_or(0.0);
        let down = Point::new(Px(down_x), Px(20.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: down,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(128.0), Px(20.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(128.0), Px(20.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        let v = app.models().get(model).cloned().unwrap_or_default();
        assert!(
            v.first().copied().unwrap_or(0.0) > 0.33,
            "expected left panel to grow, got {v:?}"
        );
    }

    #[test]
    fn pressable_on_activate_hook_runs_on_pointer_activation() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
        let mut services = FakeTextService::default();

        let activated = app.models_mut().insert(false);

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "pressable-on-activate-hook-pointer",
            |cx| {
                vec![
                    cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                        cx.pressable_on_activate(Arc::new(move |host, _cx, reason| {
                            assert_eq!(reason, ActivateReason::Pointer);
                            let _ = host.models_mut().update(activated, |v| *v = true);
                        }));
                        vec![cx.text("activate")]
                    }),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(app.models().get(activated).copied(), Some(false));

        let pressable_node = ui.children(root)[0];
        let pressable_bounds = ui
            .debug_node_bounds(pressable_node)
            .expect("pressable bounds");
        let position = Point::new(
            Px(pressable_bounds.origin.x.0 + 1.0),
            Px(pressable_bounds.origin.y.0 + 1.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        assert_eq!(app.models().get(activated).copied(), Some(true));
    }

    #[test]
    fn pressable_on_hover_change_hook_runs_on_pointer_move() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
        let mut services = FakeTextService::default();

        let hovered = app.models_mut().insert(false);

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "pressable-on-hover-change-hook",
            |cx| {
                vec![
                    cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                        cx.pressable_on_hover_change(Arc::new(move |host, _cx, is_hovered| {
                            let _ = host.models_mut().update(hovered, |v| *v = is_hovered);
                        }));
                        vec![cx.text("hover me")]
                    }),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(app.models().get(hovered).copied(), Some(false));

        let pressable_node = ui.children(root)[0];
        let pressable_bounds = ui
            .debug_node_bounds(pressable_node)
            .expect("pressable bounds");
        let inside = Point::new(
            Px(pressable_bounds.origin.x.0 + 1.0),
            Px(pressable_bounds.origin.y.0 + 1.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: inside,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        assert_eq!(app.models().get(hovered).copied(), Some(true));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        assert_eq!(app.models().get(hovered).copied(), Some(false));
    }

    #[test]
    fn pressable_on_activate_hook_runs_on_keyboard_activation() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
        let mut services = FakeTextService::default();

        let activated = app.models_mut().insert(false);

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "pressable-on-activate-hook-keyboard",
            |cx| {
                vec![
                    cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                        cx.pressable_on_activate(Arc::new(move |host, _cx, reason| {
                            assert_eq!(reason, ActivateReason::Keyboard);
                            let _ = host.models_mut().update(activated, |v| *v = true);
                        }));
                        vec![cx.text("activate")]
                    }),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(app.models().get(activated).copied(), Some(false));

        let pressable_node = ui.children(root)[0];
        ui.set_focus(Some(pressable_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );

        assert_eq!(app.models().get(activated).copied(), Some(true));
    }

    #[test]
    fn dismissible_on_dismiss_request_hook_runs_on_escape() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
        let mut services = FakeTextService::default();

        let dismissed = app.models_mut().insert(false);

        let base_root = ui.create_node(FillStack);
        ui.set_root(base_root);

        let overlay_root = super::render_dismissible_root_with_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "dismissible-hook-escape",
            |cx| {
                cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, reason| {
                    assert_eq!(reason, DismissReason::Escape);
                    let _ = host.models_mut().update(dismissed, |v| *v = true);
                }));

                vec![
                    cx.pressable(crate::element::PressableProps::default(), |cx, _| {
                        vec![cx.text("child")]
                    }),
                ]
            },
        );

        let layer = ui.push_overlay_root_ex(overlay_root, false, true);
        ui.set_layer_visible(layer, true);

        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Focus a descendant in the overlay so Escape bubbles up to the dismissible layer.
        let focused = ui.children(overlay_root)[0];
        ui.set_focus(Some(focused));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        assert_eq!(app.models().get(dismissed).copied(), Some(true));
    }

    #[test]
    fn dismissible_on_dismiss_request_hook_runs_on_outside_press_observer() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
        let mut services = FakeTextService::default();

        let dismissed = app.models_mut().insert(false);

        // Base root provides a hit-test target so the pointer down is "outside" the overlay.
        let base_root = ui.create_node(FillStack);
        ui.set_root(base_root);

        let overlay_root = super::render_dismissible_root_with_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "dismissible-hook-outside-press",
            |cx| {
                cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, reason| {
                    assert_eq!(reason, DismissReason::OutsidePress);
                    let _ = host.models_mut().update(dismissed, |v| *v = true);
                }));
                Vec::new()
            },
        );

        let layer = ui.push_overlay_root_ex(overlay_root, false, true);
        ui.set_layer_wants_pointer_down_outside_events(layer, true);
        ui.set_layer_visible(layer, true);

        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Pointer down hits the base root (overlay has no children and is hit-test transparent),
        // so outside-press observer dispatch runs for the overlay root.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(2.0), Px(2.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        assert_eq!(app.models().get(dismissed).copied(), Some(true));
    }

    #[test]
    fn roving_flex_arrow_keys_move_focus_and_update_selection() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );
        let mut services = FakeTextService::default();

        let model = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("a")));
        let values: Arc<[Arc<str>]> = Arc::from([Arc::from("a"), Arc::from("b"), Arc::from("c")]);

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "roving-flex",
            |cx| {
                let props = crate::element::RovingFlexProps {
                    flex: crate::element::FlexProps {
                        direction: fret_core::Axis::Vertical,
                        ..Default::default()
                    },
                    roving: crate::element::RovingFocusProps {
                        enabled: true,
                        wrap: true,
                        disabled: Arc::from([false, true, false]),
                    },
                };

                vec![cx.roving_flex(props, |cx| {
                    let values = values.clone();
                    cx.roving_on_active_change(Arc::new(move |host, _cx, idx| {
                        let Some(value) = values.get(idx).cloned() else {
                            return;
                        };
                        let next = Some(value);
                        let _ = host.models_mut().update(model, |v| *v = next);
                    }));

                    let mut make = |label: &'static str| {
                        cx.pressable(
                            crate::element::PressableProps::default(),
                            |child_cx, _st| vec![child_cx.text(label)],
                        )
                    };
                    vec![make("a"), make("b"), make("c")]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let roving = ui.children(root)[0];
        let a = ui.children(roving)[0];
        let c = ui.children(roving)[2];
        ui.set_focus(Some(a));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        assert_eq!(
            ui.focus(),
            Some(c),
            "expected ArrowDown to skip disabled child"
        );
        assert_eq!(
            app.models().get(model).and_then(|v| v.as_deref()),
            Some("c")
        );
    }

    #[test]
    fn roving_flex_typeahead_hook_can_choose_target_index() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );
        let mut services = FakeTextService::default();

        let root =
            render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "roving-flex-typeahead-hook",
                |cx| {
                    let props = crate::element::RovingFlexProps {
                        flex: crate::element::FlexProps {
                            direction: fret_core::Axis::Vertical,
                            ..Default::default()
                        },
                        roving: crate::element::RovingFocusProps {
                            enabled: true,
                            wrap: true,
                            disabled: Arc::from([false, false, false]),
                        },
                    };

                    vec![cx.roving_flex(props, |cx| {
                        cx.roving_on_typeahead(Arc::new(|_host, _cx, it| {
                            if it.input == 'c' { Some(2) } else { None }
                        }));

                        let mut make = |label: &'static str| {
                            cx.pressable(
                                crate::element::PressableProps::default(),
                                |child_cx, _st| vec![child_cx.text(label)],
                            )
                        };
                        vec![make("a"), make("b"), make("c")]
                    })]
                },
            );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let roving = ui.children(root)[0];
        let a = ui.children(roving)[0];
        let c = ui.children(roving)[2];
        ui.set_focus(Some(a));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::KeyC,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        assert_eq!(ui.focus(), Some(c));
    }

    #[test]
    fn pressable_semantics_checked_is_exposed() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(60.0)));
        let mut services = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "a11y-pressable-checked",
            |cx| {
                vec![cx.pressable(
                    crate::element::PressableProps {
                        enabled: true,
                        a11y: crate::element::PressableA11y {
                            role: Some(fret_core::SemanticsRole::Checkbox),
                            label: Some(Arc::from("checked")),
                            checked: Some(true),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |cx, _state| vec![cx.text("x")],
                )]
            },
        );
        ui.set_root(root);

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Checkbox
                    && n.label.as_deref() == Some("checked")
            })
            .expect("expected checkbox semantics node");

        assert_eq!(node.flags.checked, Some(true));
        assert!(node.actions.invoke, "expected checkbox to be invokable");
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
        let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(50.0)),
        );
        let mut text = FakeTextService::default();
        let mut list_element_id: Option<crate::elements::GlobalElementId> = None;

        fn build_list(
            cx: &mut ElementCx<'_, TestHost>,
            list_element_id: &mut Option<crate::elements::GlobalElementId>,
            scroll_handle: &crate::scroll::VirtualListScrollHandle,
        ) -> crate::element::AnyElement {
            let list = cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 0),
                scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                        .collect()
                },
            );
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
            |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let list_node = ui.children(root)[0];
        assert_eq!(ui.children(list_node).len(), 0);
        let viewport_h = crate::elements::with_element_state(
            &mut app,
            window,
            list_element_id.unwrap(),
            crate::element::VirtualListState::default,
            |s| s.viewport_h,
        );
        assert_eq!(viewport_h, Px(50.0));

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
            |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
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
        assert_eq!(
            props
                .visible_items
                .iter()
                .map(|item| item.index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
        assert_eq!(ui.children(list_node).len(), 5);
    }

    #[test]
    fn virtual_list_scroll_to_item_keeps_target_visible() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
        let mut list_element_id: Option<crate::elements::GlobalElementId> = None;

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
                let list = cx.virtual_list(
                    100,
                    crate::element::VirtualListOptions::new(Px(10.0), 0),
                    &scroll_handle,
                    |cx, items| {
                        items
                            .iter()
                            .copied()
                            .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                            .collect()
                    },
                );
                list_element_id = Some(list.id);
                vec![list]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        app.advance_frame();

        // Frame 1: request scroll-to on a row below the viewport.
        let target = 6usize; // row_top=60, viewport=30 => needs offset ~= 40..60
        scroll_handle.scroll_to_item(target, crate::scroll::ScrollStrategy::Nearest);
        assert_eq!(
            scroll_handle.deferred_scroll_to_item(),
            Some((target, crate::scroll::ScrollStrategy::Nearest))
        );
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-scroll-to",
            |cx| {
                let list = cx.virtual_list(
                    100,
                    crate::element::VirtualListOptions::new(Px(10.0), 0),
                    &scroll_handle,
                    |cx, items| {
                        items
                            .iter()
                            .copied()
                            .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                            .collect()
                    },
                );
                list_element_id = Some(list.id);
                vec![list]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        assert!(scroll_handle.deferred_scroll_to_item().is_none());

        let state = crate::elements::with_element_state(
            &mut app,
            window,
            list_element_id.expect("list element id"),
            crate::element::VirtualListState::default,
            |s| s.clone(),
        );
        assert_eq!(state.viewport_h, Px(30.0));
        assert!((state.metrics.offset_for_index(target).0 - 60.0).abs() < 0.01);
        assert!(
            (state.offset_y.0 - 40.0).abs() < 0.01,
            "state_offset_y={:?}",
            state.offset_y
        );

        assert!(
            (scroll_handle.offset().y.0 - 40.0).abs() < 0.01,
            "offset_y={:?}",
            scroll_handle.offset().y
        );
    }

    #[test]
    fn virtual_list_scroll_to_item_uses_measured_row_heights() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
        let mut list_element_id: Option<crate::elements::GlobalElementId> = None;

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(30.0)),
        );
        let mut text = FakeTextService::default();

        fn row_with_height<H: UiHost>(cx: &mut ElementCx<'_, H>, height: Px) -> AnyElement {
            let mut style = crate::element::LayoutStyle::default();
            style.size.height = crate::element::Length::Px(height);
            cx.container(
                crate::element::ContainerProps {
                    layout: style,
                    ..Default::default()
                },
                |_| Vec::new(),
            )
        }

        fn build_list<H: UiHost>(
            cx: &mut ElementCx<'_, H>,
            list_element_id: &mut Option<crate::elements::GlobalElementId>,
            scroll_handle: &crate::scroll::VirtualListScrollHandle,
        ) -> AnyElement {
            let list = cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 0),
                scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| {
                            cx.keyed(item.key, |cx| {
                                if item.index == 0 {
                                    row_with_height(cx, Px(100.0))
                                } else {
                                    row_with_height(cx, Px(10.0))
                                }
                            })
                        })
                        .collect()
                },
            );
            *list_element_id = Some(list.id);
            list
        }

        // Frame 0: establish viewport height.
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-measure",
            |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        app.advance_frame();

        // Frame 1: ensure row 0 gets mounted and measured.
        let prev_list_element_id = list_element_id;
        list_element_id = None;
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-measure",
            |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        assert_eq!(
            prev_list_element_id, list_element_id,
            "virtual list element id should be stable across frames"
        );
        app.advance_frame();

        // Frame 2: scroll to item 1; should account for the measured height of item 0.
        scroll_handle.scroll_to_item(1, crate::scroll::ScrollStrategy::Start);
        assert_eq!(
            scroll_handle.deferred_scroll_to_item(),
            Some((1, crate::scroll::ScrollStrategy::Start))
        );
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-measure",
            |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        assert!(
            (scroll_handle.offset().y.0 - 100.0).abs() < 0.01,
            "offset_y={:?}",
            scroll_handle.offset().y
        );
    }

    #[test]
    fn virtual_list_paint_clips_each_visible_row() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(50.0)),
        );
        let mut text = FakeTextService::default();

        fn build_list<H: UiHost>(
            cx: &mut ElementCx<'_, H>,
            scroll_handle: &crate::scroll::VirtualListScrollHandle,
        ) -> AnyElement {
            cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 0),
                scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                        .collect()
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
            "mvp50-vlist-clip",
            |cx| vec![build_list(cx, &scroll_handle)],
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
            |cx| vec![build_list(cx, &scroll_handle)],
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
        let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

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
            scroll_handle: &crate::scroll::VirtualListScrollHandle,
        ) -> AnyElement {
            let items_revision = items
                .iter()
                .fold(0u64, |acc, k| acc.wrapping_mul(1_000_003).wrapping_add(*k));
            let mut options = crate::element::VirtualListOptions::new(Px(10.0), 0);
            options.items_revision = items_revision;

            cx.virtual_list_keyed(
                items.len(),
                options,
                scroll_handle,
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
            |cx| vec![build_list(cx, &items, None, &scroll_handle)],
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
                |cx| vec![build_list(cx, &items, Some(&mut ids), &scroll_handle)],
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
    fn hover_region_reports_hovered_even_when_child_is_pressable() {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(80.0)),
        );
        let mut text = FakeTextService::default();

        fn build_root(cx: &mut ElementCx<'_, TestHost>) -> Vec<AnyElement> {
            vec![cx.hover_region(
                crate::element::HoverRegionProps::default(),
                |cx, hovered| {
                    let trigger = cx
                        .pressable(crate::element::PressableProps::default(), |cx, _state| {
                            vec![cx.text("trigger")]
                        });

                    let mut children = vec![trigger];
                    if hovered {
                        children.push(cx.text("hovered"));
                    }
                    children
                },
            )]
        }

        // Frame 0: not hovered yet, so only the trigger is present.
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "hover-region",
            build_root,
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let hover_region_node = ui.children(root)[0];
        assert_eq!(ui.children(hover_region_node).len(), 1);
        let trigger_node = ui.children(hover_region_node)[0];
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");

        let pos = fret_core::Point::new(
            Px(trigger_bounds.origin.x.0 + 2.0),
            Px(trigger_bounds.origin.y.0 + 2.0),
        );
        ui.dispatch_event(
            &mut app,
            &mut text,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: pos,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        // Frame 1: hover_region should now observe hovered=true even though the hit node is a Pressable.
        app.advance_frame();
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "hover-region",
            build_root,
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let hover_region_node = ui.children(root)[0];
        assert_eq!(ui.children(hover_region_node).len(), 2);
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
    fn overflow_clip_with_corner_radii_pushes_rounded_clip_rect() {
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
            "mvp-overflow-clip-rounded",
            |cx| {
                let mut props = crate::element::ContainerProps::default();
                props.layout.overflow = crate::element::Overflow::Clip;
                props.corner_radii = fret_core::Corners::all(Px(8.0));
                vec![cx.container(props, |cx| vec![cx.text("child")])]
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
                .any(|op| matches!(op, SceneOp::PushClipRRect { .. })),
            "expected container overflow clip + corner radii to push a rounded clip rect"
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
            !scene.ops().iter().any(|op| matches!(
                op,
                SceneOp::PushClipRect { .. } | SceneOp::PushClipRRect { .. }
            )),
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
                        layout: {
                            let mut l = crate::element::LayoutStyle::default();
                            l.size.width = crate::element::Length::Fill;
                            l
                        },
                        ..Default::default()
                    },
                    |cx| {
                        let mut a = crate::element::ContainerProps::default();
                        a.layout.size.width = crate::element::Length::Px(Px(10.0));
                        a.layout.size.height = crate::element::Length::Px(Px(10.0));

                        let mut b = crate::element::ContainerProps::default();
                        b.layout.size.width = crate::element::Length::Px(Px(10.0));
                        b.layout.size.height = crate::element::Length::Px(Px(10.0));
                        b.layout.margin.left = crate::element::MarginEdge::Px(Px(5.0));

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
    fn flex_child_auto_margins_center_child() {
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
            "mvp62-flex-mx-auto",
            |cx| {
                vec![cx.flex(
                    crate::element::FlexProps {
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(0.0),
                        layout: {
                            let mut l = crate::element::LayoutStyle::default();
                            l.size.width = crate::element::Length::Fill;
                            l
                        },
                        ..Default::default()
                    },
                    |cx| {
                        let mut a = crate::element::ContainerProps::default();
                        a.layout.size.width = crate::element::Length::Px(Px(10.0));
                        a.layout.size.height = crate::element::Length::Px(Px(10.0));
                        a.layout.margin.left = crate::element::MarginEdge::Auto;
                        a.layout.margin.right = crate::element::MarginEdge::Auto;
                        vec![cx.container(a, |_cx| vec![])]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let flex_node = ui.children(root)[0];
        let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");
        assert_eq!(flex_bounds.size.width, Px(100.0));
        let children = ui.children(flex_node);
        assert_eq!(children.len(), 1);
        let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");

        assert_eq!(a_bounds.origin.x, Px(45.0));
    }

    #[test]
    fn flex_child_negative_margin_shifts_layout() {
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
            "mvp62-flex-negative-margin",
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
                        b.layout.margin.left = crate::element::MarginEdge::Px(Px(-5.0));

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
        assert_eq!(b_bounds.origin.x, Px(5.0));
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
    fn container_absolute_negative_inset_offsets_outside_parent() {
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
            "mvp62-stack-absolute-negative-inset",
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
                        badge.layout.inset.top = Some(Px(-5.0));
                        badge.layout.inset.left = Some(Px(-6.0));

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
        let children = ui.children(container_node);
        assert_eq!(children.len(), 2);
        let badge_bounds = ui.debug_node_bounds(children[1]).expect("badge bounds");
        assert_eq!(badge_bounds.origin.x, Px(-6.0));
        assert_eq!(badge_bounds.origin.y, Px(-5.0));
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
