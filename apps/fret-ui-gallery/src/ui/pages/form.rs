use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_forms(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    let demo = {
        let content = shadcn::FieldSet::new([
            shadcn::FieldLegend::new("Contact").into_element(cx),
            shadcn::FieldDescription::new(
                "Model-bound controls keep values while you stay in the window.",
            )
            .into_element(cx),
            shadcn::FieldGroup::new([
                shadcn::Field::new([
                    shadcn::FieldLabel::new("Email").into_element(cx),
                    shadcn::Input::new(text_input.clone())
                        .a11y_label("Email")
                        .placeholder("name@example.com")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::FieldLabel::new("Message").into_element(cx),
                    shadcn::Textarea::new(text_area.clone())
                        .a11y_label("Message")
                        .refine_layout(LayoutRefinement::default().h_px(Px(96.0)))
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::Checkbox::new(checkbox.clone())
                        .control_id("ui-gallery-form-checkbox-terms")
                        .a11y_label("Accept terms")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Accept terms")
                        .for_control("ui-gallery-form-checkbox-terms")
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Enable feature")
                            .for_control("ui-gallery-form-switch-feature")
                            .into_element(cx),
                        shadcn::FieldDescription::new(
                            "This toggles an optional feature for the current session.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Switch::new(switch.clone())
                        .control_id("ui-gallery-form-switch-feature")
                        .a11y_label("Enable feature")
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .test_id("ui-gallery-form-demo");

        content
    };

    let input = {
        let content = shadcn::Input::new(text_input.clone())
            .a11y_label("Email")
            .placeholder("name@example.com")
            .refine_layout(max_w_md.clone())
            .into_element(cx)
            .test_id("ui-gallery-form-input");

        content
    };

    let textarea = {
        let content = shadcn::Textarea::new(text_area.clone())
            .a11y_label("Message")
            .refine_layout(
                max_w_md
                    .clone()
                    .merge(LayoutRefinement::default().h_px(Px(96.0))),
            )
            .into_element(cx)
            .test_id("ui-gallery-form-textarea");

        content
    };

    let controls = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .layout(max_w_md.clone())
                .items_start(),
            |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Checkbox::new(checkbox.clone())
                                    .a11y_label("Accept terms")
                                    .into_element(cx),
                                shadcn::Label::new("Accept terms").into_element(cx),
                            ]
                        },
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Switch::new(switch.clone())
                                    .a11y_label("Enable feature")
                                    .into_element(cx),
                                shadcn::Label::new("Enable feature").into_element(cx),
                            ]
                        },
                    ),
                ]
            },
        )
        .test_id("ui-gallery-form-controls");

        content
    };

    let fieldset = {
        let content = shadcn::FieldSet::new([
            shadcn::FieldLegend::new("Profile").into_element(cx),
            shadcn::FieldDescription::new("Group related fields to keep structure explicit.")
                .into_element(cx),
            shadcn::FieldGroup::new([
                shadcn::Field::new([
                    shadcn::FieldLabel::new("Display name").into_element(cx),
                    shadcn::Input::new(text_input.clone())
                        .placeholder("Evil Rabbit")
                        .a11y_label("Display name")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::FieldLabel::new("Bio").into_element(cx),
                    shadcn::Textarea::new(text_area.clone())
                        .a11y_label("Bio")
                        .refine_layout(LayoutRefinement::default().h_px(Px(88.0)))
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::Button::new("Submit").into_element(cx),
                    shadcn::Button::new("Cancel")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .test_id("ui-gallery-form-fieldset");

        content
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::FieldSet::new([
                    shadcn::FieldLegend::new("?????").into_element(cx),
                    shadcn::FieldDescription::new("??? ????? ??????? ????? RTL ?? ?????? ????????")
                        .into_element(cx),
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("?????? ??????????").into_element(cx),
                        shadcn::Input::new(text_input.clone())
                            .a11y_label("?????? ??????????")
                            .placeholder("name@example.com")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("????? ??????")
                            .for_control("ui-gallery-form-switch-rtl")
                            .into_element(cx),
                        shadcn::Switch::new(switch.clone())
                            .control_id("ui-gallery-form-switch-rtl")
                            .a11y_label("????? ??????")
                            .into_element(cx),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .into_element(cx),
                ])
                .refine_layout(max_w_md.clone())
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-form-rtl");

        rtl_content
    };

    let notes = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "API reference: `ecosystem/fret-ui-shadcn/src/form.rs` (Form alias), `ecosystem/fret-ui-shadcn/src/field.rs` (FieldSet), and control primitives: `input.rs`, `textarea.rs`, `checkbox.rs`, `switch.rs`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "This page remains a gallery integration hub (composition recipe) rather than a single primitive component page.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep stable test IDs for each recipe so future diag automation can target composition surfaces.",
                ),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Form page is a gallery hub that combines Input, Textarea, Checkbox, Switch, and FieldSet composition.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("FieldSet + FieldGroup recipe with multiple controls.")
                .max_w(Px(840.0))
                .code(
                    "rust",
                    r#"shadcn::FieldSet::new([
    shadcn::FieldLegend::new("Contact").into_element(cx),
    shadcn::FieldDescription::new("Model-bound controls keep values.").into_element(cx),
    shadcn::FieldGroup::new([
        shadcn::Field::new([
            shadcn::FieldLabel::new("Email").into_element(cx),
            shadcn::Input::new(email).placeholder("name@example.com").into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::FieldLabel::new("Message").into_element(cx),
            shadcn::Textarea::new(message).into_element(cx),
        ])
        .into_element(cx),
    ])
    .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Input", input)
                .description("A model-bound input control.")
                .max_w(Px(840.0))
                .code(
                    "rust",
                    r#"shadcn::Input::new(model)
    .a11y_label("Email")
    .placeholder("name@example.com")
    .into_element(cx);"#,
                ),
            DocSection::new("Textarea", textarea)
                .description("A model-bound textarea control with fixed height.")
                .max_w(Px(840.0))
                .code(
                    "rust",
                    r#"shadcn::Textarea::new(model)
    .a11y_label("Message")
    .refine_layout(LayoutRefinement::default().h_px(Px(96.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("Checkbox + Switch", controls)
                .description("Basic checkbox + switch controls with labels.")
                .max_w(Px(840.0))
                .code(
                    "rust",
                    r#"stack::vstack(
    cx,
    stack::VStackProps::default().gap(Space::N3).items_start(),
    |cx| {
        vec![
            stack::hstack(cx, stack::HStackProps::default().gap(Space::N2).items_center(), |cx| {
                vec![
                    shadcn::Checkbox::new(accepted).a11y_label("Accept terms").into_element(cx),
                    shadcn::Label::new("Accept terms").into_element(cx),
                ]
            }),
            stack::hstack(cx, stack::HStackProps::default().gap(Space::N2).items_center(), |cx| {
                vec![
                    shadcn::Switch::new(enabled).a11y_label("Enable feature").into_element(cx),
                    shadcn::Label::new("Enable feature").into_element(cx),
                ]
            }),
        ]
    },
)
.into_element(cx);"#,
                ),
            DocSection::new("Fieldset", fieldset)
                .description("FieldSet recipe with grouped fields and action row.")
                .max_w(Px(840.0))
                .code(
                    "rust",
                    r#"shadcn::FieldSet::new([
    shadcn::FieldLegend::new("Profile").into_element(cx),
    shadcn::FieldGroup::new([
        shadcn::Field::new([
            shadcn::FieldLabel::new("Display name").into_element(cx),
            shadcn::Input::new(name).placeholder("Evil Rabbit").into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::Button::new("Submit").into_element(cx),
            shadcn::Button::new("Cancel")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx),
    ])
    .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Form composition under an RTL direction provider.")
                .max_w(Px(840.0))
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::FieldSet::new([
            shadcn::FieldLegend::new("?????").into_element(cx),
            shadcn::FieldGroup::new([shadcn::Field::new([
                shadcn::FieldLabel::new("?????? ??????????").into_element(cx),
                shadcn::Input::new(model).into_element(cx),
            ])
            .into_element(cx)])
            .into_element(cx),
        ])
        .into_element(cx)
    },
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-form")]
}
