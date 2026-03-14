pub const SOURCE: &str = include_str!("setup.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let intro = ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::muted("Mount a `Toaster` once per window.").into_element(cx),
            shadcn::raw::typography::muted(
                "This installs the toast overlay layer and drives default styling + icons.",
            )
            .into_element(cx),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    // Copy/paste reference:
    //
    // shadcn::Toaster::new()
    //     .id("notifications")
    //     .position(shadcn::ToastPosition::TopCenter)
    //     .shadcn_lucide_icons()
    //     .into_element(cx);
    intro
}
// endregion: example
