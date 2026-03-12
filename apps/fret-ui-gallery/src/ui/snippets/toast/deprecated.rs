pub const SOURCE: &str = include_str!("deprecated.rs");

// region: example
use fret::UiCx;
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CMD_NAV_SONNER: &str = "ui_gallery.nav.select.sonner";

fn centered(cx: &mut UiCx<'_>, body: AnyElement) -> AnyElement {
    ui::h_flex(move |_cx| [body])
        .layout(LayoutRefinement::default().w_full())
        .justify_center()
        .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let deprecated_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Toast is deprecated").into_element(cx),
            shadcn::CardDescription::new(
                "The toast component is deprecated in shadcn/ui docs. Use Sonner instead.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![shadcn::raw::typography::muted(
            cx,
            "This page intentionally keeps only the deprecation guidance to match upstream docs.",
        )])
        .into_element(cx),
        shadcn::CardFooter::new(vec![
            shadcn::Button::new("Open Sonner page")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_NAV_SONNER)
                .test_id("ui-gallery-toast-open-sonner")
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-toast-deprecated");

    centered(cx, deprecated_card)
}

// endregion: example
