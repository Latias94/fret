pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2).items_start(),
                |cx| {
                    vec![
                        shadcn::typography::h3(cx, "RTL Sample"),
                        shadcn::typography::p(
                            cx,
                            "This block validates right-to-left direction in typography surfaces.",
                        ),
                        shadcn::typography::muted(
                            cx,
                            "Check paragraph wrapping and heading alignment under RTL.",
                        ),
                    ]
                },
            )
        },
    )
    .test_id("ui-gallery-typography-rtl")
}
// endregion: example
