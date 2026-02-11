#![allow(
    clippy::arc_with_non_send_sync,
    clippy::collapsible_if,
    clippy::default_constructed_unit_structs,
    clippy::field_reassign_with_default,
    clippy::if_same_then_else,
    clippy::io_other_error,
    clippy::iter_overeager_cloned,
    clippy::let_and_return,
    clippy::let_unit_value,
    clippy::manual_is_multiple_of,
    clippy::redundant_closure,
    clippy::redundant_locals,
    clippy::reserve_after_initialization,
    clippy::too_many_arguments,
    clippy::unnecessary_cast,
    clippy::unnecessary_lazy_evaluations,
    clippy::useless_format
)]

use fret_app::{App, CommandId, Model};
use fret_code_editor as code_editor;
use fret_code_editor_view as code_editor_view;
use fret_code_view as code_view;
use fret_core::{
    AttributedText, CaretAffinity, Color as CoreColor, Corners, DrawOrder, Edges, FontId,
    FontWeight, ImageId, Point, Px, Rect, SceneOp, Size, TextConstraints, TextOverflow, TextSpan,
    TextStyle, TextWrap,
};
use fret_kit::prelude::ModelWatchExt as _;
use fret_markdown as markdown;
use fret_ui::Theme;
use fret_ui::element::{CanvasProps, SemanticsDecoration, StackProps};
use fret_ui::elements::ContinuousFrames;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui_ai as ui_ai;
use fret_ui_assets as ui_assets;
use fret_ui_kit::declarative::CachedSubtreeExt as _;
pub(super) use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::ui;
use fret_ui_material3 as material3;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use time::Date;

use crate::driver::UiGalleryImageSourceDemoAssets;
use crate::spec::*;

mod content;
mod models;
mod nav;
mod pages;
mod previews;

pub(crate) use content::content_view;
pub(crate) use models::UiGalleryModels;
pub(crate) use nav::sidebar_view;
use previews::material3::*;
use previews::pages::*;

fn preview_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct CardModels {
        email: Option<Model<String>>,
        password: Option<Model<String>>,
    }

    let email = cx.with_state(CardModels::default, |st| st.email.clone());
    let email = match email {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(CardModels::default, |st| st.email = Some(model.clone()));
            model
        }
    };

    let password = cx.with_state(CardModels::default, |st| st.password.clone());
    let password = match password {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(CardModels::default, |st| st.password = Some(model.clone()));
            model
        }
    };

    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let demo = {
        let card = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Login to your account")
                    .into_element(cx)
                    .test_id("ui-gallery-card-demo-title"),
                shadcn::CardDescription::new("Enter your email below to login to your account")
                    .into_element(cx)
                    .test_id("ui-gallery-card-demo-description"),
                shadcn::CardAction::new(vec![
                    shadcn::Button::new("Sign Up")
                        .variant(shadcn::ButtonVariant::Link)
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N6)
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    let email =
                        stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |cx| {
                            vec![
                                shadcn::Label::new("Email").into_element(cx),
                                shadcn::Input::new(email.clone())
                                    .a11y_label("Email")
                                    .placeholder("m@example.com")
                                    .into_element(cx),
                            ]
                        });

                    let password =
                        stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |cx| {
                            vec![
                                stack::hstack(
                                    cx,
                                    stack::HStackProps::default()
                                        .layout(LayoutRefinement::default().w_full())
                                        .justify_between()
                                        .items_center(),
                                    |cx| {
                                        vec![
                                            shadcn::Label::new("Password").into_element(cx),
                                            shadcn::Button::new("Forgot your password?")
                                                .variant(shadcn::ButtonVariant::Link)
                                                .size(shadcn::ButtonSize::Sm)
                                                .into_element(cx),
                                        ]
                                    },
                                ),
                                shadcn::Input::new(password.clone())
                                    .a11y_label("Password")
                                    .placeholder("••••••••")
                                    .into_element(cx),
                            ]
                        });

                    vec![email, password]
                },
            )])
            .into_element(cx),
            shadcn::CardFooter::new(vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    vec![
                        shadcn::Button::new("Login")
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                        shadcn::Button::new("Login with Google")
                            .variant(shadcn::ButtonVariant::Outline)
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                    ]
                },
            )])
            .into_element(cx),
        ])
        .refine_layout(max_w_sm.clone())
        .into_element(cx)
        .test_id("ui-gallery-card-demo");

        centered(cx, card)
    };

    let size = {
        let card = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Small Card").into_element(cx),
                shadcn::CardDescription::new("This card uses the small size variant.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![cx.text(
                "The card component supports a size prop that can be set to \"sm\" for a more compact appearance.",
            )])
            .into_element(cx),
            shadcn::CardFooter::new(vec![shadcn::Button::new("Action")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .refine_layout(LayoutRefinement::default().flex_1().w_full())
                .into_element(cx)])
            .into_element(cx),
        ])
        .size(shadcn::CardSize::Sm)
        .refine_layout(max_w_sm.clone())
        .into_element(cx);

        centered(cx, card)
    };

    let image = {
        let cover_bg = cx.with_theme(|theme| theme.color_required("muted"));

        let cover = shadcn::AspectRatio::new(
            16.0 / 9.0,
            cx.container(
                fret_ui::element::ContainerProps {
                    background: Some(cover_bg),
                    ..Default::default()
                },
                |cx| vec![cx.text("Event cover")],
            ),
        )
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

        let card = shadcn::Card::new(vec![
            cover,
            shadcn::CardHeader::new(vec![
                shadcn::CardAction::new(vec![
                    shadcn::Badge::new("Featured")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::CardTitle::new("Design systems meetup").into_element(cx),
                shadcn::CardDescription::new(
                    "A practical talk on component APIs, accessibility, and shipping faster.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardFooter::new(vec![
                shadcn::Button::new("View Event")
                    .refine_layout(LayoutRefinement::default().flex_1().w_full())
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_style(ChromeRefinement::default().pt(Space::N0))
        .refine_layout(max_w_sm.clone())
        .into_element(cx);

        centered(cx, card)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                section(cx, "Demo", demo),
                section(cx, "Size", size),
                section(cx, "Image", image),
            ]
        },
    )]
}

fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let row = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            move |_cx| children,
        )
    };

    let badge_icon = |cx: &mut ElementContext<'_, App>, name: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(
            cx,
            fret_icons::IconId::new_static(name),
            Some(Px(12.0)),
            Some(fg),
        )
    };

    let variants = {
        let children = vec![
            shadcn::Badge::new("Default").into_element(cx),
            shadcn::Badge::new("Secondary")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            shadcn::Badge::new("Destructive")
                .variant(shadcn::BadgeVariant::Destructive)
                .into_element(cx),
            shadcn::Badge::new("Outline")
                .variant(shadcn::BadgeVariant::Outline)
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "Variants", body)
    };

    let with_icon = {
        let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));
        let outline_fg = ColorRef::Color(theme.color_required("foreground"));

        let children = vec![
            shadcn::Badge::new("Verified")
                .variant(shadcn::BadgeVariant::Secondary)
                .children([badge_icon(cx, "lucide.badge-check", secondary_fg.clone())])
                .into_element(cx),
            shadcn::Badge::new("Bookmark")
                .variant(shadcn::BadgeVariant::Outline)
                .children([badge_icon(cx, "lucide.bookmark", outline_fg.clone())])
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "With Icon", body)
    };

    let with_spinner = {
        let destructive_fg = ColorRef::Color(theme.color_required("destructive-foreground"));
        let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));

        let children = vec![
            shadcn::Badge::new("Deleting")
                .variant(shadcn::BadgeVariant::Destructive)
                .children([shadcn::Spinner::new()
                    .color(destructive_fg.clone())
                    .into_element(cx)])
                .into_element(cx),
            shadcn::Badge::new("Generating")
                .variant(shadcn::BadgeVariant::Secondary)
                .children([shadcn::Spinner::new()
                    .color(secondary_fg.clone())
                    .into_element(cx)])
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "With Spinner", body)
    };

    let link = {
        let outline_fg = ColorRef::Color(theme.color_required("foreground"));

        let children = vec![
            shadcn::Badge::new("Open Link")
                .variant(shadcn::BadgeVariant::Outline)
                .children([badge_icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "Link", body)
    };

    let custom_colors = {
        let border_transparent =
            ChromeRefinement::default().border_color(ColorRef::Color(CoreColor::TRANSPARENT));

        let children = vec![
            shadcn::Badge::new("Blue")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.90,
                            g: 0.95,
                            b: 1.00,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Green")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.91,
                            g: 0.98,
                            b: 0.91,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Sky")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.90,
                            g: 0.97,
                            b: 1.00,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Purple")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.95,
                            g: 0.92,
                            b: 1.00,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Red")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 1.00,
                            g: 0.92,
                            b: 0.92,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "Custom Colors", body)
    };

    let rtl = {
        let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));

        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let children = vec![
                    shadcn::Badge::new("شارة").into_element(cx),
                    shadcn::Badge::new("ثانوي")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                    shadcn::Badge::new("متحقق")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .children([badge_icon(cx, "lucide.badge-check", secondary_fg.clone())])
                        .into_element(cx),
                ];
                row(cx, children)
            },
        );
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![variants, with_icon, with_spinner, link, custom_colors, rtl],
    )]
}

fn preview_avatar(
    cx: &mut ElementContext<'_, App>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let a = {
        let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
        let fallback = shadcn::AvatarFallback::new("FR")
            .when_image_missing_model(avatar_image.clone())
            .delay_ms(120)
            .into_element(cx);
        shadcn::Avatar::new([image, fallback]).into_element(cx)
    };

    let b =
        shadcn::Avatar::new([shadcn::AvatarFallback::new("WK").into_element(cx)]).into_element(cx);

    let c = shadcn::Avatar::new([shadcn::AvatarFallback::new("?").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_px(Px(48.0)).h_px(Px(48.0)))
        .into_element(cx);

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N3).items_center(),
            |_cx| [a, b, c],
        ),
        cx.text("Tip: use AvatarImage when you have an ImageId; AvatarFallback covers missing/slow loads."),
    ]
}

fn preview_image_object_fit(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    square_image: Model<Option<ImageId>>,
    wide_image: Model<Option<ImageId>>,
    tall_image: Model<Option<ImageId>>,
    streaming_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let image_cell = |cx: &mut ElementContext<'_, App>,
                      label: &'static str,
                      source: Model<Option<ImageId>>,
                      fit: fret_core::ViewportFit|
     -> AnyElement {
        let label = cx.text(label);
        let image = shadcn::MediaImage::model(source)
            .fit(fit)
            .loading(true)
            .refine_style(ChromeRefinement::default().rounded(Radius::Md))
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(96.0)))
            .into_element(cx)
            .test_id(format!("ui-gallery-image-object-fit-cell-{:?}", fit).to_lowercase());

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default()),
            |_cx| vec![label, image],
        )
    };

    let image_cell_opt = |cx: &mut ElementContext<'_, App>,
                          label: &'static str,
                          source: Option<ImageId>,
                          fit: fret_core::ViewportFit|
     -> AnyElement {
        let label = cx.text(label);
        let image = shadcn::MediaImage::maybe(source)
            .fit(fit)
            .loading(true)
            .refine_style(ChromeRefinement::default().rounded(Radius::Md))
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(96.0)))
            .into_element(cx)
            .test_id(format!("ui-gallery-image-object-fit-cell-source-{:?}", fit).to_lowercase());

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default()),
            |_cx| vec![label, image],
        )
    };

    let row = |cx: &mut ElementContext<'_, App>,
               title: &'static str,
               image: Model<Option<ImageId>>|
     -> AnyElement {
        let stretch = image_cell(
            cx,
            "Stretch",
            image.clone(),
            fret_core::ViewportFit::Stretch,
        );
        let contain = image_cell(
            cx,
            "Contain",
            image.clone(),
            fret_core::ViewportFit::Contain,
        );
        let cover = image_cell(cx, "Cover", image, fret_core::ViewportFit::Cover);

        let header = cx.text(title);
        let grid = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![stretch, contain, cover],
        );

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![header, grid],
        )
    };

    let mapping = {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    row(
                        cx,
                        "Wide source (320×180) → fixed 160×96",
                        wide_image.clone(),
                    ),
                    row(
                        cx,
                        "Tall source (180×320) → fixed 160×96",
                        tall_image.clone(),
                    ),
                    row(
                        cx,
                        "Square source (96×96) → fixed 160×96",
                        square_image.clone(),
                    ),
                ]
            },
        );
        section(cx, "SceneOp::Image fit mapping", body)
    };

    let image_source_demo = if let Some(assets) =
        cx.app.global::<UiGalleryImageSourceDemoAssets>().cloned()
    {
        let wide_state = ui_assets::use_image_source_state(cx.app, cx.window, &assets.wide_png);
        let tall_state = ui_assets::use_image_source_state(cx.app, cx.window, &assets.tall_png);
        let square_state = ui_assets::use_image_source_state(cx.app, cx.window, &assets.square_png);

        let status = cx.text(format!(
            "Status — wide: {:?}, tall: {:?}, square: {:?}",
            wide_state.status, tall_state.status, square_state.status
        ));

        let row_opt = |cx: &mut ElementContext<'_, App>,
                       title: &'static str,
                       image: Option<ImageId>|
         -> AnyElement {
            let stretch = image_cell_opt(cx, "Stretch", image, fret_core::ViewportFit::Stretch);
            let contain = image_cell_opt(cx, "Contain", image, fret_core::ViewportFit::Contain);
            let cover = image_cell_opt(cx, "Cover", image, fret_core::ViewportFit::Cover);

            let header = cx.text(title);
            let grid = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
                |_cx| vec![stretch, contain, cover],
            );

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
                |_cx| vec![header, grid],
            )
        };

        let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    vec![
                        cx.text("Loads PNG bytes via `ImageSource` → decode (background) → `ImageAssetCache` → ImageId."),
                        status,
                        row_opt(cx, "Wide source (PNG bytes)", wide_state.image),
                        row_opt(cx, "Tall source (PNG bytes)", tall_state.image),
                        row_opt(cx, "Square source (PNG bytes)", square_state.image),
                    ]
                },
            )
            .test_id("ui-gallery-image-object-fit-image-source-demo");

        section(cx, "Ecosystem ImageSource (bytes decode)", body)
    } else {
        let note = cx.text("ImageSource demo assets missing (expected UiGalleryDriver init).");
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![note],
        )
        .test_id("ui-gallery-image-object-fit-image-source-demo");
        section(cx, "Ecosystem ImageSource (bytes decode)", body)
    };

    let intrinsic = {
        let header = cx.text(
            "Policy-owned intrinsic aspect ratio (opt-in): width-only MediaImage can stamp a ratio from ImageMetadataStore.",
        );

        let wide_intrinsic = shadcn::MediaImage::model(wide_image.clone())
            .intrinsic_aspect_ratio_from_metadata(true)
            .fit(fret_core::ViewportFit::Contain)
            .loading(true)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Md)
                    .border_1()
                    .border_color(ColorRef::Color(theme.color_required("border"))),
            )
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
            .test_id("ui-gallery-image-object-fit-intrinsic-wide");

        let tall_intrinsic = shadcn::MediaImage::model(tall_image.clone())
            .intrinsic_aspect_ratio_from_metadata(true)
            .fit(fret_core::ViewportFit::Contain)
            .loading(true)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Md)
                    .border_1()
                    .border_color(ColorRef::Color(theme.color_required("border"))),
            )
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
            .test_id("ui-gallery-image-object-fit-intrinsic-tall");

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    header,
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N4)
                            .items_start()
                            .layout(LayoutRefinement::default().w_full()),
                        |_cx| vec![wide_intrinsic, tall_intrinsic],
                    ),
                ]
            },
        );
        section(cx, "Intrinsic aspect ratio (metadata)", body)
    };

    let streaming = {
        let note = cx.text(
            "Streaming updates: the demo pushes partial ImageUpdateRgba8 writes each frame (moving bar).",
        );
        let image = shadcn::MediaImage::model(streaming_image.clone())
            .fit(fret_core::ViewportFit::Cover)
            .loading(true)
            .refine_style(ChromeRefinement::default().rounded(Radius::Md))
            .refine_layout(LayoutRefinement::default().w_px(Px(320.0)).h_px(Px(200.0)))
            .into_element(cx)
            .test_id("ui-gallery-image-object-fit-streaming");

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![note, image],
        );
        section(cx, "Streaming updates", body)
    };

    let thumbnails = {
        let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());
        let len = 500usize;

        let list_layout = fret_ui::element::LayoutStyle {
            size: fret_ui::element::SizeStyle {
                width: fret_ui::element::Length::Fill,
                height: fret_ui::element::Length::Px(Px(360.0)),
                ..Default::default()
            },
            overflow: fret_ui::element::Overflow::Clip,
            ..Default::default()
        };

        let options = fret_ui::element::VirtualListOptions::known(Px(72.0), 10, |_index| Px(72.0));

        let wide = wide_image.clone();
        let tall = tall_image.clone();

        let list = cx.virtual_list_keyed_with_layout(
            list_layout,
            len,
            options,
            &scroll_handle,
            |i| i as fret_ui::ItemKey,
            move |cx, index| {
                let source = if index % 2 == 0 {
                    wide.clone()
                } else {
                    tall.clone()
                };
                let thumb = shadcn::MediaImage::model(source)
                    .fit(fret_core::ViewportFit::Cover)
                    .loading(true)
                    .refine_style(ChromeRefinement::default().rounded(Radius::Md))
                    .refine_layout(LayoutRefinement::default().w_px(Px(56.0)).h_px(Px(56.0)))
                    .into_element(cx);

                let title = cx.text(format!("Row {index}"));
                let subtitle = cx.text(if index % 2 == 0 {
                    "wide → cover"
                } else {
                    "tall → cover"
                });

                let row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N3)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        vec![
                            thumb,
                            stack::vstack(
                                cx,
                                stack::VStackProps::default()
                                    .gap(Space::N1)
                                    .items_start()
                                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                                |_cx| vec![title, subtitle],
                            ),
                        ]
                    },
                );

                cx.container(
                    decl_style::container_props(
                        theme,
                        ChromeRefinement::default()
                            .border_1()
                            .rounded(Radius::Md)
                            .p(Space::N2),
                        LayoutRefinement::default().w_full(),
                    ),
                    |_cx| vec![row],
                )
                .test_id(Arc::<str>::from(format!(
                    "ui-gallery-image-object-fit-row-{index}"
                )))
            },
        );

        let scroll_for_jump_80 = scroll_handle.clone();
        let on_jump_80: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            scroll_for_jump_80.scroll_to_item(80, fret_ui::scroll::ScrollStrategy::Start);
            host.request_redraw(action_cx.window);
        });

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    cx.text("Virtualized thumbnails list (alternating wide/tall sources)."),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .items_center()
                            .layout(LayoutRefinement::default()),
                        |cx| {
                            vec![
                                shadcn::Button::new("Jump 80")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-image-object-fit-jump-80")
                                    .on_activate(on_jump_80)
                                    .into_element(cx),
                            ]
                        },
                    ),
                    list.test_id("ui-gallery-image-object-fit-virtual-list"),
                ]
            },
        );

        section(cx, "Thumbnails (VirtualList)", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N8)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![mapping, image_source_demo, intrinsic, streaming, thumbnails],
    )]
}

