use fret_app::App;
use fret_core::{
    AppWindowId, Edges, Event, FrameId, ImageId, Modifiers, MouseButtons, NodeId, Point,
    PointerEvent, PointerId, PointerType, Px, Rect, Scene, SceneOp, SemanticsRole,
    Size as CoreSize, TextOverflow, TextWrap, Transform2D,
};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::Theme;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, FlexProps, GridProps, LayoutStyle, Length,
    MainAlign, PressableProps, RovingFlexProps, RowProps, SizeStyle, TextProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::tree::UiTree;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::primitives::radio_group as radio_group_prim;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};
use fret_ui_shadcn::button_group::ButtonGroupText;
use fret_ui_shadcn::empty::{
    EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use fret_ui_shadcn::sidebar::SidebarMenuButtonSize;
use serde::Deserialize;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

mod css_color;
use css_color::{Rgba, color_to_rgba, parse_css_color};
mod chart_test_data;
use chart_test_data::{CHART_INTERACTIVE_DESKTOP, CHART_INTERACTIVE_MOBILE};

#[path = "web_vs_fret_layout/accordion.rs"]
mod accordion;
#[path = "web_vs_fret_layout/avatar.rs"]
mod avatar;
#[path = "web_vs_fret_layout/badge.rs"]
mod badge;
#[path = "web_vs_fret_layout/basic.rs"]
mod basic;
#[path = "web_vs_fret_layout/breadcrumb.rs"]
mod breadcrumb;
#[path = "web_vs_fret_layout/button.rs"]
mod button;
#[path = "web_vs_fret_layout/calendar.rs"]
mod calendar;
#[path = "web_vs_fret_layout/card.rs"]
mod card;
#[path = "web_vs_fret_layout/carousel.rs"]
mod carousel;
#[path = "web_vs_fret_layout/chart.rs"]
mod chart;
#[path = "web_vs_fret_layout/collapsible.rs"]
mod collapsible;
#[path = "web_vs_fret_layout/dashboard.rs"]
mod dashboard;
#[path = "web_vs_fret_layout/empty.rs"]
mod empty;
#[path = "web_vs_fret_layout/item.rs"]
mod item;
#[path = "web_vs_fret_layout/kbd.rs"]
mod kbd;
#[path = "web_vs_fret_layout/chart_scaffold.rs"]
mod layout_chart_scaffold_fixtures;
#[path = "web_vs_fret_layout/field.rs"]
mod layout_field_fixtures;
#[path = "web_vs_fret_layout/form.rs"]
mod layout_form_fixtures;
#[path = "web_vs_fret_layout/input.rs"]
mod layout_input_fixtures;
#[path = "web_vs_fret_layout/scroll.rs"]
mod layout_scroll_fixtures;
#[path = "web_vs_fret_layout/typography.rs"]
mod layout_typography_fixtures;
#[path = "web_vs_fret_layout/native_select.rs"]
mod native_select;
#[path = "web_vs_fret_layout/pagination.rs"]
mod pagination;
#[path = "web_vs_fret_layout/progress.rs"]
mod progress;
#[path = "web_vs_fret_layout/radio_group.rs"]
mod radio_group;
#[path = "web_vs_fret_layout/resizable.rs"]
mod resizable;
#[path = "web_vs_fret_layout/select.rs"]
mod select;
#[path = "web_vs_fret_layout/separator.rs"]
mod separator;
#[path = "web_vs_fret_layout/shell.rs"]
mod shell;
#[path = "web_vs_fret_layout/sidebar.rs"]
mod sidebar;
#[path = "web_vs_fret_layout/skeleton.rs"]
mod skeleton;
#[path = "web_vs_fret_layout/sonner.rs"]
mod sonner;
#[path = "web_vs_fret_layout/spinner.rs"]
mod spinner;
#[path = "web_vs_fret_layout/switch.rs"]
mod switch;
#[path = "web_vs_fret_layout/table.rs"]
mod table;
#[path = "web_vs_fret_layout/tabs.rs"]
mod tabs;
#[path = "web_vs_fret_layout/textarea.rs"]
mod textarea;
#[path = "web_vs_fret_layout/triggers.rs"]
mod triggers;

#[derive(Debug, Clone, Deserialize)]
struct WebGolden {
    themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebGoldenTheme {
    viewport: WebViewport,
    root: WebNode,
    #[serde(default)]
    portals: Vec<WebNode>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebViewport {
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebRect {
    #[allow(dead_code)]
    x: f32,
    #[allow(dead_code)]
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebNode {
    tag: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    #[serde(rename = "className")]
    class_name: Option<String>,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    #[serde(rename = "computedStyle")]
    computed_style: BTreeMap<String, String>,
    #[allow(dead_code)]
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    rect: WebRect,
    #[serde(default)]
    scroll: Option<WebScrollMetrics>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    children: Vec<WebNode>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebScrollMetrics {
    #[serde(rename = "scrollWidth")]
    scroll_width: f32,
    #[serde(rename = "scrollHeight")]
    scroll_height: f32,
    #[serde(rename = "clientWidth")]
    client_width: f32,
    #[serde(rename = "clientHeight")]
    client_height: f32,
    #[serde(rename = "offsetWidth")]
    #[allow(dead_code)]
    offset_width: f32,
    #[serde(rename = "offsetHeight")]
    #[allow(dead_code)]
    offset_height: f32,
    #[serde(rename = "scrollLeft")]
    scroll_left: f32,
    #[serde(rename = "scrollTop")]
    scroll_top: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn web_golden_path(name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(format!("{name}.json"))
}

fn read_web_golden(name: &str) -> WebGolden {
    let path = web_golden_path(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web golden: {}\nerror: {err}\n\nGenerate it via:\n  pnpm -C repo-ref/ui/apps/v4 golden:extract {name} --update\n\nDocs:\n  goldens/README.md\n  docs/shadcn-web-goldens.md",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "failed to parse web golden: {}\nerror: {err}",
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

fn find_all<'a>(node: &'a WebNode, pred: &impl Fn(&'a WebNode) -> bool) -> Vec<&'a WebNode> {
    let mut out = Vec::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if pred(n) {
            out.push(n);
        }
        for child in &n.children {
            stack.push(child);
        }
    }
    out
}

fn find_first_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    pred: &impl Fn(&'a WebNode) -> bool,
) -> Option<&'a WebNode> {
    find_first(&theme.root, pred).or_else(|| {
        theme
            .portals
            .iter()
            .find_map(|portal| find_first(portal, pred))
    })
}

fn find_all_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    pred: &impl Fn(&'a WebNode) -> bool,
) -> Vec<&'a WebNode> {
    let mut out = find_all(&theme.root, pred);
    for portal in &theme.portals {
        out.extend(find_all(portal, pred));
    }
    out
}

fn contains_text(node: &WebNode, needle: &str) -> bool {
    if node.text.as_deref().is_some_and(|t| t.contains(needle)) {
        return true;
    }
    node.children.iter().any(|c| contains_text(c, needle))
}

fn contains_id(node: &WebNode, needle: &str) -> bool {
    if node
        .id
        .as_deref()
        .or_else(|| node.attrs.get("id").map(String::as_str))
        .is_some_and(|id| id == needle)
    {
        return true;
    }
    node.children.iter().any(|c| contains_id(c, needle))
}

fn web_find_by_tag_and_text<'a>(root: &'a WebNode, tag: &str, text: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| n.tag == tag && contains_text(n, text))
}

fn web_find_badge_spans_with_spinner<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let tokens = &[
        "inline-flex",
        "items-center",
        "justify-center",
        "rounded-full",
        "px-2",
        "py-0.5",
        "text-xs",
        "gap-1",
        "overflow-hidden",
    ];

    let mut spans = find_all(root, &|n| {
        n.tag == "span" && class_has_all_tokens(n, tokens)
    });
    spans.retain(|span| {
        find_first(span, &|n| {
            n.tag == "svg" && class_has_token(n, "animate-spin")
        })
        .is_some()
    });
    spans.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    spans
}

fn web_find_by_data_slot<'a>(root: &'a WebNode, slot: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v == slot)
    })
}

fn web_find_scroll_area_scrollbar<'a>(root: &'a WebNode, orientation: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "scroll-area-scrollbar")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == orientation)
    })
}

fn web_find_scroll_area_thumb<'a>(root: &'a WebNode) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "scroll-area-thumb")
    })
}

fn web_find_scroll_area_thumb_in_scrollbar<'a>(scrollbar: &'a WebNode) -> Option<&'a WebNode> {
    web_find_scroll_area_thumb(scrollbar)
}

fn web_find_by_class_token<'a>(root: &'a WebNode, token: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| class_has_token(n, token))
}

fn web_find_by_class_token_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    token: &str,
) -> Option<&'a WebNode> {
    find_first_in_theme(theme, &|n| class_has_token(n, token))
}

fn class_has_token(node: &WebNode, token: &str) -> bool {
    node.class_name
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .any(|t| t == token)
}

fn class_has_all_tokens(node: &WebNode, tokens: &[&str]) -> bool {
    tokens.iter().all(|t| class_has_token(node, t))
}

fn web_find_by_class_tokens<'a>(root: &'a WebNode, tokens: &[&str]) -> Option<&'a WebNode> {
    find_first(root, &|n| class_has_all_tokens(n, tokens))
}

fn web_css_px(node: &WebNode, key: &str) -> Px {
    let raw = node
        .computed_style
        .get(key)
        .unwrap_or_else(|| panic!("missing computedStyle[{key:?}] for <{}>", node.tag));
    let s = raw.strip_suffix("px").unwrap_or(raw);
    Px(s.parse::<f32>().unwrap_or_else(|_| {
        panic!(
            "invalid computedStyle[{key:?}] value {raw:?} for <{}>",
            node.tag
        )
    }))
}

fn web_css_u16(node: &WebNode, key: &str) -> u16 {
    let raw = node
        .computed_style
        .get(key)
        .unwrap_or_else(|| panic!("missing computedStyle[{key:?}] for <{}>", node.tag));
    raw.parse::<u16>().unwrap_or_else(|_| {
        panic!(
            "invalid computedStyle[{key:?}] value {raw:?} for <{}>",
            node.tag
        )
    })
}

fn web_collect_all<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
    out.push(node);
    for child in &node.children {
        web_collect_all(child, out);
    }
}

fn web_collect_tag<'a>(node: &'a WebNode, tag: &str, out: &mut Vec<&'a WebNode>) {
    if node.tag == tag {
        out.push(node);
    }
    for child in &node.children {
        web_collect_tag(child, tag, out);
    }
}

fn web_collect_item_rows<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut items = find_all(root, &|n| {
        (n.tag == "div" || n.tag == "a") && class_has_token(n, "group/item")
    });
    items.sort_by(|a, b| {
        a.rect
            .y
            .total_cmp(&b.rect.y)
            .then_with(|| a.rect.x.total_cmp(&b.rect.x))
    });
    items
}

fn web_find_item_group<'a>(root: &'a WebNode) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.tag == "div" && class_has_token(n, "group/item-group")
    })
}

fn web_find_best_by<'a>(
    root: &'a WebNode,
    pred: &impl Fn(&'a WebNode) -> bool,
    score: &impl Fn(&'a WebNode) -> f32,
) -> Option<&'a WebNode> {
    let mut all = Vec::new();
    web_collect_all(root, &mut all);

    let mut best: Option<&WebNode> = None;
    let mut best_score = f32::INFINITY;
    let mut best_area = f32::INFINITY;
    for node in all.into_iter().filter(|n| pred(n)) {
        let s = score(node);
        if !s.is_finite() {
            continue;
        }
        let area = node.rect.w * node.rect.h;
        if s < best_score || (s == best_score && area < best_area) {
            best = Some(node);
            best_score = s;
            best_area = area;
        }
    }
    best
}

fn rect_contains(outer: WebRect, inner: WebRect) -> bool {
    let eps = 0.01;
    inner.x + eps >= outer.x
        && inner.y + eps >= outer.y
        && inner.x + inner.w <= outer.x + outer.w + eps
        && inner.y + inner.h <= outer.y + outer.h + eps
}

fn fret_rect_contains(outer: Rect, inner: Rect) -> bool {
    let eps = 0.01;
    inner.origin.x.0 + eps >= outer.origin.x.0
        && inner.origin.y.0 + eps >= outer.origin.y.0
        && inner.origin.x.0 + inner.size.width.0 <= outer.origin.x.0 + outer.size.width.0 + eps
        && inner.origin.y.0 + inner.size.height.0 <= outer.origin.y.0 + outer.size.height.0 + eps
}

#[derive(Debug, Clone, Copy)]
struct InsetQuad {
    left: f32,
    top_to_first_option: f32,
    right: f32,
    bottom_from_last_option: f32,
}

fn web_listbox_option_inset(theme: &WebGoldenTheme, listbox: &WebNode) -> InsetQuad {
    let mut all = Vec::new();
    web_collect_all(&theme.root, &mut all);

    let options: Vec<_> = all
        .into_iter()
        .filter(|n| n.attrs.get("role").is_some_and(|v| v == "option"))
        .filter(|n| rect_contains(listbox.rect, n.rect))
        .collect();

    if options.is_empty() {
        panic!("missing web listbox options");
    }

    let mut min_x = options[0].rect.x;
    let mut min_y = options[0].rect.y;
    let mut max_right = options[0].rect.x + options[0].rect.w;
    let mut max_bottom = options[0].rect.y + options[0].rect.h;
    for option in options.iter().skip(1) {
        min_x = min_x.min(option.rect.x);
        min_y = min_y.min(option.rect.y);
        max_right = max_right.max(option.rect.x + option.rect.w);
        max_bottom = max_bottom.max(option.rect.y + option.rect.h);
    }

    let panel_right = listbox.rect.x + listbox.rect.w;
    let panel_bottom = listbox.rect.y + listbox.rect.h;

    InsetQuad {
        left: min_x - listbox.rect.x,
        top_to_first_option: min_y - listbox.rect.y,
        right: panel_right - max_right,
        bottom_from_last_option: panel_bottom - max_bottom,
    }
}

fn fret_listbox_option_inset(snap: &fret_core::SemanticsSnapshot) -> InsetQuad {
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

fn assert_inset_quad_close(label: &str, actual: InsetQuad, expected: InsetQuad, tol: f32) {
    assert_close_px(
        &format!("{label} listbox left_inset"),
        Px(actual.left),
        expected.left,
        tol,
    );
    assert_close_px(
        &format!("{label} listbox top_to_first_option"),
        Px(actual.top_to_first_option),
        expected.top_to_first_option,
        tol,
    );
    assert_close_px(
        &format!("{label} listbox right_inset"),
        Px(actual.right),
        expected.right,
        tol,
    );
    assert_close_px(
        &format!("{label} listbox bottom_from_last_option"),
        Px(actual.bottom_from_last_option),
        expected.bottom_from_last_option,
        tol,
    );
}

fn web_find_smallest_container<'a>(root: &'a WebNode, nodes: &[&WebNode]) -> Option<&'a WebNode> {
    if nodes.is_empty() {
        return None;
    }

    let mut all = Vec::new();
    web_collect_all(root, &mut all);

    let mut best: Option<&WebNode> = None;
    let mut best_area = f32::INFINITY;
    for candidate in all {
        if nodes.iter().all(|n| rect_contains(candidate.rect, n.rect)) {
            let area = candidate.rect.w * candidate.rect.h;
            if area < best_area {
                best_area = area;
                best = Some(candidate);
            }
        }
    }
    best
}

fn assert_close_px(label: &str, actual: Px, expected: f32, tol: f32) {
    let delta = (actual.0 - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected≈{expected} (±{tol}) got={}",
        actual.0
    );
}

fn assert_rgba_close(label: &str, actual: Rgba, expected: Rgba, tol: f32) {
    let dr = (actual.r - expected.r).abs();
    let dg = (actual.g - expected.g).abs();
    let db = (actual.b - expected.b).abs();
    let da = (actual.a - expected.a).abs();
    assert!(
        dr <= tol && dg <= tol && db <= tol && da <= tol,
        "{label}: expected≈({:.3},{:.3},{:.3},{:.3}) got=({:.3},{:.3},{:.3},{:.3}) tol={tol}",
        expected.r,
        expected.g,
        expected.b,
        expected.a,
        actual.r,
        actual.g,
        actual.b,
        actual.a
    );
}

#[derive(Debug, Clone, Copy)]
struct PaintedQuad {
    rect: Rect,
    background: fret_core::Color,
}

fn find_best_background_quad(scene: &Scene, target: Rect) -> Option<PaintedQuad> {
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            continue;
        };

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        if score < best_score {
            best_score = score;
            best = Some(PaintedQuad { rect, background });
        }
    }

    best
}

fn find_best_opaque_background_quad(scene: &Scene, target: Rect) -> Option<PaintedQuad> {
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            continue;
        };

        if background.a <= 0.001 {
            continue;
        }

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        if score < best_score {
            best_score = score;
            best = Some(PaintedQuad { rect, background });
        }
    }

    best
}

struct CalendarRangeWebConfig {
    month: time::Month,
    year: i32,
    origin_x: f32,
    origin_y: f32,
    chrome_override: ChromeRefinement,
    cell_size: Px,
    week_start: time::Weekday,
    today: Option<time::Date>,
    show_week_number: bool,
    show_outside_days: bool,
    disable_outside_days: bool,
    range_min: time::Date,
    range_max: time::Date,
}

fn web_calendar_range_config(theme: &WebGoldenTheme) -> CalendarRangeWebConfig {
    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let origin_x = web_rdp_root.rect.x;
    let origin_y = web_rdp_root.rect.y;

    let padding_left = web_css_px(web_rdp_root, "paddingLeft");
    let border_left = web_css_px(web_rdp_root, "borderLeftWidth");

    let web_month_grid = find_first_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    })
    .expect("web month grid");
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();
    assert!(
        selected_dates.len() >= 3,
        "expected at least 3 selected dates for range mode"
    );

    let (range_min, range_max) = selected_dates
        .iter()
        .copied()
        .fold((selected_dates[0], selected_dates[0]), |(min, max), d| {
            (min.min(d), max.max(d))
        });

    let weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let show_outside_days =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-outside")).is_some();

    let disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let cell_size = parse_calendar_cell_size_px(theme).unwrap_or_else(|| {
        let sample = web_day_buttons[0];
        Px(sample.rect.w)
    });

    let mut chrome_override = ChromeRefinement::default();
    if (padding_left.0 - 0.0).abs() < 0.5 {
        chrome_override = chrome_override.p(Space::N0);
    } else if (padding_left.0 - 12.0).abs() < 0.5 {
        chrome_override = chrome_override.p(Space::N3);
    } else if (padding_left.0 - 8.0).abs() < 0.5 {
        chrome_override = chrome_override.p(Space::N2);
    }
    if border_left.0 >= 0.5 {
        chrome_override = chrome_override.border_1();
    }

    CalendarRangeWebConfig {
        month,
        year,
        origin_x,
        origin_y,
        chrome_override,
        cell_size,
        week_start,
        today,
        show_week_number,
        show_outside_days,
        disable_outside_days,
        range_min,
        range_max,
    }
}

fn render_fret_calendar_range_scene(
    config: &CalendarRangeWebConfig,
    viewport: WebViewport,
) -> Scene {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(viewport.w), Px(viewport.h)),
    );

    let (_snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

        let month_model: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(config.year, config.month));
        let selected: Model<DateRangeSelection> = cx.app.models_mut().insert(DateRangeSelection {
            from: Some(config.range_min),
            to: Some(config.range_max),
        });

        let mut calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
            .week_start(config.week_start)
            .show_outside_days(config.show_outside_days)
            .disable_outside_days(config.disable_outside_days)
            .show_week_number(config.show_week_number)
            .refine_style(config.chrome_override.clone())
            .cell_size(config.cell_size);

        if let Some(today) = config.today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(config.origin_x),
                    top: Px(config.origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    scene
}

fn assert_calendar_range_day_background_matches_web(
    web_name: &str,
    range_cell_class: &str,
    expected_label: &str,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let cell = find_first(&theme.root, &|n| class_has_token(n, range_cell_class))
        .unwrap_or_else(|| panic!("web missing {range_cell_class} day cell"));
    let button = find_first(cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| label.as_str() == expected_label)
    })
    .unwrap_or_else(|| {
        panic!("web missing {range_cell_class} day button label={expected_label:?}")
    });

    let web_bg_css = button
        .computed_style
        .get("backgroundColor")
        .expect("web day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let config = web_calendar_range_config(theme);
    let scene = render_fret_calendar_range_scene(&config, theme.viewport);

    let target = Rect::new(
        Point::new(Px(button.rect.x), Px(button.rect.y)),
        CoreSize::new(Px(button.rect.w), Px(button.rect.h)),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .unwrap_or_else(|| panic!("painted opaque {range_cell_class} day background quad"));

    assert_rect_xwh_close_px(
        &format!("{web_name} {range_cell_class} day quad"),
        quad.rect,
        button.rect,
        3.0,
    );
    assert_rgba_close(
        &format!("{web_name} {range_cell_class} day background"),
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

fn assert_rect_xwh_close_px(label: &str, actual: Rect, expected: WebRect, tol: f32) {
    assert_close_px(&format!("{label} x"), actual.origin.x, expected.x, tol);
    assert_close_px(&format!("{label} w"), actual.size.width, expected.w, tol);
    assert_close_px(&format!("{label} h"), actual.size.height, expected.h, tol);
}

fn collect_subtree_nodes(ui: &UiTree<App>, root: NodeId, out: &mut Vec<NodeId>) {
    out.push(root);
    for child in ui.children(root) {
        collect_subtree_nodes(ui, child, out);
    }
}

fn find_node_with_bounds_close(
    ui: &UiTree<App>,
    root: NodeId,
    expected: WebRect,
    tol: f32,
) -> Option<(NodeId, Rect)> {
    let mut nodes = Vec::new();
    collect_subtree_nodes(ui, root, &mut nodes);

    for id in nodes {
        let Some(bounds) = ui.debug_node_bounds(id) else {
            continue;
        };
        let close = (bounds.origin.x.0 - expected.x).abs() <= tol
            && (bounds.origin.y.0 - expected.y).abs() <= tol
            && (bounds.size.width.0 - expected.w).abs() <= tol
            && (bounds.size.height.0 - expected.h).abs() <= tol;
        if close {
            return Some((id, bounds));
        }
    }
    None
}

fn find_node_with_size_close(
    ui: &UiTree<App>,
    root: NodeId,
    expected_w: f32,
    expected_h: f32,
    tol: f32,
) -> Option<Rect> {
    let mut nodes = Vec::new();
    collect_subtree_nodes(ui, root, &mut nodes);

    let mut best: Option<Rect> = None;
    let mut best_score = f32::INFINITY;
    let mut best_area = f32::INFINITY;

    for id in nodes {
        let Some(bounds) = ui.debug_node_bounds(id) else {
            continue;
        };
        let dw = (bounds.size.width.0 - expected_w).abs();
        let dh = (bounds.size.height.0 - expected_h).abs();
        if dw > tol || dh > tol {
            continue;
        }

        let score = dw + dh;
        let area = bounds.size.width.0 * bounds.size.height.0;
        if score < best_score || (score == best_score && area < best_area) {
            best = Some(bounds);
            best_score = score;
            best_area = area;
        }
    }

    best
}

fn assert_rect_close_px(label: &str, actual: Rect, expected: WebRect, tol: f32) {
    assert_close_px(&format!("{label} x"), actual.origin.x, expected.x, tol);
    assert_close_px(&format!("{label} y"), actual.origin.y, expected.y, tol);
    assert_close_px(&format!("{label} w"), actual.size.width, expected.w, tol);
    assert_close_px(&format!("{label} h"), actual.size.height, expected.h, tol);
}

fn rect_close_px(actual: Rect, expected: WebRect, tol: f32) -> bool {
    (actual.origin.x.0 - expected.x).abs() <= tol
        && (actual.origin.y.0 - expected.y).abs() <= tol
        && (actual.size.width.0 - expected.w).abs() <= tol
        && (actual.size.height.0 - expected.h).abs() <= tol
}

fn overlap_area(a: Rect, b: Rect) -> f32 {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = ax0 + a.size.width.0;
    let ay1 = ay0 + a.size.height.0;

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = bx0 + b.size.width.0;
    let by1 = by0 + b.size.height.0;

    let x0 = ax0.max(bx0);
    let y0 = ay0.max(by0);
    let x1 = ax1.min(bx1);
    let y1 = ay1.min(by1);

    let w = (x1 - x0).max(0.0);
    let h = (y1 - y0).max(0.0);
    w * h
}

fn find_scene_quad_with_rect_close(scene: &Scene, expected: WebRect, tol: f32) -> Option<Rect> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            SceneOp::Quad { rect, .. } => Some(*rect),
            _ => None,
        })
        .find(|rect| rect_close_px(*rect, expected, tol))
}

fn find_scene_quad_background_with_rect_close(
    scene: &Scene,
    expected: WebRect,
    tol: f32,
) -> Option<(Rect, fret_core::Color)> {
    scene.ops().iter().find_map(|op| {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            return None;
        };
        if rect_close_px(rect, expected, tol) {
            Some((rect, background))
        } else {
            None
        }
    })
}

fn rect_aabb_after_transform(transform: Transform2D, rect: Rect) -> Rect {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;

    let p0 = transform.apply_point(Point::new(Px(x0), Px(y0)));
    let p1 = transform.apply_point(Point::new(Px(x1), Px(y0)));
    let p2 = transform.apply_point(Point::new(Px(x0), Px(y1)));
    let p3 = transform.apply_point(Point::new(Px(x1), Px(y1)));

    let min_x = p0.x.0.min(p1.x.0).min(p2.x.0).min(p3.x.0);
    let min_y = p0.y.0.min(p1.y.0).min(p2.y.0).min(p3.y.0);
    let max_x = p0.x.0.max(p1.x.0).max(p2.x.0).max(p3.x.0);
    let max_y = p0.y.0.max(p1.y.0).max(p2.y.0).max(p3.y.0);

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        CoreSize::new(Px(max_x - min_x), Px(max_y - min_y)),
    )
}

