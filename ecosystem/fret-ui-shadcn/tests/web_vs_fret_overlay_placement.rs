use fret_app::App;
use fret_core::{
    AppWindowId, Edges, Event, FontId, FontWeight, FrameId, KeyCode, Modifiers, MouseButton, Point,
    PointerEvent, PointerType, Px, Rect, SemanticsRole, Size as CoreSize, TextOverflow, TextStyle,
    TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, GridProps, LayoutStyle, Length, MainAlign,
    MarginEdge, TextProps,
};
use fret_ui::elements::{GlobalElementId, bounds_for_element};
use fret_ui::tree::UiTree;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::OverlayController;
use serde::Deserialize;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
struct WebGolden {
    themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebGoldenTheme {
    #[allow(dead_code)]
    root: WebNode,
    #[serde(default)]
    portals: Vec<WebNode>,
    #[serde(rename = "portalWrappers", default)]
    portal_wrappers: Vec<WebNode>,
    #[serde(default)]
    viewport: Option<WebViewport>,
    #[serde(default)]
    open: Option<WebOpenMeta>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebViewport {
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebPoint {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebOpenMeta {
    #[allow(dead_code)]
    action: String,
    #[allow(dead_code)]
    selector: String,
    point: WebPoint,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebNode {
    tag: String,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    rect: WebRect,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    children: Vec<WebNode>,
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy)]
enum Align {
    Start,
    Center,
    End,
}

fn bounds_for_web_theme(theme: &WebGoldenTheme) -> Rect {
    let w = theme.viewport.map(|v| v.w).unwrap_or(1440.0);
    let h = theme.viewport.map(|v| v.h).unwrap_or(900.0);
    Rect::new(Point::new(Px(0.0), Px(0.0)), CoreSize::new(Px(w), Px(h)))
}

fn pad_root<H: UiHost>(cx: &mut ElementContext<'_, H>, pad: Px, child: AnyElement) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                layout
            },
            padding: Edges::all(pad),
            ..Default::default()
        },
        move |_cx| vec![child],
    )
}

fn shadcn_text_style(size: Px, line_height: Px, weight: FontWeight) -> TextStyle {
    TextStyle {
        font: FontId::default(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn shadcn_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
    style: TextStyle,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

fn shadcn_text_with_layout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
    style: TextStyle,
    layout: LayoutStyle,
) -> AnyElement {
    cx.text_props(TextProps {
        layout,
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

fn shadcn_nav_menu_demo_home_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
) -> AnyElement {
    let gap = Px(8.0); // Tailwind `gap-2` (0.5rem).

    let link_title_style = shadcn_text_style(Px(14.0), Px(14.0), FontWeight::MEDIUM); // text-sm leading-none font-medium
    let link_desc_style = shadcn_text_style(Px(14.0), Px(19.25), FontWeight::NORMAL); // text-sm leading-snug

    let tile_title_style = shadcn_text_style(Px(18.0), Px(28.0), FontWeight::MEDIUM); // text-lg font-medium
    let tile_desc_style = shadcn_text_style(Px(14.0), Px(17.5), FontWeight::NORMAL); // text-sm leading-tight

    let tile = cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill; // `w-full`
                layout
            },
            padding: Edges::all(Px(16.0)), // p-4
            ..Default::default()
        },
        move |cx| {
            vec![cx.column(
                ColumnProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout
                    },
                    // NavigationMenuLink: `gap-1` + title: `mb-2` => 12px total.
                    gap: Px(4.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::End, // `justify-end`
                    align: CrossAlign::Stretch,
                },
                move |cx| {
                    vec![
                        shadcn_text_with_layout(cx, "shadcn/ui", tile_title_style, {
                            let mut layout = LayoutStyle::default();
                            layout.margin.bottom = MarginEdge::Px(Px(8.0)); // mb-2
                            layout
                        }),
                        shadcn_text(
                            cx,
                            "Beautifully designed components built with Tailwind CSS.",
                            tile_desc_style,
                        ),
                    ]
                },
            )]
        },
    );

    fn list_item<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        model: Model<Option<Arc<str>>>,
        title: &'static str,
        description: &'static str,
        title_style: &TextStyle,
        desc_style: &TextStyle,
    ) -> AnyElement {
        let title = Arc::<str>::from(title);
        let description = Arc::<str>::from(description);
        let label = title.clone();
        let title_style = title_style.clone();
        let desc_style = desc_style.clone();

        let body = cx.container(
            ContainerProps {
                layout: LayoutStyle::default(),
                padding: Edges::all(Px(8.0)), // NavigationMenuLink: p-2
                ..Default::default()
            },
            move |cx| {
                let desc_style = desc_style.clone();
                vec![cx.column(
                    ColumnProps {
                        layout: LayoutStyle::default(),
                        gap: Px(4.0), // NavigationMenuLink: gap-1
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                    },
                    move |cx| {
                        let desc_style = desc_style.clone();
                        let desc_box = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.height =
                                        Length::Px(Px(desc_style.line_height.unwrap().0 * 2.0));
                                    layout.overflow = fret_ui::element::Overflow::Clip;
                                    layout
                                },
                                padding: Edges::all(Px(0.0)),
                                ..Default::default()
                            },
                            move |cx| vec![shadcn_text(cx, description.clone(), desc_style)],
                        );

                        vec![shadcn_text(cx, title.clone(), title_style), desc_box]
                    },
                )]
            },
        );

        fret_ui_shadcn::NavigationMenuLink::new(model, vec![body])
            .label(label)
            .into_element(cx)
    }

    let intro = list_item(
        cx,
        model.clone(),
        "Introduction",
        "Re-usable components built using Radix UI and Tailwind CSS.",
        &link_title_style,
        &link_desc_style,
    );
    let install = list_item(
        cx,
        model.clone(),
        "Installation",
        "How to install dependencies and structure your app.",
        &link_title_style,
        &link_desc_style,
    );
    let typography = list_item(
        cx,
        model,
        "Typography",
        "Styles for headings, paragraphs, lists...etc",
        &link_title_style,
        &link_desc_style,
    );

    cx.grid(
        GridProps {
            layout: {
                let mut layout = LayoutStyle::default();
                // Match the extracted shadcn web golden (mobile viewport content width).
                layout.size.width = Length::Px(Px(271.76044));
                layout
            },
            cols: 1,
            rows: None,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        },
        move |_cx| vec![tile, intro, install, typography],
    )
}

fn build_context_menu_demo<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    checked_bookmarks: Model<bool>,
    checked_full_urls: Model<bool>,
    radio_person: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonVariant, ContextMenu, ContextMenuCheckboxItem, ContextMenuEntry,
        ContextMenuItem, ContextMenuLabel, ContextMenuRadioGroup, ContextMenuRadioItemSpec,
        ContextMenuShortcut,
    };

    ContextMenu::new(open)
        // new-york-v4 context-menu-demo: `ContextMenuContent className="w-52"`.
        .min_width(Px(208.0))
        // new-york-v4 context-menu-demo: `ContextMenuSubContent className="w-44"`.
        .submenu_min_width(Px(176.0))
        .into_element(
            cx,
            |cx| {
                Button::new("Right click here")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| {
                vec![
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Back")
                            .inset(true)
                            .trailing(ContextMenuShortcut::new("⌘[").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Forward")
                            .inset(true)
                            .disabled(true)
                            .trailing(ContextMenuShortcut::new("⌘]").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Reload")
                            .inset(true)
                            .trailing(ContextMenuShortcut::new("⌘R").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").inset(true).submenu(
                        vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Delete").variant(
                                fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                            )),
                        ],
                    )),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                        checked_bookmarks,
                        "Show Bookmarks",
                    )),
                    ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                        checked_full_urls,
                        "Show Full URLs",
                    )),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::Label(ContextMenuLabel::new("People").inset(true)),
                    ContextMenuEntry::RadioGroup(
                        ContextMenuRadioGroup::new(radio_person)
                            .item(ContextMenuRadioItemSpec::new("pedro", "Pedro Duarte"))
                            .item(ContextMenuRadioItemSpec::new("colm", "Colm Tuite")),
                    ),
                ]
            },
        )
}

fn build_menubar_demo<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    view_bookmarks_bar: Model<bool>,
    view_full_urls: Model<bool>,
    profile_value: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Menubar, MenubarCheckboxItem, MenubarEntry, MenubarItem, MenubarMenu, MenubarRadioGroup,
        MenubarRadioItemSpec, MenubarShortcut,
    };

    Menubar::new(vec![
        MenubarMenu::new("File").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("New Tab").trailing(MenubarShortcut::new("⌘T").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("New Window")
                    .trailing(MenubarShortcut::new("⌘N").into_element(cx)),
            ),
            MenubarEntry::Item(MenubarItem::new("New Incognito Window").disabled(true)),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                MenubarEntry::Item(MenubarItem::new("Email link")),
                MenubarEntry::Item(MenubarItem::new("Messages")),
                MenubarEntry::Item(MenubarItem::new("Notes")),
            ])),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Print...").trailing(MenubarShortcut::new("⌘P").into_element(cx)),
            ),
        ]),
        MenubarMenu::new("Edit").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("Undo").trailing(MenubarShortcut::new("⌘Z").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Redo").trailing(MenubarShortcut::new("⇧⌘Z").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(MenubarItem::new("Find").submenu(vec![
                MenubarEntry::Item(MenubarItem::new("Search the web")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Find...")),
                MenubarEntry::Item(MenubarItem::new("Find Next")),
                MenubarEntry::Item(MenubarItem::new("Find Previous")),
            ])),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Cut")),
            MenubarEntry::Item(MenubarItem::new("Copy")),
            MenubarEntry::Item(MenubarItem::new("Paste")),
        ]),
        MenubarMenu::new("View").entries(vec![
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_bookmarks_bar,
                "Always Show Bookmarks Bar",
            )),
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_full_urls,
                "Always Show Full URLs",
            )),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Reload")
                    .inset(true)
                    .trailing(MenubarShortcut::new("⌘R").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Force Reload")
                    .disabled(true)
                    .inset(true)
                    .trailing(MenubarShortcut::new("⇧⌘R").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Toggle Fullscreen").inset(true)),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Hide Sidebar").inset(true)),
        ]),
        MenubarMenu::new("Profiles").entries(vec![
            MenubarEntry::RadioGroup(
                MenubarRadioGroup::new(profile_value)
                    .item(MenubarRadioItemSpec::new("andy", "Andy"))
                    .item(MenubarRadioItemSpec::new("benoit", "Benoit"))
                    .item(MenubarRadioItemSpec::new("Luis", "Luis")),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Edit...").inset(true)),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Add Profile...").inset(true)),
        ]),
    ])
    .into_element(cx)
}

fn first_container_px_size(element: &AnyElement) -> Option<(f32, f32)> {
    fn visit(node: &AnyElement) -> Option<(f32, f32)> {
        if let fret_ui::element::ElementKind::Container(props) = &node.kind {
            if let (Length::Px(w), Length::Px(h)) =
                (props.layout.size.width, props.layout.size.height)
            {
                return Some((w.0, h.0));
            }
        }
        for child in &node.children {
            if let Some(found) = visit(child) {
                return Some(found);
            }
        }
        None
    }
    visit(element)
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn web_golden_open_path(name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(format!("{name}.open.json"))
}

fn read_web_golden_open(name: &str) -> WebGolden {
    let path = web_golden_open_path(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web open golden: {}\nerror: {err}\n\nGenerate it via (in-process server):\n  node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 {name} --modes=open --update\n\nOr (external server):\n  pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts {name} --modes=open --update --baseUrl=http://localhost:4020\n\nDocs:\n  docs/shadcn-web-goldens.md",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "failed to parse web open golden: {}\nerror: {err}",
            path.display()
        )
    })
}

fn web_theme<'a>(golden: &'a WebGolden) -> &'a WebGoldenTheme {
    golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden")
}

fn find_first<'a>(node: &'a WebNode, pred: &impl Fn(&'a WebNode) -> bool) -> Option<&'a WebNode> {
    if pred(node) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_first(child, pred) {
            return Some(found);
        }
    }
    None
}

fn web_find_by_data_slot_and_state<'a>(
    root: &'a WebNode,
    slot: &str,
    state: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
            && n.attrs
                .get("data-state")
                .is_some_and(|v| v.as_str() == state)
    })
}

