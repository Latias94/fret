pub const SOURCE: &str = include_str!("required_disabled.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("radio-group-required-disabled", || {
        Some(Arc::<str>::from("team"))
    });

    let self_service_id = "ui-gallery-radio-group-required-disabled-self-service";
    let team_id = "ui-gallery-radio-group-required-disabled-team";
    let enterprise_id = "ui-gallery-radio-group-required-disabled-enterprise";

    let group = shadcn::RadioGroup::new(value)
        .required(true)
        .a11y_label("Support plan")
        .refine_layout(LayoutRefinement::default().w_full())
        .test_id_prefix("ui-gallery-radio-group-required-disabled")
        .item(
            shadcn::RadioGroupItem::new("self-service", "Self-service")
                .disabled(true)
                .control_id(self_service_id),
        )
        .item(shadcn::RadioGroupItem::new("team", "Team").control_id(team_id))
        .item(shadcn::RadioGroupItem::new("enterprise", "Enterprise").control_id(enterprise_id))
        .into_element_parts(cx, |cx, parts| {
            vec![
                shadcn::Field::new([
                    parts.control(cx, "self-service"),
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Self-service")
                            .for_control(self_service_id)
                            .test_id("ui-gallery-radio-group-required-disabled-item-0-label")
                            .into_element(cx),
                        shadcn::FieldDescription::new(
                            "Disabled plans cannot be selected, even when their label is clicked.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .disabled(true)
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "team"),
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Team")
                            .for_control(team_id)
                            .test_id("ui-gallery-radio-group-required-disabled-item-1-label")
                            .into_element(cx),
                        shadcn::FieldDescription::new(
                            "Selected by default so required group semantics remain visible.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "enterprise"),
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Enterprise")
                            .for_control(enterprise_id)
                            .test_id("ui-gallery-radio-group-required-disabled-item-2-label")
                            .into_element(cx),
                        shadcn::FieldDescription::new(
                            "Enabled option used by diagnostics to prove mutation still works.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
            ]
        });

    shadcn::field_set(|cx| {
        ui::children![
            cx;
            shadcn::FieldLegend::new("Support plan")
                .variant(shadcn::FieldLegendVariant::Label),
            shadcn::FieldDescription::new(
                "Choose one required support plan. Disabled rows keep disabled action-state on the concrete radio item.",
            ),
            group,
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
    .into_element(cx)
    .test_id("ui-gallery-radio-group-required-disabled")
}
// endregion: example