fn find_scene_quad_background_with_world_rect_close(
    scene: &Scene,
    expected: WebRect,
    tol: f32,
) -> Option<(Rect, fret_core::Color)> {
    let mut transform_stack: Vec<Transform2D> = vec![Transform2D::IDENTITY];

    for op in scene.ops() {
        match *op {
            SceneOp::PushTransform { transform } => {
                let current = *transform_stack.last().expect("transform stack not empty");
                transform_stack.push(current * transform);
            }
            SceneOp::PopTransform => {
                transform_stack.pop();
                debug_assert!(!transform_stack.is_empty(), "unbalanced PopTransform");
                if transform_stack.is_empty() {
                    transform_stack.push(Transform2D::IDENTITY);
                }
            }
            SceneOp::Quad {
                rect, background, ..
            } => {
                let current = *transform_stack.last().expect("transform stack not empty");
                let world_rect = rect_aabb_after_transform(current, rect);
                if rect_close_px(world_rect, expected, tol) {
                    return Some((world_rect, background));
                }
            }
            _ => {}
        }
    }

    None
}

fn rect_diff_metric(actual: Rect, expected: WebRect) -> f32 {
    (actual.origin.x.0 - expected.x).abs()
        + (actual.origin.y.0 - expected.y).abs()
        + (actual.size.width.0 - expected.w).abs()
        + (actual.size.height.0 - expected.h).abs()
}

fn rgba_diff_metric(actual: Rgba, expected: Rgba) -> f32 {
    (actual.r - expected.r).abs()
        + (actual.g - expected.g).abs()
        + (actual.b - expected.b).abs()
        + (actual.a - expected.a).abs()
}

fn debug_dump_scene_quads_near_expected(
    scene: &Scene,
    expected: WebRect,
    expected_bg: Option<Rgba>,
) {
    let mut transform_stack: Vec<Transform2D> = vec![Transform2D::IDENTITY];
    let mut quads: Vec<(f32, Rect, fret_core::Color, Transform2D)> = Vec::new();

    for op in scene.ops() {
        match *op {
            SceneOp::PushTransform { transform } => {
                let current = *transform_stack.last().expect("transform stack not empty");
                transform_stack.push(current * transform);
            }
            SceneOp::PopTransform => {
                transform_stack.pop();
                if transform_stack.is_empty() {
                    transform_stack.push(Transform2D::IDENTITY);
                }
            }
            SceneOp::Quad {
                rect, background, ..
            } => {
                let current = *transform_stack.last().expect("transform stack not empty");
                let world_rect = rect_aabb_after_transform(current, rect);
                let d = rect_diff_metric(world_rect, expected);
                quads.push((d, world_rect, background, current));
            }
            _ => {}
        }
    }

    quads.sort_by(|a, b| a.0.total_cmp(&b.0));

    eprintln!("--- debug_dump_scene_quads_near_expected ---");
    eprintln!(
        "expected rect: x={:.2} y={:.2} w={:.2} h={:.2}",
        expected.x, expected.y, expected.w, expected.h
    );
    if let Some(bg) = expected_bg {
        eprintln!(
            "expected bg (linear rgba): r={:.4} g={:.4} b={:.4} a={:.4}",
            bg.r, bg.g, bg.b, bg.a
        );
    }

    for (idx, (d, rect, bg, transform)) in quads.iter().take(12).enumerate() {
        let rgba = color_to_rgba(*bg);
        eprintln!(
            "#{idx:02} rectΔ={d:.2} rect=({:.2},{:.2},{:.2},{:.2}) bg=({:.4},{:.4},{:.4},{:.4}) transform(tx={:.2},ty={:.2},a={:.3},b={:.3},c={:.3},d={:.3})",
            rect.origin.x.0,
            rect.origin.y.0,
            rect.size.width.0,
            rect.size.height.0,
            rgba.r,
            rgba.g,
            rgba.b,
            rgba.a,
            transform.tx,
            transform.ty,
            transform.a,
            transform.b,
            transform.c,
            transform.d
        );
    }

    if let Some(expected_bg) = expected_bg {
        let mut by_color: Vec<(f32, Rect, fret_core::Color)> = quads
            .iter()
            .map(|(_d, rect, bg, _)| {
                (
                    rgba_diff_metric(color_to_rgba(*bg), expected_bg),
                    *rect,
                    *bg,
                )
            })
            .collect();
        by_color.sort_by(|a, b| a.0.total_cmp(&b.0));
        eprintln!("top 8 by bg color diff:");
        for (idx, (d, rect, bg)) in by_color.iter().take(8).enumerate() {
            let rgba = color_to_rgba(*bg);
            eprintln!(
                "#{idx:02} bgΔ={d:.4} rect=({:.2},{:.2},{:.2},{:.2}) bg=({:.4},{:.4},{:.4},{:.4})",
                rect.origin.x.0,
                rect.origin.y.0,
                rect.size.width.0,
                rect.size.height.0,
                rgba.r,
                rgba.g,
                rgba.b,
                rgba.a
            );
        }
    }
}

#[derive(Default)]
struct FakeServices;

impl fret_core::TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        (
            fret_core::TextBlobId::default(),
            fret_core::TextMetrics {
                size: CoreSize::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for FakeServices {
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

impl fret_core::SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
struct RecordedTextPrepare {
    text: String,
    style: fret_core::TextStyle,
    constraints: fret_core::TextConstraints,
}

#[derive(Default)]
struct StyleAwareServices {
    prepared: Vec<RecordedTextPrepare>,
}

impl fret_core::TextService for StyleAwareServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        let (text, style) = match input {
            fret_core::TextInput::Plain { text, style } => (text.as_ref(), style),
            fret_core::TextInput::Attributed { text, base, .. } => (text.as_ref(), base),
            _ => {
                debug_assert!(false, "unsupported TextInput variant");
                return (
                    fret_core::TextBlobId::default(),
                    fret_core::TextMetrics {
                        size: CoreSize::new(Px(0.0), Px(0.0)),
                        baseline: Px(0.0),
                    },
                );
            }
        };
        self.prepared.push(RecordedTextPrepare {
            text: text.to_string(),
            style: style.clone(),
            constraints,
        });

        let line_height = style
            .line_height
            .unwrap_or(Px((style.size.0 * 1.4).max(0.0)));

        let char_w = (style.size.0 * 0.55).max(1.0);
        let est_w = Px(char_w * text.chars().count() as f32);

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

fn run_fret_root(
    bounds: Rect,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> fret_core::SemanticsSnapshot {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot")
}

fn run_fret_root_frames(
    bounds: Rect,
    frames: usize,
    mut render: impl FnMut(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> fret_core::SemanticsSnapshot {
    assert!(frames > 0, "frames must be > 0");

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let mut root: Option<NodeId> = None;
    let mut snapshot: Option<fret_core::SemanticsSnapshot> = None;

    for frame in 0..frames {
        let root_node = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "web-vs-fret-layout",
            |cx| render(cx),
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        snapshot = ui.semantics_snapshot().cloned();

        // Runner-owned clocks are normally advanced by the platform event loop. In tests we
        // advance them explicitly so frame-lagged layout queries (ADR 1170) can settle.
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        let next_tick = fret_runtime::TickId(app.tick_id().0.saturating_add(1));
        app.set_frame_id(next_frame);
        app.set_tick_id(next_tick);
    }

    snapshot.expect("expected semantics snapshot")
}

fn run_fret_root_with_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> fret_core::SemanticsSnapshot {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        services,
        window,
        bounds,
        "web-vs-fret-layout",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, services, bounds, 1.0);

    ui.semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot")
}

fn run_fret_root_frames_with_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    frames: usize,
    mut render: impl FnMut(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> fret_core::SemanticsSnapshot {
    assert!(frames > 0, "frames must be > 0");

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut root: Option<NodeId> = None;
    let mut snapshot: Option<fret_core::SemanticsSnapshot> = None;

    for frame in 0..frames {
        let root_node = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            services,
            window,
            bounds,
            "web-vs-fret-layout",
            |cx| render(cx),
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, services, bounds, &mut scene, 1.0);

        snapshot = ui.semantics_snapshot().cloned();

        // Runner-owned clocks are normally advanced by the platform event loop. In tests we
        // advance them explicitly so frame-lagged layout queries (ADR 1170) can settle.
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        let next_tick = fret_runtime::TickId(app.tick_id().0.saturating_add(1));
        app.set_frame_id(next_frame);
        app.set_tick_id(next_tick);
    }

    snapshot.expect("expected semantics snapshot")
}

fn run_fret_root_with_ui(
    bounds: Rect,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (UiTree<App>, fret_core::SemanticsSnapshot, NodeId) {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    (ui, snap, root)
}

fn render_and_paint_in_bounds(
    bounds: Rect,
    render: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    // Use style-aware text metrics so painted/layout-derived geometry is comparable to web goldens.
    // `FakeServices` intentionally returns constant 10x10 text metrics and will distort layout.
    let mut services = StyleAwareServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    (snap, scene)
}

fn run_fret_root_with_ui_and_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (UiTree<App>, fret_core::SemanticsSnapshot, NodeId) {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        services,
        window,
        bounds,
        "web-vs-fret-layout",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    (ui, snap, root)
}

fn run_fret_root_frames_with_ui_and_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    frames: usize,
    mut render: impl FnMut(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (UiTree<App>, fret_core::SemanticsSnapshot, NodeId) {
    assert!(frames > 0, "frames must be > 0");

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut root: Option<NodeId> = None;
    let mut snapshot: Option<fret_core::SemanticsSnapshot> = None;

    for frame in 0..frames {
        let root_node = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            services,
            window,
            bounds,
            "web-vs-fret-layout",
            |cx| render(cx),
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, services, bounds, &mut scene, 1.0);

        snapshot = ui.semantics_snapshot().cloned();

        // Runner-owned clocks are normally advanced by the platform event loop. In tests we
        // advance them explicitly so frame-lagged layout queries (ADR 1170) can settle.
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        let next_tick = fret_runtime::TickId(app.tick_id().0.saturating_add(1));
        app.set_frame_id(next_frame);
        app.set_tick_id(next_tick);
    }

    let root = root.expect("expected root node");
    let snap = snapshot.expect("expected semantics snapshot");
    (ui, snap, root)
}

fn find_semantics<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
    label: Option<&str>,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes.iter().find(|n| {
        if n.role != role {
            return false;
        }
        if let Some(label) = label {
            return n.label.as_deref() == Some(label);
        }
        true
    })
}

fn assert_panel_x_w_match(web_name: &str, label: &str, fret: &Rect, web: WebRect, tol: f32) {
    assert_close_px(&format!("{web_name} {label} x"), fret.origin.x, web.x, tol);
    assert_close_px(
        &format!("{web_name} {label} w"),
        fret.size.width,
        web.w,
        tol,
    );
}

#[cfg(any())]
fn assert_shell_container_centered_x_w_matches(web_name: &str, tokens: &[&str]) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_container = web_find_by_class_tokens(&theme.root, tokens).expect("web shell container");
    let max_w = web_container.rect.w;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label: Arc<str> = Arc::from(format!("Golden:{web_name}:container"));
    let label_str: &str = &label;
    let snap = run_fret_root(bounds, |cx| {
        vec![cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().size_full().min_w_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(40.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            {
                let label = label.clone();
                move |cx| {
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(label.clone()),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                cx.container(
                                    ContainerProps {
                                        layout: decl_style::layout_style(
                                            &Theme::global(&*cx.app),
                                            LayoutRefinement::default()
                                                .w_px(MetricRef::Px(Px(max_w)))
                                                .min_w_0(),
                                        ),
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                ),
                            ]
                        },
                    )]
                }
            },
        )]
    });

    let fret_container =
        find_semantics(&snap, SemanticsRole::Panel, Some(label_str)).expect("fret container");
    assert_panel_x_w_match(
        web_name,
        "container",
        &fret_container.bounds,
        web_container.rect,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_login_01_shell_container_matches() {
    let web = read_web_golden("login-01");
    let theme = web_theme(&web);
    let web_container = web_find_by_class_tokens(&theme.root, &["w-full", "max-w-sm"])
        .expect("web max-w-sm container");
    let max_w = web_container.rect.w;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = "Golden:login-01:container";
    let snap = run_fret_root(bounds, |cx| {
        vec![cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().size_full().min_w_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(40.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                vec![cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        label: Some(Arc::from(label)),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            cx.container(
                                ContainerProps {
                                    layout: decl_style::layout_style(
                                        &Theme::global(&*cx.app),
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(max_w)))
                                            .min_w_0(),
                                    ),
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            ),
                        ]
                    },
                )]
            },
        )]
    });

    let fret_container =
        find_semantics(&snap, SemanticsRole::Panel, Some(label)).expect("fret container");
    assert_panel_x_w_match(
        "login-01",
        "container",
        &fret_container.bounds,
        web_container.rect,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_login_02_shell_container_matches() {
    let web = read_web_golden("login-02");
    let theme = web_theme(&web);
    let web_container =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-xs"]).expect("web container");
    let max_w = web_container.rect.w;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = "Golden:login-02:container";
    let col_w = theme.viewport.w / 2.0;
    let snap = run_fret_root(bounds, |cx| {
        let center = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().flex_1().min_w_0().min_h_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                vec![cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        label: Some(Arc::from(label)),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            cx.container(
                                ContainerProps {
                                    layout: decl_style::layout_style(
                                        &Theme::global(&*cx.app),
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(max_w)))
                                            .min_w_0(),
                                    ),
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            ),
                        ]
                    },
                )]
            },
        );

        let left = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(col_w)))
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                ),
                direction: fret_core::Axis::Vertical,
                gap: Px(16.0),
                padding: Edges::all(Px(40.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![center],
        );

        let right = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(col_w)))
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                ),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        vec![cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().size_full().min_w_0().min_h_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![left, right],
        )]
    });

    let fret_container =
        find_semantics(&snap, SemanticsRole::Panel, Some(label)).expect("fret container");
    assert_panel_x_w_match(
        "login-02",
        "container",
        &fret_container.bounds,
        web_container.rect,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_login_03_shell_container_matches() {
    assert_shell_container_centered_x_w_matches(
        "login-03",
        &["flex", "w-full", "max-w-sm", "flex-col", "gap-6"],
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_login_04_shell_container_matches() {
    assert_shell_container_centered_x_w_matches(
        "login-04",
        &["w-full", "max-w-sm", "md:max-w-4xl"],
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_login_05_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("login-05", &["w-full", "max-w-sm"]);
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_signup_01_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("signup-01", &["w-full", "max-w-sm"]);
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_signup_02_shell_container_matches() {
    let web = read_web_golden("signup-02");
    let theme = web_theme(&web);
    let web_container =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-xs"]).expect("web container");
    let max_w = web_container.rect.w;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = "Golden:signup-02:container";
    let col_w = theme.viewport.w / 2.0;
    let snap = run_fret_root(bounds, |cx| {
        let center = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().flex_1().min_w_0().min_h_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                vec![cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        label: Some(Arc::from(label)),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            cx.container(
                                ContainerProps {
                                    layout: decl_style::layout_style(
                                        &Theme::global(&*cx.app),
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(max_w)))
                                            .min_w_0(),
                                    ),
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            ),
                        ]
                    },
                )]
            },
        );

        let left = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(col_w)))
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                ),
                direction: fret_core::Axis::Vertical,
                gap: Px(16.0),
                padding: Edges::all(Px(40.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![center],
        );

        let right = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(col_w)))
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                ),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        vec![cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().size_full().min_w_0().min_h_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![left, right],
        )]
    });

    let fret_container =
        find_semantics(&snap, SemanticsRole::Panel, Some(label)).expect("fret container");
    assert_panel_x_w_match(
        "signup-02",
        "container",
        &fret_container.bounds,
        web_container.rect,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_signup_03_shell_container_matches() {
    assert_shell_container_centered_x_w_matches(
        "signup-03",
        &["flex", "w-full", "max-w-sm", "flex-col", "gap-6"],
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_signup_04_shell_container_matches() {
    assert_shell_container_centered_x_w_matches(
        "signup-04",
        &["w-full", "max-w-sm", "md:max-w-4xl"],
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_signup_05_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("signup-05", &["w-full", "max-w-sm"]);
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_otp_01_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("otp-01", &["w-full", "max-w-xs"]);
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_otp_02_shell_container_matches() {
    let web = read_web_golden("otp-02");
    let theme = web_theme(&web);
    let web_container =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-xs"]).expect("web container");
    let max_w = web_container.rect.w;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = "Golden:otp-02:container";
    let col_w = theme.viewport.w / 2.0;
    let snap = run_fret_root(bounds, |cx| {
        let center = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().flex_1().min_w_0().min_h_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                vec![cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        label: Some(Arc::from(label)),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            cx.container(
                                ContainerProps {
                                    layout: decl_style::layout_style(
                                        &Theme::global(&*cx.app),
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(max_w)))
                                            .min_w_0(),
                                    ),
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            ),
                        ]
                    },
                )]
            },
        );

        let left = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(col_w)))
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                ),
                direction: fret_core::Axis::Vertical,
                gap: Px(16.0),
                padding: Edges::all(Px(40.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![center],
        );

        let right = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(col_w)))
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                ),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        vec![cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().size_full().min_w_0().min_h_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![left, right],
        )]
    });

    let fret_container =
        find_semantics(&snap, SemanticsRole::Panel, Some(label)).expect("fret container");
    assert_panel_x_w_match(
        "otp-02",
        "container",
        &fret_container.bounds,
        web_container.rect,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_otp_03_shell_container_matches() {
    assert_shell_container_centered_x_w_matches(
        "otp-03",
        &["flex", "w-full", "max-w-xs", "flex-col", "gap-6"],
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_otp_04_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("otp-04", &["w-full", "max-w-sm", "md:max-w-3xl"]);
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_otp_05_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("otp-05", &["w-full", "max-w-sm"]);
}

#[cfg(any())]
fn web_find_sidebar_menu_button_by_height<'a>(
    root: &'a WebNode,
    height_token: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        (n.tag == "button" || n.tag == "a")
            && class_has_token(n, "peer/menu-button")
            && class_has_token(n, height_token)
    })
}

#[cfg(any())]
fn assert_sidebar_menu_button_heights_match_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_default = web_find_sidebar_menu_button_by_height(&theme.root, "h-8")
        .unwrap_or_else(|| panic!("missing web sidebar menu button (h-8) in {web_name}"));
    let web_lg = web_find_sidebar_menu_button_by_height(&theme.root, "h-12");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap_default = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::SidebarMenuButton::new("Sidebar Menu Button")
                .size(SidebarMenuButtonSize::Default)
                .into_element(cx),
        ]
    });

    let fret_default = find_semantics(
        &snap_default,
        SemanticsRole::Button,
        Some("Sidebar Menu Button"),
    )
    .or_else(|| find_semantics(&snap_default, SemanticsRole::Button, None))
    .expect("fret sidebar menu button (default) semantics node");

    assert_close_px(
        &format!("{web_name} menu button height (h-8)"),
        fret_default.bounds.size.height,
        web_default.rect.h,
        1.0,
    );

    if let Some(web_lg) = web_lg {
        let collapsed = (web_lg.rect.h - 32.0).abs() <= 1.0;
        let snap_lg = run_fret_root(bounds, |cx| {
            vec![
                fret_ui_shadcn::SidebarMenuButton::new("Sidebar Menu Button")
                    .size(SidebarMenuButtonSize::Lg)
                    .collapsed(collapsed)
                    .into_element(cx),
            ]
        });

        let fret_lg = find_semantics(&snap_lg, SemanticsRole::Button, Some("Sidebar Menu Button"))
            .or_else(|| find_semantics(&snap_lg, SemanticsRole::Button, None))
            .expect("fret sidebar menu button (lg) semantics node");

        assert_close_px(
            &format!("{web_name} menu button height (h-12)"),
            fret_lg.bounds.size.height,
            web_lg.rect.h,
            1.0,
        );
    }
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_01_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-01");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_02_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-02");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_03_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-03");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_04_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-04");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_05_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-05");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_06_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-06");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_07_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-07");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_08_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-08");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_09_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-09");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_10_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-10");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_11_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-11");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_12_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-12");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_14_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-14");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_15_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-15");
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_sidebar_16_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-16");
}

#[test]
fn web_vs_fret_layout_aspect_ratio_demo_geometry_matches() {
    let web = read_web_golden("aspect-ratio-demo");
    let theme = web_theme(&web);

    let web_img = find_first(&theme.root, &|n| n.tag == "img").expect("web img node");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, _snap, root) = run_fret_root_with_ui(bounds, |cx| {
        let child = cx.container(ContainerProps::default(), |_cx| Vec::new());
        vec![fret_ui_shadcn::AspectRatio::new(16.0 / 9.0, child).into_element(cx)]
    });

    let (_node, fret_bounds) = find_node_with_bounds_close(&ui, root, web_img.rect, 2.0)
        .expect("fret aspect ratio bounds close to web image rect");
    assert_rect_close_px("aspect-ratio-demo", fret_bounds, web_img.rect, 2.0);
}

