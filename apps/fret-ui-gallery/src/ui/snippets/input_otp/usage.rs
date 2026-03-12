pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);

    shadcn::InputOTP::new(value)
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::InputOtpPart::group(shadcn::InputOTPGroup::new([
                    shadcn::InputOTPSlot::new(0),
                    shadcn::InputOTPSlot::new(1),
                    shadcn::InputOTPSlot::new(2),
                ])),
                shadcn::InputOtpPart::separator(shadcn::InputOtpSeparator),
                shadcn::InputOtpPart::group(shadcn::InputOTPGroup::new([
                    shadcn::InputOTPSlot::new(3),
                    shadcn::InputOTPSlot::new(4),
                    shadcn::InputOTPSlot::new(5),
                ])),
            ]
        })
        .test_id("ui-gallery-input-otp-usage")
}
// endregion: example
