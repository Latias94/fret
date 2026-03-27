pub const SOURCE: &str = include_str!("custom_items.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);

    let custom_combo = shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox custom items")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-custom-items")
        .items([
            shadcn::ComboboxItem::new("next", "Next.js").detail("React"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js").detail("Vue"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit").detail("Svelte"),
            shadcn::ComboboxItem::new("astro", "Astro").detail("Hybrid"),
        ])
        .trigger(shadcn::ComboboxTrigger::new().width_px(Px(280.0)))
        .input(shadcn::ComboboxInput::new().placeholder("Select framework"))
        .into_element(cx);

    custom_combo
}
// endregion: example