fn preview_skeleton(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full(),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let round = |cx: &mut ElementContext<'_, App>, size: f32| {
        shadcn::Skeleton::new()
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(Px(size))
                    .h_px(Px(size))
                    .flex_shrink_0(),
            )
            .into_element(cx)
    };

    let demo = {
        let text_lines = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_px(Px(250.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(200.0)))
                        .into_element(cx),
                ]
            },
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| vec![round(cx, 48.0), text_lines],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-demo"),
        );

        let framed = shell(cx, row);
        let body = centered(cx, framed);
        section(cx, "Demo", body)
    };

    let avatar = {
        let text_lines = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_px(Px(150.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(100.0)))
                        .into_element(cx),
                ]
            },
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| vec![round(cx, 40.0), text_lines],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-avatar"),
        );

        let framed = shell(cx, row);
        let body = centered(cx, framed);
        section(cx, "Avatar", body)
    };

    let card = {
        let demo_card = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(170.0)))
                    .into_element(cx),
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(144.0)))
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-card"),
        );

        let body = centered(cx, demo_card);
        section(cx, "Card", body)
    };

    let text_section = {
        let text = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                        .into_element(cx),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-text"),
        );

        let framed = shell(cx, text);
        let body = centered(cx, framed);
        section(cx, "Text", body)
    };

    let form = {
        let row = |cx: &mut ElementContext<'_, App>, label_w: Px| {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full()),
                move |cx| {
                    vec![
                        shadcn::Skeleton::new()
                            .refine_layout(LayoutRefinement::default().w_px(label_w))
                            .into_element(cx),
                        shadcn::Skeleton::new()
                            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(32.0)))
                            .into_element(cx),
                    ]
                },
            )
        };

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    row(cx, Px(80.0)),
                    row(cx, Px(96.0)),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(96.0)).h_px(Px(32.0)))
                        .into_element(cx),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-form"),
        );

        let framed = shell(cx, content);
        let body = centered(cx, framed);
        section(cx, "Form", body)
    };

    let table = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))),
            |cx| {
                (0..5)
                    .map(|_| {
                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .gap(Space::N4)
                                .items_center()
                                .layout(LayoutRefinement::default().w_full()),
                            |cx| {
                                vec![
                                    shadcn::Skeleton::new()
                                        .refine_layout(
                                            LayoutRefinement::default().flex_1().min_w_0(),
                                        )
                                        .into_element(cx),
                                    shadcn::Skeleton::new()
                                        .refine_layout(LayoutRefinement::default().w_px(Px(96.0)))
                                        .into_element(cx),
                                    shadcn::Skeleton::new()
                                        .refine_layout(LayoutRefinement::default().w_px(Px(80.0)))
                                        .into_element(cx),
                                ]
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-table"),
        );

        let framed = shell(cx, content);
        let body = centered(cx, framed);
        section(cx, "Table", body)
    };

    let rtl = {
        let content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let text_lines = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .layout(LayoutRefinement::default().w_px(Px(250.0))),
                    |cx| {
                        vec![
                            shadcn::Skeleton::new()
                                .refine_layout(LayoutRefinement::default().w_full())
                                .into_element(cx),
                            shadcn::Skeleton::new()
                                .refine_layout(LayoutRefinement::default().w_px(Px(200.0)))
                                .into_element(cx),
                        ]
                    },
                );

                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N4).items_center(),
                    |cx| vec![round(cx, 48.0), text_lines],
                )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-rtl"),
        );

        let framed = shell(cx, content);
        let body = centered(cx, framed);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Use to show a placeholder while content is loading."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, avatar, card, text_section, form, table, rtl]
        }),
    ]
}

fn preview_scroll_area(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let demo = {
        let versions: Vec<Arc<str>> = (1..=50)
            .map(|idx| Arc::<str>::from(format!("v1.2.0-beta.{:02}", 51 - idx)))
            .collect();

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                let mut rows: Vec<AnyElement> = Vec::with_capacity(versions.len() * 2 + 1);
                rows.push(shadcn::typography::small(cx, "Tags"));
                for tag in versions {
                    rows.push(cx.text(tag));
                    rows.push(
                        shadcn::Separator::new()
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                    );
                }
                rows
            },
        );

        let scroll = shadcn::ScrollArea::new([content])
            .axis(fret_ui::element::ScrollAxis::Y)
            .refine_layout(LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-demo"),
            );

        let card = shell(cx, LayoutRefinement::default(), scroll);
        let body = centered(cx, card);
        section(cx, "Demo", body)
    };

    let horizontal = {
        let rail = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(760.0))),
            |cx| {
                let artists = [
                    "Ornella Binni",
                    "Tom Byrom",
                    "Vladimir Malyavko",
                    "Silvia Serra",
                ];
                artists
                    .iter()
                    .map(|artist| {
                        shadcn::Card::new(vec![
                            shadcn::CardContent::new(vec![
                                {
                                    let photo_props = cx.with_theme(|theme| {
                                        decl_style::container_props(
                                            theme,
                                            ChromeRefinement::default()
                                                .rounded(Radius::Md)
                                                .border_1()
                                                .bg(ColorRef::Color(theme.color_required("muted"))),
                                            LayoutRefinement::default()
                                                .w_px(Px(140.0))
                                                .h_px(Px(180.0)),
                                        )
                                    });
                                    cx.container(photo_props, |_cx| Vec::new())
                                },
                                shadcn::typography::muted(cx, format!("Photo by {artist}")),
                            ])
                            .into_element(cx),
                        ])
                        .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                        .into_element(cx)
                    })
                    .collect::<Vec<_>>()
            },
        );

        let scroll = shadcn::ScrollArea::new([rail])
            .axis(fret_ui::element::ScrollAxis::X)
            .refine_layout(LayoutRefinement::default().w_px(Px(384.0)).h_px(Px(280.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-horizontal"),
            );

        let card = shell(cx, LayoutRefinement::default(), scroll);
        let body = centered(cx, card);
        section(cx, "Horizontal", body)
    };

    let rtl = {
        let rtl_scroll = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let content = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        let mut rows: Vec<AnyElement> =
                            vec![shadcn::typography::small(cx, "العلامات")];
                        for idx in 1..=40 {
                            rows.push(cx.text(format!("v1.2.0-beta.{:02}", 41 - idx)));
                            rows.push(
                                shadcn::Separator::new()
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .into_element(cx),
                            );
                        }
                        rows
                    },
                );

                shadcn::ScrollArea::new([content])
                    .axis(fret_ui::element::ScrollAxis::Y)
                    .refine_layout(LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)))
                    .into_element(cx)
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-scroll-area-rtl"),
        );

        let card = shell(cx, LayoutRefinement::default(), rtl_scroll);
        let body = centered(cx, card);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Scrollable region with custom scrollbars and nested content."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, horizontal, rtl]
        }),
    ]
}

fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_tooltip(cx)
}

fn preview_slider(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.slider_page", |cx| {
        #[derive(Default)]
        struct SliderPageState {
            last_commit: Option<Model<Vec<f32>>>,
            controlled_values: Option<Model<Vec<f32>>>,
        }

        let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
            stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .justify_center(),
                move |_cx| [body],
            )
        };

        let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
                move |cx| vec![shadcn::typography::h4(cx, title), body],
            )
        };

        let max_width_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

        let last_commit = cx.with_state(SliderPageState::default, |st| st.last_commit.clone());
        let last_commit = match last_commit {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(Vec::<f32>::new());
                cx.with_state(SliderPageState::default, |st| {
                    st.last_commit = Some(model.clone());
                });
                model
            }
        };

        let controlled_values =
            cx.with_state(SliderPageState::default, |st| st.controlled_values.clone());
        let controlled_values = match controlled_values {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(vec![0.3, 0.7]);
                cx.with_state(SliderPageState::default, |st| {
                    st.controlled_values = Some(model.clone());
                });
                model
            }
        };

        let demo = cx.keyed("ui_gallery.slider.demo", |cx| {
            let last_commit_for_cb = last_commit.clone();
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![75.0])
                .range(0.0, 100.0)
                .test_id("ui-gallery-slider-single")
                .a11y_label("Slider")
                .refine_layout(max_width_xs.clone())
                .on_value_commit(move |host, _cx, values| {
                    let _ = host.models_mut().update(&last_commit_for_cb, |v| {
                        *v = values;
                    });
                })
                .into_element(cx);

            let last_commit_values = cx
                .watch_model(&last_commit)
                .layout()
                .cloned()
                .unwrap_or_default();
            let last_commit_text = if last_commit_values.is_empty() {
                "<none>".to_string()
            } else {
                format!("{last_commit_values:?}")
            };
            let meta = shadcn::typography::muted(cx, format!("onValueCommit: {last_commit_text}"));

            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![slider, meta],
            );
            let body = centered(cx, body);
            section(cx, "Demo", body)
        });

        let range = cx.keyed("ui_gallery.slider.range", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![25.0, 50.0])
                .range(0.0, 100.0)
                .step(5.0)
                .test_id("ui-gallery-slider-range")
                .a11y_label("Range slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Range", body)
        });

        let multiple = cx.keyed("ui_gallery.slider.multiple", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![10.0, 20.0, 70.0])
                .range(0.0, 100.0)
                .step(10.0)
                .test_id("ui-gallery-slider-multiple")
                .a11y_label("Multiple thumbs slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Multiple Thumbs", body)
        });

        let vertical = cx.keyed("ui_gallery.slider.vertical", |cx| {
            let a = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
                .range(0.0, 100.0)
                .step(1.0)
                .orientation(fret_ui_kit::primitives::slider::SliderOrientation::Vertical)
                .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
                .test_id("ui-gallery-slider-vertical")
                .a11y_label("Vertical slider")
                .into_element(cx);

            let b = shadcn::Slider::new_controllable(cx, None, || vec![25.0])
                .range(0.0, 100.0)
                .step(1.0)
                .orientation(fret_ui_kit::primitives::slider::SliderOrientation::Vertical)
                .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
                .a11y_label("Vertical slider")
                .into_element(cx);

            let body = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N6)
                    .items_center()
                    .justify_center()
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![a, b],
            );

            section(cx, "Vertical", body)
        });

        let controlled = cx.keyed("ui_gallery.slider.controlled", |cx| {
            let values_snapshot = cx
                .watch_model(&controlled_values)
                .layout()
                .cloned()
                .unwrap_or_default();
            let values_text = values_snapshot
                .iter()
                .map(|v| format!("{v:.1}"))
                .collect::<Vec<_>>()
                .join(", ");

            let header = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .items_center()
                    .justify_between(),
                |cx| {
                    vec![
                        shadcn::Label::new("Temperature").into_element(cx),
                        shadcn::typography::muted(cx, values_text),
                    ]
                },
            );
            let slider = shadcn::Slider::new(controlled_values.clone())
                .range(0.0, 1.0)
                .step(0.1)
                .test_id("ui-gallery-slider-controlled")
                .a11y_label("Temperature")
                .into_element(cx);

            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![header, slider],
            );

            let body = centered(cx, body);
            section(cx, "Controlled", body)
        });

        let disabled = cx.keyed("ui_gallery.slider.disabled", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
                .range(0.0, 100.0)
                .step(1.0)
                .disabled(true)
                .test_id("ui-gallery-slider-disabled")
                .a11y_label("Disabled slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Disabled", body)
        });

        let rtl = cx.keyed("ui_gallery.slider.rtl", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![75.0])
                .range(0.0, 100.0)
                .step(1.0)
                .dir(fret_ui_kit::primitives::direction::LayoutDirection::Rtl)
                .test_id("ui-gallery-slider-rtl")
                .a11y_label("RTL slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "RTL", body)
        });

        let inverted = cx.keyed("ui_gallery.slider.inverted", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![25.0])
                .range(0.0, 100.0)
                .step(1.0)
                .inverted(true)
                .test_id("ui-gallery-slider-inverted")
                .a11y_label("Inverted slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Extras: Inverted", body)
        });

        vec![
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N6)
                    .items_start(),
                |_cx| vec![
                    demo,
                    range,
                    multiple,
                    vertical,
                    controlled,
                    disabled,
                    rtl,
                    inverted,
                ],
            ),
            shadcn::typography::muted(
                cx,
                "Note: demo/range/multiple/vertical/disabled/RTL are uncontrolled (element state). Controlled uses a shared model."
                    .to_string(),
            ),
        ]
    })
}

fn preview_icons(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_icons::ids;

    let icon_cell =
        |cx: &mut ElementContext<'_, App>, label: &str, icon_id: IconId| -> AnyElement {
            let row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2)
                    .items_center(),
                |cx| {
                    vec![
                        icon::icon_with(cx, icon_id, Some(Px(16.0)), None),
                        cx.text(label),
                    ]
                },
            );

            let theme = Theme::global(&*cx.app);
            cx.container(
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default()
                        .rounded(Radius::Md)
                        .border_1()
                        .p(Space::N3),
                    LayoutRefinement::default().w_full(),
                ),
                |_cx| [row],
            )
        };

    let grid = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                icon_cell(cx, "ui.search", ids::ui::SEARCH),
                icon_cell(cx, "ui.settings", ids::ui::SETTINGS),
                icon_cell(cx, "ui.chevron.right", ids::ui::CHEVRON_RIGHT),
                icon_cell(cx, "ui.close", ids::ui::CLOSE),
                icon_cell(
                    cx,
                    "lucide.loader-circle",
                    IconId::new_static("lucide.loader-circle"),
                ),
            ]
        },
    );

    let spinner_row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Spinner::new().into_element(cx),
                shadcn::Spinner::new().speed(0.0).into_element(cx),
                cx.text("Spinner (animated / static)"),
            ]
        },
    );

    vec![grid, spinner_row]
}

fn preview_field(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_field(cx)
}

fn preview_forms(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_forms(cx, text_input, text_area, checkbox, switch)
}

fn preview_select(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    let select = shadcn::Select::new(value.clone(), open)
        .trigger_test_id("ui-gallery-select-trigger")
        .placeholder("Pick a fruit")
        .items(
            [
                shadcn::SelectItem::new("apple", "Apple").test_id("ui-gallery-select-item-apple"),
                shadcn::SelectItem::new("banana", "Banana")
                    .test_id("ui-gallery-select-item-banana"),
                shadcn::SelectItem::new("orange", "Orange")
                    .test_id("ui-gallery-select-item-orange"),
            ]
            .into_iter()
            .chain((1..=40).map(|i| {
                let value: Arc<str> = Arc::from(format!("item-{i:02}"));
                let label: Arc<str> = Arc::from(format!("Item {i:02}"));
                let test_id: Arc<str> = Arc::from(format!("ui-gallery-select-item-{value}"));
                shadcn::SelectItem::new(value, label)
                    .test_id(test_id)
                    .disabled(i == 15)
            })),
        )
        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
        .into_element(cx);

    let selected_label = cx
        .scope(|cx| {
            let selected: Arc<str> = cx
                .get_model_cloned(&value, fret_ui::Invalidation::Paint)
                .unwrap_or_default()
                .unwrap_or_else(|| Arc::<str>::from("<none>"));

            fret_ui::element::AnyElement::new(
                cx.root_id(),
                fret_ui::element::ElementKind::Text(fret_ui::element::TextProps::new(format!(
                    "Selected: {selected}"
                ))),
                Vec::new(),
            )
        })
        .attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .test_id("ui-gallery-select-selected-label"),
        );

    vec![select, selected_label]
}

fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    pages::preview_combobox(cx, value, open, query)
}

fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    pages::preview_date_picker(cx, open, month, selected)
}

fn preview_resizable(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    h_fractions: Model<Vec<f32>>,
    v_fractions: Model<Vec<f32>>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct ResizableModels {
        vertical_fractions: Option<Model<Vec<f32>>>,
        handle_fractions: Option<Model<Vec<f32>>>,
        rtl_h_fractions: Option<Model<Vec<f32>>>,
        rtl_v_fractions: Option<Model<Vec<f32>>>,
    }

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let box_group =
        |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
            cx.container(
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default().border_1().rounded(Radius::Lg),
                    layout,
                ),
                move |_cx| [body],
            )
        };

    let panel = |cx: &mut ElementContext<'_, App>, label: &'static str, height: Option<Px>| {
        let layout = match height {
            Some(h) => LayoutRefinement::default().w_full().h_px(h),
            None => LayoutRefinement::default().w_full().h_full(),
        };

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().h_full())
                .items_center()
                .justify_center(),
            move |cx| vec![cx.text(label)],
        );

        cx.container(
            decl_style::container_props(theme, ChromeRefinement::default().p(Space::N6), layout),
            move |_cx| [body],
        )
    };

    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(448.0));

    let state = cx.with_state(ResizableModels::default, |st| st.clone());
    let vertical_fractions = match state.vertical_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
            cx.with_state(ResizableModels::default, |st| {
                st.vertical_fractions = Some(model.clone())
            });
            model
        }
    };
    let handle_fractions = match state.handle_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
            cx.with_state(ResizableModels::default, |st| {
                st.handle_fractions = Some(model.clone())
            });
            model
        }
    };
    let rtl_h_fractions = match state.rtl_h_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.5, 0.5]);
            cx.with_state(ResizableModels::default, |st| {
                st.rtl_h_fractions = Some(model.clone())
            });
            model
        }
    };
    let rtl_v_fractions = match state.rtl_v_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
            cx.with_state(ResizableModels::default, |st| {
                st.rtl_v_fractions = Some(model.clone())
            });
            model
        }
    };

    let demo = {
        let nested_vertical = shadcn::ResizablePanelGroup::new(v_fractions)
            .axis(fret_core::Axis::Vertical)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "Two", None)]).into(),
                shadcn::ResizableHandle::new().into(),
                shadcn::ResizablePanel::new([panel(cx, "Three", None)]).into(),
            ])
            .into_element(cx);

        let group = shadcn::ResizablePanelGroup::new(h_fractions)
            .axis(fret_core::Axis::Horizontal)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "One", Some(Px(200.0)))]).into(),
                shadcn::ResizableHandle::new().into(),
                shadcn::ResizablePanel::new([nested_vertical]).into(),
            ])
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .label("Debug:ui-gallery:resizable-panels")
                    .test_id("ui-gallery-resizable-panels"),
            );

        let group = box_group(
            cx,
            max_w_sm
                .clone()
                .merge(LayoutRefinement::default().h_px(Px(320.0))),
            group,
        );

        let body = centered(cx, group);
        section(cx, "Demo", body)
    };

    let vertical = {
        let group = shadcn::ResizablePanelGroup::new(vertical_fractions)
            .axis(fret_core::Axis::Vertical)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "Header", None)]).into(),
                shadcn::ResizableHandle::new().into(),
                shadcn::ResizablePanel::new([panel(cx, "Content", None)]).into(),
            ])
            .into_element(cx);

        let group = box_group(
            cx,
            max_w_sm
                .clone()
                .merge(LayoutRefinement::default().h_px(Px(200.0))),
            group,
        );

        let body = centered(cx, group);
        section(cx, "Vertical", body)
    };

    let handle = {
        let group = shadcn::ResizablePanelGroup::new(handle_fractions)
            .axis(fret_core::Axis::Horizontal)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "Sidebar", None)]).into(),
                shadcn::ResizableHandle::new().with_handle(true).into(),
                shadcn::ResizablePanel::new([panel(cx, "Content", None)]).into(),
            ])
            .into_element(cx);

        let group = box_group(
            cx,
            max_w_md
                .clone()
                .merge(LayoutRefinement::default().h_px(Px(200.0))),
            group,
        );

        let body = centered(cx, group);
        section(cx, "Handle", body)
    };

    let rtl = {
        let group = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let nested_vertical = shadcn::ResizablePanelGroup::new(rtl_v_fractions.clone())
                    .axis(fret_core::Axis::Vertical)
                    .entries([
                        shadcn::ResizablePanel::new([panel(cx, "اثنان", None)]).into(),
                        shadcn::ResizableHandle::new().with_handle(true).into(),
                        shadcn::ResizablePanel::new([panel(cx, "ثلاثة", None)]).into(),
                    ])
                    .into_element(cx);

                shadcn::ResizablePanelGroup::new(rtl_h_fractions.clone())
                    .axis(fret_core::Axis::Horizontal)
                    .entries([
                        shadcn::ResizablePanel::new([panel(cx, "واحد", Some(Px(200.0)))]).into(),
                        shadcn::ResizableHandle::new().with_handle(true).into(),
                        shadcn::ResizablePanel::new([nested_vertical]).into(),
                    ])
                    .into_element(cx)
            },
        );

        let group = box_group(
            cx,
            max_w_sm
                .clone()
                .merge(LayoutRefinement::default().h_px(Px(320.0))),
            group,
        );

        let body = centered(cx, group);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Drag the handles to resize panels."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, vertical, handle, rtl]
        }),
    ]
}

