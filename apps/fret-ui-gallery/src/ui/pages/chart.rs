use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::chart as snippets;

pub(super) fn preview_chart(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo_cards = snippets::demo::render(cx);
    let first_chart = snippets::usage::render(cx);
    let config = snippets::config::render(cx);
    let theming = snippets::theming::render(cx);
    let grid_axis = snippets::grid_axis::render(cx);
    let contracts_overview = snippets::contracts::render(cx);
    let tooltip_content = snippets::tooltip::render(cx);
    let legend_content = snippets::legend::render(cx);
    let accessibility = snippets::accessibility::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes_stack = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/chart.rs`.",
        "Chart now exposes both `ChartContainer::new(...).into_element(cx, |cx| ...)` and the more shadcn-like `chart_container(config, |cx| ...)` builder surface for composable child authoring.",
        "Demo cards are rendered with `delinea` + `fret-chart` (not Recharts); this is a stand-in to keep chart layout real in native builds.",
        "`ChartLegendContent::new()` can derive labels, icons, and colors from `ChartConfig` when explicit legend items are omitted, and `name_key(...)` can remap legend labels through item metadata.",
        "`ChartTooltipContent::new()` can now auto-derive label, items, colors, and icons from a shared `ChartCanvasOutput` model plus `ChartConfig`.",
        "`ChartTooltipContent` now exposes recipe-level `label_formatter(...)`, `formatter(...)`, `label_key(...)`, and `name_key(...)` hooks, with Fret-native item key/metadata remapping.",
        "For fully custom tooltip header/rows, `ChartTooltipContent::into_element_label_parts(cx, ...)`, `ChartTooltipContent::into_element_parts(cx, ...)`, and `ChartTooltipContent::into_element_parts_with_label(cx, ...)` cover header-only, row-only, or fully combined children composition.",
        "Unlike the Recharts docs path, `Add Grid` and `Add Axis` stay inside the retained chart spec today instead of surfacing as separate child widgets on the gallery page.",
        "The remaining parity gaps are full Recharts `payload.payload[...]` field lookup on engine-derived payloads and DOM-native overlay composition details.",
        "Keep color mapping stable through `chart-*` tokens to avoid dark-theme drift.",
        "`fret-chart::ChartCanvas` exposes an accessibility layer via keyboard focus + arrow navigation, mirroring Recharts `accessibilityLayer` outcomes at a high level.",
    ]);
    let notes_stack = DocSection::build(cx, "Notes", notes_stack)
        .description("Fret-specific API surface and parity notes.");
    let component = DocSection::build(cx, "Component", demo_cards)
        .description(
            "Composition-first chart recipe surface: build the chart body inside `chart_container(config, |cx| ...)`, then opt into `ChartTooltip` and `ChartLegend` only where needed.",
        )
        .no_shell()
        .max_w(Px(1100.0))
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let first_chart = DocSection::build(cx, "First Chart", first_chart)
        .description(
            "Fret-native equivalent of shadcn's first-chart walkthrough: build the chart, then add legend and tooltip on the same assembled example, with tooltip payloads auto-derived from a shared chart output model. Grid and axis stay in the retained chart spec instead of separate child widgets.",
        )
        .test_id_prefix("ui-gallery-chart-first-chart")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let config = DocSection::build(cx, "Chart Config", config)
        .description("Labels, icons, and colors live in `ChartConfig` and should stay decoupled from chart data.")
        .test_id_prefix("ui-gallery-chart-config")
        .code_rust_from_file_region(snippets::config::SOURCE, "example");
    let theming = DocSection::build(cx, "Theming", theming)
        .description("Use `chart-*` theme tokens as the stable color source across charts, legends, and tooltip recipes.")
        .test_id_prefix("ui-gallery-chart-theming")
        .code_rust_from_file_region(snippets::theming::SOURCE, "example");
    let grid_axis = DocSection::build(cx, "Grid / Axis (Fret)", grid_axis)
        .description(
            "Focused Fret follow-up: grid and axis remain spec-owned on `delinea::ChartSpec` today, so the copyable setup lives beside the retained chart engine instead of the `ChartContainer` child lane.",
        )
        .test_id_prefix("ui-gallery-chart-grid-axis")
        .code_rust_from_file_region(snippets::grid_axis::SOURCE, "example");
    let contracts_overview = DocSection::build(cx, "Contracts", contracts_overview)
        .description(
            "Fret-specific follow-up contracts once the shadcn docs path is covered: payload auto wiring, formatter hooks, and advanced adapter seams.",
        )
        .code_rust_from_file_region(snippets::contracts::SOURCE, "example");
    let tooltip_content = DocSection::build(cx, "Tooltip", tooltip_content)
        .description("Tooltip examples now read in a shadcn-like order: props first, config-driven colors and key remapping second, then formatter plus header-only, row-only, and combined custom children seams.")
        .test_id_prefix("ui-gallery-chart-tooltip")
        .code_rust_from_file_region(snippets::tooltip::SOURCE, "example");
    let legend_content = DocSection::build(cx, "Legend", legend_content)
        .description(
            "Legend examples now track the shadcn docs flow more directly: config-driven colors first, then `name_key` remapping for custom names.",
        )
        .test_id_prefix("ui-gallery-chart-legend")
        .code_rust_from_file_region(snippets::legend::SOURCE, "example");
    let accessibility = DocSection::build(cx, "Accessibility", accessibility)
        .description("Native chart accessibility lives on `fret-chart::ChartCanvas`, with a reusable helper for the common defaults.")
        .test_id_prefix("ui-gallery-chart-accessibility")
        .code_rust_from_file_region(snippets::accessibility::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Direction provider coverage for tooltip and legend recipes.")
        .test_id_prefix("ui-gallery-chart-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Chart docs path first: `Component`, `First Chart`, `Chart Config`, `Theming`, `Tooltip`, `Legend`, `Accessibility`, and `RTL`. After that, Gallery keeps Fret-specific follow-ups explicit: `Grid / Axis (Fret)`, `Contracts`, and `Notes`.",
        ),
        vec![
            component,
            first_chart,
            config,
            theming,
            tooltip_content,
            legend_content,
            accessibility,
            rtl,
            grid_axis,
            contracts_overview,
            notes_stack,
        ],
    );

    vec![body.test_id("ui-gallery-chart-component").into_element(cx)]
}
