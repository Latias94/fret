use super::super::super::super::*;

pub(in crate::ui) fn preview_select(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    let select = shadcn::Select::new(value.clone(), open)
        .trigger_test_id("ui-gallery-select-trigger")
        .placeholder("Pick a fruit")
        .items(
            [
                shadcn::SelectItem::new("apple", "Apple").test_id("ui-gallery-select-item-apple"),
                shadcn::SelectItem::new("banana", "Banana")
                    .test_id("ui-gallery-select-item-banana"),
                shadcn::SelectItem::new("orange", "Orange")
                    .test_id("ui-gallery-select-item-orange"),
            ]
            .into_iter()
            .chain((1..=40).map(|i| {
                let value: Arc<str> = Arc::from(format!("item-{i:02}"));
                let label: Arc<str> = Arc::from(format!("Item {i:02}"));
                let test_id: Arc<str> = Arc::from(format!("ui-gallery-select-item-{value}"));
                shadcn::SelectItem::new(value, label)
                    .test_id(test_id)
                    .disabled(i == 15)
            })),
        )
        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
        .into_element(cx);

    let selected_label = cx
        .scope(|cx| {
            let selected: Arc<str> = cx
                .get_model_cloned(&value, fret_ui::Invalidation::Paint)
                .unwrap_or_default()
                .unwrap_or_else(|| Arc::<str>::from("<none>"));

            fret_ui::element::AnyElement::new(
                cx.root_id(),
                fret_ui::element::ElementKind::Text(fret_ui::element::TextProps::new(format!(
                    "Selected: {selected}"
                ))),
                Vec::new(),
            )
        })
        .attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .test_id("ui-gallery-select-selected-label"),
        );

    vec![select, selected_label]
}
