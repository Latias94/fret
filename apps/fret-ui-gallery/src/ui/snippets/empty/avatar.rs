pub const SOURCE: &str = include_str!("avatar.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar_media = shadcn::Avatar::new([shadcn::AvatarFallback::new("JD").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_px(Px(48.0)).h_px(Px(48.0)))
        .into_element(cx);

    shadcn::empty(|cx| {
        ui::children![
            cx;
            shadcn::empty_header(|cx| {
                ui::children![
                    cx;
                    shadcn::empty_media(|cx| ui::children![cx; avatar_media]),
                    shadcn::empty_title("User Offline"),
                    shadcn::empty_description(
                        "This user is currently offline. Leave a message and notify later.",
                    ),
                ]
            }),
            shadcn::empty_content(|cx| {
                ui::children![cx; shadcn::Button::new("Leave Message").size(shadcn::ButtonSize::Sm),]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-avatar")
}
// endregion: example
