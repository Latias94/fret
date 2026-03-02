pub const SOURCE: &str = include_str!("avatar_group.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let invite_icon = shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.user-plus"));
    let invite_text = cx.text("Invite Members");
    let invite_button = shadcn::Button::new("Invite Members")
        .size(shadcn::ButtonSize::Sm)
        .children([invite_icon, invite_text])
        .into_element(cx);

    let avatars = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N1).items_center(),
        |cx| {
            vec![
                shadcn::Avatar::new([shadcn::AvatarFallback::new("CN").into_element(cx)])
                    .refine_layout(LayoutRefinement::default().w_px(Px(44.0)).h_px(Px(44.0)))
                    .into_element(cx),
                shadcn::Avatar::new([shadcn::AvatarFallback::new("LR").into_element(cx)])
                    .refine_layout(LayoutRefinement::default().w_px(Px(44.0)).h_px(Px(44.0)))
                    .into_element(cx),
                shadcn::Avatar::new([shadcn::AvatarFallback::new("ER").into_element(cx)])
                    .refine_layout(LayoutRefinement::default().w_px(Px(44.0)).h_px(Px(44.0)))
                    .into_element(cx),
            ]
        },
    );

    shadcn::Empty::new([
        shadcn::empty::EmptyHeader::new([
            shadcn::empty::EmptyMedia::new([avatars]).into_element(cx),
            shadcn::empty::EmptyTitle::new("No Team Members").into_element(cx),
            shadcn::empty::EmptyDescription::new(
                "Invite collaborators to start working on this project together.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::empty::EmptyContent::new([invite_button]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-avatar-group")
}
// endregion: example
