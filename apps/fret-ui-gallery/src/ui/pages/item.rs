use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_item(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let icon_media = |cx: &mut ElementContext<'_, App>, icon: &'static str| {
        shadcn::ItemMedia::new([shadcn::icon::icon(cx, fret_icons::IconId::new_static(icon))])
            .variant(shadcn::ItemMediaVariant::Icon)
            .into_element(cx)
    };

    let avatar_media = |cx: &mut ElementContext<'_, App>, initials: &'static str| {
        shadcn::ItemMedia::new([shadcn::Avatar::new([
            shadcn::AvatarFallback::new(initials).into_element(cx)
        ])
        .refine_layout(LayoutRefinement::default().w_px(Px(28.0)).h_px(Px(28.0)))
        .into_element(cx)])
        .into_element(cx)
    };

    let image_media = |cx: &mut ElementContext<'_, App>, label: &'static str| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(theme.color_token("muted")))
                    .rounded(Radius::Sm),
                LayoutRefinement::default().size_full(),
            )
        });
        shadcn::ItemMedia::new([
            cx.container(props, move |cx| vec![shadcn::typography::muted(cx, label)])
        ])
        .variant(shadcn::ItemMediaVariant::Image)
        .into_element(cx)
    };

    let item_row = |cx: &mut ElementContext<'_, App>,
                    title: &'static str,
                    description: &'static str,
                    media: AnyElement,
                    variant: shadcn::ItemVariant,
                    size: shadcn::ItemSize,
                    with_action: bool,
                    test_id: &'static str| {
        let mut children = vec![
            media,
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new(title).into_element(cx),
                shadcn::ItemDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
        ];

        if with_action {
            children.push(
                shadcn::ItemActions::new([shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx)])
                .into_element(cx),
            );
        }

        shadcn::Item::new(children)
            .variant(variant)
            .size(size)
            .on_click(CMD_APP_OPEN)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id(test_id)
    };

    let item_row_icon = |cx: &mut ElementContext<'_, App>,
                         title: &'static str,
                         description: &'static str,
                         icon: &'static str,
                         variant: shadcn::ItemVariant,
                         size: shadcn::ItemSize,
                         with_action: bool,
                         test_id: &'static str| {
        let media = icon_media(cx, icon);
        item_row(
            cx,
            title,
            description,
            media,
            variant,
            size,
            with_action,
            test_id,
        )
    };

    let item_row_avatar = |cx: &mut ElementContext<'_, App>,
                           title: &'static str,
                           description: &'static str,
                           initials: &'static str,
                           variant: shadcn::ItemVariant,
                           size: shadcn::ItemSize,
                           with_action: bool,
                           test_id: &'static str| {
        let media = avatar_media(cx, initials);
        item_row(
            cx,
            title,
            description,
            media,
            variant,
            size,
            with_action,
            test_id,
        )
    };

    let item_row_image = |cx: &mut ElementContext<'_, App>,
                          title: &'static str,
                          description: &'static str,
                          label: &'static str,
                          variant: shadcn::ItemVariant,
                          size: shadcn::ItemSize,
                          with_action: bool,
                          test_id: &'static str| {
        let media = image_media(cx, label);
        item_row(
            cx,
            title,
            description,
            media,
            variant,
            size,
            with_action,
            test_id,
        )
    };

    let demo = {
        let content = item_row_icon(
            cx,
            "Invoice.pdf",
            "Updated 2 days ago",
            "lucide.file-text",
            shadcn::ItemVariant::Default,
            shadcn::ItemSize::Default,
            true,
            "ui-gallery-item-demo",
        );
        content
    };

    let variant = {
        let content = shadcn::ItemGroup::new([
            item_row_icon(
                cx,
                "Default",
                "Neutral style with hover/press states.",
                "lucide.layout-dashboard",
                shadcn::ItemVariant::Default,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-variant-default",
            ),
            item_row_icon(
                cx,
                "Outline",
                "Visible border emphasis for dense lists.",
                "lucide.panel-top",
                shadcn::ItemVariant::Outline,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-variant-outline",
            ),
            item_row_icon(
                cx,
                "Muted",
                "Low-contrast background for secondary groups.",
                "lucide.inbox",
                shadcn::ItemVariant::Muted,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-variant-muted",
            ),
        ])
        .gap(Px(8.0))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(720.0)))
        .into_element(cx);
        content
    };

    let size = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(720.0))),
            |cx| {
                vec![
                    item_row_icon(
                        cx,
                        "Default Size",
                        "Use for regular settings and list rows.",
                        "lucide.settings",
                        shadcn::ItemVariant::Default,
                        shadcn::ItemSize::Default,
                        false,
                        "ui-gallery-item-size-default",
                    ),
                    item_row_icon(
                        cx,
                        "Small Size",
                        "Compact row density.",
                        "lucide.layers",
                        shadcn::ItemVariant::Default,
                        shadcn::ItemSize::Sm,
                        false,
                        "ui-gallery-item-size-sm",
                    ),
                    shadcn::typography::muted(
                        cx,
                        "Current Fret API supports default/sm; docs `xs` is not exposed yet.",
                    ),
                ]
            },
        );
        content
    };

    let icon = {
        let content = item_row_icon(
            cx,
            "Analytics",
            "Open dashboard metrics and trends.",
            "lucide.chart-column-big",
            shadcn::ItemVariant::Default,
            shadcn::ItemSize::Default,
            true,
            "ui-gallery-item-icon",
        );
        content
    };

    let avatar = {
        let content = item_row_avatar(
            cx,
            "Dana Chen",
            "Design review owner",
            "DC",
            shadcn::ItemVariant::Default,
            shadcn::ItemSize::Default,
            true,
            "ui-gallery-item-avatar",
        );
        content
    };

    let image = {
        let content = item_row_image(
            cx,
            "Cover Image",
            "Media-style item with image slot",
            "IMG",
            shadcn::ItemVariant::Default,
            shadcn::ItemSize::Default,
            true,
            "ui-gallery-item-image",
        );
        content
    };

    let group = {
        let content = shadcn::ItemGroup::new([
            item_row_icon(
                cx,
                "README.md",
                "Updated now",
                "lucide.file-text",
                shadcn::ItemVariant::Default,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-group-readme",
            ),
            shadcn::ItemSeparator::new().into_element(cx),
            item_row_icon(
                cx,
                "Roadmap.md",
                "Updated yesterday",
                "lucide.map",
                shadcn::ItemVariant::Default,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-group-roadmap",
            ),
            shadcn::ItemSeparator::new().into_element(cx),
            item_row_icon(
                cx,
                "Changelog.md",
                "Updated 3 days ago",
                "lucide.history",
                shadcn::ItemVariant::Default,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-group-changelog",
            ),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(720.0)))
        .into_element(cx)
        .test_id("ui-gallery-item-group");
        content
    };

    let header = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(720.0))),
            |cx| {
                vec![
                    shadcn::ItemHeader::new([
                        shadcn::ItemTitle::new("Recent Files").into_element(cx),
                        shadcn::Button::new("View all")
                            .variant(shadcn::ButtonVariant::Ghost)
                            .size(shadcn::ButtonSize::Sm)
                            .on_click(CMD_APP_OPEN)
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    item_row_icon(
                        cx,
                        "Draft proposal",
                        "Edited by Alex",
                        "lucide.file-pen-line",
                        shadcn::ItemVariant::Outline,
                        shadcn::ItemSize::Default,
                        false,
                        "ui-gallery-item-header-row",
                    ),
                ]
            },
        )
        .test_id("ui-gallery-item-header");
        content
    };

    let link = {
        let content = item_row_icon(
            cx,
            "Dashboard",
            "Overview of your account and activity.",
            "lucide.house",
            shadcn::ItemVariant::Outline,
            shadcn::ItemSize::Default,
            false,
            "ui-gallery-item-link",
        );
        content
    };

    let dropdown = {
        let dropdown_media = icon_media(cx, "lucide.folder");
        let content = shadcn::Item::new([
            dropdown_media,
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("Team Drive").into_element(cx),
                shadcn::ItemDescription::new("Shared files and permissions").into_element(cx),
            ])
            .into_element(cx),
            shadcn::ItemActions::new([shadcn::Button::new("Actions")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::Sm)
                .on_click(CMD_APP_OPEN)
                .into_element(cx)])
            .into_element(cx),
        ])
        .variant(shadcn::ItemVariant::Default)
        .on_click(CMD_APP_OPEN)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(720.0)))
        .into_element(cx)
        .test_id("ui-gallery-item-dropdown");
        content
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default(),
                LayoutRefinement::default().w_full().max_w(Px(720.0)),
            )
        });
        cx.container(props, |cx| {
            vec![item_row_icon(
                cx,
                "لوحة التحكم",
                "نظرة عامة على حسابك ونشاطك",
                "lucide.layout-dashboard",
                shadcn::ItemVariant::Default,
                shadcn::ItemSize::Default,
                true,
                "ui-gallery-item-rtl-row",
            )]
        })
    })
    .test_id("ui-gallery-item-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/item.rs`.",
            "Current API variants: default/outline/muted; sizes: default/sm.",
            "Docs `asChild` link and avatar-specific media variant are approximated with `on_click` and composed `Avatar`.",
            "Prefer stable `test_id`s on list rows and actions so diag scripts survive layout refactors.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Item docs order: Demo, Variant, Size, Icon, Avatar, Image, Group, Header, Link, Dropdown, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A file row with media, content, and a trailing action.")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"shadcn::Item::new([
    shadcn::ItemMedia::new([shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.file-text"))]).into_element(cx),
    shadcn::ItemContent::new([
        shadcn::ItemTitle::new("Invoice.pdf").into_element(cx),
        shadcn::ItemDescription::new("Updated 2 days ago").into_element(cx),
    ]).into_element(cx),
    shadcn::ItemActions::new([shadcn::Button::new("Open").into_element(cx)]).into_element(cx),
])
.on_click(CMD_APP_OPEN)
.into_element(cx);"#,
                ),
            DocSection::new("Variant", variant)
                .description("Default / Outline / Muted variants.")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"let base = shadcn::Item::new([
    shadcn::ItemContent::new([
        shadcn::ItemTitle::new("Default").into_element(cx),
        shadcn::ItemDescription::new("Neutral style").into_element(cx),
    ])
    .into_element(cx),
])
.on_click(CMD_APP_OPEN);

