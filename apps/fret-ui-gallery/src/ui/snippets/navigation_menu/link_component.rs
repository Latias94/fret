pub const SOURCE: &str = include_str!("link_component.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(|| None::<Arc<str>>);

    shadcn::navigation_menu(cx, value, |_cx| {
        [
            shadcn::NavigationMenuItem::new("docs", "Documentation", std::iter::empty())
                .href("https://example.com/docs")
                // Keep the gallery deterministic: show the link authoring surface without actually
                // launching the browser during scripted runs.
                .action("ui_gallery.app.open")
                .trigger_test_id("ui-gallery-navigation-menu-link-component-docs"),
        ]
    })
    .into_element(cx)
}
// endregion: example
