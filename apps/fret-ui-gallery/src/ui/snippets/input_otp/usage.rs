pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    value: Option<Model<String>>,
}

fn value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = value_model(cx);

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
