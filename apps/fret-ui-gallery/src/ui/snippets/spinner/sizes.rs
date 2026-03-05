pub const SOURCE: &str = include_str!("sizes.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let size_3 = shadcn::Spinner::new()
        .refine_layout(LayoutRefinement::default().w_px(Px(12.0)).h_px(Px(12.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-sizes-3");
    let size_4 = shadcn::Spinner::new()
        .into_element(cx)
        .test_id("ui-gallery-spinner-sizes-4");
    let size_6 = shadcn::Spinner::new()
        .refine_layout(LayoutRefinement::default().w_px(Px(24.0)).h_px(Px(24.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-sizes-6");
    let size_8 = shadcn::Spinner::new()
        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-sizes-8");

    ui::h_flex(|_cx| vec![size_3, size_4, size_6, size_8])
        .gap(Space::N6)
        .items_center()
        .layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(520.0)),
        )
        .into_element(cx)
        .test_id("ui-gallery-spinner-sizes")
}

// endregion: example
