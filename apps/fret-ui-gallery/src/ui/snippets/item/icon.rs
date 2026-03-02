pub const SOURCE: &str = include_str!("icon.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn icon(cx: &mut ElementContext<'_, App>, id: &'static str) -> AnyElement {
    shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
}

fn icon_button(
    cx: &mut ElementContext<'_, App>,
    icon_id: &'static str,
    variant: shadcn::ButtonVariant,
    test_id: &'static str,
) -> AnyElement {
    shadcn::Button::new("")
        .a11y_label(icon_id)
        .variant(variant)
        .size(shadcn::ButtonSize::Icon)
        .icon(fret_icons::IconId::new_static(icon_id))
        .into_element(cx)
        .test_id(test_id)
}

fn item_icon(
    cx: &mut ElementContext<'_, App>,
    icon_id: &'static str,
    title: &'static str,
    description: &'static str,
    test_id: &'static str,
) -> AnyElement {
    let media = shadcn::ItemMedia::new([icon(cx, icon_id)])
        .variant(shadcn::ItemMediaVariant::Icon)
        .into_element(cx);

    let content_children = [
        shadcn::ItemTitle::new(title).into_element(cx),
        shadcn::ItemDescription::new(description).into_element(cx),
    ];

    let review = shadcn::Button::new("Review")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx);

    let actions = shadcn::ItemActions::new([review]).into_element(cx);

    shadcn::Item::new([
        media,
        shadcn::ItemContent::new(content_children).into_element(cx),
        actions,
    ])
    .variant(shadcn::ItemVariant::Outline)
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id(test_id)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_lg = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(520.0)));

    let item = item_icon(
        cx,
        "lucide.shield-alert",
        "Security Alert",
        "New login detected from unknown device.",
        "ui-gallery-item-icon",
    );

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(max_w_lg),
        |_cx| vec![item],
    )
    .test_id("ui-gallery-item-icon-wrapper")
}
// endregion: example
