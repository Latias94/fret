pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::InputOTP::new(value)
        .length(6)
        .test_id_prefix("ui-gallery-input-otp-simple")
        .refine_layout(max_w_xs)
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
        .test_id("ui-gallery-input-otp-demo")
}
// endregion: example
