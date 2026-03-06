use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::toggle_group as snippets;

pub(super) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let outline = snippets::outline::render(cx);
    let single = snippets::single::render(cx);
    let small = snippets::small::render(cx);
    let large = snippets::large::render(cx);
    let label = snippets::label::render(cx);
    let spacing = snippets::spacing::render(cx);
    let disabled = snippets::disabled::render(cx);
    let vertical = snippets::vertical::render(cx);
    let full_width_items = snippets::full_width_items::render(cx);
    let stretch = snippets::flex_1_items::render(cx);
    let rtl = snippets::rtl::render(cx);
    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/toggle_group.rs` and `ecosystem/fret-ui-shadcn/src/toggle.rs`.",
            "Use Single mode for mutually-exclusive options (alignment, list/grid/cards).",
            "Use Multiple mode for formatting toggles where users may combine states.",
            "Use `ToggleGroupItem::style(...)` when a docs recipe needs per-item selected accents.",
            "For icon-only groups, keep explicit `a11y_label` for assistive technologies.",
            "Rust does not need a DOM-style root `children` slot here: `ToggleGroup::item(s)` plus `ToggleGroupItem::new(..., children)` is the typed equivalent, and `ToggleGroupItem::child(...)` now covers incremental composition.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview starts with a compact overview, then mirrors the shadcn Toggle Group docs order: Usage, Outline, Single, Small, Large, Disabled, Spacing. Fret-only regression sections follow.",
        ),
        vec![
            DocSection::new("Preview", demo)
                .description("Compact icon-only preview for quick visual scanning.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Minimal typed usage matching the upstream docs example.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Outline", outline)
                .description("Outline variant matching the shadcn toolbar example.")
                .code_rust_from_file_region(snippets::outline::SOURCE, "example"),
            DocSection::new("Single", single)
                .description("Single-selection icon group, matching the upstream example.")
                .code_rust_from_file_region(snippets::single::SOURCE, "example"),
            DocSection::new("Small", small)
                .description("Small size preset for compact formatting toolbars.")
                .code_rust_from_file_region(snippets::small::SOURCE, "example"),
            DocSection::new("Large", large)
                .description("Large size preset for roomier touch-friendly controls.")
                .code_rust_from_file_region(snippets::large::SOURCE, "example"),
            DocSection::new("Label Association", label)
                .description("Use `FieldLabel::for_control`, `ToggleGroup::control_id`, and `ToggleGroup::test_id_prefix` to keep label-focus behavior and automation anchors aligned.")
                .test_id_prefix("ui-gallery-toggle-group-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disabled groups keep layout but block interaction.")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Spacing", spacing)
                .description(
                    "Per-item selected accents with explicit spacing, closer to the docs recipe.",
                )
                .code_rust_from_file_region(snippets::spacing::SOURCE, "example"),
            DocSection::new("Vertical", vertical)
                .description("Vertical orientation for side panels / inspectors.")
                .code_rust_from_file_region(snippets::vertical::SOURCE, "example"),
            DocSection::new("Full Width Items", full_width_items)
                .description("Stretched items (`flex-1`) to gate control-chrome fill invariants.")
                .code_rust_from_file_region(snippets::full_width_items::SOURCE, "example"),
            DocSection::new("Flex-1 Items", stretch)
                .description("Regression gate for hit and visual alignment under `flex-1` sizing.")
                .code_rust_from_file_region(snippets::flex_1_items::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Item ordering and pressed visuals under RTL.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes."),
        ],
    );

    vec![body.test_id("ui-gallery-toggle-group")]
}
