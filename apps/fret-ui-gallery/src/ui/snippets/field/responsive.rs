pub const SOURCE: &str = include_str!("responsive.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let username = cx.local_model_keyed("username", String::new);
    let wide = cx.local_model_keyed("wide", || false);

    let wide_value = cx.watch_model(&wide).layout().copied().unwrap_or(false);
    let max_w = if wide_value { Px(900.0) } else { Px(520.0) };
    let name_id = "ui-gallery-field-responsive-name";

    let width_toggle = shadcn::FieldGroup::new([shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Responsive width")
                .for_control("ui-gallery-field-responsive-width-switch")
                .into_element(cx),
            shadcn::FieldDescription::new(
                "Toggle the container width to exercise responsive orientation via container queries.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Switch::new(wide)
            .control_id("ui-gallery-field-responsive-width-switch")
            .test_id("ui-gallery-field-responsive-width-switch")
            .a11y_label("Use wide responsive container")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .into_element(cx)])
    .into_element(cx)
    .test_id("ui-gallery-field-responsive-width-toggle");

    let content = shadcn::FieldSet::new([
        shadcn::FieldLegend::new("Profile").into_element(cx),
        shadcn::FieldDescription::new("Fill in your profile information.").into_element(cx),
        shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Name")
                        .for_control(name_id)
                        .into_element(cx),
                    shadcn::FieldDescription::new("Provide your full name for identification.")
                        .into_element(cx),
                ])
                .into_element(cx)
                .test_id("ui-gallery-field-responsive-name-content"),
                shadcn::Input::new(username)
                    .control_id(name_id)
                    .placeholder("Evil Rabbit")
                    .a11y_label("Name")
                    .into_element(cx)
                    .test_id("ui-gallery-field-responsive-name-input"),
            ])
            .orientation(shadcn::FieldOrientation::Responsive)
            .into_element(cx),
            shadcn::Field::new([
                shadcn::Button::new("Submit").into_element(cx),
                shadcn::Button::new("Cancel")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Responsive)
            .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(max_w))
    .into_element(cx)
    .test_id("ui-gallery-field-responsive");

    ui::v_flex(move |_cx| vec![width_toggle, content])
        .gap(Space::N3)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}
// endregion: example
