pub const SOURCE: &str = include_str!("list.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    value: &'static str,
) -> AnyElement {
    ui::h_flex(|cx| {
        vec![
            shadcn::raw::typography::small(cx, label),
            shadcn::raw::typography::muted(cx, value),
        ]
    })
    .justify_between()
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
        vec![
            row(cx, "Item 1", "Value 1"),
            shadcn::Separator::new().into_element(cx),
            row(cx, "Item 2", "Value 2"),
            shadcn::Separator::new().into_element(cx),
            row(cx, "Item 3", "Value 3"),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(384.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-separator-list")
}
// endregion: example
