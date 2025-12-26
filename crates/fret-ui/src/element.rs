use crate::UiHost;
use crate::elements::{ElementCx, GlobalElementId};
use fret_core::{Color, Corners, Edges, ImageId, Px, TextOverflow, TextStyle, TextWrap, UvRect};
use fret_runtime::{CommandId, Model};

use crate::primitives::TextInputStyle;

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
    Pressable(PressableProps),
    Stack(StackProps),
    Column(ColumnProps),
    Row(RowProps),
    Spacer(SpacerProps),
    Text(TextProps),
    TextInput(TextInputProps),
    VirtualList(VirtualListProps),
    Flex(FlexProps),
    Grid(GridProps),
    Image(ImageProps),
    Spinner(SpinnerProps),
    HoverCard(HoverCardProps),
    Scroll(ScrollProps),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutStyle {
    pub size: SizeStyle,
    pub flex: FlexItemStyle,
    pub overflow: Overflow,
    pub margin: Edges,
    pub position: PositionStyle,
    pub inset: InsetStyle,
    pub aspect_ratio: Option<f32>,
    pub grid: GridItemStyle,
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

#[derive(Debug, Clone, Copy, Default)]
pub struct FlexItemStyle {
    pub grow: f32,
    pub shrink: f32,
    pub basis: Length,
    pub align_self: Option<CrossAlign>,
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
    pub padding_x: Px,
    pub padding_y: Px,
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
            padding_x: Px(0.0),
            padding_y: Px(0.0),
            background: None,
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Corners::all(Px(0.0)),
        }
    }
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

#[derive(Debug, Clone)]
pub struct PressableProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub on_click: Option<CommandId>,
    pub focus_ring: Option<RingStyle>,
}

impl Default for PressableProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            enabled: true,
            on_click: None,
            focus_ring: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PressableState {
    pub hovered: bool,
    pub pressed: bool,
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

#[derive(Debug, Clone, Copy)]
pub struct StackProps {
    pub layout: LayoutStyle,
}

impl Default for StackProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnProps {
    pub layout: LayoutStyle,
    pub gap: Px,
    pub padding_x: Px,
    pub padding_y: Px,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for ColumnProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            gap: Px(0.0),
            padding_x: Px(0.0),
            padding_y: Px(0.0),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RowProps {
    pub layout: LayoutStyle,
    pub gap: Px,
    pub padding_x: Px,
    pub padding_y: Px,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for RowProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            gap: Px(0.0),
            padding_x: Px(0.0),
            padding_y: Px(0.0),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
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
            .field("chrome", &self.chrome)
            .field("text_style", &self.text_style)
            .field("submit_command", &self.submit_command)
            .field("cancel_command", &self.cancel_command)
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HoverCardAlign {
    Start,
    #[default]
    Center,
    End,
}

/// A hover-driven floating surface primitive (shadcn-style hover card).
///
/// This element is intended to have exactly two children:
/// - child 0: trigger
/// - child 1: content
///
/// Layout: the trigger participates in normal flow; the content is laid out and painted as an
/// absolute-positioned floating surface anchored to the trigger.
#[derive(Debug, Clone, Copy)]
pub struct HoverCardProps {
    pub layout: LayoutStyle,
    pub align: HoverCardAlign,
    pub side_offset: Px,
    /// Open delay expressed in frames (best-effort; driven by animation frames).
    pub open_delay_frames: u32,
    /// Close delay expressed in frames (best-effort; driven by animation frames).
    pub close_delay_frames: u32,
}

impl Default for HoverCardProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            align: HoverCardAlign::Center,
            side_offset: Px(4.0),
            open_delay_frames: 0,
            close_delay_frames: 0,
        }
    }
}

impl TextProps {
    pub fn new(text: impl Into<std::sync::Arc<str>>) -> Self {
        Self {
            layout: LayoutStyle::default(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlexProps {
    pub layout: LayoutStyle,
    pub direction: fret_core::Axis,
    pub gap: Px,
    pub padding_x: Px,
    pub padding_y: Px,
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
            padding_x: Px(0.0),
            padding_y: Px(0.0),
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
    pub padding_x: Px,
    pub padding_y: Px,
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
            padding_x: Px(0.0),
            padding_y: Px(0.0),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VirtualListProps {
    pub layout: LayoutStyle,
    pub len: usize,
    pub row_height: Px,
    pub overscan: usize,
    /// If set, adjust the list scroll offset to keep the given row index visible.
    ///
    /// This is a low-level virtualization primitive (not a selection model): component-layer code
    /// can compute the desired row index (e.g. from a selection model) and request that the list
    /// stays scrolled to it.
    pub scroll_to_index: Option<usize>,
    pub visible_start: usize,
    pub visible_end: usize,
}

/// Cross-frame element-local state for a virtual list (stored in the element state store).
#[derive(Debug, Default, Clone, Copy)]
pub struct VirtualListState {
    pub offset_y: Px,
    pub viewport_h: Px,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollProps {
    pub layout: LayoutStyle,
    pub show_scrollbar: bool,
}

impl Default for ScrollProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.overflow = Overflow::Clip;
        Self {
            layout,
            show_scrollbar: true,
        }
    }
}

/// Cross-frame element-local state for scroll containers.
#[derive(Debug, Default, Clone, Copy)]
pub struct ScrollState {
    pub offset_y: Px,
    pub viewport_h: Px,
    pub content_h: Px,
    pub dragging_thumb: bool,
    pub drag_start_pointer_y: Px,
    pub drag_start_offset_y: Px,
    pub hovered_scrollbar: bool,
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
    fn render<H: UiHost>(&mut self, cx: &mut ElementCx<'_, H>) -> AnyElement;
}

/// Stateless component authoring layer (ADR 0039).
pub trait RenderOnce {
    fn render_once<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement;
}
