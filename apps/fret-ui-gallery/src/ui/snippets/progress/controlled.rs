pub const SOURCE: &str = include_str!("controlled.rs");

// region: example
use fret::UiCx;
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let values = cx.local_model(|| vec![50.0]);

    let centered = |cx: &mut UiCx<'_>, body: AnyElement| {
        ui::h_flex(move |_cx| [body])
            .layout(LayoutRefinement::default().w_full())
            .justify_center()
            .into_element(cx)
    };

    cx.keyed("ui_gallery.progress.controlled", |cx| {
        let body = ui::v_flex(|cx| {
            vec![
                shadcn::Progress::new_values_first(values.clone())
                    .into_element(cx)
                    .test_id("ui-gallery-progress-controlled-bar"),
                shadcn::Slider::new(values.clone())
                    .range(0.0, 100.0)
                    .step(1.0)
                    .a11y_label("Progress value")
                    .into_element(cx)
                    .test_id("ui-gallery-progress-controlled-slider"),
            ]
        })
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx);

        centered(cx, body).test_id("ui-gallery-progress-controlled")
    })
}

// endregion: example
