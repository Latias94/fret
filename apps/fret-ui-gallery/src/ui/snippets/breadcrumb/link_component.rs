pub const SOURCE: &str = include_str!("link_component.rs");

// region: example
use fret_ui_shadcn::breadcrumb::primitives as bc;
use fret_ui_shadcn::prelude::*;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    bc::Breadcrumb::new()
        .into_element(cx, |cx| {
            vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                vec![
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![
                            bc::BreadcrumbLink::new("Home")
                                .href("https://example.com")
                                // Keep the gallery deterministic while preserving link semantics.
                                .on_click(CMD_APP_OPEN)
                                // This is the explicit Fret alternative to upstream `render` /
                                // `asChild`: keep the link surface typed, but allow custom inline
                                // children for the visual content.
                                .children([ui::text("Home").into_element(cx)])
                                .into_element(cx),
                        ]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![
                            bc::BreadcrumbLink::new("Components")
                                .href("https://example.com/components")
                                .on_click(CMD_APP_OPEN)
                                .children([ui::text("Components").into_element(cx)])
                                .into_element(cx),
                        ]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![
                            bc::BreadcrumbPage::new("Breadcrumb")
                                .children([ui::text("Breadcrumb").into_element(cx)])
                                .into_element(cx),
                        ]
                    }),
                ]
            })]
        })
        .test_id("ui-gallery-breadcrumb-link")
}
// endregion: example
