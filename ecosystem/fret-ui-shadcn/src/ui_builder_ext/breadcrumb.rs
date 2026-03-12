use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{IntoUiElement, UiBuilder};

use crate::breadcrumb::primitives;

pub trait BreadcrumbPrimitivesUiBuilderExt {
    fn into_element<H: UiHost, I, TChild>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = TChild>,
        TChild: IntoUiElement<H>;
}

impl BreadcrumbPrimitivesUiBuilderExt for UiBuilder<primitives::Breadcrumb> {
    fn into_element<H: UiHost, I, TChild>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = TChild>,
        TChild: IntoUiElement<H>,
    {
        self.build().into_element(cx, |cx| {
            let mut landed = Vec::new();
            for child in children(cx) {
                landed.push(child.into_element(cx));
            }
            landed
        })
    }
}

impl BreadcrumbPrimitivesUiBuilderExt for UiBuilder<primitives::BreadcrumbList> {
    fn into_element<H: UiHost, I, TChild>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = TChild>,
        TChild: IntoUiElement<H>,
    {
        self.build().into_element(cx, |cx| {
            let mut landed = Vec::new();
            for child in children(cx) {
                landed.push(child.into_element(cx));
            }
            landed
        })
    }
}

impl BreadcrumbPrimitivesUiBuilderExt for UiBuilder<primitives::BreadcrumbItem> {
    fn into_element<H: UiHost, I, TChild>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = TChild>,
        TChild: IntoUiElement<H>,
    {
        self.build().into_element(cx, |cx| {
            let mut landed = Vec::new();
            for child in children(cx) {
                landed.push(child.into_element(cx));
            }
            landed
        })
    }
}
