pub const SOURCE: &str = include_str!("with_icon.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    wrap_row(cx, |cx| {
        vec![
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .children([
                    fret_ui_shadcn::icon::icon_with(
                        cx,
                        IconId::new_static("lucide.git-branch"),
                        None,
                        None,
                    )
                    .test_id("ui-gallery-button-with-icon-icon"),
                    ui::text("New Branch")
                        .font_medium()
                        .nowrap()
                        .into_element(cx)
                        .test_id("ui-gallery-button-with-icon-label"),
                ])
                .test_id("ui-gallery-button-with-icon")
                .into_element(cx),
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .children([
                    ui::text("Fork").font_medium().nowrap().into_element(cx),
                    fret_ui_shadcn::icon::icon_with(
                        cx,
                        IconId::new_static("lucide.git-fork"),
                        None,
                        None,
                    ),
                ])
                .test_id("ui-gallery-button-with-trailing-icon")
                .into_element(cx),
        ]
    })
    .test_id("ui-gallery-button-with-icon-row")
}
// endregion: example
