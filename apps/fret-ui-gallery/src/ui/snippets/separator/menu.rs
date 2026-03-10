pub const SOURCE: &str = include_str!("menu.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: &'static str,
    description: &'static str,
) -> AnyElement {
    ui::v_stack(|cx| {
        vec![
            shadcn::typography::small(cx, title),
            shadcn::typography::muted(cx, description),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_flex(|cx| {
        vec![
            section(cx, "Settings", "Manage preferences"),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .flex_stretch_cross_axis(true)
                .into_element(cx),
            section(cx, "Account", "Profile & security"),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .flex_stretch_cross_axis(true)
                .refine_layout(LayoutRefinement::default().h_px(Px(32.0)))
                .into_element(cx),
            section(cx, "Help", "Support & docs"),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .wrap()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(520.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-separator-menu")
}
// endregion: example
