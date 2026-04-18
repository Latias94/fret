pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
#[allow(unused_imports)]
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CMD_APP_OPEN: &str = "ui_gallery.app.open";

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::BreadcrumbRoot::new()
        .into_element(cx, |cx| {
            vec![shadcn::BreadcrumbList::new().into_element(cx, |cx| {
                vec![
                    shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                        vec![
                            shadcn::BreadcrumbLink::new("Home")
                                .href("/")
                                // Keep the gallery deterministic while preserving link semantics.
                                .action(CMD_APP_OPEN)
                                .into_element(cx)
                                .test_id("ui-gallery-breadcrumb-usage-home-link"),
                        ]
                    }),
                    shadcn::BreadcrumbSeparatorPart::new().into_element(cx),
                    shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                        vec![
                            shadcn::BreadcrumbLink::new("Components")
                                .href("/components")
                                .action(CMD_APP_OPEN)
                                .into_element(cx)
                                .test_id("ui-gallery-breadcrumb-usage-components-link"),
                        ]
                    }),
                    shadcn::BreadcrumbSeparatorPart::new().into_element(cx),
                    shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                        vec![shadcn::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                    }),
                ]
            })]
        })
        .test_id("ui-gallery-breadcrumb-usage")
}
// endregion: example
