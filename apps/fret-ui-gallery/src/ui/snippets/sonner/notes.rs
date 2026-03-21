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
                    "Docs-aligned sections mirror shadcn's `Demo`, `Types`, `Description`, and `Position` previews.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Fret-specific extras stay after the docs examples: action/cancel helpers and swipe-dismiss diagnostics coverage.",
                ).into_element(cx),
                shadcn::raw::typography::muted(
                    "Composable custom children are not exposed yet: the current Sonner surface is still the message-style API tracked in `ecosystem/fret-ui-shadcn/src/sonner.rs`.",
                ).into_element(cx),
            ]
        })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx)
}
// endregion: example
