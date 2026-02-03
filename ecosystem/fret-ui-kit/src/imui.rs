//! Optional integration helpers for immediate-mode authoring frontends.
//!
//! This module lives in `fret-ui-kit` (not `fret-imui`) to preserve dependency direction:
//!
//! - `fret-imui` stays policy-light and depends only on `fret-ui` (+ `fret-authoring` contract).
//! - `fret-ui-kit` can optionally provide bridges that allow `UiBuilder<T>` patch vocabulary to be
//!   used from immediate-style control flow.

use fret_authoring::UiWriter;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::UiBuilder;
use crate::{UiIntoElement, UiPatchTarget};

/// A value that can be rendered into a declarative element within an `ElementContext`.
///
/// This is used to bridge the `UiBuilder<T>` ecosystem authoring surface (ADR 0175) into
/// immediate-mode frontends (`UiWriter`).
pub trait UiKitIntoElement<H: UiHost> {
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}

impl<H: UiHost, T> UiKitIntoElement<H> for UiBuilder<T>
where
    T: UiPatchTarget + UiIntoElement,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, B> UiKitIntoElement<H> for UiBuilder<crate::ui::FlexBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, B> UiKitIntoElement<H> for UiBuilder<crate::ui::ContainerBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::ContainerBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::StackBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::ScrollAreaBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, B> UiKitIntoElement<H> for UiBuilder<crate::ui::ScrollAreaBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

/// Extension trait bridging `fret-ui-kit` authoring (`UiBuilder<T>`) into an immediate-mode output.
pub trait UiWriterUiKitExt<H: UiHost>: UiWriter<H> {
    /// Render a `UiBuilder<T>` (or other supported authoring value) into the current output list.
    #[track_caller]
    fn add_ui<B>(&mut self, value: B)
    where
        B: UiKitIntoElement<H>,
    {
        let element = self.with_cx_mut(|cx| value.into_any_element(cx));
        self.add(element);
    }
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterUiKitExt<H> for W {}
