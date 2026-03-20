pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Field::new([
            shadcn::FieldLabel::new("مفتاح API").into_element(cx),
            shadcn::Input::new(value)
                .a11y_label("مفتاح API")
                .password()
                .placeholder("sk-...")
                .into_element(cx),
            shadcn::FieldDescription::new("مفتاح API الخاص بك مشفر ومخزن بأمان.").into_element(cx),
        ])
        .refine_layout(max_w_xs)
        .into_element(cx)
    })
    .test_id("ui-gallery-input-rtl")
}
// endregion: example
