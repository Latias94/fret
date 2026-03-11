pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::NativeSelect::new_controllable(cx, None, None, None, false)
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
