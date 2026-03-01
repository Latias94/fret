// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Breadcrumb::new()
        .separator(shadcn::BreadcrumbSeparator::Icon {
            icon: fret_icons::IconId::new_static("lucide.dot"),
            size: Px(14.0),
        })
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-separator")
}
// endregion: example
