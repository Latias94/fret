pub const SOURCE: &str = include_str!("icon.rs");

// region: example
use fret::UiCx;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn icon(cx: &mut UiCx<'_>, id: &'static str) -> impl IntoUiElement<fret_app::App> + use<> {
    fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
}

fn item_icon(
    cx: &mut UiCx<'_>,
    icon_id: &'static str,
    title: &'static str,
    description: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let media = shadcn::ItemMedia::new([icon(cx, icon_id).into_element(cx)])
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

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
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
    )
    .into_element(cx);

    ui::v_stack(|_cx| vec![item])
        .gap(Space::N6)
        .items_start()
        .layout(max_w_lg)
        .into_element(cx)
        .test_id("ui-gallery-item-icon-wrapper")
}
// endregion: example
