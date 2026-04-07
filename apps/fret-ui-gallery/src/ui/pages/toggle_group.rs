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
    let children = snippets::children::render(cx);
    let single = snippets::single::render(cx);
    let small = snippets::small::render(cx);
    let large = snippets::large::render(cx);
    let label = snippets::label::render(cx);
    let full_width_items = snippets::full_width_items::render(cx);
    let stretch = snippets::flex_1_items::render(cx);

    let api_reference = doc_layout::notes_block([
        "Reference stack for this page: shadcn Toggle Group docs, the default registry recipe, Radix Primitives Toggle Group, and Base UI Toggle Group.",
        "The upstream docs-path examples come from the default shadcn demo/outline/sm/lg/spacing set plus the vertical, font-weight-selector, and RTL examples.",
        "Source anchors for that docs path: `toggle-group-demo.tsx`, `toggle-group-outline.tsx`, `toggle-group-sm.tsx`, `toggle-group-lg.tsx`, `toggle-group-spacing.tsx`, `toggle-group-vertical.tsx`, `toggle-group-font-weight-selector.tsx`, and `toggle-group-rtl.tsx`.",
        "`fret_ui_kit::primitives::toggle_group` already covers the mechanism lane (single/multiple state, roving focus, and control-id focus forwarding), so the remaining parity work here is docs/recipe alignment rather than a `fret-ui` contract gap.",
        "`ToggleGroup::single(...)`, `ToggleGroup::multiple(...)`, and their uncontrolled constructors plus `.items([...])` cover the documented docs-path root surface.",
        "`toggle_group_single(...)`, `toggle_group_single_uncontrolled(...)`, `toggle_group_multiple(...)`, and `toggle_group_multiple_uncontrolled(...)` are the builder-preserving composable-children lane when callers want to assemble items inside a closure without landing early.",
        "`ToggleGroupItem::new(..., children)`, `child(...)`, and `children(...)` remain the source-aligned item-content surface for text, icon, and mixed content.",
        "`ToggleGroupItem::refine_layout(...)` and `refine_style(...)` now cover upstream custom item-root sizing and rounding without moving caller-owned recipe tweaks into the default component chrome.",
        "Selection semantics, roving focus, segmented borders, and pressed-state chrome remain recipe-owned; item-root custom layout and surrounding width/flex negotiation remain caller-owned.",
        "No extra root `children([...])` or generic `compose()` API is warranted on the default lane because the helper family already covers composable item assembly without widening the recipe contract.",
        "`Children (Fret)`, `Single (Fret)`, `Small (Fret)`, `Large (Fret)`, `Label Association (Fret)`, `Full Width Items (Fret)`, and `Flex-1 Items (Fret)` stay after the upstream docs path as focused Fret follow-ups and regression slices.",
    ]);
    let notes = doc_layout::notes_block([
        "This page now keeps the upstream shadcn/Base Toggle Group docs path source-aligned on content, default values, and section order before adding focused Fret follow-ups.",
        "Preview now mirrors the upstream Toggle Group docs path first: `Demo`, `Usage`, `Outline`, `Size`, `Spacing`, `Vertical`, `Disabled`, `Custom`, `RTL`, and `API Reference`.",
        "Focused Fret follow-ups stay afterward: `Children (Fret)`, `Single (Fret)`, `Small (Fret)`, `Large (Fret)`, `Label Association (Fret)`, `Full Width Items (Fret)`, `Flex-1 Items (Fret)`, and `Notes`.",
        "The `Size` lane now follows the upstream `toggle-group-sm` + `toggle-group-lg` examples, and `Spacing` keeps the upstream icon-plus-label composition instead of a text-only substitute.",
        "Prefer the documented root constructors plus `.items([...])` for copyable docs-path snippets; reach for the `toggle_group_*` helper family when you want builder-preserving item composition inside a closure.",
        "Item-root refinements belong on the item call site instead of the default group chrome.",
        "The main parity risks here are roving focus, segmented borders, RTL order, and stretch/fill ownership, so stable `ui-gallery-toggle-group-*` ids remain part of the automation contract.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-toggle-group-api-reference")
        .description("Public surface summary and ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-toggle-group-notes")
        .description("Usage guidance and parity notes.");

    let demo = DocSection::build(cx, "Demo", demo)
        .description("Default demo matching the upstream top-of-page preview.")
        .test_id_prefix("ui-gallery-toggle-group-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Minimal single-select usage matching the upstream docs example.")
        .test_id_prefix("ui-gallery-toggle-group-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let outline = DocSection::build(cx, "Outline", outline)
        .description("Outline variant matching the upstream two-item example.")
        .test_id_prefix("ui-gallery-toggle-group-outline")
        .code_rust_from_file_region(snippets::outline::SOURCE, "example");
    let size = DocSection::build(cx, "Size", size)
        .description("Small and large icon-only groups matching the upstream size examples.")
        .test_id_prefix("ui-gallery-toggle-group-size")
        .code_rust_from_file_region(snippets::size::SOURCE, "example");
    let spacing = DocSection::build(cx, "Spacing", spacing)
        .description(
            "Outline spacing with icon-plus-label items matching the upstream docs example.",
        )
        .test_id_prefix("ui-gallery-toggle-group-spacing")
        .code_rust_from_file_region(snippets::spacing::SOURCE, "example");
    let vertical = DocSection::build(cx, "Vertical", vertical)
        .description("Vertical orientation for side panels and inspectors.")
        .test_id_prefix("ui-gallery-toggle-group-vertical")
        .code_rust_from_file_region(snippets::vertical::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .description("Disabled group matching the upstream docs example.")
        .test_id_prefix("ui-gallery-toggle-group-disabled")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let custom = DocSection::build(cx, "Custom", custom)
        .description("Custom item-root sizing and rounding for a font-weight selector.")
        .test_id_prefix("ui-gallery-toggle-group-custom")
        .code_rust_from_file_region(snippets::custom::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Translated labels and item ordering under RTL.")
        .test_id_prefix("ui-gallery-toggle-group-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let children = DocSection::build(cx, "Children (Fret)", children)
        .description(
            "Use the builder-preserving `toggle_group_*` helper family when you want a composable item-children lane without widening the default root surface.",
        )
        .test_id_prefix("ui-gallery-toggle-group-children")
        .code_rust_from_file_region(snippets::children::SOURCE, "example");
    let single = DocSection::build(cx, "Single (Fret)", single)
        .description("Focused single-selection regression example.")
        .test_id_prefix("ui-gallery-toggle-group-single")
        .code_rust_from_file_region(snippets::single::SOURCE, "example");
    let small = DocSection::build(cx, "Small (Fret)", small)
        .description("Small icon-only regression slice retained after the docs path.")
        .test_id_prefix("ui-gallery-toggle-group-small")
        .code_rust_from_file_region(snippets::small::SOURCE, "example");
    let large = DocSection::build(cx, "Large (Fret)", large)
        .description("Large icon-only regression slice retained after the docs path.")
        .test_id_prefix("ui-gallery-toggle-group-large")
        .code_rust_from_file_region(snippets::large::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association (Fret)", label)
        .description("Use `FieldLabel::for_control`, `ToggleGroup::control_id`, and `test_id_prefix` to keep label-focus behavior and automation anchors aligned.")
        .test_id_prefix("ui-gallery-toggle-group-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");
    let full_width_items = DocSection::build(cx, "Full Width Items (Fret)", full_width_items)
        .description("Stretched items to gate full-row fill behavior.")
        .test_id_prefix("ui-gallery-toggle-group-full-width-items")
        .code_rust_from_file_region(snippets::full_width_items::SOURCE, "example");
    let stretch = DocSection::build(cx, "Flex-1 Items (Fret)", stretch)
        .description("Regression gate for hit and visual alignment under `flex-1` sizing.")
        .test_id_prefix("ui-gallery-toggle-group-stretch")
        .code_rust_from_file_region(snippets::flex_1_items::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the upstream Toggle Group docs path first: Demo, Usage, Outline, Size, Spacing, Vertical, Disabled, Custom, RTL, and API Reference. Focused Fret follow-ups stay afterward.",
        ),
        vec![
            demo,
            usage,
            outline,
            size,
            spacing,
            vertical,
            disabled,
            custom,
            rtl,
            api_reference,
            children,
            single,
            small,
            large,
            label,
            full_width_items,
            stretch,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-toggle-group").into_element(cx)]
}
