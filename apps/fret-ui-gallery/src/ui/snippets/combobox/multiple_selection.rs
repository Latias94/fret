pub const SOURCE: &str = include_str!("multiple_selection.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    values: Option<Model<Vec<Arc<str>>>>,
    open: Option<Model<bool>>,
    query: Option<Model<String>>,
}

fn ensure_models(
    cx: &mut ElementContext<'_, App>,
) -> (Model<Vec<Arc<str>>>, Model<bool>, Model<String>) {
    let state = cx.with_state(Models::default, |st| st.clone());

    let values = state.values.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(Vec::<Arc<str>>::new());
        cx.with_state(Models::default, |st| st.values = Some(model.clone()));
        model
    });
    let open = state.open.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(false);
        cx.with_state(Models::default, |st| st.open = Some(model.clone()));
        model
    });
    let query = state.query.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(String::new());
        cx.with_state(Models::default, |st| st.query = Some(model.clone()));
        model
    });

    (values, open, query)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let (values, open, query) = ensure_models(cx);

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
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::ComboboxTrigger::new().width_px(Px(260.0)).into(),
                shadcn::ComboboxChipsInput::new()
                    .placeholder("Select frameworks")
                    .into(),
            ]
        });

    ui::v_flex(move |_cx| vec![combo])
        .gap(Space::N2)
        .items_start()
        .layout(LayoutRefinement::default().w_full().max_w(Px(340.0)))
        .into_element(cx)
}
// endregion: example
