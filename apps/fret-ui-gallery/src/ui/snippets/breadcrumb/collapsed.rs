pub const SOURCE: &str = include_str!("collapsed.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use shadcn::raw::breadcrumb::primitives as bc;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    bc::Breadcrumb::new()
        .into_element(cx, |cx| {
            vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                vec![
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![
                            bc::BreadcrumbLink::new("Home")
                                .href("/")
                                .action(CMD_APP_OPEN)
                                .into_element(cx),
                        ]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![bc::BreadcrumbEllipsis::new().into_element(cx)]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![
                            bc::BreadcrumbLink::new("Components")
                                .href("/docs/components")
                                .action(CMD_APP_OPEN)
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
        .test_id("ui-gallery-breadcrumb-collapsed")
}
// endregion: example
