// region: example
use crate::spec::CMD_APP_OPEN;
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(448.0)));

    let icon = |cx: &mut ElementContext<'_, App>, id: &'static str| {
        shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
    };

    let outline = shadcn::ItemVariant::Outline;

    let item_basic = {
        let action = shadcn::Button::new("Action")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx);
        shadcn::Item::new([
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("Basic Item").into_element(cx),
                shadcn::ItemDescription::new("A simple item with title and description.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::ItemActions::new([action]).into_element(cx),
        ])
        .variant(outline)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-item-docs-demo-basic")
    };

    let item_sm_link = {
        let media = shadcn::ItemMedia::new([icon(cx, "lucide.badge-check")])
            .into_element(cx)
            .test_id("ui-gallery-item-docs-demo-sm-link-media");
        let content =
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("Your profile has been verified.").into_element(cx)
            ])
            .into_element(cx)
            .test_id("ui-gallery-item-docs-demo-sm-link-content");
        let actions = shadcn::ItemActions::new([icon(cx, "lucide.chevron-right")])
            .into_element(cx)
            .test_id("ui-gallery-item-docs-demo-sm-link-actions");

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
            .test_id("ui-gallery-item-docs-demo-sm-link")
    };

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(max_w_md),
        |_cx| vec![item_basic, item_sm_link],
    )
    .test_id("ui-gallery-item-demo")
}
// endregion: example
