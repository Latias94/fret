use super::super::*;

pub(super) fn preview_input(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct InputPageModels {
        field_value: Option<Model<String>>,
        fieldgroup_name: Option<Model<String>>,
        fieldgroup_email: Option<Model<String>>,
        disabled_value: Option<Model<String>>,
        invalid_value: Option<Model<String>>,
        file_value: Option<Model<String>>,
        inline_value: Option<Model<String>>,
        grid_first: Option<Model<String>>,
        grid_last: Option<Model<String>>,
        required_value: Option<Model<String>>,
        badge_value: Option<Model<String>>,
        input_group_value: Option<Model<String>>,
        button_group_value: Option<Model<String>>,
        form_name: Option<Model<String>>,
        form_email: Option<Model<String>>,
        form_phone: Option<Model<String>>,
        form_address: Option<Model<String>>,
        form_country: Option<Model<Option<Arc<str>>>>,
        form_country_open: Option<Model<bool>>,
        rtl_value: Option<Model<String>>,
    }

    let (
        field_value,
        fieldgroup_name,
        fieldgroup_email,
        disabled_value,
        invalid_value,
        file_value,
        inline_value,
        grid_first,
        grid_last,
        required_value,
        badge_value,
        input_group_value,
        button_group_value,
        form_name,
        form_email,
        form_phone,
        form_address,
        form_country,
        form_country_open,
        rtl_value,
    ) = cx.with_state(InputPageModels::default, |st| {
        (
            st.field_value.clone(),
            st.fieldgroup_name.clone(),
            st.fieldgroup_email.clone(),
            st.disabled_value.clone(),
            st.invalid_value.clone(),
            st.file_value.clone(),
            st.inline_value.clone(),
            st.grid_first.clone(),
            st.grid_last.clone(),
            st.required_value.clone(),
            st.badge_value.clone(),
            st.input_group_value.clone(),
            st.button_group_value.clone(),
            st.form_name.clone(),
            st.form_email.clone(),
            st.form_phone.clone(),
            st.form_address.clone(),
            st.form_country.clone(),
            st.form_country_open.clone(),
            st.rtl_value.clone(),
        )
    });

    let (
        field_value,
        fieldgroup_name,
        fieldgroup_email,
        disabled_value,
        invalid_value,
        file_value,
        inline_value,
        grid_first,
        grid_last,
        required_value,
        badge_value,
        input_group_value,
        button_group_value,
        form_name,
        form_email,
        form_phone,
        form_address,
        form_country,
        form_country_open,
        rtl_value,
    ) = match (
        field_value,
        fieldgroup_name,
        fieldgroup_email,
        disabled_value,
        invalid_value,
        file_value,
        inline_value,
        grid_first,
        grid_last,
        required_value,
        badge_value,
        input_group_value,
        button_group_value,
        form_name,
        form_email,
        form_phone,
        form_address,
        form_country,
        form_country_open,
        rtl_value,
    ) {
        (
            Some(field_value),
            Some(fieldgroup_name),
            Some(fieldgroup_email),
            Some(disabled_value),
            Some(invalid_value),
            Some(file_value),
            Some(inline_value),
            Some(grid_first),
            Some(grid_last),
            Some(required_value),
            Some(badge_value),
            Some(input_group_value),
            Some(button_group_value),
            Some(form_name),
            Some(form_email),
            Some(form_phone),
            Some(form_address),
            Some(form_country),
            Some(form_country_open),
            Some(rtl_value),
        ) => (
            field_value,
            fieldgroup_name,
            fieldgroup_email,
            disabled_value,
            invalid_value,
            file_value,
            inline_value,
            grid_first,
            grid_last,
            required_value,
            badge_value,
            input_group_value,
            button_group_value,
            form_name,
            form_email,
            form_phone,
            form_address,
            form_country,
            form_country_open,
            rtl_value,
        ),
        _ => {
            let field_value = cx.app.models_mut().insert(String::new());
            let fieldgroup_name = cx.app.models_mut().insert(String::new());
            let fieldgroup_email = cx.app.models_mut().insert(String::new());
            let disabled_value = cx
                .app
                .models_mut()
                .insert(String::from("disabled@example.com"));
            let invalid_value = cx.app.models_mut().insert(String::from("bad-email"));
            let file_value = cx.app.models_mut().insert(String::new());
            let inline_value = cx.app.models_mut().insert(String::new());
            let grid_first = cx.app.models_mut().insert(String::new());
            let grid_last = cx.app.models_mut().insert(String::new());
            let required_value = cx.app.models_mut().insert(String::new());
            let badge_value = cx.app.models_mut().insert(String::new());
            let input_group_value = cx.app.models_mut().insert(String::new());
            let button_group_value = cx.app.models_mut().insert(String::new());
            let form_name = cx.app.models_mut().insert(String::new());
            let form_email = cx.app.models_mut().insert(String::new());
            let form_phone = cx.app.models_mut().insert(String::new());
            let form_address = cx.app.models_mut().insert(String::new());
            let form_country = cx.app.models_mut().insert(Some(Arc::<str>::from("us")));
            let form_country_open = cx.app.models_mut().insert(false);
            let rtl_value = cx.app.models_mut().insert(String::new());

            cx.with_state(InputPageModels::default, |st| {
                st.field_value = Some(field_value.clone());
                st.fieldgroup_name = Some(fieldgroup_name.clone());
                st.fieldgroup_email = Some(fieldgroup_email.clone());
                st.disabled_value = Some(disabled_value.clone());
                st.invalid_value = Some(invalid_value.clone());
                st.file_value = Some(file_value.clone());
                st.inline_value = Some(inline_value.clone());
                st.grid_first = Some(grid_first.clone());
                st.grid_last = Some(grid_last.clone());
                st.required_value = Some(required_value.clone());
                st.badge_value = Some(badge_value.clone());
                st.input_group_value = Some(input_group_value.clone());
                st.button_group_value = Some(button_group_value.clone());
                st.form_name = Some(form_name.clone());
                st.form_email = Some(form_email.clone());
                st.form_phone = Some(form_phone.clone());
                st.form_address = Some(form_address.clone());
                st.form_country = Some(form_country.clone());
                st.form_country_open = Some(form_country_open.clone());
                st.rtl_value = Some(rtl_value.clone());
            });

            (
                field_value,
                fieldgroup_name,
                fieldgroup_email,
                disabled_value,
                invalid_value,
                file_value,
                inline_value,
                grid_first,
                grid_last,
                required_value,
                badge_value,
                input_group_value,
                button_group_value,
                form_name,
                form_email,
                form_phone,
                form_address,
                form_country,
                form_country_open,
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
                LayoutRefinement::default().w_full().max_w(Px(860.0)),
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

    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(420.0));

    let basic = {
        let content = shadcn::Input::new(value.clone())
            .a11y_label("Enter text")
            .placeholder("Enter text")
            .refine_layout(max_w_xs.clone())
            .into_element(cx)
            .test_id("ui-gallery-input-basic");
        section_card(cx, "Basic", content)
    };

    let field = {
        let content = shadcn::Field::new([
            shadcn::FieldLabel::new("Username").into_element(cx),
            shadcn::Input::new(field_value)
                .a11y_label("Username")
                .placeholder("Enter your username")
                .into_element(cx),
            shadcn::FieldDescription::new("Choose a unique username for your account.")
                .into_element(cx),
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-field");
        section_card(cx, "Field", content)
    };

    let field_group = {
        let content = shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldLabel::new("Name").into_element(cx),
                shadcn::Input::new(fieldgroup_name)
                    .a11y_label("Name")
                    .placeholder("Jordan Lee")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Email").into_element(cx),
                shadcn::Input::new(fieldgroup_email)
                    .a11y_label("Email")
                    .placeholder("name@example.com")
                    .into_element(cx),
                shadcn::FieldDescription::new("We'll send updates to this address.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::Button::new("Submit").into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-field-group");
        section_card(cx, "Field Group", content)
    };

    let disabled = {
        let content = shadcn::Field::new([
            shadcn::FieldLabel::new("Email").into_element(cx),
            shadcn::Input::new(disabled_value)
                .a11y_label("Disabled email")
                .disabled(true)
                .into_element(cx),
            shadcn::FieldDescription::new("This field is currently disabled.").into_element(cx),
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-disabled");
        section_card(cx, "Disabled", content)
    };

    let invalid = {
        let content = shadcn::Field::new([
            shadcn::FieldLabel::new("Invalid Input").into_element(cx),
            shadcn::Input::new(invalid_value)
                .a11y_label("Invalid input")
                .aria_invalid(true)
                .into_element(cx),
            shadcn::FieldDescription::new("This field contains validation errors.")
                .into_element(cx),
            shadcn::FieldError::new("Please provide a valid email format.").into_element(cx),
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-invalid");
        section_card(cx, "Invalid", content)
    };

    let file = {
        let content = shadcn::Field::new([
            shadcn::FieldLabel::new("Picture").into_element(cx),
            shadcn::ButtonGroup::new([
                shadcn::Input::new(file_value)
                    .a11y_label("Picture path")
                    .placeholder("Choose a file")
                    .into_element(cx)
                    .into(),
                shadcn::Button::new("Browse")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
                    .into(),
            ])
            .into_element(cx),
            shadcn::FieldDescription::new(
                "File input is approximated via text + browse button in current API.",
            )
            .into_element(cx),
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-file");
        section_card(cx, "File", content)
    };

    let inline = {
        let content = shadcn::Field::new([
            shadcn::Input::new(inline_value)
                .a11y_label("Search")
                .placeholder("Search...")
                .into_element(cx),
            shadcn::Button::new("Search").into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(max_w_xs.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-inline");
        section_card(cx, "Inline", content)
    };

    let grid = {
        let content = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(max_w_sm.clone())
                .gap(Space::N4)
                .items_start(),
            |cx| {
                vec![
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("First Name").into_element(cx),
                        shadcn::Input::new(grid_first)
                            .a11y_label("First name")
                            .placeholder("Jordan")
                            .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("Last Name").into_element(cx),
                        shadcn::Input::new(grid_last)
                            .a11y_label("Last name")
                            .placeholder("Lee")
                            .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-input-grid");
        section_card(cx, "Grid", content)
    };

    let required = {
        let label = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N1).items_center(),
            |cx| {
                vec![
                    shadcn::FieldLabel::new("Required Field").into_element(cx),
                    shadcn::typography::muted(cx, "*")
                        .attach_semantics(SemanticsDecoration::default().label("required-star")),
                ]
            },
        );

        let content = shadcn::Field::new([
            label,
            shadcn::Input::new(required_value)
                .a11y_label("Required field")
                .placeholder("This field is required")
                .into_element(cx),
            shadcn::FieldDescription::new("Mark required fields clearly in labels.")
                .into_element(cx),
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-required");
        section_card(cx, "Required", content)
    };

    let badge = {
        let label = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::FieldLabel::new("Webhook URL").into_element(cx),
                    shadcn::Badge::new("Recommended")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                ]
            },
        );

        let content = shadcn::Field::new([
            label,
            shadcn::Input::new(badge_value)
                .a11y_label("Webhook URL")
                .placeholder("https://example.com/webhook")
                .into_element(cx),
        ])
        .refine_layout(max_w_sm.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-badge");
        section_card(cx, "Badge", content)
    };

    let input_group = {
        let content = shadcn::Field::new([
            shadcn::FieldLabel::new("Website URL").into_element(cx),
            shadcn::InputGroup::new(input_group_value)
                .a11y_label("Website URL")
                .leading([shadcn::InputGroupText::new("https://").into_element(cx)])
                .trailing([
                    shadcn::InputGroupText::new(".com").into_element(cx),
                    shadcn::InputGroupButton::new("Info")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .into_element(cx),
                ])
                .refine_layout(max_w_xs.clone())
                .into_element(cx),
        ])
        .refine_layout(max_w_sm.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-input-group");
        section_card(cx, "Input Group", content)
    };

    let button_group = {
        let content = shadcn::Field::new([
            shadcn::FieldLabel::new("Search").into_element(cx),
            shadcn::ButtonGroup::new([
                shadcn::Input::new(button_group_value)
                    .a11y_label("Search text")
                    .placeholder("Type to search...")
                    .into_element(cx)
                    .into(),
                shadcn::Button::new("Search")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
                    .into(),
            ])
            .into_element(cx),
        ])
        .refine_layout(max_w_sm.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-button-group");
        section_card(cx, "Button Group", content)
    };

    let form = {
        let country = shadcn::Select::new(form_country, form_country_open)
            .placeholder("Country")
            .items([
                shadcn::SelectItem::new("us", "United States"),
                shadcn::SelectItem::new("uk", "United Kingdom"),
                shadcn::SelectItem::new("ca", "Canada"),
            ])
            .into_element(cx);

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N4)
                .items_start(),
            |cx| {
                vec![
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("Phone").into_element(cx),
                        shadcn::Input::new(form_phone)
                            .a11y_label("Phone")
                            .placeholder("+1 (555) 123-4567")
                            .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("Country").into_element(cx),
                        country,
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                ]
            },
        );

        let content = shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldLabel::new("Name").into_element(cx),
                shadcn::Input::new(form_name)
                    .a11y_label("Name")
                    .placeholder("Evil Rabbit")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Email").into_element(cx),
                shadcn::Input::new(form_email)
                    .a11y_label("Email")
                    .placeholder("john@example.com")
                    .into_element(cx),
                shadcn::FieldDescription::new("We'll never share your email.").into_element(cx),
            ])
            .into_element(cx),
            row,
            shadcn::Field::new([
                shadcn::FieldLabel::new("Address").into_element(cx),
                shadcn::Input::new(form_address)
                    .a11y_label("Address")
                    .placeholder("123 Main St")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::Button::new("Cancel")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::Button::new("Submit").into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
        ])
        .refine_layout(max_w_sm.clone())
        .into_element(cx)
        .test_id("ui-gallery-input-form");

        section_card(cx, "Form", content)
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Field::new([
                    shadcn::FieldLabel::new("????? API").into_element(cx),
                    shadcn::Input::new(rtl_value)
                        .a11y_label("????? API")
                        .placeholder("sk-...")
                        .into_element(cx),
                    shadcn::FieldDescription::new("??????? ???? ???? ?????? ???? ???.")
                        .into_element(cx),
                ])
                .refine_layout(max_w_xs.clone())
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-input-rtl");

        section_card(cx, "RTL", rtl_content)
    };

    let component_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Preview follows shadcn Input docs order: Basic, Field, Field Group, Disabled, Invalid, File, Inline, Grid, Required, Badge, Input Group, Button Group, Form, RTL.",
                ),
                basic,
                field,
                field_group,
                disabled,
                invalid,
                file,
                inline,
                grid,
                required,
                badge,
                input_group,
                button_group,
                form,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_panel_body).test_id("ui-gallery-input-component");

    let code_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Basic Input").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"Input::new(model).placeholder("Enter text").a11y_label("Enter text");"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Field Composition").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            "Field::new([FieldLabel, Input, FieldDescription]).into_element(cx);",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Input Group + Button Group").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            "InputGroup::new(model).leading([...]).trailing([...]); ButtonGroup::new([input.into(), button.into()]);",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        },
    );
    let code_panel = shell(cx, code_panel_body);

    let notes_panel_body = stack::vstack(
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
                    "Input page now uses docs-order examples and exposes stable test IDs for each section.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Native file input type is currently approximated using input + browse button composition.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Required styling is represented by label affordance because dedicated required visuals are not built into current Input API.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_panel_body);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-input",
        component_panel,
        code_panel,
        notes_panel,
    )
}