#[test]
fn web_vs_fret_layout_checkbox_demo_control_size() {
    let web = read_web_golden("checkbox-demo");
    let theme = web_theme(&web);
    let web_checkbox = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "checkbox")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
    })
    .expect("web checkbox");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Checkbox::new(model)
                .a11y_label("Checkbox")
                .into_element(cx),
        ]
    });

    let checkbox = find_semantics(&snap, SemanticsRole::Checkbox, Some("Checkbox"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Checkbox, None))
        .expect("fret checkbox semantics node");

    assert_close_px(
        "checkbox width",
        checkbox.bounds.size.width,
        web_checkbox.rect.w,
        1.0,
    );
    assert_close_px(
        "checkbox height",
        checkbox.bounds.size.height,
        web_checkbox.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_label_demo_geometry() {
    let web = read_web_golden("label-demo");
    let theme = web_theme(&web);
    let web_checkbox = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "checkbox")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
    })
    .expect("web checkbox");
    let web_label = web_find_by_tag_and_text(&theme.root, "label", "Accept terms and conditions")
        .expect("web label");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<bool> = cx.app.models_mut().insert(false);
        let checkbox = fret_ui_shadcn::Checkbox::new(model)
            .a11y_label("Terms")
            .into_element(cx);
        let label = fret_ui_shadcn::Label::new("Accept terms and conditions").into_element(cx);
        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:label-demo:label")),
                ..Default::default()
            },
            move |_cx| vec![label],
        );

        let row = cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                gap: Px(8.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| vec![checkbox, label],
        );

        vec![row]
    });

    let checkbox = find_semantics(&snap, SemanticsRole::Checkbox, Some("Terms"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Checkbox, None))
        .expect("fret checkbox node");
    let label = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:label-demo:label"))
        .expect("fret label node");

    assert_close_px(
        "label-demo checkbox w",
        checkbox.bounds.size.width,
        web_checkbox.rect.w,
        1.0,
    );
    assert_close_px(
        "label-demo checkbox h",
        checkbox.bounds.size.height,
        web_checkbox.rect.h,
        1.0,
    );

    assert_close_px(
        "label-demo label x",
        label.bounds.origin.x,
        web_label.rect.x,
        1.0,
    );
    assert_close_px(
        "label-demo label y",
        label.bounds.origin.y,
        web_label.rect.y,
        1.0,
    );
    assert_close_px(
        "label-demo label h",
        label.bounds.size.height,
        web_label.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_checkbox_with_text_geometry() {
    let web = read_web_golden("checkbox-with-text");
    let theme = web_theme(&web);
    let web_checkbox = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "checkbox")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
    })
    .expect("web checkbox");
    let web_label = web_find_by_tag_and_text(&theme.root, "label", "Accept terms and conditions")
        .expect("web label");
    let web_desc =
        web_find_by_tag_and_text(&theme.root, "p", "Terms of Service").expect("web desc");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let model: Model<bool> = cx.app.models_mut().insert(false);

        let checkbox = fret_ui_shadcn::Checkbox::new(model)
            .a11y_label("Terms")
            .into_element(cx);

        let label = fret_ui_shadcn::Label::new("Accept terms and conditions").into_element(cx);
        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:checkbox-with-text:label")),
                ..Default::default()
            },
            move |_cx| vec![label],
        );

        let desc = cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from("You agree to our Terms of Service and Privacy Policy."),
            style: None,
            color: Some(theme.color_required("muted-foreground")),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });
        let desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:checkbox-with-text:desc")),
                ..Default::default()
            },
            move |_cx| vec![desc],
        );

        let content = cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Vertical,
                gap: Px(6.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![label, desc],
        );

        let row = cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(8.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![checkbox, content],
        );

        vec![row]
    });

    let checkbox = find_semantics(&snap, SemanticsRole::Checkbox, Some("Terms"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Checkbox, None))
        .expect("fret checkbox node");
    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:checkbox-with-text:label"),
    )
    .expect("fret label node");
    let desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:checkbox-with-text:desc"),
    )
    .expect("fret desc node");

    assert_close_px(
        "checkbox-with-text checkbox w",
        checkbox.bounds.size.width,
        web_checkbox.rect.w,
        1.0,
    );
    assert_close_px(
        "checkbox-with-text checkbox h",
        checkbox.bounds.size.height,
        web_checkbox.rect.h,
        1.0,
    );

    assert_close_px(
        "checkbox-with-text label x",
        label.bounds.origin.x,
        web_label.rect.x,
        1.0,
    );
    assert_close_px(
        "checkbox-with-text label y",
        label.bounds.origin.y,
        web_label.rect.y,
        1.0,
    );

    assert_close_px(
        "checkbox-with-text desc y",
        desc.bounds.origin.y,
        web_desc.rect.y,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_switch_demo_track_size() {
    let web = read_web_golden("switch-demo");
    let theme = web_theme(&web);
    let web_switch = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "switch")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
    })
    .expect("web switch");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Switch::new(model)
                .a11y_label("Switch")
                .into_element(cx),
        ]
    });

    let switch = find_semantics(&snap, SemanticsRole::Switch, Some("Switch"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Switch, None))
        .expect("fret switch semantics node");

    assert_close_px(
        "switch width",
        switch.bounds.size.width,
        web_switch.rect.w,
        1.0,
    );
    assert_close_px(
        "switch height",
        switch.bounds.size.height,
        web_switch.rect.h,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_switch_demo_thumb_geometry_matches_web() {
    fn overlap_area(a: Rect, b: Rect) -> f32 {
        let ax0 = a.origin.x.0;
        let ay0 = a.origin.y.0;
        let ax1 = ax0 + a.size.width.0;
        let ay1 = ay0 + a.size.height.0;

        let bx0 = b.origin.x.0;
        let by0 = b.origin.y.0;
        let bx1 = bx0 + b.size.width.0;
        let by1 = by0 + b.size.height.0;

        let x0 = ax0.max(bx0);
        let y0 = ay0.max(by0);
        let x1 = ax1.min(bx1);
        let y1 = ay1.min(by1);

        let w = (x1 - x0).max(0.0);
        let h = (y1 - y0).max(0.0);
        w * h
    }

    let web = read_web_golden("switch-demo");
    let theme = web_theme(&web);
    let web_switch = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "switch")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
    })
    .expect("web switch");
    let web_thumb = find_first(web_switch, &|n| {
        n.tag == "span"
            && n.attrs
                .get("data-state")
                .is_some_and(|state| state == "unchecked")
            && (n.rect.w - 16.0).abs() <= 0.2
            && (n.rect.h - 16.0).abs() <= 0.2
    })
    .expect("web switch thumb");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        let model: Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Switch::new(model)
                .a11y_label("Switch")
                .into_element(cx),
        ]
    });

    let switch = find_semantics(&snap, SemanticsRole::Switch, Some("Switch"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Switch, None))
        .expect("fret switch semantics");
    let switch_bounds = switch.bounds;

    let mut track_rect: Option<Rect> = None;
    let mut best_track_score = 0.0f32;
    let mut thumb_rect: Option<Rect> = None;
    let mut best_thumb_score = 0.0f32;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect, background, ..
        } = op
        else {
            continue;
        };

        // Ignore low-alpha shadow quads. The switch thumb/track are fully opaque in shadcn-web.
        if background.a < 0.5 {
            continue;
        }

        let score = overlap_area(*rect, switch_bounds);
        if score <= 0.0 {
            continue;
        }

        let is_track = (rect.size.width.0 - switch_bounds.size.width.0).abs() <= 1.0
            && (rect.size.height.0 - switch_bounds.size.height.0).abs() <= 1.0;
        if is_track && score > best_track_score {
            best_track_score = score;
            track_rect = Some(*rect);
        }

        let is_thumb =
            (rect.size.width.0 - 16.0).abs() <= 0.2 && (rect.size.height.0 - 16.0).abs() <= 0.2;
        if is_thumb && score > best_thumb_score {
            best_thumb_score = score;
            thumb_rect = Some(*rect);
        }
    }

    let track = track_rect.expect("missing switch track quad");
    let thumb = thumb_rect.expect("missing switch thumb quad");

    let expected_dx = web_thumb.rect.x - web_switch.rect.x;
    let expected_dy = web_thumb.rect.y - web_switch.rect.y;

    assert_close_px(
        "switch thumb offset x",
        Px(thumb.origin.x.0 - track.origin.x.0),
        expected_dx,
        1.0,
    );
    assert_close_px(
        "switch thumb offset y",
        Px(thumb.origin.y.0 - track.origin.y.0),
        expected_dy,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_radio_group_demo_row_geometry() {
    let web = read_web_golden("radio-group-demo");
    let theme = web_theme(&web);

    let mut rows: Vec<&WebNode> = Vec::new();
    let mut stack = vec![&theme.root];
    while let Some(node) = stack.pop() {
        let class_name = node.class_name.as_deref().unwrap_or_default();
        if node.tag == "div"
            && class_name.contains("flex")
            && class_name.contains("items-center")
            && class_name.contains("gap-3")
            && node
                .children
                .iter()
                .any(|c| c.attrs.get("role").is_some_and(|role| role == "radio"))
        {
            rows.push(node);
        }

        for child in node.children.iter().rev() {
            stack.push(child);
        }
    }

    rows.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert!(
        rows.len() >= 2,
        "expected at least two radio-group rows in web golden"
    );

    let web_row0 = rows[0];
    let web_row1 = rows[1];

    let web_row_h = web_row0.rect.h;
    let web_gap_y = web_row1.rect.y - (web_row0.rect.y + web_row0.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::RadioGroupItem::new("default", "Default"),
            fret_ui_shadcn::RadioGroupItem::new("comfortable", "Comfortable"),
            fret_ui_shadcn::RadioGroupItem::new("compact", "Compact"),
        ];

        let group = items.into_iter().fold(
            fret_ui_shadcn::RadioGroup::uncontrolled(Some("comfortable")).a11y_label("Options"),
            |group, item| group.item(item),
        );

        vec![group.into_element(cx)]
    });

    let fret_row0 = find_semantics(&snap, SemanticsRole::RadioButton, Some("Default"))
        .expect("fret radio row Default");
    let fret_row1 = find_semantics(&snap, SemanticsRole::RadioButton, Some("Comfortable"))
        .expect("fret radio row Comfortable");

    let fret_row_h = fret_row0.bounds.size.height.0;
    let fret_gap_y = fret_row1.bounds.origin.y.0
        - (fret_row0.bounds.origin.y.0 + fret_row0.bounds.size.height.0);

    assert!(
        fret_gap_y.is_finite(),
        "expected finite fret gap; got={fret_gap_y}"
    );

    assert_close_px("radio-group row height", Px(fret_row_h), web_row_h, 1.0);
    assert_close_px("radio-group row gap", Px(fret_gap_y), web_gap_y, 1.0);
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_radio_group_demo_indicator_geometry_matches_web() {
    fn overlap_area(a: Rect, b: Rect) -> f32 {
        let ax0 = a.origin.x.0;
        let ay0 = a.origin.y.0;
        let ax1 = ax0 + a.size.width.0;
        let ay1 = ay0 + a.size.height.0;

        let bx0 = b.origin.x.0;
        let by0 = b.origin.y.0;
        let bx1 = bx0 + b.size.width.0;
        let by1 = by0 + b.size.height.0;

        let x0 = ax0.max(bx0);
        let y0 = ay0.max(by0);
        let x1 = ax1.min(bx1);
        let y1 = ay1.min(by1);

        let w = (x1 - x0).max(0.0);
        let h = (y1 - y0).max(0.0);
        w * h
    }

    let web = read_web_golden("radio-group-demo");
    let theme = web_theme(&web);
    let web_radio = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "radio")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "true")
    })
    .expect("web checked radio");
    let web_indicator = find_first(web_radio, &|n| {
        n.tag == "svg" && (n.rect.w - 8.0).abs() <= 0.2
    })
    .expect("web radio indicator svg");

    let expected_dx = web_indicator.rect.x - web_radio.rect.x;
    let expected_dy = web_indicator.rect.y - web_radio.rect.y;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::RadioGroupItem::new("default", "Default"),
            fret_ui_shadcn::RadioGroupItem::new("comfortable", "Comfortable"),
            fret_ui_shadcn::RadioGroupItem::new("compact", "Compact"),
        ];

        let group = items.into_iter().fold(
            fret_ui_shadcn::RadioGroup::uncontrolled(Some("comfortable")).a11y_label("Options"),
            |group, item| group.item(item),
        );

        vec![group.into_element(cx)]
    });

    let row = find_semantics(&snap, SemanticsRole::RadioButton, Some("Comfortable"))
        .expect("fret comfortable row");
    let row_bounds = row.bounds;

    let mut icon_rect: Option<Rect> = None;
    let mut best_icon_score = 0.0f32;
    let mut dot_rect: Option<Rect> = None;
    let mut best_dot_score = 0.0f32;

    for op in scene.ops() {
        let SceneOp::Quad { rect, .. } = op else {
            continue;
        };

        let score = overlap_area(*rect, row_bounds);
        if score <= 0.0 {
            continue;
        }

        let is_icon =
            (rect.size.width.0 - 16.0).abs() <= 0.2 && (rect.size.height.0 - 16.0).abs() <= 0.2;
        if is_icon && score > best_icon_score {
            best_icon_score = score;
            icon_rect = Some(*rect);
        }
    }

    let icon = icon_rect.expect("missing radio icon quad");

    for op in scene.ops() {
        let SceneOp::Quad { rect, .. } = op else {
            continue;
        };

        let score_icon = overlap_area(*rect, icon);
        if score_icon <= 0.0 {
            continue;
        }

        let is_dot =
            (rect.size.width.0 - 8.0).abs() <= 0.2 && (rect.size.height.0 - 8.0).abs() <= 0.2;
        if is_dot && score_icon > best_dot_score {
            best_dot_score = score_icon;
            dot_rect = Some(*rect);
        }
    }

    let dot = dot_rect.expect("missing radio indicator dot quad");

    assert_close_px(
        "radio indicator offset x",
        Px(dot.origin.x.0 - icon.origin.x.0),
        expected_dx,
        1.0,
    );
    assert_close_px(
        "radio indicator offset y",
        Px(dot.origin.y.0 - icon.origin.y.0),
        expected_dy,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_slider_demo_geometry() {
    let web = read_web_golden("slider-demo");
    let theme = web_theme(&web);
    let web_thumb = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "slider")
    })
    .expect("web slider thumb");

    let thumb_center_y = web_thumb.rect.y + web_thumb.rect.h * 0.5;
    let web_track = web_find_best_by(
        &theme.root,
        &|n| {
            n.tag == "span"
                && n.attrs
                    .get("data-orientation")
                    .is_some_and(|v| v == "horizontal")
                && class_has_token(n, "bg-muted")
                && class_has_token(n, "rounded-full")
                && (n.rect.h - 6.0).abs() <= 0.1
        },
        &|n| ((n.rect.y + n.rect.h * 0.5) - thumb_center_y).abs(),
    )
    .expect("web slider track");

    let web_range = web_find_best_by(
        &theme.root,
        &|n| {
            n.tag == "span"
                && n.attrs
                    .get("data-orientation")
                    .is_some_and(|v| v == "horizontal")
                && class_has_token(n, "bg-primary")
                && class_has_token(n, "absolute")
                && (n.rect.h - 6.0).abs() <= 0.1
        },
        &|n| ((n.rect.y + n.rect.h * 0.5) - thumb_center_y).abs(),
    )
    .expect("web slider range");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let t = (web_thumb.rect.x + web_thumb.rect.w * 0.5) / web_track.rect.w.max(1.0);
    let initial_value = 100.0 * t.clamp(0.0, 1.0);

    let (ui, snap, _root) = run_fret_root_with_ui(bounds, |cx| {
        let model: Model<Vec<f32>> = cx.app.models_mut().insert(vec![initial_value]);
        let slider = fret_ui_shadcn::Slider::new(model)
            .range(0.0, 100.0)
            .a11y_label("Slider")
            .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_track.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![slider],
        )]
    });

    let thumb = find_semantics(&snap, SemanticsRole::Slider, Some("Slider"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Slider, None))
        .expect("fret slider thumb semantics");
    let slider = thumb
        .parent
        .and_then(|parent| snap.nodes.iter().find(|n| n.id == parent))
        .unwrap_or(thumb);

    assert_close_px(
        "slider layout width",
        slider.bounds.size.width,
        web_track.rect.w,
        1.0,
    );
    assert_close_px(
        "slider layout height",
        slider.bounds.size.height,
        web_track.rect.h,
        1.0,
    );

    let mut stack = vec![slider.id];
    let mut rects: Vec<(NodeId, Rect)> = Vec::new();
    while let Some(node) = stack.pop() {
        if let Some(bounds) = ui.debug_node_bounds(node) {
            rects.push((node, bounds));
        }
        for child in ui.children(node).into_iter().rev() {
            stack.push(child);
        }
    }

    let pick_best = |label: &str, expected: WebRect, rects: &[(NodeId, Rect)]| -> Rect {
        let mut best: Option<Rect> = None;
        let mut best_score = f32::INFINITY;
        for (_, rect) in rects {
            let score = (rect.origin.x.0 - expected.x).abs()
                + (rect.origin.y.0 - expected.y).abs()
                + (rect.size.width.0 - expected.w).abs()
                + (rect.size.height.0 - expected.h).abs();
            if score < best_score {
                best_score = score;
                best = Some(*rect);
            }
        }
        best.unwrap_or_else(|| panic!("missing {label} match"))
    };

    let fret_track = pick_best("track", web_track.rect, &rects);
    let fret_range = pick_best("range", web_range.rect, &rects);
    let fret_thumb = pick_best("thumb", web_thumb.rect, &rects);

    assert_close_px("track x", fret_track.origin.x, web_track.rect.x, 1.0);
    assert_close_px("track y", fret_track.origin.y, web_track.rect.y, 1.0);
    assert_close_px("track w", fret_track.size.width, web_track.rect.w, 1.0);
    assert_close_px("track h", fret_track.size.height, web_track.rect.h, 1.0);

    assert_close_px("range x", fret_range.origin.x, web_range.rect.x, 1.0);
    assert_close_px("range y", fret_range.origin.y, web_range.rect.y, 1.0);
    assert_close_px("range w", fret_range.size.width, web_range.rect.w, 1.0);
    assert_close_px("range h", fret_range.size.height, web_range.rect.h, 1.0);

    assert_close_px("thumb x", fret_thumb.origin.x, web_thumb.rect.x, 1.0);
    assert_close_px("thumb y", fret_thumb.origin.y, web_thumb.rect.y, 1.0);
    assert_close_px("thumb w", fret_thumb.size.width, web_thumb.rect.w, 1.0);
    assert_close_px("thumb h", fret_thumb.size.height, web_thumb.rect.h, 1.0);
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_textarea_demo_geometry() {
    let web = read_web_golden("textarea-demo");
    let theme = web_theme(&web);
    let web_textarea = find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::Textarea::new(model)
                .a11y_label("Textarea")
                .into_element(cx),
        ]
    });

    let textarea = find_semantics(&snap, SemanticsRole::TextField, Some("Textarea"))
        .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
        .expect("fret textarea semantics node");

    assert_close_px(
        "textarea width",
        textarea.bounds.size.width,
        web_textarea.rect.w,
        1.0,
    );
    assert_close_px(
        "textarea height",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_textarea_disabled_geometry_matches_web() {
    let web = read_web_golden("textarea-disabled");
    let theme = web_theme(&web);
    let web_textarea = find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::Textarea::new(model)
                .a11y_label("Textarea")
                .disabled(true)
                .into_element(cx),
        ]
    });

    let textarea = find_semantics(&snap, SemanticsRole::TextField, Some("Textarea"))
        .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
        .expect("fret textarea semantics node");

    assert_close_px(
        "textarea-disabled x",
        textarea.bounds.origin.x,
        web_textarea.rect.x,
        1.0,
    );
    assert_close_px(
        "textarea-disabled y",
        textarea.bounds.origin.y,
        web_textarea.rect.y,
        1.0,
    );
    assert_close_px(
        "textarea-disabled w",
        textarea.bounds.size.width,
        web_textarea.rect.w,
        1.0,
    );
    assert_close_px(
        "textarea-disabled h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_textarea_with_button_geometry_matches_web() {
    let web = read_web_golden("textarea-with-button");
    let theme = web_theme(&web);
    let web_textarea = find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");
    let web_button = find_first(&theme.root, &|n| n.tag == "button").expect("web button");
    let gap = web_button.rect.y - (web_textarea.rect.y + web_textarea.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let textarea = fret_ui_shadcn::Textarea::new(model)
            .a11y_label("Textarea")
            .into_element(cx);
        let button = fret_ui_shadcn::Button::new("Send message")
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Vertical,
                gap: Px(gap),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![textarea, button],
        )]
    });

    let textarea = find_semantics(&snap, SemanticsRole::TextField, Some("Textarea"))
        .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
        .expect("fret textarea semantics node");
    let button = find_semantics(&snap, SemanticsRole::Button, Some("Send message"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button semantics node");

    assert_close_px(
        "textarea-with-button textarea x",
        textarea.bounds.origin.x,
        web_textarea.rect.x,
        1.0,
    );
    assert_close_px(
        "textarea-with-button textarea y",
        textarea.bounds.origin.y,
        web_textarea.rect.y,
        1.0,
    );
    assert_close_px(
        "textarea-with-button textarea w",
        textarea.bounds.size.width,
        web_textarea.rect.w,
        1.0,
    );
    assert_close_px(
        "textarea-with-button textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );

    assert_close_px(
        "textarea-with-button button x",
        button.bounds.origin.x,
        web_button.rect.x,
        1.0,
    );
    assert_close_px(
        "textarea-with-button button y",
        button.bounds.origin.y,
        web_button.rect.y,
        1.0,
    );
    assert_close_px(
        "textarea-with-button button w",
        button.bounds.size.width,
        web_button.rect.w,
        1.0,
    );
    assert_close_px(
        "textarea-with-button button h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_textarea_with_label_geometry_matches_web() {
    let web = read_web_golden("textarea-with-label");
    let theme = web_theme(&web);
    let web_label = find_first(&theme.root, &|n| n.tag == "label").expect("web label");
    let web_textarea = find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");
    let gap = web_textarea.rect.y - (web_label.rect.y + web_label.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let label = fret_ui_shadcn::Label::new("Your message").into_element(cx);
        let textarea = fret_ui_shadcn::Textarea::new(model)
            .a11y_label("Textarea")
            .into_element(cx);

        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Vertical,
                gap: Px(gap),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![label, textarea],
        )]
    });

    let textarea = find_semantics(&snap, SemanticsRole::TextField, Some("Textarea"))
        .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
        .expect("fret textarea semantics node");

    assert_close_px(
        "textarea-with-label textarea x",
        textarea.bounds.origin.x,
        web_textarea.rect.x,
        1.0,
    );
    assert_close_px(
        "textarea-with-label textarea y",
        textarea.bounds.origin.y,
        web_textarea.rect.y,
        1.0,
    );
    assert_close_px(
        "textarea-with-label textarea w",
        textarea.bounds.size.width,
        web_textarea.rect.w,
        1.0,
    );
    assert_close_px(
        "textarea-with-label textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_textarea_with_text_geometry_matches_web() {
    let web = read_web_golden("textarea-with-text");
    let theme = web_theme(&web);
    let web_textarea = find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");
    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web text");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let label = fret_ui_shadcn::Label::new("Your Message").into_element(cx);
        let textarea = fret_ui_shadcn::Textarea::new(model)
            .a11y_label("Textarea")
            .into_element(cx);
        let helper = ui::text(cx, "Your message will be copied to the support team.")
            .text_size_px(theme.metric_required("font.size"))
            .line_height_px(theme.metric_required("font.line_height"))
            .font_normal()
            .into_element(cx);
        let helper = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:textarea-with-text:helper")),
                ..Default::default()
            },
            move |_cx| vec![helper],
        );

        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Vertical,
                gap: Px(12.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![label, textarea, helper],
        )]
    });

    let textarea = find_semantics(&snap, SemanticsRole::TextField, Some("Textarea"))
        .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
        .expect("fret textarea semantics node");
    let helper = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:textarea-with-text:helper"),
    )
    .expect("fret helper wrapper");

    assert_close_px(
        "textarea-with-text textarea y",
        textarea.bounds.origin.y,
        web_textarea.rect.y,
        1.0,
    );
    assert_close_px(
        "textarea-with-text textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );

    assert_close_px(
        "textarea-with-text helper y",
        helper.bounds.origin.y,
        web_p.rect.y,
        1.0,
    );
    assert_close_px(
        "textarea-with-text helper h",
        helper.bounds.size.height,
        web_p.rect.h,
        1.0,
    );
}

