pub const SOURCE: &str = include_str!("fallback_only.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N4)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}

fn avatar_fallback_only<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: shadcn::AvatarSize,
    test_id: &'static str,
) -> AnyElement {
    shadcn::Avatar::new([shadcn::AvatarFallback::new("CN").into_element(cx)])
        .size(size)
        .into_element(cx)
        .test_id(test_id)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    wrap_row(cx, |cx| {
        vec![
            avatar_fallback_only(cx, shadcn::AvatarSize::Sm, "ui-gallery-avatar-fallback-sm"),
            avatar_fallback_only(
                cx,
                shadcn::AvatarSize::Default,
                "ui-gallery-avatar-fallback-default",
            ),
            avatar_fallback_only(cx, shadcn::AvatarSize::Lg, "ui-gallery-avatar-fallback-lg"),
        ]
    })
    .test_id("ui-gallery-avatar-fallback")
}
// endregion: example
