use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::badge as snippets;

pub(super) fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let usage = snippets::usage::render(cx);
    let demo = snippets::demo::render(cx);
    let variants = snippets::variants::render(cx);
    let with_icon = snippets::icon::render(cx);
    let with_spinner = snippets::spinner::render(cx);
    let link = snippets::link::render(cx);
    let colors = snippets::colors::render(cx);
    let rtl = snippets::rtl::render(cx);
    let counts = snippets::counts::render(cx);

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Badge::new(label)` and `variant(...)` cover the documented `default`, `secondary`, `destructive`, `outline`, `ghost`, and `link` recipe surface.",
            "Link composition uses badge-owned render semantics (`BadgeRender::Link`) instead of a generic `asChild` merge surface; this matches the upstream outcome without widening the mechanism layer.",
            "Icons, invalid state, and custom color overrides stay on the badge recipe surface; page-level width negotiation remains caller-owned.",
        ],
    );

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/badge.rs`.",
            "Gallery sections now mirror shadcn Badge docs first: Demo, Usage, Variants, With Icon, With Spinner, Link, Custom Colors, RTL, API Reference.",
            "Badge already exposes the important recipe surface, so the remaining parity work is page/docs clarity rather than new composition APIs.",
            "`Counts (Fret)` is intentionally left after the upstream path to preserve existing diag coverage for compact numeric badges without polluting the docs-aligned demo.",
            "The Link render example installs a no-op `on_activate` so diag scripts do not launch a system browser; remove it to enable the default `Effect::OpenUrl` fallback.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Badge docs order first, then appends a small Fret-specific `Counts` section to keep numeric badge diagnostics stable.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Docs-aligned badge preview with the four primary variants.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Badge`.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Variants", variants)
                .description("Use the `variant` prop to change the badge variant.")
                .code_rust_from_file_region(snippets::variants::SOURCE, "example"),
            DocSection::new("With Icon", with_icon)
                .description(
                    "Render an icon inside the badge (inline-start / inline-end patterns).",
                )
                .code_rust_from_file_region(snippets::icon::SOURCE, "example"),
            DocSection::new("With Spinner", with_spinner)
                .description("Render a spinner inside the badge (useful for loading states).")
                .code_rust_from_file_region(snippets::spinner::SOURCE, "example"),
            DocSection::new("Link", link)
                .description("Badges can be composed with link semantics (upstream `render` / `asChild` outcome).")
                .code_rust_from_file_region(snippets::link::SOURCE, "example"),
            DocSection::new("Custom Colors", colors)
                .description("Customize badge colors with explicit background + text overrides.")
                .code_rust_from_file_region(snippets::colors::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Render the badge under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .description("Public surface summary and ownership notes."),
            DocSection::new("Counts (Fret)", counts)
                .description("Compact numeric badges kept as a Fret-specific follow-up for diag coverage.")
                .code_rust_from_file_region(snippets::counts::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .no_shell()
                .description("API reference pointers and caveats."),
        ],
    );

    vec![body]
}
