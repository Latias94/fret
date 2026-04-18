pub const SOURCE: &str = include_str!("avatar_stack.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn stack<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str) -> AnyElement {
    let a = shadcn::Avatar::new([shadcn::AvatarFallback::new("A").into_element(cx)]);
    let b = shadcn::Avatar::new([shadcn::AvatarFallback::new("B").into_element(cx)]);
    let c = shadcn::Avatar::new([shadcn::AvatarFallback::new("C").into_element(cx)]);
    let d = shadcn::Avatar::new([shadcn::AvatarFallback::new("D").into_element(cx)]);
    let e = shadcn::Avatar::new([shadcn::AvatarFallback::new("E").into_element(cx)]);

    shadcn::raw::extras::AvatarStack::new([a, b, c, d, e])
        .size_px(Px(40.0))
        .max_visible(4)
        .into_element(cx)
        .test_id(test_id)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    fret_ui_kit::ui::h_flex(|cx| {
        vec![
            fret_ui_kit::ui::v_flex(|cx| {
                vec![
                    ui::text("LTR").font_medium().into_element(cx),
                    stack(cx, "ui-gallery-shadcn-extras-avatar-stack-ltr"),
                ]
            })
            .gap(Space::N2)
            .items_start()
            .into_element(cx),
            fret_ui_kit::ui::v_flex(|cx| {
                vec![
                    ui::text("RTL").font_medium().into_element(cx),
                    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                        stack(cx, "ui-gallery-shadcn-extras-avatar-stack-rtl")
                    }),
                ]
            })
            .gap(Space::N2)
            .items_start()
            .into_element(cx),
        ]
    })
    .gap(Space::N8)
    .wrap()
    .items_start()
    .into_element(cx)
    .test_id("ui-gallery-shadcn-extras-avatar-stack")
}
// endregion: example
