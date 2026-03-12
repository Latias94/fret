use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{IntoUiElement, UiBuilder};

use crate::{ContextMenu, ContextMenuEntry, DropdownMenu, DropdownMenuEntry};

pub trait DropdownMenuUiBuilderExt {
    fn into_element<H: UiHost, I, TTrigger>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = DropdownMenuEntry>,
        TTrigger: IntoUiElement<H>;
}

impl DropdownMenuUiBuilderExt for UiBuilder<DropdownMenu> {
    fn into_element<H: UiHost, I, TTrigger>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = DropdownMenuEntry>,
        TTrigger: IntoUiElement<H>,
    {
        self.build()
            .into_element(cx, |cx| trigger(cx).into_element(cx), entries)
    }
}

pub trait ContextMenuUiBuilderExt {
    fn into_element<H: UiHost, I, TTrigger>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = ContextMenuEntry>,
        TTrigger: IntoUiElement<H>;
}

impl ContextMenuUiBuilderExt for UiBuilder<ContextMenu> {
    fn into_element<H: UiHost, I, TTrigger>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = ContextMenuEntry>,
        TTrigger: IntoUiElement<H>,
    {
        self.build()
            .into_element(cx, |cx| trigger(cx).into_element(cx), entries)
    }
}
