pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(|| String::from("000000"));
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::InputOTP::new(value)
        .length(6)
        .group_size(Some(2))
        .aria_invalid(true)
        .test_id_prefix("ui-gallery-input-otp-invalid")
        .refine_layout(max_w_xs)
        .into_element(cx)
        .test_id("ui-gallery-input-otp-invalid")
}
// endregion: example
