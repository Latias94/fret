pub const SOURCE: &str = include_str!("deprecated.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CMD_NAV_SONNER: &str = "ui_gallery.nav.select.sonner";

fn centered<B>(body: B) -> impl IntoUiElement<fret_app::App> + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    ui::h_flex(move |cx| [body.into_element(cx)])
        .layout(LayoutRefinement::default().w_full())
        .justify_center()
}

pub fn render(_cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let deprecated_card = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Toast is deprecated"),
                    shadcn::card_description(
                        "The toast component is deprecated in shadcn/ui docs. Use Sonner instead.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    shadcn::raw::typography::muted(
                        "This page intentionally keeps only the deprecation guidance to match upstream docs.",
                    ),
                ]
            }),
            shadcn::card_footer(|cx| {
                ui::children![
                    cx;
                    shadcn::Button::new("Open Sonner page")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .action(CMD_NAV_SONNER)
                        .ui()
                        .test_id("ui-gallery-toast-open-sonner"),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .test_id("ui-gallery-toast-deprecated");

    centered(deprecated_card)
}

// endregion: example
