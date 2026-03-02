// region: example
use crate::spec::CMD_APP_OPEN;
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

fn icon(cx: &mut ElementContext<'_, App>, id: &'static str) -> AnyElement {
    shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(448.0)));

    let row_a = shadcn::Item::new([
        shadcn::ItemContent::new([
            shadcn::ItemTitle::new("Visit our documentation").into_element(cx),
            shadcn::ItemDescription::new("Learn how to get started with our components.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::ItemActions::new([icon(cx, "lucide.chevron-right")]).into_element(cx),
    ])
    .render(shadcn::ItemRender::Link {
        href: Arc::<str>::from("https://example.com/docs"),
        target: None,
        rel: None,
    })
    .on_click(CMD_APP_OPEN)
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-item-link-row-a");

    let row_b = shadcn::Item::new([
        shadcn::ItemContent::new([
            shadcn::ItemTitle::new("External resource").into_element(cx),
            shadcn::ItemDescription::new("Opens in a new tab with security attributes.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::ItemActions::new([icon(cx, "lucide.external-link")]).into_element(cx),
    ])
    .variant(shadcn::ItemVariant::Outline)
    .render(shadcn::ItemRender::Link {
        href: Arc::<str>::from("https://example.com/external"),
        target: Some(Arc::<str>::from("_blank")),
        rel: Some(Arc::<str>::from("noopener noreferrer")),
    })
    .on_click(CMD_APP_OPEN)
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-item-link-row-b");

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(max_w_md),
        |_cx| vec![row_a, row_b],
    )
    .test_id("ui-gallery-item-link")
}
// endregion: example