#[derive(Debug, Clone)]
struct DemoProcessRow {
    id: u64,
    name: Arc<str>,
    status: Arc<str>,
    cpu: u64,
    mem_mb: u64,
}

#[derive(Debug, Clone)]
struct DemoProcessTableAssets {
    data: Arc<[DemoProcessRow]>,
    columns: Arc<[fret_ui_headless::table::ColumnDef<DemoProcessRow>]>,
}

fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    pages::preview_data_table(cx, state)
}

fn preview_data_table_legacy(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let assets = cx.with_state(
        || {
            let data: Arc<[DemoProcessRow]> = Arc::from(vec![
                DemoProcessRow {
                    id: 1,
                    name: Arc::from("Renderer"),
                    status: Arc::from("Running"),
                    cpu: 12,
                    mem_mb: 420,
                },
                DemoProcessRow {
                    id: 2,
                    name: Arc::from("Asset Cache"),
                    status: Arc::from("Idle"),
                    cpu: 0,
                    mem_mb: 128,
                },
                DemoProcessRow {
                    id: 3,
                    name: Arc::from("Indexer"),
                    status: Arc::from("Running"),
                    cpu: 38,
                    mem_mb: 860,
                },
                DemoProcessRow {
                    id: 4,
                    name: Arc::from("Spellcheck"),
                    status: Arc::from("Disabled"),
                    cpu: 0,
                    mem_mb: 0,
                },
                DemoProcessRow {
                    id: 5,
                    name: Arc::from("Language Server"),
                    status: Arc::from("Running"),
                    cpu: 7,
                    mem_mb: 512,
                },
            ]);

            let columns: Arc<[fret_ui_headless::table::ColumnDef<DemoProcessRow>]> =
                Arc::from(vec![
                    fret_ui_headless::table::ColumnDef::new("name")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.name.cmp(&b.name))
                        .size(220.0),
                    fret_ui_headless::table::ColumnDef::new("status")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.status.cmp(&b.status))
                        .size(140.0),
                    fret_ui_headless::table::ColumnDef::new("cpu%")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.cpu.cmp(&b.cpu))
                        .size(90.0),
                    fret_ui_headless::table::ColumnDef::new("mem_mb")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.mem_mb.cmp(&b.mem_mb))
                        .size(110.0),
                ]);

            DemoProcessTableAssets { data, columns }
        },
        |st| st.clone(),
    );

    let selected_count = cx
        .app
        .models()
        .read(&state, |st| st.row_selection.len())
        .ok()
        .unwrap_or(0);
    let sorting = cx
        .app
        .models()
        .read(&state, |st| {
            st.sorting.first().map(|s| (s.column.clone(), s.desc))
        })
        .ok()
        .flatten();

    let sorting_text: Arc<str> = sorting
        .map(|(col, desc)| {
            Arc::<str>::from(format!(
                "Sorting: {} {}",
                col,
                if desc { "desc" } else { "asc" }
            ))
        })
        .unwrap_or_else(|| Arc::<str>::from("Sorting: <none>"));

    let normalize_col_id =
        |id: &str| -> Arc<str> { Arc::<str>::from(id.replace('%', "pct").replace('_', "-")) };

    let toolbar = shadcn::DataTableToolbar::new(
        state.clone(),
        assets.columns.clone(),
        |col: &fret_ui_headless::table::ColumnDef<DemoProcessRow>| col.id.clone(),
    )
    .into_element(cx);

    let table = shadcn::DataTable::new()
        .row_height(Px(36.0))
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
        .into_element(
            cx,
            assets.data.clone(),
            1,
            state.clone(),
            assets.columns.clone(),
            |row, _index, _parent| fret_ui_headless::table::RowKey(row.id),
            |col| col.id.clone(),
            move |cx, col, row| {
                let col_id = normalize_col_id(col.id.as_ref());
                let cell = match col.id.as_ref() {
                    "name" => cx.text(row.name.as_ref()),
                    "status" => cx.text(row.status.as_ref()),
                    "cpu%" => cx.text(format!("{}%", row.cpu)),
                    "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                    _ => cx.text("?"),
                };

                cell.test_id(Arc::<str>::from(format!(
                    "ui-gallery-data-table-cell-{}-{}",
                    row.id, col_id
                )))
            },
        );

    let table = table.test_id("ui-gallery-data-table-root");

    vec![
        cx.text("Click header to sort; click row to toggle selection."),
        cx.text(format!("Selected rows: {selected_count}")),
        cx.text(sorting_text.as_ref()),
        toolbar,
        table,
    ]
}

fn preview_data_table_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    _state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    use fret_ui_headless::table::{ColumnDef, RowKey, SortSpec};

    let variable_height = std::env::var_os("FRET_UI_GALLERY_DATA_TABLE_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();
    let keep_alive: usize = std::env::var("FRET_UI_GALLERY_DATA_TABLE_KEEP_ALIVE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    #[derive(Debug, Clone)]
    struct Row {
        id: u64,
        name: Arc<str>,
        status: Arc<str>,
        cpu: u64,
        mem_mb: u64,
    }

    let (data, columns) = cx.with_state(
        || {
            let mut rows: Vec<Row> = Vec::with_capacity(50_000);
            for i in 0..50_000u64 {
                let status = match i % 4 {
                    0 => "Running",
                    1 => "Idle",
                    2 => "Sleeping",
                    _ => "Blocked",
                };
                rows.push(Row {
                    id: i,
                    name: Arc::from(format!("Process {i}")),
                    status: Arc::from(status),
                    cpu: (i * 7) % 100,
                    mem_mb: 32 + ((i * 13) % 4096),
                });
            }

            let columns: Arc<[ColumnDef<Row>]> = Arc::from(vec![
                ColumnDef::new("name")
                    .sort_by(|a: &Row, b: &Row| a.name.cmp(&b.name))
                    .filter_by(|row: &Row, q| row.name.as_ref().contains(q))
                    .size(220.0),
                ColumnDef::new("status")
                    .sort_by(|a: &Row, b: &Row| a.status.cmp(&b.status))
                    .filter_by_with_meta(|row: &Row, value: &serde_json::Value, _add_meta| {
                        match value {
                            serde_json::Value::String(s) => row.status.as_ref() == s,
                            serde_json::Value::Array(items) => items
                                .iter()
                                .filter_map(|it| it.as_str())
                                .any(|s| row.status.as_ref() == s),
                            _ => false,
                        }
                    })
                    .facet_key_by(|row: &Row| match row.status.as_ref() {
                        "Running" => 1,
                        "Idle" => 2,
                        "Sleeping" => 3,
                        "Blocked" => 4,
                        _ => 0,
                    })
                    .facet_str_by(|row: &Row| row.status.as_ref())
                    .size(140.0),
                ColumnDef::new("cpu%")
                    .sort_by(|a: &Row, b: &Row| a.cpu.cmp(&b.cpu))
                    .size(90.0),
                ColumnDef::new("mem_mb")
                    .sort_by(|a: &Row, b: &Row| a.mem_mb.cmp(&b.mem_mb))
                    .size(110.0),
            ]);

            (Arc::<[Row]>::from(rows), columns)
        },
        |(data, columns)| (data.clone(), columns.clone()),
    );

    #[derive(Default)]
    struct DataTableTortureModels {
        state: Option<Model<fret_ui_headless::table::TableState>>,
    }

    let state = cx.with_state(DataTableTortureModels::default, |st| st.state.clone());
    let state = match state {
        Some(state) => state,
        None => {
            let mut state_value = fret_ui_headless::table::TableState::default();
            state_value.pagination.page_size = data.len();
            state_value.pagination.page_index = 0;
            let state = cx.app.models_mut().insert(state_value);
            cx.with_state(DataTableTortureModels::default, |st| {
                st.state = Some(state.clone());
            });
            state
        }
    };

    let sorting: Vec<SortSpec> = cx
        .app
        .models()
        .read(&state, |st| st.sorting.clone())
        .ok()
        .unwrap_or_default();
    let sorting_text: Arc<str> = if sorting.is_empty() {
        Arc::<str>::from("Sorting: <none>")
    } else {
        let parts: Vec<String> = sorting
            .iter()
            .map(|s| format!("{} {}", s.column, if s.desc { "desc" } else { "asc" }))
            .collect();
        Arc::<str>::from(format!("Sorting: {}", parts.join(", ")))
    };

    let pinning_text: Arc<str> = {
        let pinning = cx
            .app
            .models()
            .read(&state, |st| st.column_pinning.clone())
            .ok()
            .unwrap_or_default();
        if pinning.left.is_empty() && pinning.right.is_empty() {
            Arc::<str>::from("Pinning: <none>")
        } else {
            let left = pinning
                .left
                .iter()
                .map(|v| v.as_ref().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let right = pinning
                .right
                .iter()
                .map(|v| v.as_ref().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Arc::<str>::from(format!("Pinning: left=[{left}] right=[{right}]"))
        }
    };

    let global_filter_text: Arc<str> = {
        let global_filter = cx
            .app
            .models()
            .read(&state, |st| st.global_filter.clone())
            .ok()
            .flatten();
        match global_filter {
            None => Arc::<str>::from("GlobalFilter: <none>"),
            Some(v) => {
                if let Some(s) = v.as_str() {
                    Arc::<str>::from(format!("GlobalFilter: {s}"))
                } else {
                    Arc::<str>::from(format!("GlobalFilter: {v}"))
                }
            }
        }
    };

    let name_filter_text: Arc<str> = {
        let value = cx
            .app
            .models()
            .read(&state, |st| {
                st.column_filters
                    .iter()
                    .find(|f| f.column.as_ref() == "name")
                    .map(|f| f.value.clone())
            })
            .ok()
            .flatten();
        match value {
            None => Arc::<str>::from("NameFilter: <none>"),
            Some(v) => {
                if let Some(s) = v.as_str() {
                    Arc::<str>::from(format!("NameFilter: {s}"))
                } else {
                    Arc::<str>::from(format!("NameFilter: {v}"))
                }
            }
        }
    };

    let status_filter_text: Arc<str> = {
        let value = cx
            .app
            .models()
            .read(&state, |st| {
                st.column_filters
                    .iter()
                    .find(|f| f.column.as_ref() == "status")
                    .map(|f| f.value.clone())
            })
            .ok()
            .flatten();
        match value {
            None => Arc::<str>::from("StatusFilter: <none>"),
            Some(serde_json::Value::String(s)) => Arc::<str>::from(format!("StatusFilter: {s}")),
            Some(serde_json::Value::Array(items)) => {
                let parts: Vec<&str> = items.iter().filter_map(|it| it.as_str()).collect();
                if parts.is_empty() {
                    Arc::<str>::from("StatusFilter: <none>")
                } else {
                    Arc::<str>::from(format!("StatusFilter: {}", parts.join(", ")))
                }
            }
            Some(v) => Arc::<str>::from(format!("StatusFilter: {v}")),
        }
    };

    let toolbar_columns = columns.clone();
    let toolbar =
        shadcn::DataTableToolbar::new(state.clone(), toolbar_columns, |col: &ColumnDef<Row>| {
            Arc::<str>::from(col.id.as_ref())
        })
        .column_filter("name")
        .column_filter_placeholder("Filter name...")
        .column_filter_a11y_label("Name filter")
        .faceted_filter(
            "status",
            "Status",
            Arc::<[Arc<str>]>::from(vec![
                Arc::<str>::from("Running"),
                Arc::<str>::from("Idle"),
                Arc::<str>::from("Sleeping"),
                Arc::<str>::from("Blocked"),
            ]),
        );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline perf harness for a virtualized business table (TanStack-aligned headless engine + VirtualList)."),
                cx.text("Use scripted scroll + bundle stats to validate cache-root reuse and prepaint-driven windowing refactors."),
                cx.text(sorting_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(sorting_text.clone())
                        .test_id("ui-gallery-data-table-torture-sorting"),
                ),
                cx.text(pinning_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(pinning_text.clone())
                        .test_id("ui-gallery-data-table-torture-pinning"),
                ),
                cx.text(global_filter_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(global_filter_text.clone())
                        .test_id("ui-gallery-data-table-torture-global-filter"),
                ),
                cx.text(name_filter_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(name_filter_text.clone())
                        .test_id("ui-gallery-data-table-torture-name-filter"),
                ),
                cx.text(status_filter_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(status_filter_text.clone())
                        .test_id("ui-gallery-data-table-torture-status-filter"),
                ),
                toolbar.clone().into_element(cx),
            ]
        },
    );

    let state_for_table = state.clone();
    let table =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let retained = std::env::var_os("FRET_UI_GALLERY_DATA_TABLE_RETAINED").is_some();
            let data_table = if retained {
                let mut t = shadcn::DataTable::new();
                if keep_alive > 0 {
                    t = t.keep_alive(keep_alive);
                }
                t.overscan(10)
                    .row_height(Px(28.0))
                    .measure_rows(variable_height)
                    .column_actions_menu(true)
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(420.0)))
                    .into_element_retained(
                        cx,
                        data.clone(),
                        1,
                        state_for_table.clone(),
                        columns.clone(),
                        |row, _index, _parent| RowKey(row.id),
                        |col| Arc::<str>::from(col.id.as_ref()),
                        move |cx, col, row| match col.id.as_ref() {
                            "name" => {
                                if variable_height && row.id % 15 == 0 {
                                    stack::vstack(
                                        cx,
                                        stack::VStackProps::default().gap(Space::N0),
                                        |cx| {
                                            vec![
                                                cx.text(row.name.as_ref()),
                                                cx.text(format!(
                                                    "Details: id={} cpu={} mem={}",
                                                    row.id, row.cpu, row.mem_mb
                                                )),
                                            ]
                                        },
                                    )
                                } else {
                                    cx.text(row.name.as_ref())
                                }
                            }
                            "status" => cx.text(row.status.as_ref()),
                            "cpu%" => cx.text(format!("{}%", row.cpu)),
                            "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                            _ => cx.text("?"),
                        },
                        Some(Arc::<str>::from("ui-gallery-data-table-header-")),
                        Some(Arc::<str>::from("ui-gallery-data-table-row-")),
                    )
            } else {
                let mut t = shadcn::DataTable::new();
                if keep_alive > 0 {
                    t = t.keep_alive(keep_alive);
                }
                t.overscan(10)
                    .row_height(Px(28.0))
                    .measure_rows(variable_height)
                    .column_actions_menu(true)
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(420.0)))
                    .into_element(
                        cx,
                        data.clone(),
                        1,
                        state,
                        columns.clone(),
                        |row, _index, _parent| RowKey(row.id),
                        |col| Arc::<str>::from(col.id.as_ref()),
                        move |cx, col, row| match col.id.as_ref() {
                            "name" => {
                                if variable_height && row.id % 15 == 0 {
                                    stack::vstack(
                                        cx,
                                        stack::VStackProps::default().gap(Space::N0),
                                        |cx| {
                                            vec![
                                                cx.text(row.name.as_ref()),
                                                cx.text(format!(
                                                    "Details: id={} cpu={} mem={}",
                                                    row.id, row.cpu, row.mem_mb
                                                )),
                                            ]
                                        },
                                    )
                                } else {
                                    cx.text(row.name.as_ref())
                                }
                            }
                            "status" => cx.text(row.status.as_ref()),
                            "cpu%" => cx.text(format!("{}%", row.cpu)),
                            "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                            _ => cx.text("?"),
                        },
                    )
            };

            vec![
                data_table.attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Group)
                        .test_id("ui-gallery-data-table-torture-root"),
                ),
            ]
        });

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![header, cx.container(container_props, |_cx| vec![table])]
}

fn preview_tree_torture(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    use std::collections::HashSet;

    use fret_ui_kit::TreeItem;
    use fret_ui_kit::TreeState;

    let variable_height = std::env::var_os("FRET_UI_GALLERY_TREE_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();

    #[derive(Default)]
    struct TreeTortureModels {
        items: Option<Model<Vec<TreeItem>>>,
        state: Option<Model<TreeState>>,
    }

    let (items, state) = cx.with_state(TreeTortureModels::default, |st| {
        (st.items.clone(), st.state.clone())
    });
    let (items, state) = match (items, state) {
        (Some(items), Some(state)) => (items, state),
        _ => {
            let (items_value, state_value) = {
                let root_count = 200u64;
                let folders_per_root = 10u64;
                let leaves_per_folder = 25u64;

                let mut expanded: HashSet<u64> = HashSet::new();
                let mut roots: Vec<TreeItem> = Vec::with_capacity(root_count as usize);

                for r in 0..root_count {
                    let root_id = r;
                    expanded.insert(root_id);

                    let mut folders: Vec<TreeItem> = Vec::with_capacity(folders_per_root as usize);
                    for f in 0..folders_per_root {
                        let folder_id = 1_000_000 + r * 100 + f;
                        expanded.insert(folder_id);

                        let mut leaves: Vec<TreeItem> =
                            Vec::with_capacity(leaves_per_folder as usize);
                        for l in 0..leaves_per_folder {
                            let leaf_id = 2_000_000 + r * 10_000 + f * 100 + l;
                            let label = if variable_height && leaf_id % 15 == 0 {
                                format!(
                                    "Leaf {r}/{f}/{l} (id={leaf_id})\nDetails: id={} seed={}",
                                    leaf_id,
                                    leaf_id.wrapping_mul(2654435761)
                                )
                            } else {
                                format!("Leaf {r}/{f}/{l} (id={leaf_id})")
                            };
                            leaves.push(TreeItem::new(leaf_id, label).disabled(leaf_id % 97 == 0));
                        }

                        folders.push(
                            TreeItem::new(folder_id, format!("Folder {r}/{f}")).children(leaves),
                        );
                    }

                    roots.push(TreeItem::new(root_id, format!("Root {r}")).children(folders));
                }

                (
                    roots,
                    TreeState {
                        selected: None,
                        expanded,
                    },
                )
            };

            let items = cx.app.models_mut().insert(items_value);
            let state = cx.app.models_mut().insert(state_value);
            cx.with_state(TreeTortureModels::default, |st| {
                st.items = Some(items.clone());
                st.state = Some(state.clone());
            });
            (items, state)
        }
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline perf harness for a virtualized tree (expand/collapse + selection + scroll)."),
                cx.text("Use scripted scroll + bundle stats to validate cache-root reuse and prepaint-driven windowing refactors."),
            ]
        },
    );

    let tree = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        let retained = std::env::var_os("FRET_UI_GALLERY_TREE_RETAINED")
            .filter(|v| !v.is_empty())
            .is_some();

        let tree = if retained {
            if variable_height {
                fret_ui_kit::declarative::tree::tree_view_retained_with_measure_mode(
                    cx,
                    items,
                    state,
                    fret_ui_kit::Size::Medium,
                    fret_ui::element::VirtualListMeasureMode::Measured,
                    Some(Arc::<str>::from("ui-gallery-tree-row")),
                )
            } else {
                fret_ui_kit::declarative::tree::tree_view_retained(
                    cx,
                    items,
                    state,
                    fret_ui_kit::Size::Medium,
                    Some(Arc::<str>::from("ui-gallery-tree-row")),
                )
            }
        } else {
            fret_ui_kit::declarative::tree::tree_view(cx, items, state, fret_ui_kit::Size::Medium)
        };

        vec![
            tree.attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-tree-torture-root"),
            ),
        ]
    });

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![header, cx.container(container_props, |_cx| vec![tree])]
}

