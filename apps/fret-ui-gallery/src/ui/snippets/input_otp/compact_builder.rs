pub const SOURCE: &str = include_str!("compact_builder.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::InputOTP::new(value)
        .length(6)
        .group_size(Some(3))
        .test_id_prefix("ui-gallery-input-otp-compact-builder")
        .refine_layout(max_w_xs)
        .into_element(cx)
        .test_id("ui-gallery-input-otp-compact-builder")
}
// endregion: example
