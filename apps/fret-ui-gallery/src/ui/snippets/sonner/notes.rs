// region: example
use crate::ui::snippets::sonner::last_action_model;
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let last_action = last_action_model(cx);
    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    ui::v_flex(move |cx| {
            vec![
                shadcn::raw::typography::muted( Arc::<str>::from(format!("Last action: {last}"))).into_element(cx),
                shadcn::raw::typography::muted(
                    "Preview follows `sonner-demo.tsx` (new-york-v4): buttons that trigger different toast types.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Fret exposes extra knobs (position, pinned + swipe dismiss) for testing overlay behavior.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "API reference: `ecosystem/fret-ui-shadcn/src/sonner.rs`.",
                ).into_element(cx),
            ]
        })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx)
}
// endregion: example
