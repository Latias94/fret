pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{AppComponentCx, UiChild};
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

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);

    let disabled_combo = shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox disabled")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-disabled")
        .items(base_items())
        .trigger(shadcn::ComboboxTrigger::new().width_px(Px(260.0)))
        .input(
            shadcn::ComboboxInput::new()
                .placeholder("Disabled")
                .disabled(true),
        )
        .into_element(cx);

    disabled_combo
}
// endregion: example
