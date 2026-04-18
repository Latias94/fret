pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);

    shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox basic")
        .query_model(query.clone())
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(260.0))
                .min_w_0(),
        )
        .test_id_prefix("ui-gallery-combobox-basic")
        .items([
            shadcn::ComboboxItem::new("next", "Next.js"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            shadcn::ComboboxItem::new("remix", "Remix"),
            shadcn::ComboboxItem::new("astro", "Astro"),
        ])
        .trigger(shadcn::ComboboxTrigger::new())
        .input(shadcn::ComboboxInput::new().placeholder("Select a framework"))
        .content(shadcn::ComboboxContent::new([
            shadcn::ComboboxContentPart::input(
                shadcn::ComboboxInput::new().placeholder("Search framework..."),
            ),
            shadcn::ComboboxContentPart::empty(shadcn::ComboboxEmpty::new("No framework found.")),
        ]))
        .into_element(cx)
}
// endregion: example
