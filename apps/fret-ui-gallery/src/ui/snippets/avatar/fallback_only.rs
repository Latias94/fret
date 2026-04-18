pub const SOURCE: &str = include_str!("fallback_only.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn wrap_row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N4)
        .wrap()
        .w_full()
        .items_center()
}

fn avatar_fallback_only<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: shadcn::AvatarSize,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::Avatar::new([shadcn::AvatarFallback::new("CN").into_element(cx)])
        .size(size)
        .into_element(cx)
        .test_id(test_id)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    wrap_row(|cx| {
        vec![
            avatar_fallback_only(cx, shadcn::AvatarSize::Sm, "ui-gallery-avatar-fallback-sm")
                .into_element(cx),
            avatar_fallback_only(
                cx,
                shadcn::AvatarSize::Default,
                "ui-gallery-avatar-fallback-default",
            )
            .into_element(cx),
            avatar_fallback_only(cx, shadcn::AvatarSize::Lg, "ui-gallery-avatar-fallback-lg")
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-avatar-fallback")
}
// endregion: example
