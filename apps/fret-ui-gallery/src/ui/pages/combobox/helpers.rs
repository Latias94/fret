use super::prelude::*;

pub(super) fn centered(cx: &mut ElementContext<'_, App>, body: AnyElement) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_center(),
        move |_cx| [body],
    )
}

pub(super) fn section(
    cx: &mut ElementContext<'_, App>,
    title: &'static str,
    body: AnyElement,
) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        move |cx| vec![shadcn::typography::h4(cx, title), body],
    )
}

pub(super) fn shell(cx: &mut ElementContext<'_, App>, body: AnyElement) -> AnyElement {
    let props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default().w_full().max_w(Px(760.0)),
        )
    });
    cx.container(props, move |_cx| [body])
}

pub(super) fn section_card(
    cx: &mut ElementContext<'_, App>,
    title: &'static str,
    content: AnyElement,
) -> AnyElement {
    let card = shell(cx, content);
    let body = centered(cx, card);
    section(cx, title, body)
}

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