fn web_find_by_data_slot<'a>(root: &'a WebNode, slot: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
    })
}

fn web_portal_node_by_data_slot<'a>(theme: &'a WebGoldenTheme, slot: &str) -> &'a WebNode {
    for portal in &theme.portals {
        if let Some(found) = web_find_by_data_slot(portal, slot) {
            return found;
        }
    }
    for wrapper in &theme.portal_wrappers {
        if let Some(found) = web_find_by_data_slot(wrapper, slot) {
            return found;
        }
    }
    panic!("missing web portal node with data-slot={slot}")
}

fn find_attr_in_subtree<'a>(node: &'a WebNode, key: &str) -> Option<&'a str> {
    node.attrs.get(key).map(String::as_str).or_else(|| {
        for child in &node.children {
            if let Some(found) = find_attr_in_subtree(child, key) {
                return Some(found);
            }
        }
        None
    })
}

fn parse_side(value: &str) -> Option<Side> {
    Some(match value {
        "top" => Side::Top,
        "right" => Side::Right,
        "bottom" => Side::Bottom,
        "left" => Side::Left,
        _ => return None,
    })
}

fn parse_align(value: &str) -> Option<Align> {
    Some(match value {
        "start" => Align::Start,
        "center" => Align::Center,
        "end" => Align::End,
        _ => return None,
    })
}

fn rect_right(r: WebRect) -> f32 {
    r.x + r.w
}

fn fret_rect_contains(outer: Rect, inner: Rect) -> bool {
    let eps = 0.01;
    inner.origin.x.0 + eps >= outer.origin.x.0
        && inner.origin.y.0 + eps >= outer.origin.y.0
        && inner.origin.x.0 + inner.size.width.0 <= outer.origin.x.0 + outer.size.width.0 + eps
        && inner.origin.y.0 + inner.size.height.0 <= outer.origin.y.0 + outer.size.height.0 + eps
}

fn rect_bottom(r: WebRect) -> f32 {
    r.y + r.h
}

fn rect_center_x(r: WebRect) -> f32 {
    r.x + r.w * 0.5
}

fn rect_center_y(r: WebRect) -> f32 {
    r.y + r.h * 0.5
}

fn point_rect(p: WebPoint) -> WebRect {
    WebRect {
        x: p.x,
        y: p.y,
        w: 0.0,
        h: 0.0,
    }
}

fn rect_main_gap(side: Side, trigger: WebRect, content: WebRect) -> f32 {
    match side {
        Side::Bottom => content.y - rect_bottom(trigger),
        Side::Top => trigger.y - rect_bottom(content),
        Side::Right => content.x - rect_right(trigger),
        Side::Left => trigger.x - rect_right(content),
    }
}

fn rect_cross_delta(side: Side, align: Align, trigger: WebRect, content: WebRect) -> f32 {
    match side {
        Side::Top | Side::Bottom => match align {
            Align::Start => content.x - trigger.x,
            Align::Center => rect_center_x(content) - rect_center_x(trigger),
            Align::End => rect_right(content) - rect_right(trigger),
        },
        Side::Left | Side::Right => match align {
            Align::Start => content.y - trigger.y,
            Align::Center => rect_center_y(content) - rect_center_y(trigger),
            Align::End => rect_bottom(content) - rect_bottom(trigger),
        },
    }
}

fn infer_side(trigger: WebRect, content: WebRect) -> Side {
    let candidates = [
        (Side::Bottom, rect_main_gap(Side::Bottom, trigger, content)),
        (Side::Top, rect_main_gap(Side::Top, trigger, content)),
        (Side::Right, rect_main_gap(Side::Right, trigger, content)),
        (Side::Left, rect_main_gap(Side::Left, trigger, content)),
    ];
    candidates
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(side, _)| side)
        .unwrap_or(Side::Bottom)
}

fn infer_align(side: Side, trigger: WebRect, content: WebRect) -> Align {
    let candidates = [
        (
            Align::Start,
            rect_cross_delta(side, Align::Start, trigger, content).abs(),
        ),
        (
            Align::Center,
            rect_cross_delta(side, Align::Center, trigger, content).abs(),
        ),
        (
            Align::End,
            rect_cross_delta(side, Align::End, trigger, content).abs(),
        ),
    ];
    candidates
        .into_iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(align, _)| align)
        .unwrap_or(Align::Start)
}

fn assert_close(label: &str, actual: f32, expected: f32, tol: f32) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected＞{expected} (㊣{tol}) got={actual} (忖={delta})"
    );
}

fn web_portal_slot_heights(theme: &WebGoldenTheme, slots: &[&str]) -> Vec<f32> {
    let mut heights = Vec::new();

    let mut walk = |root: &WebNode| {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if let Some(slot) = node.attrs.get("data-slot") {
                if slots.iter().any(|s| slot == s) {
                    heights.push(node.rect.h);
                }
            }
            for child in &node.children {
                stack.push(child);
            }
        }
    };

    for portal in &theme.portals {
        walk(portal);
    }
    for portal in &theme.portal_wrappers {
        walk(portal);
    }

    heights
}

fn fret_menu_item_heights_in_menus(snap: &fret_core::SemanticsSnapshot) -> Vec<f32> {
    let debug = std::env::var("FRET_DEBUG_MENU_SEMANTICS")
        .ok()
        .is_some_and(|v| v == "1");
    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    if menus.is_empty() {
        if debug {
            let mut roles: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            for n in &snap.nodes {
                *roles.entry(format!("{:?}", n.role)).or_insert(0) += 1;
            }
            eprintln!("fret_menu_item_heights_in_menus: no Menu nodes; roles={roles:?}");
        }
        return Vec::new();
    }

    let menu_contains = |node: &fret_core::SemanticsNode| {
        menus
            .iter()
            .any(|menu| fret_rect_contains(menu.bounds, node.bounds))
    };

    let items: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| {
            matches!(
                n.role,
                SemanticsRole::MenuItem
                    | SemanticsRole::MenuItemCheckbox
                    | SemanticsRole::MenuItemRadio
            )
        })
        .collect();

    if debug {
        eprintln!(
            "fret_menu_item_heights_in_menus: menus={} items={}",
            menus.len(),
            items.len()
        );
        for (idx, menu) in menus.iter().take(2).enumerate() {
            eprintln!("  menu[{idx}] bounds={:?}", menu.bounds);
        }
        for (idx, item) in items.iter().take(6).enumerate() {
            eprintln!(
                "  item[{idx}] role={:?} label={:?} bounds={:?} in_menu={}",
                item.role,
                item.label.as_deref(),
                item.bounds,
                menu_contains(item)
            );
        }
    }

    items
        .into_iter()
        .filter(|n| menu_contains(n))
        .map(|n| n.bounds.size.height.0)
        .collect()
}

fn fret_largest_menu_height(snap: &fret_core::SemanticsSnapshot) -> Option<f32> {
    snap.nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .max_by(|a, b| {
            let area_a = a.bounds.size.width.0 * a.bounds.size.height.0;
            let area_b = b.bounds.size.width.0 * b.bounds.size.height.0;
            area_a.total_cmp(&area_b)
        })
        .map(|n| n.bounds.size.height.0)
}

fn fret_menu_heights(snap: &fret_core::SemanticsSnapshot) -> Vec<f32> {
    snap.nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .map(|n| n.bounds.size.height.0)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct InsetTriplet {
    left: f32,
    top_to_first_item: f32,
    right: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct InsetQuad {
    left: f32,
    top_to_first_option: f32,
    right: f32,
    bottom_from_last_option: f32,
}

fn web_menu_content_inset(menu: &WebNode) -> InsetTriplet {
    let is_menu_item_role = |node: &WebNode| {
        matches!(
            node.attrs.get("role").map(String::as_str),
            Some("menuitem") | Some("menuitemcheckbox") | Some("menuitemradio")
        )
    };

    let mut item_stack = vec![menu];
    let mut min_x = None::<f32>;
    let mut min_y = None::<f32>;
    let mut max_right = None::<f32>;

    while let Some(item_node) = item_stack.pop() {
        if is_menu_item_role(item_node) {
            let eps = 0.01;
            let menu_left = menu.rect.x;
            let menu_right = rect_right(menu.rect);
            let menu_top = menu.rect.y;
            let item_left = item_node.rect.x;
            let item_right = rect_right(item_node.rect);
            let item_top = item_node.rect.y;

            let within_panel = item_left + eps >= menu_left
                && item_right <= menu_right + eps
                && item_top + eps >= menu_top;
            if !within_panel {
                continue;
            }

            min_x = Some(min_x.unwrap_or(item_node.rect.x).min(item_node.rect.x));
            min_y = Some(min_y.unwrap_or(item_node.rect.y).min(item_node.rect.y));
            let r = rect_right(item_node.rect);
            max_right = Some(max_right.unwrap_or(r).max(r));
        }
        for child in &item_node.children {
            item_stack.push(child);
        }
    }

    let min_x = min_x.unwrap_or_else(|| panic!("web menu missing menuitem descendants"));
    let min_y = min_y.unwrap_or_else(|| panic!("web menu missing menuitem descendants"));
    let max_right = max_right.unwrap_or_else(|| panic!("web menu missing menuitem descendants"));

    InsetTriplet {
        left: min_x - menu.rect.x,
        top_to_first_item: min_y - menu.rect.y,
        right: rect_right(menu.rect) - max_right,
    }
}

fn web_menu_content_insets_for_slots(theme: &WebGoldenTheme, slots: &[&str]) -> Vec<InsetTriplet> {
    slots
        .iter()
        .map(|slot| web_portal_node_by_data_slot(theme, slot))
        .map(web_menu_content_inset)
        .collect()
}

fn web_select_listbox<'a>(theme: &'a WebGoldenTheme) -> &'a WebNode {
    theme
        .portals
        .iter()
        .chain(theme.portal_wrappers.iter())
        .find(|n| n.attrs.get("role").is_some_and(|v| v == "listbox"))
        .or_else(|| {
            theme
                .portals
                .iter()
                .chain(theme.portal_wrappers.iter())
                .find_map(|portal| {
                    find_first(portal, &|n| {
                        n.attrs.get("role").is_some_and(|v| v == "listbox")
                    })
                })
        })
        .unwrap_or_else(|| panic!("missing web select listbox portal"))
}

fn web_select_combobox<'a>(theme: &'a WebGoldenTheme) -> &'a WebNode {
    find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "combobox")
    })
    .unwrap_or_else(|| panic!("missing web select combobox"))
}

fn web_select_content_option_inset(listbox: &WebNode) -> InsetQuad {
    let mut option_stack = vec![listbox];
    let mut min_x = None::<f32>;
    let mut min_y = None::<f32>;
    let mut max_right = None::<f32>;
    let mut max_bottom = None::<f32>;

    while let Some(option_node) = option_stack.pop() {
        if option_node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "option")
        {
            let eps = 0.01;
            let panel_left = listbox.rect.x;
            let panel_right = rect_right(listbox.rect);
            let panel_top = listbox.rect.y;
            let panel_bottom = rect_bottom(listbox.rect);
            let option_left = option_node.rect.x;
            let option_right = rect_right(option_node.rect);
            let option_top = option_node.rect.y;
            let option_bottom = rect_bottom(option_node.rect);

            let within_panel = option_left + eps >= panel_left
                && option_right <= panel_right + eps
                && option_top + eps >= panel_top
                && option_bottom <= panel_bottom + eps;
            if !within_panel {
                continue;
            }

            min_x = Some(min_x.unwrap_or(option_left).min(option_left));
            min_y = Some(min_y.unwrap_or(option_top).min(option_top));
            max_right = Some(max_right.unwrap_or(option_right).max(option_right));
            max_bottom = Some(max_bottom.unwrap_or(option_bottom).max(option_bottom));
        }
        for child in &option_node.children {
            option_stack.push(child);
        }
    }

    let min_x = min_x.unwrap_or_else(|| panic!("web select listbox missing option descendants"));
    let min_y = min_y.unwrap_or_else(|| panic!("web select listbox missing option descendants"));
    let max_right =
        max_right.unwrap_or_else(|| panic!("web select listbox missing option descendants"));
    let max_bottom =
        max_bottom.unwrap_or_else(|| panic!("web select listbox missing option descendants"));

    InsetQuad {
        left: min_x - listbox.rect.x,
        top_to_first_option: min_y - listbox.rect.y,
        right: rect_right(listbox.rect) - max_right,
        bottom_from_last_option: rect_bottom(listbox.rect) - max_bottom,
    }
}

