pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);

    ui::v_flex(move |cx| {
        vec![with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
            shadcn::Combobox::new(value.clone(), open.clone())
                .a11y_label("Combobox RTL")
                .query_model(query.clone())
                .test_id_prefix("ui-gallery-combobox-rtl")
                .items([
                    shadcn::ComboboxItem::new("next", "Next.js"),
                    shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
                    shadcn::ComboboxItem::new("svelte", "SvelteKit"),
                ])
                .trigger(shadcn::ComboboxTrigger::new().width_px(Px(260.0)))
                .input(shadcn::ComboboxInput::new().placeholder("اختر إطار العمل"))
                .into_element(cx)
        })]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
}
// endregion: example
