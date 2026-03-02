// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Preview follows `resizable-demo.tsx` (new-york-v4): nested panels, with-handle, and vertical orientation.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Resizable groups expose their own semantics; keep an eye on focus order and hit-testing near handles.",
                ),
                shadcn::typography::muted(
                    cx,
                    "API reference: `ecosystem/fret-ui-shadcn/src/resizable.rs`.",
                ),
            ]
        },
    )
}
// endregion: example

