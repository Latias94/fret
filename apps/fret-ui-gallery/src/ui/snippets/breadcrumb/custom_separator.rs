pub const SOURCE: &str = include_str!("custom_separator.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
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
                    shadcn::BreadcrumbSeparatorPart::new()
                        .children(|cx| {
                            [shadcn::raw::icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SLASH,
                                Some(Px(14.0)),
                                None,
                            )]
                        })
                        .into_element(cx),
                    shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                        vec![
                            shadcn::BreadcrumbLink::new("Components")
                                .href("/components")
                                .action(CMD_APP_OPEN)
                                .into_element(cx),
                        ]
                    }),
                    shadcn::BreadcrumbSeparatorPart::new()
                        .children(|cx| {
                            [shadcn::raw::icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SLASH,
                                Some(Px(14.0)),
                                None,
                            )]
                        })
                        .into_element(cx),
                    shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                        vec![shadcn::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                    }),
                ]
            })]
        })
        .test_id("ui-gallery-breadcrumb-separator")
}
// endregion: example
