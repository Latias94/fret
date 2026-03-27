pub const SOURCE: &str = include_str!("link_component.rs");

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
                                .href("https://example.com")
                                // Keep the gallery deterministic while preserving link semantics.
                                .action(CMD_APP_OPEN)
                                // This is the explicit Fret alternative to upstream `render` /
                                // `asChild`: keep the link surface typed, but allow custom inline
                                // children for the visual content.
                                .children(|cx| {
                                    [ui::h_row(move |cx| {
                                        vec![
                                            shadcn::raw::icon::icon_with(
                                                cx,
                                                fret_icons::ids::ui::FOLDER_OPEN,
                                                Some(Px(14.0)),
                                                None,
                                            ),
                                            ui::text("Home").into_element(cx),
                                        ]
                                    })
                                    .gap(Space::N1)
                                    .items_center()
                                    .into_element(cx)]
                                })
                                .into_element(cx),
                        ]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![
                            bc::BreadcrumbLink::new("Components")
                                .href("https://example.com/components")
                                .action(CMD_APP_OPEN)
                                .children(|cx| {
                                    [ui::h_row(move |cx| {
                                        vec![
                                            shadcn::raw::icon::icon_with(
                                                cx,
                                                fret_icons::ids::ui::FOLDER,
                                                Some(Px(14.0)),
                                                None,
                                            ),
                                            ui::text("Components").into_element(cx),
                                        ]
                                    })
                                    .gap(Space::N1)
                                    .items_center()
                                    .into_element(cx)]
                                })
                                .into_element(cx),
                        ]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![
                            bc::BreadcrumbPage::new("Breadcrumb")
                                .children(|cx| [ui::text("Breadcrumb").into_element(cx)])
                                .into_element(cx),
                        ]
                    }),
                ]
            })]
        })
        .test_id("ui-gallery-breadcrumb-link")
}
// endregion: example
