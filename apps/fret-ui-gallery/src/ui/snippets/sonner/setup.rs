pub const SOURCE: &str = include_str!("setup.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let intro = ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::muted(cx, "Mount a `Toaster` once per window."),
            shadcn::raw::typography::muted(
                cx,
                "This installs the toast overlay layer and drives default styling + icons.",
            ),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    // Copy/paste reference:
    //
    // shadcn::Toaster::new()
    //     .position(shadcn::ToastPosition::TopCenter)
    //     .shadcn_lucide_icons()
    //     .into_element(cx);
    intro
}
// endregion: example