fn preview_ai_transcript_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;

    let variable_height = std::env::var_os("FRET_UI_GALLERY_AI_TRANSCRIPT_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();
    let message_count = std::env::var("FRET_UI_GALLERY_AI_TRANSCRIPT_LEN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5_000);
    let append_batch: usize = 100;

    #[derive(Default)]
    struct TranscriptModels {
        messages: Option<Model<Arc<[ui_ai::ConversationMessage]>>>,
    }

    let message_text = |i: u64| -> Arc<str> {
        if variable_height && i % 7 == 0 {
            Arc::<str>::from(format!(
                "Message {i}\nDetails: seed={} tokens={} latency={}ms",
                (i * 31) % 97,
                16 + (i % 64),
                10 + (i % 120)
            ))
        } else {
            Arc::<str>::from(format!("Message {i}: hello world"))
        }
    };

    let messages_model = cx.with_state(TranscriptModels::default, |st| st.messages.clone());
    let messages_model = match messages_model {
        Some(model) => model,
        None => {
            let mut out: Vec<ui_ai::ConversationMessage> = Vec::with_capacity(message_count);
            for i in 0..message_count as u64 {
                let role = match i % 4 {
                    0 => ui_ai::MessageRole::User,
                    1 => ui_ai::MessageRole::Assistant,
                    2 => ui_ai::MessageRole::Tool,
                    _ => ui_ai::MessageRole::System,
                };
                out.push(ui_ai::ConversationMessage::new(i, role, message_text(i)));
            }

            let out: Arc<[ui_ai::ConversationMessage]> = Arc::from(out);
            let model = cx.app.models_mut().insert(out);
            cx.with_state(TranscriptModels::default, |st| {
                st.messages = Some(model.clone())
            });
            model
        }
    };
    let messages = cx
        .get_model_cloned(&messages_model, Invalidation::Layout)
        .unwrap_or_else(|| Arc::from([]));

    let append_messages_on_activate: OnActivate = {
        let messages_model = messages_model.clone();
        Arc::new(move |host, acx, _reason| {
            let existing = host
                .models_mut()
                .get_cloned(&messages_model)
                .unwrap_or_else(|| Arc::from([]));
            let start = existing.len() as u64;

            let mut out: Vec<ui_ai::ConversationMessage> = existing.iter().cloned().collect();
            out.reserve(append_batch);
            for i in start..start + append_batch as u64 {
                let role = match i % 4 {
                    0 => ui_ai::MessageRole::User,
                    1 => ui_ai::MessageRole::Assistant,
                    2 => ui_ai::MessageRole::Tool,
                    _ => ui_ai::MessageRole::System,
                };
                let text = if variable_height && i % 5 == 0 {
                    Arc::<str>::from(format!("Appended {i}\n(extra line)"))
                } else {
                    Arc::<str>::from(format!("Appended {i}"))
                };
                out.push(ui_ai::ConversationMessage::new(i, role, text));
            }

            let out: Arc<[ui_ai::ConversationMessage]> = Arc::from(out);
            let _ = host.models_mut().update(&messages_model, |v| *v = out);
            host.request_redraw(acx.window);
        })
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline harness for long AI transcripts (scrolling + virtualization + caching)."),
                cx.text("Use scripted wheel-scroll to validate view-cache reuse stability and stale-paint safety."),
                fret_ui_shadcn::Button::new(format!("Append {append_batch} messages"))
                    .test_id("ui-gallery-ai-transcript-append")
                    .on_activate(append_messages_on_activate)
                    .into_element(cx),
            ]
        },
    );

    let transcript =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());
            let revision = messages.len().min(u64::MAX as usize) as u64;

            let transcript = ui_ai::ConversationTranscript::from_arc(messages.clone())
                .content_revision(revision)
                .scroll_handle(scroll_handle.clone())
                .stick_to_bottom(false)
                .show_scroll_to_bottom_button(false)
                .debug_root_test_id("ui-gallery-ai-transcript-root")
                .debug_row_test_id_prefix("ui-gallery-ai-transcript-row-")
                .into_element(cx);

            let scroll_button = ui_ai::ConversationScrollButton::new(scroll_handle)
                .test_id("ui-gallery-ai-transcript-scroll-bottom")
                .into_element(cx);

            let layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default().w_full().h_full().relative(),
            );

            vec![
                cx.stack_props(fret_ui::element::StackProps { layout }, |_cx| {
                    vec![transcript, scroll_button]
                }),
            ]
        });

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![
        header,
        cx.container(container_props, |_cx| vec![transcript]),
    ]
}

fn preview_ai_chat_demo(cx: &mut ElementContext<'_, App>, _theme: &Theme) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui::action::OnActivate;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Debug, Clone)]
    struct PendingReply {
        assistant_id: u64,
        chunks: Arc<[Arc<str>]>,
        next_chunk: usize,
        markdown: Arc<str>,
        tool_call_running: ui_ai::ToolCall,
        tool_call_final: ui_ai::ToolCall,
        sources: Arc<[ui_ai::SourceItem]>,
        citations: Arc<[ui_ai::CitationItem]>,
    }

    #[derive(Default)]
    struct ChatModels {
        prompt: Option<Model<String>>,
        messages: Option<Model<Arc<[ui_ai::AiMessage]>>>,
        loading: Option<Model<bool>>,
        pending: Option<Model<Option<PendingReply>>>,
        next_id: Option<Model<u64>>,
        content_revision: Option<Model<u64>>,
        exported_md_len: Option<Model<Option<usize>>>,
    }

    let prompt = cx.with_state(ChatModels::default, |st| st.prompt.clone());
    let prompt = match prompt {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ChatModels::default, |st| st.prompt = Some(model.clone()));
            model
        }
    };

    let messages = cx.with_state(ChatModels::default, |st| st.messages.clone());
    let messages = match messages {
        Some(model) => model,
        None => {
            let sources: Arc<[ui_ai::SourceItem]> = Arc::from(vec![
                ui_ai::SourceItem::new("src-0", "Example source A")
                    .url("https://example.com/a")
                    .excerpt("A short excerpt used for truncation and wrapping tests."),
                ui_ai::SourceItem::new("src-1", "Example source B")
                    .url("https://example.com/b")
                    .excerpt("Another excerpt: this should wrap and remain readable."),
            ]);

            let citations: Arc<[ui_ai::CitationItem]> = Arc::from(vec![
                ui_ai::CitationItem::new("src-0", "[1]"),
                ui_ai::CitationItem::from_arc(
                    Arc::from(vec![Arc::<str>::from("src-0"), Arc::<str>::from("src-1")]),
                    "[2]",
                ),
            ]);

            let tool_call = ui_ai::ToolCall::new("toolcall-seed-0", "search")
                .state(ui_ai::ToolCallState::InputAvailable)
                .input(ui_ai::ToolCallPayload::Json(serde_json::json!({
                    "query": "seeded tool call",
                    "k": 3
                })));

            let initial: Arc<[ui_ai::AiMessage]> = Arc::from(vec![
                ui_ai::AiMessage::new(
                    1,
                    ui_ai::MessageRole::User,
                    [ui_ai::MessagePart::Text(Arc::<str>::from("Hello!"))],
                ),
                ui_ai::AiMessage::new(
                    2,
                    ui_ai::MessageRole::Assistant,
                    [ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::new(
                        Arc::<str>::from(
                            "This is a small demo for `PromptInput` + transcript append.\n\nIt also exercises tool calls + sources blocks.\n\n```rust\nfn demo() {\n    println!(\"hello from code fence\");\n}\n```",
                        ),
                    ))],
                ),
                ui_ai::AiMessage::new(
                    3,
                    ui_ai::MessageRole::User,
                    [ui_ai::MessagePart::Text(Arc::<str>::from(
                        "Show me seeded tools + sources + citations.",
                    ))],
                ),
                ui_ai::AiMessage::new(
                    4,
                    ui_ai::MessageRole::Assistant,
                    [
                        ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::streaming(
                            Arc::<str>::from(""),
                        )),
                        ui_ai::MessagePart::ToolCall(tool_call),
                        ui_ai::MessagePart::Sources(sources),
                        ui_ai::MessagePart::Citations(citations),
                    ],
                ),
            ]);
            let model = cx.app.models_mut().insert(initial);
            cx.with_state(ChatModels::default, |st| st.messages = Some(model.clone()));
            model
        }
    };

    let loading = cx.with_state(ChatModels::default, |st| st.loading.clone());
    let loading = match loading {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ChatModels::default, |st| st.loading = Some(model.clone()));
            model
        }
    };

    let pending = cx.with_state(ChatModels::default, |st| st.pending.clone());
    let pending = match pending {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<PendingReply>);
            cx.with_state(ChatModels::default, |st| st.pending = Some(model.clone()));
            model
        }
    };

    let next_id = cx.with_state(ChatModels::default, |st| st.next_id.clone());
    let next_id = match next_id {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(5u64);
            cx.with_state(ChatModels::default, |st| st.next_id = Some(model.clone()));
            model
        }
    };

    let content_revision = cx.with_state(ChatModels::default, |st| st.content_revision.clone());
    let content_revision = match content_revision {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(0u64);
            cx.with_state(ChatModels::default, |st| {
                st.content_revision = Some(model.clone())
            });
            model
        }
    };

    let exported_md_len = cx.with_state(ChatModels::default, |st| st.exported_md_len.clone());
    let exported_md_len = match exported_md_len {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<usize>);
            cx.with_state(ChatModels::default, |st| {
                st.exported_md_len = Some(model.clone())
            });
            model
        }
    };

    let prompt_non_empty = cx
        .get_model_cloned(&prompt, Invalidation::Paint)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let prompt_non_empty_marker = prompt_non_empty.then(|| {
        cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-gallery-ai-chat-prompt-nonempty")),
                ..Default::default()
            },
            |cx| {
                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: fret_ui::element::LayoutStyle {
                            size: fret_ui::element::SizeStyle {
                                width: fret_ui::element::Length::Px(Px(0.0)),
                                height: fret_ui::element::Length::Px(Px(0.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                )]
            },
        )
    });

    let loading_value = cx
        .get_model_copied(&loading, Invalidation::Paint)
        .unwrap_or(false);
    let pending_value = cx
        .get_model_cloned(&pending, Invalidation::Paint)
        .unwrap_or(None);

    if loading_value {
        if let Some(pending_state) = pending_value {
            if pending_state.next_chunk < pending_state.chunks.len() {
                cx.request_frame();

                if let Some(chunk) = pending_state.chunks.get(pending_state.next_chunk).cloned() {
                    let new_markdown =
                        Arc::<str>::from(format!("{}{}", pending_state.markdown, chunk));

                    let _ = cx.app.models_mut().update(&pending, |v| {
                        if let Some(p) = v {
                            p.markdown = new_markdown.clone();
                            p.next_chunk = p.next_chunk.saturating_add(1);
                        }
                    });

                    let assistant_id = pending_state.assistant_id;
                    let tool_call_running = pending_state.tool_call_running.clone();
                    let sources = pending_state.sources.clone();
                    let citations = pending_state.citations.clone();

                    let _ = cx.app.models_mut().update(&messages, |list| {
                        let mut vec = list.as_ref().to_vec();
                        if let Some(msg) = vec.iter_mut().find(|m| m.id == assistant_id) {
                            msg.parts = Arc::from(vec![
                                ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::streaming(
                                    new_markdown.clone(),
                                )),
                                ui_ai::MessagePart::ToolCall(tool_call_running),
                                ui_ai::MessagePart::Sources(sources),
                                ui_ai::MessagePart::Citations(citations),
                            ]);
                        }
                        *list = vec.into();
                    });
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&content_revision, |v| *v = v.saturating_add(1));
                } else {
                    let _ = cx.app.models_mut().update(&pending, |v| *v = None);
                    let _ = cx.app.models_mut().update(&loading, |v| *v = false);
                }
            } else {
                let assistant_id = pending_state.assistant_id;
                let markdown = pending_state.markdown.clone();
                let tool_call_final = pending_state.tool_call_final.clone();
                let sources = pending_state.sources.clone();
                let citations = pending_state.citations.clone();

                let _ = cx.app.models_mut().update(&messages, |list| {
                    let mut vec = list.as_ref().to_vec();
                    if let Some(msg) = vec.iter_mut().find(|m| m.id == assistant_id) {
                        msg.parts = Arc::from(vec![
                            ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::new(markdown)),
                            ui_ai::MessagePart::ToolCall(tool_call_final),
                            ui_ai::MessagePart::Sources(sources),
                            ui_ai::MessagePart::Citations(citations),
                        ]);
                    }
                    *list = vec.into();
                });
                let _ = cx
                    .app
                    .models_mut()
                    .update(&content_revision, |v| *v = v.saturating_add(1));

                let _ = cx.app.models_mut().update(&pending, |v| *v = None);
                let _ = cx.app.models_mut().update(&loading, |v| *v = false);
            }
        }
    }

    let send: OnActivate = Arc::new({
        let prompt = prompt.clone();
        let messages = messages.clone();
        let pending = pending.clone();
        let loading = loading.clone();
        let next_id = next_id.clone();
        let content_revision = content_revision.clone();
        move |host, _action_cx, _reason| {
            fn chunk_for_demo(text: &str, chars_per_chunk: usize) -> Arc<[Arc<str>]> {
                let mut out = Vec::new();
                let mut buf = String::new();
                let mut count = 0usize;

                for ch in text.chars() {
                    buf.push(ch);
                    count = count.saturating_add(1);
                    if count >= chars_per_chunk {
                        out.push(Arc::<str>::from(std::mem::take(&mut buf)));
                        count = 0;
                    }
                }

                if !buf.is_empty() {
                    out.push(Arc::<str>::from(buf));
                }

                out.into()
            }

            let text = host.models_mut().read(&prompt, Clone::clone).ok();
            let Some(text) = text else { return };
            let text = text.trim().to_string();
            if text.is_empty() {
                return;
            }

            let user_id = host
                .models_mut()
                .update(&next_id, |v| {
                    let id = *v;
                    *v = v.saturating_add(1);
                    id
                })
                .ok()
                .unwrap_or(0);
            let assistant_id = host
                .models_mut()
                .update(&next_id, |v| {
                    let id = *v;
                    *v = v.saturating_add(1);
                    id
                })
                .ok()
                .unwrap_or(0);

            let tool_call = ui_ai::ToolCall::new("toolcall-0", "search")
                .state(ui_ai::ToolCallState::InputAvailable)
                .input(ui_ai::ToolCallPayload::Json(serde_json::json!({
                    "query": text,
                    "k": 3
                })));

            let sources: Arc<[ui_ai::SourceItem]> = Arc::from(vec![
                ui_ai::SourceItem::new("src-0", "Example source A")
                    .url("https://example.com/a")
                    .excerpt("A short excerpt used for truncation and wrapping tests."),
                ui_ai::SourceItem::new("src-1", "Example source B")
                    .url("https://example.com/b")
                    .excerpt("Another excerpt: this should wrap and remain readable."),
            ]);

            let citations: Arc<[ui_ai::CitationItem]> = Arc::from(vec![
                ui_ai::CitationItem::new("src-0", "[1]"),
                ui_ai::CitationItem::from_arc(
                    Arc::from(vec![Arc::<str>::from("src-0"), Arc::<str>::from("src-1")]),
                    "[2]",
                ),
            ]);

            let reply = format!(
                "Echo: **{text}**\n\nThis reply is streamed via append-only updates.\n\n```rust\nfn streamed_demo() {{\n    println!(\"{text}\");\n}}\n"
            );
            let chunks = chunk_for_demo(&reply, 12);

            let tool_call_final = tool_call
                .clone()
                .state(ui_ai::ToolCallState::OutputAvailable)
                .output(ui_ai::ToolCallPayload::Json(serde_json::json!({
                    "results": [
                        {"title": "A", "score": 0.9},
                        {"title": "B", "score": 0.8}
                    ]
                })));

            let _ = host.models_mut().update(&messages, |list| {
                let mut vec = list.as_ref().to_vec();
                vec.push(ui_ai::AiMessage::new(
                    user_id,
                    ui_ai::MessageRole::User,
                    [ui_ai::MessagePart::Text(Arc::<str>::from(text))],
                ));
                vec.push(ui_ai::AiMessage::new(
                    assistant_id,
                    ui_ai::MessageRole::Assistant,
                    [
                        ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::streaming(
                            Arc::<str>::from(""),
                        )),
                        ui_ai::MessagePart::ToolCall(tool_call.clone()),
                        ui_ai::MessagePart::Sources(sources.clone()),
                        ui_ai::MessagePart::Citations(citations.clone()),
                    ],
                ));
                *list = vec.into();
            });
            let _ = host
                .models_mut()
                .update(&content_revision, |v| *v = v.saturating_add(1));

            let _ = host.models_mut().update(&pending, |v| {
                *v = Some(PendingReply {
                    assistant_id,
                    chunks,
                    next_chunk: 0,
                    markdown: Arc::<str>::from(""),
                    tool_call_running: tool_call,
                    tool_call_final,
                    sources,
                    citations,
                })
            });
            let _ = host.models_mut().update(&loading, |v| *v = true);
        }
    });

    let stop: OnActivate = Arc::new({
        let messages = messages.clone();
        let pending = pending.clone();
        let loading = loading.clone();
        let content_revision = content_revision.clone();
        move |host, _action_cx, _reason| {
            let assistant_id = host
                .models_mut()
                .read(&pending, |v| v.as_ref().map(|p| p.assistant_id))
                .ok()
                .flatten();

            let _ = host.models_mut().update(&pending, |v| *v = None);
            let _ = host.models_mut().update(&loading, |v| *v = false);

            let Some(assistant_id) = assistant_id else {
                return;
            };
            let _ = host.models_mut().update(&messages, |list| {
                let vec: Vec<_> = list
                    .iter()
                    .cloned()
                    .filter(|m| m.id != assistant_id)
                    .collect();
                *list = vec.into();
            });
            let _ = host
                .models_mut()
                .update(&content_revision, |v| *v = v.saturating_add(1));
        }
    });

    let export_markdown: OnActivate = Arc::new({
        let messages = messages.clone();
        let exported_md_len = exported_md_len.clone();
        move |host, _action_cx, _reason| {
            let messages = host.models_mut().read(&messages, Clone::clone).ok();
            let Some(messages) = messages else {
                return;
            };

            let md = ui_ai::messages_to_markdown(messages.as_ref());
            let _ = host
                .models_mut()
                .update(&exported_md_len, |v| *v = Some(md.len()));
        }
    });

    let start_streaming: OnActivate = Arc::new({
        let messages = messages.clone();
        let pending = pending.clone();
        let loading = loading.clone();
        let content_revision = content_revision.clone();
        move |host, _action_cx, _reason| {
            fn chunk_for_demo(text: &str, chars_per_chunk: usize) -> Arc<[Arc<str>]> {
                let mut out = Vec::new();
                let mut buf = String::new();
                let mut count = 0usize;

                for ch in text.chars() {
                    buf.push(ch);
                    count = count.saturating_add(1);
                    if count >= chars_per_chunk {
                        out.push(Arc::<str>::from(std::mem::take(&mut buf)));
                        count = 0;
                    }
                }

                if !buf.is_empty() {
                    out.push(Arc::<str>::from(buf));
                }

                out.into()
            }

            let sources: Arc<[ui_ai::SourceItem]> = Arc::from(vec![
                ui_ai::SourceItem::new("src-0", "Example source A")
                    .url("https://example.com/a")
                    .excerpt("A short excerpt used for truncation and wrapping tests."),
                ui_ai::SourceItem::new("src-1", "Example source B")
                    .url("https://example.com/b")
                    .excerpt("Another excerpt: this should wrap and remain readable."),
            ]);

            let citations: Arc<[ui_ai::CitationItem]> = Arc::from(vec![
                ui_ai::CitationItem::new("src-0", "[1]"),
                ui_ai::CitationItem::from_arc(
                    Arc::from(vec![Arc::<str>::from("src-0"), Arc::<str>::from("src-1")]),
                    "[2]",
                ),
            ]);

            let tool_call_running = ui_ai::ToolCall::new("toolcall-seed-0", "search")
                .state(ui_ai::ToolCallState::InputAvailable)
                .input(ui_ai::ToolCallPayload::Json(serde_json::json!({
                    "query": "seeded tool call",
                    "k": 3
                })));

            let tool_call_final = tool_call_running
                .clone()
                .state(ui_ai::ToolCallState::OutputAvailable)
                .output(ui_ai::ToolCallPayload::Json(serde_json::json!({
                    "results": [
                        {"title": "A", "score": 0.9},
                        {"title": "B", "score": 0.8}
                    ]
                })));

            let reply = "This assistant message is streamed in append-only chunks.\n\n```rust\nfn streamed_demo() {\n    println!(\"hello from stream\");\n}\n```\n";
            let chunks = chunk_for_demo(reply, 12);

            let assistant_id = 4u64;

            let _ = host.models_mut().update(&messages, |list| {
                let mut vec = list.as_ref().to_vec();
                if let Some(msg) = vec.iter_mut().find(|m| m.id == assistant_id) {
                    msg.parts = Arc::from(vec![
                        ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::streaming(
                            Arc::<str>::from(""),
                        )),
                        ui_ai::MessagePart::ToolCall(tool_call_running.clone()),
                        ui_ai::MessagePart::Sources(sources.clone()),
                        ui_ai::MessagePart::Citations(citations.clone()),
                    ]);
                }
                *list = vec.into();
            });

            let _ = host.models_mut().update(&pending, |v| {
                *v = Some(PendingReply {
                    assistant_id,
                    chunks,
                    next_chunk: 0,
                    markdown: Arc::<str>::from(""),
                    tool_call_running,
                    tool_call_final,
                    sources,
                    citations,
                })
            });

            let _ = host.models_mut().update(&loading, |v| *v = true);
            let _ = host
                .models_mut()
                .update(&content_revision, |v| *v = v.saturating_add(1));
        }
    });

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: interactive demo for PromptInput + transcript append."),
                cx.text("Send triggers a short \"loading\" window where Stop is available."),
                shadcn::Button::new("Start streaming (seeded)")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-ai-chat-start-stream")
                    .on_activate(start_streaming.clone())
                    .into_element(cx),
            ]
        },
    );

    let actions_demo = {
        let copy = ui_ai::MessageAction::new("Copy")
            .tooltip("Copy")
            .test_id("ui-gallery-ai-chat-action-copy")
            .children([shadcn::icon::icon(
                cx,
                fret_icons::IconId::new_static("lucide.copy"),
            )])
            .into_element(cx);

        ui_ai::MessageActions::new([copy])
            .test_id("ui-gallery-ai-chat-actions")
            .into_element(cx)
    };

    let chat = ui_ai::AiChat::new(messages.clone(), prompt)
        .loading_model(loading.clone())
        .content_revision_model(content_revision.clone())
        .on_send(send)
        .on_stop(stop)
        .show_download(true)
        .on_download(export_markdown)
        .download_test_id("ui-gallery-ai-chat-download")
        .message_test_id_prefix("ui-ai-msg-")
        .transcript_root_test_id("ui-gallery-ai-chat-transcript-root")
        .transcript_row_test_id_prefix("ui-gallery-ai-chat-transcript-row-")
        .scroll_button_test_id("ui-gallery-ai-chat-scroll-bottom")
        .prompt_root_test_id("ui-gallery-ai-chat-prompt-root")
        .prompt_textarea_test_id("ui-gallery-ai-chat-prompt-textarea")
        .prompt_send_test_id("ui-gallery-ai-chat-prompt-send")
        .prompt_stop_test_id("ui-gallery-ai-chat-prompt-stop")
        .transcript_container_layout(LayoutRefinement::default().w_full().h_px(Px(360.0)))
        .into_element(cx);

    let exported_value = cx
        .get_model_cloned(&exported_md_len, Invalidation::Paint)
        .unwrap_or(None);
    let exported = exported_value.map(|len| {
        cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-gallery-ai-chat-exported-md-len")),
                ..Default::default()
            },
            move |cx| vec![cx.text(format!("Exported markdown: {len} chars"))],
        )
    });

    vec![
        header,
        actions_demo,
        chat,
        prompt_non_empty_marker.unwrap_or_else(|| cx.text("")),
        exported.unwrap_or_else(|| cx.text("")),
    ]
}

