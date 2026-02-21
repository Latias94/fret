use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_input_otp(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct InputOtpPageModels {
        simple_value: Option<Model<String>>,
        digits_only_value: Option<Model<String>>,
        separator_value: Option<Model<String>>,
        spacing_value: Option<Model<String>>,
    }

    let (simple_value, digits_only_value, separator_value, spacing_value) =
        cx.with_state(InputOtpPageModels::default, |st| {
            (
                st.simple_value.clone(),
                st.digits_only_value.clone(),
                st.separator_value.clone(),
                st.spacing_value.clone(),
            )
        });

    let (simple_value, digits_only_value, separator_value, spacing_value) = match (
        simple_value,
        digits_only_value,
        separator_value,
        spacing_value,
    ) {
        (Some(simple), Some(digits), Some(separator), Some(spacing)) => {
            (simple, digits, separator, spacing)
        }
        _ => {
            let models = cx.app.models_mut();
            let simple = models.insert(String::new());
            let digits = models.insert(String::new());
            let separator = models.insert(String::from("123456"));
            let spacing = models.insert(String::new());

            cx.with_state(InputOtpPageModels::default, |st| {
                st.simple_value = Some(simple.clone());
                st.digits_only_value = Some(digits.clone());
                st.separator_value = Some(separator.clone());
                st.spacing_value = Some(spacing.clone());
            });

            (simple, digits, separator, spacing)
        }
    };

    let theme = Theme::global(&*cx.app).snapshot();
    let destructive = theme.color_token("destructive");
    let otp_width = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(360.0)));

    let field = |cx: &mut ElementContext<'_, App>,
                 label: &'static str,
                 otp: shadcn::InputOtp,
                 test_id: &'static str| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().min_w_0()),
            |cx| {
                vec![
                    shadcn::Label::new(label).into_element(cx),
                    otp.refine_layout(otp_width.clone())
                        .into_element(cx)
                        .test_id(test_id),
                ]
            },
        )
    };

    let simple = field(
        cx,
        "Simple",
        shadcn::InputOtp::new(simple_value)
            .length(6)
            .numeric_only(true)
            .group_size(Some(3)),
        "ui-gallery-input-otp-simple",
    );

    let digits_only = field(
        cx,
        "Digits Only",
        shadcn::InputOtp::new(digits_only_value)
            .length(6)
            .numeric_only(true)
            .group_size(Some(6)),
        "ui-gallery-input-otp-digits-only",
    );

    let with_separator = field(
        cx,
        "With Separator",
        shadcn::InputOtp::new(separator_value)
            .length(6)
            .numeric_only(true)
            .group_size(Some(2)),
        "ui-gallery-input-otp-with-separator",
    );

    let slot_gap = MetricRef::space(Space::N2).resolve(&theme);
    let with_spacing = field(
        cx,
        "With Spacing",
        shadcn::InputOtp::new(spacing_value)
            .length(6)
            .numeric_only(true)
            .group_size(Some(6))
            .slot_gap_px(slot_gap)
            .slot_corner_mode(shadcn::input_otp::InputOtpSlotCornerMode::All)
            .refine_style(ChromeRefinement::default().border_color(ColorRef::Color(destructive))),
        "ui-gallery-input-otp-with-spacing",
    );

    let demo = doc_layout::wrap_row_snapshot(
        cx,
        &theme,
        Space::N6,
        fret_ui::element::CrossAlign::Start,
        |_cx| vec![simple, digits_only, with_separator, with_spacing],
    )
    .test_id("ui-gallery-input-otp-demo");

    let notes = doc_layout::notes(
        cx,
        [
            "This page aligns with shadcn Input OTP demo: Simple, Digits Only, With Separator, With Spacing.",
            "Fret's `InputOtp` is a higher-level recipe; the spacing demo approximates `aria-invalid` by applying a destructive border to all slots.",
            "API reference: `ecosystem/fret-ui-shadcn/src/input_otp.rs`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Input OTP demo: Simple, Digits Only, With Separator, With Spacing.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .no_shell()
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-input-otp-demo")
                .code(
                    "rust",
                    r#"// Simple
shadcn::InputOtp::new(model).length(6).group_size(Some(3));

// Digits Only
shadcn::InputOtp::new(model).length(6).numeric_only(true).group_size(Some(6));

// With Separator
shadcn::InputOtp::new(model).length(6).group_size(Some(2));

// With Spacing (approx invalid style)
shadcn::InputOtp::new(model)
    .length(6)
    .group_size(Some(6))
    .slot_gap_px(MetricRef::space(Space::N2).resolve(theme.snapshot()))
    .slot_corner_mode(shadcn::input_otp::InputOtpSlotCornerMode::All)
    .refine_style(ChromeRefinement::default().border_color(ColorRef::Color(
        theme.color_token("destructive"),
    )));"#,
                ),
            DocSection::new("Notes", notes).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-input-otp-component")]
}
