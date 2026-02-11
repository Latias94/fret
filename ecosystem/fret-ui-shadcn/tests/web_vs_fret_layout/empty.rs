use super::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutEmptyRecipe {
    Demo,
    Background,
    Outline,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutEmptyCase {
    id: String,
    web_name: String,
    recipe: LayoutEmptyRecipe,
}

#[test]
fn web_vs_fret_layout_empty_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_empty_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutEmptyCase> =
        serde_json::from_str(raw).expect("layout empty fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout empty case={}", case.id);
        let web = read_web_golden(&case.web_name);
        let theme = web_theme(&web);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        );

        match case.recipe {
            LayoutEmptyRecipe::Demo => {
                let web_empty = web_find_by_class_tokens(
                    &theme.root,
                    &["border-dashed", "text-balance", "gap-6", "rounded-lg"],
                )
                .expect("web empty root");
                let web_header = web_find_by_class_tokens(
                    &theme.root,
                    &[
                        "max-w-sm",
                        "flex-col",
                        "items-center",
                        "gap-2",
                        "text-center",
                    ],
                )
                .expect("web empty header");

                let mut services = StyleAwareServices::default();
                let snap = run_fret_root_frames_with_services(bounds, &mut services, 2, |cx| {
                    use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

                    let icon = decl_icon::icon_with(
                        cx,
                        fret_icons::ids::ui::CHEVRON_DOWN,
                        Some(Px(24.0)),
                        None,
                    );
                    let media = EmptyMedia::new(vec![icon])
                        .variant(EmptyMediaVariant::Icon)
                        .into_element(cx);

                    let title = EmptyTitle::new("No Projects Yet").into_element(cx);
                    let desc = EmptyDescription::new(
                        "You haven't created any projects yet. Get started by creating your first project.",
                    )
                    .into_element(cx);
                    let header = EmptyHeader::new(vec![media, title, desc]).into_element(cx);
                    let header = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:empty-demo:header")),
                            ..Default::default()
                        },
                        move |_cx| vec![header],
                    );

                    let actions = cx.flex(
                        FlexProps {
                            layout: LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(8.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| {
                            vec![
                                Button::new("Create Project").into_element(cx),
                                Button::new("Import Project")
                                    .variant(ButtonVariant::Outline)
                                    .into_element(cx),
                            ]
                        },
                    );
                    let content = EmptyContent::new(vec![actions]).into_element(cx);

                    let learn_more = Button::new("Learn More")
                        .variant(ButtonVariant::Link)
                        .size(ButtonSize::Sm)
                        .into_element(cx);

                    let root = fret_ui_shadcn::Empty::new(vec![header, content, learn_more])
                        .into_element(cx);
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:empty-demo:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![root],
                    )]
                });

                let root =
                    find_semantics(&snap, SemanticsRole::Panel, Some("Golden:empty-demo:root"))
                        .expect("fret empty root");
                let header = find_semantics(
                    &snap,
                    SemanticsRole::Panel,
                    Some("Golden:empty-demo:header"),
                )
                .expect("fret empty header");

                assert_close_px(
                    "empty-demo root x",
                    root.bounds.origin.x,
                    web_empty.rect.x,
                    2.0,
                );
                assert_close_px(
                    "empty-demo root y",
                    root.bounds.origin.y,
                    web_empty.rect.y,
                    2.0,
                );
                assert_close_px(
                    "empty-demo root w",
                    root.bounds.size.width,
                    web_empty.rect.w,
                    2.0,
                );
                assert_close_px(
                    "empty-demo root h",
                    root.bounds.size.height,
                    web_empty.rect.h,
                    6.0,
                );
                assert_rect_close_px("empty-demo header", header.bounds, web_header.rect, 2.0);
            }
            LayoutEmptyRecipe::Background => {
                let web_empty = web_find_by_class_tokens(
                    &theme.root,
                    &["bg-gradient-to-b", "from-muted/50", "border-dashed"],
                )
                .expect("web empty root");

                let mut services = StyleAwareServices::default();
                let snap = run_fret_root_frames_with_services(bounds, &mut services, 2, |cx| {
                    let icon = decl_icon::icon_with(
                        cx,
                        fret_icons::ids::ui::CHEVRON_DOWN,
                        Some(Px(24.0)),
                        None,
                    );
                    let media = EmptyMedia::new(vec![icon])
                        .variant(EmptyMediaVariant::Icon)
                        .into_element(cx);

                    let title = EmptyTitle::new("No Notifications").into_element(cx);
                    let desc = EmptyDescription::new(
                        "You're all caught up. New notifications will appear here.",
                    )
                    .into_element(cx);
                    let header = EmptyHeader::new(vec![media, title, desc]).into_element(cx);

                    let button = fret_ui_shadcn::Button::new("Refresh")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .size(fret_ui_shadcn::ButtonSize::Sm)
                        .into_element(cx);
                    let content = EmptyContent::new(vec![button]).into_element(cx);

                    let root = fret_ui_shadcn::Empty::new(vec![header, content]).into_element(cx);
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:empty-background:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![root],
                    )]
                });

                let root = find_semantics(
                    &snap,
                    SemanticsRole::Panel,
                    Some("Golden:empty-background:root"),
                )
                .expect("fret empty root");

                assert_close_px(
                    "empty-background root x",
                    root.bounds.origin.x,
                    web_empty.rect.x,
                    2.0,
                );
                assert_close_px(
                    "empty-background root y",
                    root.bounds.origin.y,
                    web_empty.rect.y,
                    2.0,
                );
                assert_close_px(
                    "empty-background root w",
                    root.bounds.size.width,
                    web_empty.rect.w,
                    2.0,
                );
            }
            LayoutEmptyRecipe::Outline => {
                let web_empty = web_find_by_class_tokens(
                    &theme.root,
                    &["border-dashed", "border", "gap-6", "rounded-lg"],
                )
                .expect("web empty root");

                let mut services = StyleAwareServices::default();
                let snap = run_fret_root_frames_with_services(bounds, &mut services, 2, |cx| {
                    let icon = decl_icon::icon_with(
                        cx,
                        fret_icons::ids::ui::CHEVRON_DOWN,
                        Some(Px(24.0)),
                        None,
                    );
                    let media = EmptyMedia::new(vec![icon])
                        .variant(EmptyMediaVariant::Icon)
                        .into_element(cx);

                    let title = EmptyTitle::new("Cloud Storage Empty").into_element(cx);
                    let desc = EmptyDescription::new(
                        "Upload files to your cloud storage to access them anywhere.",
                    )
                    .into_element(cx);
                    let header = EmptyHeader::new(vec![media, title, desc]).into_element(cx);

                    let button = fret_ui_shadcn::Button::new("Upload Files")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .size(fret_ui_shadcn::ButtonSize::Sm)
                        .into_element(cx);
                    let content = EmptyContent::new(vec![button]).into_element(cx);

                    let root = fret_ui_shadcn::Empty::new(vec![header, content]).into_element(cx);
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:empty-outline:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![root],
                    )]
                });

                let root = find_semantics(
                    &snap,
                    SemanticsRole::Panel,
                    Some("Golden:empty-outline:root"),
                )
                .expect("fret empty root");

                assert_rect_close_px("empty-outline root", root.bounds, web_empty.rect, 2.0);
            }
        }
    }
}
#[test]
fn web_vs_fret_layout_empty_icon_geometry_matches_web() {
    let web = read_web_golden("empty-icon");
    let theme = web_theme(&web);

    let web_grid =
        web_find_by_class_tokens(&theme.root, &["grid", "gap-8"]).expect("web grid root");

    let mut cards = find_all(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "border-dashed")
            && class_has_token(n, "gap-6")
            && class_has_token(n, "rounded-lg")
    });
    cards.sort_by(|a, b| {
        a.rect
            .y
            .total_cmp(&b.rect.y)
            .then_with(|| a.rect.x.total_cmp(&b.rect.x))
    });
    let web_first = *cards.first().expect("web first empty card");
    let web_second = *cards.get(1).expect("web second empty card");
    let gap = web_second.rect.x - (web_first.rect.x + web_first.rect.w);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let theme = Theme::global(&*cx.app).clone();

        fn mk_card(
            cx: &mut fret_ui::ElementContext<'_, App>,
            label: &'static str,
            title: &'static str,
            desc: &'static str,
        ) -> AnyElement {
            let icon =
                decl_icon::icon_with(cx, fret_icons::ids::ui::CHEVRON_DOWN, Some(Px(24.0)), None);
            let media = EmptyMedia::new(vec![icon])
                .variant(EmptyMediaVariant::Icon)
                .into_element(cx);
            let title = EmptyTitle::new(title).into_element(cx);
            let desc = EmptyDescription::new(desc).into_element(cx);
            let header = EmptyHeader::new(vec![media, title, desc]).into_element(cx);
            let card = fret_ui_shadcn::Empty::new(vec![header]).into_element(cx);
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from(label)),
                    ..Default::default()
                },
                move |_cx| vec![card],
            )
        }

        let card_1 = mk_card(
            cx,
            "Golden:empty-icon:card-1",
            "No messages",
            "Your inbox is empty. New messages will appear here.",
        );
        let card_2 = mk_card(
            cx,
            "Golden:empty-icon:card-2",
            "No favorites",
            "Items you mark as favorites will appear here.",
        );
        let card_3 = mk_card(
            cx,
            "Golden:empty-icon:card-3",
            "No likes yet",
            "Content you like will be saved here for easy access.",
        );
        let card_4 = mk_card(
            cx,
            "Golden:empty-icon:card-4",
            "No bookmarks",
            "Save interesting content by bookmarking it.",
        );

        let root_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(web_grid.rect.w)))
                .min_w_0(),
        );

        vec![cx.container(
            ContainerProps {
                layout: root_layout,
                ..Default::default()
            },
            move |cx| {
                vec![cx.grid(
                    GridProps {
                        cols: 2,
                        gap: Px(gap),
                        layout: decl_style::layout_style(
                            &theme,
                            LayoutRefinement::default().w_full(),
                        ),
                        ..Default::default()
                    },
                    move |_cx| vec![card_1, card_2, card_3, card_4],
                )]
            },
        )]
    });

    let first = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:empty-icon:card-1"),
    )
    .expect("fret card 1");
    let second = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:empty-icon:card-2"),
    )
    .expect("fret card 2");

    assert_close_px(
        "empty-icon card-1 x",
        first.bounds.origin.x,
        web_first.rect.x,
        2.0,
    );
    assert_close_px(
        "empty-icon card-1 y",
        first.bounds.origin.y,
        web_first.rect.y,
        2.0,
    );
    assert_close_px(
        "empty-icon card-1 w",
        first.bounds.size.width,
        web_first.rect.w,
        2.0,
    );
    assert_close_px(
        "empty-icon card-2 x",
        second.bounds.origin.x,
        web_second.rect.x,
        2.0,
    );
    assert_close_px(
        "empty-icon card-2 y",
        second.bounds.origin.y,
        web_second.rect.y,
        2.0,
    );
    assert_close_px(
        "empty-icon card-2 w",
        second.bounds.size.width,
        web_second.rect.w,
        2.0,
    );
}
