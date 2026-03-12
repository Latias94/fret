pub const SOURCE: &str = include_str!("link_component.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(|| None::<Arc<str>>);

    shadcn::NavigationMenu::new(value)
        .list(shadcn::NavigationMenuList::new([
            shadcn::NavigationMenuItem::new("docs", "Documentation", std::iter::empty())
                .href("https://example.com/docs")
                .target("_blank")
                .rel("noopener noreferrer")
                // Keep the gallery deterministic: show the link authoring surface without actually
                // launching the browser during scripted runs.
                .on_click("ui_gallery.app.open")
                .trigger_test_id("ui-gallery-navigation-menu-link-component-docs"),
        ]))
        .into_element(cx)
}
// endregion: example
