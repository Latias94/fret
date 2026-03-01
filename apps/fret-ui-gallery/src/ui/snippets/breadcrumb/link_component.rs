// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let trunc_layout = LayoutRefinement::default().max_w(Px(112.0));

    shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home")
                .href("https://example.com")
                // Keep this example deterministic under automation by default.
                // Remove to allow `Effect::OpenUrl` fallback.
                .on_activate(Arc::new(|_host, _acx, _reason| {}))
                .truncate(true)
                .refine_layout(trunc_layout),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-link")
}
// endregion: example

