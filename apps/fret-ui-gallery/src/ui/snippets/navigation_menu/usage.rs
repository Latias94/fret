pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(|| None::<Arc<str>>);

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
