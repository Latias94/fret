use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::UiBuilder;

use crate::breadcrumb::primitives;

pub trait BreadcrumbPrimitivesUiBuilderExt {
    fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>;
}

impl BreadcrumbPrimitivesUiBuilderExt for UiBuilder<primitives::Breadcrumb> {
    fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.build().into_element(cx, children)
    }
}

impl BreadcrumbPrimitivesUiBuilderExt for UiBuilder<primitives::BreadcrumbList> {
    fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.build().into_element(cx, children)
    }
}

impl BreadcrumbPrimitivesUiBuilderExt for UiBuilder<primitives::BreadcrumbItem> {
    fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.build().into_element(cx, children)
    }
}