#[cfg(any())]
#[test]
fn web_vs_fret_layout_empty_demo_geometry_matches_web() {
    let web = read_web_golden("empty-demo");
    let theme = web_theme(&web);

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

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_frames_with_services(bounds, &mut services, 2, |cx| {
        use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

        let icon =
            decl_icon::icon_with(cx, fret_icons::ids::ui::CHEVRON_DOWN, Some(Px(24.0)), None);
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

        let root = fret_ui_shadcn::Empty::new(vec![header, content, learn_more]).into_element(cx);
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:empty-demo:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:empty-demo:root"))
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

#[cfg(any())]
#[test]
fn web_vs_fret_layout_empty_background_geometry_matches_web() {
    let web = read_web_golden("empty-background");
    let theme = web_theme(&web);

    let web_empty = web_find_by_class_tokens(
        &theme.root,
        &["bg-gradient-to-b", "from-muted/50", "border-dashed"],
    )
    .expect("web empty root");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_frames_with_services(bounds, &mut services, 2, |cx| {
        let icon =
            decl_icon::icon_with(cx, fret_icons::ids::ui::CHEVRON_DOWN, Some(Px(24.0)), None);
        let media = EmptyMedia::new(vec![icon])
            .variant(EmptyMediaVariant::Icon)
            .into_element(cx);

        let title = EmptyTitle::new("No Notifications").into_element(cx);
        let desc =
            EmptyDescription::new("You're all caught up. New notifications will appear here.")
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

#[cfg(any())]
#[test]
fn web_vs_fret_layout_empty_outline_geometry_matches_web() {
    let web = read_web_golden("empty-outline");
    let theme = web_theme(&web);

    let web_empty = web_find_by_class_tokens(
        &theme.root,
        &["border-dashed", "border", "gap-6", "rounded-lg"],
    )
    .expect("web empty root");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_frames_with_services(bounds, &mut services, 2, |cx| {
        let icon =
            decl_icon::icon_with(cx, fret_icons::ids::ui::CHEVRON_DOWN, Some(Px(24.0)), None);
        let media = EmptyMedia::new(vec![icon])
            .variant(EmptyMediaVariant::Icon)
            .into_element(cx);

        let title = EmptyTitle::new("Cloud Storage Empty").into_element(cx);
        let desc =
            EmptyDescription::new("Upload files to your cloud storage to access them anywhere.")
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

    let snap = run_fret_root_frames(bounds, 2, |cx| {
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

#[test]
fn web_vs_fret_layout_separator_demo_geometry() {
    let web = read_web_golden("separator-demo");
    let theme = web_theme(&web);
    let web_h = find_first(&theme.root, &|n| {
        n.class_name
            .as_deref()
            .is_some_and(|c| c.contains("bg-border shrink-0"))
            && n.attrs
                .get("data-orientation")
                .is_some_and(|o| o == "horizontal")
    })
    .expect("web horizontal separator");
    let web_v = find_first(&theme.root, &|n| {
        n.class_name
            .as_deref()
            .is_some_and(|c| c.contains("bg-border shrink-0"))
            && n.attrs
                .get("data-orientation")
                .is_some_and(|o| o == "vertical")
    })
    .expect("web vertical separator");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, snap, _root) = run_fret_root_with_ui(bounds, |cx| {
        let horizontal = fret_ui_shadcn::Separator::new()
            .orientation(fret_ui_shadcn::SeparatorOrientation::Horizontal)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

        let vertical = fret_ui_shadcn::Separator::new()
            .orientation(fret_ui_shadcn::SeparatorOrientation::Vertical)
            .into_element(cx);

        vec![cx.column(
            ColumnProps {
                align: CrossAlign::Start,
                ..Default::default()
            },
            |cx| {
                vec![
                    cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:separator-demo:horizontal")),
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Px(Px(web_h.rect.w)),
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |_cx| vec![horizontal],
                    ),
                    cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:separator-demo:vertical")),
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Auto,
                                    height: Length::Px(Px(web_v.rect.h)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |_cx| vec![vertical],
                    ),
                ]
            },
        )]
    });

    let fret_h = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:separator-demo:horizontal"),
    )
    .expect("fret horizontal separator root");
    let fret_h_child = ui
        .children(fret_h.id)
        .into_iter()
        .next()
        .expect("fret horizontal separator child");
    let fret_h_child_bounds = ui
        .debug_node_bounds(fret_h_child)
        .expect("fret horizontal separator child bounds");
    assert_close_px(
        "separator horizontal inner h",
        fret_h_child_bounds.size.height,
        web_h.rect.h,
        1.0,
    );
    assert_close_px(
        "separator horizontal w",
        fret_h.bounds.size.width,
        web_h.rect.w,
        1.0,
    );
    assert_close_px(
        "separator horizontal h",
        fret_h.bounds.size.height,
        web_h.rect.h,
        1.0,
    );

    let fret_v = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:separator-demo:vertical"),
    )
    .expect("fret vertical separator root");
    let fret_v_child = ui
        .children(fret_v.id)
        .into_iter()
        .next()
        .expect("fret vertical separator child");
    let fret_v_child_bounds = ui
        .debug_node_bounds(fret_v_child)
        .expect("fret vertical separator child bounds");
    assert_close_px(
        "separator vertical inner w",
        fret_v_child_bounds.size.width,
        web_v.rect.w,
        1.0,
    );
    assert_close_px(
        "separator vertical w",
        fret_v.bounds.size.width,
        web_v.rect.w,
        1.0,
    );
    assert_close_px(
        "separator vertical h",
        fret_v.bounds.size.height,
        web_v.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_breadcrumb_separator_geometry() {
    let web = read_web_golden("breadcrumb-separator");
    let theme = web_theme(&web);

    let mut svgs: Vec<&WebNode> = Vec::new();
    web_collect_tag(&theme.root, "svg", &mut svgs);
    let mut slashes: Vec<&WebNode> = svgs
        .into_iter()
        .filter(|n| class_has_token(n, "lucide-slash"))
        .collect();
    slashes.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert!(
        slashes.len() >= 2,
        "expected at least 2 slashes in breadcrumb-separator web golden"
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, _snap, root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_shadcn::breadcrumb::primitives as bc;

        vec![bc::Breadcrumb::new().into_element(cx, |cx| {
            vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                vec![
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                    }),
                    bc::BreadcrumbSeparator::new()
                        .kind(bc::BreadcrumbSeparatorKind::Slash)
                        .into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![bc::BreadcrumbLink::new("Components").into_element(cx)]
                    }),
                ]
            })]
        })]
    });

    let mut stack = vec![root];
    let mut rects: Vec<Rect> = Vec::new();
    while let Some(node) = stack.pop() {
        if let Some(bounds) = ui.debug_node_bounds(node) {
            rects.push(bounds);
        }
        for child in ui.children(node).into_iter().rev() {
            stack.push(child);
        }
    }

    let pick_best_by_size = |label: &str, expected: WebRect, rects: &[Rect]| -> Rect {
        let mut best: Option<Rect> = None;
        let mut best_score = f32::INFINITY;
        for rect in rects {
            let score =
                (rect.size.width.0 - expected.w).abs() + (rect.size.height.0 - expected.h).abs();
            if score < best_score {
                best_score = score;
                best = Some(*rect);
            }
        }
        best.unwrap_or_else(|| panic!("missing {label} match"))
    };

    for (i, web_slash) in slashes.iter().take(2).enumerate() {
        let fret_slash = pick_best_by_size("slash", web_slash.rect, &rects);
        assert_close_px(
            &format!("breadcrumb-separator slash[{i}] w"),
            fret_slash.size.width,
            web_slash.rect.w,
            1.0,
        );
        assert_close_px(
            &format!("breadcrumb-separator slash[{i}] h"),
            fret_slash.size.height,
            web_slash.rect.h,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_breadcrumb_link_geometry() {
    let web = read_web_golden("breadcrumb-link");
    let theme = web_theme(&web);

    let web_home = web_find_by_tag_and_text(&theme.root, "a", "Home").expect("web Home link");
    let web_components =
        web_find_by_tag_and_text(&theme.root, "a", "Components").expect("web Components link");
    let web_page = find_first(&theme.root, &|n| n.text.as_deref() == Some("Breadcrumb"))
        .expect("web Breadcrumb page text");

    let mut svgs: Vec<&WebNode> = Vec::new();
    web_collect_tag(&theme.root, "svg", &mut svgs);
    let mut chevrons: Vec<&WebNode> = svgs
        .into_iter()
        .filter(|n| class_has_token(n, "lucide-chevron-right"))
        .collect();
    chevrons.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert!(
        chevrons.len() >= 2,
        "expected at least 2 chevrons in breadcrumb-link web golden"
    );

    let web_chevron0 = chevrons[0];
    let web_chevron1 = chevrons[1];

    let expected_chevron0_offset_y = web_chevron0.rect.y - web_home.rect.y;
    let expected_chevron1_offset_y = web_chevron1.rect.y - web_components.rect.y;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, snap, _root) = {
        let mut services = StyleAwareServices::default();
        run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;

            vec![bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    let label = |s: &'static str| Some(Arc::from(s));

                    let home = bc::BreadcrumbLink::new("Home").into_element(cx);
                    let home = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: label("Golden:breadcrumb-link:home"),
                            ..Default::default()
                        },
                        move |_cx| vec![home],
                    );

                    let components = bc::BreadcrumbLink::new("Components").into_element(cx);
                    let components = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: label("Golden:breadcrumb-link:components"),
                            ..Default::default()
                        },
                        move |_cx| vec![components],
                    );

                    let page = bc::BreadcrumbPage::new("Breadcrumb").into_element(cx);
                    let page = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: label("Golden:breadcrumb-link:page"),
                            ..Default::default()
                        },
                        move |_cx| vec![page],
                    );

                    let chevron0 = bc::BreadcrumbSeparator::new()
                        .kind(bc::BreadcrumbSeparatorKind::ChevronRight)
                        .into_element(cx);
                    let chevron0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: label("Golden:breadcrumb-link:chevron-0"),
                            ..Default::default()
                        },
                        move |_cx| vec![chevron0],
                    );

                    let chevron1 = bc::BreadcrumbSeparator::new()
                        .kind(bc::BreadcrumbSeparatorKind::ChevronRight)
                        .into_element(cx);
                    let chevron1 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: label("Golden:breadcrumb-link:chevron-1"),
                            ..Default::default()
                        },
                        move |_cx| vec![chevron1],
                    );

                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, move |_cx| vec![home]),
                        chevron0,
                        bc::BreadcrumbItem::new().into_element(cx, move |_cx| vec![components]),
                        chevron1,
                        bc::BreadcrumbItem::new().into_element(cx, move |_cx| vec![page]),
                    ]
                })]
            })]
        })
    };

    let home = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:breadcrumb-link:home"),
    )
    .expect("fret Home link wrapper");
    let components = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:breadcrumb-link:components"),
    )
    .expect("fret Components link wrapper");
    let page = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:breadcrumb-link:page"),
    )
    .expect("fret Breadcrumb page wrapper");

    let chevron0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:breadcrumb-link:chevron-0"),
    )
    .expect("fret chevron-0 wrapper");
    let chevron1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:breadcrumb-link:chevron-1"),
    )
    .expect("fret chevron-1 wrapper");

    assert_close_px(
        "breadcrumb-link Home height",
        home.bounds.size.height,
        web_home.rect.h,
        1.0,
    );
    assert_close_px(
        "breadcrumb-link Components height",
        components.bounds.size.height,
        web_components.rect.h,
        1.0,
    );
    assert_close_px(
        "breadcrumb-link Page height",
        page.bounds.size.height,
        web_page.rect.h,
        1.0,
    );

    assert_close_px(
        "breadcrumb-link chevron-0 w",
        chevron0.bounds.size.width,
        web_chevron0.rect.w,
        1.0,
    );
    assert_close_px(
        "breadcrumb-link chevron-0 h",
        chevron0.bounds.size.height,
        web_chevron0.rect.h,
        1.0,
    );
    assert_close_px(
        "breadcrumb-link chevron-1 w",
        chevron1.bounds.size.width,
        web_chevron1.rect.w,
        1.0,
    );
    assert_close_px(
        "breadcrumb-link chevron-1 h",
        chevron1.bounds.size.height,
        web_chevron1.rect.h,
        1.0,
    );

    let actual_chevron0_offset_y = chevron0.bounds.origin.y.0 - home.bounds.origin.y.0;
    assert_close_px(
        "breadcrumb-link chevron-0 offset y",
        Px(actual_chevron0_offset_y),
        expected_chevron0_offset_y,
        1.0,
    );
    let actual_chevron1_offset_y = chevron1.bounds.origin.y.0 - components.bounds.origin.y.0;
    assert_close_px(
        "breadcrumb-link chevron-1 offset y",
        Px(actual_chevron1_offset_y),
        expected_chevron1_offset_y,
        1.0,
    );

    // Keep `ui` alive until after the snapshot-driven assertions (matches other tests' patterns).
    drop(ui);
}

#[test]
fn web_vs_fret_layout_breadcrumb_ellipsis_geometry() {
    let web = read_web_golden("breadcrumb-ellipsis");
    let theme = web_theme(&web);

    let web_ellipsis_box = find_first(&theme.root, &|n| {
        n.tag == "span"
            && class_has_all_tokens(n, &["flex", "size-9", "items-center", "justify-center"])
    })
    .expect("web breadcrumb ellipsis box");
    let web_ellipsis_icon = find_first(&theme.root, &|n| {
        n.tag == "svg" && class_has_token(n, "lucide-ellipsis")
    })
    .expect("web breadcrumb ellipsis icon");

    let expected_icon_offset_x = web_ellipsis_icon.rect.x - web_ellipsis_box.rect.x;
    let expected_icon_offset_y = web_ellipsis_icon.rect.y - web_ellipsis_box.rect.y;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, _snap, root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_shadcn::breadcrumb::primitives as bc;

        vec![bc::Breadcrumb::new().into_element(cx, |cx| {
            vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                vec![
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![bc::BreadcrumbEllipsis::new().into_element(cx)]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![bc::BreadcrumbLink::new("Components").into_element(cx)]
                    }),
                    bc::BreadcrumbSeparator::new().into_element(cx),
                    bc::BreadcrumbItem::new().into_element(cx, |cx| {
                        vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                    }),
                ]
            })]
        })]
    });

    let mut stack = vec![root];
    let mut rects: Vec<Rect> = Vec::new();
    while let Some(node) = stack.pop() {
        if let Some(bounds) = ui.debug_node_bounds(node) {
            rects.push(bounds);
        }
        for child in ui.children(node).into_iter().rev() {
            stack.push(child);
        }
    }

    let pick_best_by_size = |label: &str, expected: WebRect, rects: &[Rect]| -> Rect {
        let mut best: Option<Rect> = None;
        let mut best_score = f32::INFINITY;
        for rect in rects {
            let score =
                (rect.size.width.0 - expected.w).abs() + (rect.size.height.0 - expected.h).abs();
            if score < best_score {
                best_score = score;
                best = Some(*rect);
            }
        }
        best.unwrap_or_else(|| panic!("missing {label} match"))
    };

    let fret_box = pick_best_by_size("ellipsis box", web_ellipsis_box.rect, &rects);
    assert_close_px(
        "breadcrumb-ellipsis box w",
        fret_box.size.width,
        web_ellipsis_box.rect.w,
        1.0,
    );
    assert_close_px(
        "breadcrumb-ellipsis box h",
        fret_box.size.height,
        web_ellipsis_box.rect.h,
        1.0,
    );

    let fret_icon = pick_best_by_size("ellipsis icon", web_ellipsis_icon.rect, &rects);
    let actual_icon_offset_x = fret_icon.origin.x.0 - fret_box.origin.x.0;
    let actual_icon_offset_y = fret_icon.origin.y.0 - fret_box.origin.y.0;
    assert_close_px(
        "breadcrumb-ellipsis icon offset x",
        Px(actual_icon_offset_x),
        expected_icon_offset_x,
        1.0,
    );
    assert_close_px(
        "breadcrumb-ellipsis icon offset y",
        Px(actual_icon_offset_y),
        expected_icon_offset_y,
        1.0,
    );
    assert_close_px(
        "breadcrumb-ellipsis icon w",
        fret_icon.size.width,
        web_ellipsis_icon.rect.w,
        1.0,
    );
    assert_close_px(
        "breadcrumb-ellipsis icon h",
        fret_icon.size.height,
        web_ellipsis_icon.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_breadcrumb_dropdown_trigger_geometry() {
    let web = read_web_golden("breadcrumb-dropdown");
    let theme = web_theme(&web);

    let web_trigger = find_first(&theme.root, &|n| {
        n.tag == "button"
            && class_has_token(n, "gap-1")
            && n.attrs
                .get("data-state")
                .is_some_and(|state| state == "closed")
            && find_first(n, &|child| {
                child.tag == "svg" && class_has_token(child, "lucide-chevron-down")
            })
            .is_some()
    })
    .expect("web breadcrumb dropdown trigger");
    let web_icon = find_first(web_trigger, &|n| {
        n.tag == "svg" && class_has_token(n, "lucide-chevron-down")
    })
    .expect("web breadcrumb dropdown chevron-down icon");

    let expected_icon_offset_y = web_icon.rect.y - web_trigger.rect.y;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, snap, _root) = {
        let mut services = StyleAwareServices::default();
        run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;

            let open: Model<bool> = cx.app.models_mut().insert(false);
            let dropdown = fret_ui_shadcn::DropdownMenu::new(open)
                .modal(false)
                .align(fret_ui_shadcn::DropdownMenuAlign::Start);

            vec![bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new()
                            .kind(bc::BreadcrumbSeparatorKind::Slash)
                            .into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![dropdown.into_element(
                                cx,
                                |cx| {
                                    let theme = Theme::global(&*cx.app).clone();
                                    let text_px = theme.metric_required("font.size");
                                    let line_height = theme.metric_required("font.line_height");
                                    let muted = theme.color_required("muted-foreground");
                                    let style = fret_core::TextStyle {
                                        font: fret_core::FontId::default(),
                                        size: text_px,
                                        weight: fret_core::FontWeight::NORMAL,
                                        slant: Default::default(),
                                        line_height: Some(line_height),
                                        letter_spacing_em: None,
                                    };

                                    let mut props = PressableProps::default();
                                    props.a11y.role = Some(SemanticsRole::Button);
                                    props.a11y.label =
                                        Some(Arc::from("Golden:breadcrumb-dropdown:trigger"));

                                    cx.pressable(props, move |cx, _st| {
                                        vec![cx.flex(
                                            FlexProps {
                                                layout: Default::default(),
                                                direction: fret_core::Axis::Horizontal,
                                                gap: Px(4.0),
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Center,
                                                wrap: false,
                                            },
                                            move |cx| {
                                                let text = cx.text_props(TextProps {
                                                    layout: Default::default(),
                                                    text: Arc::from("Components"),
                                                    style: Some(style.clone()),
                                                    color: Some(muted),
                                                    wrap: TextWrap::Word,
                                                    overflow: TextOverflow::Clip,
                                                });

                                                let icon = fret_ui_kit::declarative::icon::icon_with(
                                                    cx,
                                                    fret_icons::ids::ui::CHEVRON_DOWN,
                                                    Some(Px(14.0)),
                                                    Some(fret_ui_kit::ColorRef::Color(muted)),
                                                );

                                                let icon = cx.semantics(
                                                    fret_ui::element::SemanticsProps {
                                                        role: SemanticsRole::Panel,
                                                        label: Some(Arc::from(
                                                            "Golden:breadcrumb-dropdown:chevron-down",
                                                        )),
                                                        ..Default::default()
                                                    },
                                                    move |_cx| vec![icon],
                                                );

                                                vec![text, icon]
                                            },
                                        )]
                                    })
                                },
                                |_cx| {
                                    vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Documentation"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Themes"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("GitHub"),
                                        ),
                                    ]
                                },
                            )]
                        }),
                        bc::BreadcrumbSeparator::new()
                            .kind(bc::BreadcrumbSeparatorKind::Slash)
                            .into_element(cx),
                        bc::BreadcrumbItem::new()
                            .into_element(cx, |cx| vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]),
                    ]
                })]
            })]
        })
    };

    let trigger = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Golden:breadcrumb-dropdown:trigger"),
    )
    .expect("fret breadcrumb dropdown trigger");

    assert_close_px(
        "breadcrumb-dropdown trigger height",
        trigger.bounds.size.height,
        web_trigger.rect.h,
        1.0,
    );

    let icon = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:breadcrumb-dropdown:chevron-down"),
    )
    .expect("fret breadcrumb dropdown chevron-down icon");

    assert_close_px(
        "breadcrumb-dropdown chevron-down w",
        icon.bounds.size.width,
        web_icon.rect.w,
        1.0,
    );
    assert_close_px(
        "breadcrumb-dropdown chevron-down h",
        icon.bounds.size.height,
        web_icon.rect.h,
        1.0,
    );

    let actual_icon_offset_y = icon.bounds.origin.y.0 - trigger.bounds.origin.y.0;
    assert_close_px(
        "breadcrumb-dropdown chevron-down offset y",
        Px(actual_icon_offset_y),
        expected_icon_offset_y,
        1.0,
    );

    // Keep `ui` alive until after `debug_node_bounds` queries (matches other tests' patterns).
    drop(ui);
}

#[test]
fn web_vs_fret_layout_breadcrumb_demo_toggle_trigger_geometry() {
    let web = read_web_golden("breadcrumb-demo");
    let theme = web_theme(&web);

    let web_trigger = find_first(&theme.root, &|n| {
        n.tag == "button"
            && class_has_token(n, "gap-1")
            && n.attrs
                .get("data-state")
                .is_some_and(|state| state == "closed")
            && find_first(n, &|child| {
                child.tag == "svg" && class_has_token(child, "lucide-ellipsis")
            })
            .is_some()
            && contains_text(n, "Toggle menu")
    })
    .expect("web breadcrumb-demo toggle trigger");

    let web_box = find_first(web_trigger, &|n| {
        n.tag == "span"
            && class_has_all_tokens(n, &["flex", "size-4", "items-center", "justify-center"])
    })
    .expect("web breadcrumb-demo ellipsis box (size-4)");

    let web_icon = find_first(web_trigger, &|n| {
        n.tag == "svg" && class_has_token(n, "lucide-ellipsis")
    })
    .expect("web breadcrumb-demo ellipsis icon");

    let expected_box_offset_y = web_box.rect.y - web_trigger.rect.y;
    let expected_icon_offset_y = web_icon.rect.y - web_trigger.rect.y;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (_ui, snap, _root) = {
        let mut services = StyleAwareServices::default();
        run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;

            let open: Model<bool> = cx.app.models_mut().insert(false);
            let dropdown = fret_ui_shadcn::DropdownMenu::new(open)
                .modal(false)
                .align(fret_ui_shadcn::DropdownMenuAlign::Start);

            vec![bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![dropdown.into_element(
                                cx,
                                |cx| {
                                    let mut props = PressableProps::default();
                                    props.a11y.role = Some(SemanticsRole::Button);
                                    props.a11y.label =
                                        Some(Arc::from("Golden:breadcrumb-demo:toggle-trigger"));

                                    cx.pressable(props, move |cx, _st| {
                                        let ellipsis = bc::BreadcrumbEllipsis::new()
                                            .size(Px(16.0))
                                            .into_element(cx);
                                        let ellipsis = cx.semantics(
                                            fret_ui::element::SemanticsProps {
                                                role: SemanticsRole::Panel,
                                                label: Some(Arc::from(
                                                    "Golden:breadcrumb-demo:ellipsis-box",
                                                )),
                                                ..Default::default()
                                            },
                                            move |_cx| vec![ellipsis],
                                        );
                                        vec![ellipsis]
                                    })
                                },
                                |_cx| {
                                    vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Documentation"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Themes"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("GitHub"),
                                        ),
                                    ]
                                },
                            )]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Components").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                        }),
                    ]
                })]
            })]
        })
    };

    let trigger = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Golden:breadcrumb-demo:toggle-trigger"),
    )
    .expect("fret breadcrumb-demo toggle trigger");
    assert_close_px(
        "breadcrumb-demo toggle trigger height",
        trigger.bounds.size.height,
        web_trigger.rect.h,
        1.0,
    );

    let ellipsis_box = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:breadcrumb-demo:ellipsis-box"),
    )
    .expect("fret breadcrumb-demo ellipsis box");
    assert_close_px(
        "breadcrumb-demo ellipsis box w",
        ellipsis_box.bounds.size.width,
        web_box.rect.w,
        1.0,
    );
    assert_close_px(
        "breadcrumb-demo ellipsis box h",
        ellipsis_box.bounds.size.height,
        web_box.rect.h,
        1.0,
    );

    let actual_box_offset_y = ellipsis_box.bounds.origin.y.0 - trigger.bounds.origin.y.0;
    assert_close_px(
        "breadcrumb-demo ellipsis box offset y",
        Px(actual_box_offset_y),
        expected_box_offset_y,
        1.0,
    );

    // We don't separately stamp the inner SVG yet, but the web golden's icon rect is expected to
    // align with the box in the `size-4` variant. Assert the same offset for the box as a proxy.
    assert_close_px(
        "breadcrumb-demo ellipsis icon offset y (proxy)",
        Px(actual_box_offset_y),
        expected_icon_offset_y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_breadcrumb_responsive_mobile_truncation_geometry() {
    let web = read_web_golden("breadcrumb-responsive.vp375x812");
    let theme = web_theme(&web);

    let web_link = find_first(&theme.root, &|n| {
        n.tag == "a"
            && class_has_token(n, "max-w-20")
            && class_has_token(n, "truncate")
            && contains_text(n, "Data Fetching")
    })
    .expect("web breadcrumb-responsive (mobile) Data Fetching link");

    let web_page = find_first(&theme.root, &|n| {
        n.tag == "span"
            && class_has_token(n, "max-w-20")
            && class_has_token(n, "truncate")
            && contains_text(n, "Caching and Revalidating")
    })
    .expect("web breadcrumb-responsive (mobile) Caching and Revalidating page");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (_ui, snap, _root) = {
        let mut services = StyleAwareServices::default();
        run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;

            let trunc_layout = LayoutRefinement::default().max_w(Px(80.0));

            vec![bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            let mut props = PressableProps::default();
                            props.a11y.role = Some(SemanticsRole::Button);
                            props.a11y.label = Some(Arc::from("Toggle Menu"));
                            vec![cx.pressable(props, move |cx, _st| {
                                vec![
                                    bc::BreadcrumbEllipsis::new()
                                        .size(Px(16.0))
                                        .into_element(cx),
                                ]
                            })]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            let link = bc::BreadcrumbLink::new("Data Fetching")
                                .truncate(true)
                                .refine_layout(trunc_layout.clone())
                                .into_element(cx);
                            vec![cx.semantics(
                                fret_ui::element::SemanticsProps {
                                    role: SemanticsRole::Panel,
                                    label: Some(Arc::from(
                                        "Golden:breadcrumb-responsive:mobile:data-fetching",
                                    )),
                                    ..Default::default()
                                },
                                move |_cx| vec![link],
                            )]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            let page = bc::BreadcrumbPage::new("Caching and Revalidating")
                                .truncate(true)
                                .refine_layout(trunc_layout.clone())
                                .into_element(cx);
                            vec![cx.semantics(
                                fret_ui::element::SemanticsProps {
                                    role: SemanticsRole::Panel,
                                    label: Some(Arc::from(
                                        "Golden:breadcrumb-responsive:mobile:caching",
                                    )),
                                    ..Default::default()
                                },
                                move |_cx| vec![page],
                            )]
                        }),
                    ]
                })]
            })]
        })
    };

    let fret_link = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:breadcrumb-responsive:mobile:data-fetching"),
    )
    .expect("fret breadcrumb-responsive Data Fetching link");
    assert_close_px(
        "breadcrumb-responsive (mobile) Data Fetching link w",
        fret_link.bounds.size.width,
        web_link.rect.w,
        1.0,
    );
    assert_close_px(
        "breadcrumb-responsive (mobile) Data Fetching link h",
        fret_link.bounds.size.height,
        web_link.rect.h,
        1.0,
    );

    let fret_page = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:breadcrumb-responsive:mobile:caching"),
    )
    .expect("fret breadcrumb-responsive Caching and Revalidating page");
    assert_close_px(
        "breadcrumb-responsive (mobile) Caching page w",
        fret_page.bounds.size.width,
        web_page.rect.w,
        1.0,
    );
    assert_close_px(
        "breadcrumb-responsive (mobile) Caching page h",
        fret_page.bounds.size.height,
        web_page.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_badge_demo_heights() {
    let web = read_web_golden("badge-demo");
    let theme = web_theme(&web);
    let web_badge = web_find_by_tag_and_text(&theme.root, "span", "Badge").expect("web badge");
    let web_secondary =
        web_find_by_tag_and_text(&theme.root, "span", "Secondary").expect("web badge secondary");
    let web_destructive = web_find_by_tag_and_text(&theme.root, "span", "Destructive")
        .expect("web badge destructive");
    let web_outline =
        web_find_by_tag_and_text(&theme.root, "span", "Outline").expect("web badge outline");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, snap, _root) = run_fret_root_with_ui(bounds, |cx| {
        let badge = fret_ui_shadcn::Badge::new("Badge").into_element(cx);
        let secondary = fret_ui_shadcn::Badge::new("Secondary")
            .variant(fret_ui_shadcn::BadgeVariant::Secondary)
            .into_element(cx);
        let destructive = fret_ui_shadcn::Badge::new("Destructive")
            .variant(fret_ui_shadcn::BadgeVariant::Destructive)
            .into_element(cx);
        let outline = fret_ui_shadcn::Badge::new("Outline")
            .variant(fret_ui_shadcn::BadgeVariant::Outline)
            .into_element(cx);

        vec![
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("Golden:badge-demo:default")),
                    ..Default::default()
                },
                move |_cx| vec![badge],
            ),
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("Golden:badge-demo:secondary")),
                    ..Default::default()
                },
                move |_cx| vec![secondary],
            ),
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("Golden:badge-demo:destructive")),
                    ..Default::default()
                },
                move |_cx| vec![destructive],
            ),
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("Golden:badge-demo:outline")),
                    ..Default::default()
                },
                move |_cx| vec![outline],
            ),
        ]
    });

    let assert_badge_height = |label: &str, node: &fret_core::SemanticsNode, expected: f32| {
        let actual = node.bounds.size.height.0;
        let tol = 1.0;
        if (actual - expected).abs() <= tol {
            return;
        }

        let children = ui.children(node.id);
        let child0 = children.first().copied();
        let child0_bounds = child0.and_then(|c| ui.debug_node_bounds(c));
        let grand0 = child0.and_then(|c| ui.children(c).first().copied());
        let grand0_bounds = grand0.and_then(|c| ui.debug_node_bounds(c));

        panic!(
            "{label}: expected≈{expected} (±{tol}) got={actual}; child={:?} child_bounds={:?} grandchild={:?} grandchild_bounds={:?}",
            child0, child0_bounds, grand0, grand0_bounds
        );
    };

    let fret_badge = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:badge-demo:default"),
    )
    .expect("fret badge default");
    assert_badge_height("badge height", fret_badge, web_badge.rect.h);

    let fret_secondary = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:badge-demo:secondary"),
    )
    .expect("fret badge secondary");
    assert_badge_height(
        "badge secondary height",
        fret_secondary,
        web_secondary.rect.h,
    );

    let fret_destructive = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:badge-demo:destructive"),
    )
    .expect("fret badge destructive");
    assert_badge_height(
        "badge destructive height",
        fret_destructive,
        web_destructive.rect.h,
    );

    let fret_outline = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:badge-demo:outline"),
    )
    .expect("fret badge outline");
    assert_badge_height("badge outline height", fret_outline, web_outline.rect.h);
}

