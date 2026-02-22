use super::super::super::super::*;

#[cfg(any())]
use std::sync::Arc;

pub(in crate::ui) fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_button(cx)
}

#[cfg(any())]
pub(in crate::ui) fn preview_button_legacy(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let outline_fg = ColorRef::Color(theme.color_token("foreground"));
    let secondary_fg = ColorRef::Color(theme.color_token("secondary-foreground"));
    let muted_fg = ColorRef::Color(theme.color_token("muted-foreground"));

    let icon = |cx: &mut ElementContext<'_, App>, name: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(cx, fret_icons::IconId::new_static(name), None, Some(fg))
    };

    let content_text = |cx: &mut ElementContext<'_, App>, text: &'static str, fg: ColorRef| {
        ui::text(cx, text)
            .font_medium()
            .nowrap()
            .text_color(fg)
            .into_element(cx)
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let size = {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N3).items_start(),
            |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("Small")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx),
                                shadcn::Button::new("Submit")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::IconSm)
                                    .children([icon(
                                        cx,
                                        "lucide.arrow-up-right",
                                        outline_fg.clone(),
                                    )])
                                    .into_element(cx),
                            ]
                        },
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("Default")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .into_element(cx),
                                shadcn::Button::new("Submit")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Icon)
                                    .children([icon(
                                        cx,
                                        "lucide.arrow-up-right",
                                        outline_fg.clone(),
                                    )])
                                    .into_element(cx),
                            ]
                        },
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("Large")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Lg)
                                    .into_element(cx),
                                shadcn::Button::new("Submit")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::IconLg)
                                    .children([icon(
                                        cx,
                                        "lucide.arrow-up-right",
                                        outline_fg.clone(),
                                    )])
                                    .into_element(cx),
                            ]
                        },
                    ),
                ]
            },
        );
        section(cx, "Size", body)
    };

    let default_body = shadcn::Button::new("Button").into_element(cx);
    let default = section(cx, "Default", default_body);

    let flex_1 = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full().max_w(Px(520.0))),
            |cx| {
                vec![
                    shadcn::Button::new("Flex 1")
                        .variant(shadcn::ButtonVariant::Outline)
                        .refine_layout(LayoutRefinement::default().flex_1())
                        .test_id("ui-gallery-button-flex1-left")
                        .into_element(cx),
                    shadcn::Button::new("Flex 1")
                        .variant(shadcn::ButtonVariant::Outline)
                        .refine_layout(LayoutRefinement::default().flex_1())
                        .test_id("ui-gallery-button-flex1-right")
                        .into_element(cx),
                ]
            },
        );
        section(cx, "Flex 1 (chrome fill)", body)
    };

    let flex_1_centered_fixed_chrome = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full().max_w(Px(520.0))),
            |cx| {
                let centered = |cx: &mut ElementContext<'_, App>, id: &'static str| {
                    fret_ui_kit::declarative::chrome::centered_fixed_chrome_pressable_with_id_props(
                        cx,
                        move |_cx, _st, _id| {
                            let mut pressable = fret_ui::element::PressableProps::default();
                            pressable.enabled = true;
                            pressable.focusable = true;
                            pressable.a11y.test_id = Some(Arc::<str>::from(id));
                            pressable.layout.flex.grow = 1.0;
                            pressable.layout.flex.shrink = 1.0;
                            pressable.layout.flex.basis = fret_ui::element::Length::Px(Px(0.0));

                            let mut chrome = fret_ui::element::ContainerProps::default();
                            chrome.layout.size.width = fret_ui::element::Length::Px(Px(28.0));
                            chrome.layout.size.height = fret_ui::element::Length::Px(Px(28.0));

                            (pressable, chrome, |_cx| Vec::<AnyElement>::new())
                        },
                    )
                };

                vec![
                    centered(cx, "ui-gallery-centered-fixed-chrome-flex1-left"),
                    centered(cx, "ui-gallery-centered-fixed-chrome-flex1-right"),
                ]
            },
        );
        section(cx, "Flex 1 (centered fixed chrome)", body)
    };

    let outline_body = shadcn::Button::new("Outline")
        .variant(shadcn::ButtonVariant::Outline)
        .into_element(cx);
    let outline = section(cx, "Outline", outline_body);

    let secondary_body = shadcn::Button::new("Secondary")
        .variant(shadcn::ButtonVariant::Secondary)
        .into_element(cx);
    let secondary = section(cx, "Secondary", secondary_body);

    let ghost_body = shadcn::Button::new("Ghost")
        .variant(shadcn::ButtonVariant::Ghost)
        .into_element(cx);
    let ghost = section(cx, "Ghost", ghost_body);

    let destructive_body = shadcn::Button::new("Destructive")
        .variant(shadcn::ButtonVariant::Destructive)
        .into_element(cx);
    let destructive = section(cx, "Destructive", destructive_body);

    let link_body = shadcn::Button::new("Link")
        .variant(shadcn::ButtonVariant::Link)
        .into_element(cx);
    let link = section(cx, "Link", link_body);

    let icon_only_body = shadcn::Button::new("Submit")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .children([icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
        .into_element(cx);
    let icon_only = section(cx, "Icon", icon_only_body);

    let with_icon = {
        let body = shadcn::Button::new("New Branch")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .children([
                icon(cx, "lucide.git-branch", outline_fg.clone())
                    .test_id("ui-gallery-button-with-icon-icon"),
                content_text(cx, "New Branch", outline_fg.clone())
                    .test_id("ui-gallery-button-with-icon-label"),
            ])
            .into_element(cx)
            .test_id("ui-gallery-button-with-icon");
        section(cx, "With Icon", body)
    };

    let rounded_body = shadcn::Button::new("Scroll to top")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .children([icon(cx, "lucide.arrow-up", outline_fg.clone())])
        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
        .into_element(cx);
    let rounded = section(cx, "Rounded", rounded_body);

    let spinner = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Generating")
                        .variant(shadcn::ButtonVariant::Outline)
                        .disabled(true)
                        .children([
                            shadcn::Spinner::new()
                                .color(outline_fg.clone())
                                .into_element(cx),
                            content_text(cx, "Generating", outline_fg.clone()),
                        ])
                        .into_element(cx),
                    shadcn::Button::new("Downloading")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .disabled(true)
                        .children([
                            content_text(cx, "Downloading", secondary_fg.clone()),
                            shadcn::Spinner::new()
                                .color(secondary_fg.clone())
                                .into_element(cx),
                        ])
                        .into_element(cx),
                ]
            },
        );
        section(cx, "Spinner", body)
    };

    let button_group = {
        let demo = preview_button_group(cx)
            .into_iter()
            .next()
            .unwrap_or_else(|| cx.text("ButtonGroup demo is missing"));
        section(cx, "Button Group", demo)
    };

    let render_link = {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            |cx| {
                vec![
                    shadcn::Button::new("Documentation")
                        .variant(shadcn::ButtonVariant::Outline)
                        .render(shadcn::ButtonRender::Link {
                            href: Arc::<str>::from("https://example.com/docs"),
                            target: None,
                            rel: None,
                        })
                        .on_click(CMD_APP_OPEN)
                        .into_element(cx),
                    ui::text(cx, "`ButtonRender::Link` models shadcn `asChild` anchors by applying link semantics to the pressable root.")
                        .text_color(muted_fg.clone())
                        .into_element(cx),
                ]
            },
        );
        section(cx, "Link (render)", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Button::new("التالي")
                                .variant(shadcn::ButtonVariant::Outline)
                                .into_element(cx),
                            shadcn::Button::new("السابق")
                                .variant(shadcn::ButtonVariant::Outline)
                                .into_element(cx),
                        ]
                    },
                )
            },
        );
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N4).items_start(),
        |_cx| {
            vec![
                size,
                default,
                flex_1,
                flex_1_centered_fixed_chrome,
                outline,
                secondary,
                ghost,
                destructive,
                link,
                icon_only,
                with_icon,
                rounded,
                spinner,
                button_group,
                render_link,
                rtl,
            ]
        },
    )]
}