let outline = base.clone().variant(shadcn::ItemVariant::Outline).into_element(cx);
let muted = base.variant(shadcn::ItemVariant::Muted).into_element(cx);"#,
                ),
            DocSection::new("Size", size)
                .description("Row density presets (default + small).")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"let default_row = shadcn::Item::new([shadcn::ItemContent::new([
    shadcn::ItemTitle::new("Default Size").into_element(cx),
    shadcn::ItemDescription::new("Regular density").into_element(cx),
])
.into_element(cx)])
.size(shadcn::ItemSize::Default)
.into_element(cx);

let compact_row = shadcn::Item::new([shadcn::ItemContent::new([
    shadcn::ItemTitle::new("Small Size").into_element(cx),
    shadcn::ItemDescription::new("Compact density").into_element(cx),
])
.into_element(cx)])
.size(shadcn::ItemSize::Sm)
.into_element(cx);"#,
                ),
            DocSection::new("Icon", icon)
                .description("Icon media variant for app navigation rows.")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"let media = shadcn::ItemMedia::new([
    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.chart-column-big")),
])
.variant(shadcn::ItemMediaVariant::Icon)
.into_element(cx);

shadcn::Item::new([
    media,
    shadcn::ItemContent::new([
        shadcn::ItemTitle::new("Analytics").into_element(cx),
        shadcn::ItemDescription::new("Open dashboard metrics and trends.").into_element(cx),
    ])
    .into_element(cx),
])
.on_click(CMD_APP_OPEN)
.into_element(cx);"#,
                ),
            DocSection::new("Avatar", avatar)
                .description("Compose Avatar inside ItemMedia for people lists.")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"let avatar = shadcn::Avatar::new([shadcn::AvatarFallback::new("DC").into_element(cx)])
    .refine_layout(LayoutRefinement::default().w_px(Px(28.0)).h_px(Px(28.0)))
    .into_element(cx);

