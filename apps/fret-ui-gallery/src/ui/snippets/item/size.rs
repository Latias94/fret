pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(448.0)));

    let icon = |cx: &mut UiCx<'_>, id: &'static str| {
        let icon_id = fret_icons::IconId::new_static(id);
        match id {
            "lucide.badge-check" => icon::icon_with(cx, icon_id, Some(Px(20.0)), None),
            _ => icon::icon(cx, icon_id),
        }
    };

    let outline = shadcn::ItemVariant::Outline;

    let item_default = shadcn::item_sized(cx, shadcn::ItemSize::Default, |cx| {
        let action = shadcn::Button::new("Action")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx);
        [
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("Default Size").into_element(cx),
                shadcn::ItemDescription::new("The standard size for most use cases.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::ItemActions::new([action]).into_element(cx),
        ]
    })
    .variant(outline)
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-item-size-default");

    let item_sm = shadcn::item_sized(cx, shadcn::ItemSize::Sm, |cx| {
        let media = shadcn::ItemMedia::new([icon(cx, "lucide.badge-check")])
            .into_element(cx)
            .test_id("ui-gallery-item-size-sm-media");
        let content = shadcn::ItemContent::new([
            shadcn::ItemTitle::new("Small Size").into_element(cx),
            shadcn::ItemDescription::new("A compact size for dense layouts.").into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-item-size-sm-content");
        let actions = shadcn::ItemActions::new([icon(cx, "lucide.chevron-right")])
            .into_element(cx)
            .test_id("ui-gallery-item-size-sm-actions");

        [media, content, actions]
    })
    .variant(outline)
    .render(shadcn::ItemRender::Link {
        href: Arc::<str>::from("https://example.com/profile"),
        target: None,
        rel: None,
    })
    .on_click(CMD_APP_OPEN)
    .a11y_label("Verified profile")
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-item-size-sm");

    let item_xs = shadcn::item_sized(cx, shadcn::ItemSize::Xs, |cx| {
        let media = shadcn::ItemMedia::new([icon(cx, "lucide.inbox")])
            .variant(shadcn::ItemMediaVariant::Icon)
            .into_element(cx)
            .test_id("ui-gallery-item-size-xs-media");
        let content = shadcn::ItemContent::new([
            shadcn::ItemTitle::new("Extra Small Size").into_element(cx),
            shadcn::ItemDescription::new("The most compact size available.").into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-item-size-xs-content");

        [media, content]
    })
    .variant(outline)
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-item-size-xs");

    ui::v_stack(|_cx| vec![item_default, item_sm, item_xs])
        .gap(Space::N6)
        .items_start()
        .layout(max_w_md)
        .into_element(cx)
        .test_id("ui-gallery-item-size")
}
// endregion: example
