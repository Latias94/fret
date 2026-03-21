pub const SOURCE: &str = include_str!("sizes.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let size_small = cx.local_model_keyed("size_small", || false);
    let size_default = cx.local_model_keyed("size_default", || false);
    let small_id = ControlId::from("ui-gallery-switch-size-sm");
    let default_id = ControlId::from("ui-gallery-switch-size-default");

    let small = shadcn::Field::new([
        shadcn::Switch::new(size_small)
            .control_id(small_id.clone())
            .a11y_label("Small switch")
            .size(shadcn::SwitchSize::Sm)
            .test_id("ui-gallery-switch-size-small")
            .into_element(cx),
        shadcn::FieldLabel::new("Small")
            .for_control(small_id)
            .test_id("ui-gallery-switch-size-small-label")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .into_element(cx)
    .test_id("ui-gallery-switch-sizes-sm");

    let default = shadcn::Field::new([
        shadcn::Switch::new(size_default)
            .control_id(default_id.clone())
            .a11y_label("Default switch")
            .test_id("ui-gallery-switch-size-default")
            .into_element(cx),
        shadcn::FieldLabel::new("Default")
            .for_control(default_id)
            .test_id("ui-gallery-switch-size-default-label")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .into_element(cx)
    .test_id("ui-gallery-switch-sizes-default");

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            small,
            default,
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(160.0)))
    .into_element(cx)
    .test_id("ui-gallery-switch-sizes")
}

// endregion: example
