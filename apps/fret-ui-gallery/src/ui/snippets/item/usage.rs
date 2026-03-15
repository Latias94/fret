pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let media = shadcn::ItemMedia::new([icon::icon(
        cx,
        fret_icons::IconId::new_static("lucide.inbox"),
    )])
    .variant(shadcn::ItemMediaVariant::Icon)
    .into_element(cx);

    let content = shadcn::ItemContent::new([
        shadcn::ItemTitle::new("Inbox").into_element(cx),
        shadcn::ItemDescription::new("Review new messages and pending updates.").into_element(cx),
    ])
    .into_element(cx);

    let actions = shadcn::ItemActions::new([shadcn::Button::new("Action")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)])
    .into_element(cx);

    shadcn::Item::new([media, content, actions])
        .variant(shadcn::ItemVariant::Outline)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(448.0)))
        .into_element(cx)
        .test_id("ui-gallery-item-usage")
}
// endregion: example
