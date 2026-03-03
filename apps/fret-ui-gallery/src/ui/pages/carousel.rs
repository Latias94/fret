use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::carousel as snippets;

pub(super) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.carousel_page", |cx| {
        let demo = snippets::demo::render(cx);
        let loop_carousel = snippets::loop_carousel::render(cx);
        let loop_downgrade_cannot_loop = snippets::loop_downgrade_cannot_loop::render(cx);
        let focus = snippets::focus_watch::render(cx);
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

                    .test_id_prefix("ui-gallery-carousel-demo")
                    .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
                DocSection::new("Loop", loop_carousel)
                    .description("Seamless looping (`loop=true`) using the Embla-style headless engine.")

                    .test_id_prefix("ui-gallery-carousel-loop")
                    .code_rust_from_file_region(snippets::loop_carousel::SOURCE, "example"),
                DocSection::new("Loop downgrade (cannotLoop)", loop_downgrade_cannot_loop)
                    .description(
                        "Requested `loop=true` but the slide set cannot loop; Embla downgrades to non-loop behavior.",
                    )

                    .test_id_prefix("ui-gallery-carousel-loop-downgrade-cannot-loop")
                    .code_rust_from_file_region(
                        snippets::loop_downgrade_cannot_loop::SOURCE,
                        "example",
                    ),
                DocSection::new("Focus", focus)
                    .description(
                        "`watch_focus=true`: Tab into an offscreen slide and scroll it into view (Embla engine enabled).",
                    )

                    .test_id_prefix("ui-gallery-carousel-focus")
                    .code_rust_from_file_region(snippets::focus_watch::SOURCE, "example"),
                DocSection::new("Basic", basic)
                    .description("Default slide width (basis-full).")

                    .test_id_prefix("ui-gallery-carousel-basic")
                    .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
                DocSection::new("Parts", parts)
                    .description("Part-based authoring surface aligned with shadcn/ui v4 exports.")

                    .test_id_prefix("ui-gallery-carousel-parts")
                    .code_rust_from_file_region(snippets::parts::SOURCE, "example"),
                DocSection::new("Sizes", sizes)
                    .description("Three active items (`basis-1/3`) to mirror the docs layout.")

                    .test_id_prefix("ui-gallery-carousel-sizes")
                    .code_rust_from_file_region(snippets::sizes::SOURCE, "example"),
                DocSection::new("Spacing", spacing)
                    .description("Tighter track negative margin + item start padding (shadcn `-ml-1` / `pl-1`).")

                    .test_id_prefix("ui-gallery-carousel-spacing")
                    .code_rust_from_file_region(snippets::spacing::SOURCE, "example"),
                DocSection::new("Duration (Embla)", duration)
                    .description("Embla `duration` (integrator parameter) affects settle speed for button navigation (this demo ignores prefers-reduced-motion).")

                    .test_id_prefix("ui-gallery-carousel-duration")
                    .code_rust_from_file_region(snippets::duration_embla::SOURCE, "example"),
                DocSection::new("API", api)
                    .description("A carousel with a slide counter (shadcn `setApi`-style outcome).")

                    .test_id_prefix("ui-gallery-carousel-api")
                    .code_rust_from_file_region(snippets::api::SOURCE, "example"),
                DocSection::new("Plugin (Autoplay)", plugin)
                    .description("Autoplay: 2000ms delay; hover pauses; interaction stops.")

                    .test_id_prefix("ui-gallery-carousel-plugin")
                    .code_rust_from_file_region(snippets::plugin_autoplay::SOURCE, "example"),
                DocSection::new("Expandable", expandable)
                    .description("Content-driven height changes (used by the motion pilot suite).")

                    .test_id_prefix("ui-gallery-carousel-expandable")
                    .code_rust_from_file_region(snippets::expandable::SOURCE, "example"),
                DocSection::new("Orientation (Vertical)", orientation_vertical)
                    .description("A vertical carousel (orientation=\"vertical\").")

                    .test_id_prefix("ui-gallery-carousel-orientation-vertical")
                    .code_rust_from_file_region(snippets::orientation_vertical::SOURCE, "example"),
                DocSection::new("Notes", notes_stack),
            ],
        );

        vec![body.test_id("ui-gallery-carousel-component")]
    })
}
