use std::marker::PhantomData;
use std::sync::Arc;

use fret_core::{Axis, Edges, FontWeight, Px, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, FlexProps, Length, StackProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::style as decl_style;
use crate::declarative::text as decl_text;
use crate::{
    ChromeRefinement, Items, Justify, LayoutRefinement, MetricRef, Space, UiBuilder, UiIntoElement,
    UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout,
};

/// A patchable flex layout constructor for authoring ergonomics.
///
/// This is an ecosystem-only helper intended to reduce runtime-props boilerplate in layout-only
/// code while keeping layering rules intact (no policy in `crates/fret-ui`).
#[derive(Debug, Clone)]
pub struct FlexBox<H, F> {
    pub(crate) chrome: ChromeRefinement,
    pub(crate) layout: LayoutRefinement,
    pub(crate) direction: Axis,
    pub(crate) gap: MetricRef,
    pub(crate) justify: Justify,
    pub(crate) items: Items,
    pub(crate) wrap: bool,
    pub(crate) children: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

impl<H, F> FlexBox<H, F> {
    pub fn new(direction: Axis, children: F) -> Self {
        let items = match direction {
            Axis::Horizontal => Items::Center,
            Axis::Vertical => Items::Stretch,
        };
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            direction,
            gap: MetricRef::space(Space::N0),
            justify: Justify::Start,
            items,
            wrap: false,
            children: Some(children),
            _phantom: PhantomData,
        }
    }
}

impl<H, F> UiPatchTarget for FlexBox<H, F> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, F> UiSupportsChrome for FlexBox<H, F> {}
impl<H, F> UiSupportsLayout for FlexBox<H, F> {}

impl<H: UiHost, F, I> FlexBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);

        let container = decl_style::container_props(theme, self.chrome, self.layout);

        let gap = self.gap.resolve(theme);
        let flex_props = FlexProps {
            direction: self.direction,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: self.justify.to_main_align(),
            align: self.items.to_cross_align(),
            wrap: self.wrap,
            ..Default::default()
        };

        let children = self.children.expect("expected flex children closure");
        cx.container(container, move |cx| {
            vec![cx.flex(flex_props, move |cx| children(cx))]
        })
    }
}

/// Returns a patchable horizontal flex layout builder.
///
/// Usage:
/// - `ui::h_flex(cx, |cx| vec![...]).gap(Space::N2).px_2().into_element(cx)`
pub fn h_flex<H: UiHost, F, I>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    UiBuilder::new(FlexBox::new(Axis::Horizontal, children))
}

/// Returns a patchable vertical flex layout builder.
pub fn v_flex<H: UiHost, F, I>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    UiBuilder::new(FlexBox::new(Axis::Vertical, children))
}

/// A patchable container constructor for authoring ergonomics.
///
/// This is intended to be the default “box” layout node in the fluent authoring surface.
#[derive(Debug, Clone)]
pub struct ContainerBox<H, F> {
    pub(crate) chrome: ChromeRefinement,
    pub(crate) layout: LayoutRefinement,
    pub(crate) children: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

impl<H, F> ContainerBox<H, F> {
    pub fn new(children: F) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children: Some(children),
            _phantom: PhantomData,
        }
    }
}

impl<H, F> UiPatchTarget for ContainerBox<H, F> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, F> UiSupportsChrome for ContainerBox<H, F> {}
impl<H, F> UiSupportsLayout for ContainerBox<H, F> {}

impl<H: UiHost, F, I> ContainerBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let container = decl_style::container_props(theme, self.chrome, self.layout);
        let children = self.children.expect("expected container children closure");
        cx.container(container, move |cx| children(cx))
    }
}

/// Returns a patchable container builder.
///
/// Usage:
/// - `ui::container(cx, |cx| vec![...]).px_2().into_element(cx)`
pub fn container<H: UiHost, F, I>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<ContainerBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    UiBuilder::new(ContainerBox::new(children))
}

/// A patchable stack layout constructor for authoring ergonomics.
///
/// The runtime `Stack` element is a positioned-container style layout: children can be absolutely
/// positioned, and non-absolute children are laid out against the same bounds.
#[derive(Debug, Clone)]
pub struct StackBox<H, F> {
    pub(crate) chrome: ChromeRefinement,
    pub(crate) layout: LayoutRefinement,
    pub(crate) children: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

impl<H, F> StackBox<H, F> {
    pub fn new(children: F) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children: Some(children),
            _phantom: PhantomData,
        }
    }
}

impl<H, F> UiPatchTarget for StackBox<H, F> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, F> UiSupportsChrome for StackBox<H, F> {}
impl<H, F> UiSupportsLayout for StackBox<H, F> {}

impl<H: UiHost, F, I> StackBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let container = decl_style::container_props(theme, self.chrome, self.layout);
        let children = self.children.expect("expected stack children closure");

        cx.container(container, move |cx| {
            vec![cx.stack_props(StackProps::default(), move |cx| children(cx))]
        })
    }
}

/// Returns a patchable stack layout builder.
///
/// Usage:
/// - `ui::stack(cx, |cx| vec![...]).inset(Space::N2).into_element(cx)`
pub fn stack<H: UiHost, F, I>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<StackBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    UiBuilder::new(StackBox::new(children))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextPreset {
    Sm,
    Base,
    Prose,
    Label,
}

