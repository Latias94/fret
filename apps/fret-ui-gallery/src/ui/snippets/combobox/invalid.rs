pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::UiCx;
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
    cx: &mut UiCx<'_>,
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
    cx.container(props, move |cx| [shadcn::raw::typography::muted(cx, text)])
        .test_id(test_id)
}

fn state_rows(
    cx: &mut UiCx<'_>,
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

    ui::v_flex(move |cx| {
        vec![
            state_row(cx, selected_row_text.clone(), selected_row_test_id.clone()),
            state_row(cx, query_row_text.clone(), query_row_test_id.clone()),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);

    let invalid_combo = shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox invalid")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-invalid")
        .items(base_items())
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::ComboboxPart::from(shadcn::ComboboxTrigger::new().width_px(Px(260.0))),
                shadcn::ComboboxPart::from(
                    shadcn::ComboboxInput::new()
                        .placeholder("Select required option")
                        .aria_invalid(true),
                ),
            ]
        });

    ui::v_flex(move |cx| {
        vec![
            invalid_combo,
            state_rows(cx, &value, &query, "ui-gallery-combobox-invalid"),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
}
// endregion: example
