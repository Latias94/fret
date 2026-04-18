pub const SOURCE: &str = include_str!("multiple_selection.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let values = cx.local_model_keyed("values", Vec::<Arc<str>>::new);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);

    let combo = shadcn::ComboboxChips::new(values.clone(), open.clone())
        .a11y_label("Combobox multiple selection")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-multiple")
        .items([
            shadcn::ComboboxItem::new("next", "Next.js"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            shadcn::ComboboxItem::new("remix", "Remix"),
            shadcn::ComboboxItem::new("astro", "Astro"),
        ])
        .trigger(shadcn::ComboboxTrigger::new().width_px(Px(260.0)))
        .input(shadcn::ComboboxChipsInput::new().placeholder("Select frameworks"))
        .into_element(cx);

    ui::v_flex(move |_cx| vec![combo])
        .gap(Space::N2)
        .items_start()
        .layout(LayoutRefinement::default().w_full().max_w(Px(340.0)))
        .into_element(cx)
}
// endregion: example