fn fret_menu_content_insets(snap: &fret_core::SemanticsSnapshot) -> Vec<InsetTriplet> {
    let is_menu_item = |n: &fret_core::SemanticsNode| {
        matches!(
            n.role,
            SemanticsRole::MenuItem
                | SemanticsRole::MenuItemCheckbox
                | SemanticsRole::MenuItemRadio
        )
    };

    let mut insets = Vec::new();

    for menu in snap.nodes.iter().filter(|n| n.role == SemanticsRole::Menu) {
        let items: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| is_menu_item(n))
            .filter(|n| fret_rect_contains(menu.bounds, n.bounds))
            .collect();

        if items.is_empty() {
            continue;
        }

        let mut min_x = items[0].bounds.origin.x.0;
        let mut min_y = items[0].bounds.origin.y.0;
        let mut max_right = items[0].bounds.origin.x.0 + items[0].bounds.size.width.0;
        for item in items.iter().skip(1) {
            min_x = min_x.min(item.bounds.origin.x.0);
            min_y = min_y.min(item.bounds.origin.y.0);
            max_right = max_right.max(item.bounds.origin.x.0 + item.bounds.size.width.0);
        }

        let menu_right = menu.bounds.origin.x.0 + menu.bounds.size.width.0;
        insets.push(InsetTriplet {
            left: min_x - menu.bounds.origin.x.0,
            top_to_first_item: min_y - menu.bounds.origin.y.0,
            right: menu_right - max_right,
        });
    }

    insets
}

fn assert_sorted_insets_match(web_name: &str, actual: &[InsetTriplet], expected: &[InsetTriplet]) {
    if expected.is_empty() {
        panic!("missing web menu insets for {web_name}");
    }
    if actual.is_empty() {
        panic!("missing fret menu insets for {web_name}");
    }
    assert!(
        actual.len() == expected.len(),
        "{web_name} expected {} menus, got {}",
        expected.len(),
        actual.len()
    );

    let mut expected_sorted = expected.to_vec();
    let mut actual_sorted = actual.to_vec();
    let sort_key = |v: &InsetTriplet| (round_i32(v.top_to_first_item), round_i32(v.left));
    expected_sorted.sort_by_key(sort_key);
    actual_sorted.sort_by_key(sort_key);

    for (i, (a, e)) in actual_sorted.iter().zip(expected_sorted.iter()).enumerate() {
        assert_close(
            &format!("{web_name} menu[{i}] left_inset"),
            a.left,
            e.left,
            1.0,
        );
        assert_close(
            &format!("{web_name} menu[{i}] top_to_first_item"),
            a.top_to_first_item,
            e.top_to_first_item,
            1.5,
        );
        assert_close(
            &format!("{web_name} menu[{i}] right_inset"),
            a.right,
            e.right,
            1.0,
        );
    }
}

fn fret_select_content_option_inset(snap: &fret_core::SemanticsSnapshot) -> InsetQuad {
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox"));

    let options: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| fret_rect_contains(listbox.bounds, n.bounds))
        .collect();

    if options.is_empty() {
        panic!("missing fret listbox options");
    }

    let mut min_x = options[0].bounds.origin.x.0;
    let mut min_y = options[0].bounds.origin.y.0;
    let mut max_right = options[0].bounds.origin.x.0 + options[0].bounds.size.width.0;
    let mut max_bottom = options[0].bounds.origin.y.0 + options[0].bounds.size.height.0;
    for option in options.iter().skip(1) {
        min_x = min_x.min(option.bounds.origin.x.0);
        min_y = min_y.min(option.bounds.origin.y.0);
        max_right = max_right.max(option.bounds.origin.x.0 + option.bounds.size.width.0);
        max_bottom = max_bottom.max(option.bounds.origin.y.0 + option.bounds.size.height.0);
    }

    let panel_right = listbox.bounds.origin.x.0 + listbox.bounds.size.width.0;
    let panel_bottom = listbox.bounds.origin.y.0 + listbox.bounds.size.height.0;

    InsetQuad {
        left: min_x - listbox.bounds.origin.x.0,
        top_to_first_option: min_y - listbox.bounds.origin.y.0,
        right: panel_right - max_right,
        bottom_from_last_option: panel_bottom - max_bottom,
    }
}

fn assert_select_inset_match(web_name: &str, actual: InsetQuad, expected: InsetQuad) {
    assert_close(
        &format!("{web_name} listbox left_inset"),
        actual.left,
        expected.left,
        1.0,
    );
    assert_close(
        &format!("{web_name} listbox top_to_first_option"),
        actual.top_to_first_option,
        expected.top_to_first_option,
        1.0,
    );
    assert_close(
        &format!("{web_name} listbox right_inset"),
        actual.right,
        expected.right,
        1.0,
    );
    assert_close(
        &format!("{web_name} listbox bottom_from_last_option"),
        actual.bottom_from_last_option,
        expected.bottom_from_last_option,
        1.0,
    );
}

fn round_i32(v: f32) -> i32 {
    v.round() as i32
}

fn assert_menu_item_row_height_matches(
    web_name: &str,
    expected_h: f32,
    actual_hs: &[f32],
    tol: f32,
) {
    if actual_hs.is_empty() {
        panic!("missing fret menu items for {web_name}");
    }

    let unique: std::collections::BTreeSet<i32> =
        actual_hs.iter().copied().map(round_i32).collect();
    assert!(
        unique.len() == 1,
        "{web_name} expected uniform menu item row height; got {unique:?}"
    );
    assert_close(
        &format!("{web_name} menu_item_row_h"),
        unique.iter().next().copied().unwrap_or_default() as f32,
        expected_h,
        tol,
    );
}

#[derive(Default)]
struct StyleAwareServices;

