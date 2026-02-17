use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_checkbox(
    cx: &mut ElementContext<'_, App>,
    model: Model<bool>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct CheckboxModels {
        checked_controlled: Option<Model<bool>>,
        checked_optional: Option<Model<Option<bool>>>,
        invalid: Option<Model<bool>>,
        description: Option<Model<bool>>,
        disabled: Option<Model<bool>>,
        group_security: Option<Model<bool>>,
        group_updates: Option<Model<bool>>,
        group_marketing: Option<Model<bool>>,
        table_all: Option<Model<bool>>,
        table_row_1: Option<Model<bool>>,
        table_row_2: Option<Model<bool>>,
        table_row_3: Option<Model<bool>>,
        rtl: Option<Model<bool>>,
    }

    let (
        checked_controlled,
        checked_optional,
        invalid,
        description,
        disabled,
        group_security,
        group_updates,
        group_marketing,
        table_all,
        table_row_1,
        table_row_2,
        table_row_3,
        rtl,
    ) = cx.with_state(CheckboxModels::default, |st| {
        (
            st.checked_controlled.clone(),
            st.checked_optional.clone(),
            st.invalid.clone(),
            st.description.clone(),
            st.disabled.clone(),
            st.group_security.clone(),
            st.group_updates.clone(),
            st.group_marketing.clone(),
            st.table_all.clone(),
            st.table_row_1.clone(),
            st.table_row_2.clone(),
            st.table_row_3.clone(),
            st.rtl.clone(),
        )
    });

    let (
        checked_controlled,
        checked_optional,
        invalid,
        description,
        disabled,
        group_security,
        group_updates,
        group_marketing,
        table_all,
        table_row_1,
        table_row_2,
        table_row_3,
        rtl,
    ) = match (
        checked_controlled,
        checked_optional,
        invalid,
        description,
        disabled,
        group_security,
        group_updates,
        group_marketing,
        table_all,
        table_row_1,
        table_row_2,
        table_row_3,
        rtl,
    ) {
        (
            Some(checked_controlled),
            Some(checked_optional),
            Some(invalid),
            Some(description),
            Some(disabled),
            Some(group_security),
            Some(group_updates),
            Some(group_marketing),
            Some(table_all),
            Some(table_row_1),
            Some(table_row_2),
            Some(table_row_3),
            Some(rtl),
        ) => (
            checked_controlled,
            checked_optional,
            invalid,
            description,
            disabled,
            group_security,
            group_updates,
            group_marketing,
            table_all,
            table_row_1,
            table_row_2,
            table_row_3,
            rtl,
        ),
        _ => {
            let checked_controlled = cx.app.models_mut().insert(true);
            let checked_optional = cx.app.models_mut().insert(None);
            let invalid = cx.app.models_mut().insert(false);
            let description = cx.app.models_mut().insert(false);
            let disabled = cx.app.models_mut().insert(true);
            let group_security = cx.app.models_mut().insert(true);
            let group_updates = cx.app.models_mut().insert(false);
            let group_marketing = cx.app.models_mut().insert(false);
            let table_all = cx.app.models_mut().insert(false);
            let table_row_1 = cx.app.models_mut().insert(true);
            let table_row_2 = cx.app.models_mut().insert(false);
            let table_row_3 = cx.app.models_mut().insert(false);
            let rtl = cx.app.models_mut().insert(true);

            cx.with_state(CheckboxModels::default, |st| {
                st.checked_controlled = Some(checked_controlled.clone());
                st.checked_optional = Some(checked_optional.clone());
                st.invalid = Some(invalid.clone());
                st.description = Some(description.clone());
                st.disabled = Some(disabled.clone());
                st.group_security = Some(group_security.clone());
                st.group_updates = Some(group_updates.clone());
                st.group_marketing = Some(group_marketing.clone());
                st.table_all = Some(table_all.clone());
                st.table_row_1 = Some(table_row_1.clone());
                st.table_row_2 = Some(table_row_2.clone());
                st.table_row_3 = Some(table_row_3.clone());
                st.rtl = Some(rtl.clone());
            });

            (
                checked_controlled,
                checked_optional,
                invalid,
                description,
                disabled,
                group_security,
                group_updates,
                group_marketing,
                table_all,
                table_row_1,
                table_row_2,
                table_row_3,
                rtl,
            )
        }
    };

    let destructive = cx.with_theme(|theme| theme.color_token("destructive"));

    let demo_checked = cx
        .get_model_copied(&model, Invalidation::Layout)
        .unwrap_or(false);
    let controlled_checked = cx
        .get_model_copied(&checked_controlled, Invalidation::Layout)
        .unwrap_or(false);
    let optional_checked = cx
        .get_model_copied(&checked_optional, Invalidation::Layout)
        .unwrap_or(None);

    let demo_content = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_center(),
        |cx| {
            vec![
                shadcn::Checkbox::new(model.clone())
                    .control_id("ui-gallery-checkbox-demo-toggle")
                    .a11y_label("Accept terms")
                    .test_id("ui-gallery-checkbox-demo-toggle")
                    .into_element(cx),
                shadcn::FieldLabel::new("Accept terms and conditions")
                    .for_control("ui-gallery-checkbox-demo-toggle")
                    .into_element(cx),
                cx.spacer(fret_ui::element::SpacerProps::default()),
                shadcn::typography::muted(
                    cx,
                    if demo_checked {
                        "checked=true"
                    } else {
                        "checked=false"
                    },
                ),
            ]
        },
    )
    .test_id("ui-gallery-checkbox-demo");
    let demo = demo_content;

    let checked_state_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))),
        |cx| {
            let optional_text = match optional_checked {
                Some(true) => "state=Some(true)",
                Some(false) => "state=Some(false)",
                None => "state=None (indeterminate)",
            };

            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N3)
                        .items_center(),
                    |cx| {
                        vec![
                            shadcn::Checkbox::new(checked_controlled.clone())
                                .control_id("ui-gallery-checkbox-controlled")
                                .a11y_label("Controlled checkbox")
                                .test_id("ui-gallery-checkbox-controlled")
                                .into_element(cx),
                            shadcn::FieldLabel::new("Controlled checked state")
                                .for_control("ui-gallery-checkbox-controlled")
                                .into_element(cx),
                            cx.spacer(fret_ui::element::SpacerProps::default()),
                            shadcn::typography::muted(
                                cx,
                                if controlled_checked {
                                    "state=true"
                                } else {
                                    "state=false"
                                },
                            ),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N3)
                        .items_center(),
                    |cx| {
                        vec![
                            shadcn::Checkbox::new_optional(checked_optional.clone())
                                .control_id("ui-gallery-checkbox-optional")
                                .a11y_label("Optional checkbox")
                                .test_id("ui-gallery-checkbox-optional")
                                .into_element(cx),
                            shadcn::FieldLabel::new("Optional / indeterminate state")
                                .for_control("ui-gallery-checkbox-optional")
                                .into_element(cx),
                            cx.spacer(fret_ui::element::SpacerProps::default()),
                            shadcn::typography::muted(cx, optional_text),
                        ]
                    },
                ),
            ]
        },
    )
    .test_id("ui-gallery-checkbox-checked-state");
    let checked_state = checked_state_content;

    let invalid_content = shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Accept terms")
                .for_control("ui-gallery-checkbox-invalid")
                .into_element(cx),
            shadcn::FieldDescription::new("You must accept before continuing.").into_element(cx),
            shadcn::FieldError::new("Please accept the terms to proceed.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::Checkbox::new(invalid.clone())
            .control_id("ui-gallery-checkbox-invalid")
            .a11y_label("Invalid checkbox")
            .test_id("ui-gallery-checkbox-invalid")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_style(
        ChromeRefinement::default()
            .border_1()
            .rounded(Radius::Md)
            .border_color(ColorRef::Color(destructive))
            .p(Space::N3),
    )
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-invalid-field");
    let invalid_state = invalid_content;

    let basic_content = shadcn::Field::new([
        shadcn::Checkbox::new(model.clone())
            .control_id("ui-gallery-checkbox-basic")
            .a11y_label("Basic checkbox")
            .test_id("ui-gallery-checkbox-basic")
            .into_element(cx),
        shadcn::FieldLabel::new("Accept terms and conditions")
            .for_control("ui-gallery-checkbox-basic")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-basic-field");
    let basic = basic_content;

    let description_content = shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Enable notifications")
                .for_control("ui-gallery-checkbox-description")
                .into_element(cx),
            shadcn::FieldDescription::new(
                "Receive updates about release notes, fixes, and maintenance windows.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Checkbox::new(description.clone())
            .control_id("ui-gallery-checkbox-description")
            .a11y_label("Enable notifications")
            .test_id("ui-gallery-checkbox-description")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-description-field");
    let description_section = description_content;

    let disabled_content = shadcn::Field::new([
        shadcn::Checkbox::new(disabled.clone())
            .control_id("ui-gallery-checkbox-disabled")
            .disabled(true)
            .a11y_label("Disabled checkbox")
            .test_id("ui-gallery-checkbox-disabled")
            .into_element(cx),
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Marketing emails")
                .for_control("ui-gallery-checkbox-disabled")
                .into_element(cx),
            shadcn::FieldDescription::new("This preference is managed by your organization.")
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-disabled-field");
    let disabled_section = disabled_content;

    let group_item = |cx: &mut ElementContext<'_, App>,
                      label: &'static str,
                      desc: &'static str,
                      value: Model<bool>,
                      test_id: &'static str| {
        shadcn::Field::new([
            shadcn::Checkbox::new(value)
                .control_id(test_id)
                .a11y_label(label)
                .test_id(test_id)
                .into_element(cx),
            shadcn::FieldContent::new([
                shadcn::FieldLabel::new(label)
                    .for_control(test_id)
                    .into_element(cx),
                shadcn::FieldDescription::new(desc).into_element(cx),
            ])
            .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx)
    };

    let group_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(460.0))),
        |cx| {
            vec![
                group_item(
                    cx,
                    "Security alerts",
                    "Critical account changes and sign-in events.",
                    group_security.clone(),
                    "ui-gallery-checkbox-group-security",
                ),
                group_item(
                    cx,
                    "Product updates",
                    "Major feature releases and migration notices.",
                    group_updates.clone(),
                    "ui-gallery-checkbox-group-updates",
                ),
                group_item(
                    cx,
                    "Marketing emails",
                    "Tips, webinars, and promotional announcements.",
                    group_marketing.clone(),
                    "ui-gallery-checkbox-group-marketing",
                ),
            ]
        },
    )
    .test_id("ui-gallery-checkbox-group");
    let group = group_content;

    let table_row = |cx: &mut ElementContext<'_, App>,
                     id: &'static str,
                     role: &'static str,
                     checked: Model<bool>,
                     test_id: &'static str| {
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(
                    shadcn::Checkbox::new(checked)
                        .a11y_label(format!("Select {id}"))
                        .test_id(test_id)
                        .into_element(cx),
                )
                .into_element(cx),
                shadcn::TableCell::new(cx.text(id)).into_element(cx),
                shadcn::TableCell::new(cx.text(role)).into_element(cx),
            ],
        )
        .border_bottom(true)
        .into_element(cx)
    };

    let table_content = shadcn::Table::new(vec![
        shadcn::TableHeader::new(vec![
            shadcn::TableRow::new(
                3,
                vec![
                    shadcn::TableCell::new(
                        shadcn::Checkbox::new(table_all.clone())
                            .a11y_label("Select all rows")
                            .test_id("ui-gallery-checkbox-table-all")
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::TableHead::new("Member").into_element(cx),
                    shadcn::TableHead::new("Role").into_element(cx),
                ],
            )
            .border_bottom(true)
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::TableBody::new(vec![
            table_row(
                cx,
                "Alex Johnson",
                "Owner",
                table_row_1.clone(),
                "ui-gallery-checkbox-table-row-1",
            ),
            table_row(
                cx,
                "Riley Chen",
                "Editor",
                table_row_2.clone(),
                "ui-gallery-checkbox-table-row-2",
            ),
            table_row(
                cx,
                "Morgan Lee",
                "Viewer",
                table_row_3.clone(),
                "ui-gallery-checkbox-table-row-3",
            ),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-table");
    let table = table_content;

    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::Field::new([
                shadcn::Checkbox::new(rtl.clone())
                    .control_id("ui-gallery-checkbox-rtl")
                    .a11y_label("RTL notifications")
                    .test_id("ui-gallery-checkbox-rtl")
                    .into_element(cx),
                shadcn::FieldLabel::new("Enable notifications (RTL)")
                    .for_control("ui-gallery-checkbox-rtl")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
            .into_element(cx)
        },
    )
    .test_id("ui-gallery-checkbox-rtl-field");
    let rtl_section = rtl_content;

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
                    "API reference: `ecosystem/fret-ui-shadcn/src/checkbox.rs` (Checkbox).",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use Field composition (FieldLabel/FieldDescription) to keep label, helper text, and toggle target aligned.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For indeterminate behavior, prefer `Checkbox::new_optional(Model<Option<bool>>)`, where `None` maps to mixed state.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Table selection patterns should keep row-level and header-level states explicit; avoid hidden coupling in demos.",
                ),
                shadcn::typography::muted(
                    cx,
                    "When validating parity, test both keyboard focus ring and RTL label alignment in addition to pointer clicks.",
                ),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Checkbox docs flow: Demo -> Checked State -> Invalid State -> Basic -> Description -> Disabled -> Group -> Table -> RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Single checkbox with a live state readout.")
                .code(
                    "rust",
                    r#"let checkbox = shadcn::Checkbox::new(model)
    .a11y_label("Accept terms")
    .into_element(cx);"#,
                ),
            DocSection::new("Checked State", checked_state)
                .description("Controlled checked model and optional/indeterminate model.")
                .code(
                    "rust",
                    r#"let controlled = shadcn::Checkbox::new(controlled_model);
let optional = shadcn::Checkbox::new_optional(optional_model); // None => indeterminate"#,
                ),
            DocSection::new("Invalid State", invalid_state)
                .description("Invalid styling is currently approximated via destructive border.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Basic", basic)
                .description("Field + checkbox + label composition.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Description", description_section)
                .description("FieldContent keeps label and helper text aligned with the control.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Disabled", disabled_section)
                .description("Disabled checkbox should block interaction and use muted styling.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Group", group)
                .description("Checkbox group pattern with per-item descriptions.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Table", table)
                .description("Table selection pattern with header and row checkboxes.")
                .code(
                    "rust",
                    r#"shadcn::TableCell::new(
    shadcn::Checkbox::new(model).a11y_label("Select row").into_element(cx),
)"#,
                ),
            DocSection::new("RTL", rtl_section)
                .description("Checkbox + label alignment under an RTL direction provider.")
                .code(
                    "rust",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::Checkbox::new(model).into_element(cx)
})"#,
                ),
            DocSection::new("Notes", notes).description("API reference pointers and parity notes."),
        ],
    );

    vec![body.test_id("ui-gallery-checkbox")]
}
