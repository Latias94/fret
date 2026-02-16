use super::prelude::*;

pub(super) fn state_rows(
    cx: &mut ElementContext<'_, App>,
    value: &Model<Option<Arc<str>>>,
    query: &Model<String>,
    test_id_prefix: &'static str,
) -> AnyElement {
    let selected = cx
        .app
        .models()
        .read(value, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let query_text = cx
        .get_model_cloned(query, Invalidation::Layout)
        .unwrap_or_default();

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(cx, format!("Selected: {selected}"))
                    .test_id(format!("{test_id_prefix}-selected")),
                shadcn::typography::muted(cx, format!("Query: {query_text}"))
                    .test_id(format!("{test_id_prefix}-query")),
            ]
        },
    )
}

pub(super) fn base_items() -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("apple", "Apple"),
        shadcn::ComboboxItem::new("banana", "Banana"),
        shadcn::ComboboxItem::new("orange", "Orange"),
        shadcn::ComboboxItem::new("disabled", "Disabled").disabled(true),
    ]
}
