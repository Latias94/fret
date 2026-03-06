pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_app::App;
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
                .into_element_parts(cx, |_cx| {
                    vec![
                        shadcn::ComboboxPart::from(
                            shadcn::ComboboxTrigger::new().width_px(Px(260.0)),
                        ),
                        shadcn::ComboboxPart::from(
                            shadcn::ComboboxInput::new()
                                .placeholder("丕亘丨孬 毓賳 廿胤丕乇 毓賲賱"),
                        ),
                    ]
                })
        })]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
}
// endregion: example
