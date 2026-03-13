pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("ui-gallery-native-select-usage-value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("ui-gallery-native-select-usage-open", || false);

    shadcn::native_select(value, open)
        .a11y_label("Native select")
        .placeholder("Select a fruit")
        .options([
            shadcn::NativeSelectOption::placeholder("Select a fruit"),
            shadcn::NativeSelectOption::new("apple", "Apple"),
            shadcn::NativeSelectOption::new("banana", "Banana"),
            shadcn::NativeSelectOption::new("blueberry", "Blueberry"),
            shadcn::NativeSelectOption::new("grapes", "Grapes").disabled(true),
            shadcn::NativeSelectOption::new("pineapple", "Pineapple"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-native-select-usage")
}
// endregion: example
