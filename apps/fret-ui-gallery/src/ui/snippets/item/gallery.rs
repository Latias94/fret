// region: example
use crate::spec::CMD_APP_OPEN;
use fret_app::App;
use fret_ui::Theme;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    download_progress: Option<Model<f32>>,
}

fn icon(cx: &mut ElementContext<'_, App>, id: &'static str) -> AnyElement {
    shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
}

fn icon_button(
    cx: &mut ElementContext<'_, App>,
    icon_id: &'static str,
    variant: shadcn::ButtonVariant,
    test_id: Arc<str>,
) -> AnyElement {
    shadcn::Button::new("")
        .a11y_label(icon_id)
        .variant(variant)
        .size(shadcn::ButtonSize::Icon)
        .icon(fret_icons::IconId::new_static(icon_id))
        .into_element(cx)
        .test_id(test_id)
}

fn outline_button(cx: &mut ElementContext<'_, App>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .into_element(cx)
}

fn outline_button_sm(cx: &mut ElementContext<'_, App>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

fn item_basic(
    cx: &mut ElementContext<'_, App>,
    variant: shadcn::ItemVariant,
    title: &'static str,
    description: Option<&'static str>,
    actions: Vec<AnyElement>,
    test_id: &'static str,
) -> AnyElement {
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
}

fn item_icon(
    cx: &mut ElementContext<'_, App>,
    variant: shadcn::ItemVariant,
    icon_id: &'static str,
    title: &'static str,
    description: Option<&'static str>,
    actions: Vec<AnyElement>,
    test_id: &'static str,
) -> AnyElement {
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
}

fn item_avatar(
    cx: &mut ElementContext<'_, App>,
    username: &'static str,
    message: &'static str,
    initials: &'static str,
    test_id: Arc<str>,
    add_action_test_id: Arc<str>,
) -> AnyElement {
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
}

fn item_team(
    cx: &mut ElementContext<'_, App>,
    test_id: &'static str,
    action_test_id: &'static str,
) -> AnyElement {
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
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let download_progress = cx
        .with_state(Models::default, |st| st.download_progress.clone())
        .unwrap_or_else(|| {
            let model = cx.app.models_mut().insert(50.0);
            cx.with_state(Models::default, |st| {
                st.download_progress = Some(model.clone())
            });
            model
        });

    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(384.0)));
    let max_w_lg = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(520.0)));

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

    let gallery_demo = crate::ui::doc_layout::wrap_row_snapshot(
        cx,
        &theme,
        Space::N6,
        fret_ui::element::CrossAlign::Start,
        |_cx| vec![column_basic, column_people, column_music, column_issues],
    )
    .test_id("ui-gallery-item-gallery-demo");

    gallery_demo
}
// endregion: example
