pub const SOURCE: &str = include_str!("list.rs");

// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    shadcn::Tabs::uncontrolled(Some(Arc::<str>::from("home")))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("home", "Home", Vec::<AnyElement>::new()),
            shadcn::TabsItem::new("settings", "Settings", Vec::<AnyElement>::new()),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-list")
}

// endregion: example
