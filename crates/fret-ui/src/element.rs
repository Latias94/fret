use crate::UiHost;
use crate::elements::{ElementContext, GlobalElementId};
use fret_core::{
    Color, Corners, Edges, ImageId, NodeId, Px, SemanticsRole, SvgFit, TextOverflow, TextStyle,
    TextWrap, UvRect,
};
use fret_runtime::{CommandId, Model};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{ResizablePanelGroupStyle, SvgSource, TextAreaStyle, TextInputStyle};

/// Declarative element tree node (ephemeral per frame), keyed by a stable `GlobalElementId`.
///
/// This is the authoring-layer representation described by ADR 0028 / ADR 0039.
#[derive(Debug, Clone)]
pub struct AnyElement {
    pub id: GlobalElementId,
    pub kind: ElementKind,
    pub children: Vec<AnyElement>,
}

impl AnyElement {
    pub fn new(id: GlobalElementId, kind: ElementKind, children: Vec<AnyElement>) -> Self {
        Self { id, kind, children }
    }
}

#[derive(Debug, Clone)]
pub enum ElementKind {
    Container(ContainerProps),
    Semantics(SemanticsProps),
    FocusScope(FocusScopeProps),
    Opacity(OpacityProps),
    VisualTransform(VisualTransformProps),
    Pressable(PressableProps),
    PointerRegion(PointerRegionProps),
    RovingFlex(RovingFlexProps),
    Stack(StackProps),
    Column(ColumnProps),
    Row(RowProps),
    Spacer(SpacerProps),
    Text(TextProps),
    TextInput(TextInputProps),
    TextArea(TextAreaProps),
    ResizablePanelGroup(ResizablePanelGroupProps),
    VirtualList(VirtualListProps),
    Flex(FlexProps),
    Grid(GridProps),
    Image(ImageProps),
    SvgIcon(SvgIconProps),
    Spinner(SpinnerProps),
    HoverRegion(HoverRegionProps),
    Scroll(ScrollProps),
    Scrollbar(ScrollbarProps),
}

/// Per-element pointer state for `PointerRegion`.
#[derive(Debug, Default, Clone)]
pub struct PointerRegionState {
    pub last_down: Option<crate::action::PointerDownCx>,
}

/// A pointer event listener region primitive.
///
/// This is a mechanism-only building block: it does not imply click/activation semantics.
#[derive(Debug, Clone, Copy)]
pub struct PointerRegionProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
}

