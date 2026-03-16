pub const SOURCE: &str = include_str!("file_tree.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Point, Transform2D};
use fret_ui::Theme;
use fret_ui::element::VisualTransformProps;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Clone, Copy)]
enum TreeItem {
    File {
        key: &'static str,
        label: &'static str,
    },
    Folder {
        key: &'static str,
        label: &'static str,
        children: &'static [TreeItem],
    },
}

const COMPONENTS_UI_ITEMS: &[TreeItem] = &[
    TreeItem::File {
        key: "components-ui-button",
        label: "button.tsx",
    },
    TreeItem::File {
        key: "components-ui-card",
        label: "card.tsx",
    },
    TreeItem::File {
        key: "components-ui-dialog",
        label: "dialog.tsx",
    },
    TreeItem::File {
        key: "components-ui-input",
        label: "input.tsx",
    },
    TreeItem::File {
        key: "components-ui-select",
        label: "select.tsx",
    },
    TreeItem::File {
        key: "components-ui-table",
        label: "table.tsx",
    },
];

const COMPONENTS_ITEMS: &[TreeItem] = &[
    TreeItem::Folder {
        key: "components-ui",
        label: "ui",
        children: COMPONENTS_UI_ITEMS,
    },
    TreeItem::File {
        key: "components-login-form",
        label: "login-form.tsx",
    },
    TreeItem::File {
        key: "components-register-form",
        label: "register-form.tsx",
    },
];

const LIB_ITEMS: &[TreeItem] = &[
    TreeItem::File {
        key: "lib-utils",
        label: "utils.ts",
    },
    TreeItem::File {
        key: "lib-cn",
        label: "cn.ts",
    },
    TreeItem::File {
        key: "lib-api",
        label: "api.ts",
    },
];

const HOOKS_ITEMS: &[TreeItem] = &[
    TreeItem::File {
        key: "hooks-use-media-query",
        label: "use-media-query.ts",
    },
    TreeItem::File {
        key: "hooks-use-debounce",
        label: "use-debounce.ts",
    },
    TreeItem::File {
        key: "hooks-use-local-storage",
        label: "use-local-storage.ts",
    },
];

const TYPES_ITEMS: &[TreeItem] = &[
    TreeItem::File {
        key: "types-index",
        label: "index.d.ts",
    },
    TreeItem::File {
        key: "types-api",
        label: "api.d.ts",
    },
];

const PUBLIC_ITEMS: &[TreeItem] = &[
    TreeItem::File {
        key: "public-favicon",
        label: "favicon.ico",
    },
    TreeItem::File {
        key: "public-logo",
        label: "logo.svg",
    },
    TreeItem::File {
        key: "public-images",
        label: "images",
    },
];

const FILE_TREE_ITEMS: &[TreeItem] = &[
    TreeItem::Folder {
        key: "components",
        label: "components",
        children: COMPONENTS_ITEMS,
    },
    TreeItem::Folder {
        key: "lib",
        label: "lib",
        children: LIB_ITEMS,
    },
    TreeItem::Folder {
        key: "hooks",
        label: "hooks",
        children: HOOKS_ITEMS,
    },
    TreeItem::Folder {
        key: "types",
        label: "types",
        children: TYPES_ITEMS,
    },
    TreeItem::Folder {
        key: "public",
        label: "public",
        children: PUBLIC_ITEMS,
    },
    TreeItem::File {
        key: "app",
        label: "app.tsx",
    },
    TreeItem::File {
        key: "layout",
        label: "layout.tsx",
    },
    TreeItem::File {
        key: "globals",
        label: "globals.css",
    },
    TreeItem::File {
        key: "package-json",
        label: "package.json",
    },
    TreeItem::File {
        key: "tsconfig",
        label: "tsconfig.json",
    },
    TreeItem::File {
        key: "readme",
        label: "README.md",
    },
    TreeItem::File {
        key: "gitignore",
        label: ".gitignore",
    },
];

