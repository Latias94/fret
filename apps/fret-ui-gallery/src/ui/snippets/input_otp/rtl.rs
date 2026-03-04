pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    value: Option<Model<String>>,
}

fn value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("123456"));
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = value_model(cx);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::with_direction_provider(cx, shadcn::LayoutDirection::Rtl, |cx| {
        shadcn::Field::new([
            shadcn::FieldLabel::new("رمز التحقق").into_element(cx),
            shadcn::InputOTP::new(value)
                .length(6)
                .test_id_prefix("ui-gallery-input-otp-rtl")
                .refine_layout(max_w_xs.clone())
                .into_element_parts(cx, |_cx| {
                    vec![shadcn::InputOtpPart::group(shadcn::InputOTPGroup::new([
                        shadcn::InputOTPSlot::new(0),
                        shadcn::InputOTPSlot::new(1),
                        shadcn::InputOTPSlot::new(2),
                        shadcn::InputOTPSlot::new(3),
                        shadcn::InputOTPSlot::new(4),
                        shadcn::InputOTPSlot::new(5),
                    ]))]
                }),
            shadcn::FieldDescription::new("أدخل رمز التحقق لمتابعة تسجيل الدخول.").into_element(cx),
        ])
        .refine_layout(max_w_xs)
        .into_element(cx)
    })
    .test_id("ui-gallery-input-otp-rtl")
}
// endregion: example
