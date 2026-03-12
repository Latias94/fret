use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::carousel as snippets;

pub(super) fn preview_carousel(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.carousel_page", |cx| {
        let demo = snippets::demo::render(cx);
        let loop_carousel = snippets::loop_carousel::render(cx);
        let loop_downgrade_cannot_loop = snippets::loop_downgrade_cannot_loop::render(cx);
        let focus = snippets::focus_watch::render(cx);
        let basic = snippets::basic::render(cx);
        let usage = snippets::usage::render(cx);
        let parts = snippets::parts::render(cx);
        let sizes_thirds = snippets::sizes_thirds::render(cx);
        let sizes = snippets::sizes::render(cx);
        let spacing = snippets::spacing::render(cx);
        let spacing_responsive = snippets::spacing_responsive::render(cx);
        let duration = snippets::duration_embla::render(cx);
        let options = snippets::options::render(cx);
        let api = snippets::api::render(cx);
        let events = snippets::events::render(cx);
        let plugin = snippets::plugin_autoplay::render(cx);
        let plugin_controlled = snippets::plugin_autoplay_controlled::render(cx);
        let plugin_stop_on_focus = snippets::plugin_autoplay_stop_on_focus::render(cx);
        let plugin_stop_on_last_snap = snippets::plugin_autoplay_stop_on_last_snap::render(cx);
        let plugin_delays = snippets::plugin_autoplay_delays::render(cx);
        let plugin_wheel = snippets::plugin_wheel_gestures::render(cx);
        let expandable = snippets::expandable::render(cx);
        let orientation_vertical = snippets::orientation_vertical::render(cx);
        let rtl = snippets::rtl::render(cx);

        let about = doc_layout::notes(
            cx,
            [
                "Upstream shadcn Carousel is built on Embla; Fret mirrors the authoring outcomes with an Embla-style headless engine plus a compact builder and a parts surface.",
                "The upstream demo uses responsive item widths (`md:basis-1/2` / `lg:basis-1/3`). Fret mirrors this via `CarouselItem::viewport_layout_breakpoint(tailwind::MD/LG, ...)`.",
                "Spacing parity depends on pairing `track_start_neg_margin` with `item_padding_start` (shadcn `-ml-*` + `pl-*`).",
                "The extra sections below exist to keep engine + diagnostics coverage visible (loop downgrade, focus watch, duration, wheel gestures, etc.).",
            ],
        );

        let api_reference = doc_layout::notes(
            cx,
            [
                "API reference: `ecosystem/fret-ui-shadcn/src/carousel.rs`.",
                "`Carousel::new/items` is the compact builder path, while `CarouselContent`, `CarouselItem`, `CarouselPrevious`, and `CarouselNext` are exposed through `Carousel::into_element_parts(...)` for upstream-shaped copyable examples.",
                "Carousel chrome, buttons, and the Embla-style headless behaviors stay recipe-owned; surrounding width/height negotiation and breakpoint choices remain caller-owned.",
                "No extra generic `compose()` / children surface is needed here because the parts authoring surface already maps to the upstream exports.",
            ],
        );

        let body = doc_layout::render_doc_page(
            cx,
            Some(
                "Preview mirrors the shadcn Carousel docs path first: Demo, About, Usage, Examples (Basic/Sizes/Spacing/Orientation), Options, API, Events, Plugins, RTL, then keeps engine/diagnostics follow-ups explicit before `API Reference`.",
            ),
            vec![
                DocSection::new("Demo", demo)
                    .description("A carousel with 5 items and previous/next buttons.")

                    .test_id_prefix("ui-gallery-carousel-demo")
                    .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
                DocSection::new("About", about)
                    .description("Background, ownership notes, and why extra diagnostics sections exist.")
                    .no_shell()
                    .test_id_prefix("ui-gallery-carousel-about"),
                DocSection::new("Usage", usage)
                    .description("Copyable minimal usage for the builder + parts authoring surface.")
                    .test_id_prefix("ui-gallery-carousel-usage")
                    .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
                DocSection::new("Parts", parts)
                    .description("Part-based authoring surface aligned with shadcn/ui v4 exports.")

                    .test_id_prefix("ui-gallery-carousel-parts")
                    .code_rust_from_file_region(snippets::parts::SOURCE, "example"),
                DocSection::new("Basic", basic)
                    .description("A minimal baseline (basis-full) used by docs parity screenshots.")
                    .test_id_prefix("ui-gallery-carousel-basic")
                    .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
                DocSection::new("Sizes (1/3)", sizes_thirds)
                    .description("Fixed slide width (shadcn `basis-1/3`).")
                    .test_id_prefix("ui-gallery-carousel-sizes-thirds")
                    .code_rust_from_file_region(snippets::sizes_thirds::SOURCE, "example"),
                DocSection::new("Sizes", sizes)
                    .description("Three active items (`basis-1/3`) to mirror the docs layout.")

                    .test_id_prefix("ui-gallery-carousel-sizes")
                    .code_rust_from_file_region(snippets::sizes::SOURCE, "example"),
                DocSection::new("Spacing", spacing)
                    .description("Tighter track negative margin + item start padding (shadcn `-ml-1` / `pl-1`).")

                    .test_id_prefix("ui-gallery-carousel-spacing")
                    .code_rust_from_file_region(snippets::spacing::SOURCE, "example"),
                DocSection::new("Spacing (Responsive)", spacing_responsive)
                    .description(
                        "Viewport breakpoints for spacing (shadcn `-ml-2 md:-ml-4` / `pl-2 md:pl-4`).",
                    )

                    .test_id_prefix("ui-gallery-carousel-spacing-responsive")
                    .code_rust_from_file_region(
                        snippets::spacing_responsive::SOURCE,
                        "example",
                    ),
                DocSection::new("Orientation (Vertical)", orientation_vertical)
                    .description("A vertical carousel (orientation=\"vertical\").")

                    .test_id_prefix("ui-gallery-carousel-orientation-vertical")
                    .code_rust_from_file_region(snippets::orientation_vertical::SOURCE, "example"),
                DocSection::new("Options", options)
                    .description("Pass options via `opts` (Embla-style): `align=start`, `loop=true`.")
                    .test_id_prefix("ui-gallery-carousel-options")
                    .code_rust_from_file_region(snippets::options::SOURCE, "example"),
                DocSection::new("API", api)
                    .description("A carousel with a slide counter (shadcn `setApi`-style outcome).")
                    .test_id_prefix("ui-gallery-carousel-api")
                    .code_rust_from_file_region(snippets::api::SOURCE, "example"),
                DocSection::new("Events", events)
                    .description("Listen to select/reInit events (shadcn `api.on(...)`-style outcomes).")
                    .test_id_prefix("ui-gallery-carousel-events")
                    .code_rust_from_file_region(snippets::events::SOURCE, "example"),
                DocSection::new("Plugin (Autoplay)", plugin)
                    .description("Autoplay: 2000ms delay; hover pauses; interaction stops.")
                    .test_id_prefix("ui-gallery-carousel-plugin")
                    .code_rust_from_file_region(snippets::plugin_autoplay::SOURCE, "example"),
                DocSection::new("Plugin (Autoplay, Controlled)", plugin_controlled)
                    .description(
                        "Autoplay with external stop/reset/play controls (Embla plugin-style outcomes).",
                    )
                    .test_id_prefix("ui-gallery-carousel-plugin-controlled")
                    .code_rust_from_file_region(
                        snippets::plugin_autoplay_controlled::SOURCE,
                        "example",
                    ),
                DocSection::new("Plugin (Autoplay, stopOnInteraction via focus)", plugin_stop_on_focus)
                    .description(
                        "Autoplay stops when focus enters a slide (`stop_on_interaction=true`, Embla `slidefocus`-style outcome).",
                    )
                    .test_id_prefix("ui-gallery-carousel-plugin-stop-on-interaction-focus")
                    .code_rust_from_file_region(
                        snippets::plugin_autoplay_stop_on_focus::SOURCE,
                        "example",
                    ),
                DocSection::new("Plugin (Autoplay, stopOnLastSnap)", plugin_stop_on_last_snap)
                    .description(
                        "Autoplay stops after reaching the last snap (`stop_on_last_snap=true`).",
                    )
                    .test_id_prefix("ui-gallery-carousel-plugin-stop-on-last-snap")
                    .code_rust_from_file_region(
                        snippets::plugin_autoplay_stop_on_last_snap::SOURCE,
                        "example",
                    ),
                DocSection::new("Plugin (Autoplay, per-snap delays)", plugin_delays)
                    .description("Autoplay delay can be varied per snap (`set_delays`).")
                    .test_id_prefix("ui-gallery-carousel-plugin-delays")
                    .code_rust_from_file_region(snippets::plugin_autoplay_delays::SOURCE, "example"),
                DocSection::new("Plugin (Wheel gestures)", plugin_wheel)
                    .description(
                        "Wheel/trackpad gestures: horizontal scroll steps between snaps (Shift swaps axes).",
                    )
                    .test_id_prefix("ui-gallery-carousel-plugin-wheel")
                    .code_rust_from_file_region(snippets::plugin_wheel_gestures::SOURCE, "example"),
                DocSection::new("RTL", rtl)
                    .description(
                        "RTL carousel: set `DirectionProvider` and `CarouselOptions::direction(Rtl)` (shadcn `dir` + `opts.direction`). Prev/Next remain physically left/right (shadcn docs), while arrow direction adapts to RTL.",
                    )

                    .test_id_prefix("ui-gallery-carousel-rtl")
                    .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
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
                DocSection::new("Duration (Embla)", duration)
                    .description(
                        "Embla `duration` (integrator parameter) affects settle speed for button navigation (this demo ignores prefers-reduced-motion).",
                    )
                    .test_id_prefix("ui-gallery-carousel-duration")
                    .code_rust_from_file_region(snippets::duration_embla::SOURCE, "example"),
                DocSection::new("Expandable", expandable)
                    .description("Content-driven height changes (used by the motion pilot suite).")
                    .test_id_prefix("ui-gallery-carousel-expandable")
                    .code_rust_from_file_region(snippets::expandable::SOURCE, "example"),
                DocSection::new("API Reference", api_reference)
                    .description("Public surface summary and ownership notes.")
                    .no_shell()
                    .test_id_prefix("ui-gallery-carousel-api-reference"),
            ],
        );

        vec![body.test_id("ui-gallery-carousel-component")]
    })
}
