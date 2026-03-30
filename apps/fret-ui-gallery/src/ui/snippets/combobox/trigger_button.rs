pub const SOURCE: &str = include_str!("trigger_button.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);

    let combo = shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox popup trigger")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-popup")
        .items([
            shadcn::ComboboxItem::new("next", "Next.js"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            shadcn::ComboboxItem::new("remix", "Remix"),
            shadcn::ComboboxItem::new("astro", "Astro"),
        ])
        .trigger(
            shadcn::ComboboxTrigger::new()
                .variant(shadcn::ComboboxTriggerVariant::Button)
                .width_px(Px(256.0)),
        )
        .input(shadcn::ComboboxInput::new().placeholder("Select a framework"))
        .content(shadcn::ComboboxContent::new([
            shadcn::ComboboxContentPart::input(
                shadcn::ComboboxInput::new().placeholder("Change framework..."),
            ),
            shadcn::ComboboxContentPart::empty(shadcn::ComboboxEmpty::new("No results found.")),
        ]))
        .into_element(cx);

    combo
}
// endregion: example
