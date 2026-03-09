pub const SOURCE: &str = include_str!("link_component.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    value: Option<Model<Option<Arc<str>>>>,
}

fn value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = value_model(cx);

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
