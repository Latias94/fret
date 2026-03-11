pub const SOURCE: &str = include_str!("notes.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
            vec![
                shadcn::raw::typography::muted(
                    cx,
                    "This page mirrors shadcn Date Picker docs (new-york-v4) and keeps the diag suite stable.",
                ),
                shadcn::raw::typography::muted(
                    cx,
                    "Calendar dropdown caption improves large-jump navigation compared with arrow-only controls.",
                ),
                shadcn::raw::typography::muted(
                    cx,
                    "For diag runs, some dates are intentionally disabled (via env flag) to validate skip behavior.",
                ),
                shadcn::raw::typography::muted(
                    cx,
                    "Trigger width stays caller-owned: add `w_full`, fixed widths, or max-width constraints at the example/form layer.",
                ),
                shadcn::raw::typography::muted(
                    cx,
                    "The compact builder keeps recipe-owned overlay chrome (`PopoverContent` remains `w-auto p-0`).",
                ),
                shadcn::raw::typography::muted(
                    cx,
                    "Natural language picker uses a small built-in parser (subset of chrono-node behavior).",
                ),
            ]
        })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx)
}
// endregion: example
