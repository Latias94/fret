pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_app::App;
use fret_ui_kit::primitives::direction::{LayoutDirection, with_direction_provider};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    value: Option<Model<Option<Arc<str>>>>,
    open: Option<Model<bool>>,
    query: Option<Model<String>>,
}

fn ensure_models(
    cx: &mut ElementContext<'_, App>,
) -> (Model<Option<Arc<str>>>, Model<bool>, Model<String>) {
    let state = cx.with_state(Models::default, |st| st.clone());

    let value = state.value.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(None);
        cx.with_state(Models::default, |st| st.value = Some(model.clone()));
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

    (value, open, query)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let (value, open, query) = ensure_models(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        move |cx| {
            vec![with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                shadcn::Combobox::new(value.clone(), open.clone())
                    .a11y_label("Combobox RTL")
                    .width(Px(260.0))
                    .placeholder("ابحث عن إطار عمل")
                    .query_model(query.clone())
                    .test_id_prefix("ui-gallery-combobox-rtl")
                    .trigger_test_id("ui-gallery-combobox-rtl-trigger")
                    .options([
                        shadcn::combobox_option("next", "Next.js"),
                        shadcn::combobox_option("nuxt", "Nuxt.js"),
                        shadcn::combobox_option("svelte", "SvelteKit"),
                    ])
                    .into_element(cx)
            })]
        },
    )
}
// endregion: example
