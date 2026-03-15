pub const SOURCE: &str = include_str!("avatar_group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let invite_icon =
        fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.user-plus"));
    let invite_text = cx.text("Invite Members");
    let invite_button = shadcn::Button::new("Invite Members")
        .size(shadcn::ButtonSize::Sm)
        .children([invite_icon, invite_text])
        .into_element(cx);

    let avatars = ui::h_row(|cx| {
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
    })
    .gap(Space::N1)
    .items_center()
    .into_element(cx);

    shadcn::empty(|cx| {
        ui::children![
            cx;
            shadcn::empty_header(|cx| {
                ui::children![
                    cx;
                    shadcn::empty_media(|cx| ui::children![cx; avatars]),
                    shadcn::empty_title("No Team Members"),
                    shadcn::empty_description(
                        "Invite collaborators to start working on this project together.",
                    ),
                ]
            }),
            shadcn::empty_content(|cx| ui::children![cx; invite_button]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-avatar-group")
}
// endregion: example
