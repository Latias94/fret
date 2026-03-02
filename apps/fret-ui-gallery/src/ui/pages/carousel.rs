use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::carousel as snippets;

pub(super) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.carousel_page", |cx| {
        let demo = snippets::demo::render(cx);
        let loop_carousel = snippets::loop_carousel::render(cx);
        let basic = snippets::basic::render(cx);
        let parts = snippets::parts::render(cx);
        let sizes = snippets::sizes::render(cx);
        let spacing = snippets::spacing::render(cx);
        let duration = snippets::duration_embla::render(cx);
        let api = snippets::api::render(cx);
        let plugin = snippets::plugin_autoplay::render(cx);
        let expandable = snippets::expandable::render(cx);
        let orientation_vertical = snippets::orientation_vertical::render(cx);

        let notes_stack = doc_layout::notes(
            cx,
            [
                "Preview follows shadcn Carousel demo: Basic, Sizes, and Spacing.",
                "The upstream demo uses responsive item widths (`md:basis-1/2` / `lg:basis-1/3`). Fret uses a fixed `item_basis_main_px` to keep geometry deterministic in native builds.",
                "Spacing parity depends on pairing `track_start_neg_margin` with `item_padding_start`.",
            ],
        );

        let body = doc_layout::render_doc_page(
            cx,
            Some("Preview follows shadcn Carousel demo cards (Fret builder API; not Embla)."),
            vec![
                DocSection::new("Demo", demo)
                    .description("A carousel with 5 items and previous/next buttons.")
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-demo")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/demo.rs"),
                        "example",
                    ),
                DocSection::new("Loop", loop_carousel)
                    .description(
                        "Seamless looping (`loop=true`) using the Embla-style headless engine.",
                    )
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-loop")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/loop_carousel.rs"),
                        "example",
                    ),
                DocSection::new("Basic", basic)
                    .description("Default slide width (basis-full).")
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-basic")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/basic.rs"),
                        "example",
                    ),
                DocSection::new("Parts", parts)
                    .description("Part-based authoring surface aligned with shadcn/ui v4 exports.")
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-parts")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/parts.rs"),
                        "example",
                    ),
                DocSection::new("Sizes", sizes)
                    .description("Three active items (`basis-1/3`) to mirror the docs layout.")
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-sizes")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/sizes.rs"),
                        "example",
                    ),
                DocSection::new("Spacing", spacing)
                    .description(
                        "Tighter track negative margin + item start padding (shadcn `-ml-1` / `pl-1`).",
                    )
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-spacing")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/spacing.rs"),
                        "example",
                    ),
                DocSection::new("Duration (Embla)", duration)
                    .description("Embla `duration` (integrator parameter) affects settle speed for button navigation (this demo ignores prefers-reduced-motion).")
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-duration")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/duration_embla.rs"),
                        "example",
                    ),
                DocSection::new("API", api)
                    .description("A carousel with a slide counter (shadcn `setApi`-style outcome).")
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-api")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/api.rs"),
                        "example",
                    ),
                DocSection::new("Plugin (Autoplay)", plugin)
                    .description("Autoplay: 2000ms delay; hover pauses; interaction stops.")
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-plugin")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/plugin_autoplay.rs"),
                        "example",
                    ),
                DocSection::new("Expandable", expandable)
                    .description("Content-driven height changes (used by the motion pilot suite).")
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-expandable")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/expandable.rs"),
                        "example",
                    ),
                DocSection::new("Orientation (Vertical)", orientation_vertical)
                    .description("A vertical carousel (orientation=\"vertical\").")
                    .max_w(Px(760.0))
                    .test_id_prefix("ui-gallery-carousel-orientation-vertical")
                    .code_rust_from_file_region(
                        include_str!("../snippets/carousel/orientation_vertical.rs"),
                        "example",
                    ),
                DocSection::new("Notes", notes_stack).max_w(Px(760.0)),
            ],
        );

        vec![body.test_id("ui-gallery-carousel-component")]
    })
}

