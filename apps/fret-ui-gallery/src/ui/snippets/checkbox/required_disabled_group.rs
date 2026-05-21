pub const SOURCE: &str = include_str!("required_disabled_group.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let backups = cx.local_model_keyed("checkbox_required_disabled_group_backups", || true);
    let analytics = cx.local_model_keyed("checkbox_required_disabled_group_analytics", || false);
    let beta = cx.local_model_keyed("checkbox_required_disabled_group_beta", || false);

    let group_item = |cx: &mut AppComponentCx<'_>,
                      label: &'static str,
                      description: &'static str,
                      value: Model<bool>,
                      test_id: &'static str,
                      label_test_id: &'static str,
                      disabled: bool| {
        let checkbox = shadcn::Checkbox::new(value)
            .required(true)
            .control_id(test_id)
            .a11y_label(label)
            .test_id(test_id);
        let checkbox = if disabled {
            checkbox.disabled(true)
        } else {
            checkbox
        };

        let field = shadcn::Field::new([
            checkbox.into_element(cx),
            shadcn::FieldContent::new([
                shadcn::FieldLabel::new(label)
                    .for_control(test_id)
                    .test_id(label_test_id)
                    .into_element(cx),
                shadcn::FieldDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w_full());
        let field = if disabled {
            field.disabled(true)
        } else {
            field
        };

        field.into_element(cx)
    };

    shadcn::field_set(|cx| {
        ui::children![
            cx;
            shadcn::FieldLegend::new("Required desktop items")
                .variant(shadcn::FieldLegendVariant::Label),
            shadcn::FieldDescription::new(
                "Choose required items. Disabled managed options cannot be toggled.",
            ),
            shadcn::field_group(|cx| {
                ui::children![
                    cx;
                    group_item(
                        cx,
                        "Backups",
                        "Enabled option selected by default.",
                        backups,
                        "ui-gallery-checkbox-required-disabled-backups",
                        "ui-gallery-checkbox-required-disabled-backups-label",
                        false,
                    ),
                    group_item(
                        cx,
                        "Usage analytics",
                        "Managed by your organization and locked off.",
                        analytics,
                        "ui-gallery-checkbox-required-disabled-analytics",
                        "ui-gallery-checkbox-required-disabled-analytics-label",
                        true,
                    ),
                    group_item(
                        cx,
                        "Beta updates",
                        "Enabled option used by diagnostics to prove mutation still works.",
                        beta,
                        "ui-gallery-checkbox-required-disabled-beta",
                        "ui-gallery-checkbox-required-disabled-beta-label",
                        false,
                    ),
                ]
            })
            .checkbox_group(),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(480.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-required-disabled-group")
}
// endregion: example
