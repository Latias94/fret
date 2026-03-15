pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    shadcn::InputOTP::new(value)
        .group_size(Some(3))
        .into_element(cx)
        .test_id("ui-gallery-input-otp-usage")
}
// endregion: example