impl fret_core::TextService for StyleAwareServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        let (text, style) = match input {
            fret_core::TextInput::Plain { text, style } => (text.as_ref(), style.clone()),
            fret_core::TextInput::Attributed { text, base, .. } => (text.as_ref(), base.clone()),
            _ => (input.text(), fret_core::TextStyle::default()),
        };
        let line_height = style
            .line_height
            .unwrap_or(Px((style.size.0 * 1.4).max(0.0)));

        fn estimate_width_px(text: &str, font_size: f32) -> Px {
            let mut units = 0.0f32;
            for ch in text.chars() {
                units += match ch {
                    // Most overlay placement goldens use Geist @ 14px. We approximate its advance
                    // widths with a small heuristic table so both short labels ("Open popover") and
                    // long mixed-case strings ("Australian Western Standard Time (AWST)") land close
                    // to the web snapshots.
                    ' ' => 0.28,
                    '(' | ')' => 0.28,
                    // Narrow glyphs.
                    'i' | 'l' | 'I' | 't' | 'f' | 'j' | 'r' => 0.32,
                    // Wide glyphs.
                    'm' | 'w' | 'M' | 'W' => 0.75,
                    // Round glyphs.
                    'o' | 'O' | 'p' | 'P' => 0.62,
                    // Uppercase baseline.
                    'A'..='Z' => 0.62,
                    // Default lowercase baseline.
                    'a'..='z' => 0.56,
                    // Everything else (digits/punctuation) uses a neutral baseline.
                    _ => 0.56,
                };
            }
            Px((units * font_size).max(1.0))
        }

        let est_w = estimate_width_px(text, style.size.0);

        let max_w = constraints.max_width.unwrap_or(est_w);
        let (lines, w) = match constraints.wrap {
            fret_core::TextWrap::Word if max_w.0.is_finite() && max_w.0 > 0.0 => {
                let lines = (est_w.0 / max_w.0).ceil().max(1.0) as u32;
                (lines, Px(est_w.0.min(max_w.0)))
            }
            _ => (1, est_w),
        };

        let h = Px(line_height.0 * lines as f32);

        (
            fret_core::TextBlobId::default(),
            fret_core::TextMetrics {
                size: CoreSize::new(w, h),
                baseline: Px(h.0 * 0.8),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for StyleAwareServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for StyleAwareServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

fn setup_app_with_shadcn_theme(app: &mut App) {
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id: FrameId,
    request_semantics: bool,
    render: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) {
    app.set_frame_id(frame_id);
    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "web-vs-fret-overlay-placement",
        render,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn assert_overlay_placement_matches(
    web_name: &str,
    web_portal_role: Option<&str>,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
    fret_trigger_role: SemanticsRole,
    fret_trigger_label: Option<&str>,
    fret_portal_role: SemanticsRole,
) {
    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger = match web_name {
        "select-scrollable" => find_first(&web.themes["light"].root, &|n| {
            n.attrs.get("role").is_some_and(|v| v == "combobox")
        })
        .or_else(|| {
            find_first(&web.themes["dark"].root, &|n| {
                n.attrs.get("role").is_some_and(|v| v == "combobox")
            })
        })
        .expect("web trigger (combobox)"),
        "context-menu-demo" => find_first(&web.themes["light"].root, &|n| {
            n.text
                .as_deref()
                .is_some_and(|t| t.contains("Right click here"))
        })
        .or_else(|| {
            find_first(&web.themes["dark"].root, &|n| {
                n.text
                    .as_deref()
                    .is_some_and(|t| t.contains("Right click here"))
            })
        })
        .expect("web trigger (context menu)"),
        _ => find_first(&web.themes["light"].root, &|n| n.tag == "button")
            .or_else(|| find_first(&web.themes["dark"].root, &|n| n.tag == "button"))
            .expect("web trigger (button)"),
    };

    let web_portal_index = if let Some(web_portal_role) = web_portal_role {
        theme
            .portals
            .iter()
            .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
            .unwrap_or_else(|| panic!("missing web portal role={web_portal_role}"))
    } else {
        if theme.portals.is_empty() {
            panic!("missing web portals for {web_name}");
        }
        0
    };
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let content = build_frame1(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for step in 0..=settle_frames {
        let frame = 2 + step;
        let request_semantics = step == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let content = build_frame(cx, &open);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let trigger = snap
        .nodes
        .iter()
        .find(|n| {
            if n.role != fret_trigger_role {
                return false;
            }
            if let Some(label) = fret_trigger_label {
                return n.label.as_deref() == Some(label);
            }
            true
        })
        .unwrap_or_else(|| panic!("missing fret trigger role={fret_trigger_role:?}"));

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let ah = a.bounds.size.height.0;
            let bw = b.bounds.size.width.0;
            let bh = b.bounds.size.height.0;

            let score_a = (aw - expected_portal_w).abs() + (ah - expected_portal_h).abs();
            let score_b = (bw - expected_portal_w).abs() + (bh - expected_portal_h).abs();
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?}"));

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    if debug {
        let candidates: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == fret_portal_role)
            .collect();
        eprintln!(
            "{web_name} fret portal candidates role={fret_portal_role:?}: {}",
            candidates.len()
        );
        for (idx, n) in candidates.iter().enumerate().take(6) {
            eprintln!(
                "  [{idx}] bounds={:?} label={:?} flags={:?}",
                n.bounds, n.label, n.flags
            );
        }
        eprintln!(
            "{web_name} web side={web_side:?} align={web_align:?}\n  web trigger={:?}\n  web portal={:?}\n  fret trigger={:?}\n  fret portal={:?}",
            web_trigger.rect, web_portal.rect, fret_trigger, fret_portal
        );
    }

    assert_close(
        &format!("{web_name} main_gap"),
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        &format!("{web_name} cross_delta"),
        actual_cross,
        expected_cross,
        1.5,
    );

    if matches!(
        fret_portal_role,
        SemanticsRole::Menu | SemanticsRole::ListBox
    ) {
        assert_close(
            &format!("{web_name} portal_w"),
            fret_portal.w,
            expected_portal_w,
            2.0,
        );
        assert_close(
            &format!("{web_name} portal_h"),
            fret_portal.h,
            expected_portal_h,
            2.0,
        );
    }
}

fn assert_centered_overlay_placement_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_portal_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
        .unwrap_or_else(|| panic!("missing web portal role={web_portal_role} for {web_name}"));
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let expected_center_x = rect_center_x(web_portal.rect);
    let expected_center_y = rect_center_y(web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let content = build_frame1(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for step in 0..=settle_frames {
        let frame = 2 + step;
        let request_semantics = step == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let content = build_frame(cx, &open);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let center = (rect_center_x(r) - expected_center_x).abs()
                    + (rect_center_y(r) - expected_center_y).abs();
                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                center + 0.02 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?} for {web_name}"));

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    if debug {
        eprintln!(
            "{web_name} web portal={:?} expected_center=({}, {})",
            web_portal.rect, expected_center_x, expected_center_y
        );
        eprintln!("{web_name} selected fret portal={:?}", fret_portal);
    }

    assert_close(
        &format!("{web_name} center_x"),
        rect_center_x(fret_portal),
        expected_center_x,
        2.0,
    );
    assert_close(
        &format!("{web_name} width"),
        fret_portal.w,
        expected_portal_w,
        2.0,
    );
    assert_close(
        &format!("{web_name} center_y"),
        rect_center_y(fret_portal),
        expected_center_y,
        2.0,
    );
}

fn assert_viewport_anchored_overlay_placement_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_portal_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
        .unwrap_or_else(|| panic!("missing web portal role={web_portal_role} for {web_name}"));
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let expected_left = web_portal.rect.x;
    let expected_top = web_portal.rect.y;
    let expected_right = 1440.0 - rect_right(web_portal.rect);
    let expected_bottom = 900.0 - rect_bottom(web_portal.rect);

    let anchor_tol = 2.0;
    let anchor_left = expected_left.abs() <= anchor_tol;
    let anchor_top = expected_top.abs() <= anchor_tol;
    let anchor_right = expected_right.abs() <= anchor_tol;
    let anchor_bottom = expected_bottom.abs() <= anchor_tol;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let content = build_frame1(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_500 + 2;
    for frame_id in 2..=(2 + settle_frames) {
        let request_semantics = frame_id == 2 + settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_id),
            request_semantics,
            |cx| {
                let content = build_frame(cx, &open);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let left = r.x;
                let top = r.y;
                let right = 1440.0 - rect_right(r);
                let bottom = 900.0 - rect_bottom(r);

                let mut score = 0.0;
                if anchor_left {
                    score += (left - expected_left).abs();
                }
                if anchor_top {
                    score += (top - expected_top).abs();
                }
                if anchor_right {
                    score += (right - expected_right).abs();
                }
                if anchor_bottom {
                    score += (bottom - expected_bottom).abs();
                }

                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                score + 0.02 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?} for {web_name}"));

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_left = fret_portal.x;
    let actual_top = fret_portal.y;
    let actual_right = 1440.0 - rect_right(fret_portal);
    let actual_bottom = 900.0 - rect_bottom(fret_portal);

    if debug {
        eprintln!(
            "{web_name} anchors: left={anchor_left} top={anchor_top} right={anchor_right} bottom={anchor_bottom}"
        );
        eprintln!(
            "{web_name} web portal={:?} expected_insets=(l={expected_left}, t={expected_top}, r={expected_right}, b={expected_bottom})",
            web_portal.rect
        );
        eprintln!(
            "{web_name} fret portal={:?} actual_insets=(l={actual_left}, t={actual_top}, r={actual_right}, b={actual_bottom})",
            fret_portal
        );
    }

    if anchor_left {
        assert_close(
            &format!("{web_name} inset_left"),
            actual_left,
            expected_left,
            2.0,
        );
    }
    if anchor_top {
        assert_close(
            &format!("{web_name} inset_top"),
            actual_top,
            expected_top,
            2.0,
        );
    }
    if anchor_right {
        assert_close(
            &format!("{web_name} inset_right"),
            actual_right,
            expected_right,
            2.0,
        );
    }
    if anchor_bottom {
        assert_close(
            &format!("{web_name} inset_bottom"),
            actual_bottom,
            expected_bottom,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_popover_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "popover-demo",
        Some("dialog"),
        |cx, open| {
            fret_ui_shadcn::Popover::new(open.clone()).into_element(
                cx,
                |cx| {
                    fret_ui_shadcn::Button::new("Open popover")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    let content = fret_ui_shadcn::PopoverContent::new(Vec::new())
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(fret_ui_kit::MetricRef::Px(Px(320.0)))
                                .h_px(fret_ui_kit::MetricRef::Px(Px(245.33334))),
                        )
                        .into_element(cx);
                    if std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
                        .ok()
                        .is_some_and(|v| v == "1")
                    {
                        eprintln!(
                            "popover-demo content container px size={:?}",
                            first_container_px_size(&content)
                        );
                    }
                    content
                },
            )
        },
        SemanticsRole::Button,
        Some("Open popover"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "dropdown-menu-demo",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::{
                Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
                DropdownMenuLabel, DropdownMenuShortcut,
            };

            DropdownMenu::new(open.clone())
                // new-york-v4 dropdown-menu-demo: `DropdownMenuContent className="w-56"`.
                .min_width(Px(224.0))
                .into_element(
                    cx,
                    |cx| {
                        Button::new("Open")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Profile")
                                    .trailing(DropdownMenuShortcut::new("⇧⌘P").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Billing")
                                    .trailing(DropdownMenuShortcut::new("⌘B").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Settings")
                                    .trailing(DropdownMenuShortcut::new("⌘S").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Keyboard shortcuts")
                                    .trailing(DropdownMenuShortcut::new("⌘K").into_element(cx)),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                                vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                    DropdownMenuEntry::Separator,
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                                ],
                            )),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("New Team")
                                    .trailing(DropdownMenuShortcut::new("⌘+T").into_element(cx)),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Log out")
                                    .trailing(DropdownMenuShortcut::new("⇧⌘Q").into_element(cx)),
                            ),
                        ]
                    },
                )
        },
        SemanticsRole::Button,
        Some("Open"),
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "dropdown-menu-demo.vp1440x320",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::{
                Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
                DropdownMenuLabel, DropdownMenuShortcut,
            };

            DropdownMenu::new(open.clone())
                // new-york-v4 dropdown-menu-demo: `DropdownMenuContent className="w-56"`.
                .min_width(Px(224.0))
                .into_element(
                    cx,
                    |cx| {
                        Button::new("Open")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Profile")
                                    .trailing(DropdownMenuShortcut::new("??P").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Billing")
                                    .trailing(DropdownMenuShortcut::new("?B").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Settings")
                                    .trailing(DropdownMenuShortcut::new("?S").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Keyboard shortcuts")
                                    .trailing(DropdownMenuShortcut::new("?K").into_element(cx)),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                                vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                    DropdownMenuEntry::Separator,
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                                ],
                            )),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("New Team")
                                    .trailing(DropdownMenuShortcut::new("?+T").into_element(cx)),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Log out")
                                    .trailing(DropdownMenuShortcut::new("??Q").into_element(cx)),
                            ),
                        ]
                    },
                )
        },
        SemanticsRole::Button,
        Some("Open"),
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_menu_item_height_matches() {
    let web = read_web_golden_open("dropdown-menu-demo.vp1440x320");
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(
        &theme,
        &[
            "dropdown-menu-item",
            "dropdown-menu-checkbox-item",
            "dropdown-menu-radio-item",
            "dropdown-menu-sub-trigger",
        ],
    );
    let expected_h =
        expected_hs.iter().copied().next().unwrap_or_else(|| {
            panic!("missing web menu item rows for dropdown-menu-demo.vp1440x320")
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
            DropdownMenuLabel, DropdownMenuShortcut,
        };

        DropdownMenu::new(open.clone())
            .min_width(Px(224.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Profile")
                                .trailing(DropdownMenuShortcut::new("??P").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Billing")
                                .trailing(DropdownMenuShortcut::new("?B").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Settings")
                                .trailing(DropdownMenuShortcut::new("?S").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Keyboard shortcuts")
                                .trailing(DropdownMenuShortcut::new("?K").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                            vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ],
                        )),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("New Team")
                                .trailing(DropdownMenuShortcut::new("?+T").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Log out")
                                .trailing(DropdownMenuShortcut::new("??Q").into_element(cx)),
                        ),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(
        "dropdown-menu-demo.vp1440x320",
        expected_h.round(),
        &actual_hs,
        1.0,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_menu_content_insets_match() {
    let web = read_web_golden_open("dropdown-menu-demo.vp1440x320");
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
            DropdownMenuLabel, DropdownMenuShortcut,
        };

        DropdownMenu::new(open.clone())
            .min_width(Px(224.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Profile")
                                .trailing(DropdownMenuShortcut::new("??P").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Billing")
                                .trailing(DropdownMenuShortcut::new("?B").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Settings")
                                .trailing(DropdownMenuShortcut::new("?S").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Keyboard shortcuts")
                                .trailing(DropdownMenuShortcut::new("?K").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                            vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ],
                        )),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("New Team")
                                .trailing(DropdownMenuShortcut::new("?+T").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Log out")
                                .trailing(DropdownMenuShortcut::new("??Q").into_element(cx)),
                        ),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match("dropdown-menu-demo.vp1440x320", &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for dropdown-menu-demo.vp1440x320"));
    assert_close(
        "dropdown-menu-demo.vp1440x320 menu height",
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

fn assert_dropdown_menu_demo_submenu_overlay_placement_matches(web_name: &str) {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_sub_menu = web_portal_node_by_data_slot(theme, "dropdown-menu-sub-content");
    let web_sub_trigger = web_portal_node_by_data_slot(theme, "dropdown-menu-sub-trigger");

    let expected_dx = web_sub_menu.rect.x - rect_right(web_sub_trigger.rect);
    let expected_dy = web_sub_menu.rect.y - web_sub_trigger.rect.y;
    let expected_w = web_sub_menu.rect.w;
    let expected_h = web_sub_menu.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = DropdownMenu::new(open.clone())
                // new-york-v4 dropdown-menu-demo: `DropdownMenuContent className="w-56"`.
                .min_width(Px(224.0))
                .into_element(
                    cx,
                    |cx| Button::new("Open").into_element(cx),
                    |_cx| {
                        vec![DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Invite users").submenu(vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ]),
                        )]
                    },
                );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            let el = DropdownMenu::new(open.clone())
                .min_width(Px(224.0))
                .into_element(
                    cx,
                    |cx| Button::new("Open").into_element(cx),
                    |_cx| {
                        vec![DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Invite users").submenu(vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ]),
                        )]
                    },
                );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Invite users"))
        .expect("fret submenu trigger semantics");
    ui.set_focus(Some(trigger.id));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            request_semantics,
            |cx| {
                let el = DropdownMenu::new(open.clone())
                    .min_width(Px(224.0))
                    .into_element(
                        cx,
                        |cx| Button::new("Open").into_element(cx),
                        |_cx| {
                            vec![DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Invite users").submenu(vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                    DropdownMenuEntry::Separator,
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                                ]),
                            )]
                        },
                    );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Invite users"))
        .expect("fret submenu trigger semantics (final)");

    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    assert!(
        menus.len() >= 2,
        "expected at least 2 menu panels after opening submenu; got {}",
        menus.len()
    );

    let root_menu = menus
        .iter()
        .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
        .expect("root menu contains sub-trigger");
    let submenu = menus
        .iter()
        .find(|m| !fret_rect_contains(m.bounds, trigger.bounds))
        .expect("submenu menu does not contain sub-trigger");

    let actual_dx =
        submenu.bounds.origin.x.0 - (trigger.bounds.origin.x.0 + trigger.bounds.size.width.0);
    let actual_dy = submenu.bounds.origin.y.0 - trigger.bounds.origin.y.0;
    let actual_w = submenu.bounds.size.width.0;
    let actual_h = submenu.bounds.size.height.0;

    assert_close(
        &format!("{web_name} submenu dx"),
        actual_dx,
        expected_dx,
        2.0,
    );
    assert_close(
        &format!("{web_name} submenu dy"),
        actual_dy,
        expected_dy,
        2.0,
    );
    assert_close(&format!("{web_name} submenu w"), actual_w, expected_w, 2.0);
    assert_close(&format!("{web_name} submenu h"), actual_h, expected_h, 2.0);

    // Ensure the root menu is also present (guards against selecting some unrelated menu).
    assert!(
        root_menu.bounds.size.width.0 > 0.0 && root_menu.bounds.size.height.0 > 0.0,
        "expected root menu bounds to be non-zero"
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches("dropdown-menu-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_hover_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches("dropdown-menu-demo.submenu");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_small_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_small_viewport_menu_content_insets_match() {
    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x320");
    let theme = web_theme(&web);
    let expected_slots = ["dropdown-menu-content", "dropdown-menu-sub-content"];
    let expected = web_menu_content_insets_for_slots(&theme, &expected_slots);
    let expected_hs: Vec<f32> = expected_slots
        .iter()
        .map(|slot| web_portal_node_by_data_slot(&theme, slot).rect.h)
        .collect();

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
            DropdownMenuLabel, DropdownMenuShortcut,
        };

        DropdownMenu::new(open.clone())
            .min_width(Px(224.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Profile")
                                .trailing(DropdownMenuShortcut::new("??P").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Billing")
                                .trailing(DropdownMenuShortcut::new("?B").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Settings")
                                .trailing(DropdownMenuShortcut::new("?S").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Keyboard shortcuts")
                                .trailing(DropdownMenuShortcut::new("?K").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                            vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ],
                        )),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("New Team")
                                .trailing(DropdownMenuShortcut::new("?+T").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Log out")
                                .trailing(DropdownMenuShortcut::new("??Q").into_element(cx)),
                        ),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let root_menu = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Menu)
        .expect("fret root menu semantics");
    let wheel_pos = Point::new(
        Px(root_menu.bounds.origin.x.0 + root_menu.bounds.size.width.0 * 0.5),
        Px(root_menu.bounds.origin.y.0 + root_menu.bounds.size.height.0 * 0.5),
    );

    // The web golden scrolls the menu before opening the submenu (via `scrollIntoView` in the
    // golden extraction openSteps). Replicate the same state by wheel-scrolling the menu.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: wheel_pos,
            delta: Point::new(Px(0.0), Px(-80.0)),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Invite users"))
        .expect("fret submenu trigger semantics (Invite users)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(4 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
        &actual,
        &expected,
    );

    let mut actual_hs = fret_menu_heights(&snap);
    assert!(
        actual_hs.len() == expected_hs.len(),
        "dropdown-menu-demo.submenu-kbd-vp1440x320 expected {} menus, got {}",
        expected_hs.len(),
        actual_hs.len()
    );
    let mut expected_hs = expected_hs;
    expected_hs.sort_by(|a, b| b.total_cmp(a));
    actual_hs.sort_by(|a, b| b.total_cmp(a));
    for (i, (a, e)) in actual_hs.iter().zip(expected_hs.iter()).enumerate() {
        assert_close(
            &format!("dropdown-menu-demo.submenu-kbd-vp1440x320 menu[{i}] height"),
            *a,
            *e,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_select_scrollable_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-scrollable",
        Some("listbox"),
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("North America").into(),
                    SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Europe & Africa").into(),
                    SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    SelectItem::new("cet", "Central European Time (CET)").into(),
                    SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Asia").into(),
                    SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    SelectItem::new("ist", "India Standard Time (IST)").into(),
                    SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)")
                        .into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Australia & Pacific").into(),
                    SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                    SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                    SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                    SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("South America").into(),
                    SelectItem::new("art", "Argentina Time (ART)").into(),
                    SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a timezone")
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(280.0))),
                )
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-scrollable.vp1440x450",
        Some("listbox"),
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("North America").into(),
                    SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Europe & Africa").into(),
                    SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    SelectItem::new("cet", "Central European Time (CET)").into(),
                    SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Asia").into(),
                    SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    SelectItem::new("ist", "India Standard Time (IST)").into(),
                    SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)")
                        .into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Australia & Pacific").into(),
                    SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                    SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                    SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                    SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("South America").into(),
                    SelectItem::new("art", "Argentina Time (ART)").into(),
                    SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a timezone")
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(280.0))),
                )
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-scrollable.vp1440x240",
        Some("listbox"),
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("North America").into(),
                    SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Europe & Africa").into(),
                    SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    SelectItem::new("cet", "Central European Time (CET)").into(),
                    SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Asia").into(),
                    SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    SelectItem::new("ist", "India Standard Time (IST)").into(),
                    SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)")
                        .into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Australia & Pacific").into(),
                    SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                    SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                    SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                    SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("South America").into(),
                    SelectItem::new("art", "Argentina Time (ART)").into(),
                    SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a timezone")
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(280.0))),
                )
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}

fn assert_select_scrollable_listbox_option_insets_match(web_name: &str) {
    let debug = std::env::var("FRET_DEBUG_SELECT_SCROLLABLE")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_select_listbox(theme);
    let expected_h = web_listbox.rect.h;
    let expected_inset = web_select_content_option_inset(web_listbox);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

        let entries: Vec<SelectEntry> = vec![
            SelectGroup::new(vec![
                SelectLabel::new("North America").into(),
                SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                SelectItem::new("cst", "Central Standard Time (CST)").into(),
                SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Europe & Africa").into(),
                SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                SelectItem::new("cet", "Central European Time (CET)").into(),
                SelectItem::new("eet", "Eastern European Time (EET)").into(),
                SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                SelectItem::new("eat", "East Africa Time (EAT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Asia").into(),
                SelectItem::new("msk", "Moscow Time (MSK)").into(),
                SelectItem::new("ist", "India Standard Time (IST)").into(),
                SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Australia & Pacific").into(),
                SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                SelectItem::new("fjt", "Fiji Time (FJT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("South America").into(),
                SelectItem::new("art", "Argentina Time (ART)").into(),
                SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
            ])
            .into(),
        ];

        fret_ui_shadcn::Select::new(value.clone(), open.clone())
            .a11y_label("Select")
            .placeholder("Select a timezone")
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(fret_ui_kit::MetricRef::Px(Px(280.0))),
            )
            .entries(entries)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox for {web_name}"));

    if debug {
        let mut options: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::ListBoxOption)
            .filter(|n| fret_rect_contains(listbox.bounds, n.bounds))
            .collect();
        options.sort_by(|a, b| {
            a.bounds
                .origin
                .y
                .0
                .total_cmp(&b.bounds.origin.y.0)
                .then_with(|| a.bounds.origin.x.0.total_cmp(&b.bounds.origin.x.0))
        });

        eprintln!(
            "[{web_name}] fret listbox y={} h={}",
            listbox.bounds.origin.y.0, listbox.bounds.size.height.0
        );
        for (idx, opt) in options.iter().take(8).enumerate() {
            eprintln!(
                "  opt[{idx}] y={} h={} label={:?}",
                opt.bounds.origin.y.0,
                opt.bounds.size.height.0,
                opt.label.as_deref()
            );
        }

        let scroll_buttons: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Button)
            .filter(|n| {
                n.label.as_deref() == Some("Scroll up") || n.label.as_deref() == Some("Scroll down")
            })
            .collect();
        for btn in scroll_buttons {
            eprintln!(
                "  button {:?} y={} h={}",
                btn.label.as_deref(),
                btn.bounds.origin.y.0,
                btn.bounds.size.height.0
            );
        }
    }

    assert_close(
        &format!("{web_name} listbox_h"),
        listbox.bounds.size.height.0,
        expected_h,
        1.0,
    );

    let actual_inset = fret_select_content_option_inset(&snap);
    assert_select_inset_match(web_name, actual_inset, expected_inset);
}

