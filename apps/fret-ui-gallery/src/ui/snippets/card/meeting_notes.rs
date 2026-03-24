pub const SOURCE: &str = include_str!("meeting_notes.rs");

// region: example
use super::super::avatar::demo_image;
use fret::{UiChild, UiCx};
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn marker(cx: &mut UiCx<'_>, text: &'static str) -> impl IntoUiElement<fret_app::App> + use<> {
    ui::text(text)
        .text_sm()
        .w_space(Space::N4)
        .text_align(fret_core::TextAlign::End)
        .into_element(cx)
}

fn item(
    cx: &mut UiCx<'_>,
    n: &'static str,
    content: &'static str,
    test_id: Option<&'static str>,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let row = ui::h_flex(|cx| {
        vec![
            marker(cx, n).into_element(cx),
            ui::text(content)
                .text_sm()
                .flex_1()
                .min_w_0()
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx);

    match test_id {
        Some(id) => row.test_id(id),
        None => row,
    }
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app).snapshot();

    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let avatars = {
        let avatar_image = demo_image(cx);
        let avatar_fallbacks = ["CN", "LR", "ER"];
        let avatars = avatar_fallbacks
            .iter()
            .map(|fallback| {
                let image = shadcn::AvatarImage::maybe(avatar_image).into_element(cx);
                let fallback = shadcn::AvatarFallback::new(*fallback)
                    .when_image_missing(avatar_image)
                    .delay_ms(120)
                    .into_element(cx);
                shadcn::Avatar::new([image, fallback]).into_element(cx)
            })
            .collect::<Vec<_>>();

        let count =
            shadcn::AvatarGroupCount::new([ui::text("+8").font_medium().nowrap().into_element(cx)])
                .into_element(cx)
                .test_id("ui-gallery-card-notes-avatar-count");

        shadcn::AvatarGroup::new(avatars.into_iter().chain([count]).collect::<Vec<_>>())
            .into_element(cx)
            .test_id("ui-gallery-card-notes-avatars")
    };

    let list = ui::v_flex(|cx| {
        let paragraph = ui::text("Here are the meeting notes:")
            .text_sm()
            .into_element(cx);

        let ordered_list = {
            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default().pt(Space::N4),
                LayoutRefinement::default().w_full(),
            );

            cx.container(props, |cx| {
                vec![
                    ui::v_flex(|cx| {
                        vec![
                            item(
                                cx,
                                "1.",
                                "New analytics widgets for daily/weekly metrics",
                                None,
                            )
                            .into_element(cx),
                            item(cx, "2.", "Simplified navigation menu", None).into_element(cx),
                            item(
                                cx,
                                "3.",
                                "Dark mode support",
                                Some("ui-gallery-card-notes-item-dark-mode"),
                            )
                            .into_element(cx),
                            item(cx, "4.", "Timeline: 6 weeks", None).into_element(cx),
                            item(
                                cx,
                                "5.",
                                "Follow-up meeting scheduled for next Tuesday",
                                None,
                            )
                            .into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .items_stretch()
                    .layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                ]
            })
        };

        vec![paragraph, ordered_list]
    })
    .gap(Space::N2)
    .items_stretch()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-card-notes-list");

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Meeting Notes"),
                    shadcn::card_description("Transcript from the meeting with the client."),
                    shadcn::card_action(|cx| {
                        ui::children![
                            cx;
                            shadcn::Button::new("Transcribe")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .leading_icon(fret_icons::IconId::new_static("lucide.captions"))
                                .ui()
                                .test_id("ui-gallery-card-notes-transcribe"),
                        ]
                    }),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; list]),
            shadcn::card_footer(|cx| ui::children![cx; avatars]),
        ]
    })
    .refine_layout(max_w_sm.clone().max_h(Px(400.0)))
    .into_element(cx)
    .test_id("ui-gallery-card-meeting-notes")
}
// endregion: example
