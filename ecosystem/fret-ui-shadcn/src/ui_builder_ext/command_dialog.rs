use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{IntoUiElement, UiBuilder};

use crate::CommandDialog;

pub trait CommandDialogUiBuilderExt {
    fn into_element<H: UiHost, TTrigger>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>;
}

impl CommandDialogUiBuilderExt for UiBuilder<CommandDialog> {
    fn into_element<H: UiHost, TTrigger>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
    {
        self.build()
            .into_element(cx, |cx| trigger(cx).into_element(cx))
    }
}
