use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{IntoUiElement, UiBuilder};

use crate::{AlertDialog, Dialog, Drawer, Popover, Sheet};

pub trait PopoverUiBuilderExt {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>;
}

impl PopoverUiBuilderExt for UiBuilder<Popover> {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>,
    {
        self.build().into_element_with(
            cx,
            |cx| trigger(cx).into_element(cx),
            |cx| content(cx).into_element(cx),
        )
    }
}

pub trait DialogUiBuilderExt {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>;
}

impl DialogUiBuilderExt for UiBuilder<Dialog> {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>,
    {
        self.build().into_element(
            cx,
            |cx| trigger(cx).into_element(cx),
            |cx| content(cx).into_element(cx),
        )
    }
}

pub trait AlertDialogUiBuilderExt {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>;
}

impl AlertDialogUiBuilderExt for UiBuilder<AlertDialog> {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>,
    {
        self.build().into_element(
            cx,
            |cx| trigger(cx).into_element(cx),
            |cx| content(cx).into_element(cx),
        )
    }
}

pub trait SheetUiBuilderExt {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>;
}

impl SheetUiBuilderExt for UiBuilder<Sheet> {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>,
    {
        self.build().into_element(
            cx,
            |cx| trigger(cx).into_element(cx),
            |cx| content(cx).into_element(cx),
        )
    }
}

pub trait DrawerUiBuilderExt {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>;
}

impl DrawerUiBuilderExt for UiBuilder<Drawer> {
    fn into_element<H: UiHost, TTrigger, TContent>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent,
    ) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
        TContent: IntoUiElement<H>,
    {
        self.build().into_element(
            cx,
            |cx| trigger(cx).into_element(cx),
            |cx| content(cx).into_element(cx),
        )
    }
}
