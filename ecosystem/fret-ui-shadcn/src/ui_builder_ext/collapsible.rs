use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{IntoUiElement, UiBuilder};

use crate::collapsible::Collapsible;

pub trait CollapsibleUiBuilderExt {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>, bool) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>;

    fn into_element_with_open_model<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>, Model<bool>, bool) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>;
}

impl CollapsibleUiBuilderExt for UiBuilder<Collapsible> {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>, bool) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>,
    {
        self.build().into_element(
            cx,
            |cx, open| trigger(cx, open).into_element(cx),
            |cx| content(cx).into_element(cx),
        )
    }

    fn into_element_with_open_model<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>, Model<bool>, bool) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>,
    {
        self.build().into_element_with_open_model(
            cx,
            |cx, open_model, open| trigger(cx, open_model, open).into_element(cx),
            |cx| content(cx).into_element(cx),
        )
    }
}
