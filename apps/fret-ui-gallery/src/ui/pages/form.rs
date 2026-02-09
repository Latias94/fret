use super::super::*;

pub(super) fn preview_forms(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
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
                LayoutRefinement::default().w_full().max_w(Px(840.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

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
                        .a11y_label("Accept terms")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Accept terms").into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Enable feature").into_element(cx),
                        shadcn::FieldDescription::new(
                            "This toggles an optional feature for the current session.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Switch::new(switch.clone())
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

        section_card(cx, "Demo", content)
    };

    let input = {
        let content = shadcn::Input::new(text_input.clone())
            .a11y_label("Email")
            .placeholder("name@example.com")
            .refine_layout(max_w_md.clone())
            .into_element(cx)
            .test_id("ui-gallery-form-input");

        section_card(cx, "Input", content)
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

        section_card(cx, "Textarea", content)
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

        section_card(cx, "Checkbox + Switch", content)
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

        section_card(cx, "Fieldset", content)
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
                        shadcn::FieldLabel::new("????? ??????").into_element(cx),
                        shadcn::Switch::new(switch.clone())
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

        section_card(cx, "RTL", rtl_content)
    };

    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Form page is a gallery hub that combines Input, Textarea, Checkbox, Switch and FieldSet composition.",
                ),
                demo,
                input,
                textarea,
                controls,
                fieldset,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-form-component");

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("FieldSet Composition").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            "FieldSet::new([FieldLegend, FieldDescription, FieldGroup]).into_element(cx);",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Model-Bound Controls").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            "Input::new(text_model), Textarea::new(text_model), Checkbox::new(bool_model), Switch::new(bool_model);",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "This page remains a gallery integration hub instead of a direct shadcn radix component page.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Stable test IDs are added for each section to support future diag automation.",
                ),
                shadcn::typography::muted(
                    cx,
                    "When Form docs get formalized, this page can be reordered to mirror upstream sections exactly.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-form",
        component_panel,
        code_panel,
        notes_panel,
    )
}