fn preview_ai_file_tree_demo(cx: &mut ElementContext<'_, App>, _theme: &Theme) -> Vec<AnyElement> {
    use std::collections::HashSet;
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::action::ActionCx;
    use fret_ui::element::SemanticsProps;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Default)]
    struct FileTreeModels {
        expanded: Option<Model<HashSet<Arc<str>>>>,
        selected: Option<Model<Option<Arc<str>>>>,
    }

    let expanded = cx.with_state(FileTreeModels::default, |st| st.expanded.clone());
    let expanded = match expanded {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(HashSet::<Arc<str>>::new());
            cx.with_state(FileTreeModels::default, |st| {
                st.expanded = Some(model.clone())
            });
            model
        }
    };

    let selected = cx.with_state(FileTreeModels::default, |st| st.selected.clone());
    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(FileTreeModels::default, |st| {
                st.selected = Some(model.clone())
            });
            model
        }
    };

    let selected_value = cx.watch_model(&selected).layout().cloned().flatten();

    let tree = ui_ai::FileTree::new([
        ui_ai::FileTreeFolder::new("src", "src")
            .test_id("ui-ai-file-tree-folder-src")
            .children([
                ui_ai::FileTreeFile::new("src/lib.rs", "lib.rs")
                    .test_id("ui-ai-file-tree-file-lib")
                    .into(),
                ui_ai::FileTreeFile::new("src/main.rs", "main.rs")
                    .test_id("ui-ai-file-tree-file-main")
                    .into(),
            ])
            .into(),
        ui_ai::FileTreeFile::new("Cargo.toml", "Cargo.toml")
            .test_id("ui-ai-file-tree-file-cargo-toml")
            .into(),
        ui_ai::FileTreeFolder::new("tests", "tests")
            .test_id("ui-ai-file-tree-folder-tests")
            .child(
                ui_ai::FileTreeFile::new("tests/file_tree.rs", "file_tree.rs")
                    .test_id("ui-ai-file-tree-file-tests-file-tree"),
            )
            .into(),
    ])
    .expanded_paths(expanded.clone())
    .selected_path(selected_value.clone())
    .on_select(Arc::new({
        let selected = selected.clone();
        move |host, _action_cx: ActionCx, path| {
            let _ = host.models_mut().update(&selected, |v| *v = Some(path));
        }
    }))
    .test_id_root("ui-ai-file-tree-root")
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let selected_label_text = selected_value
        .as_deref()
        .map(|s| format!("Selected: {s}"))
        .unwrap_or_else(|| "Selected: <none>".to_string());

    let selected_label = cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Text,
            test_id: Some(Arc::<str>::from("ui-ai-file-tree-selected-label")),
            ..Default::default()
        },
        move |cx| vec![cx.text(selected_label_text)],
    );

    let selected_marker = (selected_value.as_deref() == Some("src/lib.rs")).then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Generic,
                test_id: Some(Arc::<str>::from("ui-ai-file-tree-selected-marker")),
                ..Default::default()
            },
            move |_cx| vec![],
        )
    });

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3),
        move |cx| {
            vec![
                cx.text("FileTree (AI Elements)"),
                tree,
                selected_label,
                selected_marker.unwrap_or_else(|| cx.text("")),
            ]
        },
    )]
}

fn preview_inspector_torture(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let len: usize = std::env::var("FRET_UI_GALLERY_INSPECTOR_LEN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(50_000)
        .clamp(16, 200_000);
    let row_height = Px(28.0);
    let overscan = 12;
    let keep_alive: usize = std::env::var("FRET_UI_GALLERY_INSPECTOR_KEEP_ALIVE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
        .clamp(0, 4096);

    let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

    let list_layout = fret_ui::element::LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: fret_ui::element::Length::Fill,
            height: fret_ui::element::Length::Px(Px(460.0)),
            ..Default::default()
        },
        overflow: fret_ui::element::Overflow::Clip,
        ..Default::default()
    };

    let options =
        fret_ui::element::VirtualListOptions::known(row_height, overscan, move |_index| row_height)
            .keep_alive(keep_alive);

    let theme = theme.clone();
    let row = move |cx: &mut ElementContext<'_, App>, index: usize| {
        let zebra = (index % 2) == 0;
        let background = if zebra {
            theme.color_required("muted")
        } else {
            theme.color_required("background")
        };

        let depth = (index % 8) as f32;
        let indent_px = Px(depth * 12.0);

        let name = cx.text(format!("prop_{index}"));
        let value = cx.text(format!("value {index}"));

        let spacer = cx.container(
            fret_ui::element::ContainerProps {
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Px(indent_px),
                        height: fret_ui::element::Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        let mut row_props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(background))
                .p(Space::N2),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(row_height)),
        );
        row_props.layout.overflow = fret_ui::element::Overflow::Clip;

        let row = cx.container(row_props, |cx| {
            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .gap(Space::N2)
                    .items_center(),
                |_cx| vec![spacer, name, value],
            )]
        });

        row.test_id(format!("ui-gallery-inspector-row-{index}-label"))
    };

    let list = cx.virtual_list_keyed_retained_with_layout_fn(
        list_layout,
        len,
        options,
        &scroll_handle,
        |i| i as fret_ui::ItemKey,
        row,
    );

    let list = list.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::List)
            .test_id("ui-gallery-inspector-root"),
    );

    vec![cx.cached_subtree_with(
        CachedSubtreeProps::default().contained_layout(true),
        |_cx| vec![list],
    )]
}

fn preview_file_tree_torture(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let _ = theme;
    use std::collections::HashSet;

    let row_height = Px(26.0);
    let overscan = 12;

    let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

    let list_layout = fret_ui::element::LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: fret_ui::element::Length::Fill,
            height: fret_ui::element::Length::Px(Px(460.0)),
            ..Default::default()
        },
        overflow: fret_ui::element::Overflow::Clip,
        ..Default::default()
    };

    use fret_ui_kit::{TreeItem, TreeItemId, TreeState};

    #[derive(Default)]
    struct FileTreeTortureModels {
        items: Option<Model<Vec<TreeItem>>>,
        state: Option<Model<TreeState>>,
    }

    let (items, state) = cx.with_state(FileTreeTortureModels::default, |st| {
        (st.items.clone(), st.state.clone())
    });
    let (items, state) = match (items, state) {
        (Some(items), Some(state)) => (items, state),
        _ => {
            let (items_value, state_value) = {
                let root_count: u64 = std::env::var("FRET_UI_GALLERY_FILE_TREE_ROOTS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(200);
                let folders_per_root = 10u64;
                let leaves_per_folder = 25u64;

                let mut expanded: HashSet<TreeItemId> = HashSet::new();
                let mut roots: Vec<TreeItem> = Vec::with_capacity(root_count as usize);

                for r in 0..root_count {
                    let root_id = r;
                    expanded.insert(root_id);

                    let mut folders: Vec<TreeItem> = Vec::with_capacity(folders_per_root as usize);
                    for f in 0..folders_per_root {
                        let folder_id = 1_000_000 + r * 100 + f;
                        expanded.insert(folder_id);

                        let mut leaves: Vec<TreeItem> =
                            Vec::with_capacity(leaves_per_folder as usize);
                        for l in 0..leaves_per_folder {
                            let leaf_id = 2_000_000 + r * 10_000 + f * 100 + l;
                            leaves.push(TreeItem::new(
                                leaf_id,
                                Arc::<str>::from(format!("file_{r}_{f}_{l}.rs")),
                            ));
                        }

                        folders.push(
                            TreeItem::new(folder_id, Arc::<str>::from(format!("dir_{r}_{f}")))
                                .children(leaves),
                        );
                    }

                    roots.push(
                        TreeItem::new(root_id, Arc::<str>::from(format!("root_{r}")))
                            .children(folders),
                    );
                }

                (
                    roots,
                    TreeState {
                        expanded,
                        selected: None,
                    },
                )
            };

            let items = cx.app.models_mut().insert(items_value);
            let state = cx.app.models_mut().insert(state_value);
            cx.with_state(FileTreeTortureModels::default, |st| {
                st.items = Some(items.clone());
                st.state = Some(state.clone());
            });
            (items, state)
        }
    };

    let mut props = fret_ui_kit::declarative::file_tree::FileTreeViewProps::default();
    props.layout = list_layout;
    props.row_height = row_height;
    props.overscan = overscan;
    props.debug_root_test_id = Some(Arc::<str>::from("ui-gallery-file-tree-root"));
    props.debug_row_test_id_prefix = Some(Arc::<str>::from("ui-gallery-file-tree-node"));

    vec![
        fret_ui_kit::declarative::file_tree::file_tree_view_retained_v0(
            cx,
            items,
            state,
            &scroll_handle,
            props,
        ),
    ]
}

fn preview_table_retained_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::headless::table::{
        ColumnDef, RowKey, RowPinPosition, TableState, pagination_bounds, pin_rows,
    };
    let variable_height = std::env::var_os("FRET_UI_GALLERY_TABLE_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();
    let keep_alive: usize = std::env::var("FRET_UI_GALLERY_TABLE_KEEP_ALIVE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    #[derive(Clone)]
    struct TableRow {
        id: u64,
        name: Arc<str>,
        status: Arc<str>,
        cpu: u32,
        mem_mb: u32,
    }

    #[derive(Default)]
    struct TableTortureModels {
        data: Option<Arc<[TableRow]>>,
        columns: Option<Arc<[ColumnDef<TableRow>]>>,
        state: Option<Model<TableState>>,
        keep_pinned_rows: Option<Model<bool>>,
    }

    let (data, columns, state, keep_pinned_rows) =
        cx.with_state(TableTortureModels::default, |st| {
            (
                st.data.clone(),
                st.columns.clone(),
                st.state.clone(),
                st.keep_pinned_rows.clone(),
            )
        });

    let (data, columns, state, keep_pinned_rows) = match (data, columns, state, keep_pinned_rows) {
        (Some(data), Some(columns), Some(state), Some(keep_pinned_rows)) => {
            (data, columns, state, keep_pinned_rows)
        }
        _ => {
            let mut rows: Vec<TableRow> = Vec::with_capacity(50_000);
            for i in 0..50_000u64 {
                rows.push(TableRow {
                    id: i,
                    name: Arc::from(format!("Row {i}")),
                    status: Arc::from(if i % 3 == 0 {
                        "idle"
                    } else if i % 3 == 1 {
                        "busy"
                    } else {
                        "offline"
                    }),
                    cpu: ((i * 7) % 100) as u32,
                    mem_mb: (128 + (i % 4096)) as u32,
                });
            }
            let data: Arc<[TableRow]> = Arc::from(rows);

            let cols: Vec<ColumnDef<TableRow>> = vec![
                ColumnDef::new("name").sort_by(|a: &TableRow, b: &TableRow| a.name.cmp(&b.name)),
                ColumnDef::new("status")
                    .sort_by(|a: &TableRow, b: &TableRow| a.status.cmp(&b.status)),
                ColumnDef::new("cpu%").sort_by(|a: &TableRow, b: &TableRow| a.cpu.cmp(&b.cpu)),
                ColumnDef::new("mem_mb")
                    .sort_by(|a: &TableRow, b: &TableRow| a.mem_mb.cmp(&b.mem_mb)),
            ];
            let columns: Arc<[ColumnDef<TableRow>]> = Arc::from(cols);

            let state = cx.app.models_mut().insert(TableState::default());
            let keep_pinned_rows = cx.app.models_mut().insert(true);

            cx.with_state(TableTortureModels::default, |st| {
                st.data = Some(data.clone());
                st.columns = Some(columns.clone());
                st.state = Some(state.clone());
                st.keep_pinned_rows = Some(keep_pinned_rows.clone());
            });

            (data, columns, state, keep_pinned_rows)
        }
    };

    let sorting: Vec<fret_ui_kit::headless::table::SortSpec> = cx
        .app
        .models()
        .read(&state, |st| st.sorting.clone())
        .ok()
        .unwrap_or_default();
    let sorting_text: Arc<str> = if sorting.is_empty() {
        Arc::<str>::from("Sorting: <none>")
    } else {
        let parts: Vec<String> = sorting
            .iter()
            .map(|s| format!("{} {}", s.column, if s.desc { "desc" } else { "asc" }))
            .collect();
        Arc::<str>::from(format!("Sorting: {}", parts.join(", ")))
    };

    let row_pinning_text: Arc<str> = {
        let pinning = cx
            .app
            .models()
            .read(&state, |st| st.row_pinning.clone())
            .ok()
            .unwrap_or_default();
        let top = pinning
            .top
            .iter()
            .map(|k| k.0.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let bottom = pinning
            .bottom
            .iter()
            .map(|k| k.0.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Arc::<str>::from(format!("RowPinning: top=[{top}] bottom=[{bottom}]"))
    };

    let keep_pinned_rows_value = cx
        .get_model_copied(&keep_pinned_rows, Invalidation::Paint)
        .unwrap_or(true);
    let keep_pinned_rows_text: Arc<str> =
        Arc::<str>::from(format!("KeepPinnedRows: {keep_pinned_rows_value}"));

    let page_text: Arc<str> = {
        let pagination = cx
            .app
            .models()
            .read(&state, |st| st.pagination)
            .ok()
            .unwrap_or_default();
        let bounds = pagination_bounds(data.len(), pagination);
        if bounds.page_count == 0 {
            Arc::<str>::from("Page: 0/0")
        } else {
            Arc::<str>::from(format!(
                "Page: {}/{}",
                bounds.page_index + 1,
                bounds.page_count
            ))
        }
    };

    let state_for_pin_top = state.clone();
    let on_pin_top: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&state_for_pin_top, |st| {
            let Some(&row_key) = st.row_selection.iter().next() else {
                return;
            };
            pin_rows(&mut st.row_pinning, Some(RowPinPosition::Top), [row_key]);
        });
        host.request_redraw(action_cx.window);
    });

    let state_for_pin_bottom = state.clone();
    let on_pin_bottom: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&state_for_pin_bottom, |st| {
            let Some(&row_key) = st.row_selection.iter().next() else {
                return;
            };
            pin_rows(&mut st.row_pinning, Some(RowPinPosition::Bottom), [row_key]);
        });
        host.request_redraw(action_cx.window);
    });

    let state_for_unpin = state.clone();
    let on_unpin: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&state_for_unpin, |st| {
            let Some(&row_key) = st.row_selection.iter().next() else {
                return;
            };
            pin_rows(&mut st.row_pinning, None, [row_key]);
        });
        host.request_redraw(action_cx.window);
    });

    let state_for_prev_page = state.clone();
    let on_prev_page: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&state_for_prev_page, |st| {
            st.pagination.page_index = st.pagination.page_index.saturating_sub(1);
        });
        host.request_redraw(action_cx.window);
    });

    let state_for_next_page = state.clone();
    let on_next_page: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&state_for_next_page, |st| {
            st.pagination.page_index = st.pagination.page_index.saturating_add(1);
        });
        host.request_redraw(action_cx.window);
    });

    let actions = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Prev page")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-table-retained-prev-page")
                    .on_activate(on_prev_page)
                    .into_element(cx),
                shadcn::Button::new("Next page")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-table-retained-next-page")
                    .on_activate(on_next_page)
                    .into_element(cx),
                shadcn::Button::new("Pin top")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-table-retained-pin-top")
                    .on_activate(on_pin_top)
                    .into_element(cx),
                shadcn::Button::new("Pin bottom")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-table-retained-pin-bottom")
                    .on_activate(on_pin_bottom)
                    .into_element(cx),
                shadcn::Button::new("Unpin")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-table-retained-unpin")
                    .on_activate(on_unpin)
                    .into_element(cx),
                shadcn::Switch::new(keep_pinned_rows.clone())
                    .a11y_label("Keep pinned rows")
                    .test_id("ui-gallery-table-retained-keep-pinned-rows")
                    .into_element(cx),
                cx.text("Keep pinned rows"),
            ]
        },
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text(
                    "Goal: baseline harness for `fret-ui-kit::declarative::table` running on the virt-003 retained host path.",
                ),
                cx.text(
                    "Use scripted sort/selection + scroll to validate reconcile deltas under view-cache reuse (no notify-based dirty views).",
                ),
                cx.text(sorting_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(sorting_text.clone())
                        .test_id("ui-gallery-table-retained-sorting"),
                ),
                cx.text(row_pinning_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(row_pinning_text.clone())
                        .test_id("ui-gallery-table-retained-row-pinning"),
                ),
                cx.text(keep_pinned_rows_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(keep_pinned_rows_text.clone())
                        .test_id("ui-gallery-table-retained-keep-pinned-rows-text"),
                ),
                cx.text(page_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(page_text.clone())
                        .test_id("ui-gallery-table-retained-pagination"),
                ),
                actions,
            ]
        },
    );

    let table =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

            let state_revision = cx.app.models().revision(&state).unwrap_or(0);
            let items_revision = 1 ^ state_revision.rotate_left(17);

            let mut props = fret_ui_kit::declarative::table::TableViewProps::default();
            props.overscan = 10;
            props.row_height = Some(Px(28.0));
            if keep_alive > 0 {
                props.keep_alive = Some(keep_alive);
            }
            props.row_measure_mode = if variable_height {
                fret_ui_kit::declarative::table::TableRowMeasureMode::Measured
            } else {
                fret_ui_kit::declarative::table::TableRowMeasureMode::Fixed
            };
            props.enable_column_grouping = false;
            props.enable_column_resizing = false;
            props.keep_pinned_rows = cx
                .get_model_copied(&keep_pinned_rows, Invalidation::Layout)
                .unwrap_or(true);

            let header_label =
                Arc::new(|col: &ColumnDef<TableRow>| Arc::<str>::from(col.id.as_ref()));
            let row_key_at = Arc::new(|row: &TableRow, _index: usize| RowKey(row.id));
            let cell_at = Arc::new(
                move |cx: &mut ElementContext<'_, App>,
                      col: &ColumnDef<TableRow>,
                      row: &TableRow| {
                    match col.id.as_ref() {
                        "name" => {
                            if variable_height && row.id % 15 == 0 {
                                stack::vstack(
                                    cx,
                                    stack::VStackProps::default().gap(Space::N0),
                                    |cx| {
                                        vec![
                                            cx.text(row.name.as_ref()),
                                            cx.text(format!(
                                                "Details: id={} cpu={} mem={}",
                                                row.id, row.cpu, row.mem_mb
                                            )),
                                        ]
                                    },
                                )
                            } else {
                                cx.text(row.name.as_ref())
                            }
                        }
                        "status" => cx.text(row.status.as_ref()),
                        "cpu%" => cx.text(format!("{}%", row.cpu)),
                        "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                        _ => cx.text("?"),
                    }
                },
            );

            let table = fret_ui_kit::declarative::table::table_virtualized_retained_v0(
                cx,
                data.clone(),
                columns.clone(),
                state.clone(),
                &scroll_handle,
                items_revision,
                row_key_at,
                Some(Arc::new(|row: &TableRow, _index: usize| {
                    Arc::from(row.id.to_string())
                })),
                props,
                header_label,
                None,
                cell_at,
                Some(Arc::<str>::from("ui-gallery-table-retained-header-")),
                Some(Arc::<str>::from("ui-gallery-table-retained-row-")),
            );

            vec![
                table.attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Group)
                        .test_id("ui-gallery-table-retained-torture-root"),
                ),
            ]
        });

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![header, cx.container(container_props, |_cx| vec![table])]
}

