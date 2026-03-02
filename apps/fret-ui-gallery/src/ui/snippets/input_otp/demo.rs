// region: example
use fret_ui::Theme;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    simple_value: Option<Model<String>>,
    digits_only_value: Option<Model<String>>,
    separator_value: Option<Model<String>>,
    spacing_value: Option<Model<String>>,
    invalid_value: Option<Model<String>>,
}

fn ensure_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (
    Model<String>,
    Model<String>,
    Model<String>,
    Model<String>,
    Model<String>,
) {
    let state = cx.with_state(Models::default, |st| st.clone());

    match (
        state.simple_value,
        state.digits_only_value,
        state.separator_value,
        state.spacing_value,
        state.invalid_value,
    ) {
        (Some(simple), Some(digits), Some(separator), Some(spacing), Some(invalid)) => {
            (simple, digits, separator, spacing, invalid)
        }
        _ => {
            let models = cx.app.models_mut();
            let simple = models.insert(String::new());
            let digits = models.insert(String::new());
            let separator = models.insert(String::from("123456"));
            let spacing = models.insert(String::new());
            let invalid = models.insert(String::new());

            cx.with_state(Models::default, |st| {
                st.simple_value = Some(simple.clone());
                st.digits_only_value = Some(digits.clone());
                st.separator_value = Some(separator.clone());
                st.spacing_value = Some(spacing.clone());
                st.invalid_value = Some(invalid.clone());
            });

            (simple, digits, separator, spacing, invalid)
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (simple_value, digits_only_value, separator_value, spacing_value, invalid_value) =
        ensure_models(cx);

    let theme = Theme::global(&*cx.app).snapshot();
    let otp_width = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(360.0)));

    let field = |cx: &mut ElementContext<'_, H>,
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
            .group_size(Some(3))
            .test_id_prefix("ui-gallery-input-otp-simple"),
        "ui-gallery-input-otp-simple",
    );

    let digits_only = field(
        cx,
        "Digits Only",
        shadcn::InputOtp::new(digits_only_value)
            .length(6)
            .numeric_only(true)
            .group_size(Some(6))
            .test_id_prefix("ui-gallery-input-otp-digits-only"),
        "ui-gallery-input-otp-digits-only",
    );

    let with_separator = field(
        cx,
        "With Separator",
        shadcn::InputOtp::new(separator_value)
            .length(6)
            .numeric_only(true)
            .group_size(Some(2))
            .test_id_prefix("ui-gallery-input-otp-with-separator"),
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
            .test_id_prefix("ui-gallery-input-otp-with-spacing"),
        "ui-gallery-input-otp-with-spacing",
    );

    let invalid = field(
        cx,
        "Invalid",
        shadcn::InputOtp::new(invalid_value)
            .length(6)
            .numeric_only(true)
            .group_size(Some(3))
            .aria_invalid(true)
            .test_id_prefix("ui-gallery-input-otp-invalid"),
        "ui-gallery-input-otp-invalid",
    );

    fret_ui_kit::ui::h_flex(cx, |_cx| {
        vec![simple, digits_only, with_separator, with_spacing, invalid]
    })
    .gap(Space::N6)
    .wrap()
    .w_full()
    .items_start()
    .into_element(cx)
    .test_id("ui-gallery-input-otp-demo")
}
// endregion: example

