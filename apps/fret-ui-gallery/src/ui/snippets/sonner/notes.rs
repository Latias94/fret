// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                shadcn::typography::muted(cx, Arc::<str>::from(format!("Last action: {last}"))),
                shadcn::typography::muted(
                    cx,
                    "Preview follows `sonner-demo.tsx` (new-york-v4): buttons that trigger different toast types.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Fret exposes extra knobs (position, pinned + swipe dismiss) for testing overlay behavior.",
                ),
                shadcn::typography::muted(
                    cx,
                    "API reference: `ecosystem/fret-ui-shadcn/src/sonner.rs`.",
                ),
            ]
        },
    )
}
// endregion: example
