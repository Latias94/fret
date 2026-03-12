pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::UiCx;
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);

    shadcn::Combobox::new(value, open)
        .query_model(query)
        .a11y_label("Framework combobox")
        .test_id_prefix("ui-gallery-combobox-usage")
        .items([
            shadcn::ComboboxItem::new("next.js", "Next.js"),
            shadcn::ComboboxItem::new("sveltekit", "SvelteKit"),
            shadcn::ComboboxItem::new("nuxt.js", "Nuxt.js"),
            shadcn::ComboboxItem::new("remix", "Remix"),
            shadcn::ComboboxItem::new("astro", "Astro"),
        ])
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::ComboboxPart::from(shadcn::ComboboxTrigger::new().width_px(Px(200.0))),
                shadcn::ComboboxPart::from(
                    shadcn::ComboboxInput::new().placeholder("Select framework..."),
                ),
            ]
        })
}
// endregion: example
