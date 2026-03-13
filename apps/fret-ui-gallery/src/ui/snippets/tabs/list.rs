pub const SOURCE: &str = include_str!("list.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::tabs_uncontrolled(cx, Some("home"), |_cx| {
        [
            shadcn::TabsItem::new("home", "Home", Vec::<AnyElement>::new()),
            shadcn::TabsItem::new("settings", "Settings", Vec::<AnyElement>::new()),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
    .into_element(cx)
    .test_id("ui-gallery-tabs-list")
}

// endregion: example
