pub const SOURCE: &str = include_str!("usage.rs");

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

    let overview = shadcn::NavigationMenuItem::new(
        "overview",
        "Overview",
        [ui::text("Overview content").into_element(cx)],
    );
    let docs = shadcn::NavigationMenuItem::new("docs", "Docs", std::iter::empty());

    shadcn::NavigationMenu::new(value)
        .list(shadcn::NavigationMenuList::new(vec![overview, docs]))
        .into_element(cx)
}
// endregion: example
