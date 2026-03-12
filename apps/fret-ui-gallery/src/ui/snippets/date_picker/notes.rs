pub const SOURCE: &str = include_str!("notes.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
            vec![
                shadcn::raw::typography::muted(
                    "This page mirrors shadcn Date Picker docs (new-york-v4) and keeps the diag suite stable.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Calendar dropdown caption improves large-jump navigation compared with arrow-only controls.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "For diag runs, some dates are intentionally disabled (via env flag) to validate skip behavior.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Trigger width stays caller-owned: add `w_full`, fixed widths, or max-width constraints at the example/form layer.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "The compact builder keeps recipe-owned overlay chrome (`PopoverContent` remains `w-auto p-0`).",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Natural language picker uses a small built-in parser (subset of chrono-node behavior).",
                ).into_element(cx),
            ]
        })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx)
}
// endregion: example