/// A patchable text constructor for authoring ergonomics.
///
/// This is intentionally small: it supports layout patching and a minimal text refinement surface
/// (size preset, weight, color, wrap/overflow).
#[derive(Debug, Clone)]
pub struct TextBox {
    pub(crate) layout: LayoutRefinement,
    pub(crate) text: Arc<str>,
    pub(crate) preset: TextPreset,
    pub(crate) size_override: Option<Px>,
    pub(crate) line_height_override: Option<Px>,
    pub(crate) weight_override: Option<FontWeight>,
    pub(crate) letter_spacing_em_override: Option<f32>,
    pub(crate) color_override: Option<crate::ColorRef>,
    pub(crate) wrap: TextWrap,
    pub(crate) overflow: TextOverflow,
}

impl TextBox {
    pub fn new(text: impl Into<Arc<str>>, preset: TextPreset) -> Self {
        let wrap = match preset {
            TextPreset::Label => TextWrap::None,
            TextPreset::Sm | TextPreset::Base | TextPreset::Prose => TextWrap::Word,
        };

        Self {
            layout: LayoutRefinement::default(),
            text: text.into(),
            preset,
            size_override: None,
            line_height_override: None,
            weight_override: None,
            letter_spacing_em_override: None,
            color_override: None,
            wrap,
            overflow: TextOverflow::Clip,
        }
    }
}

impl UiPatchTarget for TextBox {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl UiSupportsLayout for TextBox {}

impl UiIntoElement for TextBox {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut style = match self.preset {
            TextPreset::Sm => decl_text::text_sm_style(&theme),
            TextPreset::Base => decl_text::text_base_style(&theme),
            TextPreset::Prose => decl_text::text_prose_style(&theme),
            TextPreset::Label => {
                let (style, _) = decl_text::label_style(&theme);
                style
            }
        };

        if let Some(size) = self.size_override {
            style.size = size;
        }
        if let Some(height) = self.line_height_override {
            style.line_height = Some(height);
        }
        if let Some(weight) = self.weight_override {
            style.weight = weight;
        }
        if let Some(letter_spacing_em) = self.letter_spacing_em_override {
            style.letter_spacing_em = Some(letter_spacing_em);
        }

        let mut layout = decl_style::layout_style(&theme, self.layout);
        if self.preset == TextPreset::Label && matches!(layout.size.height, Length::Auto) {
            let line_height = self
                .line_height_override
                .unwrap_or_else(|| decl_text::label_style(&theme).1);
            layout.size.height = Length::Px(line_height);
        }

        let color = self.color_override.map(|c| c.resolve(&theme)).or_else(|| {
            (self.preset == TextPreset::Label).then(|| {
                theme
                    .color_by_key("foreground")
                    .unwrap_or_else(|| theme.color_required("foreground"))
            })
        });

        cx.text_props(TextProps {
            layout,
            text: self.text,
            style: Some(style),
            color,
            wrap: self.wrap,
            overflow: self.overflow,
        })
    }
}

/// Returns a patchable text builder (shadcn-aligned defaults).
///
/// Usage:
/// - `ui::text(cx, "Hello").text_sm().font_medium().into_element(cx)`
pub fn text<H: UiHost>(
    _cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> UiBuilder<TextBox> {
    UiBuilder::new(TextBox::new(text, TextPreset::Sm))
}

/// Returns a patchable label builder (single-line, medium weight).
pub fn label<H: UiHost>(
    _cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> UiBuilder<TextBox> {
    UiBuilder::new(TextBox::new(text, TextPreset::Label))
}

/// A patchable unstyled text builder matching `TextProps::new(...)` defaults.
#[derive(Debug, Clone)]
pub struct RawTextBox {
    pub(crate) layout: LayoutRefinement,
    pub(crate) text: Arc<str>,
    pub(crate) color_override: Option<crate::ColorRef>,
    pub(crate) wrap: TextWrap,
    pub(crate) overflow: TextOverflow,
}

impl RawTextBox {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            layout: LayoutRefinement::default(),
            text: text.into(),
            color_override: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        }
    }
}

impl UiPatchTarget for RawTextBox {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl UiSupportsLayout for RawTextBox {}

impl UiIntoElement for RawTextBox {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let layout = decl_style::layout_style(&theme, self.layout);
        let color = self.color_override.map(|c| c.resolve(&theme));

        cx.text_props(TextProps {
            layout,
            text: self.text,
            style: None,
            color,
            wrap: self.wrap,
            overflow: self.overflow,
        })
    }
}

/// Returns a patchable unstyled text builder matching `TextProps::new(...)` defaults.
pub fn raw_text<H: UiHost>(
    _cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> UiBuilder<RawTextBox> {
    UiBuilder::new(RawTextBox::new(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiExt;
    use crate::{LengthRefinement, MetricRef};

    #[test]
    fn container_box_accepts_ui_patches() {
        let container = ContainerBox::<(), ()>::new(())
            .ui()
            .p_1()
            .w(LengthRefinement::Fill)
            .build();

        let padding = container
            .chrome
            .padding
            .expect("expected padding refinement");
        assert!(matches!(padding.left, Some(MetricRef::Token { .. })));
        assert!(container.layout.size.is_some());
    }

    #[test]
    fn text_box_supports_layout_and_text_refinements() {
        let text = TextBox::new("hello", TextPreset::Sm)
            .ui()
            .w(LengthRefinement::Fill)
            .font_bold()
            .build();

        assert!(text.layout.size.is_some());
        assert_eq!(text.weight_override, Some(FontWeight::BOLD));
    }

    #[test]
    fn stack_box_accepts_ui_patches() {
        let stack = StackBox::<(), ()>::new(())
            .ui()
            .p_1()
            .w(LengthRefinement::Fill)
            .build();

        let padding = stack.chrome.padding.expect("expected padding refinement");
        assert!(matches!(padding.left, Some(MetricRef::Token { .. })));
        assert!(stack.layout.size.is_some());
    }
}
