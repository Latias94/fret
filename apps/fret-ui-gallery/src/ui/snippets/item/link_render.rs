pub const SOURCE: &str = include_str!("link_render.rs");

// region: example
use fret::UiCx;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";

fn icon(cx: &mut UiCx<'_>, id: &'static str) -> AnyElement {
    fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let media = shadcn::ItemMedia::new([icon(cx, "lucide.house")])
        .variant(shadcn::ItemMediaVariant::Icon)
        .into_element(cx);
    let content = shadcn::ItemContent::new([
        shadcn::ItemTitle::new("Dashboard").into_element(cx),
        shadcn::ItemDescription::new("Overview of your account and activity.").into_element(cx),
    ])
    .into_element(cx);
    let actions = shadcn::ItemActions::new([icon(cx, "lucide.chevron-right")]).into_element(cx);

    shadcn::Item::new([media, content, actions])
        .render(shadcn::ItemRender::Link {
            href: Arc::<str>::from("https://example.com/dashboard"),
            target: None,
            rel: None,
        })
        .on_click(CMD_APP_OPEN)
        .a11y_label("Dashboard")
        .test_id("ui-gallery-item-link-render")
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
}
// endregion: example
