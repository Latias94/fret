use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

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

    let icon = doc_layout::icon;

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
        .test_id("ui-gallery-empty-demo");
        empty
    };

    let outline = {
        let muted_foreground = cx.with_theme(|theme| theme.color_token("muted-foreground"));
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
        .refine_style(ChromeRefinement::default().border_color(ColorRef::Color(muted_foreground)))
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
        .test_id("ui-gallery-empty-outline");
        empty
    };

    let background = {
        let muted = cx.with_theme(|theme| theme.color_token("muted"));
        let refresh_icon = icon(cx, "lucide.refresh-cw");
        let refresh_text = cx.text("Refresh");
        let refresh_button = shadcn::Button::new("Refresh")
            .variant(shadcn::ButtonVariant::Outline)
            .children([refresh_icon, refresh_text])
            .into_element(cx);

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
            shadcn::empty::EmptyContent::new([refresh_button]).into_element(cx),
        ])
        .refine_style(ChromeRefinement::default().bg(ColorRef::Color(muted)))
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
        .test_id("ui-gallery-empty-background");
        empty
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
        .test_id("ui-gallery-empty-avatar");

        empty
    };

    let avatar_group = {
        let invite_icon = icon(cx, "lucide.user-plus");
        let invite_text = cx.text("Invite Members");
        let invite_button = shadcn::Button::new("Invite Members")
            .size(shadcn::ButtonSize::Sm)
            .children([invite_icon, invite_text])
            .into_element(cx);

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
            shadcn::empty::EmptyContent::new([invite_button]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
        .test_id("ui-gallery-empty-avatar-group");

        empty
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
        .test_id("ui-gallery-empty-input-group");

        empty
    };

    let rtl = doc_layout::rtl(cx, |cx| {
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
    })
    .test_id("ui-gallery-empty-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "Empty page mirrors docs example sequence so parity audit can compare section-by-section.",
            "Outline/background recipes are currently style approximations because utility-level dashed/gradient tokens are not fully exposed here.",
            "Avatar and InputGroup scenarios keep state local to this page and expose stable test IDs for automation.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Empty docs order: Demo, Outline, Background, Avatar, Avatar Group, InputGroup, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A primary empty state with actions and a secondary link.")
                .code(
                    "rust",
                    r#"let empty = shadcn::Empty::new([header, content]).into_element(cx);"#,
                ),
            DocSection::new("Outline", outline)
                .description("Outlined empty state for low-emphasis surfaces.")
                .code(
                    "rust",
                    r#"let muted_fg = cx.with_theme(|theme| theme.color_token("muted-foreground"));

shadcn::Empty::new([header, content])
    .refine_style(ChromeRefinement::default().border_color(ColorRef::Color(muted_fg)))
    .into_element(cx);"#,
                ),
            DocSection::new("Background", background)
                .description("Muted background recipe for empty states embedded in cards.")
                .code(
                    "rust",
                    r#"let muted = cx.with_theme(|theme| theme.color_token("muted"));

shadcn::Empty::new([header, content])
    .refine_style(ChromeRefinement::default().bg(ColorRef::Color(muted)))
    .into_element(cx);"#,
                ),
            DocSection::new("Avatar", avatar)
                .description("Empty state media can be an avatar instead of an icon.")
                .code(
                    "rust",
                    r#"let avatar = shadcn::Avatar::new([shadcn::AvatarFallback::new("JD").into_element(cx)])
    .refine_layout(LayoutRefinement::default().w_px(Px(48.0)).h_px(Px(48.0)))
    .into_element(cx);

shadcn::Empty::new([
    shadcn::empty::EmptyHeader::new([
        shadcn::empty::EmptyMedia::new([avatar]).into_element(cx),
        shadcn::empty::EmptyTitle::new("User Offline").into_element(cx),
    ])
    .into_element(cx),
    shadcn::empty::EmptyContent::new([shadcn::Button::new("Leave Message").into_element(cx)])
        .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Avatar Group", avatar_group)
                .description("Media can also be a composed row of avatars.")
                .code(
                    "rust",
                    r#"let avatars = stack::hstack(
    cx,
    stack::HStackProps::default().gap(Space::N1).items_center(),
    |cx| {
        vec![
            shadcn::Avatar::new([shadcn::AvatarFallback::new("CN").into_element(cx)]).into_element(cx),
            shadcn::Avatar::new([shadcn::AvatarFallback::new("LR").into_element(cx)]).into_element(cx),
            shadcn::Avatar::new([shadcn::AvatarFallback::new("ER").into_element(cx)]).into_element(cx),
        ]
    },
);

shadcn::Empty::new([
    shadcn::empty::EmptyHeader::new([
        shadcn::empty::EmptyMedia::new([avatars]).into_element(cx),
        shadcn::empty::EmptyTitle::new("No Team Members").into_element(cx),
    ])
    .into_element(cx),
    shadcn::empty::EmptyContent::new([shadcn::Button::new("Invite Members").into_element(cx)])
        .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("InputGroup", input_group)
                .description("Empty states can include search inputs and trailing affordances.")
                .code(
                    "rust",
                    r#"shadcn::InputGroup::new(query)
    .leading([shadcn::InputGroupText::new("Search").into_element(cx)])
    .trailing([shadcn::InputGroupText::new("/").into_element(cx)]);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Empty layout should follow right-to-left direction context.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::Empty::new([header, content]).into_element(cx),
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines."),
        ],
    )
    .test_id("ui-gallery-empty-component");

    vec![body]
}
