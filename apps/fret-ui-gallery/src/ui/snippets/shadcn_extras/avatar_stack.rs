pub const SOURCE: &str = include_str!("avatar_stack.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let a = shadcn::Avatar::new([shadcn::AvatarFallback::new("A").into_element(cx)]);
    let b = shadcn::Avatar::new([shadcn::AvatarFallback::new("B").into_element(cx)]);
    let c = shadcn::Avatar::new([shadcn::AvatarFallback::new("C").into_element(cx)]);
    let d = shadcn::Avatar::new([shadcn::AvatarFallback::new("D").into_element(cx)]);
    let e = shadcn::Avatar::new([shadcn::AvatarFallback::new("E").into_element(cx)]);

    fret_ui_shadcn::extras::AvatarStack::new([a, b, c, d, e])
        .size_px(Px(40.0))
        .max_visible(4)
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-avatar-stack")
}
// endregion: example
