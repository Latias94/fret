use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use fret_ui_kit::declarative::style as decl_style;

pub(super) fn preview_item(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ItemPageModels {
        download_progress: Option<Model<f32>>,
        dropdown_open: Option<Model<bool>>,
    }

    let download_progress =
        cx.with_state(ItemPageModels::default, |st| st.download_progress.clone());
    let download_progress = match download_progress {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(50.0);
            cx.with_state(ItemPageModels::default, |st| {
                st.download_progress = Some(model.clone())
            });
            model
        }
    };

    let dropdown_open = cx.with_state(ItemPageModels::default, |st| st.dropdown_open.clone());
    let dropdown_open = match dropdown_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ItemPageModels::default, |st| {
                st.dropdown_open = Some(model.clone())
            });
            model
        }
    };

    let theme = Theme::global(&*cx.app).snapshot();

    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(384.0)));
    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(448.0)));
    let max_w_lg = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(520.0)));
    let max_w_xl = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(576.0)));

    let icon = doc_layout::icon;

    let icon_button = |cx: &mut ElementContext<'_, App>,
                       icon_id: &'static str,
                       variant: shadcn::ButtonVariant,
                       test_id: Arc<str>| {
        shadcn::Button::new("")
            .a11y_label(icon_id)
            .variant(variant)
            .size(shadcn::ButtonSize::Icon)
            .icon(fret_icons::IconId::new_static(icon_id))
            .into_element(cx)
            .test_id(test_id)
    };

    let outline_button = |cx: &mut ElementContext<'_, App>, label: &'static str| {
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx)
    };

    let outline_button_sm = |cx: &mut ElementContext<'_, App>, label: &'static str| {
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx)
    };

    let item_basic = |cx: &mut ElementContext<'_, App>,
                      variant: shadcn::ItemVariant,
                      title: &'static str,
                      description: Option<&'static str>,
                      actions: Vec<AnyElement>,
                      test_id: &'static str| {
        let mut content_children = vec![shadcn::ItemTitle::new(title).into_element(cx)];
        if let Some(description) = description {
            content_children.push(shadcn::ItemDescription::new(description).into_element(cx));
        }

        let mut children = vec![shadcn::ItemContent::new(content_children).into_element(cx)];
        if !actions.is_empty() {
            children.push(shadcn::ItemActions::new(actions).into_element(cx));
        }

        shadcn::Item::new(children)
            .variant(variant)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id(test_id)
    };

    let item_icon = |cx: &mut ElementContext<'_, App>,
                     variant: shadcn::ItemVariant,
                     icon_id: &'static str,
                     title: &'static str,
                     description: Option<&'static str>,
                     actions: Vec<AnyElement>,
                     test_id: &'static str| {
        let media = shadcn::ItemMedia::new([icon(cx, icon_id)])
            .variant(shadcn::ItemMediaVariant::Icon)
            .into_element(cx);

        let mut content_children = vec![shadcn::ItemTitle::new(title).into_element(cx)];
        if let Some(description) = description {
            content_children.push(shadcn::ItemDescription::new(description).into_element(cx));
        }

        let mut children = vec![
            media,
            shadcn::ItemContent::new(content_children).into_element(cx),
        ];
        if !actions.is_empty() {
            children.push(shadcn::ItemActions::new(actions).into_element(cx));
        }

        shadcn::Item::new(children)
            .variant(variant)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id(test_id)
    };

    let item_avatar = |cx: &mut ElementContext<'_, App>,
                       username: &'static str,
                       message: &'static str,
                       initials: &'static str,
                       test_id: Arc<str>,
                       add_action_test_id: Arc<str>| {
        let avatar = shadcn::Avatar::new([shadcn::AvatarFallback::new(initials).into_element(cx)])
            .into_element(cx);
        let media = shadcn::ItemMedia::new([avatar]).into_element(cx);
        let content = shadcn::ItemContent::new([
            shadcn::ItemTitle::new(username).into_element(cx),
            shadcn::ItemDescription::new(message).into_element(cx),
        ])
        .into_element(cx);

        let add = icon_button(
            cx,
            "lucide.plus",
            shadcn::ButtonVariant::Outline,
            add_action_test_id,
        );
        let actions = shadcn::ItemActions::new([add]).into_element(cx);

        shadcn::Item::new([media, content, actions])
            .on_click(CMD_APP_OPEN)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id(test_id)
    };

    let item_team = |cx: &mut ElementContext<'_, App>,
                     test_id: &'static str,
                     action_test_id: &'static str| {
        let avatars = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N1).items_center(),
            |cx| {
                vec![
                    shadcn::Avatar::new([shadcn::AvatarFallback::new("CN").into_element(cx)])
                        .into_element(cx),
                    shadcn::Avatar::new([shadcn::AvatarFallback::new("LR").into_element(cx)])
                        .into_element(cx),
                    shadcn::Avatar::new([shadcn::AvatarFallback::new("ER").into_element(cx)])
                        .into_element(cx),
                ]
            },
        );
        let media = shadcn::ItemMedia::new([avatars]).into_element(cx);
        let content = shadcn::ItemContent::new([
            shadcn::ItemTitle::new("Design Department").into_element(cx),
            shadcn::ItemDescription::new("Meet our team of designers, engineers, and researchers.")
                .into_element(cx),
        ])
        .into_element(cx);

        let chevron = icon_button(
            cx,
            "lucide.chevron-right",
            shadcn::ButtonVariant::Outline,
            Arc::<str>::from(action_test_id),
        );
        let actions = shadcn::ItemActions::new([chevron])
            .refine_layout(LayoutRefinement::default().mt(Space::N1))
            .into_element(cx);

        shadcn::Item::new([media, content, actions])
            .on_click(CMD_APP_OPEN)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id(test_id)
    };

    let item_download = {
        let header =
            shadcn::ItemHeader::new([ui::text(cx, "Your download has started.").into_element(cx)])
                .into_element(cx);
        let media = shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)])
            .variant(shadcn::ItemMediaVariant::Icon)
            .into_element(cx);
        let content = shadcn::ItemContent::new([
            shadcn::ItemTitle::new("Downloading...").into_element(cx),
            shadcn::ItemDescription::new("129 MB / 1000 MB").into_element(cx),
        ])
        .into_element(cx);
        let actions = shadcn::ItemActions::new([outline_button_sm(cx, "Cancel")]).into_element(cx);
        let footer = shadcn::ItemFooter::new([shadcn::Progress::new(download_progress)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)])
        .into_element(cx);

        shadcn::Item::new([header, media, content, actions, footer])
            .variant(shadcn::ItemVariant::Outline)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-download")
    };

    let column_basic = {
        let button_outline = outline_button(cx, "Button");
        let item_title_button = item_basic(
            cx,
            shadcn::ItemVariant::Default,
            "Item Title",
            None,
            vec![button_outline],
            "ui-gallery-item-basic-default",
        );

        let button_outline = outline_button(cx, "Button");
        let item_title_button_outline = item_basic(
            cx,
            shadcn::ItemVariant::Outline,
            "Item Title",
            None,
            vec![button_outline],
            "ui-gallery-item-basic-outline",
        );

        let button_outline = outline_button(cx, "Button");
        let item_desc_button = item_basic(
            cx,
            shadcn::ItemVariant::Default,
            "Item Title",
            Some("Item Description"),
            vec![button_outline],
            "ui-gallery-item-basic-default-desc",
        );
        let item_desc_outline = item_basic(
            cx,
            shadcn::ItemVariant::Outline,
            "Item Title",
            Some("Item Description"),
            Vec::new(),
            "ui-gallery-item-basic-outline-desc",
        );
        let item_desc_muted = item_basic(
            cx,
            shadcn::ItemVariant::Muted,
            "Item Title",
            Some("Item Description"),
            Vec::new(),
            "ui-gallery-item-basic-muted-desc",
        );
        let button_a = outline_button(cx, "Button");
        let button_b = outline_button(cx, "Button");
        let item_desc_muted_actions = item_basic(
            cx,
            shadcn::ItemVariant::Muted,
            "Item Title",
            Some("Item Description"),
            vec![button_a, button_b],
            "ui-gallery-item-basic-muted-actions",
        );

        let purchase = shadcn::Button::new("Purchase")
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx);
        let item_ticket = item_icon(
            cx,
            shadcn::ItemVariant::Outline,
            "lucide.ticket",
            "Item Title",
            None,
            vec![purchase],
            "ui-gallery-item-basic-ticket-outline",
        );

        let upgrade = shadcn::Button::new("Upgrade")
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx);
        let item_ticket_muted = item_icon(
            cx,
            shadcn::ItemVariant::Muted,
            "lucide.ticket",
            "Item Title",
            Some("Item Description"),
            vec![upgrade],
            "ui-gallery-item-basic-ticket-muted",
        );

        let field = {
            let field = shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldTitle::new("Field Title").into_element(cx),
                    shadcn::FieldDescription::new("Field Description").into_element(cx),
                ])
                .into_element(cx),
                shadcn::Button::new("Button")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx);

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                |cx| {
                    vec![
                        shadcn::FieldLabel::new("Field Label").into_element(cx),
                        field,
                    ]
                },
            )
            .test_id("ui-gallery-item-field")
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_sm.clone()),
            |_cx| {
                vec![
                    item_title_button,
                    item_title_button_outline,
                    item_desc_button,
                    item_desc_outline,
                    item_desc_muted,
                    item_desc_muted_actions,
                    item_ticket,
                    item_ticket_muted,
                    field,
                ]
            },
        )
        .test_id("ui-gallery-item-column-basic")
    };

    let column_people = {
        let people = [
            ("shadcn", "Just shipped a component that fixes itself", "S"),
            (
                "pranathip",
                "My code is so clean, it does its own laundry",
                "P",
            ),
            (
                "evilrabbit",
                "Debugging is like being a detective in a crime movie where you're also the murderer",
                "E",
            ),
            (
                "maxleiter",
                "I don't always test my code, but when I do, I test it in production",
                "M",
            ),
        ];

        let mut group_children: Vec<AnyElement> = Vec::new();
        for (idx, (username, message, initials)) in people.iter().copied().enumerate() {
            group_children.push(item_avatar(
                cx,
                username,
                message,
                initials,
                Arc::<str>::from(format!("ui-gallery-item-people-{idx}")),
                Arc::<str>::from(format!("ui-gallery-item-people-{idx}-action-add")),
            ));
            if idx + 1 < people.len() {
                group_children.push(shadcn::ItemSeparator::new().into_element(cx));
            }
        }

        let group = shadcn::ItemGroup::new(group_children)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-people-group");

        let team = item_team(cx, "ui-gallery-item-team", "ui-gallery-item-team-action");
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_sm.clone()),
            |_cx| vec![group, team, item_download],
        )
        .test_id("ui-gallery-item-column-people")
    };

    let column_music = {
        let gap_4 = MetricRef::space(Space::N4).resolve(&theme);
        let music = [
            (
                "Midnight City Lights",
                "Neon Dreams",
                "Electric Nights",
                "3:45",
            ),
            (
                "Coffee Shop Conversations",
                "The Morning Brew",
                "Urban Stories",
                "4:05",
            ),
            ("Digital Rain", "Cyber Symphony", "Binary Beats", "3:30"),
            (
                "Sunset Boulevard",
                "Golden Hour",
                "California Dreams",
                "3:55",
            ),
            ("Neon Sign Romance", "Retro Wave", "80s Forever", "4:10"),
            ("Ocean Depths", "Deep Blue", "Underwater Symphony", "3:40"),
            (
                "Space Station Alpha",
                "Cosmic Explorers",
                "Galactic Journey",
                "3:50",
            ),
            (
                "Forest Whispers",
                "Nature's Choir",
                "Woodland Tales",
                "3:35",
            ),
        ];

        let mut rows: Vec<AnyElement> = Vec::new();
        for (idx, (title, artist, album, duration)) in music.iter().copied().enumerate() {
            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(theme.color_token("muted")))
                    .rounded(Radius::Sm),
                LayoutRefinement::default().size_full(),
            );
            let image = cx
                .container(props, move |cx| vec![shadcn::typography::muted(cx, "IMG")])
                .test_id(format!("ui-gallery-item-music-image-{idx}"));
            let media = shadcn::ItemMedia::new([image])
                .variant(shadcn::ItemMediaVariant::Image)
                .into_element(cx);

            let title_text: Arc<str> = Arc::from(format!("{title} - {album}"));
            let content = shadcn::ItemContent::new([
                shadcn::ItemTitle::new(title_text).into_element(cx),
                shadcn::ItemDescription::new(artist).into_element(cx),
            ])
            .into_element(cx);

            let duration =
                shadcn::ItemContent::new([shadcn::ItemDescription::new(duration).into_element(cx)])
                    .refine_layout(LayoutRefinement::default().flex_none())
                    .into_element(cx);

            let download = icon_button(
                cx,
                "lucide.download",
                shadcn::ButtonVariant::Ghost,
                Arc::<str>::from(format!("ui-gallery-item-music-{idx}-download")),
            );
            let actions = shadcn::ItemActions::new([download]).into_element(cx);

            rows.push(
                shadcn::Item::new([media, content, duration, actions])
                    .variant(shadcn::ItemVariant::Outline)
                    .on_click(CMD_APP_OPEN)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
                    .test_id(format!("ui-gallery-item-music-{idx}")),
            );
        }

        let group = shadcn::ItemGroup::new(rows)
            .gap(gap_4)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-music-group");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_lg.clone()),
            |_cx| vec![group],
        )
        .test_id("ui-gallery-item-column-music")
    };

    let column_issues = {
        let issues = [
            (
                1247,
                "Button component doesn't respect disabled state when using custom variants",
                "When applying custom variants to the Button component, the disabled prop is ignored and the button remains clickable.",
            ),
            (
                892,
                "Dialog component causes scroll lock on mobile devices",
                "The Dialog component prevents scrolling on the background content but doesn't restore scroll position properly.",
            ),
            (
                1156,
                "TypeScript errors with Select component in strict mode",
                "Using the Select component with TypeScript strict mode enabled throws type errors related to value typing.",
            ),
            (
                734,
                "Dark mode toggle causes flash of unstyled content",
                "When switching between light and dark themes, there's a brief moment where components render with incorrect styling.",
            ),
            (
                1389,
                "Form validation messages overlap with floating labels",
                "Error messages in Form components with floating labels appear underneath the label text, making them difficult to read.",
            ),
        ];

        let mut children: Vec<AnyElement> = Vec::new();
        for (idx, (number, title, description)) in issues.iter().copied().enumerate() {
            let content = shadcn::ItemContent::new([
                shadcn::ItemTitle::new(title).into_element(cx),
                shadcn::ItemDescription::new(description).into_element(cx),
            ])
            .into_element(cx);

            let number_text: Arc<str> = Arc::from(format!("#{number}"));
            let number_col = shadcn::ItemContent::new([ui::text(cx, number_text).into_element(cx)])
                .refine_layout(LayoutRefinement::default().flex_none())
                .into_element(cx);

            children.push(
                shadcn::Item::new([content, number_col])
                    .on_click(CMD_APP_OPEN)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
                    .test_id(format!("ui-gallery-item-issue-{idx}")),
            );
            if idx + 1 < issues.len() {
                children.push(shadcn::ItemSeparator::new().into_element(cx));
            }
        }

        let group = shadcn::ItemGroup::new(children)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-issues-group");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_lg.clone()),
            |_cx| vec![group],
        )
        .test_id("ui-gallery-item-column-issues")
    };

    let docs_demo = {
        let outline = shadcn::ItemVariant::Outline;

        let item_basic = {
            let action = shadcn::Button::new("Action")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx);
            shadcn::Item::new([
                shadcn::ItemContent::new([
                    shadcn::ItemTitle::new("Basic Item").into_element(cx),
                    shadcn::ItemDescription::new("A simple item with title and description.")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::ItemActions::new([action]).into_element(cx),
            ])
            .variant(outline)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-docs-demo-basic")
        };

        let item_sm_link = {
            let media = shadcn::ItemMedia::new([icon(cx, "lucide.badge-check")])
                .into_element(cx)
                .test_id("ui-gallery-item-docs-demo-sm-link-media");
            let content = shadcn::ItemContent::new([shadcn::ItemTitle::new(
                "Your profile has been verified.",
            )
            .into_element(cx)])
            .into_element(cx)
            .test_id("ui-gallery-item-docs-demo-sm-link-content");
            let actions = shadcn::ItemActions::new([icon(cx, "lucide.chevron-right")])
                .into_element(cx)
                .test_id("ui-gallery-item-docs-demo-sm-link-actions");

            shadcn::Item::new([media, content, actions])
                .variant(outline)
                .size(shadcn::ItemSize::Sm)
                .render(shadcn::ItemRender::Link {
                    href: Arc::<str>::from("https://example.com/profile"),
                    target: None,
                    rel: None,
                })
                // Keep the gallery deterministic: demonstrate link semantics + styling without opening
                // the browser during scripted runs.
                .on_click(CMD_APP_OPEN)
                .a11y_label("Verified profile")
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
                .test_id("ui-gallery-item-docs-demo-sm-link")
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_md.clone()),
            |_cx| vec![item_basic, item_sm_link],
        )
        .test_id("ui-gallery-item-demo")
    };

    let docs_variants = {
        let button = |cx: &mut ElementContext<'_, App>| {
            shadcn::Button::new("Open")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx)
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_md.clone()),
            |cx| {
                vec![
                    shadcn::Item::new([
                        shadcn::ItemContent::new([
                            shadcn::ItemTitle::new("Default Variant").into_element(cx),
                            shadcn::ItemDescription::new(
                                "Standard styling with subtle background and borders.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::ItemActions::new([button(cx)]).into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
                    .test_id("ui-gallery-item-variant-default"),
                    shadcn::Item::new([
                        shadcn::ItemContent::new([
                            shadcn::ItemTitle::new("Outline Variant").into_element(cx),
                            shadcn::ItemDescription::new(
                                "Outlined style with clear borders and transparent background.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::ItemActions::new([button(cx)]).into_element(cx),
                    ])
                    .variant(shadcn::ItemVariant::Outline)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
                    .test_id("ui-gallery-item-variant-outline"),
                    shadcn::Item::new([
                        shadcn::ItemContent::new([
                            shadcn::ItemTitle::new("Muted Variant").into_element(cx),
                            shadcn::ItemDescription::new(
                                "Subdued appearance with muted colors for secondary content.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::ItemActions::new([button(cx)]).into_element(cx),
                    ])
                    .variant(shadcn::ItemVariant::Muted)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
                    .test_id("ui-gallery-item-variant-muted"),
                ]
            },
        )
        .test_id("ui-gallery-item-variants")
    };

    let docs_size = {
        let outline = shadcn::ItemVariant::Outline;

        let item_default = {
            let action = shadcn::Button::new("Action")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx);
            shadcn::Item::new([
                shadcn::ItemContent::new([
                    shadcn::ItemTitle::new("Basic Item").into_element(cx),
                    shadcn::ItemDescription::new("A simple item with title and description.")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::ItemActions::new([action]).into_element(cx),
            ])
            .variant(outline)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-size-default")
        };

        let item_sm = {
            let media = shadcn::ItemMedia::new([icon(cx, "lucide.badge-check")])
                .into_element(cx)
                .test_id("ui-gallery-item-size-sm-media");
            let content = shadcn::ItemContent::new([shadcn::ItemTitle::new(
                "Your profile has been verified.",
            )
            .into_element(cx)])
            .into_element(cx)
            .test_id("ui-gallery-item-size-sm-content");
            let actions = shadcn::ItemActions::new([icon(cx, "lucide.chevron-right")])
                .into_element(cx)
                .test_id("ui-gallery-item-size-sm-actions");

            shadcn::Item::new([media, content, actions])
                .variant(outline)
                .size(shadcn::ItemSize::Sm)
                .render(shadcn::ItemRender::Link {
                    href: Arc::<str>::from("https://example.com/profile"),
                    target: None,
                    rel: None,
                })
                .on_click(CMD_APP_OPEN)
                .a11y_label("Verified profile")
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
                .test_id("ui-gallery-item-size-sm")
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_md.clone()),
            |_cx| vec![item_default, item_sm],
        )
        .test_id("ui-gallery-item-size")
    };

    let docs_icon = {
        let review = shadcn::Button::new("Review")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx);
        let item = item_icon(
            cx,
            shadcn::ItemVariant::Outline,
            "lucide.shield-alert",
            "Security Alert",
            Some("New login detected from unknown device."),
            vec![review],
            "ui-gallery-item-icon",
        );

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_lg.clone()),
            |_cx| vec![item],
        )
        .test_id("ui-gallery-item-icon-wrapper")
    };

    let docs_avatar = {
        let avatar = shadcn::Avatar::new([shadcn::AvatarFallback::new("ER").into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_px(Px(40.0)).h_px(Px(40.0)))
            .into_element(cx);
        let media = shadcn::ItemMedia::new([avatar]).into_element(cx);
        let content = shadcn::ItemContent::new([
            shadcn::ItemTitle::new("Evil Rabbit").into_element(cx),
            shadcn::ItemDescription::new("Last seen 5 months ago").into_element(cx),
        ])
        .into_element(cx);

        let invite = shadcn::Button::new("")
            .a11y_label("Invite")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::IconSm)
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .icon(fret_icons::IconId::new_static("lucide.plus"))
            .into_element(cx)
            .test_id("ui-gallery-item-avatar-invite");
        let actions = shadcn::ItemActions::new([invite]).into_element(cx);

        let item_one = shadcn::Item::new([media, content, actions])
            .variant(shadcn::ItemVariant::Outline)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-avatar-one");

        let item_team = item_team(
            cx,
            "ui-gallery-item-avatar-team",
            "ui-gallery-item-avatar-team-action",
        );
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_lg.clone()),
            |_cx| vec![item_one, item_team],
        )
        .test_id("ui-gallery-item-avatar")
    };

    let docs_image = {
        let music = [
            (
                "Midnight City Lights",
                "Neon Dreams",
                "Electric Nights",
                "3:45",
            ),
            (
                "Coffee Shop Conversations",
                "The Morning Brew",
                "Urban Stories",
                "4:05",
            ),
            ("Digital Rain", "Cyber Symphony", "Binary Beats", "3:30"),
        ];

        let mut rows: Vec<AnyElement> = Vec::new();
        for (idx, (title, artist, album, duration)) in music.iter().copied().enumerate() {
            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(theme.color_token("muted")))
                    .rounded(Radius::Sm),
                LayoutRefinement::default().size_full(),
            );
            let image = cx
                .container(props, move |cx| vec![shadcn::typography::muted(cx, "IMG")])
                .test_id(format!("ui-gallery-item-image-image-{idx}"));
            let media = shadcn::ItemMedia::new([image])
                .variant(shadcn::ItemMediaVariant::Image)
                .into_element(cx);

            let title_text: Arc<str> = Arc::from(format!("{title} - {album}"));
            let content = shadcn::ItemContent::new([
                shadcn::ItemTitle::new(title_text).into_element(cx),
                shadcn::ItemDescription::new(artist).into_element(cx),
            ])
            .into_element(cx);

            let duration =
                shadcn::ItemContent::new([shadcn::ItemDescription::new(duration).into_element(cx)])
                    .refine_layout(LayoutRefinement::default().flex_none())
                    .into_element(cx);

            rows.push(
                shadcn::Item::new([media, content, duration])
                    .variant(shadcn::ItemVariant::Outline)
                    .render(shadcn::ItemRender::Link {
                        href: Arc::<str>::from("https://example.com/music"),
                        target: None,
                        rel: None,
                    })
                    .on_click(CMD_APP_OPEN)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
                    .test_id(format!("ui-gallery-item-image-{idx}")),
            );
        }

        let group = shadcn::ItemGroup::new(rows)
            .gap(MetricRef::space(Space::N4).resolve(&theme))
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-image-group");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_md.clone()),
            |_cx| vec![group],
        )
        .test_id("ui-gallery-item-image")
    };

    let docs_group = {
        let people = [
            ("shadcn", "shadcn@vercel.com", "S"),
            ("maxleiter", "maxleiter@vercel.com", "M"),
            ("evilrabbit", "evilrabbit@vercel.com", "E"),
        ];

        let mut children: Vec<AnyElement> = Vec::new();
        for (idx, (username, email, initials)) in people.iter().copied().enumerate() {
            let avatar =
                shadcn::Avatar::new([shadcn::AvatarFallback::new(initials).into_element(cx)])
                    .into_element(cx);
            let media = shadcn::ItemMedia::new([avatar]).into_element(cx);
            let content = shadcn::ItemContent::new([
                shadcn::ItemTitle::new(username).into_element(cx),
                shadcn::ItemDescription::new(email).into_element(cx),
            ])
            .into_element(cx);

            let add = shadcn::Button::new("")
                .a11y_label("Add")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::IconSm)
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .icon(fret_icons::IconId::new_static("lucide.plus"))
                .into_element(cx)
                .test_id(format!("ui-gallery-item-group-add-{idx}"));
            let actions = shadcn::ItemActions::new([add]).into_element(cx);

            children.push(
                shadcn::Item::new([media, content, actions])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
                    .test_id(format!("ui-gallery-item-group-item-{idx}")),
            );
            if idx + 1 < people.len() {
                children.push(shadcn::ItemSeparator::new().into_element(cx));
            }
        }

        let group = shadcn::ItemGroup::new(children)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-group");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_md.clone()),
            |_cx| vec![group],
        )
        .test_id("ui-gallery-item-group-wrapper")
    };

    let docs_header = {
        let models = [
            ("v0-1.5-sm", "Everyday tasks and UI generation."),
            ("v0-1.5-lg", "Advanced thinking or reasoning."),
            ("v0-2.0-mini", "Open Source model for everyone."),
        ];

        let mut children: Vec<AnyElement> = Vec::new();
        for (idx, (name, description)) in models.iter().copied().enumerate() {
            let header = {
                let props = decl_style::container_props(
                    &theme,
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(theme.color_token("muted")))
                        .rounded(Radius::Sm),
                    LayoutRefinement::default()
                        .w_full()
                        .aspect_ratio(1.0)
                        .overflow_hidden(),
                );
                let image = cx
                    .container(props, move |cx| vec![shadcn::typography::muted(cx, "IMG")])
                    .test_id(format!("ui-gallery-item-header-image-{idx}"));
                shadcn::ItemHeader::new([image]).into_element(cx)
            };

            let content = shadcn::ItemContent::new([
                shadcn::ItemTitle::new(name).into_element(cx),
                shadcn::ItemDescription::new(description).into_element(cx),
            ])
            .into_element(cx);

            children.push(
                shadcn::Item::new([header, content])
                    .variant(shadcn::ItemVariant::Outline)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
                    .test_id(format!("ui-gallery-item-header-{idx}")),
            );
        }

        let group = shadcn::ItemGroup::new(children)
            .grid(3)
            .gap(MetricRef::space(Space::N4).resolve(&theme))
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-item-header-group");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(max_w_xl.clone()),
            |_cx| vec![group],
        )
        .test_id("ui-gallery-item-header")
    };

    let docs_link = {
        let row_a = shadcn::Item::new([
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("Visit our documentation").into_element(cx),
                shadcn::ItemDescription::new("Learn how to get started with our components.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::ItemActions::new([icon(cx, "lucide.chevron-right")]).into_element(cx),
        ])
        .render(shadcn::ItemRender::Link {
            href: Arc::<str>::from("https://example.com/docs"),
            target: None,
            rel: None,
        })
        .on_click(CMD_APP_OPEN)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-item-link-row-a");

        let row_b = shadcn::Item::new([
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("External resource").into_element(cx),
                shadcn::ItemDescription::new("Opens in a new tab with security attributes.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::ItemActions::new([icon(cx, "lucide.external-link")]).into_element(cx),
        ])
        .variant(shadcn::ItemVariant::Outline)
        .render(shadcn::ItemRender::Link {
            href: Arc::<str>::from("https://example.com/external"),
            target: Some(Arc::<str>::from("_blank")),
            rel: Some(Arc::<str>::from("noopener noreferrer")),
        })
        .on_click(CMD_APP_OPEN)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-item-link-row-b");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(max_w_md.clone()),
            |_cx| vec![row_a, row_b],
        )
        .test_id("ui-gallery-item-link")
    };

    let docs_dropdown = {
        let people = [
            ("shadcn", "S", "shadcn@vercel.com"),
            ("maxleiter", "M", "maxleiter@vercel.com"),
            ("evilrabbit", "E", "evilrabbit@vercel.com"),
        ];

        let menu = shadcn::DropdownMenu::new(dropdown_open.clone())
            .align(shadcn::DropdownMenuAlign::End)
            .min_width(Px(288.0))
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Select")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .toggle_model(dropdown_open.clone())
                        .children([
                            ui::text(cx, "Select").into_element(cx),
                            icon(cx, "lucide.chevron-down"),
                        ])
                        .test_id("ui-gallery-item-dropdown-trigger")
                        .into_element(cx)
                },
                |cx| {
                    people
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(idx, (username, initials, email))| {
                            let avatar =
                                shadcn::Avatar::new([
                                    shadcn::AvatarFallback::new(initials).into_element(cx)
                                ])
                                .refine_layout(
                                    LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)),
                                )
                                .into_element(cx);
                            let media = shadcn::ItemMedia::new([avatar]).into_element(cx);
                            let content = shadcn::ItemContent::new([
                                shadcn::ItemTitle::new(username).into_element(cx),
                                shadcn::ItemDescription::new(email).into_element(cx),
                            ])
                            .gap(Px(6.0))
                            .into_element(cx);

                            let item = shadcn::Item::new([media, content])
                                .size(shadcn::ItemSize::Sm)
                                .variant(shadcn::ItemVariant::Outline)
                                .refine_style(
                                    ChromeRefinement::default().px(Space::N2).py(Space::N2),
                                )
                                .refine_layout(LayoutRefinement::default().w_full())
                                .into_element(cx)
                                .test_id(format!("ui-gallery-item-dropdown-item-{idx}"));

                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new(username)
                                    .padding(Edges::all(Px(0.0)))
                                    .content(item),
                            )
                        })
                        .collect::<Vec<_>>()
                },
            );

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_center()
                .layout(
                    LayoutRefinement::default()
                        .w_full()
                        .min_w_0()
                        .min_h(Px(256.0)),
                ),
            |_cx| vec![menu],
        )
        .test_id("ui-gallery-item-dropdown")
    };

    let gallery_demo = doc_layout::wrap_row_snapshot(
        cx,
        &theme,
        Space::N6,
        fret_ui::element::CrossAlign::Start,
        |_cx| vec![column_basic, column_people, column_music, column_issues],
    )
    .test_id("ui-gallery-item-gallery-demo");

    let link_render = {
        let media = shadcn::ItemMedia::new([icon(cx, "lucide.house")])
            .variant(shadcn::ItemMediaVariant::Icon)
            .into_element(cx);
        let content = shadcn::ItemContent::new([
            shadcn::ItemTitle::new("Dashboard").into_element(cx),
            shadcn::ItemDescription::new("Overview of your account and activity.").into_element(cx),
        ])
        .into_element(cx);
        let actions = shadcn::ItemActions::new([icon(cx, "lucide.chevron-right")]).into_element(cx);

        shadcn::Item::new([media, content, actions])
            .render(shadcn::ItemRender::Link {
                href: Arc::<str>::from("https://example.com/dashboard"),
                target: None,
                rel: None,
            })
            // Keep the gallery deterministic: demonstrate link semantics + styling without opening
            // the browser during scripted runs.
            .on_click(CMD_APP_OPEN)
            .a11y_label("Dashboard")
            .test_id("ui-gallery-item-link-render")
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        let action = outline_button_sm(cx, "فتح");
        item_basic(
            cx,
            shadcn::ItemVariant::Outline,
            "لوحة التحكم",
            Some("نظرة عامة على حسابك ونشاطك."),
            vec![action],
            "ui-gallery-item-rtl",
        )
    })
    .test_id("ui-gallery-item-rtl-wrapper");

    let notes = doc_layout::notes(
        cx,
        [
            "Docs sections align to shadcn Item examples (new-york-v4).",
            "The Gallery section is an extended snapshot used for internal regression coverage.",
            "Upstream uses `render={<a .../>}`; Fret uses `ItemRender::Link` to express link semantics on the pressable root.",
            "API reference: `ecosystem/fret-ui-shadcn/src/item.rs`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Item docs (new-york-v4) with a few Fret-specific extras.",
        ),
        vec![
            DocSection::new("Demo", docs_demo)
                .no_shell()
                .max_w(Px(720.0))
                .code(
                    "rust",
                    r#"let item = shadcn::Item::new([
    shadcn::ItemContent::new([
        shadcn::ItemTitle::new("Basic Item").into_element(cx),
        shadcn::ItemDescription::new("A simple item with title and description.").into_element(cx),
    ]).into_element(cx),
    shadcn::ItemActions::new([
        shadcn::Button::new("Action")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx),
    ]).into_element(cx),
])
.variant(shadcn::ItemVariant::Outline)
.refine_layout(LayoutRefinement::default().w_full())
.into_element(cx);"#,
                ),
            DocSection::new("Variants", docs_variants)
                .description("Default, Outline, and Muted variants (new-york-v4).")
                .no_shell()
                .max_w(Px(640.0))
                .code("rust", r#"shadcn::Item::new([/* ... */]).variant(shadcn::ItemVariant::Muted).into_element(cx);"#),
            DocSection::new("Size", docs_size)
                .description("Default vs `sm` item sizing (new-york-v4).")
                .no_shell()
                .max_w(Px(640.0))
                .code("rust", r#"shadcn::Item::new([/* ... */]).size(shadcn::ItemSize::Sm).into_element(cx);"#),
            DocSection::new("Icon", docs_icon)
                .no_shell()
                .max_w(Px(640.0))
                .code("rust", r#"shadcn::ItemMedia::new([doc_layout::icon(cx, "lucide.shield-alert")]).variant(shadcn::ItemMediaVariant::Icon).into_element(cx);"#),
            DocSection::new("Avatar", docs_avatar)
                .no_shell()
                .max_w(Px(720.0))
                .code(
                    "rust",
                    r#"let avatar = shadcn::Avatar::new([
    shadcn::AvatarFallback::new("ER").into_element(cx),
])
.into_element(cx);

shadcn::Item::new([
    shadcn::ItemMedia::new([avatar]).into_element(cx),
    shadcn::ItemContent::new([
        shadcn::ItemTitle::new("Evil Rabbit").into_element(cx),
        shadcn::ItemDescription::new("Last seen 5 months ago").into_element(cx),
    ]).into_element(cx),
    shadcn::ItemActions::new([
        shadcn::Button::new("")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::IconSm)
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .icon(fret_icons::IconId::new_static("lucide.plus"))
            .a11y_label("Invite")
            .into_element(cx),
    ]).into_element(cx),
])
.variant(shadcn::ItemVariant::Outline)
.refine_layout(LayoutRefinement::default().w_full())
.into_element(cx);"#,
                ),
            DocSection::new("Image", docs_image)
                .no_shell()
                .max_w(Px(640.0))
                .code(
                    "rust",
                    r#"let theme = Theme::global(&*cx.app).snapshot();
let props = decl_style::container_props(
    &theme,
    ChromeRefinement::default()
        .bg(ColorRef::Color(theme.color_token("muted")))
        .rounded(Radius::Sm),
    LayoutRefinement::default().size_full(),
);
let image = cx
    .container(props, |cx| vec![shadcn::typography::muted(cx, "IMG")])
    .into_element(cx);

let media = shadcn::ItemMedia::new([image])
    .variant(shadcn::ItemMediaVariant::Image)
    .into_element(cx);"#,
                ),
            DocSection::new("Group", docs_group).no_shell().max_w(Px(640.0)),
            DocSection::new("Header", docs_header).no_shell().max_w(Px(820.0)),
            DocSection::new("Link", docs_link)
                .description("Links are modeled via `ItemRender::Link` so the root carries link semantics.")
                .no_shell()
                .max_w(Px(640.0))
                .code(
                    "rust",
                    r#"shadcn::Item::new([/* ... */])
    .render(shadcn::ItemRender::Link {
        href: Arc::<str>::from("https://example.com/docs"),
        target: None,
        rel: None,
    })
    .on_click(CMD_APP_OPEN)
    .into_element(cx);"#,
                ),
            DocSection::new("Dropdown", docs_dropdown)
                .description("Item composed inside a DropdownMenu row (new-york-v4).")
                .no_shell()
                .max_w(Px(720.0)),
            DocSection::new("Gallery", gallery_demo)
                .description("Extended coverage snapshot: columns + mixed compositions.")
                .no_shell()
                .max_w(Px(1100.0)),
            DocSection::new("Link (render)", link_render)
                .description("Minimal link row with media + chevron (gallery-friendly, deterministic).")
                .no_shell()
                .max_w(Px(640.0)),
            DocSection::new("Extras", rtl)
                .description("RTL smoke check (not present in upstream demo).")
                .no_shell()
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Item::new([/* ... */]).variant(shadcn::ItemVariant::Outline).into_element(cx)
});"#,
                ),
            DocSection::new("Notes", notes).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-item")]
}
