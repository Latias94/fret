use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::UiBuilder;

use crate::breadcrumb::primitives;

pub trait BreadcrumbPrimitivesUiBuilderExt {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement;
}

impl BreadcrumbPrimitivesUiBuilderExt for UiBuilder<primitives::Breadcrumb> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.build().into_element(cx, children)
    }
}

impl BreadcrumbPrimitivesUiBuilderExt for UiBuilder<primitives::BreadcrumbList> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.build().into_element(cx, children)
    }
}

impl BreadcrumbPrimitivesUiBuilderExt for UiBuilder<primitives::BreadcrumbItem> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.build().into_element(cx, children)
    }
}
