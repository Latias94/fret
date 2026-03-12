pub const SOURCE: &str = include_str!("menu.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: &'static str,
    description: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let _ = cx;
    ui::v_stack(move |cx| {
        vec![
            shadcn::raw::typography::small(title).into_element(cx),
            shadcn::raw::typography::muted(description).into_element(cx),
        ]
    })
    .gap(Space::N1)
    .items_start()
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_flex(|cx| {
        vec![
            section(cx, "Settings", "Manage preferences").into_element(cx),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .flex_stretch_cross_axis(true)
                .into_element(cx),
            section(cx, "Account", "Profile & security").into_element(cx),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .flex_stretch_cross_axis(true)
                .refine_layout(LayoutRefinement::default().h_px(Px(32.0)))
                .into_element(cx),
            section(cx, "Help", "Support & docs").into_element(cx),
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