#[test]
fn web_vs_fret_layout_avatar_demo_geometry() {
    let web = read_web_golden("avatar-demo");
    let theme = web_theme(&web);

    let web_avatar_round = web_find_by_class_tokens(
        &theme.root,
        &[
            "relative",
            "flex",
            "size-8",
            "shrink-0",
            "overflow-hidden",
            "rounded-full",
        ],
    )
    .expect("web avatar round");
    let web_avatar_rounded = web_find_by_class_tokens(
        &theme.root,
        &[
            "relative",
            "flex",
            "size-8",
            "shrink-0",
            "overflow-hidden",
            "rounded-lg",
        ],
    )
    .expect("web avatar rounded");
    let web_group =
        web_find_by_class_tokens(&theme.root, &["flex", "-space-x-2"]).expect("web avatar group");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, _snap, root) = run_fret_root_with_ui(bounds, |cx| {
        let image = ImageId::default();

        let avatar_round = fret_ui_shadcn::Avatar::new(vec![
            fret_ui_shadcn::AvatarImage::new(image).into_element(cx),
            fret_ui_shadcn::AvatarFallback::new("CN")
                .when_image_missing(Some(image))
                .into_element(cx),
        ])
        .into_element(cx);

        let avatar_rounded = fret_ui_shadcn::Avatar::new(vec![
            fret_ui_shadcn::AvatarImage::new(image).into_element(cx),
            fret_ui_shadcn::AvatarFallback::new("CN")
                .when_image_missing(Some(image))
                .into_element(cx),
        ])
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .into_element(cx);

        let group_items = (0..3)
            .map(|idx| {
                let mut avatar = fret_ui_shadcn::Avatar::new(vec![
                    fret_ui_shadcn::AvatarImage::new(image).into_element(cx),
                    fret_ui_shadcn::AvatarFallback::new("CN")
                        .when_image_missing(Some(image))
                        .into_element(cx),
                ]);
                if idx > 0 {
                    avatar = avatar.refine_layout(LayoutRefinement::default().ml_neg(Space::N2));
                }
                avatar.into_element(cx)
            })
            .collect::<Vec<_>>();

        let group = cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| group_items,
        );

        let group = cx.container(ContainerProps::default(), move |_cx| vec![group]);

        let row = cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                gap: Px(48.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| vec![avatar_round, avatar_rounded, group],
        );

        vec![row]
    });

    let mut stack = vec![root];
    let mut rects: Vec<(NodeId, Rect)> = Vec::new();
    while let Some(node) = stack.pop() {
        if let Some(bounds) = ui.debug_node_bounds(node) {
            rects.push((node, bounds));
        }
        for child in ui.children(node).into_iter().rev() {
            stack.push(child);
        }
    }

    let pick_best = |label: &str, expected: WebRect, rects: &[(NodeId, Rect)]| -> Rect {
        let mut best: Option<Rect> = None;
        let mut best_score = f32::INFINITY;
        for (_, rect) in rects {
            let score = (rect.origin.x.0 - expected.x).abs()
                + (rect.origin.y.0 - expected.y).abs()
                + (rect.size.width.0 - expected.w).abs()
                + (rect.size.height.0 - expected.h).abs();
            if score < best_score {
                best_score = score;
                best = Some(*rect);
            }
        }
        best.unwrap_or_else(|| panic!("missing {label} match"))
    };

    let fret_avatar_round = pick_best("avatar round", web_avatar_round.rect, &rects);
    let fret_avatar_rounded = pick_best("avatar rounded", web_avatar_rounded.rect, &rects);

    let group_items: Vec<Rect> = rects
        .iter()
        .filter_map(|(_id, rect)| {
            if (rect.origin.y.0 - web_group.rect.y).abs() > 1.0 {
                return None;
            }
            if (rect.size.width.0 - web_avatar_round.rect.w).abs() > 1.0 {
                return None;
            }
            if (rect.size.height.0 - web_avatar_round.rect.h).abs() > 1.0 {
                return None;
            }
            let x = rect.origin.x.0;
            if x < web_group.rect.x - 1.0 {
                return None;
            }
            if x > web_group.rect.x + web_group.rect.w + 1.0 {
                return None;
            }
            Some(*rect)
        })
        .collect();

    assert!(
        group_items.len() >= 3,
        "expected at least 3 avatar group items; got={}; items={group_items:?}",
        group_items.len(),
    );

    let mut group_items = group_items;
    group_items.sort_by(|a, b| a.origin.x.0.total_cmp(&b.origin.x.0));
    let mut distinct_items: Vec<Rect> = Vec::with_capacity(3);
    for rect in group_items {
        if distinct_items
            .last()
            .is_some_and(|prev| (rect.origin.x.0 - prev.origin.x.0).abs() <= 1.0)
        {
            continue;
        }
        distinct_items.push(rect);
        if distinct_items.len() == 3 {
            break;
        }
    }

    assert!(
        distinct_items.len() == 3,
        "expected 3 distinct avatar group x positions; got={}; items={distinct_items:?}",
        distinct_items.len(),
    );

    let min_x = distinct_items
        .iter()
        .map(|r| r.origin.x.0)
        .fold(f32::INFINITY, f32::min);
    let min_y = distinct_items
        .iter()
        .map(|r| r.origin.y.0)
        .fold(f32::INFINITY, f32::min);
    let max_x = distinct_items
        .iter()
        .map(|r| r.origin.x.0 + r.size.width.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = distinct_items
        .iter()
        .map(|r| r.origin.y.0 + r.size.height.0)
        .fold(f32::NEG_INFINITY, f32::max);

    let fret_group = Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        CoreSize::new(Px(max_x - min_x), Px(max_y - min_y)),
    );

    assert_close_px(
        "avatar round x",
        fret_avatar_round.origin.x,
        web_avatar_round.rect.x,
        1.0,
    );
    assert_close_px(
        "avatar round y",
        fret_avatar_round.origin.y,
        web_avatar_round.rect.y,
        1.0,
    );
    assert_close_px(
        "avatar round w",
        fret_avatar_round.size.width,
        web_avatar_round.rect.w,
        1.0,
    );
    assert_close_px(
        "avatar round h",
        fret_avatar_round.size.height,
        web_avatar_round.rect.h,
        1.0,
    );

    assert_close_px(
        "avatar rounded x",
        fret_avatar_rounded.origin.x,
        web_avatar_rounded.rect.x,
        1.0,
    );
    assert_close_px(
        "avatar rounded y",
        fret_avatar_rounded.origin.y,
        web_avatar_rounded.rect.y,
        1.0,
    );
    assert_close_px(
        "avatar rounded w",
        fret_avatar_rounded.size.width,
        web_avatar_rounded.rect.w,
        1.0,
    );
    assert_close_px(
        "avatar rounded h",
        fret_avatar_rounded.size.height,
        web_avatar_rounded.rect.h,
        1.0,
    );

    assert_close_px("avatar group x", fret_group.origin.x, web_group.rect.x, 1.0);
    assert_close_px("avatar group y", fret_group.origin.y, web_group.rect.y, 1.0);
    assert_close_px(
        "avatar group w",
        fret_group.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "avatar group h",
        fret_group.size.height,
        web_group.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_empty_avatar_geometry() {
    let web = read_web_golden("empty-avatar");
    let theme = web_theme(&web);

    let web_avatar = web_find_by_class_tokens(
        &theme.root,
        &[
            "relative",
            "flex",
            "shrink-0",
            "overflow-hidden",
            "rounded-full",
            "size-12",
        ],
    )
    .expect("web empty avatar root");
    let web_fallback = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-muted",
            "flex",
            "size-full",
            "items-center",
            "justify-center",
            "rounded-full",
        ],
    )
    .expect("web empty avatar fallback");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, _snap, root) = run_fret_root_with_ui(bounds, |cx| {
        let avatar = fret_ui_shadcn::Avatar::new(vec![
            fret_ui_shadcn::AvatarFallback::new("CN").into_element(cx),
        ])
        .refine_layout(
            LayoutRefinement::default()
                .w_px(Px(web_avatar.rect.w))
                .h_px(Px(web_avatar.rect.h)),
        )
        .into_element(cx);

        vec![avatar]
    });

    let mut stack = vec![root];
    let mut rects: Vec<(NodeId, Rect)> = Vec::new();
    while let Some(node) = stack.pop() {
        if let Some(bounds) = ui.debug_node_bounds(node) {
            rects.push((node, bounds));
        }
        for child in ui.children(node).into_iter().rev() {
            stack.push(child);
        }
    }

    let pick_best = |label: &str, expected: WebRect, rects: &[(NodeId, Rect)]| -> Rect {
        let mut best: Option<Rect> = None;
        let mut best_score = f32::INFINITY;
        for (_, rect) in rects {
            let score =
                (rect.size.width.0 - expected.w).abs() + (rect.size.height.0 - expected.h).abs();
            if score < best_score {
                best_score = score;
                best = Some(*rect);
            }
        }
        best.unwrap_or_else(|| panic!("missing {label} match"))
    };

    let fret_avatar = pick_best("avatar", web_avatar.rect, &rects);
    let fret_fallback = pick_best("fallback", web_fallback.rect, &rects);

    assert_close_px(
        "empty avatar w",
        fret_avatar.size.width,
        web_avatar.rect.w,
        1.0,
    );
    assert_close_px(
        "empty avatar h",
        fret_avatar.size.height,
        web_avatar.rect.h,
        1.0,
    );
    assert_close_px(
        "empty avatar fallback w",
        fret_fallback.size.width,
        web_fallback.rect.w,
        1.0,
    );
    assert_close_px(
        "empty avatar fallback h",
        fret_fallback.size.height,
        web_fallback.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_empty_avatar_group_geometry() {
    let web = read_web_golden("empty-avatar-group");
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(&theme.root, &["flex", "-space-x-2"])
        .expect("web empty avatar group");
    let web_item = web_find_by_class_tokens(
        &theme.root,
        &[
            "relative",
            "flex",
            "size-8",
            "shrink-0",
            "overflow-hidden",
            "rounded-full",
        ],
    )
    .expect("web empty avatar group item");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, _snap, root) = run_fret_root_with_ui(bounds, |cx| {
        let image = ImageId::default();
        let size = Px(web_item.rect.w);

        let avatars = (0..3)
            .map(|idx| {
                let mut avatar = fret_ui_shadcn::Avatar::new(vec![
                    fret_ui_shadcn::AvatarImage::new(image).into_element(cx),
                    fret_ui_shadcn::AvatarFallback::new("CN")
                        .when_image_missing(Some(image))
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_px(size).h_px(size));
                if idx > 0 {
                    avatar = avatar.refine_layout(LayoutRefinement::default().ml_neg(Space::N2));
                }
                avatar.into_element(cx)
            })
            .collect::<Vec<_>>();

        let group = cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| avatars,
        );

        vec![group]
    });

    let mut stack = vec![root];
    let mut rects: Vec<(NodeId, Rect)> = Vec::new();
    while let Some(node) = stack.pop() {
        if let Some(bounds) = ui.debug_node_bounds(node) {
            rects.push((node, bounds));
        }
        for child in ui.children(node).into_iter().rev() {
            stack.push(child);
        }
    }

    let group_items: Vec<Rect> = rects
        .iter()
        .filter_map(|(_id, rect)| {
            if (rect.size.width.0 - web_item.rect.w).abs() > 1.0 {
                return None;
            }
            if (rect.size.height.0 - web_item.rect.h).abs() > 1.0 {
                return None;
            }
            Some(*rect)
        })
        .collect();

    assert!(
        group_items.len() >= 3,
        "expected at least 3 avatar group items; got={}; items={group_items:?}",
        group_items.len(),
    );

    let mut group_items = group_items;
    group_items.sort_by(|a, b| a.origin.x.0.total_cmp(&b.origin.x.0));
    let mut distinct_items: Vec<Rect> = Vec::with_capacity(3);
    for rect in group_items {
        if distinct_items
            .last()
            .is_some_and(|prev| (rect.origin.x.0 - prev.origin.x.0).abs() <= 1.0)
        {
            continue;
        }
        distinct_items.push(rect);
        if distinct_items.len() == 3 {
            break;
        }
    }

    assert!(
        distinct_items.len() == 3,
        "expected 3 distinct avatar group x positions; got={}; items={distinct_items:?}",
        distinct_items.len(),
    );

    let min_x = distinct_items
        .iter()
        .map(|r| r.origin.x.0)
        .fold(f32::INFINITY, f32::min);
    let min_y = distinct_items
        .iter()
        .map(|r| r.origin.y.0)
        .fold(f32::INFINITY, f32::min);
    let max_x = distinct_items
        .iter()
        .map(|r| r.origin.x.0 + r.size.width.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = distinct_items
        .iter()
        .map(|r| r.origin.y.0 + r.size.height.0)
        .fold(f32::NEG_INFINITY, f32::max);

    let fret_group = Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        CoreSize::new(Px(max_x - min_x), Px(max_y - min_y)),
    );

    assert_close_px(
        "empty avatar group w",
        fret_group.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "empty avatar group h",
        fret_group.size.height,
        web_group.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_item_avatar_geometry() {
    let web = read_web_golden("item-avatar");
    let theme = web_theme(&web);

    let web_item_avatar = web_find_by_class_tokens(
        &theme.root,
        &[
            "relative",
            "flex",
            "shrink-0",
            "overflow-hidden",
            "rounded-full",
            "size-10",
        ],
    )
    .expect("web item avatar root");
    let web_group = web_find_by_class_tokens(&theme.root, &["flex", "-space-x-2"])
        .expect("web item avatar group");
    let web_group_item = web_find_by_class_tokens(
        &theme.root,
        &[
            "relative",
            "flex",
            "size-8",
            "shrink-0",
            "overflow-hidden",
            "rounded-full",
        ],
    )
    .expect("web item avatar group item");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, _snap, root) = run_fret_root_with_ui(bounds, |cx| {
        let image = ImageId::default();

        let item_avatar = fret_ui_shadcn::Avatar::new(vec![
            fret_ui_shadcn::AvatarImage::new(image).into_element(cx),
            fret_ui_shadcn::AvatarFallback::new("CN")
                .when_image_missing(Some(image))
                .into_element(cx),
        ])
        .refine_layout(
            LayoutRefinement::default()
                .w_px(Px(web_item_avatar.rect.w))
                .h_px(Px(web_item_avatar.rect.h)),
        )
        .into_element(cx);

        let group_items = (0..3)
            .map(|idx| {
                let mut avatar = fret_ui_shadcn::Avatar::new(vec![
                    fret_ui_shadcn::AvatarImage::new(image).into_element(cx),
                    fret_ui_shadcn::AvatarFallback::new("CN")
                        .when_image_missing(Some(image))
                        .into_element(cx),
                ])
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(Px(web_group_item.rect.w))
                        .h_px(Px(web_group_item.rect.h)),
                );
                if idx > 0 {
                    avatar = avatar.refine_layout(LayoutRefinement::default().ml_neg(Space::N2));
                }
                avatar.into_element(cx)
            })
            .collect::<Vec<_>>();

        let group = cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| group_items,
        );

        let col = cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Vertical,
                gap: Px(16.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![item_avatar, group],
        );

        vec![col]
    });

    let mut stack = vec![root];
    let mut rects: Vec<(NodeId, Rect)> = Vec::new();
    while let Some(node) = stack.pop() {
        if let Some(bounds) = ui.debug_node_bounds(node) {
            rects.push((node, bounds));
        }
        for child in ui.children(node).into_iter().rev() {
            stack.push(child);
        }
    }

    let pick_best = |label: &str, expected: WebRect, rects: &[(NodeId, Rect)]| -> Rect {
        let mut best: Option<Rect> = None;
        let mut best_score = f32::INFINITY;
        for (_, rect) in rects {
            let score =
                (rect.size.width.0 - expected.w).abs() + (rect.size.height.0 - expected.h).abs();
            if score < best_score {
                best_score = score;
                best = Some(*rect);
            }
        }
        best.unwrap_or_else(|| panic!("missing {label} match"))
    };

    let fret_item_avatar = pick_best("item avatar", web_item_avatar.rect, &rects);

    let group_items: Vec<Rect> = rects
        .iter()
        .filter_map(|(_id, rect)| {
            if (rect.size.width.0 - web_group_item.rect.w).abs() > 1.0 {
                return None;
            }
            if (rect.size.height.0 - web_group_item.rect.h).abs() > 1.0 {
                return None;
            }
            Some(*rect)
        })
        .collect();

    assert!(
        group_items.len() >= 3,
        "expected at least 3 item-avatar group items; got={}; items={group_items:?}",
        group_items.len(),
    );

    let mut group_items = group_items;
    group_items.sort_by(|a, b| a.origin.x.0.total_cmp(&b.origin.x.0));
    let mut distinct_items: Vec<Rect> = Vec::with_capacity(3);
    for rect in group_items {
        if distinct_items
            .last()
            .is_some_and(|prev| (rect.origin.x.0 - prev.origin.x.0).abs() <= 1.0)
        {
            continue;
        }
        distinct_items.push(rect);
        if distinct_items.len() == 3 {
            break;
        }
    }

    assert!(
        distinct_items.len() == 3,
        "expected 3 distinct item-avatar group x positions; got={}; items={distinct_items:?}",
        distinct_items.len(),
    );

    let min_x = distinct_items
        .iter()
        .map(|r| r.origin.x.0)
        .fold(f32::INFINITY, f32::min);
    let min_y = distinct_items
        .iter()
        .map(|r| r.origin.y.0)
        .fold(f32::INFINITY, f32::min);
    let max_x = distinct_items
        .iter()
        .map(|r| r.origin.x.0 + r.size.width.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = distinct_items
        .iter()
        .map(|r| r.origin.y.0 + r.size.height.0)
        .fold(f32::NEG_INFINITY, f32::max);

    let fret_group = Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        CoreSize::new(Px(max_x - min_x), Px(max_y - min_y)),
    );

    assert_close_px(
        "item avatar w",
        fret_item_avatar.size.width,
        web_item_avatar.rect.w,
        1.0,
    );
    assert_close_px(
        "item avatar h",
        fret_item_avatar.size.height,
        web_item_avatar.rect.h,
        1.0,
    );
    assert_close_px(
        "item avatar group w",
        fret_group.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "item avatar group h",
        fret_group.size.height,
        web_group.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_item_demo_item_rects_match_web() {
    let web = read_web_golden("item-demo");
    let theme = web_theme(&web);

    let web_items = web_collect_item_rows(&theme.root);
    assert_eq!(web_items.len(), 2, "expected 2 items");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let max_w = MetricRef::Px(Px(web_items[0].rect.w));
        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
            &Theme::global(&*cx.app),
            LayoutRefinement::default().w_full().max_w(max_w),
        );

        let outline = fret_ui_shadcn::ItemVariant::Outline;

        let item0 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:item-demo:0")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Item::new([
                        fret_ui_shadcn::ItemContent::new([
                            fret_ui_shadcn::ItemTitle::new("Basic Item").into_element(cx),
                            fret_ui_shadcn::ItemDescription::new(
                                "A simple item with title and description.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        fret_ui_shadcn::ItemActions::new([fret_ui_shadcn::Button::new("Action")
                            .variant(fret_ui_shadcn::ButtonVariant::Outline)
                            .size(fret_ui_shadcn::ButtonSize::Sm)
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .variant(outline)
                    .into_element(cx),
                ]
            },
        );

        let item1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:item-demo:1")),
                ..Default::default()
            },
            move |cx| {
                let badge = decl_icon::icon_with(
                    cx,
                    IconId::new_static("lucide.badge-check"),
                    Some(Px(20.0)),
                    None,
                );
                let chevron = decl_icon::icon_with(
                    cx,
                    IconId::new_static("lucide.chevron-right"),
                    Some(Px(16.0)),
                    None,
                );

                vec![
                    fret_ui_shadcn::Item::new([
                        fret_ui_shadcn::ItemMedia::new([badge]).into_element(cx),
                        fret_ui_shadcn::ItemContent::new([fret_ui_shadcn::ItemTitle::new(
                            "Your profile has been verified.",
                        )
                        .into_element(cx)])
                        .into_element(cx),
                        fret_ui_shadcn::ItemActions::new([chevron]).into_element(cx),
                    ])
                    .variant(outline)
                    .size(fret_ui_shadcn::ItemSize::Sm)
                    .into_element(cx),
                ]
            },
        );

        vec![cx.column(
            ColumnProps {
                layout: wrapper_layout,
                gap: Px(0.0),
                ..Default::default()
            },
            move |_cx| vec![item0, item1],
        )]
    });

    for i in 0..2 {
        let web_item = web_items[i];
        let item = find_by_test_id(&snap, &format!("Golden:item-demo:{i}"));
        assert_close_px(
            &format!("item-demo[{i}] w"),
            item.bounds.size.width,
            web_item.rect.w,
            2.0,
        );
        assert_close_px(
            &format!("item-demo[{i}] h"),
            item.bounds.size.height,
            web_item.rect.h,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_item_size_item_rects_match_web() {
    let web = read_web_golden("item-size");
    let theme = web_theme(&web);

    let web_items = web_collect_item_rows(&theme.root);
    assert_eq!(web_items.len(), 2, "expected 2 items");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let max_w = MetricRef::Px(Px(web_items[0].rect.w));
        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
            &Theme::global(&*cx.app),
            LayoutRefinement::default().w_full().max_w(max_w),
        );

        let outline = fret_ui_shadcn::ItemVariant::Outline;

        let item0 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:item-size:0")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Item::new([
                        fret_ui_shadcn::ItemContent::new([
                            fret_ui_shadcn::ItemTitle::new("Basic Item").into_element(cx),
                            fret_ui_shadcn::ItemDescription::new(
                                "A simple item with title and description.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        fret_ui_shadcn::ItemActions::new([fret_ui_shadcn::Button::new("Action")
                            .variant(fret_ui_shadcn::ButtonVariant::Outline)
                            .size(fret_ui_shadcn::ButtonSize::Sm)
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .variant(outline)
                    .into_element(cx),
                ]
            },
        );

        let item1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:item-size:1")),
                ..Default::default()
            },
            move |cx| {
                let badge = decl_icon::icon_with(
                    cx,
                    IconId::new_static("lucide.badge-check"),
                    Some(Px(20.0)),
                    None,
                );
                let chevron = decl_icon::icon_with(
                    cx,
                    IconId::new_static("lucide.chevron-right"),
                    Some(Px(16.0)),
                    None,
                );

                vec![
                    fret_ui_shadcn::Item::new([
                        fret_ui_shadcn::ItemMedia::new([badge]).into_element(cx),
                        fret_ui_shadcn::ItemContent::new([fret_ui_shadcn::ItemTitle::new(
                            "Your profile has been verified.",
                        )
                        .into_element(cx)])
                        .into_element(cx),
                        fret_ui_shadcn::ItemActions::new([chevron]).into_element(cx),
                    ])
                    .variant(outline)
                    .size(fret_ui_shadcn::ItemSize::Sm)
                    .into_element(cx),
                ]
            },
        );

        vec![cx.column(
            ColumnProps {
                layout: wrapper_layout,
                gap: Px(0.0),
                ..Default::default()
            },
            move |_cx| vec![item0, item1],
        )]
    });

    for i in 0..2 {
        let web_item = web_items[i];
        let item = find_by_test_id(&snap, &format!("Golden:item-size:{i}"));
        assert_close_px(
            &format!("item-size[{i}] w"),
            item.bounds.size.width,
            web_item.rect.w,
            2.0,
        );
        assert_close_px(
            &format!("item-size[{i}] h"),
            item.bounds.size.height,
            web_item.rect.h,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_item_variant_item_heights_match_web() {
    let web = read_web_golden("item-variant");
    let theme = web_theme(&web);

    let web_items = web_collect_item_rows(&theme.root);
    assert_eq!(web_items.len(), 3, "expected 3 items");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
            &Theme::global(&*cx.app),
            LayoutRefinement::default().w_px(MetricRef::Px(Px(web_items[0].rect.w))),
        );

        let mk_item = |cx: &mut fret_ui::ElementContext<'_, App>,
                       variant: fret_ui_shadcn::ItemVariant,
                       title: &str,
                       desc: &str,
                       test_id: &'static str| {
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    test_id: Some(Arc::from(test_id)),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        fret_ui_shadcn::Item::new([
                            fret_ui_shadcn::ItemContent::new([
                                fret_ui_shadcn::ItemTitle::new(title).into_element(cx),
                                fret_ui_shadcn::ItemDescription::new(desc).into_element(cx),
                            ])
                            .into_element(cx),
                            fret_ui_shadcn::ItemActions::new([fret_ui_shadcn::Button::new("Open")
                                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                                .size(fret_ui_shadcn::ButtonSize::Sm)
                                .into_element(cx)])
                            .into_element(cx),
                        ])
                        .variant(variant)
                        .into_element(cx),
                    ]
                },
            )
        };

        let item0 = mk_item(
            cx,
            fret_ui_shadcn::ItemVariant::Default,
            "Default Variant",
            "Standard styling with subtle background and borders.",
            "Golden:item-variant:0",
        );
        let item1 = mk_item(
            cx,
            fret_ui_shadcn::ItemVariant::Outline,
            "Outline Variant",
            "Outlined style with clear borders and transparent background.",
            "Golden:item-variant:1",
        );
        let item2 = mk_item(
            cx,
            fret_ui_shadcn::ItemVariant::Muted,
            "Muted Variant",
            "Subdued appearance with muted colors for secondary content.",
            "Golden:item-variant:2",
        );

        vec![cx.column(
            ColumnProps {
                layout: wrapper_layout,
                gap: Px(0.0),
                ..Default::default()
            },
            move |_cx| vec![item0, item1, item2],
        )]
    });

    for i in 0..3 {
        let web_item = web_items[i];
        let item = find_by_test_id(&snap, &format!("Golden:item-variant:{i}"));
        assert_close_px(
            &format!("item-variant[{i}] h"),
            item.bounds.size.height,
            web_item.rect.h,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_item_icon_item_rect_matches_web() {
    let web = read_web_golden("item-icon");
    let theme = web_theme(&web);

    let web_items = web_collect_item_rows(&theme.root);
    assert_eq!(web_items.len(), 1, "expected 1 item");
    let web_item = web_items[0];

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let max_w = MetricRef::Px(Px(web_item.rect.w));
        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
            &Theme::global(&*cx.app),
            LayoutRefinement::default().w_full().max_w(max_w),
        );

        let item = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:item-icon:item")),
                ..Default::default()
            },
            move |cx| {
                let alert = decl_icon::icon(cx, IconId::new_static("lucide.shield-alert"));
                vec![
                    fret_ui_shadcn::Item::new([
                        fret_ui_shadcn::ItemMedia::new([alert])
                            .variant(fret_ui_shadcn::ItemMediaVariant::Icon)
                            .into_element(cx),
                        fret_ui_shadcn::ItemContent::new([
                            fret_ui_shadcn::ItemTitle::new("Security Alert").into_element(cx),
                            fret_ui_shadcn::ItemDescription::new(
                                "New login detected from unknown device.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        fret_ui_shadcn::ItemActions::new([fret_ui_shadcn::Button::new("Review")
                            .variant(fret_ui_shadcn::ButtonVariant::Outline)
                            .size(fret_ui_shadcn::ButtonSize::Sm)
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .variant(fret_ui_shadcn::ItemVariant::Outline)
                    .into_element(cx),
                ]
            },
        );

        vec![cx.column(
            ColumnProps {
                layout: wrapper_layout,
                gap: Px(0.0),
                ..Default::default()
            },
            move |_cx| vec![item],
        )]
    });

    let item = find_by_test_id(&snap, "Golden:item-icon:item");
    assert_close_px("item-icon w", item.bounds.size.width, web_item.rect.w, 2.0);
    assert_close_px("item-icon h", item.bounds.size.height, web_item.rect.h, 2.0);
}

#[test]
fn web_vs_fret_layout_item_link_item_rects_match_web() {
    let web = read_web_golden("item-link");
    let theme = web_theme(&web);

    let web_items = web_collect_item_rows(&theme.root);
    assert_eq!(web_items.len(), 2, "expected 2 items");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let max_w = MetricRef::Px(Px(web_items[0].rect.w));
        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
            &Theme::global(&*cx.app),
            LayoutRefinement::default().w_full().max_w(max_w),
        );

        let item0 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:item-link:0")),
                ..Default::default()
            },
            move |cx| {
                let chevron = decl_icon::icon_with(
                    cx,
                    IconId::new_static("lucide.chevron-right"),
                    Some(Px(16.0)),
                    None,
                );
                vec![
                    fret_ui_shadcn::Item::new([
                        fret_ui_shadcn::ItemContent::new([
                            fret_ui_shadcn::ItemTitle::new("Visit our documentation")
                                .into_element(cx),
                            fret_ui_shadcn::ItemDescription::new(
                                "Learn how to get started with our components.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        fret_ui_shadcn::ItemActions::new([chevron]).into_element(cx),
                    ])
                    .into_element(cx),
                ]
            },
        );

        let item1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:item-link:1")),
                ..Default::default()
            },
            move |cx| {
                let external = decl_icon::icon_with(
                    cx,
                    IconId::new_static("lucide.external-link"),
                    Some(Px(16.0)),
                    None,
                );
                vec![
                    fret_ui_shadcn::Item::new([
                        fret_ui_shadcn::ItemContent::new([
                            fret_ui_shadcn::ItemTitle::new("External resource").into_element(cx),
                            fret_ui_shadcn::ItemDescription::new(
                                "Opens in a new tab with security attributes.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        fret_ui_shadcn::ItemActions::new([external]).into_element(cx),
                    ])
                    .variant(fret_ui_shadcn::ItemVariant::Outline)
                    .into_element(cx),
                ]
            },
        );

        vec![cx.column(
            ColumnProps {
                layout: wrapper_layout,
                gap: Px(0.0),
                ..Default::default()
            },
            move |_cx| vec![item0, item1],
        )]
    });

    for i in 0..2 {
        let web_item = web_items[i];
        let item = find_by_test_id(&snap, &format!("Golden:item-link:{i}"));
        assert_close_px(
            &format!("item-link[{i}] w"),
            item.bounds.size.width,
            web_item.rect.w,
            2.0,
        );
        assert_close_px(
            &format!("item-link[{i}] h"),
            item.bounds.size.height,
            web_item.rect.h,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_item_group_item_and_separator_heights_match_web() {
    let web = read_web_golden("item-group");
    let theme = web_theme(&web);

    let web_group = web_find_item_group(&theme.root).expect("web item-group");
    let web_items = web_collect_item_rows(web_group);
    assert_eq!(web_items.len(), 3, "expected 3 items");

    let mut web_seps = find_all(web_group, &|n| {
        n.tag == "div"
            && class_has_token(n, "bg-border")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
            && n.computed_style.get("height").is_some_and(|h| h == "1px")
    });
    web_seps.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_seps.len(), 2, "expected 2 separators");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let max_w = MetricRef::Px(Px(web_group.rect.w));
        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
            &Theme::global(&*cx.app),
            LayoutRefinement::default().w_full().max_w(max_w),
        );

        let plus = |cx: &mut fret_ui::ElementContext<'_, App>| {
            let icon = decl_icon::icon(cx, IconId::new_static("lucide.plus"));
            fret_ui_shadcn::Button::new("")
                .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                .size(fret_ui_shadcn::ButtonSize::Icon)
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .children([icon])
                .into_element(cx)
        };

        let people = [
            ("shadcn", "shadcn@vercel.com"),
            ("maxleiter", "maxleiter@github.com"),
            ("evilrabbit", "evilrabbit@github.com"),
        ];

        let mut rows: Vec<AnyElement> = Vec::new();
        for (idx, (username, email)) in people.into_iter().enumerate() {
            let item = cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    test_id: Some(Arc::from(format!("Golden:item-group:item-{idx}"))),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        fret_ui_shadcn::Item::new([
                            fret_ui_shadcn::ItemMedia::new([fret_ui_shadcn::Avatar::new([
                                fret_ui_shadcn::AvatarFallback::new(
                                    username.chars().next().unwrap_or('S').to_string(),
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx)])
                            .into_element(cx),
                            fret_ui_shadcn::ItemContent::new([
                                fret_ui_shadcn::ItemTitle::new(username).into_element(cx),
                                fret_ui_shadcn::ItemDescription::new(email).into_element(cx),
                            ])
                            .gap(Px(4.0))
                            .into_element(cx),
                            fret_ui_shadcn::ItemActions::new([plus(cx)]).into_element(cx),
                        ])
                        .into_element(cx),
                    ]
                },
            );
            rows.push(item);
            if idx < 2 {
                let sep = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        test_id: Some(Arc::from(format!("Golden:item-group:sep-{idx}"))),
                        ..Default::default()
                    },
                    move |cx| vec![fret_ui_shadcn::ItemSeparator::new().into_element(cx)],
                );
                rows.push(sep);
            }
        }

        let group = fret_ui_shadcn::ItemGroup::new(rows).into_element(cx);

        vec![cx.column(
            ColumnProps {
                layout: wrapper_layout,
                gap: Px(0.0),
                ..Default::default()
            },
            move |_cx| vec![group],
        )]
    });

    for (i, web_item) in web_items.iter().enumerate() {
        let id = format!("Golden:item-group:item-{i}");
        let item = find_by_test_id(&snap, &id);
        assert_close_px(
            &format!("item-group item[{i}] h"),
            item.bounds.size.height,
            web_item.rect.h,
            2.0,
        );
    }
    for (i, web_sep) in web_seps.iter().enumerate() {
        let id = format!("Golden:item-group:sep-{i}");
        let sep = find_by_test_id(&snap, &id);
        assert_close_px(
            &format!("item-group sep[{i}] h"),
            sep.bounds.size.height,
            web_sep.rect.h,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_item_header_grid_item_rects_match_web() {
    let web = read_web_golden("item-header");
    let theme = web_theme(&web);

    let web_group = web_find_item_group(&theme.root).expect("web item-group");
    let mut web_items = web_collect_item_rows(web_group);
    assert_eq!(web_items.len(), 3, "expected 3 items");
    web_items.sort_by(|a, b| a.rect.x.total_cmp(&b.rect.x));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let max_w = MetricRef::Px(Px(web_group.rect.w));
        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
            &Theme::global(&*cx.app),
            LayoutRefinement::default().w_full().max_w(max_w),
        );

        let gap = web_css_px(web_group, "gap");

        let models = [
            ("v0-1.5-sm", "Everyday tasks and UI generation."),
            ("v0-1.5-lg", "Advanced thinking or reasoning."),
            ("v0-2.0-mini", "Open Source model for everyone."),
        ];

        let mut items: Vec<AnyElement> = Vec::new();
        for (idx, (name, desc)) in models.into_iter().enumerate() {
            let item = cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    test_id: Some(Arc::from(format!("Golden:item-header:{idx}"))),
                    ..Default::default()
                },
                move |cx| {
                    let image = ui::container(cx, |_cx| Vec::new())
                        .w_full()
                        .aspect_ratio(1.0)
                        .into_element(cx);

                    vec![
                        fret_ui_shadcn::Item::new([
                            fret_ui_shadcn::ItemHeader::new([image]).into_element(cx),
                            fret_ui_shadcn::ItemContent::new([
                                fret_ui_shadcn::ItemTitle::new(name).into_element(cx),
                                fret_ui_shadcn::ItemDescription::new(desc).into_element(cx),
                            ])
                            .into_element(cx),
                        ])
                        .variant(fret_ui_shadcn::ItemVariant::Outline)
                        .into_element(cx),
                    ]
                },
            );
            items.push(item);
        }

        let group = fret_ui_shadcn::ItemGroup::new(items)
            .grid(3)
            .gap(gap)
            .into_element(cx);

        vec![cx.column(
            ColumnProps {
                layout: wrapper_layout,
                gap: Px(0.0),
                ..Default::default()
            },
            move |_cx| vec![group],
        )]
    });

    for i in 0..3 {
        let web_item = web_items[i];
        let item = find_by_test_id(&snap, &format!("Golden:item-header:{i}"));
        assert_close_px(
            &format!("item-header[{i}] w"),
            item.bounds.size.width,
            web_item.rect.w,
            2.0,
        );
        assert_close_px(
            &format!("item-header[{i}] h"),
            item.bounds.size.height,
            web_item.rect.h,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_item_image_list_item_heights_match_web() {
    let web = read_web_golden("item-image");
    let theme = web_theme(&web);

    let web_group = web_find_item_group(&theme.root).expect("web item-group");
    let web_items = web_collect_item_rows(web_group);
    assert_eq!(web_items.len(), 3, "expected 3 items");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let max_w = MetricRef::Px(Px(web_group.rect.w));
        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
            &Theme::global(&*cx.app),
            LayoutRefinement::default().w_full().max_w(max_w),
        );

        let gap = web_css_px(web_group, "rowGap");

        let songs = [
            (
                "Midnight City Lights",
                "Electric Nights",
                "Neon Dreams",
                "3:45",
            ),
            (
                "Coffee Shop Conversations",
                "Urban Stories",
                "The Morning Brew",
                "4:05",
            ),
            ("Digital Rain", "Binary Beats", "Cyber Symphony", "3:30"),
        ];

        let mut rows: Vec<AnyElement> = Vec::new();
        for (idx, (title, album, artist, duration)) in songs.into_iter().enumerate() {
            let item = cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    test_id: Some(Arc::from(format!("Golden:item-image:{idx}"))),
                    ..Default::default()
                },
                move |cx| {
                    let image = ui::container(cx, |_cx| Vec::new())
                        .w_px(MetricRef::Px(Px(32.0)))
                        .h_px(MetricRef::Px(Px(32.0)))
                        .into_element(cx);

                    vec![
                        fret_ui_shadcn::Item::new([
                            fret_ui_shadcn::ItemMedia::new([image])
                                .variant(fret_ui_shadcn::ItemMediaVariant::Image)
                                .into_element(cx),
                            fret_ui_shadcn::ItemContent::new([
                                fret_ui_shadcn::ItemTitle::new(format!("{title} - {album}"))
                                    .into_element(cx),
                                fret_ui_shadcn::ItemDescription::new(artist).into_element(cx),
                            ])
                            .into_element(cx),
                            fret_ui_shadcn::ItemContent::new([
                                fret_ui_shadcn::ItemDescription::new(duration).into_element(cx),
                            ])
                            .refine_layout(LayoutRefinement::default().flex_none())
                            .into_element(cx),
                        ])
                        .variant(fret_ui_shadcn::ItemVariant::Outline)
                        .into_element(cx),
                    ]
                },
            );
            rows.push(item);
        }

        let group = fret_ui_shadcn::ItemGroup::new(rows)
            .gap(gap)
            .into_element(cx);

        vec![cx.column(
            ColumnProps {
                layout: wrapper_layout,
                gap: Px(0.0),
                ..Default::default()
            },
            move |_cx| vec![group],
        )]
    });

    for (i, web_item) in web_items.iter().enumerate() {
        let id = format!("Golden:item-image:{i}");
        let item = find_by_test_id(&snap, &id);
        assert_close_px(
            &format!("item-image[{i}] h"),
            item.bounds.size.height,
            web_item.rect.h,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_tabs_demo_tab_list_height() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_tab_list = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-muted",
            "text-muted-foreground",
            "inline-flex",
            "h-9",
            "w-fit",
        ],
    )
    .expect("web tab list");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab_list = find_semantics(&snap, SemanticsRole::TabList, None).expect("fret tab list");
    assert_close_px(
        "tab list height",
        tab_list.bounds.size.height,
        web_tab_list.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_tabs_demo_active_tab_height() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_active_tab = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tab")
            && n.attrs.get("aria-selected").is_some_and(|v| v == "true")
            && contains_text(n, "Account")
    })
    .expect("web active tab");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab = find_semantics(&snap, SemanticsRole::Tab, Some("Account"))
        .expect("fret active tab semantics node");

    assert_close_px(
        "tab height",
        tab.bounds.size.height,
        web_active_tab.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_tabs_demo_inactive_tab_text_color_matches_web() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_inactive_tab = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tab")
            && n.attrs.get("aria-selected").is_some_and(|v| v == "false")
            && contains_text(n, "Password")
    })
    .expect("web inactive tab");
    let expected = web_inactive_tab
        .computed_style
        .get("color")
        .and_then(|s| parse_css_color(s))
        .expect("web inactive tab computedStyle.color");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab = find_semantics(&snap, SemanticsRole::Tab, Some("Password"))
        .expect("fret inactive tab semantics node");

    let mut actual: Option<Rgba> = None;
    for op in scene.ops() {
        if let SceneOp::Text { origin, color, .. } = *op
            && tab.bounds.contains(origin)
        {
            actual = Some(color_to_rgba(color));
            break;
        }
    }
    let actual = actual.expect("fret inactive tab text color");
    assert_rgba_close("inactive tab text color", actual, expected, 0.06);
}

