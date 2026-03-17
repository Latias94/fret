use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::chart as snippets;

pub(super) fn preview_chart(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo_cards = snippets::demo::render(cx);
    let first_chart = snippets::usage::render(cx);
    let config = snippets::config::render(cx);
    let theming = snippets::theming::render(cx);
    let contracts_overview = snippets::contracts::render(cx);
    let tooltip_content = snippets::tooltip::render(cx);
    let legend_content = snippets::legend::render(cx);
    let accessibility = snippets::accessibility::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes_stack = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/chart.rs`.",
        "Chart now exposes both `ChartContainer::new(...).into_element(cx, |cx| ...)` and the more shadcn-like `chart_container(config, |cx| ...)` builder surface for composable child authoring.",
        "Demo cards are rendered with `delinea` + `fret-chart` (not Recharts); this is a stand-in to keep chart layout real in native builds.",
        "`ChartLegendContent::new()` can derive labels, icons, and colors from `ChartConfig` when explicit legend items are omitted.",
        "`ChartTooltipContent::new()` can now auto-derive label, items, colors, and icons from a shared `ChartCanvasOutput` model plus `ChartConfig`.",
        "`ChartTooltipContent` now exposes recipe-level `label_formatter(...)`, `formatter(...)`, `label_key(...)`, and `name_key(...)` hooks, with Fret-native item key/metadata remapping.",
        "For fully custom tooltip header/rows, `ChartTooltipContent::into_element_label_parts(cx, ...)` and `ChartTooltipContent::into_element_parts(cx, ...)` are the advanced adapter seams for arbitrary children composition.",
        "The remaining parity gaps are full Recharts `payload.payload[...]` field lookup on engine-derived payloads and DOM-native overlay composition details.",
        "Keep color mapping stable through `chart-*` tokens to avoid dark-theme drift.",
        "`fret-chart::ChartCanvas` exposes an accessibility layer via keyboard focus + arrow navigation, mirroring Recharts `accessibilityLayer` outcomes at a high level.",
    ]);
    let notes_stack =
        DocSection::build(cx, "Notes", notes_stack).description("API surface and parity notes.");
    let demo_cards = DocSection::build(cx, "Demo", demo_cards)
        .no_shell()
        .max_w(Px(1100.0))
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let first_chart = DocSection::build(cx, "First Chart", first_chart)
        .description(
            "Fret-native equivalent of shadcn's composition flow: chart container, chart canvas, legend defaults from config, and tooltip content auto-derived from a shared chart output model.",
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
    let contracts_overview = DocSection::build(cx, "Contracts", contracts_overview)
        .description("Fret-specific chart recipe contracts that still matter once payload auto-wiring is enabled.")
        .code_rust_from_file_region(snippets::contracts::SOURCE, "example");
    let tooltip_content = DocSection::build(cx, "Tooltip", tooltip_content)
        .description("Recipe-level tooltip variants, label/item format hooks, and advanced custom header/row children seams aligned to the shadcn tooltip teaching surface.")
        .test_id_prefix("ui-gallery-chart-tooltip")
        .code_rust_from_file_region(snippets::tooltip::SOURCE, "example");
    let legend_content = DocSection::build(cx, "Legend", legend_content)
        .description(
            "Legend recipes can be explicit or derive their items from `ChartConfig` by default.",
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
            "Preview follows the shadcn Chart docs shape more closely: first-chart composition, config, theming, tooltip/legend, accessibility, and RTL. Fret-specific chart-engine constraints still stay explicit, but the first-chart path now includes output-model-driven tooltip payload binding.",
        ),
        vec![
            demo_cards,
            first_chart,
            config,
            theming,
            contracts_overview,
            tooltip_content,
            legend_content,
            accessibility,
            rtl,
            notes_stack,
        ],
    );

    vec![body.test_id("ui-gallery-chart-component").into_element(cx)]
}