fn preview_data_grid(
    cx: &mut ElementContext<'_, App>,
    selected_row: Model<Option<u64>>,
) -> Vec<AnyElement> {
    let selected = cx
        .get_model_copied(&selected_row, Invalidation::Paint)
        .flatten();

    let selected_text: Arc<str> = selected
        .map(|v| Arc::<str>::from(v.to_string()))
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let grid = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        let selected = cx
            .get_model_copied(&selected_row, Invalidation::Layout)
            .flatten();

        let grid = shadcn::experimental::DataGridElement::new(
            ["PID", "Name", "State", "CPU%"],
            DATA_GRID_ROWS,
        )
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(320.0)))
        .into_element(
            cx,
            1,
            1,
            |row| row as u64,
            move |row| {
                let is_selected = selected == Some(row as u64);
                let cmd = data_grid_row_command(row).unwrap_or_else(|| {
                    // Fallback for out-of-range row IDs.
                    CommandId::new(format!("{CMD_DATA_GRID_ROW_PREFIX}{row}"))
                });
                shadcn::experimental::DataGridRowState {
                    selected: is_selected,
                    enabled: row % 17 != 0,
                    on_click: Some(cmd),
                }
            },
            |cx, row, col| {
                let pid = 1000 + row as u64;
                match col {
                    0 => cx.text(pid.to_string()),
                    1 => cx.text(format!("Process {row}")),
                    2 => cx.text(if row % 3 == 0 { "Running" } else { "Idle" }),
                    _ => cx.text(((row * 7) % 100).to_string()),
                }
            },
        );

        vec![grid]
    });

    vec![
        cx.text("Virtualized rows/cols viewport; click a row to select (disabled every 17th row)."),
        cx.text(format!("Selected row: {selected_text}")),
        grid,
    ]
}

