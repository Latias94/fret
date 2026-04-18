pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CONTROL_ID: &str = "ui-gallery-input-otp-invalid-control";

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(|| String::from("000000"));
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::Field::new([
        shadcn::FieldLabel::new("Invalid State")
            .for_control(CONTROL_ID)
            .into_element(cx),
        shadcn::FieldDescription::new("Example showing the invalid error state.")
            .for_control(CONTROL_ID)
            .into_element(cx),
        shadcn::InputOTP::new(value)
            .control_id(CONTROL_ID)
            .length(6)
            .test_id_prefix("ui-gallery-input-otp-invalid")
            .into_element_parts(cx, |_cx| {
                vec![
                    shadcn::InputOTPGroup::new([
                        shadcn::InputOTPSlot::new(0).aria_invalid(true),
                        shadcn::InputOTPSlot::new(1).aria_invalid(true),
                    ])
                    .into(),
                    shadcn::InputOTPSeparator::default().into(),
                    shadcn::InputOTPGroup::new([
                        shadcn::InputOTPSlot::new(2).aria_invalid(true),
                        shadcn::InputOTPSlot::new(3).aria_invalid(true),
                    ])
                    .into(),
                    shadcn::InputOTPSeparator::default().into(),
                    shadcn::InputOTPGroup::new([
                        shadcn::InputOTPSlot::new(4).aria_invalid(true),
                        shadcn::InputOTPSlot::new(5).aria_invalid(true),
                    ])
                    .into(),
                ]
            }),
        shadcn::FieldError::new("Invalid code. Please try again.")
            .for_control(CONTROL_ID)
            .into_element(cx),
    ])
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-otp-invalid")
}
// endregion: example
