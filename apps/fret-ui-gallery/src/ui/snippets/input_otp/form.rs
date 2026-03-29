pub const SOURCE: &str = include_str!("form.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons_lucide::generated_ids::lucide::REFRESH_CW;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CONTROL_ID: &str = "ui-gallery-input-otp-form-verification";

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let otp = cx.local_model(String::new);
    let card_layout = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(448.0));

    let label_row = ui::h_flex(|cx| {
        vec![
            shadcn::FieldLabel::new("Verification code")
                .for_control(CONTROL_ID)
                .test_id("ui-gallery-input-otp-form-label")
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
            .required(true)
            .test_id_prefix("ui-gallery-input-otp-form")
            .into_element_parts(cx, |_cx| {
                vec![
                    shadcn::InputOTPGroup::new([
                        shadcn::InputOTPSlot::new(0),
                        shadcn::InputOTPSlot::new(1),
                        shadcn::InputOTPSlot::new(2),
                    ])
                    .slot_size_px(Px(44.0), Px(48.0))
                    .slot_text_px(Px(20.0))
                    .slot_line_height_px(Px(28.0))
                    .into(),
                    shadcn::InputOTPSeparator::default().into(),
                    shadcn::InputOTPGroup::new([
                        shadcn::InputOTPSlot::new(3),
                        shadcn::InputOTPSlot::new(4),
                        shadcn::InputOTPSlot::new(5),
                    ])
                    .slot_size_px(Px(44.0), Px(48.0))
                    .slot_text_px(Px(20.0))
                    .slot_line_height_px(Px(28.0))
                    .into(),
                ]
            }),
        shadcn::FieldDescription::new("I no longer have access to this email address.")
            .for_control(CONTROL_ID)
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx);

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Verify your login"),
                    shadcn::card_description(
                        "Enter the verification code we sent to your email address: m@example.com.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; otp_field]),
            shadcn::card_footer(|cx| {
                ui::children![
                    cx;
                    shadcn::Button::new("Verify")
                        .refine_layout(LayoutRefinement::default().w_full())
                        .ui(),
                    shadcn::FieldDescription::new(
                        "Having trouble signing in? Contact support.",
                    ),
                ]
            })
            .direction(shadcn::CardFooterDirection::Column)
            .gap(Space::N3),
        ]
    })
    .refine_layout(card_layout)
    .into_element(cx)
    .test_id("ui-gallery-input-otp-form")
}
// endregion: example
