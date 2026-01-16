use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::UiBuilder;

use crate::{ContextMenu, ContextMenuEntry, DropdownMenu, DropdownMenuEntry};

pub trait DropdownMenuUiBuilderExt {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<DropdownMenuEntry>,
    ) -> AnyElement;
}

impl DropdownMenuUiBuilderExt for UiBuilder<DropdownMenu> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<DropdownMenuEntry>,
    ) -> AnyElement {
        self.build().into_element(cx, trigger, entries)
    }
}

pub trait ContextMenuUiBuilderExt {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ContextMenuEntry>,
    ) -> AnyElement;
}

impl ContextMenuUiBuilderExt for UiBuilder<ContextMenu> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ContextMenuEntry>,
    ) -> AnyElement {
        self.build().into_element(cx, trigger, entries)
    }
}
