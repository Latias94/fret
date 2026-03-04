pub const SOURCE: &str = include_str!("controlled.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

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
    let current = cx.watch_model(&value).cloned().unwrap_or_default();
    let theme = Theme::global(&*cx.app).snapshot();
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            let otp = shadcn::InputOTP::new(value)
                .length(6)
                .test_id_prefix("ui-gallery-input-otp-controlled")
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
                });

            let message: Arc<str> = if current.is_empty() {
                Arc::from("Enter your one-time password.")
            } else {
                Arc::from(format!("You entered: {current}"))
            };

            vec![
                otp.test_id("ui-gallery-input-otp-controlled"),
                ui::label(cx, message)
                    .text_size_px(Px(14.0))
                    .text_color(shadcn::ColorRef::Color(
                        theme.color_token("muted-foreground"),
                    ))
                    .into_element(cx),
            ]
        },
    )
}
// endregion: example
