pub const SOURCE: &str = include_str!("custom_separator.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
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
                    bc::BreadcrumbSeparator::new()
                        .children(|cx| {
                            [shadcn::raw::icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SLASH,
                                Some(Px(14.0)),
                                None,
                            )]
                        })
                        .into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![
                            bc::BreadcrumbLink::new("Components")
                                .href("/components")
                                .action(CMD_APP_OPEN)
                                .into_element(cx),
                        ]
                    }),
                    bc::BreadcrumbSeparator::new()
                        .children(|cx| {
                            [shadcn::raw::icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SLASH,
                                Some(Px(14.0)),
                                None,
                            )]
                        })
                        .into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                    }),
                ]
            })]
        })
        .test_id("ui-gallery-breadcrumb-separator")
}
// endregion: example
