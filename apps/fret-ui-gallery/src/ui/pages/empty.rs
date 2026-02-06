use super::super::*;

pub(super) fn preview_empty(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct EmptyPageModels {
        search_query: Option<Model<String>>,
        rtl_search_query: Option<Model<String>>,
    }

    let (search_query, rtl_search_query) = cx.with_state(EmptyPageModels::default, |st| {
        (st.search_query.clone(), st.rtl_search_query.clone())
    });

    let (search_query, rtl_search_query) = match (search_query, rtl_search_query) {
        (Some(search_query), Some(rtl_search_query)) => (search_query, rtl_search_query),
        _ => {
            let search_query = cx.app.models_mut().insert(String::new());
            let rtl_search_query = cx.app.models_mut().insert(String::new());
            cx.with_state(EmptyPageModels::default, |st| {
                st.search_query = Some(search_query.clone());
                st.rtl_search_query = Some(rtl_search_query.clone());
            });
            (search_query, rtl_search_query)
        }
    };

    let theme = Theme::global(&*cx.app).clone();

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
        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(820.0)),
            ),
            move |_cx| [body],
        )
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let icon = |cx: &mut ElementContext<'_, App>, id: &'static str| {
        shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
    };

    let demo = {
        let empty = shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyMedia::new([icon(cx, "lucide.folder-search")])
                    .variant(shadcn::empty::EmptyMediaVariant::Icon)
                    .into_element(cx),
                shadcn::empty::EmptyTitle::new("No Projects Yet").into_element(cx),
                shadcn::empty::EmptyDescription::new(
                    "You have not created any projects yet. Start by creating your first project.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([
                shadcn::Button::new("Create Project").into_element(cx),
                shadcn::Button::new("Import Project")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::Button::new("Learn more")
                .variant(shadcn::ButtonVariant::Link)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-empty-demo"));
        section_card(cx, "Demo", empty)
    };

    let outline = {
        let empty = shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyMedia::new([icon(cx, "lucide.cloud")])
                    .variant(shadcn::empty::EmptyMediaVariant::Icon)
                    .into_element(cx),
                shadcn::empty::EmptyTitle::new("Cloud Storage Empty").into_element(cx),
                shadcn::empty::EmptyDescription::new(
                    "Upload files to cloud storage to access them from any device.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([shadcn::Button::new("Upload Files")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx)])
            .into_element(cx),
        ])
        .refine_style(
            ChromeRefinement::default()
                .border_color(ColorRef::Color(theme.color_required("muted-foreground"))),
        )
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-empty-outline"));
        section_card(cx, "Outline", empty)
    };

    let background = {
        let empty = shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyMedia::new([icon(cx, "lucide.bell")])
                    .variant(shadcn::empty::EmptyMediaVariant::Icon)
                    .into_element(cx),
                shadcn::empty::EmptyTitle::new("No Notifications").into_element(cx),
                shadcn::empty::EmptyDescription::new(
                    "You're all caught up. New notifications will appear here.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([shadcn::Button::new("Refresh")
                .variant(shadcn::ButtonVariant::Outline)
                .leading(icon(cx, "lucide.refresh-cw"))
                .into_element(cx)])
            .into_element(cx),
        ])
        .refine_style(
            ChromeRefinement::default().bg(ColorRef::Color(theme.color_required("muted"))),
        )
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-empty-background"));
        section_card(cx, "Background", empty)
    };

    let avatar = {
        let avatar_media =
            shadcn::Avatar::new([shadcn::AvatarFallback::new("JD").into_element(cx)])
                .refine_layout(LayoutRefinement::default().w_px(Px(48.0)).h_px(Px(48.0)))
                .into_element(cx);

        let empty = shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyMedia::new([avatar_media]).into_element(cx),
                shadcn::empty::EmptyTitle::new("User Offline").into_element(cx),
                shadcn::empty::EmptyDescription::new(
                    "This user is currently offline. Leave a message and notify later.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([shadcn::Button::new("Leave Message")
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx)])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-empty-avatar"));

        section_card(cx, "Avatar", empty)
    };

    let avatar_group = {
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

        let empty = shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyMedia::new([avatars]).into_element(cx),
                shadcn::empty::EmptyTitle::new("No Team Members").into_element(cx),
                shadcn::empty::EmptyDescription::new(
                    "Invite collaborators to start working on this project together.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([shadcn::Button::new("Invite Members")
                .size(shadcn::ButtonSize::Sm)
                .leading(icon(cx, "lucide.user-plus"))
                .into_element(cx)])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-empty-avatar-group"));

        section_card(cx, "Avatar Group", empty)
    };

    let input_group = {
        let search = shadcn::InputGroup::new(search_query.clone())
            .a11y_label("Search pages")
            .leading([shadcn::InputGroupText::new("Search").into_element(cx)])
            .trailing([shadcn::InputGroupText::new("/").into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
            .test_id("ui-gallery-empty-input-group-search")
            .into_element(cx);

        let empty = shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyTitle::new("404 - Not Found").into_element(cx),
                shadcn::empty::EmptyDescription::new(
                    "The page you are looking for doesn't exist. Try searching below.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([
                search,
                shadcn::empty::EmptyDescription::new("Need help? Contact support.")
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-empty-input-group"));

        section_card(cx, "InputGroup", empty)
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Empty::new([
                    shadcn::empty::EmptyHeader::new([
                        shadcn::empty::EmptyMedia::new([icon(cx, "lucide.folder-search")])
                            .variant(shadcn::empty::EmptyMediaVariant::Icon)
                            .into_element(cx),
                        shadcn::empty::EmptyTitle::new("RTL Empty State").into_element(cx),
                        shadcn::empty::EmptyDescription::new(
                            "This empty state uses RTL direction context for layout and alignment.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::empty::EmptyContent::new([
                        shadcn::InputGroup::new(rtl_search_query.clone())
                            .a11y_label("RTL search")
                            .leading([shadcn::InputGroupText::new("亘丨孬").into_element(cx)])
                            .trailing([shadcn::InputGroupText::new("/").into_element(cx)])
                            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
                            .test_id("ui-gallery-empty-rtl-input-group")
                            .into_element(cx),
                        shadcn::Button::new("Create Project").into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
                .into_element(cx)
            },
        )
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-empty-rtl"));

        section_card(cx, "RTL", rtl_content)
    };

    let component_panel = shell(
        cx,
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    shadcn::typography::muted(
                        cx,
                        "Preview follows shadcn Empty docs order: Demo, Outline, Background, Avatar, Avatar Group, InputGroup, RTL.",
                    ),
                    demo,
                    outline,
                    background,
                    avatar,
                    avatar_group,
                    input_group,
                    rtl,
                ]
            },
        ),
    )
    .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-empty-component"));

    let code_panel = shell(
        cx,
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Basic Empty").into_element(cx)])
                            .into_element(cx),
                        shadcn::CardContent::new(vec![
                            ui::text_block(
                                cx,
                                r#"shadcn::Empty::new([
    shadcn::empty::EmptyHeader::new([
        shadcn::empty::EmptyMedia::new([icon]).variant(shadcn::empty::EmptyMediaVariant::Icon).into_element(cx),
        shadcn::empty::EmptyTitle::new("No data").into_element(cx),
        shadcn::empty::EmptyDescription::new("No data found").into_element(cx),
    ]).into_element(cx),
    shadcn::empty::EmptyContent::new([
        shadcn::Button::new("Create").into_element(cx),
    ]).into_element(cx),
]).into_element(cx);"#,
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new("Avatar Group + InputGroup").into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardContent::new(vec![
                            ui::text_block(
                                cx,
                                r#"let avatars = stack::hstack(cx, stack::HStackProps::default().gap(Space::N1), |cx| {
    vec![avatar_a, avatar_b, avatar_c]
});

let search = shadcn::InputGroup::new(query)
    .leading([shadcn::InputGroupText::new("Search").into_element(cx)])
    .trailing([shadcn::InputGroupText::new("/").into_element(cx)])
    .into_element(cx);"#,
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ]
            },
        ),
    );

    let notes_panel = shell(
        cx,
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    shadcn::typography::h4(cx, "Notes"),
                    shadcn::typography::muted(
                        cx,
                        "Empty page mirrors docs example sequence so parity audit can compare section-by-section.",
                    ),
                    shadcn::typography::muted(
                        cx,
                        "Outline/background recipes are currently style approximations because utility-level dashed/gradient tokens are not fully exposed here.",
                    ),
                    shadcn::typography::muted(
                        cx,
                        "Avatar and InputGroup scenarios keep state local to this page and expose stable test IDs for automation.",
                    ),
                ]
            },
        ),
    );

    super::render_component_page_tabs(
        cx,
        "ui-gallery-empty",
        component_panel,
        code_panel,
        notes_panel,
    )
}
