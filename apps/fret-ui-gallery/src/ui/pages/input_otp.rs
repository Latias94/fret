use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_input_otp(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct InputOtpPageModels {
        demo_value: Option<Model<String>>,
        pattern_value: Option<Model<String>>,
        separator_value: Option<Model<String>>,
        disabled_value: Option<Model<String>>,
        controlled_value: Option<Model<String>>,
        invalid_value: Option<Model<String>>,
        four_digits_value: Option<Model<String>>,
        alphanumeric_value: Option<Model<String>>,
        form_value: Option<Model<String>>,
        rtl_value: Option<Model<String>>,
    }

    let (
        demo_value,
        pattern_value,
        separator_value,
        disabled_value,
        controlled_value,
        invalid_value,
        four_digits_value,
        alphanumeric_value,
        form_value,
        rtl_value,
    ) = cx.with_state(InputOtpPageModels::default, |st| {
        (
            st.demo_value.clone(),
            st.pattern_value.clone(),
            st.separator_value.clone(),
            st.disabled_value.clone(),
            st.controlled_value.clone(),
            st.invalid_value.clone(),
            st.four_digits_value.clone(),
            st.alphanumeric_value.clone(),
            st.form_value.clone(),
            st.rtl_value.clone(),
        )
    });

    let (
        demo_value,
        pattern_value,
        separator_value,
        disabled_value,
        controlled_value,
        invalid_value,
        four_digits_value,
        alphanumeric_value,
        form_value,
        rtl_value,
    ) = match (
        demo_value,
        pattern_value,
        separator_value,
        disabled_value,
        controlled_value,
        invalid_value,
        four_digits_value,
        alphanumeric_value,
        form_value,
        rtl_value,
    ) {
        (
            Some(demo_value),
            Some(pattern_value),
            Some(separator_value),
            Some(disabled_value),
            Some(controlled_value),
            Some(invalid_value),
            Some(four_digits_value),
            Some(alphanumeric_value),
            Some(form_value),
            Some(rtl_value),
        ) => (
            demo_value,
            pattern_value,
            separator_value,
            disabled_value,
            controlled_value,
            invalid_value,
            four_digits_value,
            alphanumeric_value,
            form_value,
            rtl_value,
        ),
        _ => {
            let demo_value = cx.app.models_mut().insert(String::from("123456"));
            let pattern_value = cx.app.models_mut().insert(String::new());
            let separator_value = cx.app.models_mut().insert(String::new());
            let disabled_value = cx.app.models_mut().insert(String::from("123456"));
            let controlled_value = cx.app.models_mut().insert(String::new());
            let invalid_value = cx.app.models_mut().insert(String::from("000000"));
            let four_digits_value = cx.app.models_mut().insert(String::from("1234"));
            let alphanumeric_value = cx.app.models_mut().insert(String::from("A1B2C3"));
            let form_value = cx.app.models_mut().insert(String::new());
            let rtl_value = cx.app.models_mut().insert(String::new());

            cx.with_state(InputOtpPageModels::default, |st| {
                st.demo_value = Some(demo_value.clone());
                st.pattern_value = Some(pattern_value.clone());
                st.separator_value = Some(separator_value.clone());
                st.disabled_value = Some(disabled_value.clone());
                st.controlled_value = Some(controlled_value.clone());
                st.invalid_value = Some(invalid_value.clone());
                st.four_digits_value = Some(four_digits_value.clone());
                st.alphanumeric_value = Some(alphanumeric_value.clone());
                st.form_value = Some(form_value.clone());
                st.rtl_value = Some(rtl_value.clone());
            });

            (
                demo_value,
                pattern_value,
                separator_value,
                disabled_value,
                controlled_value,
                invalid_value,
                four_digits_value,
                alphanumeric_value,
                form_value,
                rtl_value,
            )
        }
    };

    let (muted, muted_foreground, destructive) = cx.with_theme(|theme| {
        (
            theme.color_token("muted"),
            theme.color_token("muted-foreground"),
            theme.color_token("destructive"),
        )
    });

    let otp_width = LayoutRefinement::default().w_full().max_w(Px(360.0));

    let demo = shadcn::InputOtp::new(demo_value)
        .length(6)
        .refine_layout(otp_width.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-otp-demo");

    let pattern = shadcn::InputOtp::new(pattern_value)
        .length(6)
        .numeric_only(true)
        .group_size(Some(6))
        .refine_layout(otp_width.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-otp-pattern");

    let separator = shadcn::InputOtp::new(separator_value)
        .length(6)
        .group_size(Some(2))
        .refine_layout(otp_width.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-otp-separator");

    let disabled = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(otp_width.clone()),
        |cx| {
            vec![
                shadcn::InputOtp::new(disabled_value)
                    .length(6)
                    .group_size(Some(3))
                    .refine_style(
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(muted))
                            .text_color(ColorRef::Color(muted_foreground)),
                    )
                    .into_element(cx)
                    .test_id("ui-gallery-input-otp-disabled"),
                shadcn::typography::muted(
                    cx,
                    "Current API has no true disabled state yet; this section is visual-only approximation.",
                ),
            ]
        },
    );

    let controlled = {
        let value_text = cx
            .app
            .models()
            .get_cloned(&controlled_value)
            .unwrap_or_default();
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(otp_width.clone()),
            |cx| {
                vec![
                    shadcn::InputOtp::new(controlled_value)
                        .length(6)
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx)
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .test_id("ui-gallery-input-otp-controlled"),
                        ),
                    shadcn::typography::muted(
                        cx,
                        if value_text.is_empty() {
                            Arc::<str>::from("Enter your one-time password.")
                        } else {
                            Arc::<str>::from(format!("You entered: {value_text}"))
                        },
                    ),
                ]
            },
        );
        content
    };

    let invalid = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(otp_width.clone()),
        |cx| {
            vec![
                shadcn::InputOtp::new(invalid_value)
                    .length(6)
                    .group_size(Some(2))
                    .refine_style(
                        ChromeRefinement::default().border_color(ColorRef::Color(destructive)),
                    )
                    .into_element(cx)
                    .test_id("ui-gallery-input-otp-invalid"),
                shadcn::typography::muted(
                    cx,
                    "Invalid is approximated with destructive border style on all slots.",
                ),
            ]
        },
    );

    let four_digits = shadcn::InputOtp::new(four_digits_value)
        .length(4)
        .numeric_only(true)
        .refine_layout(otp_width.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-otp-four-digits");

    let alphanumeric = shadcn::InputOtp::new(alphanumeric_value)
        .length(6)
        .numeric_only(false)
        .group_size(Some(3))
        .refine_layout(otp_width.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-otp-alphanumeric");

    let form = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Verify your login").into_element(cx),
            shadcn::CardDescription::new("Enter the verification code we sent to your email.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    shadcn::InputOtp::new(form_value)
                        .length(6)
                        .group_size(Some(3))
                        .slot_size_px(Px(44.0), Px(48.0))
                        .slot_text_px(Px(18.0))
                        .slot_corner_mode(shadcn::input_otp::InputOtpSlotCornerMode::All)
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx)
                        .test_id("ui-gallery-input-otp-form"),
                    shadcn::Button::new("Verify").into_element(cx),
                ]
            },
        )])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
    .into_element(cx);

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::InputOtp::new(rtl_value)
                    .length(6)
                    .group_size(Some(3))
                    .refine_layout(otp_width.clone())
                    .into_element(cx)
            },
        )
        .test_id("ui-gallery-input-otp-rtl");

        rtl_content
    };

    let notes_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "InputOtp API maps docs `pattern` to `numeric_only`, and `separator` to `group_size`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "`Disabled` and `Invalid` are currently style approximations due missing explicit state APIs.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Each section has stable test_id for future diag scripts.",
                ),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Input OTP docs order: Demo, Pattern, Separator, Disabled, Controlled, Invalid, Four Digits, Alphanumeric, Form, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-input-otp-demo")
                .code("rust", r#"shadcn::InputOtp::new(model).length(6);"#),
            DocSection::new("Pattern", pattern)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-input-otp-pattern")
                .code(
                    "rust",
                    r#"shadcn::InputOtp::new(model)
    .length(6)
    .numeric_only(true)
    .group_size(Some(6));"#,
                ),
            DocSection::new("Separator", separator)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-input-otp-separator")
                .code(
                    "rust",
                    r#"shadcn::InputOtp::new(model).length(6).group_size(Some(2));"#,
                ),
            DocSection::new("Disabled", disabled)
                .max_w(Px(720.0))
                .test_id_prefix("ui-gallery-input-otp-disabled")
                .code(
                    "rust",
                    r#"// Disabled is currently a visual-only approximation (no explicit disabled API yet).
shadcn::InputOtp::new(model)
    .length(6)
    .group_size(Some(3))
    .refine_style(
        ChromeRefinement::default()
            .bg(ColorRef::Color(theme.color_token(\"muted\")))
            .text_color(ColorRef::Color(theme.color_token(\"muted-foreground\"))),
    );"#,
                ),
            DocSection::new("Controlled", controlled)
                .max_w(Px(720.0))
                .test_id_prefix("ui-gallery-input-otp-controlled")
                .code(
                    "rust",
                    r#"// Read `model` to render helper text, but let InputOtp mutate it.
shadcn::InputOtp::new(model)
    .length(6)
    .refine_layout(LayoutRefinement::default().w_full());"#,
                ),
            DocSection::new("Invalid", invalid)
                .max_w(Px(720.0))
                .test_id_prefix("ui-gallery-input-otp-invalid")
                .code(
                    "rust",
                    r#"// Invalid is currently approximated via destructive border styling.
shadcn::InputOtp::new(model)
    .length(6)
    .group_size(Some(2))
    .refine_style(ChromeRefinement::default().border_color(ColorRef::Color(
        theme.color_token(\"destructive\"),
    )));"#,
                ),
            DocSection::new("Four Digits", four_digits)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-input-otp-four-digits")
                .code(
                    "rust",
                    r#"shadcn::InputOtp::new(model)
    .length(4)
    .numeric_only(true);"#,
                ),
            DocSection::new("Alphanumeric", alphanumeric)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-input-otp-alphanumeric")
                .code(
                    "rust",
                    r#"shadcn::InputOtp::new(model)
    .length(6)
    .numeric_only(false)
    .group_size(Some(3));"#,
                ),
            DocSection::new("Form", form)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-input-otp-form")
                .code(
                    "rust",
                    r#"shadcn::InputOtp::new(model)
    .slot_size_px(Px(44.0), Px(48.0))
    .slot_text_px(Px(18.0))
    .slot_corner_mode(shadcn::input_otp::InputOtpSlotCornerMode::All);"#,
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-input-otp-rtl")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::InputOtp::new(model)
            .length(6)
            .group_size(Some(3))
            .refine_layout(LayoutRefinement::default().max_w(Px(220.0)));
    },
);"#,
                ),
            DocSection::new("Notes", notes_panel_body).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-input-otp-component")]
}
