pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home").href("/"),
            shadcn::BreadcrumbItem::new("Components").href("/components"),
            shadcn::BreadcrumbItem::new("Breadcrumb").disabled(true),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-usage")
}
// endregion: example
