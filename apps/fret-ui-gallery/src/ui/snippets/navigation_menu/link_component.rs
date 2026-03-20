pub const SOURCE: &str = include_str!("link_component.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(|| None::<Arc<str>>);
    let trigger = ui::h_row(|cx| {
        vec![
            cx.text("Documentation"),
            icon::icon(cx, fret_icons::IconId::new_static("lucide.chevron-right")),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    shadcn::navigation_menu(cx, value, |_cx| {
        [
            shadcn::NavigationMenuItem::new("docs", "Documentation", std::iter::empty())
                .href("https://example.com/docs")
                .target("_blank")
                .rel("noopener noreferrer")
                // `trigger_child(...)` is the Fret-side equivalent of composing a custom
                // top-level link child without introducing a DOM-specific `asChild` API.
                .trigger_child(trigger)
                // Keep the gallery deterministic: show the link authoring surface without actually
                // launching the browser during scripted runs.
                .action("ui_gallery.app.open")
                .trigger_test_id("ui-gallery-navigation-menu-link-component-docs"),
        ]
    })
    .into_element(cx)
}
// endregion: example
