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
        let compact_builder = snippets::compact_builder::render(cx);
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

        let about = doc_layout::notes_block([
                "Upstream shadcn Carousel is built on Embla; Fret mirrors the authoring outcomes with an Embla-style headless engine plus a compact builder and a parts surface.",
                "The upstream demo uses responsive item widths (`md:basis-1/2` / `lg:basis-1/3`). Fret mirrors this via `CarouselItem::viewport_layout_breakpoint(tailwind::MD/LG, ...)`.",
                "Spacing parity depends on pairing `track_start_neg_margin` with `item_padding_start` (shadcn `-ml-*` + `pl-*`).",
                "The follow-up sections below keep the Fret shorthand and the engine-facing regression harnesses visible without changing the upstream docs path.",
            ]);
        let fret_follow_ups = doc_layout::notes_block([
                "The upstream shadcn docs stop after `RTL`; the sections below stay on the page to document Fret-specific authoring shortcuts and regression harnesses without diluting that docs path.",
                "`Basic` remains a gallery follow-up baseline preview because the upstream docs jump straight from `Usage` into the `Sizes` examples instead of showing a separate single-slide baseline section.",
                "`Plugin (Autoplay, Controlled)`, `Plugin (Autoplay, stopOnInteraction via focus)`, `Plugin (Autoplay, stopOnLastSnap)`, `Plugin (Autoplay, per-snap delays)`, and `Plugin (Wheel gestures)` remain follow-ups because the upstream docs only show the base autoplay plugin example.",
                "`Compact Builder` keeps `Carousel::new(items)` visible for app code, `Parts` keeps the explicit adapter/diagnostics seam visible, and `Loop` is a dedicated `loop=true` preview that the upstream docs only imply through `Options`.",
                "`Loop downgrade`, `Focus`, `Duration`, and `Expandable` remain gallery follow-ups because they primarily exist to keep Embla-style engine and motion regressions reviewable.",
            ]);

        let api_reference = doc_layout::notes_block([
                "API reference: `ecosystem/fret-ui-shadcn/src/carousel.rs`.",
                "`CarouselContent`, `CarouselItem`, `CarouselPrevious`, and `CarouselNext` keep the upstream-shaped copyable surface that matches the shadcn docs `Usage` story, while `Carousel::new/items` stays as the compact Fret shorthand for app call sites.",
                "When the parts are already assembled eagerly, prefer `Carousel::into_element_parts_content(...)`; keep `into_element_parts(...)` for seams that genuinely need `cx` before landing.",
                "`Usage` now mirrors the upstream docs-shaped parts lane, `Compact Builder` keeps the ergonomic Fret shorthand visible, and `Parts` remains the explicit adapter/diagnostics seam on that same copyable lane rather than an advanced escape hatch.",
                "The docs-path examples below (`Sizes`, `Spacing`, `Orientation`, `Options`) and the docs-aligned previews (`Demo`, `API`, base autoplay plugin, `RTL`) still stay on the compact builder lane unless a snippet explicitly needs control-level parts or diagnostics-specific control IDs.",
                "Docs-path snippets keep the upstream `w_full().max_w(...)` width lane on the carousel root itself; diagnostics follow-ups may switch to fixed-width shells (`w_px(...)`) when deterministic control geometry matters more than copyable docs parity.",
                "For the common `setApi` docs outcome (slide counter / can-prev / can-next status), prefer `api_snapshot_model(...)`; reserve `api_handle_model(...)` + `CarouselEventCursor` for explicit event-stream examples such as `Events`.",
                "Carousel chrome, buttons, and the Embla-style headless behaviors stay recipe-owned; surrounding width/height negotiation and breakpoint choices remain caller-owned.",
                "We intentionally do not add a generic heterogeneous children API here: the upstream-shaped usage lane plus the compact shorthand already cover the shadcn authoring shapes without reopening another default teaching surface.",
            ]);
        let about = DocSection::build(cx, "About", about)
            .description("Background, ownership notes, and why extra diagnostics sections exist.")
            .no_shell()
            .test_id_prefix("ui-gallery-carousel-about");
        let fret_follow_ups = DocSection::build(cx, "Fret Follow-ups", fret_follow_ups)
            .description(
                "Why the page continues after the upstream docs path and how the follow-up sections are grouped.",
            )
            .no_shell()
            .test_id_prefix("ui-gallery-carousel-follow-ups");
        let api_reference = DocSection::build(cx, "API Reference", api_reference)
            .description("Public surface summary and ownership notes.")
            .no_shell()
            .test_id_prefix("ui-gallery-carousel-api-reference");
        let demo = DocSection::build(cx, "Demo", demo)
            .description("A carousel with 5 items and previous/next buttons.")
            .test_id_prefix("ui-gallery-carousel-demo")
            .code_rust_from_file_region(snippets::demo::SOURCE, "example");
        let usage = DocSection::build(cx, "Usage", usage)
            .description(
                "Upstream shadcn docs shape using `CarouselContent`, `CarouselItem`, `CarouselPrevious`, and `CarouselNext`.",
            )
            .test_id_prefix("ui-gallery-carousel-usage")
            .code_rust_from_file_region(snippets::usage::SOURCE, "example");
        let compact_builder = DocSection::build(cx, "Compact Builder", compact_builder)
            .description("Compact Fret shorthand for common app call sites: `Carousel::new(items)`.")
            .test_id_prefix("ui-gallery-carousel-compact-builder")
            .code_rust_from_file_region(snippets::compact_builder::SOURCE, "example");
        let parts = DocSection::build(cx, "Parts", parts)
            .description(
                "Focused adapter example on the same upstream-shaped lane when callers want explicit part values or diagnostics-specific control IDs.",
            )
            .test_id_prefix("ui-gallery-carousel-parts")
            .code_rust_from_file_region(snippets::parts::SOURCE, "example");
        let basic = DocSection::build(cx, "Basic", basic)
            .description("A minimal baseline (basis-full) on the compact builder lane.")
            .test_id_prefix("ui-gallery-carousel-basic")
            .code_rust_from_file_region(snippets::basic::SOURCE, "example");
        let sizes_thirds = DocSection::build(cx, "Sizes (1/3)", sizes_thirds)
            .description("Fixed slide width (shadcn `basis-1/3`) on the compact builder lane.")
            .test_id_prefix("ui-gallery-carousel-sizes-thirds")
            .code_rust_from_file_region(snippets::sizes_thirds::SOURCE, "example");
        let sizes = DocSection::build(cx, "Sizes", sizes)
            .description("Three active items (`basis-1/3`) to mirror the docs layout on the compact builder lane.")
            .test_id_prefix("ui-gallery-carousel-sizes")
            .code_rust_from_file_region(snippets::sizes::SOURCE, "example");
        let spacing = DocSection::build(cx, "Spacing", spacing)
            .description(
                "Tighter track negative margin + item start padding (shadcn `-ml-1` / `pl-1`) on the compact builder lane.",
            )
            .test_id_prefix("ui-gallery-carousel-spacing")
            .code_rust_from_file_region(snippets::spacing::SOURCE, "example");
        let spacing_responsive =
            DocSection::build(cx, "Spacing (Responsive)", spacing_responsive)
                .description(
                    "Viewport breakpoints for spacing (shadcn `-ml-2 md:-ml-4` / `pl-2 md:pl-4`) on the compact builder lane.",
                )
                .test_id_prefix("ui-gallery-carousel-spacing-responsive")
                .code_rust_from_file_region(snippets::spacing_responsive::SOURCE, "example");
        let orientation_vertical =
            DocSection::build(cx, "Orientation (Vertical)", orientation_vertical)
                .description("A vertical carousel (orientation=\"vertical\") on the compact builder lane.")
                .test_id_prefix("ui-gallery-carousel-orientation-vertical")
                .code_rust_from_file_region(snippets::orientation_vertical::SOURCE, "example");
        let options = DocSection::build(cx, "Options", options)
            .description("Pass options via `opts` (Embla-style): `align=start`, `loop=true`, still on the compact builder lane.")
            .test_id_prefix("ui-gallery-carousel-options")
            .code_rust_from_file_region(snippets::options::SOURCE, "example");
        let api = DocSection::build(cx, "API", api)
            .description("A carousel with a slide counter using the compact snapshot surface (`api_snapshot_model`) for the common shadcn `setApi`-style docs outcome.")
            .test_id_prefix("ui-gallery-carousel-api")
            .code_rust_from_file_region(snippets::api::DOCS_SOURCE, "example");
        let events = DocSection::build(cx, "Events", events)
            .description("Listen to select/reInit events (`api_handle_model` + `CarouselEventCursor`, equivalent to shadcn `api.on(...)`); the status rows make the example self-verifying in Gallery, and the named controls stay only for diagnostics.")
            .test_id_prefix("ui-gallery-carousel-events")
            .code_rust_from_file_region(snippets::events::DOCS_SOURCE, "example");
        let plugin = DocSection::build(cx, "Plugin (Autoplay)", plugin)
            .description("Autoplay docs parity: `plugins=[Autoplay(...)]` plus hover pause/reset; the self-drawn example maps the docs' DOM hover handlers onto a hover region while staying on the compact builder lane.")
            .test_id_prefix("ui-gallery-carousel-plugin")
            .code_rust_from_file_region(snippets::plugin_autoplay::DOCS_SOURCE, "example");
        let plugin_controlled =
            DocSection::build(cx, "Plugin (Autoplay, Controlled)", plugin_controlled)
                .description(
                    "Autoplay with external stop/reset/play controls (Embla plugin-style outcomes) on the compact builder lane.",
                )
                .test_id_prefix("ui-gallery-carousel-plugin-controlled")
                .code_rust_from_file_region(
                    snippets::plugin_autoplay_controlled::SOURCE,
                    "example",
                );
        let plugin_stop_on_focus =
            DocSection::build(cx, "Plugin (Autoplay, stopOnInteraction via focus)", plugin_stop_on_focus)
                .description(
                    "Autoplay stops when focus enters a slide (`stop_on_interaction=true`, Embla `slidefocus`-style outcome) on the compact builder lane.",
                )
                .test_id_prefix("ui-gallery-carousel-plugin-stop-on-interaction-focus")
                .code_rust_from_file_region(
                    snippets::plugin_autoplay_stop_on_focus::SOURCE,
                    "example",
                );
        let plugin_stop_on_last_snap =
            DocSection::build(cx, "Plugin (Autoplay, stopOnLastSnap)", plugin_stop_on_last_snap)
                .description(
                    "Autoplay stops after reaching the last snap (`stop_on_last_snap=true`) on the compact builder lane.",
                )
                .test_id_prefix("ui-gallery-carousel-plugin-stop-on-last-snap")
                .code_rust_from_file_region(
                    snippets::plugin_autoplay_stop_on_last_snap::SOURCE,
                    "example",
                );
        let plugin_delays =
            DocSection::build(cx, "Plugin (Autoplay, per-snap delays)", plugin_delays)
                .description("Autoplay delay can be varied per snap (`set_delays`) on the compact builder lane.")
                .test_id_prefix("ui-gallery-carousel-plugin-delays")
                .code_rust_from_file_region(
                    snippets::plugin_autoplay_delays::SOURCE,
                    "example",
                );
        let plugin_wheel = DocSection::build(cx, "Plugin (Wheel gestures)", plugin_wheel)
            .description(
                "Wheel/trackpad gestures: horizontal scroll steps between snaps (Shift swaps axes) on the compact builder lane.",
            )
            .test_id_prefix("ui-gallery-carousel-plugin-wheel")
            .code_rust_from_file_region(snippets::plugin_wheel_gestures::SOURCE, "example");
        let rtl = DocSection::build(cx, "RTL", rtl)
            .description(
                "RTL carousel: keep `DirectionProvider` and `CarouselOptions::direction(Rtl)` aligned (shadcn docs `dir` + `opts.direction`). Prev/Next remain physically left/right, while arrow direction adapts to RTL; named controls stay only for diagnostics.",
            )
            .test_id_prefix("ui-gallery-carousel-rtl")
            .code_rust_from_file_region(snippets::rtl::DOCS_SOURCE, "example");
        let loop_carousel = DocSection::build(cx, "Loop", loop_carousel)
            .description(
                "Dedicated `loop=true` preview kept after the docs path so the looping behavior stays directly reviewable on the compact builder lane.",
            )
            .test_id_prefix("ui-gallery-carousel-loop")
            .code_rust_from_file_region(snippets::loop_carousel::SOURCE, "example");
        let loop_downgrade_cannot_loop = DocSection::build_diagnostics(
            cx,
            "Loop downgrade (cannotLoop)",
            loop_downgrade_cannot_loop,
        )
                .description(
                    "Engine follow-up: requested `loop=true` but the slide set cannot loop, so Embla downgrades to non-loop behavior.",
                )
                .test_id_prefix("ui-gallery-carousel-loop-downgrade-cannot-loop")
                .code_rust_from_file_region(
                    snippets::loop_downgrade_cannot_loop::SOURCE,
                    "example",
                );
        let focus = DocSection::build_diagnostics(cx, "Focus", focus)
            .description(
                "Engine follow-up: `watch_focus=true` tabs into an offscreen slide and scrolls it into view (Embla engine enabled) on the compact builder lane.",
            )
            .test_id_prefix("ui-gallery-carousel-focus")
            .code_rust_from_file_region(snippets::focus_watch::SOURCE, "example");
        let duration = DocSection::build_diagnostics(cx, "Duration (Embla)", duration)
            .description(
                "Engine follow-up: Embla `duration` (integrator parameter) affects settle speed for button navigation; this demo ignores prefers-reduced-motion on purpose.",
            )
            .test_id_prefix("ui-gallery-carousel-duration")
            .code_rust_from_file_region(snippets::duration_embla::SOURCE, "example");
        let expandable = DocSection::build_diagnostics(cx, "Expandable", expandable)
            .description("Motion follow-up: content-driven height changes used by the motion pilot suite on the compact builder lane.")
            .test_id_prefix("ui-gallery-carousel-expandable")
            .code_rust_from_file_region(snippets::expandable::SOURCE, "example");

        let body = doc_layout::render_doc_page(
            cx,
            Some(
                "Preview mirrors the shadcn Carousel docs path first: Demo, About, Usage, Examples (Sizes/Spacing/Orientation), Options, API, Events, Plugin, RTL. After that, Gallery keeps Fret-only follow-ups explicit: `Fret Follow-ups`, `Basic`, extra plugin variants, `Compact Builder`, `Parts`, a dedicated `Loop` preview, engine/motion diagnostics, then `API Reference`.",
            ),
            vec![
                demo,
                about,
                usage,
                sizes_thirds,
                sizes,
                spacing,
                spacing_responsive,
                orientation_vertical,
                options,
                api,
                events,
                plugin,
                rtl,
                fret_follow_ups,
                basic,
                plugin_controlled,
                plugin_stop_on_focus,
                plugin_stop_on_last_snap,
                plugin_delays,
                plugin_wheel,
                compact_builder,
                parts,
                loop_carousel,
                loop_downgrade_cannot_loop,
                focus,
                duration,
                expandable,
                api_reference,
            ],
        );

        vec![body.test_id("ui-gallery-carousel-component").into_element(cx)]
    })
}
