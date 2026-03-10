pub const SOURCE: &str = include_str!("form.rs");

// region: example
use fret_core::Px;
use fret_icons_lucide::generated_ids::lucide::REFRESH_CW;
use fret_ui_shadcn::{self as shadcn, prelude::*};

const CONTROL_ID: &str = "ui-gallery-input-otp-form-verification";

#[derive(Default, Clone)]
struct Models {
    otp: Option<Model<String>>,
}

fn ensure_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    if let Some(model) = state.otp {
        return model;
    }

    let model = cx.app.models_mut().insert(String::new());
    cx.with_state(Models::default, |st| st.otp = Some(model.clone()));
    model
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let otp = ensure_model(cx);
    let card_layout = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(448.0));

    let label_row = ui::h_flex(|cx| {
        vec![
            shadcn::FieldLabel::new("Verification code")
                .for_control(CONTROL_ID)
                .into_element(cx),
            shadcn::Button::new("Resend Code")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Xs)
                .leading_icon(REFRESH_CW)
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .justify_between()
    .items_center()
    .gap(Space::N2)
    .into_element(cx);

    let otp_field = shadcn::Field::new([
        label_row,
        shadcn::InputOTP::new(otp)
            .control_id(CONTROL_ID)
            .aria_required(true)
            .slot_size_px(Px(44.0), Px(48.0))
            .slot_text_px(Px(20.0))
            .slot_line_height_px(Px(28.0))
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
            }),
        shadcn::FieldDescription::new("I no longer have access to this email address.")
            .for_control(CONTROL_ID)
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx);

    let footer = shadcn::CardFooter::new([
        shadcn::Button::new("Verify")
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        shadcn::FieldDescription::new("Having trouble signing in? Contact support.")
            .into_element(cx),
    ])
    .direction(shadcn::CardFooterDirection::Column)
    .gap(Space::N3)
    .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Verify your login").into_element(cx),
            shadcn::CardDescription::new(
                "Enter the verification code we sent to your email address: m@example.com.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([otp_field]).into_element(cx),
        footer,
    ])
    .refine_layout(card_layout)
    .into_element(cx)
    .test_id("ui-gallery-input-otp-form")
}
// endregion: example
