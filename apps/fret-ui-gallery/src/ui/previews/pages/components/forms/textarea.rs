use super::super::super::super::super::*;

pub(in crate::ui) fn preview_textarea(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    #[derive(Default, Clone)]
    struct TextareaModels {
        invalid: Option<Model<String>>,
        labeled: Option<Model<String>>,
        labeled_description: Option<Model<String>>,
        disabled: Option<Model<String>>,
        extras_button: Option<Model<String>>,
        rtl: Option<Model<String>>,
    }

    let state = cx.with_state(TextareaModels::default, |st| st.clone());
    let (
        invalid_value,
        labeled_value,
        labeled_description_value,
        disabled_value,
        extras_button_value,
        rtl_value,
    ) = match (
        state.invalid,
        state.labeled,
        state.labeled_description,
        state.disabled,
        state.extras_button,
        state.rtl,
    ) {
        (
            Some(invalid_value),
            Some(labeled_value),
            Some(labeled_description_value),
            Some(disabled_value),
            Some(extras_button_value),
            Some(rtl_value),
        ) => (
            invalid_value,
            labeled_value,
            labeled_description_value,
            disabled_value,
            extras_button_value,
            rtl_value,
        ),
        _ => {
            let invalid_value = cx.app.models_mut().insert(String::new());
            let labeled_value = cx.app.models_mut().insert(String::new());
            let labeled_description_value = cx.app.models_mut().insert(String::new());
            let disabled_value = cx.app.models_mut().insert(String::new());
            let extras_button_value = cx.app.models_mut().insert(String::new());
            let rtl_value = cx.app.models_mut().insert(String::new());
            cx.with_state(TextareaModels::default, |st| {
                st.invalid = Some(invalid_value.clone());
                st.labeled = Some(labeled_value.clone());
                st.labeled_description = Some(labeled_description_value.clone());
                st.disabled = Some(disabled_value.clone());
                st.extras_button = Some(extras_button_value.clone());
                st.rtl = Some(rtl_value.clone());
            });
            (
                invalid_value,
                labeled_value,
                labeled_description_value,
                disabled_value,
                extras_button_value,
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
        .min_height(Px(96.0))
        .refine_layout(area_layout.clone())
        .into_element(cx)
        .test_id("ui-gallery-textarea-demo");
    let demo = centered(cx, demo_inner);

    let invalid_inner = shadcn::Textarea::new(invalid_value)
        .a11y_label("Message")
        .placeholder("Type your message here.")
        .aria_invalid(true)
        .min_height(Px(96.0))
        .refine_layout(area_layout.clone())
        .into_element(cx)
        .test_id("ui-gallery-textarea-invalid");
    let invalid = centered(cx, invalid_inner);

    let labeled_inner = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |cx| {
            vec![
                shadcn::Label::new("Label").into_element(cx),
                shadcn::Textarea::new(labeled_value)
                    .a11y_label("Label")
                    .placeholder("Type your message here.")
                    .min_height(Px(144.0))
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-textarea-labeled");
    let labeled = centered(cx, labeled_inner);

    let labeled_description_inner = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |cx| {
            vec![
                shadcn::Label::new("With label and description").into_element(cx),
                shadcn::Textarea::new(labeled_description_value)
                    .a11y_label("With label and description")
                    .placeholder("Type your message here.")
                    .min_height(Px(144.0))
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                shadcn::typography::muted(cx, "Type your message and press enter to send."),
            ]
        },
    )
    .test_id("ui-gallery-textarea-labeled-description");
    let labeled_description = centered(cx, labeled_description_inner);

    let disabled_inner = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |cx| {
            vec![
                shadcn::Label::new("Disabled").into_element(cx),
                shadcn::Textarea::new(disabled_value)
                    .a11y_label("Disabled")
                    .placeholder("Type your message here.")
                    .disabled(true)
                    .min_height(Px(96.0))
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-textarea-disabled");
    let disabled = centered(cx, disabled_inner);

    let extras_button_inner = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |cx| {
            vec![
                shadcn::Textarea::new(extras_button_value)
                    .a11y_label("Send message")
                    .placeholder("Type your message here.")
                    .min_height(Px(96.0))
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                shadcn::Button::new("Send message").into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-textarea-extras-button");
    let extras_button = centered(cx, extras_button_inner);

    let rtl_inner = doc_layout::rtl(cx, |cx| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    shadcn::Label::new("Label").into_element(cx),
                    shadcn::Textarea::new(rtl_value)
                        .a11y_label("Label")
                        .placeholder("Type your message here.")
                        .min_height(Px(144.0))
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                ]
            },
        )
    })
    .test_id("ui-gallery-textarea-rtl");
    let rtl = centered(cx, rtl_inner);

    let extras = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Extras are Fret-specific demos and regression gates (not part of upstream shadcn TextareaDemo).",
                ),
                extras_button,
            ]
        },
    )
    .test_id("ui-gallery-textarea-extras");

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Textarea demo (new-york-v4).",
            "Placeholder text is rendered when the model is empty.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Textarea demo order: Basic, Invalid, With label, With label + description, Disabled. Extras include RTL + a button composition.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-textarea-demo")
                .code(
                    "rust",
                    r#"shadcn::Textarea::new(model)
    .a11y_label("Message")
    .placeholder("Type your message here.")
    .min_height(Px(96.0))
    .into_element(cx);"#,
                ),
            DocSection::new("Invalid", invalid)
                .test_id_prefix("ui-gallery-textarea-invalid")
                .code(
                    "rust",
                    r#"shadcn::Textarea::new(model)
    .a11y_label("Message")
    .placeholder("Type your message here.")
    .aria_invalid(true)
    .into_element(cx);"#,
                ),
            DocSection::new("With label", labeled)
                .test_id_prefix("ui-gallery-textarea-labeled")
                .code(
                    "rust",
                    r#"stack::vstack(cx, props, |cx| vec![
    shadcn::Label::new("Label").into_element(cx),
    shadcn::Textarea::new(model)
        .a11y_label("Label")
        .placeholder("Type your message here.")
        .into_element(cx),
]);"#,
                ),
            DocSection::new("With label and description", labeled_description)
                .test_id_prefix("ui-gallery-textarea-labeled-description")
                .code(
                    "rust",
                    r#"stack::vstack(cx, props, |cx| vec![
    shadcn::Label::new("With label and description").into_element(cx),
    shadcn::Textarea::new(model)
        .a11y_label("With label and description")
        .placeholder("Type your message here.")
        .into_element(cx),
    shadcn::typography::muted(cx, "Type your message and press enter to send."),
]);"#,
                ),
            DocSection::new("Disabled", disabled)
                .test_id_prefix("ui-gallery-textarea-disabled")
                .code(
                    "rust",
                    r#"shadcn::Textarea::new(model)
    .a11y_label("Disabled")
    .placeholder("Type your message here.")
    .disabled(true)
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-textarea-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Textarea::new(model)
        .a11y_label("Label")
        .placeholder("Type your message here.")
        .into_element(cx)
 });"#,
                ),
            DocSection::new("Extras", extras)
                .no_shell()
                .test_id_prefix("ui-gallery-textarea-extras"),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-textarea-notes"),
        ],
    );

    vec![body]
}
