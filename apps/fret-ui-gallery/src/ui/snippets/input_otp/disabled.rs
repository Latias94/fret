pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(|| String::from("123456"));
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::InputOTP::new(value)
        .length(6)
        .disabled(true)
        .refine_layout(max_w_xs)
        .test_id_prefix("ui-gallery-input-otp-disabled")
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::InputOTPGroup::new([
                    shadcn::InputOTPSlot::new(0),
                    shadcn::InputOTPSlot::new(1),
                    shadcn::InputOTPSlot::new(2),
                ])
                .into(),
                shadcn::InputOTPSeparator::default().into(),
                shadcn::InputOTPGroup::new([
                    shadcn::InputOTPSlot::new(3),
                    shadcn::InputOTPSlot::new(4),
                    shadcn::InputOTPSlot::new(5),
                ])
                .into(),
            ]
        })
        .test_id("ui-gallery-input-otp-disabled")
}
// endregion: example