fn preview_tabs(
    cx: &mut ElementContext<'_, App>,
    _value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let primary = cx.with_theme(|theme| theme.color_required("primary"));
    let line_style = shadcn::tabs::TabsStyle::default()
        .trigger_background(fret_ui_kit::WidgetStateProperty::new(Some(
            ColorRef::Color(CoreColor::TRANSPARENT),
        )))
        .trigger_border_color(
            fret_ui_kit::WidgetStateProperty::new(Some(ColorRef::Color(CoreColor::TRANSPARENT)))
                .when(
                    fret_ui_kit::WidgetStates::SELECTED,
                    Some(ColorRef::Color(primary)),
                ),
        );

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let card_panel = |cx: &mut ElementContext<'_, App>,
                      title: &'static str,
                      description: &'static str,
                      content: &'static str| {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new(title).into_element(cx),
                shadcn::CardDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![shadcn::typography::muted(cx, content)]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .into_element(cx)
    };

    let demo = {
        let tabs = shadcn::Tabs::uncontrolled(Some("overview"))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new(
                    "overview",
                    "Overview",
                    [card_panel(
                        cx,
                        "Overview",
                        "View your key metrics and recent project activity.",
                        "You have 12 active projects and 3 pending tasks.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "analytics",
                    "Analytics",
                    [card_panel(
                        cx,
                        "Analytics",
                        "Track performance and user engagement metrics.",
                        "Page views are up 25% compared to last month.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "reports",
                    "Reports",
                    [card_panel(
                        cx,
                        "Reports",
                        "Generate and download your detailed reports.",
                        "You have 5 reports ready and available to export.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "settings",
                    "Settings",
                    [card_panel(
                        cx,
                        "Settings",
                        "Manage your account preferences and options.",
                        "Configure notifications, security, and themes.",
                    )],
                ),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-demo");

        let demo_shell = shell(cx, tabs);
        let body = centered(cx, demo_shell);
        section(cx, "Demo", body)
    };

    let line = {
        let tabs = shadcn::Tabs::uncontrolled(Some("overview"))
            .style(line_style.clone())
            .refine_style(ChromeRefinement::default().bg(ColorRef::Color(CoreColor::TRANSPARENT)))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new("overview", "Overview", Vec::<AnyElement>::new()),
                shadcn::TabsItem::new("analytics", "Analytics", Vec::<AnyElement>::new()),
                shadcn::TabsItem::new("reports", "Reports", Vec::<AnyElement>::new()),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-line");

        let group = stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |cx| {
            vec![
                tabs,
                shadcn::typography::muted(
                    cx,
                    "Line variant is approximated with trigger style overrides in current API.",
                ),
            ]
        });
        let body = centered(cx, group);
        section(cx, "Line", body)
    };

    let vertical = {
        let tabs = shadcn::Tabs::uncontrolled(Some("account"))
            .orientation(shadcn::tabs::TabsOrientation::Vertical)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
            .items([
                shadcn::TabsItem::new(
                    "account",
                    "Account",
                    [card_panel(
                        cx,
                        "Account",
                        "Update your account details and profile settings.",
                        "Display name and avatar were updated 2 days ago.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "password",
                    "Password",
                    [card_panel(
                        cx,
                        "Password",
                        "Change your password and keep your account secure.",
                        "Last password update was 28 days ago.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "notifications",
                    "Notifications",
                    [card_panel(
                        cx,
                        "Notifications",
                        "Choose how and when you receive updates.",
                        "Email alerts are enabled for build failures.",
                    )],
                ),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-vertical");

        let vertical_shell = shell(cx, tabs);
        let body = centered(cx, vertical_shell);
        section(cx, "Vertical", body)
    };

    let disabled = {
        let tabs = shadcn::Tabs::uncontrolled(Some("home"))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new(
                    "home",
                    "Home",
                    [card_panel(
                        cx,
                        "Home",
                        "This panel remains interactive.",
                        "The disabled tab cannot be focused or activated.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "settings",
                    "Disabled",
                    [card_panel(
                        cx,
                        "Disabled",
                        "This panel should not become active.",
                        "",
                    )],
                )
                .disabled(true),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-disabled");

        let disabled_shell = shell(cx, tabs);
        let body = centered(cx, disabled_shell);
        section(cx, "Disabled", body)
    };

    let icons = {
        let preview_trigger = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N1).items_center(),
            |cx| {
                vec![
                    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.app-window")),
                    cx.text("Preview"),
                ]
            },
        );
        let code_trigger = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N1).items_center(),
            |cx| {
                vec![
                    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.code")),
                    cx.text("Code"),
                ]
            },
        );

        let tabs = shadcn::Tabs::uncontrolled(Some("preview"))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new(
                    "preview",
                    "Preview",
                    [card_panel(
                        cx,
                        "Preview",
                        "Visual output for the current component.",
                        "Switch between preview and code using icon tabs.",
                    )],
                )
                .trigger_child(preview_trigger),
                shadcn::TabsItem::new(
                    "code",
                    "Code",
                    [card_panel(
                        cx,
                        "Code",
                        "Implementation details and source view.",
                        "This panel can host syntax-highlighted snippets.",
                    )],
                )
                .trigger_child(code_trigger),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-icons");

        let icons_shell = shell(cx, tabs);
        let body = centered(cx, icons_shell);
        section(cx, "Icons", body)
    };

    let rtl = {
        let tabs = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Tabs::uncontrolled(Some("overview"))
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
                    .items([
                        shadcn::TabsItem::new(
                            "overview",
                            "Overview",
                            [card_panel(
                                cx,
                                "Overview",
                                "RTL layout should keep keyboard and focus behavior intact.",
                                "Direction-sensitive navigation is provided by direction context.",
                            )],
                        ),
                        shadcn::TabsItem::new(
                            "analytics",
                            "Analytics",
                            [card_panel(
                                cx,
                                "Analytics",
                                "Arrow-key movement follows RTL expectations.",
                                "Verify trigger order and selected styling in RTL mode.",
                            )],
                        ),
                        shadcn::TabsItem::new(
                            "reports",
                            "Reports",
                            [card_panel(
                                cx,
                                "Reports",
                                "Panel composition remains identical under RTL.",
                                "Only directional behavior should change.",
                            )],
                        ),
                    ])
                    .into_element(cx)
            },
        )
        .test_id("ui-gallery-tabs-rtl");

        let rtl_shell = shell(cx, tabs);
        let body = centered(cx, rtl_shell);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("A set of layered sections of content that are displayed one at a time."),
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![demo, line, vertical, disabled, icons, rtl],
        ),
    ]
}

fn preview_accordion(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let _ = value;

    let max_w_lg = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(512.0)))
        .min_w_0();
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    // Mirrors the top-level `accordion-demo` preview slot.
    let demo = {
        let accordion = shadcn::Accordion::single_uncontrolled(Some("shipping"))
            .collapsible(true)
            .refine_layout(max_w_lg.clone())
            .items([
                shadcn::AccordionItem::new(
                    "shipping",
                    shadcn::AccordionTrigger::new(vec![cx.text("What are your shipping options?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "We offer standard (5-7 days), express (2-3 days), and overnight shipping. Free shipping on international orders.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "returns",
                    shadcn::AccordionTrigger::new(vec![cx.text("What is your return policy?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Returns accepted within 30 days. Items must be unused and in original packaging. Refunds processed within 5-7 business days.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "support",
                    shadcn::AccordionTrigger::new(vec![cx.text("How can I contact customer support?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Reach us via email, live chat, or phone. We respond within 24 hours during business days.",
                    )]),
                ),
            ])
            .into_element(cx);
        centered(cx, accordion)
    };

    let basic = {
        let accordion = shadcn::Accordion::single_uncontrolled(Some("item-1"))
            .collapsible(true)
            .refine_layout(max_w_lg.clone())
            .items([
                shadcn::AccordionItem::new(
                    "item-1",
                    shadcn::AccordionTrigger::new(vec![cx.text("How do I reset my password?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Click on 'Forgot Password' on the login page, enter your email address, and we'll send you a link to reset your password. The link will expire in 24 hours.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "item-2",
                    shadcn::AccordionTrigger::new(vec![cx.text("Can I change my subscription plan?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Yes, you can upgrade or downgrade your plan at any time from your account settings. Changes will be reflected in your next billing cycle.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "item-3",
                    shadcn::AccordionTrigger::new(vec![cx.text("What payment methods do you accept?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "We accept all major credit cards, PayPal, and bank transfers. All payments are processed securely through our payment partners.",
                    )]),
                ),
            ])
            .into_element(cx);
        let body = centered(cx, accordion);
        section(cx, "Basic", body)
    };

    let multiple = {
        let accordion = shadcn::Accordion::multiple_uncontrolled(["notifications"])
            .refine_layout(max_w_lg.clone())
            .items([
                shadcn::AccordionItem::new(
                    "notifications",
                    shadcn::AccordionTrigger::new(vec![cx.text("Notification Settings")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Manage how you receive notifications. You can enable email alerts for updates or push notifications for mobile devices.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "privacy",
                    shadcn::AccordionTrigger::new(vec![cx.text("Privacy & Security")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Control your privacy settings and security preferences. Enable two-factor authentication, manage connected devices, review active sessions, and configure data sharing preferences. You can also download your data or delete your account.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "billing",
                    shadcn::AccordionTrigger::new(vec![cx.text("Billing & Subscription")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "View your current plan, payment history, and upcoming invoices. Update your payment method, change your subscription tier, or cancel your subscription.",
                    )]),
                ),
            ])
            .into_element(cx);
        let body = centered(cx, accordion);
        section(cx, "Multiple", body)
    };

    let disabled = {
        let accordion = shadcn::Accordion::single_uncontrolled(None::<Arc<str>>)
            .collapsible(true)
            .refine_layout(max_w_lg.clone())
            .items([
                shadcn::AccordionItem::new(
                    "item-1",
                    shadcn::AccordionTrigger::new(vec![cx.text("Can I access my account history?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Yes, you can view your complete account history including all transactions, plan changes, and support tickets in the Account History section of your dashboard.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "item-2",
                    shadcn::AccordionTrigger::new(vec![cx.text("Premium feature information")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "This section contains information about premium features. Upgrade your plan to access this content.",
                    )]),
                )
                .disabled(true),
                shadcn::AccordionItem::new(
                    "item-3",
                    shadcn::AccordionTrigger::new(vec![cx.text("How do I update my email address?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "You can update your email address in your account settings. You'll receive a verification email at your new address to confirm the change.",
                    )]),
                ),
            ])
            .into_element(cx);
        let body = centered(cx, accordion);
        section(cx, "Disabled", body)
    };

    let borders = {
        let accordion = shadcn::Accordion::single_uncontrolled(Some("billing"))
            .collapsible(true)
            .refine_layout(LayoutRefinement::default().w_full())
            .items([
                shadcn::AccordionItem::new(
                    "billing",
                    shadcn::AccordionTrigger::new(vec![cx.text("How does billing work?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "We offer monthly and annual subscription plans. Billing is charged at the beginning of each cycle, and you can cancel anytime. All plans include automatic backups, 24/7 support, and unlimited team members.",
                    )]),
                )
                .refine_style(ChromeRefinement::default().px(Space::N4)),
                shadcn::AccordionItem::new(
                    "security",
                    shadcn::AccordionTrigger::new(vec![cx.text("Is my data secure?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Yes. We use end-to-end encryption, SOC 2 Type II compliance, and regular third-party security audits. All data is encrypted at rest and in transit using industry-standard protocols.",
                    )]),
                )
                .refine_style(ChromeRefinement::default().px(Space::N4)),
                shadcn::AccordionItem::new(
                    "integration",
                    shadcn::AccordionTrigger::new(vec![cx.text("What integrations do you support?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "We integrate with 500+ popular tools including Slack, Zapier, Salesforce, HubSpot, and more. You can also build custom integrations using our REST API and webhooks.",
                    )]),
                )
                .refine_style(ChromeRefinement::default().px(Space::N4)),
            ])
            .into_element(cx);

        let wrapper_props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Lg),
                max_w_lg.clone(),
            )
        });
        let wrapper = cx.container(wrapper_props, move |_cx| vec![accordion]);

        let body = centered(cx, wrapper);
        section(cx, "Borders", body)
    };

    let card = {
        let accordion = shadcn::Accordion::single_uncontrolled(Some("plans"))
            .collapsible(true)
            .refine_layout(LayoutRefinement::default().w_full())
            .items([
                shadcn::AccordionItem::new(
                    "plans",
                    shadcn::AccordionTrigger::new(vec![cx.text("What subscription plans do you offer?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "We offer three subscription tiers: Starter ($9/month), Professional ($29/month), and Enterprise ($99/month). Each plan includes increasing storage limits, API access, priority support, and team collaboration features.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "billing",
                    shadcn::AccordionTrigger::new(vec![cx.text("How does billing work?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Billing occurs automatically at the start of each billing cycle. We accept all major credit cards, PayPal, and ACH transfers for enterprise customers. You'll receive an invoice via email after each payment.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "cancel",
                    shadcn::AccordionTrigger::new(vec![cx.text("How do I cancel my subscription?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "You can cancel your subscription anytime from your account settings. There are no cancellation fees or penalties. Your access will continue until the end of your current billing period.",
                    )]),
                ),
            ])
            .into_element(cx);

        let card = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Subscription & Billing").into_element(cx),
                shadcn::CardDescription::new(
                    "Common questions about your account, plans, payments and cancellations.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![accordion]).into_element(cx),
        ])
        .refine_layout(max_w_sm.clone())
        .into_element(cx);

        let body = centered(cx, card);
        section(cx, "Card", body)
    };

    let rtl = {
        let accordion = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Accordion::single_uncontrolled(Some("item-1"))
                    .collapsible(true)
                    .dir(Some(fret_ui_kit::primitives::direction::LayoutDirection::Rtl))
                    .refine_layout(max_w_lg.clone())
                    .items([
                        shadcn::AccordionItem::new(
                            "item-1",
                            shadcn::AccordionTrigger::new(vec![cx.text("كيف يمكنني إعادة تعيين كلمة المرور؟")]),
                            shadcn::AccordionContent::new(vec![cx.text(
                                "انقر على 'نسيت كلمة المرور' في صفحة تسجيل الدخول، أدخل عنوان بريدك الإلكتروني، وسنرسل لك رابطًا لإعادة تعيين كلمة المرور. سينتهي صلاحية الرابط خلال 24 ساعة.",
                            )]),
                        ),
                        shadcn::AccordionItem::new(
                            "item-2",
                            shadcn::AccordionTrigger::new(vec![cx.text("هل يمكنني تغيير خطة الاشتراك الخاصة بي؟")]),
                            shadcn::AccordionContent::new(vec![cx.text(
                                "نعم، يمكنك ترقية أو تخفيض خطتك في أي وقت من إعدادات حسابك. ستظهر التغييرات في دورة الفوترة التالية.",
                            )]),
                        ),
                        shadcn::AccordionItem::new(
                            "item-3",
                            shadcn::AccordionTrigger::new(vec![cx.text("ما هي طرق الدفع التي تقبلونها؟")]),
                            shadcn::AccordionContent::new(vec![cx.text(
                                "نقبل جميع بطاقات الائتمان الرئيسية و PayPal والتحويلات المصرفية. تتم معالجة جميع المدفوعات بأمان من خلال شركاء الدفع لدينا.",
                            )]),
                        ),
                    ])
                    .into_element(cx)
            },
        );
        let body = centered(cx, accordion);
        section(cx, "RTL", body)
    };

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![basic, multiple, disabled, borders, card, rtl],
    );

    vec![demo, examples]
}

fn preview_table(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct TableModels {
        actions_open_1: Option<Model<bool>>,
        actions_open_2: Option<Model<bool>>,
        actions_open_3: Option<Model<bool>>,
    }

    let state = cx.with_state(TableModels::default, |st| st.clone());
    let (actions_open_1, actions_open_2, actions_open_3) = match (
        state.actions_open_1,
        state.actions_open_2,
        state.actions_open_3,
    ) {
        (Some(open_1), Some(open_2), Some(open_3)) => (open_1, open_2, open_3),
        _ => {
            let open_1 = cx.app.models_mut().insert(false);
            let open_2 = cx.app.models_mut().insert(false);
            let open_3 = cx.app.models_mut().insert(false);
            cx.with_state(TableModels::default, |st| {
                st.actions_open_1 = Some(open_1.clone());
                st.actions_open_2 = Some(open_2.clone());
                st.actions_open_3 = Some(open_3.clone());
            });
            (open_1, open_2, open_3)
        }
    };

    let invoice_w = fret_core::Px(128.0);
    let status_w = fret_core::Px(120.0);
    let method_w = fret_core::Px(180.0);
    let amount_w = fret_core::Px(132.0);

    let invoices: [(&str, &str, &str, &str); 7] = [
        ("INV001", "Paid", "$250.00", "Credit Card"),
        ("INV002", "Pending", "$150.00", "PayPal"),
        ("INV003", "Unpaid", "$350.00", "Bank Transfer"),
        ("INV004", "Paid", "$450.00", "Credit Card"),
        ("INV005", "Paid", "$550.00", "PayPal"),
        ("INV006", "Pending", "$200.00", "Bank Transfer"),
        ("INV007", "Unpaid", "$300.00", "Credit Card"),
    ];

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let align_end = |cx: &mut ElementContext<'_, App>, child: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_end(),
            move |_cx| [child],
        )
    };

    let make_invoice_table =
        |cx: &mut ElementContext<'_, App>,
         rows: &[(&'static str, &'static str, &'static str, &'static str)],
         include_footer: bool,
         test_id: &'static str| {
            let header = shadcn::TableHeader::new(vec![
                shadcn::TableRow::new(
                    4,
                    vec![
                        shadcn::TableHead::new("Invoice")
                            .refine_layout(LayoutRefinement::default().w_px(invoice_w))
                            .into_element(cx),
                        shadcn::TableHead::new("Status")
                            .refine_layout(LayoutRefinement::default().w_px(status_w))
                            .into_element(cx),
                        shadcn::TableHead::new("Method")
                            .refine_layout(LayoutRefinement::default().w_px(method_w))
                            .into_element(cx),
                        shadcn::TableHead::new("Amount")
                            .refine_layout(LayoutRefinement::default().w_px(amount_w))
                            .into_element(cx),
                    ],
                )
                .border_bottom(true)
                .into_element(cx),
            ])
            .into_element(cx);

            let body_rows = rows
                .iter()
                .copied()
                .map(|(invoice, status, amount, method)| {
                    shadcn::TableRow::new(
                        4,
                        vec![
                            shadcn::TableCell::new(cx.text(invoice))
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w))
                                .into_element(cx),
                            shadcn::TableCell::new(cx.text(status))
                                .refine_layout(LayoutRefinement::default().w_px(status_w))
                                .into_element(cx),
                            shadcn::TableCell::new(cx.text(method))
                                .refine_layout(LayoutRefinement::default().w_px(method_w))
                                .into_element(cx),
                            {
                                let amount_text = cx.text(amount);
                                shadcn::TableCell::new(align_end(cx, amount_text))
                                    .refine_layout(LayoutRefinement::default().w_px(amount_w))
                                    .into_element(cx)
                            },
                        ],
                    )
                    .into_element(cx)
                })
                .collect::<Vec<_>>();

            let body = shadcn::TableBody::new(body_rows).into_element(cx);

            let mut children = vec![header, body];
            if include_footer {
                let footer = shadcn::TableFooter::new(vec![
                    shadcn::TableRow::new(
                        4,
                        vec![
                            shadcn::TableCell::new(cx.text("Total"))
                                .col_span(3)
                                .refine_layout(
                                    LayoutRefinement::default()
                                        .w_px(invoice_w + status_w + method_w),
                                )
                                .into_element(cx),
                            {
                                let total_amount = cx.text("$2,500.00");
                                shadcn::TableCell::new(align_end(cx, total_amount))
                                    .refine_layout(LayoutRefinement::default().w_px(amount_w))
                                    .into_element(cx)
                            },
                        ],
                    )
                    .border_bottom(false)
                    .into_element(cx),
                ])
                .into_element(cx);
                children.push(footer);
            }

            children.push(
                shadcn::TableCaption::new("A list of your recent invoices.").into_element(cx),
            );

            shadcn::Table::new(children)
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
                .test_id(test_id)
        };

    let demo = {
        let table = make_invoice_table(cx, &invoices, true, "ui-gallery-table-demo");
        let table_shell = shell(cx, table);
        let body = centered(cx, table_shell);
        section(cx, "Demo", body)
    };

    let footer = {
        let table = make_invoice_table(cx, &invoices[..3], true, "ui-gallery-table-footer");
        let table_shell = shell(cx, table);
        let body = centered(cx, table_shell);
        section(cx, "Footer", body)
    };

    let actions = {
        let action_row = |cx: &mut ElementContext<'_, App>,
                          product: &'static str,
                          price: &'static str,
                          open_model: Model<bool>,
                          key: &'static str| {
            let trigger_id = format!("ui-gallery-table-actions-trigger-{key}");
            let dropdown = shadcn::DropdownMenu::new(open_model.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("?")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .toggle_model(open_model.clone())
                        .test_id(trigger_id.clone())
                        .into_element(cx)
                },
                |_cx| {
                    vec![
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Edit")),
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Duplicate")),
                        shadcn::DropdownMenuEntry::Separator,
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Delete").variant(
                                shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                            ),
                        ),
                    ]
                },
            );

            shadcn::TableRow::new(
                3,
                vec![
                    shadcn::TableCell::new(cx.text(product)).into_element(cx),
                    shadcn::TableCell::new(cx.text(price)).into_element(cx),
                    {
                        let action_cell = align_end(cx, dropdown);
                        shadcn::TableCell::new(action_cell).into_element(cx)
                    },
                ],
            )
            .into_element(cx)
        };

        let table = shadcn::Table::new(vec![
            shadcn::TableHeader::new(vec![
                shadcn::TableRow::new(
                    3,
                    vec![
                        shadcn::TableHead::new("Product")
                            .refine_layout(LayoutRefinement::default().w_px(Px(280.0)))
                            .into_element(cx),
                        shadcn::TableHead::new("Price")
                            .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
                            .into_element(cx),
                        shadcn::TableHead::new("Actions")
                            .refine_layout(LayoutRefinement::default().w_px(Px(120.0)))
                            .into_element(cx),
                    ],
                )
                .border_bottom(true)
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::TableBody::new(vec![
                action_row(cx, "Gaming Mouse", "$129.99", actions_open_1, "row-1"),
                action_row(cx, "Mechanical Keyboard", "$89.99", actions_open_2, "row-2"),
                action_row(cx, "4K Monitor", "$299.99", actions_open_3, "row-3"),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-table-actions");

        let table_shell = shell(cx, table);
        let body = centered(cx, table_shell);
        section(cx, "Actions", body)
    };

    let rtl = {
        let rtl_table = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let rows: [(&str, &str, &str, &str); 3] = [
                    ("INV001", "Paid", "$250.00", "Credit Card"),
                    ("INV002", "Pending", "$150.00", "PayPal"),
                    ("INV003", "Unpaid", "$350.00", "Bank Transfer"),
                ];
                make_invoice_table(cx, &rows, true, "ui-gallery-table-rtl")
            },
        );

        let table_shell = shell(cx, rtl_table);
        let body = centered(cx, table_shell);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("A responsive table component."),
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![demo, footer, actions, rtl],
        ),
    ]
}

fn preview_progress(cx: &mut ElementContext<'_, App>, _progress: Model<f32>) -> Vec<AnyElement> {
    use std::time::Duration;

    use fret_core::{SemanticsRole, TimerToken};
    use fret_runtime::Effect;
    use fret_ui::Invalidation;
    use fret_ui::element::SemanticsProps;
    use fret_ui_kit::primitives::direction as direction_prim;

    #[derive(Default, Clone)]
    struct ProgressModels {
        demo_value: Option<Model<f32>>,
        demo_token: Option<Model<Option<TimerToken>>>,
        label_value: Option<Model<f32>>,
        controlled_values: Option<Model<Vec<f32>>>,
    }

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let state = cx.with_state(ProgressModels::default, |st| st.clone());

    let demo_value = match state.demo_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(13.0);
            cx.with_state(ProgressModels::default, |st| {
                st.demo_value = Some(model.clone())
            });
            model
        }
    };

    let demo_token = match state.demo_token {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<TimerToken>);
            cx.with_state(ProgressModels::default, |st| {
                st.demo_token = Some(model.clone())
            });
            model
        }
    };

    let label_value = match state.label_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(66.0);
            cx.with_state(ProgressModels::default, |st| {
                st.label_value = Some(model.clone())
            });
            model
        }
    };

    let controlled_values = match state.controlled_values {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![50.0]);
            cx.with_state(ProgressModels::default, |st| {
                st.controlled_values = Some(model.clone())
            });
            model
        }
    };

    let demo = cx.keyed("ui_gallery.progress.demo", |cx| {
        let demo_value_for_timer = demo_value.clone();
        let demo_token_for_timer = demo_token.clone();

        let body = cx.semantics_with_id(
            SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("ui-gallery-progress-demo")),
                ..Default::default()
            },
            move |cx, id| {
                cx.timer_on_timer_for(
                    id,
                    Arc::new(move |host, action_cx, token| {
                        let expected = host
                            .models_mut()
                            .read(&demo_token_for_timer, Clone::clone)
                            .ok()
                            .flatten();
                        if expected != Some(token) {
                            return false;
                        }
                        let _ = host
                            .models_mut()
                            .update(&demo_value_for_timer, |v| *v = 66.0);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );

                let armed = cx
                    .get_model_copied(&demo_token, Invalidation::Paint)
                    .unwrap_or(None)
                    .is_some();
                if !armed {
                    let token = cx.app.next_timer_token();
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&demo_token, |v| *v = Some(token));
                    let _ = cx.app.models_mut().update(&demo_value, |v| *v = 13.0);
                    cx.app.push_effect(Effect::SetTimer {
                        window: Some(cx.window),
                        token,
                        after: Duration::from_millis(500),
                        repeat: None,
                    });
                }

                let bar = shadcn::Progress::new(demo_value.clone())
                    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                    .into_element(cx);

                vec![centered(cx, bar)]
            },
        );

        section(cx, "Demo", body)
    });

    let label = cx.keyed("ui_gallery.progress.label", |cx| {
        let label_row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center(),
            |cx| {
                vec![
                    shadcn::FieldLabel::new("Upload progress").into_element(cx),
                    shadcn::FieldLabel::new("66%")
                        .refine_layout(LayoutRefinement::default().ml_auto())
                        .into_element(cx),
                ]
            },
        );

        let field = shadcn::Field::new(vec![
            label_row,
            shadcn::Progress::new(label_value.clone()).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx);

        let body = centered(cx, field);
        section(cx, "Label", body)
    });

    let controlled = cx.keyed("ui_gallery.progress.controlled", |cx| {
        let values = controlled_values.clone();
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
                vec![
                    shadcn::Progress::new_values_first(values.clone()).into_element(cx),
                    shadcn::Slider::new(values)
                        .range(0.0, 100.0)
                        .step(1.0)
                        .a11y_label("Progress value")
                        .into_element(cx),
                ]
            },
        );

        let centered_body = centered(cx, body);
        section(cx, "Controlled", centered_body)
    });

    let rtl = cx.keyed("ui_gallery.progress.rtl", |cx| {
        let body = direction_prim::with_direction_provider(
            cx,
            direction_prim::LayoutDirection::Rtl,
            |cx| {
                let label_row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .items_center(),
                    |cx| {
                        vec![
                            shadcn::FieldLabel::new("٦٦%").into_element(cx),
                            shadcn::FieldLabel::new("تقدم الرفع")
                                .refine_layout(LayoutRefinement::default().ml_auto())
                                .into_element(cx),
                        ]
                    },
                );

                let field = shadcn::Field::new(vec![
                    label_row,
                    shadcn::Progress::new(label_value.clone())
                        .mirror_in_rtl(true)
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
                .into_element(cx);

                centered(cx, field)
            },
        );

        section(cx, "RTL", body)
    });

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![label, controlled, rtl],
    );

    vec![demo, examples]
}

fn preview_dropdown_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_dropdown_menu(cx, open, last_action)
}

fn preview_menus(
    cx: &mut ElementContext<'_, App>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("DropdownMenu")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-menus-dropdown-trigger")
                .toggle_model(dropdown_open.clone())
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Apple")
                        .test_id("ui-gallery-menus-dropdown-item-apple")
                        .on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Orange").on_select(CMD_MENU_DROPDOWN_ORANGE),
                ),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let context_menu = shadcn::ContextMenu::new(context_menu_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("ContextMenu (right click)")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-menus-context-trigger")
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Action")
                        .test_id("ui-gallery-menus-context-item-action")
                        .on_select(CMD_MENU_CONTEXT_ACTION),
                ),
                shadcn::ContextMenuEntry::Separator,
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| [dropdown, context_menu],
        ),
        cx.text(format!("last action: {last}")),
    ]
}

fn preview_context_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_context_menu(cx, open, last_action)
}

fn preview_command_palette(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    query: Model<String>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_command_palette(cx, open, query, last_action)
}

fn sonner_position_key(position: shadcn::ToastPosition) -> &'static str {
    match position {
        shadcn::ToastPosition::TopLeft => "top-left",
        shadcn::ToastPosition::TopCenter => "top-center",
        shadcn::ToastPosition::TopRight => "top-right",
        shadcn::ToastPosition::BottomLeft => "bottom-left",
        shadcn::ToastPosition::BottomCenter => "bottom-center",
        shadcn::ToastPosition::BottomRight => "bottom-right",
    }
}

