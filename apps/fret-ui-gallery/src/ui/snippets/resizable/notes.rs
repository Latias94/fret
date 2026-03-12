// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
            vec![
                shadcn::raw::typography::muted(
                    "Preview follows `resizable-demo.tsx` (new-york-v4): nested panels, with-handle, and vertical orientation.",
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
            ]
        })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx)
}
// endregion: example
