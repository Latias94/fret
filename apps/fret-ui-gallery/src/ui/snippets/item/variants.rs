pub const SOURCE: &str = include_str!("variants.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(448.0)));

    let button = |cx: &mut ElementContext<'_, App>| {
        shadcn::Button::new("Open")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx)
    };

    ui::v_stack(|cx| {
        vec![
            shadcn::Item::new([
                shadcn::ItemContent::new([
                    shadcn::ItemTitle::new("Default Variant").into_element(cx),
                    shadcn::ItemDescription::new(
                        "Standard styling with subtle background and borders.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::ItemActions::new([button(cx)]).into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-variant-default"),
            shadcn::Item::new([
                shadcn::ItemContent::new([
                    shadcn::ItemTitle::new("Outline Variant").into_element(cx),
                    shadcn::ItemDescription::new(
                        "Outlined style with clear borders and transparent background.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::ItemActions::new([button(cx)]).into_element(cx),
            ])
            .variant(shadcn::ItemVariant::Outline)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-variant-outline"),
            shadcn::Item::new([
                shadcn::ItemContent::new([
                    shadcn::ItemTitle::new("Muted Variant").into_element(cx),
                    shadcn::ItemDescription::new(
                        "Subdued appearance with muted colors for secondary content.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::ItemActions::new([button(cx)]).into_element(cx),
            ])
            .variant(shadcn::ItemVariant::Muted)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-variant-muted"),
        ]
    })
    .gap(Space::N6)
    .items_start()
    .layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-item-variants")
}
// endregion: example
