pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct Models {
    value: Option<Model<Option<Arc<str>>>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.with_state(Models::default, |st| st.value.clone());
    let value = match value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("free")));
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    };

    let control_id = ControlId::from("ui-gallery-radio-group-label");
    let radio_group = shadcn::RadioGroup::new(value)
        .control_id(control_id.clone())
        .test_id_prefix("ui-gallery-radio-group-label")
        .item(shadcn::RadioGroupItem::new("free", "Free"))
        .item(shadcn::RadioGroupItem::new("pro", "Pro"))
        .item(shadcn::RadioGroupItem::new("enterprise", "Enterprise"))
        .into_element(cx);

    shadcn::FieldGroup::new([shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Plan")
                .for_control(control_id.clone())
                .test_id("ui-gallery-radio-group-label-label")
                .into_element(cx),
            shadcn::FieldDescription::new("Click the label to focus the active radio item.")
                .for_control(control_id.clone())
                .into_element(cx),
        ])
        .into_element(cx),
        radio_group,
    ])
    .into_element(cx)])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-radio-group-label")
}
// endregion: example
