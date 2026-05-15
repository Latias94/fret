pub const SOURCE: &str = include_str!("placement_ownership.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::element::ScrollAxis;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn deployment_items() -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("draft", "Draft"),
        shadcn::ComboboxItem::new("review", "In Review"),
        shadcn::ComboboxItem::new("staged", "Staged"),
        shadcn::ComboboxItem::new("release", "Release Ready"),
        shadcn::ComboboxItem::new("archived", "Archived"),
    ]
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("placement-ownership-value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("placement-ownership-open", || false);
    let query = cx.local_model_keyed("placement-ownership-query", String::new);

    let body = ui::v_flex(move |cx| {
        let top_gap = ui::container(|_cx| Vec::<AnyElement>::new())
            .layout(LayoutRefinement::default().w_full().h_px(Px(84.0)))
            .into_element(cx)
            .test_id("ui-gallery-combobox-placement-ownership-top-gap");

        let combo = with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
            shadcn::Combobox::new(value.clone(), open.clone())
                .a11y_label("Deployment status")
                .query_model(query.clone())
                .test_id_prefix("ui-gallery-combobox-placement-ownership")
                .trigger(
                    shadcn::ComboboxTrigger::new()
                        .variant(shadcn::ComboboxTriggerVariant::Button)
                        .width_px(Px(220.0)),
                )
                .input(shadcn::ComboboxInput::new().placeholder("Pick status"))
                .content(
                    shadcn::ComboboxContent::new([
                        shadcn::ComboboxContentPart::input(
                            shadcn::ComboboxInput::new().placeholder("Filter status..."),
                        ),
                        shadcn::ComboboxContentPart::empty(shadcn::ComboboxEmpty::new(
                            "No status found.",
                        )),
                        shadcn::ComboboxContentPart::list(
                            shadcn::ComboboxList::new().items(deployment_items()),
                        ),
                    ])
                    .width_px(Px(260.0))
                    .test_id("ui-gallery-combobox-placement-ownership-content"),
                )
                .into_element(cx)
        });

        let bottom_gap = ui::container(|_cx| Vec::<AnyElement>::new())
            .layout(LayoutRefinement::default().w_full().h_px(Px(160.0)))
            .into_element(cx)
            .test_id("ui-gallery-combobox-placement-ownership-bottom-gap");

        vec![top_gap, combo, bottom_gap]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx);

    let area = shadcn::ScrollArea::new([body])
        .axis(ScrollAxis::Y)
        .viewport_test_id("ui-gallery-combobox-placement-ownership-scroll-viewport")
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-combobox-placement-ownership-scroll-root");

    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().border_1().rounded(Radius::Md),
        LayoutRefinement::default()
            .w_px(Px(360.0))
            .h_px(Px(156.0))
            .overflow_hidden(),
    );

    cx.container(props, move |_cx| [area])
}
// endregion: example
