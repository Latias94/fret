use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::button as snippets;

pub(super) fn preview_button(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let size = snippets::size::render(cx);
    let default = snippets::default::render(cx);
    let outline = snippets::outline::render(cx);
    let secondary = snippets::secondary::render(cx);
    let ghost = snippets::ghost::render(cx);
    let destructive = snippets::destructive::render(cx);
    let link = snippets::link::render(cx);
    let icon_only = snippets::icon::render(cx);
    let with_icon = snippets::with_icon::render(cx);
    let rounded = snippets::rounded::render(cx);
    let spinner = snippets::loading::render(cx);
    let button_group = snippets::button_group::render(cx);
    let link_render = snippets::link_render::render(cx);
    let rtl = snippets::rtl::render(cx);
    let variants = snippets::variants::render(cx);

    let cursor = doc_layout::notes(
        cx,
        [
            "The upstream cursor note is Tailwind CSS-specific (`cursor: default` vs `cursor: pointer`).",
            "Fret is self-drawn, so this exact CSS footgun does not apply one-to-one to the button recipe.",
            "If host-specific pointer cursor behavior needs adjustment, treat it as runtime / pressable policy, not a `Button` default-style change.",
        ],
    );

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Button::new(label)` plus `variant(...)` covers the documented `default`, `outline`, `secondary`, `ghost`, `destructive`, and `link` recipe surface.",
            "`size(...)` covers `default`, `xs`, `sm`, `lg`, `icon`, `icon-xs`, `icon-sm`, and `icon-lg`.",
            "`leading_children(...)` / `trailing_children(...)` are the preferred Fret equivalent of upstream `data-icon=\"inline-start|inline-end\"` child composition for dynamic affordances such as `Spinner`.",
            "`children(...)` remains the full content override when you want to replace the entire inner row on purpose.",
            "`ButtonRender::Link` is the Fret equivalent of the second upstream `Link` section: semantic link rendering stays button-owned instead of widening the public surface with a generic `asChild`/`compose()` API.",
            "Intrinsic chrome stays recipe-owned; page-level width, wrapping, `flex-1`, `min-w-0`, and fully-rounded one-off examples stay caller-owned refinements.",
        ],
    );

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/button.rs`.",
            "Gallery sections now mirror shadcn Button docs first: Demo, Usage, Cursor, Size, Default, Outline, Secondary, Ghost, Destructive, Link, Icon, With Icon, Rounded, Spinner, Button Group, Link (semantic), RTL, API Reference.",
            "`Variants Overview (Fret)` stays after the upstream path so existing variant chrome diagnostics remain easy to compare without displacing the docs order.",
            "The main parity fix here is recipe/public-surface work: logical inline child slots now cover spinner/icon compositions without widening `Button` into a generic `asChild` surface.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Button docs order first, then keeps a compact Fret-specific variants overview for fast visual comparison and existing diag coverage.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Docs-aligned outline button + icon button preview.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Button`.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Cursor", cursor)
                .no_shell()
                .description("Translate the upstream Tailwind cursor note into Fret ownership terms."),
            DocSection::new("Size", size)
                .description("Use `size(...)` to change text and icon button sizes.")
                .code_rust_from_file_region(snippets::size::SOURCE, "example"),
            DocSection::new("Default", default)
                .code_rust_from_file_region(snippets::default::SOURCE, "example"),
            DocSection::new("Outline", outline)
                .code_rust_from_file_region(snippets::outline::SOURCE, "example"),
            DocSection::new("Secondary", secondary)
                .code_rust_from_file_region(snippets::secondary::SOURCE, "example"),
            DocSection::new("Ghost", ghost)
                .code_rust_from_file_region(snippets::ghost::SOURCE, "example"),
            DocSection::new("Destructive", destructive)
                .code_rust_from_file_region(snippets::destructive::SOURCE, "example"),
            DocSection::new("Link", link)
                .description("The documented link variant remains a button-styled text action.")
                .code_rust_from_file_region(snippets::link::SOURCE, "example"),
            DocSection::new("Icon", icon_only)
                .description("Icon-only outline button.")
                .code_rust_from_file_region(snippets::icon::SOURCE, "example"),
            DocSection::new("With Icon", with_icon)
                .description("Leading and trailing icon patterns.")
                .code_rust_from_file_region(snippets::with_icon::SOURCE, "example"),
            DocSection::new("Rounded", rounded)
                .description("Keep `rounded-full` as an explicit call-site refinement.")
                .code_rust_from_file_region(snippets::rounded::SOURCE, "example"),
            DocSection::new("Spinner", spinner)
                .description("Disabled button compositions with inline spinners.")
                .code_rust_from_file_region(snippets::loading::SOURCE, "example"),
            DocSection::new("Button Group", button_group)
                .description("See `ButtonGroup` for grouped actions with shared chrome.")
                .code_rust_from_file_region(snippets::button_group::SOURCE, "example"),
            DocSection::new("Link (Semantic)", link_render)
                .description("Fret equivalent of the upstream second `Link` example (`buttonVariants` on a semantic link).")
                .code_rust_from_file_region(snippets::link_render::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Button layout under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .description("Public surface summary and ownership notes."),
            DocSection::new("Variants Overview (Fret)", variants)
                .description("Compact variant comparison kept for diagnostics and quick visual diffing.")
                .code_rust_from_file_region(snippets::variants::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .no_shell()
                .description("Parity notes and implementation pointers."),
        ],
    );

    vec![body.test_id("ui-gallery-button")]
}
