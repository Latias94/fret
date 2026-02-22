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
        with_title: Option<Model<bool>>,
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
        with_title,
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
            st.with_title.clone(),
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
        with_title,
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
        with_title,
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
            Some(with_title),
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
            with_title,
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
            let with_title = cx.app.models_mut().insert(true);
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
                st.with_title = Some(with_title.clone());
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
                with_title,
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

    let demo_checked = cx
        .get_model_copied(&model, Invalidation::Layout)
        .unwrap_or(false);
    let controlled_checked = cx
        .get_model_copied(&checked_controlled, Invalidation::Layout)
        .unwrap_or(false);
    let optional_checked = cx
        .get_model_copied(&checked_optional, Invalidation::Layout)
        .unwrap_or(None);
    let invalid_checked = cx
        .get_model_copied(&invalid, Invalidation::Layout)
        .unwrap_or(false);

    let demo = stack::hstack(
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
                    .test_id("ui-gallery-checkbox-demo-label")
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

    let checked_state = stack::vstack(
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
                                .test_id("ui-gallery-checkbox-controlled-label")
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
                                .test_id("ui-gallery-checkbox-optional-label")
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

    let invalid_state = shadcn::Field::new([
        shadcn::Checkbox::new(invalid.clone())
            .control_id("ui-gallery-checkbox-invalid")
            .a11y_label("Invalid checkbox")
            .aria_invalid(!invalid_checked)
            .test_id("ui-gallery-checkbox-invalid")
            .into_element(cx),
        shadcn::FieldLabel::new("Accept terms and conditions")
            .for_control("ui-gallery-checkbox-invalid")
            .into_element(cx),
    ])
    .invalid(!invalid_checked)
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-invalid-field");

    let basic = shadcn::Field::new([
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

    let description_section = shadcn::Field::new([
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

    let disabled_section = shadcn::Field::new([
        shadcn::Checkbox::new(disabled.clone())
            .control_id("ui-gallery-checkbox-disabled")
            .disabled(true)
            .a11y_label("Disabled checkbox")
            .test_id("ui-gallery-checkbox-disabled")
            .into_element(cx),
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Marketing emails")
                .for_control("ui-gallery-checkbox-disabled")
                .test_id("ui-gallery-checkbox-disabled-label")
                .into_element(cx),
            shadcn::FieldDescription::new("This preference is managed by your organization.")
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .disabled(true)
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-disabled-field");

    let with_title_section = shadcn::FieldGroup::new([
        shadcn::FieldLabel::new("Enable notifications")
            .for_control("ui-gallery-checkbox-with-title")
            .test_id("ui-gallery-checkbox-with-title-label")
            .wrap([shadcn::Field::new([
                shadcn::Checkbox::new(with_title.clone())
                    .control_id("ui-gallery-checkbox-with-title")
                    .a11y_label("Enable notifications")
                    .test_id("ui-gallery-checkbox-with-title")
                    .into_element(cx),
                shadcn::FieldContent::new([
                    shadcn::FieldTitle::new("Enable notifications").into_element(cx),
                    shadcn::FieldDescription::new(
                        "You can enable or disable notifications at any time.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx)])
            .into_element(cx),
        shadcn::FieldLabel::new("Enable notifications (disabled)")
            .for_control("ui-gallery-checkbox-with-title-disabled")
            .test_id("ui-gallery-checkbox-with-title-disabled-label")
            .wrap([shadcn::Field::new([
                shadcn::Checkbox::new(with_title.clone())
                    .control_id("ui-gallery-checkbox-with-title-disabled")
                    .disabled(true)
                    .a11y_label("Enable notifications (disabled)")
                    .test_id("ui-gallery-checkbox-with-title-disabled")
                    .into_element(cx),
                shadcn::FieldContent::new([
                    shadcn::FieldTitle::new("Enable notifications").into_element(cx),
                    shadcn::FieldDescription::new(
                        "You can enable or disable notifications at any time.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .disabled(true)
            .into_element(cx)])
            .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-checkbox-with-title-section");

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

    let group = stack::vstack(
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

    let table = shadcn::Table::new(vec![
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

    let rtl_section = doc_layout::rtl(cx, |cx| {
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
    })
    .test_id("ui-gallery-checkbox-rtl-field");

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/checkbox.rs` (Checkbox).",
            "Use Field composition (FieldLabel/FieldDescription) to keep label, helper text, and toggle target aligned.",
            "For indeterminate behavior, prefer `Checkbox::new_optional(Model<Option<bool>>)`, where `None` maps to mixed state.",
            "Table selection patterns should keep row-level and header-level states explicit; avoid hidden coupling in demos.",
            "When validating parity, test both keyboard focus ring and RTL label alignment in addition to pointer clicks.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Checkbox docs flow: Demo -> Checked State -> Invalid State -> Basic -> Description -> Disabled -> With Title -> Group -> Table -> RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Single checkbox with a live state readout.")
                .code(
                    "rust",
                    r#"let accepted = cx.app.models_mut().insert(false);

let row = stack::hstack(
    cx,
    stack::HStackProps::default()
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N3)
        .items_center(),
    |cx| {
        vec![
            shadcn::Checkbox::new(accepted.clone())
                // Required for label click -> focus/toggle forwarding.
                .control_id("terms")
                .a11y_label("Accept terms")
                .into_element(cx),
            shadcn::FieldLabel::new("Accept terms and conditions")
                .for_control("terms")
                .into_element(cx),
        ]
    },
);

row"#,
                ),
            DocSection::new("Checked State", checked_state)
                .description("Controlled checked model and optional/indeterminate model.")
                .code(
                    "rust",
                    r#"let controlled = cx.app.models_mut().insert(true);
let optional = cx.app.models_mut().insert(None::<bool>); // None => indeterminate

stack::vstack(cx, stack::VStackProps::default().gap(Space::N3), |cx| {
    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N3).items_center(),
            |cx| {
                vec![
                    shadcn::Checkbox::new(controlled.clone())
                        .control_id("controlled")
                        .a11y_label("Controlled checkbox")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Controlled checked state")
                        .for_control("controlled")
                        .into_element(cx),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N3).items_center(),
            |cx| {
                vec![
                    shadcn::Checkbox::new_optional(optional.clone())
                        .control_id("optional")
                        .a11y_label("Optional checkbox")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Optional / indeterminate state")
                        .for_control("optional")
                        .into_element(cx),
                ]
            },
        ),
    ]
})"#,
                ),
            DocSection::new("Invalid State", invalid_state)
                .description("Invalid styling uses `aria_invalid` on the checkbox and destructive label text.")
                .code(
                    "rust",
                    r#"let accepted = cx.app.models_mut().insert(false);

let is_invalid = !cx
    .get_model_copied(&accepted, Invalidation::Layout)
    .unwrap_or(false);

shadcn::Field::new([
    shadcn::Checkbox::new(accepted.clone())
        .control_id("accept")
        .a11y_label("Accept terms")
        .aria_invalid(is_invalid)
        .into_element(cx),
    shadcn::FieldLabel::new("Accept terms and conditions")
        .for_control("accept")
        .into_element(cx),
])
.orientation(shadcn::FieldOrientation::Horizontal)
.invalid(is_invalid)
.into_element(cx);"#,
                ),
            DocSection::new("Basic", basic)
                .description("Field + checkbox + label composition.")
                .code(
                    "rust",
                    r#"shadcn::Field::new([
    shadcn::Checkbox::new(model)
        .control_id("accept")
        .a11y_label("Accept terms")
        .into_element(cx),
    shadcn::FieldLabel::new("Accept terms and conditions")
        .for_control("accept")
        .into_element(cx),
])
.orientation(shadcn::FieldOrientation::Horizontal)
.into_element(cx);"#,
                ),
            DocSection::new("Description", description_section)
                .description("FieldContent keeps label and helper text aligned with the control.")
                .code(
                    "rust",
                    r#"shadcn::Field::new([
    shadcn::FieldContent::new([
        shadcn::FieldLabel::new("Enable notifications")
            .for_control("notify")
            .into_element(cx),
        shadcn::FieldDescription::new("Receive updates and maintenance windows.").into_element(cx),
    ])
    .into_element(cx),
    shadcn::Checkbox::new(model)
        .control_id("notify")
        .a11y_label("Enable notifications")
        .into_element(cx),
])
.orientation(shadcn::FieldOrientation::Horizontal)
.into_element(cx);"#,
                ),
            DocSection::new("Disabled", disabled_section)
                .description("Disabled checkbox should block interaction and use muted styling.")
                .code(
                    "rust",
                    r#"shadcn::Field::new([
    shadcn::Checkbox::new(model)
        .control_id("marketing")
        .disabled(true)
        .a11y_label("Marketing emails")
        .into_element(cx),
    shadcn::FieldContent::new([
        shadcn::FieldLabel::new("Marketing emails")
            .for_control("marketing")
            .into_element(cx),
        shadcn::FieldDescription::new("Managed by your organization.").into_element(cx),
    ])
    .into_element(cx),
])
.orientation(shadcn::FieldOrientation::Horizontal)
.disabled(true)
.into_element(cx);"#,
                ),
            DocSection::new("With Title", with_title_section)
                .description("FieldLabel can wrap a full Field layout (card-style label).")
                .code(
                    "rust",
                    r#"let checked = cx.app.models_mut().insert(true);

shadcn::FieldGroup::new([
    shadcn::FieldLabel::new("Enable notifications")
        .for_control("toggle")
        .wrap([shadcn::Field::new([
            shadcn::Checkbox::new(checked.clone())
                .control_id("toggle")
                .a11y_label("Enable notifications")
                .into_element(cx),
            shadcn::FieldContent::new([
                shadcn::FieldTitle::new("Enable notifications").into_element(cx),
                shadcn::FieldDescription::new(
                    "You can enable or disable notifications at any time.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx)])
        .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Group", group)
                .description("Checkbox group pattern with per-item descriptions.")
                .code(
                    "rust",
                    r#"let security = cx.app.models_mut().insert(true);
let updates = cx.app.models_mut().insert(false);
let marketing = cx.app.models_mut().insert(false);

let item = |cx: &mut ElementContext<'_, App>,
            id: &'static str,
            label: &'static str,
            desc: &'static str,
            model: Model<bool>| {
    shadcn::Field::new([
        shadcn::Checkbox::new(model)
            // Required for label click -> focus/toggle forwarding.
            .control_id(id)
            .a11y_label(label)
            .into_element(cx),
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new(label)
                .for_control(id)
                .into_element(cx),
            shadcn::FieldDescription::new(desc).into_element(cx),
        ])
        .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .into_element(cx)
};

stack::vstack(cx, stack::VStackProps::default().gap(Space::N3).items_start(), |cx| {
    vec![
        item(
            cx,
            "security",
            "Security alerts",
            "Critical account changes.",
            security.clone(),
        ),
        item(
            cx,
            "updates",
            "Product updates",
            "Major feature releases.",
            updates.clone(),
        ),
        item(
            cx,
            "marketing",
            "Marketing emails",
            "Tips and announcements.",
            marketing.clone(),
        ),
    ]
});"#,
                ),
            DocSection::new("Table", table)
                .description("Table selection pattern with header and row checkboxes.")
                .code(
                    "rust",
                    r#"let all = cx.app.models_mut().insert(false);
let row_1 = cx.app.models_mut().insert(true);

let header = shadcn::TableRow::new(
    3,
    vec![
        shadcn::TableCell::new(
            shadcn::Checkbox::new(all.clone())
                .a11y_label("Select all rows")
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::TableHead::new("Member").into_element(cx),
        shadcn::TableHead::new("Role").into_element(cx),
    ],
)
.border_bottom(true)
.into_element(cx);

let row = shadcn::TableRow::new(
    3,
    vec![
        shadcn::TableCell::new(
            shadcn::Checkbox::new(row_1.clone())
                .a11y_label("Select Alex Johnson")
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::TableCell::new(cx.text("Alex Johnson")).into_element(cx),
        shadcn::TableCell::new(cx.text("Owner")).into_element(cx),
    ],
)
.border_bottom(true)
.into_element(cx);

shadcn::Table::new(vec![
    shadcn::TableHeader::new(vec![header]).into_element(cx),
    shadcn::TableBody::new(vec![row]).into_element(cx),
])
.into_element(cx)"#,
                ),
            DocSection::new("RTL", rtl_section)
                .description("Checkbox + label alignment under an RTL direction provider.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::Field::new([
            shadcn::Checkbox::new(model)
                .control_id("notify")
                .a11y_label("Enable notifications")
                .into_element(cx),
            shadcn::FieldLabel::new("Enable notifications (RTL)")
                .for_control("notify")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx)
    },
)"#,
                ),
            DocSection::new("Notes", notes).description("API reference pointers and parity notes."),
        ],
    );

    vec![body.test_id("ui-gallery-checkbox")]
}
