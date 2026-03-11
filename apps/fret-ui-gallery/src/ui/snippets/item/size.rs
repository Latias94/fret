pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(448.0)));

    let icon = |cx: &mut ElementContext<'_, App>, id: &'static str| {
        let icon_id = fret_icons::IconId::new_static(id);
        match id {
            "lucide.badge-check" => {
                fret_ui_shadcn::icon::icon_with(cx, icon_id, Some(Px(20.0)), None)
            }
            _ => fret_ui_shadcn::icon::icon(cx, icon_id),
        }
    };

    let outline = shadcn::ItemVariant::Outline;

    let item_default = {
        let action = shadcn::Button::new("Action")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx);
        shadcn::Item::new([
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("Default Size").into_element(cx),
                shadcn::ItemDescription::new("The standard size for most use cases.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::ItemActions::new([action]).into_element(cx),
        ])
        .variant(outline)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-item-size-default")
    };

    let item_sm = {
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

        shadcn::Item::new([media, content, actions])
            .variant(outline)
            .size(shadcn::ItemSize::Sm)
            .render(shadcn::ItemRender::Link {
                href: Arc::<str>::from("https://example.com/profile"),
                target: None,
                rel: None,
            })
            .on_click(CMD_APP_OPEN)
            .a11y_label("Verified profile")
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-size-sm")
    };

    let item_xs = {
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

        shadcn::Item::new([media, content])
            .variant(outline)
            .size(shadcn::ItemSize::Xs)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-size-xs")
    };

    ui::v_stack(|_cx| vec![item_default, item_sm, item_xs])
        .gap(Space::N6)
        .items_start()
        .layout(max_w_md)
        .into_element(cx)
        .test_id("ui-gallery-item-size")
}
// endregion: example