#[test]
fn web_vs_fret_layout_tabs_demo_active_tab_text_color_matches_web() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_active_tab = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tab")
            && n.attrs.get("aria-selected").is_some_and(|v| v == "true")
            && contains_text(n, "Account")
    })
    .expect("web active tab");
    let expected = web_active_tab
        .computed_style
        .get("color")
        .and_then(|s| parse_css_color(s))
        .expect("web active tab computedStyle.color");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab = find_semantics(&snap, SemanticsRole::Tab, Some("Account"))
        .expect("fret active tab semantics node");

    let mut actual: Option<Rgba> = None;
    for op in scene.ops() {
        if let SceneOp::Text { origin, color, .. } = *op
            && tab.bounds.contains(origin)
        {
            actual = Some(color_to_rgba(color));
            break;
        }
    }
    let actual = actual.expect("fret active tab text color");
    assert_rgba_close("active tab text color", actual, expected, 0.06);
}

#[test]
fn web_vs_fret_layout_tabs_demo_active_tab_inset_matches_web() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_tab_list = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tablist")
    })
    .expect("web tablist role");
    let web_active_tab = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tab")
            && n.attrs.get("aria-selected").is_some_and(|v| v == "true")
            && contains_text(n, "Account")
    })
    .expect("web active tab");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let active_tab =
        find_semantics(&snap, SemanticsRole::Tab, Some("Account")).expect("fret active tab");
    let tab_list = {
        let mut parent = active_tab.parent;
        let mut out = None;
        while let Some(pid) = parent {
            let p = snap
                .nodes
                .iter()
                .find(|n| n.id == pid)
                .expect("semantics parent node");
            if p.role == SemanticsRole::TabList {
                out = Some(p);
                break;
            }
            parent = p.parent;
        }
        out.expect("fret tab list ancestor")
    };

    let web_dx = web_active_tab.rect.x - web_tab_list.rect.x;
    let web_dy = web_active_tab.rect.y - web_tab_list.rect.y;
    let fret_dx = active_tab.bounds.origin.x.0 - tab_list.bounds.origin.x.0;
    let fret_dy = active_tab.bounds.origin.y.0 - tab_list.bounds.origin.y.0;

    if std::env::var_os("FRET_TEST_DEBUG_TABS").is_some() {
        eprintln!("web tablist: {:?}", web_tab_list.rect);
        eprintln!("web active tab: {:?}", web_active_tab.rect);
        eprintln!("web inset: ({web_dx:.3}, {web_dy:.3})");
        eprintln!("fret tablist: {:?}", tab_list.bounds);
        eprintln!("fret active tab: {:?}", active_tab.bounds);
        eprintln!("fret inset: ({fret_dx:.3}, {fret_dy:.3})");

        eprintln!("fret tablist ancestors for active tab:");
        let mut parent = active_tab.parent;
        while let Some(pid) = parent {
            let p = snap
                .nodes
                .iter()
                .find(|n| n.id == pid)
                .expect("semantics parent node");
            eprintln!(
                "  - {:?} label={:?} bounds={:?}",
                p.role,
                p.label.as_deref(),
                p.bounds
            );
            parent = p.parent;
        }

        eprintln!("fret tablists:");
        for n in snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::TabList)
        {
            eprintln!("  - label={:?} bounds={:?}", n.label.as_deref(), n.bounds);
        }
        eprintln!("fret tabs:");
        for n in snap.nodes.iter().filter(|n| n.role == SemanticsRole::Tab) {
            eprintln!(
                "  - label={:?} selected={} bounds={:?} parent={:?}",
                n.label.as_deref(),
                n.flags.selected,
                n.bounds,
                n.parent
            );
        }
    }

    assert_close_px("active tab inset x", Px(fret_dx), web_dx, 1.0);
    assert_close_px("active tab inset y", Px(fret_dy), web_dy, 1.0);
}

#[test]
fn web_vs_fret_layout_tabs_demo_panel_gap() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_tab_list = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tablist")
    })
    .expect("web tablist role");
    let web_panel = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "tabpanel")
    })
    .expect("web tabpanel role");

    let web_gap_y = web_panel.rect.y - (web_tab_list.rect.y + web_tab_list.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab_list = find_semantics(&snap, SemanticsRole::TabList, None).expect("fret tab list");
    let panel = find_semantics(&snap, SemanticsRole::TabPanel, None).expect("fret tab panel");

    let fret_gap_y =
        panel.bounds.origin.y.0 - (tab_list.bounds.origin.y.0 + tab_list.bounds.size.height.0);

    assert_close_px("tab panel gap", Px(fret_gap_y), web_gap_y, 1.0);
}

#[test]
fn web_vs_fret_layout_card_with_form_width() {
    let web = read_web_golden("card-with-form");
    let theme = web_theme(&web);
    let web_card = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-card",
            "text-card-foreground",
            "rounded-xl",
            "border",
            "w-[350px]",
        ],
    )
    .expect("web card root");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let card = fret_ui_shadcn::Card::new(vec![
            fret_ui_shadcn::CardHeader::new(vec![
                fret_ui_shadcn::CardTitle::new("Title").into_element(cx),
                fret_ui_shadcn::CardDescription::new("Description").into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardContent::new(vec![cx.text("Content")]).into_element(cx),
        ])
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(web_card.rect.w)))
        .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:card-with-form:root")),
                ..Default::default()
            },
            move |_cx| vec![card],
        )]
    });

    let card = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:card-with-form:root"),
    )
    .expect("fret card root");

    assert_close_px("card width", card.bounds.size.width, web_card.rect.w, 1.0);
}

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("missing semantics node with test_id={id:?}"))
}

fn web_find_button_by_sr_text<'a>(root: &'a WebNode, text: &str) -> Option<&'a WebNode> {
    web_find_by_tag_and_text(root, "button", text)
}

fn web_find_carousel_root<'a>(root: &'a WebNode, max_w: &str) -> Option<&'a WebNode> {
    web_find_by_class_tokens(root, &["relative", "w-full", max_w])
}

fn web_find_first_div_by_class_tokens<'a>(
    root: &'a WebNode,
    tokens: &[&str],
) -> Option<&'a WebNode> {
    let mut matches = find_all(root, &|n| n.tag == "div" && class_has_all_tokens(n, tokens));
    matches.sort_by(|a, b| {
        a.rect
            .y
            .total_cmp(&b.rect.y)
            .then_with(|| a.rect.x.total_cmp(&b.rect.x))
    });
    matches.into_iter().next()
}

fn carousel_card_content(
    cx: &mut fret_ui::ElementContext<'_, App>,
    number: u32,
    text_px: Px,
    line_height: Px,
    aspect_square: bool,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let mut layout = LayoutRefinement::default().w_full();
    if aspect_square {
        layout = layout.aspect_ratio(1.0);
    }

    let text = ui::text(cx, format!("{number}"))
        .text_size_px(text_px)
        .line_height_px(line_height)
        .font_semibold()
        .into_element(cx);

    cx.flex(
        FlexProps {
            layout: fret_ui_kit::declarative::style::layout_style(&theme, layout),
            direction: fret_core::Axis::Horizontal,
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            padding: Edges::all(Px(24.0)),
            ..Default::default()
        },
        move |_cx| vec![text],
    )
}

fn carousel_slide(
    cx: &mut fret_ui::ElementContext<'_, App>,
    number: u32,
    text_px: Px,
    line_height: Px,
    aspect_square: bool,
    with_p1_wrapper: bool,
) -> AnyElement {
    let content = carousel_card_content(cx, number, text_px, line_height, aspect_square);
    let card = fret_ui_shadcn::Card::new([content]).into_element(cx);

    if with_p1_wrapper {
        ui::container(cx, move |_cx| vec![card])
            .p_1()
            .into_element(cx)
    } else {
        card
    }
}

fn assert_carousel_geometry_matches_web(
    web_name: &str,
    max_w: &str,
    web_item_tokens: &[&str],
    build: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> AnyElement,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_carousel = web_find_carousel_root(&theme.root, max_w).expect("web carousel root");
    let web_prev =
        web_find_button_by_sr_text(&theme.root, "Previous slide").expect("web prev button");
    let web_next = web_find_button_by_sr_text(&theme.root, "Next slide").expect("web next button");
    let web_item = web_find_first_div_by_class_tokens(&theme.root, web_item_tokens)
        .expect("web carousel item");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| vec![build(cx)]);

    let carousel = find_by_test_id(&snap, "carousel");
    let prev = find_by_test_id(&snap, "carousel-previous");
    let next = find_by_test_id(&snap, "carousel-next");
    let item = find_by_test_id(&snap, "carousel-item-1");

    assert_close_px(
        "carousel width",
        carousel.bounds.size.width,
        web_carousel.rect.w,
        1.0,
    );
    assert_close_px(
        "carousel height",
        carousel.bounds.size.height,
        web_carousel.rect.h,
        1.0,
    );

    assert_close_px("prev width", prev.bounds.size.width, web_prev.rect.w, 1.0);
    assert_close_px("prev height", prev.bounds.size.height, web_prev.rect.h, 1.0);
    assert_close_px("next width", next.bounds.size.width, web_next.rect.w, 1.0);
    assert_close_px("next height", next.bounds.size.height, web_next.rect.h, 1.0);

    assert_close_px(
        "prev dx",
        Px(prev.bounds.origin.x.0 - carousel.bounds.origin.x.0),
        web_prev.rect.x - web_carousel.rect.x,
        1.0,
    );
    assert_close_px(
        "prev dy",
        Px(prev.bounds.origin.y.0 - carousel.bounds.origin.y.0),
        web_prev.rect.y - web_carousel.rect.y,
        1.0,
    );
    assert_close_px(
        "next dx",
        Px(next.bounds.origin.x.0 - carousel.bounds.origin.x.0),
        web_next.rect.x - web_carousel.rect.x,
        1.0,
    );
    assert_close_px(
        "next dy",
        Px(next.bounds.origin.y.0 - carousel.bounds.origin.y.0),
        web_next.rect.y - web_carousel.rect.y,
        1.0,
    );

    assert_close_px(
        "item dx",
        Px(item.bounds.origin.x.0 - carousel.bounds.origin.x.0),
        web_item.rect.x - web_carousel.rect.x,
        1.0,
    );
    assert_close_px(
        "item dy",
        Px(item.bounds.origin.y.0 - carousel.bounds.origin.y.0),
        web_item.rect.y - web_carousel.rect.y,
        1.0,
    );
    assert_close_px("item width", item.bounds.size.width, web_item.rect.w, 1.0);
    assert_close_px("item height", item.bounds.size.height, web_item.rect.h, 1.0);
}