fn rotated_lucide<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &'static str,
    rotation_deg: f32,
) -> impl IntoUiElement<H> + use<H> {
    let size = Px(16.0);
    let center = Point::new(Px(8.0), Px(8.0));
    let transform = Transform2D::rotation_about_degrees(rotation_deg, center);

    cx.visual_transform_props(
        VisualTransformProps {
            layout: {
                let theme = Theme::global(&*cx.app);
                decl_style::layout_style(
                    theme,
                    LayoutRefinement::default()
                        .w_px(size)
                        .h_px(size)
                        .flex_shrink_0(),
                )
            },
            transform,
        },
        move |cx| {
            vec![icon::icon_with(
                cx,
                fret_icons::IconId::new_static(id),
                Some(size),
                None,
            )]
        },
    )
}

fn file_leaf<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    key: &'static str,
    label: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let theme = Theme::global(&*cx.app).snapshot();
    let foreground = ColorRef::Color(theme.color_token("foreground"));
    let row = ui::h_flex(|cx| {
        vec![
            icon::icon_with(
                cx,
                fret_icons::IconId::new_static("lucide.file"),
                Some(Px(16.0)),
                None,
            ),
            cx.text(label),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N2)
    .justify_start()
    .items_center()
    .into_element(cx);

    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Link)
        .size(shadcn::ButtonSize::Sm)
        .content_justify_start()
        .style(
            shadcn::raw::button::ButtonStyle::default()
                .foreground(fret_ui_kit::WidgetStateProperty::new(Some(foreground))),
        )
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .children([row])
        .into_element(cx)
        .test_id(format!("ui-gallery-collapsible-tree-leaf-{key}"))
}

fn folder<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    key: &'static str,
    label: &'static str,
    children: &'static [TreeItem],
) -> impl IntoUiElement<H> + use<H> {
    shadcn::Collapsible::uncontrolled(false)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element_with_open_model(
            cx,
            |cx, open, is_open| {
                let chevron =
                    rotated_lucide(cx, "lucide.chevron-right", if is_open { 90.0 } else { 0.0 })
                        .into_element(cx);
                let icon = fret_icons::IconId::new_static(if is_open {
                    "lucide.folder-open"
                } else {
                    "lucide.folder"
                });

                let row = ui::h_flex(|cx| {
                    vec![
                        chevron,
                        icon::icon_with(cx, icon, Some(Px(16.0)), None),
                        cx.text(label),
                    ]
                })
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2)
                .justify_start()
                .items_center()
                .into_element(cx);

                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .content_justify_start()
                    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                    .children([row])
                    .toggle_model(open)
                    .test_id(format!("ui-gallery-collapsible-tree-trigger-{key}"))
                    .into_element(cx)
            },
            |cx| {
                shadcn::CollapsibleContent::new(vec![
                    ui::v_flex(|cx| render_items(cx, children))
                        .gap(Space::N1)
                        .items_stretch()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().min_w_0().ml(Space::N5))
                .into_element(cx)
                .test_id(format!("ui-gallery-collapsible-tree-content-{key}"))
            },
        )
}

fn render_item<H: UiHost>(cx: &mut ElementContext<'_, H>, item: &TreeItem) -> AnyElement {
    match item {
        TreeItem::File { key, label } => file_leaf(cx, key, label).into_element(cx),
        TreeItem::Folder {
            key,
            label,
            children,
        } => folder(cx, key, label, children).into_element(cx),
    }
}

fn render_items<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: &'static [TreeItem],
) -> Vec<AnyElement> {
    items.iter().map(|item| render_item(cx, item)).collect()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let tabs = shadcn::TabsRoot::uncontrolled(Some("explorer"))
        .list(
            shadcn::TabsList::new()
                .trigger(shadcn::TabsTrigger::new("explorer", "Explorer"))
                .trigger(shadcn::TabsTrigger::new("outline", "Outline")),
        )
        .gap_px(Px(0.0))
        .content_fill_remaining(false)
        .list_full_width(true)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    let tree = ui::v_flex(|cx| render_items(cx, FILE_TREE_ITEMS))
        .gap(Space::N1)
        .items_stretch()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| ui::children![cx; tabs]),
            shadcn::card_content(|cx| ui::children![cx; tree]),
        ]
    })
    .size(shadcn::CardSize::Sm)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(256.0)))
    .into_element(cx)
    .test_id("ui-gallery-collapsible-file-tree")
}
// endregion: example
