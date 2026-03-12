use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::chart as snippets;

pub(super) fn preview_chart(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo_cards = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let contracts_overview = snippets::contracts::render(cx);
    let tooltip_content = snippets::tooltip::render(cx);
    let legend_content = snippets::legend::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes_stack = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/chart.rs`.",
            "Chart already exposes the important authoring surface (`ChartConfig`, `ChartContainer`, `ChartTooltip`, `ChartTooltipContent`, `ChartLegend`, `ChartLegendContent`), so the main parity gap here was usage clarity rather than missing mechanism work in the shadcn-facing layer.",
            "Demo cards are rendered with `delinea` + `fret-chart` (not Recharts); this is a stand-in to keep chart layout real in native builds.",
            "The shadcn `ChartTooltipContent` / `ChartLegendContent` recipes are validated independently (no runtime wire-up yet).",
            "Keep color mapping stable through `chart-*` tokens to avoid dark-theme drift.",
            "`fret-chart::ChartCanvas` exposes an accessibility layer via keyboard focus + arrow navigation, mirroring Recharts `accessibilityLayer` outcomes at a high level.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Chart docs flow: Demo -> Usage -> Tooltip/Legend contracts. Fret-specific chart-engine notes stay explicit so parity gaps remain visible.",
        ),
        vec![
            DocSection::new("Demo", demo_cards)
                .no_shell()
                .max_w(Px(1100.0))
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for the chart container + tooltip/legend recipe surface.")
                .test_id_prefix("ui-gallery-chart-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Contracts", contracts_overview)
                .code_rust_from_file_region(snippets::contracts::SOURCE, "example"),
            DocSection::new("Tooltip", tooltip_content)
                .test_id_prefix("ui-gallery-chart-tooltip")
                .code_rust_from_file_region(snippets::tooltip::SOURCE, "example"),
            DocSection::new("Legend", legend_content)
                .test_id_prefix("ui-gallery-chart-legend")
                .code_rust_from_file_region(snippets::legend::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-chart-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes_stack).description("API surface and parity notes."),
        ],
    );

    vec![body.test_id("ui-gallery-chart-component")]
}
