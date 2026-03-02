pub const SOURCE: &str = include_str!("conformance_demo.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

fn base_items() -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("apple", "Apple"),
        shadcn::ComboboxItem::new("banana", "Banana"),
        shadcn::ComboboxItem::new("orange", "Orange"),
        shadcn::ComboboxItem::new("disabled", "Disabled").disabled(true),
    ]
}

pub fn render(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> AnyElement {
    shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox demo")
        .width(Px(260.0))
        .placeholder("Select a fruit")
        .auto_highlight(true)
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-demo")
        .trigger_test_id("ui-gallery-combobox-demo-trigger")
        .items(base_items())
        .into_element(cx)
}
// endregion: example