#[test]
fn web_vs_fret_layout_calendar_demo_day_grid_geometry_and_a11y_labels_match_web() {
    let web = read_web_golden("calendar-demo");
    let theme = web_theme(&web);

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");

    let web_day = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Sunday, December 28th, 2025")
    })
    .expect("web day button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use time::{Month, Weekday};

        let month: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

        vec![
            fret_ui_shadcn::Calendar::new(month, selected)
                .week_start(Weekday::Sunday)
                .disable_outside_days(false)
                .into_element(cx),
        ]
    });

    fn is_calendar_day_label(label: &str) -> bool {
        // Examples:
        // - "Sunday, December 28th, 2025"
        // - "Thursday, June 12th, 2025, selected"
        let label = label.strip_suffix(", selected").unwrap_or(label);
        let label = label.strip_prefix("Today, ").unwrap_or(label);
        if !label.contains(',') {
            return false;
        }
        let Some((_weekday, rest)) = label.split_once(", ") else {
            return false;
        };
        let Some((_month_and_day, year)) = rest.rsplit_once(", ") else {
            return false;
        };
        if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        label.contains("st, ")
            || label.contains("nd, ")
            || label.contains("rd, ")
            || label.contains("th, ")
    }

    let day_buttons = snap
        .nodes
        .iter()
        .filter(|n| {
            n.role == SemanticsRole::Button
                && n.label
                    .as_deref()
                    .is_some_and(|label| is_calendar_day_label(label))
        })
        .count();
    assert_eq!(
        day_buttons, 35,
        "expected a 5-week (35-day) grid for January 2026 when week starts on Sunday"
    );

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    assert_close_px(
        "calendar prev button width",
        prev.bounds.size.width,
        web_prev.rect.w,
        1.0,
    );
    assert_close_px(
        "calendar prev button height",
        prev.bounds.size.height,
        web_prev.rect.h,
        1.0,
    );

    let day = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Sunday, December 28th, 2025"),
    )
    .expect("fret day semantics node");
    assert_close_px(
        "calendar day button width",
        day.bounds.size.width,
        web_day.rect.w,
        1.0,
    );
    assert_close_px(
        "calendar day button height",
        day.bounds.size.height,
        web_day.rect.h,
        1.0,
    );

    let node_bounds = ui.debug_node_bounds(day.id).expect("fret day node bounds");
    assert_close_px("calendar day x", node_bounds.origin.x, web_day.rect.x, 3.0);
    assert_close_px("calendar day y", node_bounds.origin.y, web_day.rect.y, 3.0);
}

#[test]
fn web_vs_fret_layout_calendar_hijri_day_grid_geometry_and_a11y_labels_match_web() {
    let web = read_web_golden("calendar-hijri");
    let theme = web_theme(&web);

    fn parse_css_px(s: &str) -> Option<f32> {
        s.strip_suffix("px")?.parse::<f32>().ok()
    }

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;
    let web_padding_left = web_rdp_root
        .computed_style
        .get("paddingLeft")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);
    let web_border_left = web_rdp_root
        .computed_style
        .get("borderLeftWidth")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);

    let web_month_grid =
        web_find_by_class_token(&theme.root, "rdp-month_grid").expect("web month grid");
    let web_title = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label")
        .as_str();

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");
    let web_next = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Next Month")
    })
    .expect("web next-month button");

    const HIJRI_WEEKDAYS: [&str; 7] = [
        "شنبه",
        "یک\u{200c}شنبه",
        "دوشنبه",
        "سه\u{200c}شنبه",
        "چهارشنبه",
        "پنج\u{200c}شنبه",
        "جمعه",
    ];

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| HIJRI_WEEKDAYS.iter().any(|wd| label.starts_with(wd)))
    });
    assert_eq!(
        web_day_buttons.len(),
        42,
        "expected a 6-week (42-day) grid for calendar-hijri"
    );

    let cell_size = parse_calendar_cell_size_px(&theme);

    let chrome_override = {
        let mut chrome = ChromeRefinement::default();
        if (web_padding_left - 0.0).abs() < 0.5 {
            chrome = chrome.p(Space::N0);
        } else if (web_padding_left - 12.0).abs() < 0.5 {
            chrome = chrome.p(Space::N3);
        } else if (web_padding_left - 8.0).abs() < 0.5 {
            chrome = chrome.p(Space::N2);
        }
        if web_border_left >= 0.5 {
            chrome = chrome.border_1();
        }
        chrome
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar_solar_hijri::SolarHijriMonth;
        use time::{Date, Month};

        let selected_date = Date::from_calendar_date(2025, Month::June, 12).expect("valid date");
        let month = SolarHijriMonth::from_gregorian(selected_date);

        let month_model: Model<SolarHijriMonth> = cx.app.models_mut().insert(month);
        let selected: Model<Option<Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut cal = fret_ui_shadcn::CalendarHijri::new(month_model, selected)
            .show_outside_days(true)
            .refine_style(chrome_override);
        if let Some(cell_size) = cell_size {
            cal = cal.cell_size(cell_size);
        }

        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |cx| vec![cal.into_element(cx)],
        )]
    });

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    let next = find_semantics(&snap, SemanticsRole::Button, Some("Go to the Next Month"))
        .expect("fret next-month semantics node");

    let prev_bounds = ui.debug_node_bounds(prev.id).expect("prev bounds");
    let next_bounds = ui.debug_node_bounds(next.id).expect("next bounds");
    assert_close_px(
        "calendar-hijri prev x",
        prev_bounds.origin.x,
        web_prev.rect.x,
        3.0,
    );
    assert_close_px(
        "calendar-hijri prev y",
        prev_bounds.origin.y,
        web_prev.rect.y,
        3.0,
    );
    assert_close_px(
        "calendar-hijri next x",
        next_bounds.origin.x,
        web_next.rect.x,
        3.0,
    );
    assert_close_px(
        "calendar-hijri next y",
        next_bounds.origin.y,
        web_next.rect.y,
        3.0,
    );

    let title = find_semantics(&snap, SemanticsRole::Text, Some(web_title))
        .expect("fret calendar-hijri title semantics node");
    let web_title_node = find_first(&theme.root, &|n| n.text.as_deref() == Some(web_title))
        .expect("web calendar-hijri title node");
    let title_bounds = ui.debug_node_bounds(title.id).expect("title bounds");
    // Title text width is font-metrics dependent (Persian shaping), so gate the center position.
    let title_center_x = title_bounds.origin.x.0 + title_bounds.size.width.0 / 2.0;
    let web_title_center_x = web_title_node.rect.x + web_title_node.rect.w / 2.0;
    assert_close_px(
        "calendar-hijri title center x",
        Px(title_center_x),
        web_title_center_x,
        3.0,
    );

    for web_day in web_day_buttons {
        let label = web_day.attrs.get("aria-label").expect("web day aria-label");
        let fret_day = find_semantics(&snap, SemanticsRole::Button, Some(label.as_str()))
            .unwrap_or_else(|| panic!("missing fret hijri day button label={label:?}"));
        let fret_bounds = ui.debug_node_bounds(fret_day.id).expect("fret day bounds");

        assert_close_px(
            "calendar-hijri day w",
            fret_bounds.size.width,
            web_day.rect.w,
            1.0,
        );
        assert_close_px(
            "calendar-hijri day h",
            fret_bounds.size.height,
            web_day.rect.h,
            1.0,
        );
        assert_close_px(
            "calendar-hijri day x",
            fret_bounds.origin.x,
            web_day.rect.x,
            3.0,
        );
        assert_close_px(
            "calendar-hijri day y",
            fret_bounds.origin.y,
            web_day.rect.y,
            3.0,
        );
    }
}

fn parse_calendar_title_label(label: &str) -> Option<(time::Month, i32)> {
    let label = label.trim();
    let (month, year) = label.rsplit_once(' ')?;
    if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let year: i32 = year.parse().ok()?;

    let month_lower = month.to_lowercase();
    let month = match (month, month_lower.as_str()) {
        ("January", _) | (_, "january") | (_, "enero") => time::Month::January,
        ("February", _) | (_, "february") | (_, "febrero") => time::Month::February,
        ("March", _) | (_, "march") | (_, "marzo") => time::Month::March,
        ("April", _) | (_, "april") | (_, "abril") => time::Month::April,
        ("May", _) | (_, "may") | (_, "mayo") => time::Month::May,
        ("June", _) | (_, "june") | (_, "junio") => time::Month::June,
        ("July", _) | (_, "july") | (_, "julio") => time::Month::July,
        ("August", _) | (_, "august") | (_, "agosto") => time::Month::August,
        ("September", _) | (_, "september") | (_, "septiembre") | (_, "setiembre") => {
            time::Month::September
        }
        ("October", _) | (_, "october") | (_, "octubre") => time::Month::October,
        ("November", _) | (_, "november") | (_, "noviembre") => time::Month::November,
        ("December", _) | (_, "december") | (_, "diciembre") => time::Month::December,
        _ => return None,
    };
    Some((month, year))
}

fn parse_calendar_weekday_label(label: &str) -> Option<time::Weekday> {
    let label = label.trim();
    let lower = label.to_lowercase();
    match (label, lower.as_str()) {
        ("Monday", _) | (_, "monday") | (_, "lunes") => Some(time::Weekday::Monday),
        ("Tuesday", _) | (_, "tuesday") | (_, "martes") => Some(time::Weekday::Tuesday),
        ("Wednesday", _) | (_, "wednesday") | (_, "miércoles") | (_, "miercoles") => {
            Some(time::Weekday::Wednesday)
        }
        ("Thursday", _) | (_, "thursday") | (_, "jueves") => Some(time::Weekday::Thursday),
        ("Friday", _) | (_, "friday") | (_, "viernes") => Some(time::Weekday::Friday),
        ("Saturday", _) | (_, "saturday") | (_, "sábado") | (_, "sabado") => {
            Some(time::Weekday::Saturday)
        }
        ("Sunday", _) | (_, "sunday") | (_, "domingo") => Some(time::Weekday::Sunday),
        _ => None,
    }
}

fn parse_calendar_day_aria_label(label: &str) -> Option<(time::Date, bool)> {
    let selected = label.ends_with(", selected");
    let label = label.strip_suffix(", selected").unwrap_or(label);
    let label = label.strip_prefix("Today, ").unwrap_or(label);
    let label = label.strip_prefix("Hoy, ").unwrap_or(label);

    if let Some((prefix, year)) = label.rsplit_once(", ") {
        if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
            let year: i32 = year.parse().ok()?;

            let (_weekday, month_and_day) = prefix.split_once(", ")?;
            let (month, day_with_suffix) = month_and_day.split_once(' ')?;
            let (month, _label_year) = parse_calendar_title_label(&format!("{month} {year}"))?;

            let day_digits: String = day_with_suffix
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if day_digits.is_empty() {
                return None;
            }
            let day: u8 = day_digits.parse().ok()?;

            let date = time::Date::from_calendar_date(year, month, day).ok()?;
            return Some((date, selected));
        }
    }

    // e.g. "lunes, 1 de septiembre de 2025"
    let (weekday, rest) = label.split_once(", ")?;
    let _weekday = parse_calendar_weekday_label(weekday)?;
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() != 5 || parts[1] != "de" || parts[3] != "de" {
        return None;
    }
    let day: u8 = parts[0].parse().ok()?;
    let (month, year) = parse_calendar_title_label(&format!("{} {}", parts[2], parts[4]))?;
    let date = time::Date::from_calendar_date(year, month, day).ok()?;
    Some((date, selected))
}

fn days_in_month(year: i32, month: time::Month) -> u8 {
    match month {
        time::Month::January => 31,
        time::Month::February => {
            let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            if leap { 29 } else { 28 }
        }
        time::Month::March => 31,
        time::Month::April => 30,
        time::Month::May => 31,
        time::Month::June => 30,
        time::Month::July => 31,
        time::Month::August => 31,
        time::Month::September => 30,
        time::Month::October => 31,
        time::Month::November => 30,
        time::Month::December => 31,
    }
}

fn parse_calendar_cell_size_px(theme: &WebGoldenTheme) -> Option<Px> {
    let rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root")?;
    let class_name = rdp_root.class_name.as_deref().unwrap_or("");

    fn parse_spacing_value(token: &str, prefix: &str) -> Option<f32> {
        let rest = token.strip_prefix(prefix)?;
        let rest = rest.strip_suffix(")]")?;
        rest.parse::<f32>().ok()
    }

    let mut base: Option<f32> = None;
    let mut md: Option<f32> = None;
    for token in class_name.split_whitespace() {
        if let Some(v) = parse_spacing_value(token, "[--cell-size:--spacing(") {
            base = Some(v);
        }
        if let Some(v) = parse_spacing_value(token, "md:[--cell-size:--spacing(") {
            md = Some(v);
        }
    }

    let viewport_w = theme.viewport.w;
    let spacing = if viewport_w >= 768.0 {
        md.or(base)
    } else {
        base
    }?;

    Some(Px(spacing * 4.0))
}

fn assert_calendar_single_month_variant_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    fn parse_css_px(s: &str) -> Option<f32> {
        s.strip_suffix("px")?.parse::<f32>().ok()
    }

    let web_padding_left = web_rdp_root
        .computed_style
        .get("paddingLeft")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);
    let web_border_left = web_rdp_root
        .computed_style
        .get("borderLeftWidth")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(
        web_month_grids.len(),
        1,
        "expected a single month grid for {web_name} (multi-month variants are gated separately)"
    );
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(
        !web_day_buttons.is_empty(),
        "expected calendar day buttons for {web_name}"
    );

    let web_weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();

    let web_is_range_mode = find_first(&theme.root, &|n| {
        class_has_token(n, "rdp-range_start")
            || class_has_token(n, "rdp-range_middle")
            || class_has_token(n, "rdp-range_end")
    })
    .is_some();

    let web_selected = web_day_buttons
        .iter()
        .find(|n| {
            n.attrs
                .get("aria-label")
                .is_some_and(|label| label.as_str().ends_with(", selected"))
        })
        .copied();
    let selected_date = match web_selected_dates.as_slice() {
        [] => None,
        [d] => Some(*d),
        _ => None,
    };

    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let web_sample = web_selected.unwrap_or(web_day_buttons[0]);
    let web_sample_label = web_sample
        .attrs
        .get("aria-label")
        .expect("web sample day aria-label")
        .clone();

    let cell_size = parse_calendar_cell_size_px(&theme);

    let chrome_override = {
        let mut chrome = ChromeRefinement::default();
        if (web_padding_left - 0.0).abs() < 0.5 {
            chrome = chrome.p(Space::N0);
        } else if (web_padding_left - 12.0).abs() < 0.5 {
            chrome = chrome.p(Space::N3);
        } else if (web_padding_left - 8.0).abs() < 0.5 {
            chrome = chrome.p(Space::N2);
        }
        if web_border_left >= 0.5 {
            chrome = chrome.border_1();
        }
        chrome
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) =
        run_fret_root_frames_with_ui_and_services(bounds, &mut services, 2, |cx| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_headless::calendar::DateRangeSelection;

            let month_model: Model<CalendarMonth> =
                cx.app.models_mut().insert(CalendarMonth::new(year, month));
            match web_selected_dates.as_slice() {
                [] | [_] => {
                    let selected: Model<Option<time::Date>> =
                        cx.app.models_mut().insert(selected_date);
                    let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                        .week_start(week_start)
                        .show_outside_days(web_show_outside_days)
                        .disable_outside_days(web_disable_outside_days)
                        .show_week_number(web_show_week_number)
                        .refine_style(chrome_override.clone());
                    if let Some(cell_size) = cell_size {
                        calendar = calendar.cell_size(cell_size);
                    }
                    if let Some(today) = web_today {
                        calendar = calendar.today(today);
                    }
                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            padding: fret_core::Edges {
                                left: Px(web_origin_x),
                                top: Px(web_origin_y),
                                right: Px(0.0),
                                bottom: Px(0.0),
                            },
                            ..Default::default()
                        },
                        move |cx| vec![calendar.into_element(cx)],
                    )]
                }
                _ if web_is_range_mode => {
                    let (min, max) = web_selected_dates.iter().fold(
                        (web_selected_dates[0], web_selected_dates[0]),
                        |(min, max), d| (min.min(*d), max.max(*d)),
                    );
                    let selected: Model<DateRangeSelection> =
                        cx.app.models_mut().insert(DateRangeSelection {
                            from: Some(min),
                            to: Some(max),
                        });
                    let mut calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
                        .week_start(week_start)
                        .show_outside_days(web_show_outside_days)
                        .disable_outside_days(web_disable_outside_days)
                        .show_week_number(web_show_week_number)
                        .refine_style(chrome_override.clone());
                    if let Some(cell_size) = cell_size {
                        calendar = calendar.cell_size(cell_size);
                    }
                    if let Some(today) = web_today {
                        calendar = calendar.today(today);
                    }
                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            padding: fret_core::Edges {
                                left: Px(web_origin_x),
                                top: Px(web_origin_y),
                                right: Px(0.0),
                                bottom: Px(0.0),
                            },
                            ..Default::default()
                        },
                        move |cx| vec![calendar.into_element(cx)],
                    )]
                }
                _ => {
                    let selected: Model<Vec<time::Date>> =
                        cx.app.models_mut().insert(web_selected_dates.clone());
                    let mut calendar = fret_ui_shadcn::CalendarMultiple::new(month_model, selected)
                        .week_start(week_start)
                        .show_outside_days(web_show_outside_days)
                        .disable_outside_days(web_disable_outside_days)
                        .show_week_number(web_show_week_number)
                        .refine_style(chrome_override.clone());
                    if let Some(cell_size) = cell_size {
                        calendar = calendar.cell_size(cell_size);
                    }
                    if let Some(today) = web_today {
                        calendar = calendar.today(today);
                    }
                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            padding: fret_core::Edges {
                                left: Px(web_origin_x),
                                top: Px(web_origin_y),
                                right: Px(0.0),
                                bottom: Px(0.0),
                            },
                            ..Default::default()
                        },
                        move |cx| vec![calendar.into_element(cx)],
                    )]
                }
            }
        });

    let fret_day_buttons = snap
        .nodes
        .iter()
        .filter(|n| {
            n.role == SemanticsRole::Button
                && n.label
                    .as_deref()
                    .is_some_and(|label| parse_calendar_day_aria_label(label).is_some())
        })
        .count();
    assert_eq!(
        fret_day_buttons,
        web_day_buttons.len(),
        "expected the same number of calendar day buttons for {web_name}"
    );

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    assert_close_px(
        &format!("{web_name} prev button width"),
        prev.bounds.size.width,
        web_prev.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} prev button height"),
        prev.bounds.size.height,
        web_prev.rect.h,
        1.0,
    );

    let day = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some(web_sample_label.as_ref()),
    )
    .expect("fret day semantics node");
    assert_close_px(
        &format!("{web_name} day button width"),
        day.bounds.size.width,
        web_sample.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} day button height"),
        day.bounds.size.height,
        web_sample.rect.h,
        1.0,
    );

    let node_bounds = ui.debug_node_bounds(day.id).expect("fret day node bounds");
    assert_close_px(
        &format!("{web_name} day x"),
        node_bounds.origin.x,
        web_sample.rect.x,
        3.0,
    );
    assert_close_px(
        &format!("{web_name} day y"),
        node_bounds.origin.y,
        web_sample.rect.y,
        3.0,
    );

    if let Some(web_selected) = web_selected {
        let label = web_selected
            .attrs
            .get("aria-label")
            .expect("web selected day label");
        let fret_selected = find_semantics(&snap, SemanticsRole::Button, Some(label))
            .expect("fret selected day semantics node");
        assert!(
            fret_selected.flags.selected,
            "expected fret selected day to have selected semantics flag for {web_name}"
        );
    }
}

fn assert_calendar_multi_month_variant_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    fn parse_css_px(s: &str) -> Option<f32> {
        s.strip_suffix("px")?.parse::<f32>().ok()
    }

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_padding_left = web_rdp_root
        .computed_style
        .get("paddingLeft")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);
    let web_border_left = web_rdp_root
        .computed_style
        .get("borderLeftWidth")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();

    let mut web_month_grids = find_all(&theme.root, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    web_month_grids.sort_by(|a, b| {
        let by_y = a.rect.y.total_cmp(&b.rect.y);
        if !matches!(by_y, std::cmp::Ordering::Equal) {
            return by_y;
        }
        a.rect.x.total_cmp(&b.rect.x)
    });
    assert_eq!(
        web_month_grids.len(),
        2,
        "expected two month grids for {web_name}"
    );

    let month_labels: Vec<(time::Month, i32)> = web_month_grids
        .iter()
        .map(|grid| {
            let label = grid
                .attrs
                .get("aria-label")
                .expect("web month grid aria-label");
            let (m, y) = parse_calendar_title_label(label).expect("web month label (Month YYYY)");
            (m, y)
        })
        .collect();
    let (month_a, year_a) = month_labels[0];
    let (month_b, year_b) = month_labels[1];

    let locale = web_month_grids
        .first()
        .and_then(|grid| grid.attrs.get("aria-label"))
        .and_then(|label| label.chars().next())
        .map(|c| {
            if c.is_ascii_uppercase() {
                fret_ui_shadcn::calendar::CalendarLocale::En
            } else {
                fret_ui_shadcn::calendar::CalendarLocale::Es
            }
        })
        .unwrap_or(fret_ui_shadcn::calendar::CalendarLocale::En);

    let in_view = |d: time::Date| {
        (d.month() == month_a && d.year() == year_a) || (d.month() == month_b && d.year() == year_b)
    };

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");
    let web_next = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Next Month")
    })
    .expect("web next-month button");

    let web_disable_navigation = web_prev
        .attrs
        .get("aria-disabled")
        .is_some_and(|v| v == "true")
        && web_next
            .attrs
            .get("aria-disabled")
            .is_some_and(|v| v == "true");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(
        !web_day_buttons.is_empty(),
        "expected calendar day buttons for {web_name}"
    );

    let web_weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();

    let web_is_range_mode = find_first(&theme.root, &|n| {
        class_has_token(n, "rdp-range_start")
            || class_has_token(n, "rdp-range_middle")
            || class_has_token(n, "rdp-range_end")
    })
    .is_some();

    let web_selected = web_day_buttons
        .iter()
        .find(|n| {
            n.attrs
                .get("aria-label")
                .is_some_and(|label| label.as_str().ends_with(", selected"))
        })
        .copied();
    let selected_date = match web_selected_dates.as_slice() {
        [] => None,
        [d] => Some(*d),
        _ => None,
    };

    let web_show_outside_days =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-outside")).is_some();
    let web_has_out_of_view_days = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).map(|(d, _)| d))
        .any(|d| !in_view(d));

    let web_month_bounds =
        if web_disable_navigation && web_show_outside_days && !web_has_out_of_view_days {
            Some(((month_a, year_a), (month_b, year_b)))
        } else {
            None
        };

    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if in_view(date) {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let web_sample = web_selected.unwrap_or(web_day_buttons[0]);
    let web_sample_label = web_sample
        .attrs
        .get("aria-label")
        .expect("web sample day aria-label")
        .clone();

    let cell_size = parse_calendar_cell_size_px(&theme);

    let chrome_override = {
        let mut chrome = ChromeRefinement::default();
        if (web_padding_left - 0.0).abs() < 0.5 {
            chrome = chrome.p(Space::N0);
        } else if (web_padding_left - 12.0).abs() < 0.5 {
            chrome = chrome.p(Space::N3);
        } else if (web_padding_left - 8.0).abs() < 0.5 {
            chrome = chrome.p(Space::N2);
        }
        if web_border_left >= 0.5 {
            chrome = chrome.border_1();
        }
        chrome
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) =
        run_fret_root_frames_with_ui_and_services(bounds, &mut services, 2, |cx| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_headless::calendar::DateRangeSelection;

            let month_model: Model<CalendarMonth> = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(year_a, month_a));

            match web_selected_dates.as_slice() {
                [] | [_] => {
                    let selected: Model<Option<time::Date>> =
                        cx.app.models_mut().insert(selected_date);
                    let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                        .number_of_months(2)
                        .locale(locale)
                        .disable_navigation(web_disable_navigation)
                        .week_start(week_start)
                        .show_outside_days(web_show_outside_days)
                        .disable_outside_days(web_disable_outside_days)
                        .show_week_number(web_show_week_number)
                        .refine_style(chrome_override.clone());
                    if let Some(((start_month, start_year), (end_month, end_year))) =
                        web_month_bounds
                    {
                        calendar = calendar.month_bounds(
                            CalendarMonth::new(start_year, start_month),
                            CalendarMonth::new(end_year, end_month),
                        );
                    }
                    if let Some(cell_size) = cell_size {
                        calendar = calendar.cell_size(cell_size);
                    }
                    if let Some(today) = web_today {
                        calendar = calendar.today(today);
                    }
                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            padding: fret_core::Edges {
                                left: Px(web_origin_x),
                                top: Px(web_origin_y),
                                right: Px(0.0),
                                bottom: Px(0.0),
                            },
                            ..Default::default()
                        },
                        move |cx| vec![calendar.into_element(cx)],
                    )]
                }
                _ if web_is_range_mode => {
                    let (min, max) = web_selected_dates.iter().fold(
                        (web_selected_dates[0], web_selected_dates[0]),
                        |(min, max), d| (min.min(*d), max.max(*d)),
                    );
                    let selected: Model<DateRangeSelection> =
                        cx.app.models_mut().insert(DateRangeSelection {
                            from: Some(min),
                            to: Some(max),
                        });
                    let mut calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
                        .number_of_months(2)
                        .locale(locale)
                        .disable_navigation(web_disable_navigation)
                        .week_start(week_start)
                        .show_outside_days(web_show_outside_days)
                        .disable_outside_days(web_disable_outside_days)
                        .show_week_number(web_show_week_number)
                        .refine_style(chrome_override.clone());
                    if let Some(((start_month, start_year), (end_month, end_year))) =
                        web_month_bounds
                    {
                        calendar = calendar.month_bounds(
                            CalendarMonth::new(start_year, start_month),
                            CalendarMonth::new(end_year, end_month),
                        );
                    }
                    if let Some(cell_size) = cell_size {
                        calendar = calendar.cell_size(cell_size);
                    }
                    if let Some(today) = web_today {
                        calendar = calendar.today(today);
                    }
                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            padding: fret_core::Edges {
                                left: Px(web_origin_x),
                                top: Px(web_origin_y),
                                right: Px(0.0),
                                bottom: Px(0.0),
                            },
                            ..Default::default()
                        },
                        move |cx| vec![calendar.into_element(cx)],
                    )]
                }
                _ => {
                    let selected: Model<Vec<time::Date>> =
                        cx.app.models_mut().insert(web_selected_dates.clone());
                    let mut calendar = fret_ui_shadcn::CalendarMultiple::new(month_model, selected)
                        .number_of_months(2)
                        .locale(locale)
                        .disable_navigation(web_disable_navigation)
                        .week_start(week_start)
                        .show_outside_days(web_show_outside_days)
                        .disable_outside_days(web_disable_outside_days)
                        .show_week_number(web_show_week_number)
                        .refine_style(chrome_override.clone());

                    if web_name == "calendar-03" {
                        calendar = calendar.required(true).max(5);
                    }
                    if let Some(((start_month, start_year), (end_month, end_year))) =
                        web_month_bounds
                    {
                        calendar = calendar.month_bounds(
                            CalendarMonth::new(start_year, start_month),
                            CalendarMonth::new(end_year, end_month),
                        );
                    }
                    if let Some(cell_size) = cell_size {
                        calendar = calendar.cell_size(cell_size);
                    }
                    if let Some(today) = web_today {
                        calendar = calendar.today(today);
                    }

                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            padding: fret_core::Edges {
                                left: Px(web_origin_x),
                                top: Px(web_origin_y),
                                right: Px(0.0),
                                bottom: Px(0.0),
                            },
                            ..Default::default()
                        },
                        move |cx| vec![calendar.into_element(cx)],
                    )]
                }
            }
        });

    let fret_day_buttons = snap
        .nodes
        .iter()
        .filter(|n| {
            n.role == SemanticsRole::Button
                && n.label
                    .as_deref()
                    .is_some_and(|label| parse_calendar_day_aria_label(label).is_some())
        })
        .count();
    assert_eq!(
        fret_day_buttons,
        web_day_buttons.len(),
        "expected the same number of calendar day buttons for {web_name}"
    );

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    let next = find_semantics(&snap, SemanticsRole::Button, Some("Go to the Next Month"))
        .expect("fret next-month semantics node");

    assert_close_px(
        &format!("{web_name} prev button width"),
        prev.bounds.size.width,
        web_prev.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} prev button height"),
        prev.bounds.size.height,
        web_prev.rect.h,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} next button width"),
        next.bounds.size.width,
        web_next.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} next button height"),
        next.bounds.size.height,
        web_next.rect.h,
        1.0,
    );

    let prev_bounds = ui
        .debug_node_bounds(prev.id)
        .expect("fret prev button node bounds");
    assert_close_px(
        &format!("{web_name} prev x"),
        prev_bounds.origin.x,
        web_prev.rect.x,
        3.0,
    );
    assert_close_px(
        &format!("{web_name} prev y"),
        prev_bounds.origin.y,
        web_prev.rect.y,
        3.0,
    );

    let next_bounds = ui
        .debug_node_bounds(next.id)
        .expect("fret next button node bounds");
    assert_close_px(
        &format!("{web_name} next x"),
        next_bounds.origin.x,
        web_next.rect.x,
        3.0,
    );
    assert_close_px(
        &format!("{web_name} next y"),
        next_bounds.origin.y,
        web_next.rect.y,
        3.0,
    );

    let day = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some(web_sample_label.as_ref()),
    )
    .expect("fret day semantics node");
    assert_close_px(
        &format!("{web_name} day button width"),
        day.bounds.size.width,
        web_sample.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} day button height"),
        day.bounds.size.height,
        web_sample.rect.h,
        1.0,
    );

    let node_bounds = ui.debug_node_bounds(day.id).expect("fret day node bounds");
    assert_close_px(
        &format!("{web_name} day x"),
        node_bounds.origin.x,
        web_sample.rect.x,
        3.0,
    );
    assert_close_px(
        &format!("{web_name} day y"),
        node_bounds.origin.y,
        web_sample.rect.y,
        3.0,
    );
}

