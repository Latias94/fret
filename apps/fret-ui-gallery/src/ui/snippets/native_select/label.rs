pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("ui-gallery-native-select-label-value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("ui-gallery-native-select-label-open", || false);

    let control_id = ControlId::from("ui-gallery-native-select-label");
    let native_select = shadcn::native_select(value, open)
        .control_id(control_id.clone())
        .test_id_prefix("ui-gallery-native-select-label")
        .options([
            shadcn::NativeSelectOption::placeholder("Select a fruit"),
            shadcn::NativeSelectOption::new("apple", "Apple"),
            shadcn::NativeSelectOption::new("banana", "Banana"),
            shadcn::NativeSelectOption::new("blueberry", "Blueberry"),
        ])
        .into_element(cx);

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
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
            ]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-native-select-label")
}
// endregion: example
