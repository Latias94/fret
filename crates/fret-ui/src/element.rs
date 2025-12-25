use crate::UiHost;
use crate::elements::{ElementCx, GlobalElementId};
use fret_core::{Color, Corners, Edges, Px, TextStyle, TextWrap};
use fret_runtime::CommandId;

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
    VirtualList(VirtualListProps),
}

/// A low-opinionated container primitive for declarative authoring.
///
/// This is intentionally small and composable: it provides padding and an optional quad background
/// (including border and corner radii) so component-layer recipes can build shadcn-like widgets
/// via composition.
#[derive(Debug, Clone, Copy)]
pub struct ContainerProps {
    pub padding_x: Px,
    pub padding_y: Px,
    pub background: Option<Color>,
    pub border: Edges,
    pub border_color: Option<Color>,
    pub corner_radii: Corners,
}

impl Default for ContainerProps {
    fn default() -> Self {
        Self {
            padding_x: Px(0.0),
            padding_y: Px(0.0),
            background: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Corners::all(Px(0.0)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PressableProps {
    pub enabled: bool,
    pub on_click: Option<CommandId>,
}

impl Default for PressableProps {
    fn default() -> Self {
        Self {
            enabled: true,
            on_click: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PressableState {
    pub hovered: bool,
    pub pressed: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct StackProps;

impl Default for StackProps {
    fn default() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnProps {
    pub gap: Px,
    pub padding_x: Px,
    pub padding_y: Px,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for ColumnProps {
    fn default() -> Self {
        Self {
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
    pub gap: Px,
    pub padding_x: Px,
    pub padding_y: Px,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for RowProps {
    fn default() -> Self {
        Self {
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
    pub min: Px,
}

impl Default for SpacerProps {
    fn default() -> Self {
        Self { min: Px(0.0) }
    }
}

#[derive(Debug, Clone)]
pub struct TextProps {
    pub text: std::sync::Arc<str>,
    pub style: Option<TextStyle>,
    pub color: Option<Color>,
    pub wrap: TextWrap,
}

impl TextProps {
    pub fn new(text: impl Into<std::sync::Arc<str>>) -> Self {
        Self {
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VirtualListProps {
    pub len: usize,
    pub row_height: Px,
    pub overscan: usize,
    pub visible_start: usize,
    pub visible_end: usize,
}

/// Cross-frame element-local state for a virtual list (stored in the element state store).
#[derive(Debug, Default, Clone, Copy)]
pub struct VirtualListState {
    pub offset_y: Px,
    pub viewport_h: Px,
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