impl Default for PointerRegionProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutStyle {
    pub size: SizeStyle,
    pub flex: FlexItemStyle,
    pub overflow: Overflow,
    pub margin: MarginEdges,
    pub position: PositionStyle,
    pub inset: InsetStyle,
    pub aspect_ratio: Option<f32>,
    pub grid: GridItemStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginEdge {
    Px(Px),
    Auto,
}

impl Default for MarginEdge {
    fn default() -> Self {
        Self::Px(Px(0.0))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MarginEdges {
    pub top: MarginEdge,
    pub right: MarginEdge,
    pub bottom: MarginEdge,
    pub left: MarginEdge,
}

impl MarginEdges {
    pub fn all(edge: MarginEdge) -> Self {
        Self {
            top: edge,
            right: edge,
            bottom: edge,
            left: edge,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Overflow {
    #[default]
    Visible,
    Clip,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PositionStyle {
    /// Default flow position; inset offsets are ignored.
    #[default]
    Static,
    /// Inset offsets tweak the final position without affecting siblings.
    Relative,
    /// Removed from flow and positioned via inset offsets.
    Absolute,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct InsetStyle {
    pub top: Option<Px>,
    pub right: Option<Px>,
    pub bottom: Option<Px>,
    pub left: Option<Px>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GridItemStyle {
    pub column: GridLine,
    pub row: GridLine,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GridLine {
    pub start: Option<i16>,
    pub span: Option<u16>,
}

#[derive(Debug, Clone, Copy)]
pub struct SizeStyle {
    pub width: Length,
    pub height: Length,
    pub min_width: Option<Px>,
    pub min_height: Option<Px>,
    pub max_width: Option<Px>,
    pub max_height: Option<Px>,
}

impl Default for SizeStyle {
    fn default() -> Self {
        Self {
            width: Length::Auto,
            height: Length::Auto,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlexItemStyle {
    pub grow: f32,
    pub shrink: f32,
    pub basis: Length,
    pub align_self: Option<CrossAlign>,
}

impl Default for FlexItemStyle {
    fn default() -> Self {
        Self {
            grow: 0.0,
            // Tailwind/DOM default is `flex-shrink: 1`. Recipes should opt out via
            // `LayoutRefinement::flex_shrink_0()` when needed.
            shrink: 1.0,
            basis: Length::Auto,
            align_self: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Length {
    #[default]
    Auto,
    Px(Px),
    Fill,
}

/// A low-opinionated container primitive for declarative authoring.
///
/// This is intentionally small and composable: it provides padding and an optional quad background
/// (including border and corner radii) so component-layer recipes can build shadcn-like widgets
/// via composition.
#[derive(Debug, Clone, Copy)]
pub struct ContainerProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub background: Option<Color>,
    pub shadow: Option<ShadowStyle>,
    pub border: Edges,
    pub border_color: Option<Color>,
    pub corner_radii: Corners,
}

impl Default for ContainerProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            padding: Edges::all(Px(0.0)),
            background: None,
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Corners::all(Px(0.0)),
        }
    }
}

/// A transparent semantics wrapper for structuring the accessibility tree.
///
/// This is intentionally input-transparent (hit-test passes through) and paint-transparent: it
/// only contributes layout and semantics.
#[derive(Debug, Clone)]
pub struct SemanticsProps {
    pub layout: LayoutStyle,
    pub role: SemanticsRole,
    pub label: Option<Arc<str>>,
    pub value: Option<Arc<str>>,
    pub disabled: bool,
    pub selected: bool,
    pub expanded: Option<bool>,
    pub checked: Option<bool>,
    pub active_descendant: Option<NodeId>,
}

impl Default for SemanticsProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            role: SemanticsRole::Generic,
            label: None,
            value: None,
            disabled: false,
            selected: false,
            expanded: None,
            checked: None,
            active_descendant: None,
        }
    }
}

/// A transparent focus-scope wrapper that can trap focus traversal within its subtree.
///
/// This is a small, mechanism-oriented primitive intended to support component-owned focus scopes
/// (ADR 0068). It does not imply modal barriers or pointer blocking; it only affects `focus.next`
/// / `focus.previous` command routing when focus is inside the subtree.
#[derive(Debug, Default, Clone, Copy)]
pub struct FocusScopeProps {
    pub layout: LayoutStyle,
    pub trap_focus: bool,
}

/// A paint-only opacity group wrapper (ADR 0019).
///
/// This is intentionally layout-only + paint-only: it does not imply semantics beyond its
/// children, and it is input-transparent (hit-test passes through).
#[derive(Debug, Clone, Copy)]
pub struct OpacityProps {
    pub layout: LayoutStyle,
    pub opacity: f32,
}

impl Default for OpacityProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            opacity: 1.0,
        }
    }
}

/// Paint-only transform wrapper for declarative element subtrees.
///
/// This applies a `SceneOp::PushTransform` / `PopTransform` around the subtree during painting,
/// without affecting layout, hit-testing, or pointer event coordinates.
///
/// This is intentionally similar to GPUI's `with_transformation(...)` semantics for elements like
/// `Svg`: it is useful for spinners and decorative animations, and is cheap to optimize because it
/// does not require inverse mapping during hit-testing.
#[derive(Debug, Clone, Copy, Default)]
pub struct VisualTransformProps {
    pub layout: LayoutStyle,
    /// A transform expressed in the element's local coordinate space.
    ///
    /// The runtime composes this around the element's bounds origin so that local transforms can be
    /// expressed in px relative to the element (e.g. rotate around `Point(Px(w/2), Px(h/2))`).
    pub transform: fret_core::Transform2D,
}

/// A low-level drop shadow primitive for component-level elevation recipes.
///
/// This intentionally does not require a dedicated blur pipeline: the runtime can approximate
/// softness by drawing multiple expanded quads with alpha falloff (see ADR 0060).
#[derive(Debug, Clone, Copy)]
pub struct ShadowStyle {
    pub color: Color,
    pub offset_x: Px,
    pub offset_y: Px,
    pub spread: Px,
    /// Additional "soft" layers to draw around the shadow.
    ///
    /// `0` draws a single hard-edge quad. Higher values approximate blur via multiple layers.
    pub softness: u8,
    pub corner_radii: Corners,
}

#[derive(Clone)]
pub struct PressableProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
    /// Whether this pressable is a focus traversal stop (Tab order).
    ///
    /// When `false`, the node can still be focused programmatically (e.g. roving focus),
    /// but it is skipped by the default focus traversal.
    pub focusable: bool,
    pub focus_ring: Option<RingStyle>,
    pub a11y: PressableA11y,
}

impl std::fmt::Debug for PressableProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = f.debug_struct("PressableProps");
        out.field("layout", &self.layout)
            .field("enabled", &self.enabled)
            .field("focusable", &self.focusable);

        out.field("focus_ring", &self.focus_ring)
            .field("a11y", &self.a11y)
            .finish()
    }
}

impl Default for PressableProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            enabled: true,
            focusable: true,
            focus_ring: None,
            a11y: PressableA11y::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct RovingFlexProps {
    pub flex: FlexProps,
    pub roving: RovingFocusProps,
}

impl std::fmt::Debug for RovingFlexProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RovingFlexProps")
            .field("flex", &self.flex)
            .field("roving", &self.roving)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct RovingFocusProps {
    pub enabled: bool,
    pub wrap: bool,
    pub disabled: Arc<[bool]>,
}

impl Default for RovingFocusProps {
    fn default() -> Self {
        Self {
            enabled: true,
            wrap: true,
            disabled: Arc::from([]),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PressableA11y {
    pub role: Option<SemanticsRole>,
    pub label: Option<Arc<str>>,
    pub selected: bool,
    pub expanded: Option<bool>,
    pub checked: Option<bool>,
    pub pos_in_set: Option<u32>,
    pub set_size: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PressableState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RingPlacement {
    /// Draw the ring inside the element bounds.
    Inset,
    /// Draw the ring outside the element bounds (best effort; may be clipped by parent clips).
    #[default]
    Outset,
}

/// A simple focus ring decoration, intended for component-layer recipes (e.g. shadcn-style
/// focus-visible ring).
///
/// This is intentionally small and renderer-friendly: it maps to one or two `SceneOp::Quad`
/// operations.
#[derive(Debug, Clone, Copy)]
pub struct RingStyle {
    pub placement: RingPlacement,
    pub width: Px,
    pub offset: Px,
    pub color: Color,
    pub offset_color: Option<Color>,
    pub corner_radii: Corners,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StackProps {
    pub layout: LayoutStyle,
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnProps {
    pub layout: LayoutStyle,
    pub gap: Px,
    pub padding: Edges,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for ColumnProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RowProps {
    pub layout: LayoutStyle,
    pub gap: Px,
    pub padding: Edges,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for RowProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MainAlign {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CrossAlign {
    Start,
    #[default]
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy)]
pub struct SpacerProps {
    pub layout: LayoutStyle,
    pub min: Px,
}

impl Default for SpacerProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.flex.grow = 1.0;
        layout.flex.shrink = 1.0;
        layout.flex.basis = Length::Px(Px(0.0));
        Self {
            layout,
            min: Px(0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextProps {
    pub layout: LayoutStyle,
    pub text: std::sync::Arc<str>,
    pub style: Option<TextStyle>,
    pub color: Option<Color>,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
}

#[derive(Clone)]
pub struct TextInputProps {
    pub layout: LayoutStyle,
    pub model: Model<String>,
    pub a11y_label: Option<std::sync::Arc<str>>,
    pub placeholder: Option<std::sync::Arc<str>>,
    pub active_descendant: Option<NodeId>,
    pub chrome: TextInputStyle,
    pub text_style: TextStyle,
    pub submit_command: Option<CommandId>,
    pub cancel_command: Option<CommandId>,
}

impl TextInputProps {
    pub fn new(model: Model<String>) -> Self {
        Self {
            layout: LayoutStyle::default(),
            model,
            a11y_label: None,
            placeholder: None,
            active_descendant: None,
            chrome: TextInputStyle::default(),
            text_style: TextStyle::default(),
            submit_command: None,
            cancel_command: None,
        }
    }
}

impl std::fmt::Debug for TextInputProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInputProps")
            .field("layout", &self.layout)
            .field("model", &"<model>")
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field(
                "placeholder",
                &self.placeholder.as_ref().map(|s| s.as_ref()),
            )
            .field("chrome", &self.chrome)
            .field("text_style", &self.text_style)
            .field("submit_command", &self.submit_command)
            .field("cancel_command", &self.cancel_command)
            .finish()
    }
}

#[derive(Clone)]
pub struct TextAreaProps {
    pub layout: LayoutStyle,
    pub model: Model<String>,
    pub a11y_label: Option<std::sync::Arc<str>>,
    pub chrome: TextAreaStyle,
    pub text_style: TextStyle,
    pub min_height: Px,
}

impl TextAreaProps {
    pub fn new(model: Model<String>) -> Self {
        Self {
            layout: LayoutStyle::default(),
            model,
            a11y_label: None,
            chrome: TextAreaStyle::default(),
            text_style: TextStyle::default(),
            min_height: Px(80.0),
        }
    }
}

impl std::fmt::Debug for TextAreaProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextAreaProps")
            .field("layout", &self.layout)
            .field("model", &"<model>")
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("chrome", &self.chrome)
            .field("text_style", &self.text_style)
            .field("min_height", &self.min_height)
            .finish()
    }
}

#[derive(Clone)]
pub struct ResizablePanelGroupProps {
    pub layout: LayoutStyle,
    pub axis: fret_core::Axis,
    pub model: Model<Vec<f32>>,
    pub min_px: Vec<Px>,
    pub enabled: bool,
    pub chrome: ResizablePanelGroupStyle,
}

impl ResizablePanelGroupProps {
    pub fn new(axis: fret_core::Axis, model: Model<Vec<f32>>) -> Self {
        Self {
            layout: LayoutStyle::default(),
            axis,
            model,
            min_px: Vec::new(),
            enabled: true,
            chrome: ResizablePanelGroupStyle::default(),
        }
    }
}

impl std::fmt::Debug for ResizablePanelGroupProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResizablePanelGroupProps")
            .field("layout", &self.layout)
            .field("axis", &self.axis)
            .field("model", &"<model>")
            .field("min_px_len", &self.min_px.len())
            .field("enabled", &self.enabled)
            .field("chrome", &self.chrome)
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageProps {
    pub layout: LayoutStyle,
    pub image: ImageId,
    pub opacity: f32,
    pub uv: Option<UvRect>,
}

impl ImageProps {
    pub fn new(image: ImageId) -> Self {
        Self {
            layout: LayoutStyle::default(),
            image,
            opacity: 1.0,
            uv: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SvgIconProps {
    pub layout: LayoutStyle,
    pub svg: SvgSource,
    pub fit: SvgFit,
    pub color: Color,
    pub opacity: f32,
}

impl SvgIconProps {
    pub fn new(svg: SvgSource) -> Self {
        Self {
            layout: LayoutStyle::default(),
            svg,
            fit: SvgFit::Contain,
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            opacity: 1.0,
        }
    }
}

/// A simple loading spinner primitive.
///
/// This is intentionally low-opinionated and renderer-friendly: it paints a ring of small rounded
/// quads with frame-driven alpha modulation (`Effect::RequestAnimationFrame`).
#[derive(Debug, Clone, Copy)]
pub struct SpinnerProps {
    pub layout: LayoutStyle,
    pub color: Option<Color>,
    pub dot_count: u8,
    /// Phase increment per frame, in dot steps. (`0.0` disables animation.)
    pub speed: f32,
}

impl Default for SpinnerProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Px(Px(16.0));
        layout.size.height = Length::Px(Px(16.0));

        Self {
            layout,
            color: None,
            dot_count: 12,
            speed: 0.2,
        }
    }
}

/// A hover tracking region primitive.
///
/// This is a small substrate building block: it provides a `hovered: bool` signal to component
/// code (via `ElementCx::hover_region(...)`) without imposing click/focus semantics.
#[derive(Debug, Clone, Copy, Default)]
pub struct HoverRegionProps {
    pub layout: LayoutStyle,
}

impl TextProps {
    pub fn new(text: impl Into<std::sync::Arc<str>>) -> Self {
        Self {
            layout: LayoutStyle::default(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlexProps {
    pub layout: LayoutStyle,
    pub direction: fret_core::Axis,
    pub gap: Px,
    pub padding: Edges,
    pub justify: MainAlign,
    pub align: CrossAlign,
    pub wrap: bool,
}

impl Default for FlexProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridProps {
    pub layout: LayoutStyle,
    pub cols: u16,
    pub rows: Option<u16>,
    pub gap: Px,
    pub padding: Edges,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for GridProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            cols: 1,
            rows: None,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VirtualListProps {
    pub layout: LayoutStyle,
    pub len: usize,
    pub items_revision: u64,
    pub estimate_row_height: Px,
    pub overscan: usize,
    pub scroll_margin: Px,
    pub gap: Px,
    pub scroll_handle: crate::scroll::VirtualListScrollHandle,
    pub visible_items: Vec<crate::virtual_list::VirtualItem>,
}

#[derive(Debug, Clone, Copy)]
pub struct VirtualListOptions {
    pub items_revision: u64,
    pub estimate_row_height: Px,
    pub overscan: usize,
    pub scroll_margin: Px,
    pub gap: Px,
}

impl VirtualListOptions {
    pub fn new(estimate_row_height: Px, overscan: usize) -> Self {
        Self {
            items_revision: 0,
            estimate_row_height,
            overscan,
            scroll_margin: Px(0.0),
            gap: Px(0.0),
        }
    }
}

/// Cross-frame element-local state for a virtual list (stored in the element state store).
#[derive(Debug, Default, Clone)]
pub struct VirtualListState {
    pub offset_y: Px,
    pub viewport_h: Px,
    pub(crate) metrics: crate::virtual_list::VirtualListMetrics,
    pub(crate) items_revision: u64,
    pub(crate) keys: Vec<crate::ItemKey>,
    pub(crate) size_cache: HashMap<crate::ItemKey, Px>,
}

#[derive(Debug, Clone)]
pub struct ScrollProps {
    pub layout: LayoutStyle,
    pub axis: ScrollAxis,
    pub scroll_handle: Option<crate::scroll::ScrollHandle>,
}

impl Default for ScrollProps {
    fn default() -> Self {
        let layout = LayoutStyle {
            overflow: Overflow::Clip,
            ..Default::default()
        };
        Self {
            layout,
            axis: ScrollAxis::Y,
            scroll_handle: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAxis {
    X,
    Y,
    Both,
}

impl ScrollAxis {
    pub fn scroll_x(self) -> bool {
        matches!(self, Self::X | Self::Both)
    }

    pub fn scroll_y(self) -> bool {
        matches!(self, Self::Y | Self::Both)
    }
}

/// Cross-frame element-local state for scroll containers.
#[derive(Debug, Default, Clone)]
pub struct ScrollState {
    pub scroll_handle: crate::scroll::ScrollHandle,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollbarStyle {
    pub thumb: Color,
    pub thumb_hover: Color,
    pub thumb_idle_alpha: f32,
}

impl Default for ScrollbarStyle {
    fn default() -> Self {
        Self {
            thumb: Color {
                r: 0.35,
                g: 0.38,
                b: 0.45,
                a: 1.0,
            },
            thumb_hover: Color {
                r: 0.45,
                g: 0.50,
                b: 0.60,
                a: 1.0,
            },
            thumb_idle_alpha: 0.65,
        }
    }
}

/// A mechanism-only scrollbar primitive.
///
/// Component libraries decide when to show/hide scrollbars and resolve theme tokens into this
/// style. The runtime owns hit-testing, thumb/track interactions, and paints using the resolved
/// style.
#[derive(Debug, Clone, Default)]
pub struct ScrollbarProps {
    pub layout: LayoutStyle,
    /// Declarative element id for the associated scroll container, if any.
    ///
    /// When provided, the scrollbar will invalidate the target node's layout/paint when the
    /// scroll handle offset changes (e.g. thumb drag or track paging).
    pub scroll_target: Option<GlobalElementId>,
    pub scroll_handle: crate::scroll::ScrollHandle,
    pub style: ScrollbarStyle,
}

/// Cross-frame element-local state for scrollbars.
#[derive(Debug, Default, Clone)]
pub struct ScrollbarState {
    pub dragging_thumb: bool,
    pub drag_start_pointer_y: Px,
    pub drag_start_offset_y: Px,
    pub hovered: bool,
}

/// Authoring conversion boundary (ADR 0039).
pub trait IntoElement {
    fn into_element(self, id: GlobalElementId) -> AnyElement;
}

impl IntoElement for AnyElement {
    fn into_element(self, _id: GlobalElementId) -> AnyElement {
        self
    }
}

impl IntoElement for TextProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::Text(self), Vec::new())
    }
}

impl IntoElement for ImageProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::Image(self), Vec::new())
    }
}

impl IntoElement for SvgIconProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::SvgIcon(self), Vec::new())
    }
}

impl IntoElement for ScrollProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::Scroll(self), Vec::new())
    }
}

impl IntoElement for std::sync::Arc<str> {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        TextProps::new(self).into_element(id)
    }
}

impl IntoElement for &'static str {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        TextProps::new(self).into_element(id)
    }
}

/// Stateful view authoring layer (ADR 0039).
pub trait Render {
    fn render<H: UiHost>(&mut self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}

/// Stateless component authoring layer (ADR 0039).
pub trait RenderOnce {
    fn render_once<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}
