pub const SOURCE: &str = include_str!("notes.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
            vec![
                shadcn::raw::typography::muted(
                    "This page mirrors the shadcn Date Picker docs path first, then appends the upstream `date-picker-with-presets` follow-up plus focused Fret/gallery extensions.",
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
                    "`Usage` stays on the upstream `Popover + Calendar` composition lane; `Compact Builder (Fret)` keeps the ergonomic shorthand explicit without replacing the docs surface.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "No generic date-picker children API is currently warranted: custom trigger/content authoring already stays explicit on the composed `Popover` / `PopoverTrigger` / `PopoverContent` + `Calendar` surface.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "The compact builder keeps recipe-owned overlay chrome (`PopoverContent` remains `w-auto p-0`) and leaves width negotiation caller-owned.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Compact wrapper default selection now matches upstream docs: choosing a day does not dismiss the popover unless the caller opts into `.close_on_select(true)` or controls `open` explicitly.",
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
