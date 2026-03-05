pub const SOURCE: &str = include_str!("meeting_notes.rs");

// region: example
use fret_app::App;
use fret_ui::Theme;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::{Arc, OnceLock};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let avatars = {
        let avatar_source = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                static AVATAR_TEST_JPG: OnceLock<Option<fret_ui_assets::ImageSource>> =
                    OnceLock::new();
                AVATAR_TEST_JPG.get_or_init(|| {
                    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../assets/textures/test.jpg");
                    if path.exists() {
                        Some(fret_ui_assets::ImageSource::from_path(Arc::new(path)))
                    } else {
                        None
                    }
                })
            }

            #[cfg(target_arch = "wasm32")]
            {
                static AVATAR_TEST_JPG: OnceLock<fret_ui_assets::ImageSource> = OnceLock::new();
                Some(AVATAR_TEST_JPG.get_or_init(|| {
                    fret_ui_assets::ImageSource::from_url(Arc::<str>::from("textures/test.jpg"))
                }))
            }
        };

        let avatar_fallbacks = ["CN", "LR", "ER"];
        let avatars = avatar_fallbacks
            .iter()
            .map(|fallback| {
                let image_id = avatar_source
                    .as_ref()
                    .map(|source| cx.use_image_source_state(source).image)
                    .flatten();
                let image = shadcn::AvatarImage::maybe(image_id).into_element(cx);
                let fallback = shadcn::AvatarFallback::new(*fallback)
                    .when_image_missing(image_id)
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
                        let marker = |cx: &mut ElementContext<'_, App>, text: &str| {
                            ui::text(text)
                                .text_sm()
                                .w_space(Space::N4)
                                .text_align(fret_core::TextAlign::End)
                                .into_element(cx)
                        };

                        let item =
                            |cx: &mut ElementContext<'_, App>,
                             n: &str,
                             content: &str,
                             test_id: Option<&'static str>| {
                                let row = ui::h_flex(|cx| {
                                    vec![
                                        marker(cx, n),
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
                            };

                        vec![
                            item(
                                cx,
                                "1.",
                                "New analytics widgets for daily/weekly metrics",
                                None,
                            ),
                            item(cx, "2.", "Simplified navigation menu", None),
                            item(
                                cx,
                                "3.",
                                "Dark mode support",
                                Some("ui-gallery-card-notes-item-dark-mode"),
                            ),
                            item(cx, "4.", "Timeline: 6 weeks", None),
                            item(
                                cx,
                                "5.",
                                "Follow-up meeting scheduled for next Tuesday",
                                None,
                            ),
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

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Meeting Notes").into_element(cx),
            shadcn::CardDescription::new("Transcript from the meeting with the client.")
                .into_element(cx),
            shadcn::CardAction::new([shadcn::Button::new("Transcribe")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .leading_icon(fret_icons::IconId::new_static("lucide.captions"))
                .into_element(cx)
                .test_id("ui-gallery-card-notes-transcribe")])
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![list]).into_element(cx),
        shadcn::CardFooter::new(vec![avatars]).into_element(cx),
    ])
    .refine_layout(max_w_sm.clone().max_h(Px(400.0)))
    .into_element(cx)
    .test_id("ui-gallery-card-meeting-notes")
}
// endregion: example