#[test]
fn web_vs_fret_select_scrollable_listbox_option_insets_match() {
    assert_select_scrollable_listbox_option_insets_match("select-scrollable");
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_listbox_option_insets_match() {
    assert_select_scrollable_listbox_option_insets_match("select-scrollable.vp1440x450");
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_listbox_option_insets_match() {
    assert_select_scrollable_listbox_option_insets_match("select-scrollable.vp1440x240");
}

fn assert_select_scrollable_listbox_width_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_select_listbox(theme);
    let expected_w = web_listbox.rect.w;
    let expected_trigger_w = web_select_combobox(theme).rect.w;
    assert!(expected_w + 0.01 >= expected_trigger_w);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

        let entries: Vec<SelectEntry> = vec![
            SelectGroup::new(vec![
                SelectLabel::new("North America").into(),
                SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                SelectItem::new("cst", "Central Standard Time (CST)").into(),
                SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Europe & Africa").into(),
                SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                SelectItem::new("cet", "Central European Time (CET)").into(),
                SelectItem::new("eet", "Eastern European Time (EET)").into(),
                SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                SelectItem::new("eat", "East Africa Time (EAT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Asia").into(),
                SelectItem::new("msk", "Moscow Time (MSK)").into(),
                SelectItem::new("ist", "India Standard Time (IST)").into(),
                SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Australia & Pacific").into(),
                SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                SelectItem::new("fjt", "Fiji Time (FJT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("South America").into(),
                SelectItem::new("art", "Argentina Time (ART)").into(),
                SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
            ])
            .into(),
        ];

        fret_ui_shadcn::Select::new(value.clone(), open.clone())
            .a11y_label("Select")
            .placeholder("Select a timezone")
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(fret_ui_kit::MetricRef::Px(Px(280.0))),
            )
            .entries(entries)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let combobox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox)
        .unwrap_or_else(|| panic!("missing fret combobox for {web_name}"));
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox for {web_name}"));

    assert!(
        listbox.bounds.size.width.0 + 0.01 >= combobox.bounds.size.width.0,
        "{web_name} expected listbox width >= trigger width ({} >= {})",
        listbox.bounds.size.width.0,
        combobox.bounds.size.width.0
    );
    assert_close(
        &format!("{web_name} listbox_w"),
        listbox.bounds.size.width.0,
        expected_w,
        2.0,
    );
}

#[test]
fn web_vs_fret_select_scrollable_listbox_width_matches() {
    assert_select_scrollable_listbox_width_matches("select-scrollable");
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_listbox_width_matches() {
    assert_select_scrollable_listbox_width_matches("select-scrollable.vp1440x450");
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_listbox_width_matches() {
    assert_select_scrollable_listbox_width_matches("select-scrollable.vp1440x240");
}

fn web_portal_first_node_by_role<'a>(theme: &'a WebGoldenTheme, role: &str) -> &'a WebNode {
    for portal in &theme.portals {
        if let Some(found) = find_first(portal, &|n| n.attrs.get("role").is_some_and(|v| v == role))
        {
            return found;
        }
    }
    for wrapper in &theme.portal_wrappers {
        if let Some(found) =
            find_first(wrapper, &|n| n.attrs.get("role").is_some_and(|v| v == role))
        {
            return found;
        }
    }
    panic!("missing web portal node with role={role}")
}

fn assert_combobox_demo_listbox_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_portal_first_node_by_role(theme, "listbox");
    let expected_h = web_listbox.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Combobox, ComboboxItem};

        let items = vec![
            ComboboxItem::new("apple", "Apple"),
            ComboboxItem::new("banana", "Banana"),
            ComboboxItem::new("blueberry", "Blueberry"),
            ComboboxItem::new("grapes", "Grapes"),
            ComboboxItem::new("pineapple", "Pineapple"),
        ];

        Combobox::new(value.clone(), open.clone())
            .a11y_label("Select a fruit")
            .width(Px(200.0))
            .items(items)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret combobox listbox for {web_name}"));

    assert_close(
        &format!("{web_name} combobox listbox_h"),
        listbox.bounds.size.height.0,
        expected_h,
        2.0,
    );
}

#[test]
fn web_vs_fret_combobox_demo_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-demo");
}

#[test]
fn web_vs_fret_combobox_demo_small_viewport_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-demo.vp375x320");
}

