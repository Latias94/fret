pub const SOURCE: &str = include_str!("icons.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::tabs_uncontrolled(cx, Some("preview"), |cx| {
        [
            shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new())
                .trigger_children([
                    icon::icon(cx, IconId::new_static("lucide.app-window")),
                    cx.text("Preview"),
                ])
                .trigger_test_id("ui-gallery-tabs-icons-trigger-preview"),
            shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new())
                .trigger_children([
                    icon::icon(cx, IconId::new_static("lucide.code")),
                    cx.text("Code"),
                ])
                .trigger_test_id("ui-gallery-tabs-icons-trigger-code"),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
    .into_element(cx)
    .test_id("ui-gallery-tabs-icons")
}

// endregion: example
