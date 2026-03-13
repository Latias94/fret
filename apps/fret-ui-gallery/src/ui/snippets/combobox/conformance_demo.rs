pub const SOURCE: &str = include_str!("conformance_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
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
    cx: &mut UiCx<'_>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> impl UiChild + use<> {
    shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox demo")
        .auto_highlight(true)
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-demo")
        .items(base_items())
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::ComboboxPart::from(shadcn::ComboboxTrigger::new().width_px(Px(260.0))),
                shadcn::ComboboxPart::from(
                    shadcn::ComboboxInput::new().placeholder("Select a fruit"),
                ),
            ]
        })
}
// endregion: example
