pub const SOURCE: &str = include_str!("setup.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let intro = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(cx, "Mount a `Toaster` once per window."),
                shadcn::typography::muted(
                    cx,
                    "This installs the toast overlay layer and drives default styling + icons.",
                ),
            ]
        },
    );

    // Copy/paste reference:
    //
    // shadcn::Toaster::new()
    //     .position(shadcn::ToastPosition::TopCenter)
    //     .shadcn_lucide_icons()
    //     .into_element(cx);
    intro
}
// endregion: example
