pub const SOURCE: &str = include_str!("vertical.rs");

// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    shadcn::Tabs::uncontrolled(Some(Arc::<str>::from("preview")))
        .orientation(shadcn::tabs::TabsOrientation::Vertical)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.app-window")),
            shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.code")),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-vertical")
}

// endregion: example