fn assert_point_anchored_overlay_placement_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_portal_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
    open_fret_at: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        AppWindowId,
        WebPoint,
    ),
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let open_point = theme
        .open
        .as_ref()
        .map(|m| m.point)
        .unwrap_or_else(|| panic!("missing web open point for {web_name}"));

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
        .unwrap_or_else(|| panic!("missing web portal role={web_portal_role}"));
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let web_trigger = point_rect(open_point);
    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let content = build_frame1(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    open_fret_at(&mut ui, &mut app, &mut services, window, open_point);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    let build_settle = build.clone();
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let content = build_settle(cx, &open);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let ah = a.bounds.size.height.0;
            let bw = b.bounds.size.width.0;
            let bh = b.bounds.size.height.0;

            let score_a = (aw - expected_portal_w).abs() + (ah - expected_portal_h).abs();
            let score_b = (bw - expected_portal_w).abs() + (bh - expected_portal_h).abs();
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?}"));

    let fret_trigger = point_rect(open_point);
    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close(
        &format!("{web_name} main_gap"),
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        &format!("{web_name} cross_delta"),
        actual_cross,
        expected_cross,
        1.5,
    );

    if fret_portal_role == SemanticsRole::Menu {
        assert_close(
            &format!("{web_name} portal_w"),
            fret_portal.w,
            expected_portal_w,
            2.0,
        );
        assert_close(
            &format!("{web_name} portal_h"),
            fret_portal.h,
            expected_portal_h,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_context_menu_demo_overlay_placement_matches() {
    assert_point_anchored_overlay_placement_matches(
        "context-menu-demo",
        "menu",
        SemanticsRole::Menu,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_bookmarks: Option<Model<bool>>,
                checked_full_urls: Option<Model<bool>>,
                radio_person: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_bookmarks.as_ref(),
                    st.checked_full_urls.as_ref(),
                    st.radio_person.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_bookmarks, checked_full_urls, radio_person) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_bookmarks = cx.app.models_mut().insert(true);
                    let checked_full_urls = cx.app.models_mut().insert(false);
                    let radio_person = cx.app.models_mut().insert(Some(Arc::from("pedro")));

                    cx.with_state(Models::default, |st| {
                        st.checked_bookmarks = Some(checked_bookmarks.clone());
                        st.checked_full_urls = Some(checked_full_urls.clone());
                        st.radio_person = Some(radio_person.clone());
                    });

                    (checked_bookmarks, checked_full_urls, radio_person)
                };

            fret_ui_shadcn::ContextMenu::new(open.clone())
                // new-york-v4 context-menu-demo: `ContextMenuContent className="w-52"`.
                .min_width(Px(208.0))
                // new-york-v4 context-menu-demo: `ContextMenuSubContent className="w-44"`.
                .submenu_min_width(Px(176.0))
                .into_element(
                cx,
                |cx| {
                    cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(300.0));
                                layout.size.height = Length::Px(Px(150.0));
                                layout
                            },
                            ..Default::default()
                        },
                        |cx| vec![cx.text("Right click here")],
                    )
                },
                |cx| {
                    vec![
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Back")
                                .inset(true)
                                .trailing(
                                    fret_ui_shadcn::ContextMenuShortcut::new("⌘[")
                                        .into_element(cx),
                                ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Forward")
                                .inset(true)
                                .disabled(true)
                                .trailing(
                                    fret_ui_shadcn::ContextMenuShortcut::new("⌘]")
                                        .into_element(cx),
                                ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Reload")
                                .inset(true)
                                .trailing(
                                    fret_ui_shadcn::ContextMenuShortcut::new("⌘R")
                                        .into_element(cx),
                                ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("More Tools").inset(true).submenu(
                                vec![
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Save Page..."),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new(
                                            "Create Shortcut...",
                                        ),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Name Window..."),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Separator,
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Developer Tools"),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Separator,
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Delete").variant(
                                            fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                        ),
                                    ),
                                ],
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Separator,
                        fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                            fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                checked_bookmarks,
                                "Show Bookmarks",
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                            fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                checked_full_urls,
                                "Show Full URLs",
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Separator,
                        fret_ui_shadcn::ContextMenuEntry::Label(
                            fret_ui_shadcn::ContextMenuLabel::new("People").inset(true),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::RadioGroup(
                            fret_ui_shadcn::ContextMenuRadioGroup::new(radio_person)
                                .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                    "pedro",
                                    "Pedro Duarte",
                                ))
                                .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                    "colm",
                                    "Colm Tuite",
                                )),
                        ),
                    ]
                },
            )
        },
        |ui, app, services, _window, point| {
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Down {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Up {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_small_viewport_overlay_placement_matches() {
    assert_point_anchored_overlay_placement_matches(
        "context-menu-demo.vp1440x320",
        "menu",
        SemanticsRole::Menu,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_bookmarks: Option<Model<bool>>,
                checked_full_urls: Option<Model<bool>>,
                radio_person: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_bookmarks.as_ref(),
                    st.checked_full_urls.as_ref(),
                    st.radio_person.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_bookmarks, checked_full_urls, radio_person) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_bookmarks = cx.app.models_mut().insert(true);
                    let checked_full_urls = cx.app.models_mut().insert(false);
                    let radio_person = cx.app.models_mut().insert(Some(Arc::from("pedro")));

                    cx.with_state(Models::default, |st| {
                        st.checked_bookmarks = Some(checked_bookmarks.clone());
                        st.checked_full_urls = Some(checked_full_urls.clone());
                        st.radio_person = Some(radio_person.clone());
                    });

                    (checked_bookmarks, checked_full_urls, radio_person)
                };

            fret_ui_shadcn::ContextMenu::new(open.clone())
                // new-york-v4 context-menu-demo: `ContextMenuContent className="w-52"`.
                .min_width(Px(208.0))
                // new-york-v4 context-menu-demo: `ContextMenuSubContent className="w-44"`.
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| {
                        cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(300.0));
                                    layout.size.height = Length::Px(Px(150.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            |cx| vec![cx.text("Right click here")],
                        )
                    },
                    |cx| {
                        vec![
                            fret_ui_shadcn::ContextMenuEntry::Item(
                                fret_ui_shadcn::ContextMenuItem::new("Back")
                                    .inset(true)
                                    .trailing(
                                        fret_ui_shadcn::ContextMenuShortcut::new("?[")
                                            .into_element(cx),
                                    ),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::Item(
                                fret_ui_shadcn::ContextMenuItem::new("Forward")
                                    .inset(true)
                                    .disabled(true)
                                    .trailing(
                                        fret_ui_shadcn::ContextMenuShortcut::new("?]")
                                            .into_element(cx),
                                    ),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::Item(
                                fret_ui_shadcn::ContextMenuItem::new("Reload")
                                    .inset(true)
                                    .trailing(
                                        fret_ui_shadcn::ContextMenuShortcut::new("?R")
                                            .into_element(cx),
                                    ),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::Item(
                                fret_ui_shadcn::ContextMenuItem::new("More Tools")
                                    .inset(true)
                                    .submenu(vec![
                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                            fret_ui_shadcn::ContextMenuItem::new("Save Page..."),
                                        ),
                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                            fret_ui_shadcn::ContextMenuItem::new(
                                                "Create Shortcut...",
                                            ),
                                        ),
                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                            fret_ui_shadcn::ContextMenuItem::new("Name Window..."),
                                        ),
                                        fret_ui_shadcn::ContextMenuEntry::Separator,
                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                            fret_ui_shadcn::ContextMenuItem::new(
                                                "Developer Tools",
                                            ),
                                        ),
                                        fret_ui_shadcn::ContextMenuEntry::Separator,
                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                            fret_ui_shadcn::ContextMenuItem::new("Delete").variant(
                                                fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                            ),
                                        ),
                                    ]),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::Separator,
                            fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                                fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                    checked_bookmarks.clone(),
                                    "Show Bookmarks",
                                ),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                                fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                    checked_full_urls.clone(),
                                    "Show Full URLs",
                                ),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::Separator,
                            fret_ui_shadcn::ContextMenuEntry::Label(
                                fret_ui_shadcn::ContextMenuLabel::new("People").inset(true),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::RadioGroup(
                                fret_ui_shadcn::ContextMenuRadioGroup::new(radio_person.clone())
                                    .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                        "pedro",
                                        "Pedro Duarte",
                                    ))
                                    .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                        "colm",
                                        "Colm Tuite",
                                    )),
                            ),
                        ]
                    },
                )
        },
        |ui, app, services, _window, point| {
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Down {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Up {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_small_viewport_menu_item_height_matches() {
    let web = read_web_golden_open("context-menu-demo.vp1440x320");
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(
        &theme,
        &[
            "context-menu-item",
            "context-menu-checkbox-item",
            "context-menu-radio-item",
            "context-menu-sub-trigger",
        ],
    );
    let expected_h =
        expected_hs.iter().copied().next().unwrap_or_else(|| {
            panic!("missing web menu item rows for context-menu-demo.vp1440x320")
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for frame in 2..=4 {
        let request_semantics = frame == 4;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_context_menu_demo(
                    cx,
                    open.clone(),
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(
        "context-menu-demo.vp1440x320",
        expected_h.round(),
        &actual_hs,
        1.0,
    );
}

#[test]
fn web_vs_fret_context_menu_demo_small_viewport_menu_content_insets_match() {
    let web = read_web_golden_open("context-menu-demo.vp1440x320");
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["context-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "context-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for frame in 2..=4 {
        let request_semantics = frame == 4;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_context_menu_demo(
                    cx,
                    open.clone(),
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match("context-menu-demo.vp1440x320", &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for context-menu-demo.vp1440x320"));
    assert_close(
        "context-menu-demo.vp1440x320 menu height",
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

fn assert_context_menu_demo_submenu_overlay_placement_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_sub_menu = web_portal_node_by_data_slot(theme, "context-menu-sub-content");
    let web_sub_trigger = web_portal_node_by_data_slot(theme, "context-menu-sub-trigger");

    let expected_dx = web_sub_menu.rect.x - rect_right(web_sub_trigger.rect);
    let expected_dy = web_sub_menu.rect.y - web_sub_trigger.rect.y;
    let expected_w = web_sub_menu.rect.w;
    let expected_h = web_sub_menu.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger_button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("fret trigger button semantics");
    let click_point = Point::new(
        Px(trigger_button.bounds.origin.x.0 + trigger_button.bounds.size.width.0 * 0.5),
        Px(trigger_button.bounds.origin.y.0 + trigger_button.bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"));
    let trigger = trigger.unwrap_or_else(|| {
        let menu_items: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::MenuItem)
            .filter_map(|n| n.label.as_deref())
            .collect();
        panic!("fret submenu trigger semantics missing; menu_items={menu_items:?}");
    });
    ui.set_focus(Some(trigger.id));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            request_semantics,
            |cx| {
                let el = build_context_menu_demo(
                    cx,
                    open.clone(),
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
        .expect("fret submenu trigger semantics (final)");

    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    assert!(
        menus.len() >= 2,
        "expected at least 2 menu panels after opening submenu; got {}",
        menus.len()
    );

    let _root_menu = menus
        .iter()
        .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
        .expect("root menu contains sub-trigger");
    let submenu = menus
        .iter()
        .find(|m| !fret_rect_contains(m.bounds, trigger.bounds))
        .expect("submenu menu does not contain sub-trigger");

    let actual_dx =
        submenu.bounds.origin.x.0 - (trigger.bounds.origin.x.0 + trigger.bounds.size.width.0);
    let actual_dy = submenu.bounds.origin.y.0 - trigger.bounds.origin.y.0;
    let actual_w = submenu.bounds.size.width.0;
    let actual_h = submenu.bounds.size.height.0;

    assert_close(
        &format!("{web_name} submenu dx"),
        actual_dx,
        expected_dx,
        2.0,
    );
    assert_close(
        &format!("{web_name} submenu dy"),
        actual_dy,
        expected_dy,
        2.0,
    );
    assert_close(&format!("{web_name} submenu w"), actual_w, expected_w, 2.0);
    assert_close(&format!("{web_name} submenu h"), actual_h, expected_h, 2.0);
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches("context-menu-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_hover_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches("context-menu-demo.submenu");
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_small_viewport_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches(
        "context-menu-demo.submenu-kbd-vp1440x320",
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_small_viewport_menu_content_insets_match() {
    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x320");
    let theme = web_theme(&web);
    let expected_slots = ["context-menu-content", "context-menu-sub-content"];
    let expected = web_menu_content_insets_for_slots(&theme, &expected_slots);
    let expected_hs: Vec<f32> = expected_slots
        .iter()
        .map(|slot| web_portal_node_by_data_slot(&theme, slot).rect.h)
        .collect();

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger_button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("fret trigger button semantics");
    let click_point = Point::new(
        Px(trigger_button.bounds.origin.x.0 + trigger_button.bounds.size.width.0 * 0.5),
        Px(trigger_button.bounds.origin.y.0 + trigger_button.bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
        .expect("fret submenu trigger semantics (More Tools)");
    ui.set_focus(Some(trigger.id));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            request_semantics,
            |cx| {
                let el = build_context_menu_demo(
                    cx,
                    open.clone(),
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(
        "context-menu-demo.submenu-kbd-vp1440x320",
        &actual,
        &expected,
    );

    let mut actual_hs = fret_menu_heights(&snap);
    assert!(
        actual_hs.len() == expected_hs.len(),
        "context-menu-demo.submenu-kbd-vp1440x320 expected {} menus, got {}",
        expected_hs.len(),
        actual_hs.len()
    );
    let mut expected_hs = expected_hs;
    expected_hs.sort_by(|a, b| b.total_cmp(a));
    actual_hs.sort_by(|a, b| b.total_cmp(a));
    for (i, (a, e)) in actual_hs.iter().zip(expected_hs.iter()).enumerate() {
        assert_close(
            &format!("context-menu-demo.submenu-kbd-vp1440x320 menu[{i}] height"),
            *a,
            *e,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_tooltip_demo_overlay_placement_matches() {
    let web = read_web_golden_open("tooltip-demo");
    let theme = web_theme(&web);

    let web_trigger = find_first(&web.themes["light"].root, &|n| n.tag == "button")
        .or_else(|| find_first(&web.themes["dark"].root, &|n| n.tag == "button"))
        .expect("web trigger (button)");
    let trigger_w = web_trigger.rect.w;
    let trigger_h = web_trigger.rect.h;

    if theme.portals.is_empty() {
        panic!("missing web portals for tooltip-demo");
    }
    let web_portal_leaf = &theme.portals[0];
    let web_portal = theme.portal_wrappers.get(0).unwrap_or(web_portal_leaf);
    let content_w = web_portal.rect.w;
    let content_h = web_portal.rect.h;

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let trigger_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
    let content_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let trigger_id_out = trigger_id_out.clone();
            let content_id_out = content_id_out.clone();
            let trigger = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(trigger_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(trigger_h))),
                )
                .into_element(cx);
            trigger_id_out.set(Some(trigger.id));
            let content = fret_ui_shadcn::TooltipContent::new(vec![cx.text("Add to library")])
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(content_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(content_h))),
                )
                .into_element(cx);
            content_id_out.set(Some(content.id));
            let tooltip = fret_ui_shadcn::Tooltip::new(trigger, content).into_element(cx);
            vec![pad_root(cx, Px(0.0), tooltip)]
        },
    );

    let trigger_element = trigger_id_out.get().expect("tooltip trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("tooltip trigger node");
    ui.set_focus(Some(trigger_node));

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let trigger_id_out = trigger_id_out.clone();
                let content_id_out = content_id_out.clone();
                let trigger = fret_ui_shadcn::Button::new("Hover")
                    .variant(fret_ui_shadcn::ButtonVariant::Outline)
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(fret_ui_kit::MetricRef::Px(Px(trigger_w)))
                            .h_px(fret_ui_kit::MetricRef::Px(Px(trigger_h))),
                    )
                    .into_element(cx);
                trigger_id_out.set(Some(trigger.id));
                let content = fret_ui_shadcn::TooltipContent::new(vec![cx.text("Add to library")])
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(fret_ui_kit::MetricRef::Px(Px(content_w)))
                            .h_px(fret_ui_kit::MetricRef::Px(Px(content_h))),
                    )
                    .into_element(cx);
                content_id_out.set(Some(content.id));
                let tooltip = fret_ui_shadcn::Tooltip::new(trigger, content).into_element(cx);
                vec![pad_root(cx, Px(0.0), tooltip)]
            },
        );
    }

    let trigger_element = trigger_id_out.get().expect("tooltip trigger element id");
    let content_element = content_id_out.get().expect("tooltip content element id");

    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_element).expect("tooltip trigger bounds");
    let portal_bounds =
        bounds_for_element(&mut app, window, content_element).expect("tooltip content bounds");

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "tooltip-demo web trigger={:?} web portal={:?} fret trigger={:?} fret portal={:?}",
            web_trigger.rect, web_portal.rect, trigger_bounds, portal_bounds
        );
    }

    let fret_trigger = WebRect {
        x: trigger_bounds.origin.x.0,
        y: trigger_bounds.origin.y.0,
        w: trigger_bounds.size.width.0,
        h: trigger_bounds.size.height.0,
    };
    let fret_portal = WebRect {
        x: portal_bounds.origin.x.0,
        y: portal_bounds.origin.y.0,
        w: portal_bounds.size.width.0,
        h: portal_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close("tooltip-demo main_gap", actual_gap, expected_gap, 1.0);
    assert_close(
        "tooltip-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
}

#[test]
fn web_vs_fret_hover_card_demo_overlay_placement_matches() {
    let web = read_web_golden_open("hover-card-demo");
    let theme = web_theme(&web);

    let web_trigger = find_first(&web.themes["light"].root, &|n| n.tag == "button")
        .or_else(|| find_first(&web.themes["dark"].root, &|n| n.tag == "button"))
        .expect("web trigger (button)");
    let trigger_w = web_trigger.rect.w;
    let trigger_h = web_trigger.rect.h;

    if theme.portals.is_empty() {
        panic!("missing web portals for hover-card-demo");
    }
    let web_portal_leaf = &theme.portals[0];
    let web_portal = theme.portal_wrappers.get(0).unwrap_or(web_portal_leaf);
    let content_w = web_portal.rect.w;
    let content_h = web_portal.rect.h;

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let trigger_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
    let content_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let trigger_id_out = trigger_id_out.clone();
            let content_id_out = content_id_out.clone();
            let trigger = fret_ui_shadcn::Button::new("@nextjs")
                .variant(fret_ui_shadcn::ButtonVariant::Link)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(trigger_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(trigger_h))),
                )
                .into_element(cx);
            trigger_id_out.set(Some(trigger.id));

            let content = fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")])
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(content_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(content_h))),
                )
                .into_element(cx);
            content_id_out.set(Some(content.id));

            let hover_card = fret_ui_shadcn::HoverCard::new(trigger, content)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx);

            vec![pad_root(cx, Px(0.0), hover_card)]
        },
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::KeyA,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let trigger_element = trigger_id_out.get().expect("hover card trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("hover card trigger node");
    ui.set_focus(Some(trigger_node));

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let trigger_id_out = trigger_id_out.clone();
                let content_id_out = content_id_out.clone();
                let trigger = fret_ui_shadcn::Button::new("@nextjs")
                    .variant(fret_ui_shadcn::ButtonVariant::Link)
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(fret_ui_kit::MetricRef::Px(Px(trigger_w)))
                            .h_px(fret_ui_kit::MetricRef::Px(Px(trigger_h))),
                    )
                    .into_element(cx);
                trigger_id_out.set(Some(trigger.id));

                let content = fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")])
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(fret_ui_kit::MetricRef::Px(Px(content_w)))
                            .h_px(fret_ui_kit::MetricRef::Px(Px(content_h))),
                    )
                    .into_element(cx);
                content_id_out.set(Some(content.id));

                let hover_card = fret_ui_shadcn::HoverCard::new(trigger, content)
                    .open_delay_frames(0)
                    .close_delay_frames(0)
                    .into_element(cx);

                vec![pad_root(cx, Px(0.0), hover_card)]
            },
        );
    }

    let trigger_element = trigger_id_out.get().expect("hover card trigger element id");
    let content_element = content_id_out.get().expect("hover card content element id");

    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_element).expect("hover card trigger bounds");
    let portal_bounds =
        bounds_for_element(&mut app, window, content_element).expect("hover card content bounds");

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "hover-card-demo web trigger={:?} web portal={:?} fret trigger={:?} fret portal={:?}",
            web_trigger.rect, web_portal.rect, trigger_bounds, portal_bounds
        );
    }

    let fret_trigger = WebRect {
        x: trigger_bounds.origin.x.0,
        y: trigger_bounds.origin.y.0,
        w: trigger_bounds.size.width.0,
        h: trigger_bounds.size.height.0,
    };
    let fret_portal = WebRect {
        x: portal_bounds.origin.x.0,
        y: portal_bounds.origin.y.0,
        w: portal_bounds.size.width.0,
        h: portal_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close("hover-card-demo main_gap", actual_gap, expected_gap, 1.0);
    assert_close(
        "hover-card-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_overlay_placement_matches() {
    let web = read_web_golden_open("navigation-menu-demo");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .unwrap_or_else(|| {
                find_first(&theme.root, &|n| {
                    n.attrs
                        .get("data-slot")
                        .is_some_and(|v| v.as_str() == "navigation-menu-trigger")
                })
                .expect("web trigger slot=navigation-menu-trigger")
            });
    let web_content =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-content", "open")
            .expect("web content slot=navigation-menu-content state=open");

    let web_side = infer_side(web_trigger.rect, web_content.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_content.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_content.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_content.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(vec![fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(vec![fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![cx.text("Content")],
                    )])
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, "home",
            )
        },
    )
    .expect("fret nav menu content id");
    let content_bounds =
        bounds_for_element(&mut app, window, content_id).expect("fret nav menu content bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_content = WebRect {
        x: content_bounds.origin.x.0,
        y: content_bounds.origin.y.0,
        w: content_bounds.size.width.0,
        h: content_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_content);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_content);

    assert_close(
        "navigation-menu-demo main_gap",
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        "navigation-menu-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
}

fn web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
    web_name: &str,
    open_value: &str,
    open_label: &str,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .unwrap_or_else(|| {
                find_first(&theme.root, &|n| {
                    n.attrs
                        .get("data-slot")
                        .is_some_and(|v| v.as_str() == "navigation-menu-trigger")
                })
                .expect("web trigger slot=navigation-menu-trigger")
            });
    let web_content =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-content", "open")
            .expect("web content slot=navigation-menu-content state=open");

    let web_side = infer_side(web_trigger.rect, web_content.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_content.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_content.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_content.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "components",
                    "Components",
                    vec![cx.text("Components")],
                ),
                fret_ui_shadcn::NavigationMenuItem::new("list", "List", vec![cx.text("List")]),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "simple",
                    "Simple",
                    vec![cx.text("Simple")],
                ),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "with-icon",
                    "With Icon",
                    vec![cx.text("With Icon")],
                ),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(open_label))
        .unwrap_or_else(|| panic!("fret trigger semantics ({open_label})"));
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "components",
                        "Components",
                        vec![cx.text("Components")],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new("list", "List", vec![cx.text("List")]),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "simple",
                        "Simple",
                        vec![cx.text("Simple")],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "with-icon",
                        "With Icon",
                        vec![cx.text("With Icon")],
                    ),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu content id for {open_value}"));
    let content_bounds =
        bounds_for_element(&mut app, window, content_id).expect("fret nav menu content bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(open_label))
        .unwrap_or_else(|| panic!("fret trigger semantics ({open_label})"));

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_content = WebRect {
        x: content_bounds.origin.x.0,
        y: content_bounds.origin.y.0,
        w: content_bounds.size.width.0,
        h: content_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_content);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_content);

    let label = format!("{web_name} main_gap");
    assert_close(&label, actual_gap, expected_gap, 1.0);
    let label = format!("{web_name} cross_delta");
    assert_close(&label, actual_cross, expected_cross, 1.5);
    let label = format!("{web_name} trigger_height");
    assert_close(&label, fret_trigger.h, web_trigger.rect.h, 1.0);
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.components",
        "components",
        "Components",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.components-vp1440x320",
        "components",
        "Components",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_list_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.list",
        "list",
        "List",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_list_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.list-vp1440x320",
        "list",
        "List",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_simple_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.simple-vp1440x320",
        "simple",
        "Simple",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_simple_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.simple",
        "simple",
        "Simple",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_with_icon_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.with-icon-vp1440x320",
        "with-icon",
        "With Icon",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_with_icon_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.with-icon",
        "with-icon",
        "With Icon",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_viewport_height_matches() {
    let web = read_web_golden_open("navigation-menu-demo.home-mobile");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .expect("web trigger slot=navigation-menu-trigger state=open");
    let web_viewport =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-viewport", "open")
            .expect("web viewport slot=navigation-menu-viewport state=open");

    let web_side = infer_side(web_trigger.rect, web_viewport.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_viewport.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_viewport.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_viewport.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new("components", "Components", Vec::new()),
                fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new("components", "Components", Vec::new()),
                    fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let viewport_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("fret nav menu viewport panel id");
    let viewport_bounds =
        bounds_for_element(&mut app, window, viewport_id).expect("fret nav menu viewport bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_viewport = WebRect {
        x: viewport_bounds.origin.x.0,
        y: viewport_bounds.origin.y.0,
        w: viewport_bounds.size.width.0,
        h: viewport_bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_NAV_MENU_MOBILE")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "nav-menu home-mobile web viewport={:?} web trigger={:?} fret viewport={:?} fret trigger={:?}",
            web_viewport.rect, web_trigger.rect, fret_viewport, fret_trigger
        );
    }

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_viewport);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_viewport);

    assert_close(
        "navigation-menu-demo.home-mobile main_gap",
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        "navigation-menu-demo.home-mobile cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile viewport_height",
        fret_viewport.h,
        web_viewport.rect.h,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile viewport_width",
        fret_viewport.w,
        web_viewport.rect.w,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile trigger_height",
        fret_trigger.h,
        web_trigger.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_small_viewport_height_matches() {
    let web = read_web_golden_open("navigation-menu-demo.home-mobile-vp375x320");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .expect("web trigger slot=navigation-menu-trigger state=open");
    let web_viewport =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-viewport", "open")
            .expect("web viewport slot=navigation-menu-viewport state=open");

    let web_side = infer_side(web_trigger.rect, web_viewport.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_viewport.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_viewport.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_viewport.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new("components", "Components", Vec::new()),
                fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new("components", "Components", Vec::new()),
                    fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let viewport_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-query-small",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("fret nav menu viewport panel id");
    let viewport_bounds =
        bounds_for_element(&mut app, window, viewport_id).expect("fret nav menu viewport bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_viewport = WebRect {
        x: viewport_bounds.origin.x.0,
        y: viewport_bounds.origin.y.0,
        w: viewport_bounds.size.width.0,
        h: viewport_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_viewport);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_viewport);

    assert_close(
        "navigation-menu-demo.home-mobile-vp375x320 main_gap",
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        "navigation-menu-demo.home-mobile-vp375x320 cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile-vp375x320 viewport_height",
        fret_viewport.h,
        web_viewport.rect.h,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile-vp375x320 viewport_width",
        fret_viewport.w,
        web_viewport.rect.w,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile-vp375x320 trigger_height",
        fret_trigger.h,
        web_trigger.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_menubar_demo_overlay_placement_matches() {
    let web = read_web_golden_open("menubar-demo");
    let theme = web_theme(&web);

    let web_trigger = web_find_by_data_slot_and_state(&theme.root, "menubar-trigger", "open")
        .unwrap_or_else(|| {
            find_first(&theme.root, &|n| {
                n.attrs
                    .get("data-slot")
                    .is_some_and(|v| v.as_str() == "menubar-trigger")
            })
            .expect("web trigger slot=menubar-trigger")
        });

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == "menu"))
        .expect("web portal role=menu");
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let gap = rect_main_gap(web_side, fret_trigger, r);
                let cross = rect_cross_delta(web_side, web_align, fret_trigger, r);
                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                (gap - expected_gap).abs() + (cross - expected_cross).abs() + 0.05 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("fret menubar portal semantics (Menu)");

    if debug {
        let candidates: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Menu)
            .collect();
        eprintln!("menubar-demo fret Menu candidates: {}", candidates.len());
        for (idx, n) in candidates.iter().enumerate().take(8) {
            eprintln!("  [{idx}] bounds={:?} label={:?}", n.bounds, n.label);
        }
        eprintln!(
            "menubar-demo web trigger={:?} web portal={:?}\n  fret trigger={:?}\n  selected portal={:?}",
            web_trigger.rect, web_portal.rect, fret_trigger, portal.bounds
        );
        eprintln!(
            "menubar-demo fret trigger flags={:?} root_count={} node_count={}",
            trigger.flags,
            snap.roots.len(),
            snap.nodes.len()
        );
    }

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close("menubar-demo main_gap", actual_gap, expected_gap, 1.0);
    assert_close(
        "menubar-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "menubar-demo portal_w",
        fret_portal.w,
        expected_portal_w,
        2.0,
    );
    assert_close(
        "menubar-demo portal_h",
        fret_portal.h,
        expected_portal_h,
        2.0,
    );
}

