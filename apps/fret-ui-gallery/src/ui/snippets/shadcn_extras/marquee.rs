pub const SOURCE: &str = include_str!("marquee.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::extras::Marquee::new(["Alpha", "Beta", "Gamma", "Delta", "Epsilon"])
        .pause_on_hover(true)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-marquee")
}
// endregion: example
