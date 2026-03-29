use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::separator as snippets;

pub(super) fn preview_separator(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let vertical = snippets::vertical::render(cx);
    let menu = snippets::menu::render(cx);
    let list = snippets::list::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "Docs path: `repo-ref/ui/apps/v4/content/docs/components/base/separator.mdx`; recipe sources: `repo-ref/ui/apps/v4/registry/bases/base/ui/separator.tsx` and `repo-ref/ui/apps/v4/registry/bases/radix/ui/separator.tsx`; headless references: `repo-ref/primitives/packages/react/separator/src/separator.tsx` and `repo-ref/base-ui/packages/react/src/separator/Separator.tsx`.",
        "`fret_ui_kit::primitives::separator::Separator` owns the mechanism layer: orientation, separator semantics, and decorative hiding. `fret_ui_shadcn::Separator` owns the recipe defaults (`shrink-0`, 1px rule chrome, and the vertical self-stretch mapping).",
        "`Separator::new()`, `Separator::orientation(...)`, and `Separator::decorative(...)` cover the public surface Fret needs for shadcn/Radix/Base UI parity. Fret keeps the Radix-aligned `.decorative(...)` knob on the shadcn lane even though the current Base UI docs axis does not surface that prop explicitly.",
        "Vertical recipe parity maps upstream `data-vertical:self-stretch` to `align-self: stretch` plus auto cross-axis sizing, so surrounding row height remains caller-owned while the separator still stretches correctly in docs-style flex rows.",
        "No generic composable children / `compose()` / `asChild` surface is warranted here because separator is a leaf primitive. Base UI's `render` prop is a tag-swap seam, not a content-owned children API.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary and ownership notes.");

    let demo = DocSection::build(cx, "Demo", demo)
        .description("Official top-of-page preview from the current Base docs path.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for the default separator; use `.decorative(false)` when the divider should be announced.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let vertical = DocSection::build(cx, "Vertical", vertical)
        .description("Use `orientation=vertical`; the shadcn surface already carries the upstream self-stretch default.")
        .code_rust_from_file_region(snippets::vertical::SOURCE, "example");
    let menu = DocSection::build(cx, "Menu", menu)
        .description("Responsive menu example: the trailing Help section and second divider appear from `md` upward, matching the docs composition.")
        .code_rust_from_file_region(snippets::menu::SOURCE, "example");
    let list = DocSection::build(cx, "List", list)
        .description("Horizontal separators between list items.")
        .code_rust_from_file_region(snippets::list::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Separator layout should hold under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the current shadcn Base Separator docs path first: Demo, Usage, Vertical, Menu, List, RTL, and API Reference. The API Reference also calls out the Radix/Base UI split behind `decorative(...)`.",
        ),
        vec![demo, usage, vertical, menu, list, rtl, api_reference],
    );

    vec![body.test_id("ui-gallery-separator").into_element(cx)]
}
