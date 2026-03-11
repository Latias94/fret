pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use std::f32::consts::TAU;

use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let custom_icon_row = ui::h_row(|cx| {
        vec![
            shadcn::Spinner::new().into_element(cx),
            shadcn::Spinner::new()
                .icon(fret_icons::ids::ui::SETTINGS)
                .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-spinner-extras-custom-icon");

    let speed_row = ui::h_row(|cx| {
        vec![
            shadcn::Spinner::new().speed(0.0).into_element(cx),
            shadcn::Spinner::new().into_element(cx),
            shadcn::Spinner::new().speed(-TAU / 60.0).into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-spinner-extras-speeds");

    ui::v_flex(|cx| {
            vec![
                shadcn::raw::typography::muted(
                    cx,
                    "Extras are Fret-specific demos and regression gates (not part of upstream shadcn SpinnerDemo).",
                ),
                custom_icon_row,
                speed_row,
            ]
        })
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx)
    .test_id("ui-gallery-spinner-extras")
}

// endregion: example
