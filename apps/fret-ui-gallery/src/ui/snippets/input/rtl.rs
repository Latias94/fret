pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let api_key_id = "ui-gallery-input-rtl-api-key";

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Field::new([
            shadcn::FieldLabel::new("مفتاح API")
                .for_control(api_key_id)
                .into_element(cx),
            shadcn::Input::new(value)
                .control_id(api_key_id)
                .password()
                .placeholder("sk-...")
                .into_element(cx),
            shadcn::FieldDescription::new("مفتاح API الخاص بك مشفر ومخزن بأمان.")
                .for_control(api_key_id)
                .into_element(cx),
        ])
        .refine_layout(max_w_xs)
        .into_element(cx)
    })
    .test_id("ui-gallery-input-rtl")
}
// endregion: example
