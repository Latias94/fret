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
        .a11y_label("Combobox groups with separator")
        .width(Px(300.0))
        .placeholder("Select a timezone")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-groups-separator")
        .trigger_test_id("ui-gallery-combobox-groups-separator-trigger")
        .group_separators(true)
        .option_groups([
            shadcn::combobox_option_group(
                "Americas",
                [
                    shadcn::combobox_option("americas-ny", "(GMT-5) New York"),
                    shadcn::combobox_option("americas-la", "(GMT-8) Los Angeles"),
                    shadcn::combobox_option("americas-chi", "(GMT-6) Chicago"),
                ],
            ),
            shadcn::combobox_option_group(
                "Europe",
                [
                    shadcn::combobox_option("europe-lon", "(GMT+0) London"),
                    shadcn::combobox_option("europe-paris", "(GMT+1) Paris"),
                    shadcn::combobox_option("europe-berlin", "(GMT+1) Berlin"),
                ],
            ),
            shadcn::combobox_option_group(
                "Asia/Pacific",
                [
                    shadcn::combobox_option("asia-tokyo", "(GMT+9) Tokyo"),
                    shadcn::combobox_option("asia-shanghai", "(GMT+8) Shanghai"),
                    shadcn::combobox_option("asia-singapore", "(GMT+8) Singapore"),
                ],
            ),
        ])
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(340.0))),
        move |cx| {
            vec![
                combo,
                state_rows(cx, &value, &query, "ui-gallery-combobox-groups-separator"),
            ]
        },
    )
}
// endregion: example
