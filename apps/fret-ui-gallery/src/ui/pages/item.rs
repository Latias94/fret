use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::item as snippets;

pub(super) fn preview_item(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let variants = snippets::variants::render(cx);
    let size = snippets::size::render(cx);
    let icon = snippets::icon::render(cx);
    let avatar = snippets::avatar::render(cx);
    let image = snippets::image::render(cx);
    let group = snippets::group::render(cx);
    let header = snippets::header::render(cx);
    let link = snippets::link::render(cx);
    let dropdown = snippets::dropdown::render(cx);
    let rtl = snippets::extras_rtl::render(cx);
    let gallery = snippets::gallery::render(cx);
    let link_render = snippets::link_render::render(cx);

    let item_vs_field = doc_layout::muted_full_width(
        cx,
        "Use `Field` when the row owns a form control such as a checkbox, input, radio, or select. Use `Item` when the row only presents media, title, description, and actions.",
    );

    let api_reference = doc_layout::notes_block([
        "`Item::new([...])` plus `ItemMedia`, `ItemContent`, `ItemTitle`, `ItemDescription`, `ItemActions`, `ItemGroup`, and `ItemHeader` matches the upstream slot model directly, while `item_sized(...)` and `item_group(...)` are the preferred thin helpers on first-party teaching surfaces.",
        "`ItemRender::Link` is the Fret equivalent of the upstream `render={<a ... />}` pattern and keeps link semantics on the pressable root rather than burying them in a nested child.",
        "Intrinsic item chrome, slot spacing, and size presets remain recipe-owned because the upstream component source defines those defaults on the item itself.",
        "Caller-owned layout stays explicit for `max-w-*`, grid placement, page columns, and broader list composition. The recipe should not absorb those negotiation rules.",
        "No extra generic `asChild` / `compose()` surface is needed here: the existing slot parts and `ItemRender::Link` already cover the documented composition model.",
        "`ItemSize::Xs` is already supported in Fret and is now shown explicitly in the gallery size example.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary and ownership notes.");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Item docs path first: Demo, Usage, Item vs Field, Variant, Size, the example set through RTL, then keeps `API Reference`, `Gallery`, and `Link (render)` as Fret follow-ups.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .no_shell()
                .description("Top-of-page item preview matching the upstream docs intent.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .no_shell()
                .description("Copyable minimal usage for `Item` and its slot parts.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Item vs Field", item_vs_field)
                .no_shell()
                .description(
                    "Choose `Item` for presentation rows and `Field` for input-bearing rows.",
                ),
            DocSection::new("Variant", variants)
                .description("Default, outline, and muted variants.")
                .no_shell()
                .code_rust_from_file_region(snippets::variants::SOURCE, "example"),
            DocSection::new("Size", size)
                .description("`default`, `sm`, and `xs` item sizing; the size-scoped gallery lane uses `item_sized(...)`.")
                .no_shell()
                .code_rust_from_file_region(snippets::size::SOURCE, "example"),
            DocSection::new("Icon", icon)
                .no_shell()
                .code_rust_from_file_region(snippets::icon::SOURCE, "example"),
            DocSection::new("Avatar", avatar)
                .no_shell()
                .code_rust_from_file_region(snippets::avatar::SOURCE, "example"),
            DocSection::new("Image", image)
                .no_shell()
                .code_rust_from_file_region(snippets::image::SOURCE, "example"),
            DocSection::new("Group", group)
                .description("Grouped item rows using the thin `item_group(...)` helper.")
                .no_shell()
                .code_rust_from_file_region(snippets::group::SOURCE, "example"),
            DocSection::new("Header", header)
                .no_shell()
                .code_rust_from_file_region(snippets::header::SOURCE, "example"),
            DocSection::new("Link", link)
                .description(
                    "Links are modeled via `ItemRender::Link` so the root carries link semantics.",
                )
                .no_shell()
                .code_rust_from_file_region(snippets::link::SOURCE, "example"),
            DocSection::new("Dropdown", dropdown)
                .description("Item composed inside a dropdown menu row.")
                .no_shell()
                .code_rust_from_file_region(snippets::dropdown::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("RTL smoke check for the item recipe and slot ordering.")
                .no_shell()
                .code_rust_from_file_region(snippets::extras_rtl::SOURCE, "example"),
            api_reference,
            DocSection::new("Gallery", gallery)
                .description(
                    "Extended regression coverage snapshot: columns plus mixed compositions.",
                )
                .no_shell()
                .max_w(Px(1100.0)),
            DocSection::new("Link (render)", link_render)
                .description("A gallery-focused deterministic link row example.")
                .no_shell()
                .code_rust_from_file_region(snippets::link_render::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-item")]
}
