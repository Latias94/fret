pub const SOURCE: &str = include_str!("input_group.rs");

// region: example
use fret_app::App;
use fret_ui_kit::declarative::{ElementContextThemeExt, style as decl_style};
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

fn state_row(cx: &mut ElementContext<'_, App>, text: Arc<str>, test_id: Arc<str>) -> AnyElement {
    let props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default().bg(ColorRef::Color(theme.color_token("background"))),
            LayoutRefinement::default().w_full().min_w_0(),
        )
    });
    cx.container(props, move |cx| [shadcn::typography::muted(cx, text)])
        .test_id(test_id)
}

fn state_rows(
    cx: &mut ElementContext<'_, App>,
    value: &Model<Option<Arc<str>>>,
    query: &Model<String>,
    test_id_prefix: &'static str,
) -> AnyElement {
    let selected: Arc<str> = cx
        .get_model_cloned(value, Invalidation::Paint)
        .unwrap_or_default()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let query_text = cx
        .get_model_cloned(query, Invalidation::Paint)
        .unwrap_or_default();

    let selected_row_text: Arc<str> = Arc::from(format!("Selected: {selected}"));
    let query_row_text: Arc<str> = Arc::from(format!("Query: {query_text}"));
    let selected_row_test_id: Arc<str> = Arc::from(format!("{test_id_prefix}-selected"));
    let query_row_test_id: Arc<str> = Arc::from(format!("{test_id_prefix}-query"));

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                state_row(cx, selected_row_text.clone(), selected_row_test_id.clone()),
                state_row(cx, query_row_text.clone(), query_row_test_id.clone()),
            ]
        },
    )
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let (value, open, query) = ensure_models(cx);

    let combo = shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox input group")
        .width(Px(220.0))
        .placeholder("Search command")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-input-group")
        .trigger_test_id("ui-gallery-combobox-input-group-trigger")
        .items([
            shadcn::ComboboxItem::new("new-file", "New File"),
            shadcn::ComboboxItem::new("open-file", "Open File"),
            shadcn::ComboboxItem::new("save-all", "Save All"),
        ])
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(360.0))),
        move |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            {
                                let props = cx.with_theme(|theme| {
                                    decl_style::container_props(
                                        theme,
                                        ChromeRefinement::default()
                                            .border_1()
                                            .rounded(Radius::Sm)
                                            .px(Space::N2)
                                            .py(Space::N1),
                                        LayoutRefinement::default(),
                                    )
                                });
                                cx.container(props, |cx| vec![shadcn::typography::muted(cx, "Cmd")])
                            },
                            combo,
                        ]
                    },
                ),
                state_rows(cx, &value, &query, "ui-gallery-combobox-input-group"),
            ]
        },
    )
}
// endregion: example
