pub const SOURCE: &str = include_str!("icons.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::tabs_uncontrolled(cx, Some("preview"), |cx| {
        [
            shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new())
                .trigger_children([
                    icon::icon(cx, IconId::new_static("lucide.app-window")),
                    decl_text::text_button_label(cx, "Preview"),
                ])
                .trigger_test_id("ui-gallery-tabs-icons-trigger-preview"),
            shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new())
                .trigger_children([
                    icon::icon(cx, IconId::new_static("lucide.code")),
                    decl_text::text_button_label(cx, "Code"),
                ])
                .trigger_test_id("ui-gallery-tabs-icons-trigger-code"),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
    .into_element(cx)
    .test_id("ui-gallery-tabs-icons")
}

// endregion: example