#[test]
fn web_vs_fret_menubar_demo_small_viewport_overlay_placement_matches() {
    let web = read_web_golden_open("menubar-demo.vp1440x320");
    let theme = web_theme(&web);

    let web_trigger = web_find_by_data_slot_and_state(&theme.root, "menubar-trigger", "open")
        .unwrap_or_else(|| {
            find_first(&theme.root, &|n| {
                n.attrs
                    .get("data-slot")
                    .is_some_and(|v| v.as_str() == "menubar-trigger")
            })
            .expect("web trigger slot=menubar-trigger")
        });

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == "menu"))
        .expect("web portal role=menu");
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let gap = rect_main_gap(web_side, fret_trigger, r);
                let cross = rect_cross_delta(web_side, web_align, fret_trigger, r);
                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                (gap - expected_gap).abs() + (cross - expected_cross).abs() + 0.05 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("fret menubar portal semantics (Menu)");

    if debug {
        let candidates: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Menu)
            .collect();
        eprintln!(
            "menubar-demo.vp1440x320 fret Menu candidates: {}",
            candidates.len()
        );
        for (idx, n) in candidates.iter().enumerate().take(8) {
            eprintln!("  [{idx}] bounds={:?} label={:?}", n.bounds, n.label);
        }
        eprintln!(
            "menubar-demo.vp1440x320 web trigger={:?} web portal={:?}\n  fret trigger={:?}\n  selected portal={:?}",
            web_trigger.rect, web_portal.rect, fret_trigger, portal.bounds
        );
        eprintln!(
            "menubar-demo.vp1440x320 fret trigger flags={:?} root_count={} node_count={}",
            trigger.flags,
            snap.roots.len(),
            snap.nodes.len()
        );
    }

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close(
        "menubar-demo.vp1440x320 main_gap",
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        "menubar-demo.vp1440x320 cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "menubar-demo.vp1440x320 portal_w",
        fret_portal.w,
        expected_portal_w,
        2.0,
    );
    assert_close(
        "menubar-demo.vp1440x320 portal_h",
        fret_portal.h,
        expected_portal_h,
        2.0,
    );
}