#[test]
fn web_vs_fret_layout_calendar_01_background_matches_web() {
    let web = read_web_golden("calendar-01");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;
    let web_bg_css = web_rdp_root
        .computed_style
        .get("backgroundColor")
        .expect("web calendar root backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();
    let selected_date = match web_selected_dates.as_slice() {
        [] => None,
        [d] => Some(*d),
        _ => None,
    };

    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let cell_size = parse_calendar_cell_size_px(&theme);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (_snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(selected_date);

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        if let Some(cell_size) = cell_size {
            calendar = calendar.cell_size(cell_size);
        }
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let target = Rect::new(
        Point::new(Px(web_rdp_root.rect.x), Px(web_rdp_root.rect.y)),
        CoreSize::new(Px(web_rdp_root.rect.w), Px(web_rdp_root.rect.h)),
    );
    let quad = find_best_background_quad(&scene, target).expect("painted calendar background quad");

    assert_rect_xwh_close_px("calendar-01 root quad", quad.rect, web_rdp_root.rect, 3.0);
    assert_rgba_close(
        "calendar-01 root background",
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_selected_day_background_matches_web() {
    let web = read_web_golden("calendar-14");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    // New shadcn/day-picker versions no longer annotate aria-label with ", selected", and
    // aria-selected can live on a containing gridcell instead of the button. Find a selected cell
    // and then locate its day button.
    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));
    let web_bg_css = web_selected_button
        .computed_style
        .get("backgroundColor")
        .expect("web selected day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    // Some calendar variants don't expose the cell size contract via a CSS variable in the golden.
    // Fall back to the measured web day button width to keep the geometry gate stable.
    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_selected_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (_snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let target = Rect::new(
        Point::new(
            Px(web_selected_button.rect.x),
            Px(web_selected_button.rect.y),
        ),
        CoreSize::new(
            Px(web_selected_button.rect.w),
            Px(web_selected_button.rect.h),
        ),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .expect("painted opaque selected day background quad");

    assert_rect_xwh_close_px(
        "calendar-14 selected day quad",
        quad.rect,
        web_selected_button.rect,
        3.0,
    );
    assert_rgba_close(
        "calendar-14 selected day background",
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_vp375x320_selected_day_background_matches_web() {
    let web = read_web_golden("calendar-14.vp375x320");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    // New shadcn/day-picker versions no longer annotate aria-label with ", selected", and
    // aria-selected can live on a containing gridcell instead of the button. Find a selected cell
    // and then locate its day button.
    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));
    let web_bg_css = web_selected_button
        .computed_style
        .get("backgroundColor")
        .expect("web selected day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    // Some calendar variants don't expose the cell size contract via a CSS variable in the golden.
    // Fall back to the measured web day button width to keep the geometry gate stable.
    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_selected_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (_snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let target = Rect::new(
        Point::new(
            Px(web_selected_button.rect.x),
            Px(web_selected_button.rect.y),
        ),
        CoreSize::new(
            Px(web_selected_button.rect.w),
            Px(web_selected_button.rect.h),
        ),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .expect("painted opaque selected day background quad");

    assert_rect_xwh_close_px(
        "calendar-14.vp375x320 selected day quad",
        quad.rect,
        web_selected_button.rect,
        3.0,
    );
    assert_rgba_close(
        "calendar-14.vp375x320 selected day background",
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_hover_day_background_matches_web() {
    let web = read_web_golden("calendar-14.hover-day-13");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));

    let web_hovered_button = web_day_buttons
        .iter()
        .filter(|n| {
            n.computed_style
                .get("backgroundColor")
                .is_some_and(|v| v != "rgba(0, 0, 0, 0)")
        })
        .find(|n| {
            n.attrs
                .get("aria-label")
                .is_some_and(|label| label != web_selected_label)
        })
        .copied()
        .expect("web hovered day button (non-transparent backgroundColor)");
    let web_hovered_label = web_hovered_button
        .attrs
        .get("aria-label")
        .expect("web hovered day aria-label");
    let web_bg_css = web_hovered_button
        .computed_style
        .get("backgroundColor")
        .expect("web hovered day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_hovered_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");

    let hover_button1 = find_semantics(&snap1, SemanticsRole::Button, Some(web_hovered_label))
        .expect("fret hovered day button semantics node (pre-hover)");
    let hover_pos = Point::new(
        Px(hover_button1.bounds.origin.x.0 + hover_button1.bounds.size.width.0 * 0.5),
        Px(hover_button1.bounds.origin.y.0 + hover_button1.bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let target = Rect::new(
        Point::new(Px(web_hovered_button.rect.x), Px(web_hovered_button.rect.y)),
        CoreSize::new(Px(web_hovered_button.rect.w), Px(web_hovered_button.rect.h)),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .expect("painted opaque hovered day background quad");

    assert_rect_xwh_close_px(
        "calendar-14 hover day quad",
        quad.rect,
        web_hovered_button.rect,
        3.0,
    );
    assert_rgba_close(
        "calendar-14 hover day background",
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_vp375x320_hover_day_background_matches_web() {
    let web = read_web_golden("calendar-14.hover-day-13-vp375x320");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));

    let web_hovered_button = web_day_buttons
        .iter()
        .filter(|n| {
            n.computed_style
                .get("backgroundColor")
                .is_some_and(|v| v != "rgba(0, 0, 0, 0)")
        })
        .find(|n| {
            n.attrs
                .get("aria-label")
                .is_some_and(|label| label != web_selected_label)
        })
        .copied()
        .expect("web hovered day button (non-transparent backgroundColor)");
    let web_hovered_label = web_hovered_button
        .attrs
        .get("aria-label")
        .expect("web hovered day aria-label");
    let web_bg_css = web_hovered_button
        .computed_style
        .get("backgroundColor")
        .expect("web hovered day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_hovered_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");

    let hover_button1 = find_semantics(&snap1, SemanticsRole::Button, Some(web_hovered_label))
        .expect("fret hovered day button semantics node (pre-hover)");
    let hover_pos = Point::new(
        Px(hover_button1.bounds.origin.x.0 + hover_button1.bounds.size.width.0 * 0.5),
        Px(hover_button1.bounds.origin.y.0 + hover_button1.bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let target = Rect::new(
        Point::new(Px(web_hovered_button.rect.x), Px(web_hovered_button.rect.y)),
        CoreSize::new(Px(web_hovered_button.rect.w), Px(web_hovered_button.rect.h)),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .expect("painted opaque hovered day background quad");

    assert_rect_xwh_close_px(
        "calendar-14.vp375x320 hover day quad",
        quad.rect,
        web_hovered_button.rect,
        3.0,
    );
    assert_rgba_close(
        "calendar-14.vp375x320 hover day background",
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_selected_day_text_rect_matches_web() {
    let web = read_web_golden("calendar-14");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));

    let web_day_number = {
        let mut stack = vec![web_selected_button];
        let mut best: Option<&WebNode> = None;
        while let Some(node) = stack.pop() {
            for child in &node.children {
                stack.push(child);
            }

            let Some(text) = node.text.as_deref() else {
                continue;
            };
            let text = text.trim();
            if text.is_empty() || text.len() > 2 || !text.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            best = Some(node);
        }
        best.expect("web selected day number text node")
    };

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_selected_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (snap, _scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let fret_selected_button =
        find_semantics(&snap, SemanticsRole::Button, Some(web_selected_label))
            .expect("fret selected day button semantics node");

    let fret_day_number_text = {
        let want = web_day_number.text.as_deref().unwrap_or("").trim();
        assert!(!want.is_empty(), "expected web day number text");

        let mut candidates: Vec<&fret_core::SemanticsNode> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Text)
            .filter(|n| n.label.as_deref() == Some(want))
            .filter(|n| {
                let eps = 0.01;
                let outer = fret_selected_button.bounds;
                let inner = n.bounds;
                inner.origin.x.0 + eps >= outer.origin.x.0
                    && inner.origin.y.0 + eps >= outer.origin.y.0
                    && inner.origin.x.0 + inner.size.width.0
                        <= outer.origin.x.0 + outer.size.width.0 + eps
                    && inner.origin.y.0 + inner.size.height.0
                        <= outer.origin.y.0 + outer.size.height.0 + eps
            })
            .collect();

        candidates.sort_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let bw = b.bounds.size.width.0;
            bw.total_cmp(&aw)
        });
        candidates
            .first()
            .copied()
            .unwrap_or_else(|| panic!("missing fret selected day number text node label={want:?}"))
    };

    // The web golden captures element rects, not glyph bounding boxes. Day numbers are typically
    // flex items whose rect spans the full cell. Gate a high-signal invariant we can check today:
    // the day number text in Fret should be centered within the selected day button.
    let fret_button_cx =
        fret_selected_button.bounds.origin.x.0 + fret_selected_button.bounds.size.width.0 / 2.0;
    let fret_button_cy =
        fret_selected_button.bounds.origin.y.0 + fret_selected_button.bounds.size.height.0 / 2.0;
    let fret_text_cx =
        fret_day_number_text.bounds.origin.x.0 + fret_day_number_text.bounds.size.width.0 / 2.0;
    let fret_text_cy =
        fret_day_number_text.bounds.origin.y.0 + fret_day_number_text.bounds.size.height.0 / 2.0;

    assert_close_px(
        "calendar-14 day number center x ~= button center x",
        Px(fret_text_cx),
        fret_button_cx,
        2.5,
    );
    assert_close_px(
        "calendar-14 day number center y ~= button center y",
        Px(fret_text_cy),
        fret_button_cy,
        2.5,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_vp375x320_selected_day_text_rect_matches_web() {
    let web = read_web_golden("calendar-14.vp375x320");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));

    let web_day_number = {
        let mut stack = vec![web_selected_button];
        let mut best: Option<&WebNode> = None;
        while let Some(node) = stack.pop() {
            for child in &node.children {
                stack.push(child);
            }

            let Some(text) = node.text.as_deref() else {
                continue;
            };
            let text = text.trim();
            if text.is_empty() || text.len() > 2 || !text.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            best = Some(node);
        }
        best.expect("web selected day number text node")
    };

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_selected_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (snap, _scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let fret_selected_button =
        find_semantics(&snap, SemanticsRole::Button, Some(web_selected_label))
            .expect("fret selected day button semantics node");

    let fret_day_number_text = {
        let want = web_day_number.text.as_deref().unwrap_or("").trim();
        assert!(!want.is_empty(), "expected web day number text");

        let mut candidates: Vec<&fret_core::SemanticsNode> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Text)
            .filter(|n| n.label.as_deref() == Some(want))
            .filter(|n| {
                let eps = 0.01;
                let outer = fret_selected_button.bounds;
                let inner = n.bounds;
                inner.origin.x.0 + eps >= outer.origin.x.0
                    && inner.origin.y.0 + eps >= outer.origin.y.0
                    && inner.origin.x.0 + inner.size.width.0
                        <= outer.origin.x.0 + outer.size.width.0 + eps
                    && inner.origin.y.0 + inner.size.height.0
                        <= outer.origin.y.0 + outer.size.height.0 + eps
            })
            .collect();

        candidates.sort_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let bw = b.bounds.size.width.0;
            bw.total_cmp(&aw)
        });
        candidates
            .first()
            .copied()
            .unwrap_or_else(|| panic!("missing fret selected day number text node label={want:?}"))
    };

    // The web golden captures element rects, not glyph bounding boxes. Day numbers are typically
    // flex items whose rect spans the full cell. Gate a high-signal invariant we can check today:
    // the day number text in Fret should be centered within the selected day button.
    let fret_button_cx =
        fret_selected_button.bounds.origin.x.0 + fret_selected_button.bounds.size.width.0 / 2.0;
    let fret_button_cy =
        fret_selected_button.bounds.origin.y.0 + fret_selected_button.bounds.size.height.0 / 2.0;
    let fret_text_cx =
        fret_day_number_text.bounds.origin.x.0 + fret_day_number_text.bounds.size.width.0 / 2.0;
    let fret_text_cy =
        fret_day_number_text.bounds.origin.y.0 + fret_day_number_text.bounds.size.height.0 / 2.0;

    assert_close_px(
        "calendar-14.vp375x320 day number center x ~= button center x",
        Px(fret_text_cx),
        fret_button_cx,
        2.5,
    );
    assert_close_px(
        "calendar-14.vp375x320 day number center y ~= button center y",
        Px(fret_text_cy),
        fret_button_cy,
        2.5,
    );
}

#[test]
fn web_vs_fret_layout_calendar_04_range_middle_day_background_matches_web() {
    let web = read_web_golden("calendar-04");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_middle"))
        .expect("web range-middle day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-middle day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-middle day aria-label");
    assert_calendar_range_day_background_matches_web("calendar-04", "rdp-range_middle", label);
}

#[test]
fn web_vs_fret_layout_calendar_04_range_start_day_background_matches_web() {
    let web = read_web_golden("calendar-04");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_start"))
        .expect("web range-start day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-start day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-start day aria-label");
    assert_calendar_range_day_background_matches_web("calendar-04", "rdp-range_start", label);
}

#[test]
fn web_vs_fret_layout_calendar_04_range_end_day_background_matches_web() {
    let web = read_web_golden("calendar-04");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_end"))
        .expect("web range-end day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-end day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-end day aria-label");
    assert_calendar_range_day_background_matches_web("calendar-04", "rdp-range_end", label);
}

#[test]
fn web_vs_fret_layout_calendar_04_vp375x320_range_middle_day_background_matches_web() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_middle"))
        .expect("web range-middle day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-middle day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-middle day aria-label");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_middle",
        label,
    );
}

#[test]
fn web_vs_fret_layout_calendar_04_vp375x320_range_start_day_background_matches_web() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_start"))
        .expect("web range-start day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-start day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-start day aria-label");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_start",
        label,
    );
}

#[test]
fn web_vs_fret_layout_calendar_04_vp375x320_range_end_day_background_matches_web() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_end"))
        .expect("web range-end day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-end day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-end day aria-label");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_end",
        label,
    );
}

#[test]
fn web_vs_fret_layout_calendar_22_open_background_matches_web() {
    let web = read_web_golden("calendar-22.open");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_bg_css = web_rdp_root
        .computed_style
        .get("backgroundColor")
        .expect("web calendar root backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_day_buttons = find_all_in_theme(theme, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(
        !web_day_buttons.is_empty(),
        "expected calendar day buttons for calendar-22.open"
    );
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let cell_size = parse_calendar_cell_size_px(&theme);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    use fret_ui_headless::calendar::CalendarMonth;
    let open: Model<bool> = app.models_mut().insert(true);
    let month_model: Model<CalendarMonth> =
        app.models_mut().insert(CalendarMonth::new(year, month));
    let selected: Model<Option<time::Date>> = app.models_mut().insert(None);

    let calendar_bg: Rc<Cell<Option<fret_core::Color>>> = Rc::new(Cell::new(None));
    let calendar_bg_for_render = calendar_bg.clone();
    let render = move |cx: &mut fret_ui::ElementContext<'_, App>| {
        use fret_ui_kit::{LengthRefinement, Space};

        let popover =
            fret_ui_shadcn::Popover::new(open.clone()).align(fret_ui_shadcn::PopoverAlign::Start);
        let calendar_bg = calendar_bg_for_render.clone();
        let month_model = month_model.clone();
        let selected = selected.clone();
        vec![popover.into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Select date").into_element(cx),
            move |cx| {
                let mut calendar =
                    fret_ui_shadcn::Calendar::new(month_model.clone(), selected.clone())
                        .week_start(week_start)
                        .show_outside_days(web_show_outside_days)
                        .disable_outside_days(web_disable_outside_days);
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }

                let calendar = calendar.into_element(cx);
                let container_props = match &calendar.kind {
                    fret_ui::element::ElementKind::Container(props) => props,
                    fret_ui::element::ElementKind::LayoutQueryRegion(_) => calendar
                        .children
                        .first()
                        .and_then(|c| match &c.kind {
                            fret_ui::element::ElementKind::Container(props) => Some(props),
                            _ => None,
                        })
                        .expect("expected calendar root container child under LayoutQueryRegion"),
                    other => panic!("expected calendar root container, got {other:?}"),
                };
                let bg = container_props
                    .background
                    .expect("calendar root background (resolved)");
                calendar_bg.set(Some(bg));

                fret_ui_shadcn::PopoverContent::new(vec![calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                    .into_element(cx)
            },
        )]
    };

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    for frame in 1..=2 {
        app.set_frame_id(FrameId(frame));
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "web-vs-fret-layout",
            &render,
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    let actual_bg = calendar_bg
        .get()
        .expect("calendar-22.open calendar root background captured");
    assert_rgba_close(
        "calendar-22.open root background",
        color_to_rgba(actual_bg),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_background_transparent_in_card_content_scope() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(800.0), Px(600.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    use fret_ui_headless::calendar::CalendarMonth;
    let month_model: Model<CalendarMonth> = app
        .models_mut()
        .insert(CalendarMonth::new(2026, time::Month::January));
    let selected: Model<Option<time::Date>> = app.models_mut().insert(None);

    let calendar_bg: Rc<Cell<Option<fret_core::Color>>> = Rc::new(Cell::new(None));
    let calendar_bg_for_render = calendar_bg.clone();
    let render = move |cx: &mut fret_ui::ElementContext<'_, App>| {
        let calendar_bg = calendar_bg_for_render.clone();
        let month_model = month_model.clone();
        let selected = selected.clone();

        vec![fret_ui_shadcn::card::card_content(cx, move |cx| {
            let calendar = fret_ui_shadcn::Calendar::new(month_model.clone(), selected.clone())
                .into_element(cx);
            let container_props = match &calendar.kind {
                fret_ui::element::ElementKind::Container(props) => props,
                fret_ui::element::ElementKind::LayoutQueryRegion(_) => calendar
                    .children
                    .first()
                    .and_then(|c| match &c.kind {
                        fret_ui::element::ElementKind::Container(props) => Some(props),
                        _ => None,
                    })
                    .expect("expected calendar root container child under LayoutQueryRegion"),
                other => panic!("expected calendar root container, got {other:?}"),
            };
            let bg = container_props
                .background
                .expect("calendar root background (resolved)");
            calendar_bg.set(Some(bg));

            [calendar]
        })]
    };

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        &render,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let actual_bg = calendar_bg
        .get()
        .expect("calendar card-content background captured");
    assert!(
        color_to_rgba(actual_bg).a <= 0.001,
        "expected transparent calendar bg inside CardContent, got {:?}",
        color_to_rgba(actual_bg)
    );
}

fn web_find_all_by_data_slot<'a>(root: &'a WebNode, slot: &str) -> Vec<&'a WebNode> {
    find_all(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v == slot)
    })
}

#[test]
fn web_vs_fret_layout_button_as_child_geometry_matches_web() {
    let web = read_web_golden("button-as-child");
    let theme = web_theme(&web);
    let web_link = web_find_by_tag_and_text(&theme.root, "a", "Login").expect("web link");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        vec![fret_ui_shadcn::Button::new("Login").into_element(cx)]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Login"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button");

    assert_close_px(
        "button-as-child w",
        button.bounds.size.width,
        web_link.rect.w,
        4.0,
    );
    assert_close_px(
        "button-as-child h",
        button.bounds.size.height,
        web_link.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_checkbox_disabled_control_size_matches_web() {
    let web = read_web_golden("checkbox-disabled");
    let theme = web_theme(&web);
    let web_checkbox = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "checkbox")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
            && n.attrs.contains_key("data-disabled")
    })
    .expect("web checkbox");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Checkbox::new(model)
                .a11y_label("Checkbox")
                .disabled(true)
                .into_element(cx),
        ]
    });

    let checkbox = find_semantics(&snap, SemanticsRole::Checkbox, Some("Checkbox"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Checkbox, None))
        .expect("fret checkbox semantics node");

    assert_close_px(
        "checkbox-disabled width",
        checkbox.bounds.size.width,
        web_checkbox.rect.w,
        1.0,
    );
    assert_close_px(
        "checkbox-disabled height",
        checkbox.bounds.size.height,
        web_checkbox.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_collapsible_demo_trigger_icon_size_matches_web() {
    let web = read_web_golden("collapsible-demo");
    let theme = web_theme(&web);

    let web_trigger = find_first(&theme.root, &|n| {
        n.tag == "button" && class_has_token(n, "size-8")
    })
    .expect("web trigger");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let open: Model<bool> = cx.app.models_mut().insert(false);

        let trigger = fret_ui_shadcn::Button::new("Toggle")
            .variant(fret_ui_shadcn::ButtonVariant::Ghost)
            .size(fret_ui_shadcn::ButtonSize::IconSm)
            .children(vec![decl_icon::icon(cx, fret_icons::ids::ui::CHEVRON_DOWN)])
            .into_element(cx);

        let header = cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                gap: Px(16.0),
                padding: Edges::symmetric(Px(16.0), Px(0.0)),
                justify: MainAlign::SpaceBetween,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                vec![
                    ui::text(cx, "@peduarte starred 3 repositories")
                        .font_semibold()
                        .into_element(cx),
                    trigger,
                ]
            },
        );

        let item = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                border: Edges::all(Px(1.0)),
                padding: Edges::symmetric(Px(16.0), Px(8.0)),
                ..Default::default()
            },
            move |cx| vec![ui::text(cx, "@radix-ui/primitives").into_element(cx)],
        );

        let trigger_stack = cx.column(
            ColumnProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                gap: Px(8.0),
                ..Default::default()
            },
            move |_cx| vec![header, item],
        );

        vec![fret_ui_shadcn::Collapsible::new(open).into_element(
            cx,
            move |_cx, _is_open| trigger_stack,
            move |cx| {
                cx.column(
                    ColumnProps {
                        layout: LayoutStyle::default(),
                        gap: Px(8.0),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            ui::text(cx, "@radix-ui/colors").into_element(cx),
                            ui::text(cx, "@stitches/react").into_element(cx),
                        ]
                    },
                )
            },
        )]
    });

    let trigger = find_semantics(&snap, SemanticsRole::Button, Some("Toggle"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret trigger");

    assert_close_px(
        "collapsible-demo trigger w",
        trigger.bounds.size.width,
        web_trigger.rect.w,
        1.0,
    );
    assert_close_px(
        "collapsible-demo trigger h",
        trigger.bounds.size.height,
        web_trigger.rect.h,
        1.0,
    );
}
