pub const SOURCE: &str = include_str!("collapsed.rs");

// region: example
use fret::{UiChild, UiCx};
#[allow(unused_imports)]
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CMD_APP_OPEN: &str = "ui_gallery.app.open";

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::BreadcrumbRoot::new()
        .into_element(cx, |cx| {
            vec![shadcn::BreadcrumbList::new().into_element(cx, |cx| {
                vec![
                    shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                        vec![
                            shadcn::BreadcrumbLink::new("Home")
                                .href("/")
                                .action(CMD_APP_OPEN)
                                .into_element(cx),
                        ]
                    }),
                    shadcn::BreadcrumbSeparatorPart::new().into_element(cx),
                    shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                        vec![shadcn::BreadcrumbEllipsis::new().into_element(cx)]
                    }),
                    shadcn::BreadcrumbSeparatorPart::new().into_element(cx),
                    shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                        vec![
                            shadcn::BreadcrumbLink::new("Components")
                                .href("/docs/components")
                                .action(CMD_APP_OPEN)
                                .into_element(cx),
                        ]
                    }),
                    shadcn::BreadcrumbSeparatorPart::new().into_element(cx),
                    shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                        vec![shadcn::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                    }),
                ]
            })]
        })
        .test_id("ui-gallery-breadcrumb-collapsed")
}
// endregion: example
