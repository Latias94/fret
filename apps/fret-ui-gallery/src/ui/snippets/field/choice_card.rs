pub const SOURCE: &str = include_str!("choice_card.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::FieldSet::new([
        shadcn::FieldLegend::new("Compute Environment")
            .variant(shadcn::FieldLegendVariant::Label)
            .into_element(cx),
        shadcn::FieldDescription::new("Select the compute environment for your cluster.")
            .into_element(cx),
        shadcn::RadioGroup::uncontrolled(Some("kubernetes"))
            .a11y_label("Compute environment")
            .item(
                shadcn::RadioGroupItem::new("kubernetes", "Kubernetes")
                    .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                    .child(
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("Kubernetes").into_element(cx),
                            shadcn::FieldDescription::new(
                                "Run GPU workloads on a K8s configured cluster.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("vm", "Virtual Machine")
                    .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                    .child(
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("Virtual Machine").into_element(cx),
                            shadcn::FieldDescription::new(
                                "Access a VM configured cluster to run GPU workloads.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    ),
            )
            .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-choice-card")
}
// endregion: example
