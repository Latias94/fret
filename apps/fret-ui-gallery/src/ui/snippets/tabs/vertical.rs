pub const SOURCE: &str = include_str!("vertical.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::tabs_uncontrolled(cx, Some("account"), |_cx| {
        [
            shadcn::TabsItem::new("account", "Account", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-vertical-trigger-account"),
            shadcn::TabsItem::new("password", "Password", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-vertical-trigger-password"),
            shadcn::TabsItem::new("notifications", "Notifications", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-vertical-trigger-notifications"),
        ]
    })
    .orientation(shadcn::raw::tabs::TabsOrientation::Vertical)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
    .test_id("ui-gallery-tabs-vertical")
    .into_element(cx)
}

// endregion: example
