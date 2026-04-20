#[allow(dead_code)]
pub const SOURCE: &str = include_str!("notes.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
            vec![
                shadcn::raw::typography::muted(
                    "Gallery now mirrors the shadcn/Base UI Resizable docs path after collapsing the top preview into `Demo` and skipping `Installation`; the Fret-only `Adaptive Panel Proof` section now promotes one fixed-window panel-resize/container-query teaching surface before this closeout note.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Resizable groups expose their own semantics; keep an eye on focus order and hit-testing near handles.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Evidence anchors: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/resizable.rs`, `apps/fret-ui-gallery/src/driver/render_flow.rs`, `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-adaptive-panel-proof.json`, and `tools/diag-scripts/ui-gallery/resizable/`.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Default-style ownership follows upstream: `ResizablePanelGroup` owns `w-full h-full` and handle chrome, while border/rounded demo shells remain caller-owned.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "No extra generic children API is planned unless a real authoring cliff appears: `resizable_panel_group(cx, model, |cx| ..)` plus typed `ResizableEntry` ordering already preserve the source-aligned composition lane without hiding handle/panel structure.",
                ).into_element(cx),
            ]
        })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx)
}
// endregion: example
