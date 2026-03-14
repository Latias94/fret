pub const SOURCE: &str = include_str!("diag_surface.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);

    // Keep the long-list select stable for existing diag scripts (trigger + item test_ids).
    let entries: Vec<shadcn::SelectEntry> = std::iter::once(
        shadcn::SelectGroup::new([
            shadcn::SelectLabel::new("Fruits").into(),
            shadcn::SelectItem::new("apple", "Apple")
                .test_id("ui-gallery-select-item-apple")
                .into(),
            shadcn::SelectItem::new("banana", "Banana")
                .test_id("ui-gallery-select-item-banana")
                .into(),
            shadcn::SelectItem::new("blueberry", "Blueberry")
                .test_id("ui-gallery-select-item-blueberry")
                .into(),
            shadcn::SelectItem::new("grapes", "Grapes").into(),
            shadcn::SelectItem::new("pineapple", "Pineapple").into(),
        ])
        .into(),
    )
    .chain(std::iter::once(shadcn::SelectSeparator::default().into()))
    .chain(std::iter::once(
        shadcn::SelectGroup::new(
            std::iter::once(shadcn::SelectLabel::new("More").into()).chain((1..=40).map(|i| {
                let value: Arc<str> = Arc::from(format!("item-{i:02}"));
                let label: Arc<str> = Arc::from(format!("Item {i:02}"));
                let test_id: Arc<str> = Arc::from(format!("ui-gallery-select-item-{value}"));
                shadcn::SelectItem::new(value, label)
                    .test_id(test_id)
                    .disabled(i == 15)
                    .into()
            })),
        )
        .into(),
    ))
    .collect();

    let select = shadcn::Select::new(value.clone(), open)
        .trigger_test_id("ui-gallery-select-trigger")
        .trigger(
            shadcn::SelectTrigger::new().refine_layout(LayoutRefinement::default().w_px(Px(180.0))),
        )
        .value(shadcn::SelectValue::new().placeholder("Select a fruit"))
        .content(shadcn::SelectContent::new())
        .entries(entries)
        .into_element(cx);

    let selected_value = value.clone();
    let selected_label = cx.scope(move |cx| {
        let selected: Arc<str> = cx
            .get_model_cloned(&selected_value, Invalidation::Paint)
            .unwrap_or_default()
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        shadcn::raw::typography::muted(Arc::<str>::from(format!("Selected: {selected}")))
            .into_element(cx)
            .test_id("ui-gallery-select-selected-label")
    });

    ui::v_flex(|_cx| vec![select, selected_label])
        .gap(Space::N2)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-select-diag-surface")
}

// endregion: example
