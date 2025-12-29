//! shadcn/ui `Drawer` facade.
//!
//! Fret currently models drawers as a `Sheet` that defaults to the `Bottom` side.

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementCx, UiHost};

pub use crate::sheet::{
    SheetContent as DrawerContent, SheetDescription as DrawerDescription, SheetFooter as DrawerFooter,
    SheetHeader as DrawerHeader, SheetSide as DrawerSide, SheetTitle as DrawerTitle,
};
use crate::Sheet;

#[derive(Clone)]
pub struct Drawer {
    inner: Sheet,
}

impl std::fmt::Debug for Drawer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drawer").finish()
    }
}

impl Drawer {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            inner: Sheet::new(open).side(DrawerSide::Bottom),
        }
    }

    pub fn overlay_closable(mut self, overlay_closable: bool) -> Self {
        self.inner = self.inner.overlay_closable(overlay_closable);
        self
    }

    pub fn overlay_color(mut self, overlay_color: fret_core::Color) -> Self {
        self.inner = self.inner.overlay_color(overlay_color);
        self
    }

    /// Sets the drawer size (height by default, since drawers default to `Bottom`).
    pub fn size(mut self, size: fret_core::Px) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    /// Optional escape hatch: allow non-bottom drawers by forwarding to `Sheet`.
    pub fn side(mut self, side: DrawerSide) -> Self {
        self.inner = self.inner.side(side);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementCx<'_, H>,
        trigger: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.inner.into_element(cx, trigger, content)
    }
}

pub fn drawer<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    open: Model<bool>,
    trigger: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
    content: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
) -> AnyElement {
    Drawer::new(open).into_element(cx, trigger, content)
}
