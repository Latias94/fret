use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::UiBuilder;

use crate::{
    AlertDialog, CommandDialog, ContextMenu, ContextMenuEntry, Dialog, Drawer, DropdownMenu,
    DropdownMenuEntry, Popover, Sheet,
};

pub trait PopoverUiBuilderExt {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement;
}

impl PopoverUiBuilderExt for UiBuilder<Popover> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.build().into_element(cx, trigger, content)
    }
}

pub trait DialogUiBuilderExt {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement;
}

impl DialogUiBuilderExt for UiBuilder<Dialog> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.build().into_element(cx, trigger, content)
    }
}

pub trait AlertDialogUiBuilderExt {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement;
}

impl AlertDialogUiBuilderExt for UiBuilder<AlertDialog> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.build().into_element(cx, trigger, content)
    }
}

pub trait SheetUiBuilderExt {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement;
}

impl SheetUiBuilderExt for UiBuilder<Sheet> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.build().into_element(cx, trigger, content)
    }
}

pub trait DrawerUiBuilderExt {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement;
}

impl DrawerUiBuilderExt for UiBuilder<Drawer> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.build().into_element(cx, trigger, content)
    }
}

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

pub trait CommandDialogUiBuilderExt {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement;
}

impl CommandDialogUiBuilderExt for UiBuilder<CommandDialog> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.build().into_element(cx, trigger)
    }
}
