pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::breadcrumb::primitives as bc;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    bc::Breadcrumb::new()
        .into_element(cx, |cx| {
            vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                vec![
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![bc::BreadcrumbLink::new("Home").href("/").into_element(cx)]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![
                            bc::BreadcrumbLink::new("Components")
                                .href("/components")
                                .into_element(cx),
                        ]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                    }),
                ]
            })]
        })
        .test_id("ui-gallery-breadcrumb-usage")
}
// endregion: example