fn preview_sonner(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
    sonner_position: Model<shadcn::ToastPosition>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct SonnerModels {
        pending_promise: Option<Model<Option<shadcn::ToastId>>>,
    }

    let pending_promise = cx.with_state(SonnerModels::default, |st| st.pending_promise.clone());
    let sonner = shadcn::Sonner::global(&mut *cx.app);

    let pending_promise = match pending_promise {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<shadcn::ToastId>);
            cx.with_state(SonnerModels::default, |st| {
                st.pending_promise = Some(model.clone())
            });
            model
        }
    };

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let row = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            move |_cx| children,
        )
    };

    let button = |cx: &mut ElementContext<'_, App>,
                  label: &'static str,
                  test_id: &'static str,
                  on_activate: fret_ui::action::OnActivate| {
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .on_activate(on_activate)
            .test_id(test_id)
            .into_element(cx)
    };

    let demo = {
        let sonner = sonner.clone();
        let last_action_model = last_action.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            sonner.toast_message(
                host,
                action_cx.window,
                "Event has been created",
                shadcn::ToastMessageOptions::new()
                    .description("Sunday, December 03, 2023 at 9:00 AM")
                    .action("Undo", CMD_TOAST_ACTION),
            );
            let _ = host.models_mut().update(&last_action_model, |v| {
                *v = Arc::<str>::from("sonner.demo");
            });
            host.request_redraw(action_cx.window);
        });

        let show = button(cx, "Show Toast", "ui-gallery-sonner-demo-show", on_activate);
        let content = centered(cx, show).attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-demo"),
        );
        section(cx, "Demo", content)
    };

    let types = {
        let default_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_message(
                        host,
                        action_cx.window,
                        "Event has been created",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.default");
                    });
                    host.request_redraw(action_cx.window);
                });
            button(
                cx,
                "Default",
                "ui-gallery-sonner-types-default",
                on_activate,
            )
        };

        let success_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_success_message(
                        host,
                        action_cx.window,
                        "Event has been created",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.success");
                    });
                    host.request_redraw(action_cx.window);
                });
            button(
                cx,
                "Success",
                "ui-gallery-sonner-types-success",
                on_activate,
            )
        };

        let info_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_info_message(
                        host,
                        action_cx.window,
                        "Be at the area 10 minutes before the event time",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.info");
                    });
                    host.request_redraw(action_cx.window);
                });
            button(cx, "Info", "ui-gallery-sonner-types-info", on_activate)
        };

        let warning_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_warning_message(
                        host,
                        action_cx.window,
                        "Event start time cannot be earlier than 8am",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.warning");
                    });
                    host.request_redraw(action_cx.window);
                });
            button(
                cx,
                "Warning",
                "ui-gallery-sonner-types-warning",
                on_activate,
            )
        };

        let error_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_error_message(
                        host,
                        action_cx.window,
                        "Event has not been created",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.error");
                    });
                    host.request_redraw(action_cx.window);
                });
            button(cx, "Error", "ui-gallery-sonner-types-error", on_activate)
        };

        let promise_button = {
            let sonner = sonner.clone();
            let pending_model = pending_promise.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let pending = host.models_mut().get_copied(&pending_model).flatten();
                    if let Some(id) = pending {
                        sonner.toast_success_update(
                            host,
                            action_cx.window,
                            id,
                            "Event has been created",
                        );
                        let _ = host
                            .models_mut()
                            .update(&pending_model, |slot| *slot = None);
                        let _ = host.models_mut().update(&last_action_model, |v| {
                            *v = Arc::<str>::from("sonner.types.promise.resolve");
                        });
                    } else {
                        let promise = sonner.toast_promise(host, action_cx.window, "Loading...");
                        let _ = host
                            .models_mut()
                            .update(&pending_model, |slot| *slot = Some(promise.id()));
                        let _ = host.models_mut().update(&last_action_model, |v| {
                            *v = Arc::<str>::from("sonner.types.promise.start");
                        });
                    }
                    host.request_redraw(action_cx.window);
                });
            button(
                cx,
                "Promise",
                "ui-gallery-sonner-types-promise",
                on_activate,
            )
        };

        let buttons_row = row(
            cx,
            vec![
                default_button,
                success_button,
                info_button,
                warning_button,
                error_button,
                promise_button,
            ],
        );

        let pending = cx
            .get_model_copied(&pending_promise, Invalidation::Layout)
            .flatten()
            .is_some();

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full()),
            move |cx| {
                vec![
                    buttons_row,
                    shadcn::typography::muted(
                        cx,
                        if pending {
                            "Promise toast pending: click Promise again to resolve."
                        } else {
                            "Promise toast idle: click Promise to start loading state."
                        },
                    ),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-types"),
        );

        section(cx, "Types", content)
    };

    let description = {
        let sonner = sonner.clone();
        let last_action_model = last_action.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            sonner.toast_message(
                host,
                action_cx.window,
                "Event has been created",
                shadcn::ToastMessageOptions::new().description("Monday, January 3rd at 6:00pm"),
            );
            let _ = host.models_mut().update(&last_action_model, |v| {
                *v = Arc::<str>::from("sonner.description");
            });
            host.request_redraw(action_cx.window);
        });

        let show = button(
            cx,
            "Show Toast",
            "ui-gallery-sonner-description-show",
            on_activate,
        );
        let content = centered(cx, show).attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-description"),
        );

        section(cx, "Description", content)
    };

    let position = {
        let current = cx
            .get_model_copied(&sonner_position, Invalidation::Layout)
            .unwrap_or(shadcn::ToastPosition::TopCenter);

        let make_position_button =
            |cx: &mut ElementContext<'_, App>,
             label: &'static str,
             test_id: &'static str,
             target: shadcn::ToastPosition| {
                let sonner = sonner.clone();
                let position_model = sonner_position.clone();
                let last_action_model = last_action.clone();
                let on_activate: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let _ = host.models_mut().update(&position_model, |v| *v = target);
                        sonner.toast_message(
                            host,
                            action_cx.window,
                            "Event has been created",
                            shadcn::ToastMessageOptions::new()
                                .description(format!("position: {}", sonner_position_key(target))),
                        );
                        let _ = host.models_mut().update(&last_action_model, |v| {
                            *v = Arc::<str>::from(format!(
                                "sonner.position.{}",
                                sonner_position_key(target)
                            ));
                        });
                        host.request_redraw(action_cx.window);
                    });
                button(cx, label, test_id, on_activate)
            };

        let make_position_button = make_position_button;
        let top_left = make_position_button(
            cx,
            "Top Left",
            "ui-gallery-sonner-position-top-left",
            shadcn::ToastPosition::TopLeft,
        );
        let top_center = make_position_button(
            cx,
            "Top Center",
            "ui-gallery-sonner-position-top-center",
            shadcn::ToastPosition::TopCenter,
        );
        let top_right = make_position_button(
            cx,
            "Top Right",
            "ui-gallery-sonner-position-top-right",
            shadcn::ToastPosition::TopRight,
        );
        let bottom_left = make_position_button(
            cx,
            "Bottom Left",
            "ui-gallery-sonner-position-bottom-left",
            shadcn::ToastPosition::BottomLeft,
        );
        let bottom_center = make_position_button(
            cx,
            "Bottom Center",
            "ui-gallery-sonner-position-bottom-center",
            shadcn::ToastPosition::BottomCenter,
        );
        let bottom_right = make_position_button(
            cx,
            "Bottom Right",
            "ui-gallery-sonner-position-bottom-right",
            shadcn::ToastPosition::BottomRight,
        );

        let top_row = row(cx, vec![top_left, top_center, top_right]);
        let bottom_row = row(cx, vec![bottom_left, bottom_center, bottom_right]);
        let rows = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default()),
            move |_cx| vec![top_row, bottom_row],
        );

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full()),
            move |cx| {
                vec![
                    centered(cx, rows),
                    shadcn::typography::muted(
                        cx,
                        format!("Current toaster position: {}", sonner_position_key(current)),
                    ),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-position"),
        );

        section(cx, "Position", content)
    };

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        cx.text("An opinionated toast component for React."),
        cx.text(format!("last action: {last}")),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, types, description, position]
        }),
    ]
}

fn preview_toast(
    cx: &mut ElementContext<'_, App>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let deprecated_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Toast is deprecated").into_element(cx),
            shadcn::CardDescription::new(
                "The toast component is deprecated in shadcn/ui docs. Use Sonner instead.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![shadcn::typography::muted(
            cx,
            "This page intentionally keeps only the deprecation guidance to match upstream docs.",
        )])
        .into_element(cx),
        shadcn::CardFooter::new(vec![
            shadcn::Button::new("Open Sonner page")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_NAV_SONNER)
                .test_id("ui-gallery-toast-open-sonner")
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-toast-deprecated");

    let centered_card = centered(cx, deprecated_card);

    vec![
        cx.text("A succinct message that is displayed temporarily."),
        centered_card,
    ]
}

fn preview_overlay(
    cx: &mut ElementContext<'_, App>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnDismissRequest;

    let last_action_status = {
        let last = cx
            .app
            .models()
            .get_cloned(&last_action)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        let text = format!("last action: {last}");
        cx.text(text).test_id("ui-gallery-overlay-last-action")
    };

    let overlays =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let overlay_reset = {
                use fret_ui::action::OnActivate;

                let dropdown_open = dropdown_open.clone();
                let context_menu_open = context_menu_open.clone();
                let context_menu_edge_open = context_menu_edge_open.clone();
                let popover_open = popover_open.clone();
                let dialog_open = dialog_open.clone();
                let alert_dialog_open = alert_dialog_open.clone();
                let sheet_open = sheet_open.clone();
                let portal_geometry_popover_open = portal_geometry_popover_open.clone();
                let last_action = last_action.clone();

                let on_activate: OnActivate = Arc::new(move |host, _cx, _reason| {
                    let _ = host.models_mut().update(&dropdown_open, |v| *v = false);
                    let _ = host.models_mut().update(&context_menu_open, |v| *v = false);
                    let _ = host
                        .models_mut()
                        .update(&context_menu_edge_open, |v| *v = false);
                    let _ = host.models_mut().update(&popover_open, |v| *v = false);
                    let _ = host.models_mut().update(&dialog_open, |v| *v = false);
                    let _ = host.models_mut().update(&alert_dialog_open, |v| *v = false);
                    let _ = host.models_mut().update(&sheet_open, |v| *v = false);
                    let _ = host
                        .models_mut()
                        .update(&portal_geometry_popover_open, |v| *v = false);
                    let _ = host.models_mut().update(&last_action, |v| {
                        *v = Arc::<str>::from("overlay:reset");
                    });
                });

                shadcn::Button::new("Reset overlays")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .test_id("ui-gallery-overlay-reset")
                    .on_activate(on_activate)
                    .into_element(cx)
            };

            let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone())
                .modal(false)
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("DropdownMenu")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-dropdown-trigger")
                            .toggle_model(dropdown_open.clone())
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Apple")
                                    .test_id("ui-gallery-dropdown-item-apple")
                                    .on_select(CMD_MENU_DROPDOWN_APPLE),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("More")
                                    .test_id("ui-gallery-dropdown-item-more")
                                    .close_on_select(false)
                                    .submenu(vec![
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("Nested action")
                                                .test_id("ui-gallery-dropdown-submenu-item-nested")
                                                .on_select(CMD_MENU_CONTEXT_ACTION),
                                        ),
                                        shadcn::DropdownMenuEntry::Separator,
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("Nested disabled")
                                                .disabled(true),
                                        ),
                                    ]),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Orange")
                                    .on_select(CMD_MENU_DROPDOWN_ORANGE),
                            ),
                            shadcn::DropdownMenuEntry::Separator,
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                            ),
                        ]
                    },
                );

            let context_menu = shadcn::ContextMenu::new(context_menu_open.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("ContextMenu (right click)")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-context-trigger")
                        .into_element(cx)
                },
                |_cx| {
                    vec![
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Action")
                                .test_id("ui-gallery-context-item-action")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Disabled").disabled(true),
                        ),
                    ]
                },
            );

            let context_menu_edge = shadcn::ContextMenu::new(context_menu_edge_open.clone())
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("ContextMenu (edge, right click)")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-context-trigger-edge")
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Action")
                                    .test_id("ui-gallery-context-edge-item-action")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                            shadcn::ContextMenuEntry::Separator,
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Disabled").disabled(true),
                            ),
                        ]
                    },
                );

            let underlay = shadcn::Button::new("Underlay (outside-press target)")
                .variant(shadcn::ButtonVariant::Secondary)
                .test_id("ui-gallery-overlay-underlay")
                .into_element(cx);

            let tooltip = shadcn::Tooltip::new(
                shadcn::Button::new("Tooltip (hover)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-tooltip-trigger")
                    .into_element(cx),
                shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                    cx,
                    "Tooltip: hover intent + placement",
                )])
                .into_element(cx)
                .test_id("ui-gallery-tooltip-content"),
            )
            .arrow(true)
            .arrow_test_id("ui-gallery-tooltip-arrow")
            .panel_test_id("ui-gallery-tooltip-panel")
            .open_delay_frames(10)
            .close_delay_frames(10)
            .side(shadcn::TooltipSide::Top)
            .into_element(cx);

            let hover_card = shadcn::HoverCard::new(
                shadcn::Button::new("HoverCard (hover)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-hovercard-trigger")
                    .into_element(cx),
                shadcn::HoverCardContent::new(vec![
                    cx.text("HoverCard content (overlay-root)"),
                    cx.text("Move pointer from trigger to content."),
                ])
                .into_element(cx)
                .test_id("ui-gallery-hovercard-content"),
            )
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx);

            let popover_open_for_dismiss = popover_open.clone();
            let last_action_for_dismiss = last_action.clone();
            let popover_on_dismiss: OnDismissRequest = Arc::new(move |host, _cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&popover_open_for_dismiss, |open| *open = false);
                let _ = host.models_mut().update(&last_action_for_dismiss, |cur| {
                    *cur = Arc::<str>::from("popover:dismissed");
                });
            });

            let popover = shadcn::Popover::new(popover_open.clone())
                .auto_focus(true)
                .on_dismiss_request(Some(popover_on_dismiss))
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Popover")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-popover-trigger")
                            .toggle_model(popover_open.clone())
                            .into_element(cx)
                    },
                    |cx| {
                        let open_dialog = shadcn::Button::new("Open dialog")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-popover-dialog-trigger")
                            .toggle_model(dialog_open.clone())
                            .into_element(cx);

                        let close = shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .test_id("ui-gallery-popover-close")
                            .toggle_model(popover_open.clone())
                            .into_element(cx);

                        shadcn::PopoverContent::new(vec![
                            cx.text("Popover content"),
                            open_dialog,
                            close,
                        ])
                        .into_element(cx)
                        .test_id("ui-gallery-popover-content")
                    },
                );

            let dialog = shadcn::Dialog::new(dialog_open.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Dialog")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dialog-trigger")
                        .toggle_model(dialog_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::DialogContent::new(vec![
                        shadcn::DialogHeader::new(vec![
                            shadcn::DialogTitle::new("Dialog").into_element(cx),
                            shadcn::DialogDescription::new("Escape / overlay click closes")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                        {
                            let body = stack::vstack(
                                cx,
                                stack::VStackProps::default().gap(Space::N2).layout(
                                    LayoutRefinement::default().w_full().min_w_0().min_h_0(),
                                ),
                                |cx| {
                                    (0..64)
                                        .map(|i| {
                                            cx.text(format!("Scrollable content line {}", i + 1))
                                        })
                                        .collect::<Vec<_>>()
                                },
                            );

                            shadcn::ScrollArea::new([body])
                                .refine_layout(
                                    LayoutRefinement::default()
                                        .w_full()
                                        .h_px(Px(240.0))
                                        .min_w_0()
                                        .min_h_0(),
                                )
                                .viewport_test_id("ui-gallery-dialog-scroll-viewport")
                                .into_element(cx)
                        },
                        shadcn::DialogFooter::new(vec![
                            shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .test_id("ui-gallery-dialog-close")
                                .toggle_model(dialog_open.clone())
                                .into_element(cx),
                            shadcn::Button::new("Confirm")
                                .variant(shadcn::ButtonVariant::Outline)
                                .test_id("ui-gallery-dialog-confirm")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id("ui-gallery-dialog-content")
                },
            );

            let alert_dialog = shadcn::AlertDialog::new(alert_dialog_open.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("AlertDialog")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-alert-dialog-trigger")
                        .toggle_model(alert_dialog_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::AlertDialogContent::new(vec![
                        shadcn::AlertDialogHeader::new(vec![
                            shadcn::AlertDialogTitle::new("Are you absolutely sure?")
                                .into_element(cx),
                            shadcn::AlertDialogDescription::new(
                                "This is non-closable by overlay click.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::AlertDialogFooter::new(vec![
                            shadcn::AlertDialogCancel::new("Cancel", alert_dialog_open.clone())
                                .test_id("ui-gallery-alert-dialog-cancel")
                                .into_element(cx),
                            shadcn::AlertDialogAction::new("Continue", alert_dialog_open.clone())
                                .test_id("ui-gallery-alert-dialog-action")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id("ui-gallery-alert-dialog-content")
                },
            );

            let sheet = shadcn::Sheet::new(sheet_open.clone())
                .side(shadcn::SheetSide::Right)
                .size(Px(360.0))
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Sheet")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-sheet-trigger")
                            .toggle_model(sheet_open.clone())
                            .into_element(cx)
                    },
                    |cx| {
                        shadcn::SheetContent::new(vec![
                            shadcn::SheetHeader::new(vec![
                                shadcn::SheetTitle::new("Sheet").into_element(cx),
                                shadcn::SheetDescription::new("A modal side panel.")
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                            {
                                let body = stack::vstack(
                                    cx,
                                    stack::VStackProps::default().gap(Space::N2).layout(
                                        LayoutRefinement::default().w_full().min_w_0().min_h_0(),
                                    ),
                                    |cx| {
                                        (0..96)
                                            .map(|i| cx.text(format!("Sheet body line {}", i + 1)))
                                            .collect::<Vec<_>>()
                                    },
                                );

                                shadcn::ScrollArea::new([body])
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .flex_1()
                                            .w_full()
                                            .min_w_0()
                                            .min_h_0(),
                                    )
                                    .viewport_test_id("ui-gallery-sheet-scroll-viewport")
                                    .into_element(cx)
                            },
                            shadcn::SheetFooter::new(vec![
                                shadcn::Button::new("Close")
                                    .variant(shadcn::ButtonVariant::Secondary)
                                    .test_id("ui-gallery-sheet-close")
                                    .toggle_model(sheet_open.clone())
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ])
                        .into_element(cx)
                        .test_id("ui-gallery-sheet-content")
                    },
                );

            let portal_geometry = {
                let popover = shadcn::Popover::new(portal_geometry_popover_open.clone())
                    .side(shadcn::PopoverSide::Right)
                    .align(shadcn::PopoverAlign::Start)
                    .side_offset(Px(8.0))
                    .window_margin(Px(8.0))
                    .arrow(true)
                    .into_element(
                        cx,
                        |cx| {
                            shadcn::Button::new("Portal geometry (scroll + clamp)")
                                .variant(shadcn::ButtonVariant::Outline)
                                .test_id("ui-gallery-portal-geometry-trigger")
                                .toggle_model(portal_geometry_popover_open.clone())
                                .into_element(cx)
                        },
                        |cx| {
                            let close = shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .test_id("ui-gallery-portal-geometry-popover-close")
                                .toggle_model(portal_geometry_popover_open.clone())
                                .into_element(cx);

                            shadcn::PopoverContent::new(vec![
                                cx.text("Popover content (placement + clamp)"),
                                cx.text("Wheel-scroll the viewport while open."),
                                close,
                            ])
                            .refine_layout(
                                LayoutRefinement::default().w_px(Px(360.0)).h_px(Px(220.0)),
                            )
                            .into_element(cx)
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .test_id("ui-gallery-portal-geometry-popover-content"),
                            )
                        },
                    );

                let items = (1..=48)
                    .map(|i| cx.text(format!("Scroll item {i:02}")))
                    .collect::<Vec<_>>();

                let body = stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |_cx| {
                    let mut out: Vec<AnyElement> = Vec::with_capacity(items.len() + 2);
                    out.push(popover);
                    out.extend(items);
                    out
                });

                let scroll = shadcn::ScrollArea::new(vec![body])
                    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)).h_px(Px(160.0)))
                    .into_element(cx);

                let scroll = scroll.attach_semantics(
                    SemanticsDecoration::default()
                        .test_id("ui-gallery-portal-geometry-scroll-area"),
                );

                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Portal geometry").into_element(cx),
                        shadcn::CardDescription::new(
                            "Validates floating placement under scroll + window clamp.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![scroll]).into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
            };

            let body = stack::vstack(
                cx,
                stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
                |cx| {
                    let gap = cx.with_theme(|theme| {
                        fret_ui_kit::MetricRef::space(Space::N2).resolve(theme)
                    });

                    let row = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
                        let layout = cx.with_theme(|theme| {
                            decl_style::layout_style(
                                theme,
                                LayoutRefinement::default().w_full().min_w_0(),
                            )
                        });
                        cx.flex(
                            fret_ui::element::FlexProps {
                                layout,
                                direction: fret_core::Axis::Horizontal,
                                gap,
                                padding: Edges::all(Px(0.0)),
                                justify: fret_ui::element::MainAlign::Start,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: true,
                            },
                            |_cx| children,
                        )
                    };

                    let row_end = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
                        let layout = cx.with_theme(|theme| {
                            decl_style::layout_style(
                                theme,
                                LayoutRefinement::default().w_full().min_w_0(),
                            )
                        });
                        cx.flex(
                            fret_ui::element::FlexProps {
                                layout,
                                direction: fret_core::Axis::Horizontal,
                                gap,
                                padding: Edges::all(Px(0.0)),
                                justify: fret_ui::element::MainAlign::End,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: false,
                            },
                            |_cx| children,
                        )
                    };

                    vec![
                        row(cx, vec![dropdown, context_menu, overlay_reset]),
                        row_end(cx, vec![context_menu_edge]),
                        row(cx, vec![tooltip, hover_card, popover, underlay, dialog]),
                        row(cx, vec![alert_dialog, sheet]),
                        portal_geometry,
                    ]
                },
            );

            vec![body]
        });

    let dialog_open_flag = {
        let open = cx
            .get_model_copied(&dialog_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(cx.text("Dialog open").test_id("ui-gallery-dialog-open"))
        } else {
            None
        }
    };

    let alert_dialog_open_flag = {
        let open = cx
            .get_model_copied(&alert_dialog_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(
                cx.text("AlertDialog open")
                    .test_id("ui-gallery-alert-dialog-open"),
            )
        } else {
            None
        }
    };

    let popover_dismissed_flag = {
        let last = cx
            .get_model_cloned(&last_action, Invalidation::Layout)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        if last.as_ref() == "popover:dismissed" {
            Some(
                cx.text("Popover dismissed")
                    .test_id("ui-gallery-popover-dismissed"),
            )
        } else {
            None
        }
    };

    let mut out: Vec<AnyElement> = vec![overlays, last_action_status];

    if let Some(flag) = popover_dismissed_flag {
        out.push(flag);
    }
    if let Some(flag) = dialog_open_flag {
        out.push(flag);
    }
    if let Some(flag) = alert_dialog_open_flag {
        out.push(flag);
    }

    out
}
