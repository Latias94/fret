pub const SOURCE: &str = include_str!("line.rs");

// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    shadcn::Tabs::uncontrolled(Some(Arc::<str>::from("preview")))
        .list_variant(shadcn::TabsListVariant::Line)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .test_id("ui-gallery-tabs-line")
        .items([
            shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.app-window"))
                .trigger_test_id("ui-gallery-tabs-line-trigger-preview"),
            shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.code"))
                .trigger_test_id("ui-gallery-tabs-line-trigger-code"),
        ])
        .into_element(cx)
}

// endregion: example
