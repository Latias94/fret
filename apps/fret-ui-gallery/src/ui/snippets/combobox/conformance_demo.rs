pub const SOURCE: &str = include_str!("conformance_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::{ElementContextThemeExt, style as decl_style};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn base_items() -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("apple", "Apple"),
        shadcn::ComboboxItem::new("banana", "Banana"),
        shadcn::ComboboxItem::new("orange", "Orange"),
        shadcn::ComboboxItem::new("disabled", "Disabled").disabled(true),
    ]
}

fn state_row(
    cx: &mut AppComponentCx<'_>,
    text: Arc<str>,
    test_id: Arc<str>,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default().bg(ColorRef::Color(theme.color_token("background"))),
            LayoutRefinement::default().w_full().min_w_0(),
        )
    });
    let label = text.clone();
    cx.container(props, move |cx| {
        [shadcn::typography::muted(text).into_element(cx)]
    })
    .a11y_label(label)
    .test_id(test_id)
}

fn state_rows(
    cx: &mut AppComponentCx<'_>,
    value: &Model<Option<Arc<str>>>,
    query: &Model<String>,
    test_id_prefix: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
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

    ui::v_flex(move |cx| {
        vec![
            state_row(cx, selected_row_text.clone(), selected_row_test_id.clone()),
            state_row(cx, query_row_text.clone(), query_row_test_id.clone()),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub fn render(
    cx: &mut AppComponentCx<'_>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> impl UiChild + use<> {
    let combo = shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox demo")
        .auto_highlight(true)
        .query_model(query.clone())
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(260.0))
                .min_w_0(),
        )
        .test_id_prefix("ui-gallery-combobox-demo")
        .items(base_items())
        .trigger(shadcn::ComboboxTrigger::new())
        .input(shadcn::ComboboxInput::new().placeholder("Select a fruit"))
        .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            combo,
            state_rows(cx, &value, &query, "ui-gallery-combobox-demo").into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(260.0)))
    .into_element(cx)
}
// endregion: example
