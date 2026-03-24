pub const SOURCE: &str = include_str!("notes.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
            vec![
                shadcn::raw::typography::muted(
                    "Preview follows the shadcn Resizable docs path first: nested demo, usage, vertical, handle, and RTL coverage.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Resizable groups expose their own semantics; keep an eye on focus order and hit-testing near handles.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "API reference: `ecosystem/fret-ui-shadcn/src/resizable.rs`.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Default-style ownership follows upstream: `ResizablePanelGroup` owns `w-full h-full` and handle chrome, while border/rounded demo shells remain caller-owned.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "No extra generic children API is warranted here: `resizable_panel_group(cx, model, |cx| ..)` plus typed `ResizableEntry` ordering already preserve the source-aligned composition lane without hiding handle/panel structure.",
                ).into_element(cx),
            ]
        })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx)
}
// endregion: example
