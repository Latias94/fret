use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::toggle_group as snippets;

pub(super) fn preview_toggle_group(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let outline = snippets::outline::render(cx);
    let size = snippets::size::render(cx);
    let spacing = snippets::spacing::render(cx);
    let vertical = snippets::vertical::render(cx);
    let disabled = snippets::disabled::render(cx);
    let custom = snippets::custom::render(cx);
    let rtl = snippets::rtl::render(cx);
    let single = snippets::single::render(cx);
    let small = snippets::small::render(cx);
    let large = snippets::large::render(cx);
    let label = snippets::label::render(cx);
    let full_width_items = snippets::full_width_items::render(cx);
    let stretch = snippets::flex_1_items::render(cx);

    let api_reference = doc_layout::notes(
        cx,
        [
            "`ToggleGroup::single(...)`, `ToggleGroup::multiple(...)`, and their uncontrolled constructors cover the documented single/multiple selection paths.",
            "`ToggleGroupItem::new(..., children)`, `child(...)`, and `children(...)` remain the source-aligned content surface; no extra generic `compose()` API is needed here.",
            "`ToggleGroupItem::refine_layout(...)` and `refine_style(...)` now cover upstream custom item-root sizing and rounding without moving caller-owned recipe tweaks into the default component chrome.",
            "Selection semantics, roving focus, segmented borders, and pressed-state chrome remain recipe-owned; item-root custom layout and surrounding width/flex negotiation remain caller-owned.",
            "`Single`, `Small`, `Large`, `Label Association`, `Full Width Items`, and `Flex-1 Items` stay after the upstream docs path as focused Fret follow-ups and regression slices.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Toggle Group docs path first: Demo, Usage, Outline, Size, Spacing, Vertical, Disabled, Custom, RTL, and API Reference. Focused Fret follow-ups stay afterward.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Compact icon-only preview for quick visual scanning.")
                .test_id_prefix("ui-gallery-toggle-group-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Minimal typed usage matching the upstream docs example.")
                .test_id_prefix("ui-gallery-toggle-group-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Outline", outline)
                .description("Outline variant matching the shadcn example.")
                .test_id_prefix("ui-gallery-toggle-group-outline")
                .code_rust_from_file_region(snippets::outline::SOURCE, "example"),
            DocSection::new("Size", size)
                .description("Size presets for compact and roomier toggle groups.")
                .test_id_prefix("ui-gallery-toggle-group-size")
                .code_rust_from_file_region(snippets::size::SOURCE, "example"),
            DocSection::new("Spacing", spacing)
                .description("Spacing between items with per-item selected accents.")
                .test_id_prefix("ui-gallery-toggle-group-spacing")
                .code_rust_from_file_region(snippets::spacing::SOURCE, "example"),
            DocSection::new("Vertical", vertical)
                .description("Vertical orientation for side panels and inspectors.")
                .test_id_prefix("ui-gallery-toggle-group-vertical")
                .code_rust_from_file_region(snippets::vertical::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disabled groups keep layout but block interaction.")
                .test_id_prefix("ui-gallery-toggle-group-disabled")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Custom", custom)
                .description("Custom item-root sizing and rounding for a font-weight selector.")
                .test_id_prefix("ui-gallery-toggle-group-custom")
                .code_rust_from_file_region(snippets::custom::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Item ordering and pressed visuals under RTL.")
                .test_id_prefix("ui-gallery-toggle-group-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .test_id_prefix("ui-gallery-toggle-group-api-reference")
                .description("Public surface summary and ownership notes."),
            DocSection::new("Single (Fret)", single)
                .description("Focused single-selection regression example.")
                .test_id_prefix("ui-gallery-toggle-group-single")
                .code_rust_from_file_region(snippets::single::SOURCE, "example"),
            DocSection::new("Small (Fret)", small)
                .description("Small icon-only regression slice retained after the docs path.")
                .test_id_prefix("ui-gallery-toggle-group-small")
                .code_rust_from_file_region(snippets::small::SOURCE, "example"),
            DocSection::new("Large (Fret)", large)
                .description("Large icon-only regression slice retained after the docs path.")
                .test_id_prefix("ui-gallery-toggle-group-large")
                .code_rust_from_file_region(snippets::large::SOURCE, "example"),
            DocSection::new("Label Association (Fret)", label)
                .description("Use `FieldLabel::for_control`, `ToggleGroup::control_id`, and `test_id_prefix` to keep label-focus behavior and automation anchors aligned.")
                .test_id_prefix("ui-gallery-toggle-group-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
            DocSection::new("Full Width Items (Fret)", full_width_items)
                .description("Stretched items to gate full-row fill behavior.")
                .test_id_prefix("ui-gallery-toggle-group-full-width-items")
                .code_rust_from_file_region(snippets::full_width_items::SOURCE, "example"),
            DocSection::new("Flex-1 Items (Fret)", stretch)
                .description("Regression gate for hit and visual alignment under `flex-1` sizing.")
                .test_id_prefix("ui-gallery-toggle-group-stretch")
                .code_rust_from_file_region(snippets::flex_1_items::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-toggle-group")]
}
