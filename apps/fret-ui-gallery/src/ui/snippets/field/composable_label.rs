pub const SOURCE: &str = include_str!("composable_label.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let manual_review = cx.local_model_keyed("manual_review", || false);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::FieldSet::new([
        shadcn::FieldLabel::new("Deployment Policy").into_element(cx),
        shadcn::FieldDescription::new(
            "Use FieldLabel::wrap(...) when a richer card layout should still forward activation to a control.",
        )
        .into_element(cx),
        shadcn::FieldLabel::new("Require manual approval")
            .for_control("ui-gallery-field-composable-label-manual-review")
            .wrap([shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldTitle::new("Require manual approval").into_element(cx),
                    shadcn::FieldDescription::new(
                        "Pause deployment changes until an operator explicitly approves them.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Switch::new(manual_review)
                    .control_id("ui-gallery-field-composable-label-manual-review")
                    .a11y_label("Require manual approval")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx)])
            .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-composable-label")
}
// endregion: example
