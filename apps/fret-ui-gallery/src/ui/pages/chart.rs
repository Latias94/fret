use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::chart as snippets;

pub(super) fn preview_chart(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo_cards = snippets::demo::render(cx);
    let contracts_overview = snippets::contracts::render(cx);
    let tooltip_content = snippets::tooltip::render(cx);
    let legend_content = snippets::legend::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes_stack = doc_layout::notes(
        cx,
        [
            "Demo cards are rendered with `delinea` + `fret-chart` (not Recharts); this is a stand-in to keep chart layout real in native builds.",
            "The shadcn `ChartTooltipContent` / `ChartLegendContent` recipes are validated independently (no runtime wire-up yet).",
            "Keep color mapping stable through `chart-*` tokens to avoid dark-theme drift.",
            "`fret-chart::ChartCanvas` exposes an accessibility layer via keyboard focus + arrow navigation, mirroring Recharts `accessibilityLayer` outcomes at a high level.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Chart demo: Area, Bar (Multiple), Bar (Mixed), Line (Multiple).",
        ),
        vec![
            DocSection::new("Demo", demo_cards)
                .no_shell()
                .max_w(Px(1100.0))
                .code_rust_from_file_region(include_str!("../snippets/chart/demo.rs"), "example"),
            DocSection::new("Contracts", contracts_overview)
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/chart/contracts.rs"),
                    "example",
                ),
            DocSection::new("Tooltip", tooltip_content)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-chart-tooltip")
                .code_rust_from_file_region(
                    include_str!("../snippets/chart/tooltip.rs"),
                    "example",
                ),
            DocSection::new("Legend", legend_content)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-chart-legend")
                .code_rust_from_file_region(include_str!("../snippets/chart/legend.rs"), "example"),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-chart-rtl")
                .code_rust_from_file_region(include_str!("../snippets/chart/rtl.rs"), "example"),
            DocSection::new("Notes", notes_stack).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-chart-component")]
}