#[test]
fn web_vs_fret_menubar_demo_small_viewport_menu_item_height_matches() {
    let web = read_web_golden_open("menubar-demo.vp1440x320");
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(
        &theme,
        &[
            "menubar-item",
            "menubar-checkbox-item",
            "menubar-radio-item",
            "menubar-sub-trigger",
        ],
    );
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web menu item rows for menubar-demo.vp1440x320"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(
        "menubar-demo.vp1440x320",
        expected_h.round(),
        &actual_hs,
        1.0,
    );
}

#[test]
fn web_vs_fret_menubar_demo_small_viewport_menu_content_insets_match() {
    let web = read_web_golden_open("menubar-demo.vp1440x320");
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["menubar-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "menubar-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match("menubar-demo.vp1440x320", &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for menubar-demo.vp1440x320"));
    assert_close(
        "menubar-demo.vp1440x320 menu height",
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

fn assert_menubar_demo_submenu_overlay_placement_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_sub_menu = web_portal_node_by_data_slot(theme, "menubar-sub-content");
    let web_sub_trigger = web_portal_node_by_data_slot(theme, "menubar-sub-trigger");

    let expected_dx = web_sub_menu.rect.x - rect_right(web_sub_trigger.rect);
    let expected_dy = web_sub_menu.rect.y - web_sub_trigger.rect.y;
    let expected_w = web_sub_menu.rect.w;
    let expected_h = web_sub_menu.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
        .expect("fret submenu trigger semantics (Share)");
    ui.set_focus(Some(trigger.id));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + settle_frames + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
        .expect("fret submenu trigger semantics (Share, final)");

    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    assert!(
        menus.len() >= 2,
        "expected at least 2 menu panels after opening submenu; got {}",
        menus.len()
    );

    let _root_menu = menus
        .iter()
        .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
        .expect("root menu contains sub-trigger");
    let submenu = menus
        .iter()
        .find(|m| !fret_rect_contains(m.bounds, trigger.bounds))
        .expect("submenu menu does not contain sub-trigger");

    let actual_dx =
        submenu.bounds.origin.x.0 - (trigger.bounds.origin.x.0 + trigger.bounds.size.width.0);
    let actual_dy = submenu.bounds.origin.y.0 - trigger.bounds.origin.y.0;
    let actual_w = submenu.bounds.size.width.0;
    let actual_h = submenu.bounds.size.height.0;

    assert_close(
        &format!("{web_name} submenu dx"),
        actual_dx,
        expected_dx,
        2.0,
    );
    assert_close(
        &format!("{web_name} submenu dy"),
        actual_dy,
        expected_dy,
        2.0,
    );
    assert_close(&format!("{web_name} submenu w"), actual_w, expected_w, 2.0);
    assert_close(&format!("{web_name} submenu h"), actual_h, expected_h, 2.0);
}

#[test]
fn web_vs_fret_menubar_demo_submenu_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_hover_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_small_viewport_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu-kbd-vp1440x320");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_small_viewport_menu_content_insets_match() {
    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x320");
    let theme = web_theme(&web);
    let expected_slots = ["menubar-content", "menubar-sub-content"];
    let expected = web_menu_content_insets_for_slots(&theme, &expected_slots);
    let expected_hs: Vec<f32> = expected_slots
        .iter()
        .map(|slot| web_portal_node_by_data_slot(&theme, slot).rect.h)
        .collect();

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let file_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");
    let click_point = Point::new(
        Px(file_trigger.bounds.origin.x.0 + file_trigger.bounds.size.width.0 * 0.5),
        Px(file_trigger.bounds.origin.y.0 + file_trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let share_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
        .expect("fret submenu trigger semantics (Share)");
    ui.set_focus(Some(share_trigger.id));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + settle_frames + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match("menubar-demo.submenu-kbd-vp1440x320", &actual, &expected);

    let mut actual_hs = fret_menu_heights(&snap);
    assert!(
        actual_hs.len() == expected_hs.len(),
        "menubar-demo.submenu-kbd-vp1440x320 expected {} menus, got {}",
        expected_hs.len(),
        actual_hs.len()
    );
    let mut expected_hs = expected_hs;
    expected_hs.sort_by(|a, b| b.total_cmp(a));
    actual_hs.sort_by(|a, b| b.total_cmp(a));
    for (i, (a, e)) in actual_hs.iter().zip(expected_hs.iter()).enumerate() {
        assert_close(
            &format!("menubar-demo.submenu-kbd-vp1440x320 menu[{i}] height"),
            *a,
            *e,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_dialog_demo_overlay_center_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "dialog-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(vec![cx.text("Edit profile")])
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .max_w(fret_ui_kit::MetricRef::Px(Px(425.0))),
                        )
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_command_dialog_overlay_center_matches() {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    assert_centered_overlay_placement_matches(
        "command-dialog",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            CommandDialog::new(open.clone(), query, items)
                .into_element(cx, |cx| Button::new("Open").into_element(cx))
        },
    );
}

#[test]
fn web_vs_fret_alert_dialog_demo_overlay_center_matches() {
    use fret_ui_shadcn::{AlertDialog, AlertDialogContent, Button, ButtonVariant};

    assert_centered_overlay_placement_matches(
        "alert-dialog-demo",
        "alertdialog",
        SemanticsRole::AlertDialog,
        |cx, open| {
            AlertDialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Show Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    AlertDialogContent::new(vec![cx.text("Are you absolutely sure?")])
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_demo_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_top_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Top).into_element(
                cx,
                |cx| {
                    Button::new("top")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_right_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.right",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Right)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("right")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_bottom_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.bottom",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Bottom)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("bottom")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_left_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.left",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Left).into_element(
                cx,
                |cx| {
                    Button::new("left")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
