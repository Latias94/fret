pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct Models {
    value: Option<Model<Option<Arc<str>>>>,
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (value, open) = cx.with_state(Models::default, |st| (st.value.clone(), st.open.clone()));
    let (value, open) = match (value, open) {
        (Some(value), Some(open)) => (value, open),
        _ => {
            let models = cx.app.models_mut();
            let value = models.insert(None::<Arc<str>>);
            let open = models.insert(false);
            cx.with_state(Models::default, |st| {
                st.value = Some(value.clone());
                st.open = Some(open.clone());
            });
            (value, open)
        }
    };

    let control_id = ControlId::from("ui-gallery-native-select-label");
    let native_select = shadcn::NativeSelect::new(value, open)
        .control_id(control_id.clone())
        .test_id_prefix("ui-gallery-native-select-label")
        .options([
            shadcn::NativeSelectOption::placeholder("Select a fruit"),
            shadcn::NativeSelectOption::new("apple", "Apple"),
            shadcn::NativeSelectOption::new("banana", "Banana"),
            shadcn::NativeSelectOption::new("blueberry", "Blueberry"),
        ])
        .into_element(cx);

    shadcn::FieldGroup::new([shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Fruit")
                .for_control(control_id.clone())
                .test_id("ui-gallery-native-select-label-label")
                .into_element(cx),
            shadcn::FieldDescription::new("Click the label to open the NativeSelect popup.")
                .for_control(control_id.clone())
                .into_element(cx),
        ])
        .into_element(cx),
        native_select,
    ])
    .into_element(cx)])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-native-select-label")
}
// endregion: example
