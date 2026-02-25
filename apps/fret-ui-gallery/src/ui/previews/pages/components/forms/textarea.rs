use super::super::super::super::super::*;

pub(in crate::ui) fn preview_textarea(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    #[derive(Default, Clone)]
    struct TextareaModels {
        field: Option<Model<String>>,
        invalid: Option<Model<String>>,
        disabled: Option<Model<String>>,
        button: Option<Model<String>>,
        rtl: Option<Model<String>>,
    }

    let state = cx.with_state(TextareaModels::default, |st| st.clone());
    let (field_value, invalid_value, disabled_value, button_value, rtl_value) = match (
        state.field,
        state.invalid,
        state.disabled,
        state.button,
        state.rtl,
    ) {
        (
            Some(field_value),
            Some(invalid_value),
            Some(disabled_value),
            Some(button_value),
            Some(rtl_value),
        ) => (
            field_value,
            invalid_value,
            disabled_value,
            button_value,
            rtl_value,
        ),
        _ => {
            let field_value = cx.app.models_mut().insert(String::new());
            let invalid_value = cx.app.models_mut().insert(String::new());
            let disabled_value = cx.app.models_mut().insert(String::new());
            let button_value = cx.app.models_mut().insert(String::new());
            let rtl_value = cx.app.models_mut().insert(String::new());
            cx.with_state(TextareaModels::default, |st| {
                st.field = Some(field_value.clone());
                st.invalid = Some(invalid_value.clone());
                st.disabled = Some(disabled_value.clone());
                st.button = Some(button_value.clone());
                st.rtl = Some(rtl_value.clone());
            });
            (
                field_value,
                invalid_value,
                disabled_value,
                button_value,
                rtl_value,
            )
        }
    };

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let area_layout = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let demo_inner = shadcn::Textarea::new(value)
        .a11y_label("Message")
        .placeholder("Type your message here.")
        .refine_layout(area_layout.clone())
        .into_element(cx)
        .test_id("ui-gallery-textarea-demo");
    let demo = centered(cx, demo_inner);

    let field_inner = shadcn::Field::new([
        shadcn::FieldLabel::new("Message").into_element(cx),
        shadcn::FieldDescription::new("Enter your message below.").into_element(cx),
        shadcn::Textarea::new(field_value)
            .a11y_label("Message")
            .placeholder("Type your message here.")
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
    ])
    .refine_layout(area_layout.clone())
    .into_element(cx)
    .test_id("ui-gallery-textarea-field");
    let field = centered(cx, field_inner);

    let disabled_inner = shadcn::Field::new([
        shadcn::FieldLabel::new("Message").into_element(cx),
        shadcn::Textarea::new(disabled_value)
            .a11y_label("Message")
            .placeholder("Type your message here.")
            .disabled(true)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
    ])
    .disabled(true)
    .refine_layout(area_layout.clone())
    .into_element(cx)
    .test_id("ui-gallery-textarea-disabled");
    let disabled = centered(cx, disabled_inner);

    let invalid_inner = shadcn::Field::new([
        shadcn::FieldLabel::new("Message").into_element(cx),
        shadcn::Textarea::new(invalid_value)
            .a11y_label("Message")
            .placeholder("Type your message here.")
            .aria_invalid(true)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        shadcn::FieldDescription::new("Please enter a valid message.").into_element(cx),
    ])
    .invalid(true)
    .refine_layout(area_layout.clone())
    .into_element(cx)
    .test_id("ui-gallery-textarea-invalid");
    let invalid = centered(cx, invalid_inner);

    let button_inner = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |cx| {
            vec![
                shadcn::Textarea::new(button_value)
                    .a11y_label("Send message")
                    .placeholder("Type your message here.")
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                shadcn::Button::new("Send message").into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-textarea-button");
    let button = centered(cx, button_inner);

    let rtl_inner = doc_layout::rtl(cx, |cx| {
        shadcn::Field::new([
            shadcn::FieldLabel::new("التعليقات").into_element(cx),
            shadcn::Textarea::new(rtl_value)
                .a11y_label("Feedback")
                .placeholder("تعليقاتك تساعدنا على التحسين...")
                .min_height(Px(96.0))
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
            shadcn::FieldDescription::new("شاركنا أفكارك حول خدمتنا.").into_element(cx),
        ])
        .refine_layout(area_layout.clone())
        .into_element(cx)
    })
    .test_id("ui-gallery-textarea-rtl");
    let rtl = centered(cx, rtl_inner);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn/ui v4 Textarea docs (radix/base nova).",
            "Placeholder text is rendered when the model is empty.",
            "Drag the bottom-right corner to resize the textarea height.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the shadcn/ui Textarea docs examples: Demo, Field, Disabled, Invalid, Button, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-textarea-demo")
                .code(
                    "rust",
                    r#"shadcn::Textarea::new(model)
    .a11y_label("Message")
    .placeholder("Type your message here.")
    .into_element(cx);"#,
                ),
            DocSection::new("Field", field)
                .test_id_prefix("ui-gallery-textarea-field")
                .code(
                    "rust",
                    r#"shadcn::Field::new([
    shadcn::FieldLabel::new("Message").into_element(cx),
    shadcn::FieldDescription::new("Enter your message below.").into_element(cx),
    shadcn::Textarea::new(model)
        .a11y_label("Message")
        .placeholder("Type your message here.")
        .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Disabled", disabled)
                .test_id_prefix("ui-gallery-textarea-disabled")
                .code(
                    "rust",
                    r#"shadcn::Field::new([
    shadcn::FieldLabel::new("Message").into_element(cx),
    shadcn::Textarea::new(model)
        .a11y_label("Message")
        .placeholder("Type your message here.")
        .disabled(true)
        .into_element(cx),
])
.disabled(true)
.into_element(cx);"#,
                ),
            DocSection::new("Invalid", invalid)
                .test_id_prefix("ui-gallery-textarea-invalid")
                .code(
                    "rust",
                    r#"shadcn::Field::new([
    shadcn::FieldLabel::new("Message").into_element(cx),
    shadcn::Textarea::new(model)
        .a11y_label("Message")
        .placeholder("Type your message here.")
        .aria_invalid(true)
        .into_element(cx),
    shadcn::FieldDescription::new("Please enter a valid message.").into_element(cx),
])
.invalid(true)
.into_element(cx);"#,
                ),
            DocSection::new("Button", button)
                .test_id_prefix("ui-gallery-textarea-button")
                .code(
                    "rust",
                    r#"stack::vstack(
    cx,
    stack::VStackProps::default().gap(Space::N2),
    |cx| vec![
        shadcn::Textarea::new(model)
            .a11y_label("Send message")
            .placeholder("Type your message here.")
            .into_element(cx),
        shadcn::Button::new("Send message").into_element(cx),
    ],
);"#,
                ),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-textarea-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Field::new([
        shadcn::FieldLabel::new("التعليقات").into_element(cx),
        shadcn::Textarea::new(model)
            .a11y_label("Feedback")
            .placeholder("تعليقاتك تساعدنا على التحسين...")
            .into_element(cx),
        shadcn::FieldDescription::new("شاركنا أفكارك حول خدمتنا.").into_element(cx),
    ])
    .into_element(cx)
});"#,
                ),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-textarea-notes"),
        ],
    );

    vec![body]
}