let media = shadcn::ItemMedia::new([avatar]).into_element(cx);

shadcn::Item::new([
    media,
    shadcn::ItemContent::new([
        shadcn::ItemTitle::new("Dana Chen").into_element(cx),
        shadcn::ItemDescription::new("Design review owner").into_element(cx),
    ])
    .into_element(cx),
])
.on_click(CMD_APP_OPEN)
.into_element(cx);"#,
                ),
            DocSection::new("Image", image)
                .description("Media slot can be styled as an image placeholder.")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"let props = cx.with_theme(|theme| {
    decl_style::container_props(
        theme,
        ChromeRefinement::default()
            .bg(ColorRef::Color(theme.color_token("muted")))
            .rounded(Radius::Sm),
        LayoutRefinement::default().size_full(),
    )
});
let placeholder = cx.container(props, |cx| vec![shadcn::typography::muted(cx, "IMG")]);

let media = shadcn::ItemMedia::new([placeholder])
    .variant(shadcn::ItemMediaVariant::Image)
    .into_element(cx);"#,
                ),
            DocSection::new("Group", group)
                .description("Group rows with separators and consistent spacing.")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"shadcn::ItemGroup::new([
    shadcn::Item::new([/* row */]).into_element(cx),
    shadcn::ItemSeparator::new().into_element(cx),
    shadcn::Item::new([/* row */]).into_element(cx),
])
.gap(Px(8.0))
.into_element(cx);"#,
                ),
            DocSection::new("Header", header)
                .description("Header row pairs a title with a trailing action.")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"shadcn::ItemHeader::new([
    shadcn::ItemTitle::new("Recent Files").into_element(cx),
    shadcn::Button::new("View all")
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::ButtonSize::Sm)
        .on_click(CMD_APP_OPEN)
        .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Link", link)
                .description("Clickable list row (approximates docs link usage).")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"shadcn::Item::new([shadcn::ItemContent::new([
    shadcn::ItemTitle::new("Dashboard").into_element(cx),
    shadcn::ItemDescription::new("Overview of your account and activity.").into_element(cx),
])
.into_element(cx)])
.variant(shadcn::ItemVariant::Outline)
.on_click(CMD_APP_OPEN)
.into_element(cx);"#,
                ),
            DocSection::new("Dropdown", dropdown)
                .description("Trailing ghost action button (menu placeholder).")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"shadcn::ItemActions::new([shadcn::Button::new("Actions")
    .variant(shadcn::ButtonVariant::Ghost)
    .size(shadcn::ButtonSize::Sm)
    .on_click(CMD_APP_OPEN)
    .into_element(cx)])
.into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Validate text alignment and action placement under RTL.")
                .max_w(Px(920.0))
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
     |cx| {
         shadcn::Item::new([shadcn::ItemContent::new([
            shadcn::ItemTitle::new("لوحة التحكم").into_element(cx),
            shadcn::ItemDescription::new("...").into_element(cx),
        ])
        .into_element(cx)])
        .into_element(cx)
     },
);"#,
                 ),
            DocSection::new("Notes", notes)
                .description("API reference pointers and implementation notes.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-item")]
}
