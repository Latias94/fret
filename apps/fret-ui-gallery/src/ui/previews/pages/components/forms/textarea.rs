use super::super::super::super::super::*;

pub(in crate::ui) fn preview_textarea(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct TextareaModels {
        field: Option<Model<String>>,
        disabled: Option<Model<String>>,
        invalid: Option<Model<String>>,
        button: Option<Model<String>>,
        rtl: Option<Model<String>>,
    }

    let state = cx.with_state(TextareaModels::default, |st| st.clone());
    let (field_value, disabled_value, invalid_value, button_value, rtl_value) = match (
        state.field,
        state.disabled,
        state.invalid,
        state.button,
        state.rtl,
    ) {
        (
            Some(field_value),
            Some(disabled_value),
            Some(invalid_value),
            Some(button_value),
            Some(rtl_value),
        ) => (
            field_value,
            disabled_value,
            invalid_value,
            button_value,
            rtl_value,
        ),
        _ => {
            let field_value = cx.app.models_mut().insert(String::new());
            let disabled_value = cx.app.models_mut().insert(String::new());
            let invalid_value = cx.app.models_mut().insert(String::new());
            let button_value = cx.app.models_mut().insert(String::new());
            let rtl_value = cx.app.models_mut().insert(String::new());
            cx.with_state(TextareaModels::default, |st| {
                st.field = Some(field_value.clone());
                st.disabled = Some(disabled_value.clone());
                st.invalid = Some(invalid_value.clone());
                st.button = Some(button_value.clone());
                st.rtl = Some(rtl_value.clone());
            });
            (
                field_value,
                disabled_value,
                invalid_value,
                button_value,
                rtl_value,
            )
        }
    };

    let destructive = cx.with_theme(|theme| theme.color_required("destructive"));

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(420.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let area_layout = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let demo = {
        let area = shadcn::Textarea::new(value)
            .a11y_label("Message")
            .min_height(Px(96.0))
            .refine_layout(area_layout.clone())
            .into_element(cx)
            .test_id("ui-gallery-textarea-demo");

        let body = centered(cx, area);
        section(cx, "Demo", body)
    };

    let field = {
        let field = shadcn::Field::new([
            shadcn::FieldLabel::new("Message").into_element(cx),
            shadcn::FieldDescription::new("Enter your message below.").into_element(cx),
            shadcn::Textarea::new(field_value)
                .a11y_label("Message field")
                .min_height(Px(96.0))
                .refine_layout(area_layout.clone())
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-textarea-field");

        let body = centered(cx, field);
        section(cx, "Field", body)
    };

    let disabled = {
        let field = shadcn::Field::new([
            shadcn::FieldLabel::new("Message").into_element(cx),
            shadcn::Textarea::new(disabled_value)
                .a11y_label("Disabled message")
                .disabled(true)
                .min_height(Px(96.0))
                .refine_layout(area_layout.clone())
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-textarea-disabled");

        let body = centered(cx, field);
        section(cx, "Disabled", body)
    };

    let invalid = {
        let field = shadcn::Field::new([
            ui::label(cx, "Message")
                .text_color(ColorRef::Color(destructive))
                .into_element(cx),
            shadcn::Textarea::new(invalid_value)
                .a11y_label("Invalid message")
                .aria_invalid(true)
                .min_height(Px(96.0))
                .refine_layout(area_layout.clone())
                .into_element(cx),
            shadcn::FieldDescription::new("Please enter a valid message.").into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-textarea-invalid");

        let body = centered(cx, field);
        section(cx, "Invalid", body)
    };

    let button = {
        let group = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    shadcn::Textarea::new(button_value)
                        .a11y_label("Send message")
                        .min_height(Px(96.0))
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Button::new("Send message").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-textarea-button");

        let body = centered(cx, group);
        section(cx, "Button", body)
    };

    let rtl = {
        let rtl_field = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Field::new([
                    shadcn::FieldLabel::new("Feedback").into_element(cx),
                    shadcn::Textarea::new(rtl_value)
                        .a11y_label("Feedback")
                        .min_height(Px(96.0))
                        .refine_layout(area_layout.clone())
                        .into_element(cx),
                    shadcn::FieldDescription::new("Share your thoughts about our service.")
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-textarea-rtl");

        let rtl_shell = shell(cx, rtl_field);
        let body = centered(cx, rtl_shell);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Displays a form textarea or a component that looks like a textarea."),
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![demo, field, disabled, invalid, button, rtl],
        ),
    ]
}
