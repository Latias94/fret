use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

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

    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(420.0));

    let basic = {
        let content = shadcn::Input::new(value.clone())
            .a11y_label("Enter text")
            .placeholder("Enter text")
            .refine_layout(max_w_xs.clone())
            .into_element(cx)
            .test_id("ui-gallery-input-basic");
        content
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
        content
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
        content
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
        content
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
        content
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
        content
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
        content
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
        content
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
        content
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
        content
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
        content
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
        content
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
        content
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::Field::new([
            shadcn::FieldLabel::new("????? API").into_element(cx),
            shadcn::Input::new(rtl_value)
                .a11y_label("????? API")
                .placeholder("sk-...")
                .into_element(cx),
            shadcn::FieldDescription::new("??????? ???? ???? ?????? ???? ???.").into_element(cx),
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx)
    })
    .test_id("ui-gallery-input-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/input.rs` (Input), `ecosystem/fret-ui-shadcn/src/input_group.rs` (InputGroup), `ecosystem/fret-ui-shadcn/src/button_group.rs` (ButtonGroup).",
            "Native file input type is currently approximated using input + browse button composition.",
            "Required styling is represented by label affordance because dedicated required visuals are not built into current Input API.",
            "Keep `ui-gallery-input-basic` stable for IME routing regression scripts.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Input docs order: Basic, Field, Field Group, Disabled, Invalid, File, Inline, Grid, Required, Badge, Input Group, Button Group, Form, RTL.",
        ),
        vec![
            DocSection::new("Basic", basic)
                .description("Single input field (used by IME routing regression scripts).")
                .code(
                    "rust",
                    r#"shadcn::Input::new(model)
    .a11y_label("Enter text")
    .placeholder("Enter text")
    .into_element(cx);"#,
                ),
            DocSection::new("Field", field)
                .description("Field composition with label, description, and error slots.")
                .code(
                    "rust",
                    r#"shadcn::Field::new([
    shadcn::FieldLabel::new("Username").into_element(cx),
    shadcn::Input::new(model).placeholder("Enter your username").into_element(cx),
    shadcn::FieldDescription::new("Choose a unique username.").into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Field Group", field_group)
                .description("FieldGroup stacks related fields and action rows.")
                .code(
                    "rust",
                    r#"shadcn::FieldGroup::new([
    shadcn::Field::new([
        shadcn::FieldLabel::new("Name").into_element(cx),
        shadcn::Input::new(name).placeholder("Jordan Lee").into_element(cx),
    ])
    .into_element(cx),
    shadcn::Field::new([
        shadcn::FieldLabel::new("Email").into_element(cx),
        shadcn::Input::new(email).placeholder("name@example.com").into_element(cx),
        shadcn::FieldDescription::new("We'll send updates to this address.").into_element(cx),
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
.into_element(cx);"#,
                ),
            DocSection::new("Disabled", disabled).description(
                "Disabled inputs should block focus/interaction and use muted styling.",
            )
            .code(
                "rust",
                r#"shadcn::Input::new(model)
    .a11y_label("Disabled email")
    .disabled(true)
    .into_element(cx);"#,
            ),
            DocSection::new("Invalid", invalid)
                .description("Invalid state uses `aria_invalid` + field-level error copy.")
                .code(
                    "rust",
                    r#"shadcn::Field::new([
    shadcn::FieldLabel::new("Invalid Input").into_element(cx),
    shadcn::Input::new(model)
        .a11y_label("Invalid input")
        .aria_invalid(true)
        .into_element(cx),
    shadcn::FieldError::new("Please provide a valid email format.").into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("File", file).description(
                "File input is approximated via text + browse button composition in current API.",
            )
            .code(
                "rust",
                r#"shadcn::ButtonGroup::new([
    shadcn::Input::new(model)
        .a11y_label("Picture path")
        .placeholder("Choose a file")
        .into_element(cx)
        .into(),
    shadcn::Button::new("Browse")
        .variant(shadcn::ButtonVariant::Outline)
        .into_element(cx)
        .into(),
])
.into_element(cx);"#,
            ),
            DocSection::new("Inline", inline)
                .description("Horizontal Field orientation is useful for compact toolbars.")
                .code(
                    "rust",
                    r#"shadcn::Field::new([
    shadcn::Input::new(model)
        .a11y_label("Search")
        .placeholder("Search...")
        .into_element(cx),
    shadcn::Button::new("Search").into_element(cx),
])
.orientation(shadcn::FieldOrientation::Horizontal)
.into_element(cx);"#,
                ),
            DocSection::new("Grid", grid)
                .description("Two-column input layout with shared row alignment.")
                .code(
                    "rust",
                    r#"stack::hstack(
    cx,
    stack::HStackProps::default().gap(Space::N4).items_start(),
    |cx| {
        vec![
            shadcn::Field::new([
                shadcn::FieldLabel::new("First Name").into_element(cx),
                shadcn::Input::new(first).placeholder("Jordan").into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Last Name").into_element(cx),
                shadcn::Input::new(last).placeholder("Lee").into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        ]
    },
)
.into_element(cx);"#,
                ),
            DocSection::new("Required", required)
                .description("Required affordance is represented by label copy in this gallery.")
                .code(
                    "rust",
                    r#"let label = stack::hstack(
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

shadcn::Field::new([
    label,
    shadcn::Input::new(model).placeholder("This field is required").into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Badge", badge).description("Use Badge inside a label row.")
                .code(
                    "rust",
                    r#"let label = stack::hstack(
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

shadcn::Field::new([
    label,
    shadcn::Input::new(model)
        .a11y_label("Webhook URL")
        .placeholder("https://example.com/webhook")
        .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Input Group", input_group)
                .description("Inline addons and trailing buttons via InputGroup composition.")
                .code(
                    "rust",
                    r#"shadcn::InputGroup::new(model)
    .leading([shadcn::InputGroupText::new("https://").into_element(cx)])
    .trailing([shadcn::InputGroupText::new(".com").into_element(cx)])
    .into_element(cx);"#,
                ),
            DocSection::new("Button Group", button_group)
                .description("ButtonGroup composes an input and a button with shared chrome.")
                .code(
                    "rust",
                    r#"shadcn::ButtonGroup::new([
    shadcn::Input::new(model)
        .a11y_label("Search text")
        .placeholder("Type to search...")
        .into_element(cx)
        .into(),
    shadcn::Button::new("Search")
        .variant(shadcn::ButtonVariant::Outline)
        .into_element(cx)
        .into(),
])
.into_element(cx);"#,
                ),
            DocSection::new("Form", form)
                .description("Multi-field form layout using FieldGroup + responsive rows.")
                .code(
                    "rust",
                    r#"let country = shadcn::Select::new(country, country_open)
    .placeholder("Country")
    .items([
        shadcn::SelectItem::new("us", "United States"),
        shadcn::SelectItem::new("uk", "United Kingdom"),
    ])
    .into_element(cx);

shadcn::FieldGroup::new([
    shadcn::Field::new([
        shadcn::FieldLabel::new("Name").into_element(cx),
        shadcn::Input::new(name).placeholder("Evil Rabbit").into_element(cx),
    ])
    .into_element(cx),
    shadcn::Field::new([shadcn::FieldLabel::new("Country").into_element(cx), country]).into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Input + Field composition under an RTL direction provider.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::Field::new([
            shadcn::FieldLabel::new("????? API").into_element(cx),
            shadcn::Input::new(model)
                .a11y_label("????? API")
                .placeholder("sk-...")
                .into_element(cx),
        ])
        .into_element(cx)
    },
);"#,
                ),
            DocSection::new("Notes", notes).description("API reference pointers and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-input")]
}
