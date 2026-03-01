use super::prelude::*;

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

pub(super) fn state_rows(
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

pub(super) fn base_items() -> Vec<shadcn::ComboboxOption> {
    vec![
        shadcn::combobox_option("apple", "Apple"),
        shadcn::combobox_option("banana", "Banana"),
        shadcn::combobox_option("orange", "Orange"),
        shadcn::combobox_option("disabled", "Disabled").disabled(true),
    ]
}
