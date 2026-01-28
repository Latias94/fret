use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::UiBuilder;

use crate::{ContextMenu, ContextMenuEntry, DropdownMenu, DropdownMenuEntry};

pub trait DropdownMenuUiBuilderExt {
    fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = DropdownMenuEntry>;
}

impl DropdownMenuUiBuilderExt for UiBuilder<DropdownMenu> {
    fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = DropdownMenuEntry>,
    {
        self.build().into_element(cx, trigger, entries)
    }
}

pub trait ContextMenuUiBuilderExt {
    fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = ContextMenuEntry>;
}

impl ContextMenuUiBuilderExt for UiBuilder<ContextMenu> {
    fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = ContextMenuEntry>,
    {
        self.build().into_element(cx, trigger, entries)
    }
}
