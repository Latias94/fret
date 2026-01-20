use std::marker::PhantomData;

use fret_core::{Axis, Edges, Px};
use fret_ui::element::{AnyElement, FlexProps, StackProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::style as decl_style;
use crate::{
    ChromeRefinement, Items, Justify, LayoutRefinement, Space, UiBuilder, UiPatch, UiPatchTarget,
    UiSupportsChrome, UiSupportsLayout,
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
    pub(crate) gap: Space,
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
            gap: Space::N0,
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

impl<H: UiHost, F> FlexBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);

        let container = decl_style::container_props(theme, self.chrome, self.layout);

        let gap = decl_style::space(theme, self.gap);
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
pub fn h_flex<H: UiHost, F>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    UiBuilder::new(FlexBox::new(Axis::Horizontal, children))
}

/// Returns a patchable vertical flex layout builder.
pub fn v_flex<H: UiHost, F>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    UiBuilder::new(FlexBox::new(Axis::Vertical, children))
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

impl<H: UiHost, F> StackBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
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
pub fn stack<H: UiHost, F>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<StackBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    UiBuilder::new(StackBox::new(children))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiExt;
    use crate::{LengthRefinement, MetricRef};

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
