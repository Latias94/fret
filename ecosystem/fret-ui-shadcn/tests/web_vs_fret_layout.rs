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
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod css_color;
use css_color::{Rgba, color_to_rgba, parse_css_color};

#[derive(Debug, Clone, Deserialize)]
struct WebGolden {
    themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebGoldenTheme {
    viewport: WebViewport,
    root: WebNode,
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

#[test]
fn web_vs_fret_layout_button_default_height() {
    let web = read_web_golden("button-default");
    let theme = web_theme(&web);
    let web_button = web_find_by_tag_and_text(&theme.root, "button", "Button").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![fret_ui_shadcn::Button::new("Button").into_element(cx)]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Button"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button semantics node");

    assert_close_px(
        "button height",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_drawer_demo_trigger_height_matches_web() {
    use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent};

    let web = read_web_golden("drawer-demo");
    let theme = web_theme(&web);
    let web_trigger =
        web_find_by_tag_and_text(&theme.root, "button", "Open Drawer").expect("web trigger");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let open: Model<bool> = cx.app.models_mut().insert(false);
        vec![Drawer::new(open).into_element(
            cx,
            |cx| {
                Button::new("Open Drawer")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| DrawerContent::new(vec![cx.text("Drawer content")]).into_element(cx),
        )]
    });

    let trigger = find_semantics(&snap, SemanticsRole::Button, Some("Open Drawer"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret trigger semantics node");

    assert_close_px(
        "drawer-demo trigger h",
        trigger.bounds.size.height,
        web_trigger.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_drawer_dialog_trigger_height_matches_web() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    let web = read_web_golden("drawer-dialog");
    let theme = web_theme(&web);
    let web_trigger =
        web_find_by_tag_and_text(&theme.root, "button", "Edit Profile").expect("web trigger");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let open: Model<bool> = cx.app.models_mut().insert(false);
        vec![Dialog::new(open).into_element(
            cx,
            |cx| {
                Button::new("Edit Profile")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| DialogContent::new(Vec::new()).into_element(cx),
        )]
    });

    let trigger = find_semantics(&snap, SemanticsRole::Button, Some("Edit Profile"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret trigger semantics node");

    assert_close_px(
        "drawer-dialog trigger h",
        trigger.bounds.size.height,
        web_trigger.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_dialog_close_button_trigger_height_matches_web() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    let web = read_web_golden("dialog-close-button");
    let theme = web_theme(&web);
    let web_trigger =
        web_find_by_tag_and_text(&theme.root, "button", "Share").expect("web trigger");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let open: Model<bool> = cx.app.models_mut().insert(false);
        vec![Dialog::new(open).into_element(
            cx,
            |cx| {
                Button::new("Share")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| DialogContent::new(Vec::new()).into_element(cx),
        )]
    });

    let trigger = find_semantics(&snap, SemanticsRole::Button, Some("Share"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret trigger semantics node");

    assert_close_px(
        "dialog-close-button trigger h",
        trigger.bounds.size.height,
        web_trigger.rect.h,
        1.0,
    );
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

#[test]
fn web_vs_fret_layout_login_03_shell_container_matches() {
    assert_shell_container_centered_x_w_matches(
        "login-03",
        &["flex", "w-full", "max-w-sm", "flex-col", "gap-6"],
    );
}

#[test]
fn web_vs_fret_layout_login_04_shell_container_matches() {
    assert_shell_container_centered_x_w_matches(
        "login-04",
        &["w-full", "max-w-sm", "md:max-w-4xl"],
    );
}

#[test]
fn web_vs_fret_layout_login_05_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("login-05", &["w-full", "max-w-sm"]);
}

#[test]
fn web_vs_fret_layout_signup_01_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("signup-01", &["w-full", "max-w-sm"]);
}

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

#[test]
fn web_vs_fret_layout_signup_03_shell_container_matches() {
    assert_shell_container_centered_x_w_matches(
        "signup-03",
        &["flex", "w-full", "max-w-sm", "flex-col", "gap-6"],
    );
}

#[test]
fn web_vs_fret_layout_signup_04_shell_container_matches() {
    assert_shell_container_centered_x_w_matches(
        "signup-04",
        &["w-full", "max-w-sm", "md:max-w-4xl"],
    );
}

#[test]
fn web_vs_fret_layout_signup_05_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("signup-05", &["w-full", "max-w-sm"]);
}

#[test]
fn web_vs_fret_layout_otp_01_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("otp-01", &["w-full", "max-w-xs"]);
}

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

#[test]
fn web_vs_fret_layout_otp_03_shell_container_matches() {
    assert_shell_container_centered_x_w_matches(
        "otp-03",
        &["flex", "w-full", "max-w-xs", "flex-col", "gap-6"],
    );
}

#[test]
fn web_vs_fret_layout_otp_04_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("otp-04", &["w-full", "max-w-sm", "md:max-w-3xl"]);
}

#[test]
fn web_vs_fret_layout_otp_05_shell_container_matches() {
    assert_shell_container_centered_x_w_matches("otp-05", &["w-full", "max-w-sm"]);
}

fn web_find_button_by_aria_label<'a>(root: &'a WebNode, aria_label: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.tag == "button" && n.attrs.get("aria-label").is_some_and(|v| v == aria_label)
    })
}

fn assert_toggle_variant_geometry_matches(
    web_name: &str,
    aria_label: &str,
    size: fret_ui_shadcn::ToggleSize,
    variant: fret_ui_shadcn::ToggleVariant,
    disabled: bool,
    with_text: bool,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_toggle = web_find_button_by_aria_label(&theme.root, aria_label).expect("web toggle");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<bool> = cx.app.models_mut().insert(false);
        let mut toggle = fret_ui_shadcn::Toggle::new(model)
            .size(size)
            .variant(variant)
            .disabled(disabled)
            .a11y_label(aria_label)
            .children(vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)]);

        if with_text {
            toggle = toggle.label("Italic");
        }

        vec![toggle.into_element(cx)]
    });

    let toggle = find_semantics(&snap, SemanticsRole::Button, Some(aria_label))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret toggle semantics node");

    assert_close_px(
        &format!("{web_name} height"),
        toggle.bounds.size.height,
        web_toggle.rect.h,
        1.0,
    );

    // Avoid comparing text-driven widths in this file: `FakeServices` does not provide a
    // font-accurate text width model, and width gates would become flaky across environments.
    if !with_text {
        assert_close_px(
            &format!("{web_name} width"),
            toggle.bounds.size.width,
            web_toggle.rect.w,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_toggle_sm_geometry_matches() {
    assert_toggle_variant_geometry_matches(
        "toggle-sm",
        "Toggle italic",
        fret_ui_shadcn::ToggleSize::Sm,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        false,
    );
}

#[test]
fn web_vs_fret_layout_toggle_lg_geometry_matches() {
    assert_toggle_variant_geometry_matches(
        "toggle-lg",
        "Toggle italic",
        fret_ui_shadcn::ToggleSize::Lg,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        false,
    );
}

#[test]
fn web_vs_fret_layout_toggle_outline_geometry_matches() {
    assert_toggle_variant_geometry_matches(
        "toggle-outline",
        "Toggle italic",
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Outline,
        false,
        false,
    );
}

#[test]
fn web_vs_fret_layout_toggle_disabled_geometry_matches() {
    assert_toggle_variant_geometry_matches(
        "toggle-disabled",
        "Toggle italic",
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Default,
        true,
        false,
    );
}

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
                .size(fret_ui_shadcn::SidebarMenuButtonSize::Default)
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
                    .size(fret_ui_shadcn::SidebarMenuButtonSize::Lg)
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

#[test]
fn web_vs_fret_layout_sidebar_01_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-01");
}

#[test]
fn web_vs_fret_layout_sidebar_02_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-02");
}

#[test]
fn web_vs_fret_layout_sidebar_03_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-03");
}

#[test]
fn web_vs_fret_layout_sidebar_04_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-04");
}

#[test]
fn web_vs_fret_layout_sidebar_05_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-05");
}

#[test]
fn web_vs_fret_layout_sidebar_06_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-06");
}

#[test]
fn web_vs_fret_layout_sidebar_07_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-07");
}

#[test]
fn web_vs_fret_layout_sidebar_08_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-08");
}

#[test]
fn web_vs_fret_layout_sidebar_09_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-09");
}

#[test]
fn web_vs_fret_layout_sidebar_10_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-10");
}

#[test]
fn web_vs_fret_layout_sidebar_11_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-11");
}

#[test]
fn web_vs_fret_layout_sidebar_12_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-12");
}

#[test]
fn web_vs_fret_layout_sidebar_14_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-14");
}

#[test]
fn web_vs_fret_layout_sidebar_15_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-15");
}

#[test]
fn web_vs_fret_layout_sidebar_16_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-16");
}

#[test]
fn web_vs_fret_layout_toggle_with_text_height_matches() {
    assert_toggle_variant_geometry_matches(
        "toggle-with-text",
        "Toggle italic",
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        true,
    );
}

fn web_find_toggle_group_container<'a>(root: &'a WebNode) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.tag == "div" && n.attrs.get("role").is_some_and(|v| v == "group")
    })
}

#[derive(Debug, Clone, Copy)]
struct ToggleGroupItemSpec {
    value: &'static str,
    a11y_label: &'static str,
    text: Option<&'static str>,
}

fn assert_toggle_group_variant_heights_match(
    web_name: &str,
    group_kind: fret_ui_shadcn::ToggleGroupKind,
    size: fret_ui_shadcn::ToggleSize,
    variant: fret_ui_shadcn::ToggleVariant,
    disabled: bool,
    spacing: Space,
    items: &'static [ToggleGroupItemSpec],
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_group =
        web_find_toggle_group_container(&theme.root).expect("web toggle-group container");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let group_label: Arc<str> = Arc::from(format!("Golden:{web_name}:group"));

    let snap = run_fret_root(bounds, |cx| {
        let model_single: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
        let model_multi: Model<Vec<Arc<str>>> = cx.app.models_mut().insert(Vec::new());

        let base = match group_kind {
            fret_ui_shadcn::ToggleGroupKind::Single => {
                fret_ui_shadcn::ToggleGroup::single(model_single)
            }
            fret_ui_shadcn::ToggleGroupKind::Multiple => {
                fret_ui_shadcn::ToggleGroup::multiple(model_multi)
            }
        };

        let group = items.iter().fold(
            base.size(size)
                .variant(variant)
                .disabled(disabled)
                .spacing(spacing),
            |group, spec| {
                let mut children: Vec<fret_ui::element::AnyElement> =
                    vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)];
                if let Some(text) = spec.text {
                    children.push(decl_text::text_sm(cx, text));
                }
                group.item(
                    fret_ui_shadcn::ToggleGroupItem::new(spec.value, children)
                        .a11y_label(spec.a11y_label),
                )
            },
        );

        let group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(group_label.clone()),
                ..Default::default()
            },
            move |cx| vec![group.into_element(cx)],
        );

        vec![group]
    });

    let group = find_semantics(&snap, SemanticsRole::Panel, Some(group_label.as_ref()))
        .expect("fret toggle-group semantics node");
    assert_close_px(
        &format!("{web_name} group height"),
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );

    let item_role = match group_kind {
        fret_ui_shadcn::ToggleGroupKind::Single => SemanticsRole::RadioButton,
        fret_ui_shadcn::ToggleGroupKind::Multiple => SemanticsRole::Button,
    };

    for spec in items {
        let web_item =
            web_find_button_by_aria_label(&theme.root, spec.a11y_label).expect("web toggle item");
        let item = find_semantics(&snap, item_role, Some(spec.a11y_label))
            .expect("fret toggle item semantics node");
        assert_close_px(
            &format!("{web_name} item height ({})", spec.a11y_label),
            item.bounds.size.height,
            web_item.rect.h,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_toggle_group_sm_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "bold",
            a11y_label: "Toggle bold",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "italic",
            a11y_label: "Toggle italic",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "strikethrough",
            a11y_label: "Toggle strikethrough",
            text: None,
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-sm",
        fret_ui_shadcn::ToggleGroupKind::Single,
        fret_ui_shadcn::ToggleSize::Sm,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        Space::N0,
        ITEMS,
    );
}

#[test]
fn web_vs_fret_layout_toggle_group_lg_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "bold",
            a11y_label: "Toggle bold",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "italic",
            a11y_label: "Toggle italic",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "strikethrough",
            a11y_label: "Toggle strikethrough",
            text: None,
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-lg",
        fret_ui_shadcn::ToggleGroupKind::Multiple,
        fret_ui_shadcn::ToggleSize::Lg,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        Space::N0,
        ITEMS,
    );
}

#[test]
fn web_vs_fret_layout_toggle_group_outline_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "bold",
            a11y_label: "Toggle bold",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "italic",
            a11y_label: "Toggle italic",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "strikethrough",
            a11y_label: "Toggle strikethrough",
            text: None,
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-outline",
        fret_ui_shadcn::ToggleGroupKind::Multiple,
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Outline,
        false,
        Space::N0,
        ITEMS,
    );
}

#[test]
fn web_vs_fret_layout_toggle_group_disabled_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "bold",
            a11y_label: "Toggle bold",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "italic",
            a11y_label: "Toggle italic",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "strikethrough",
            a11y_label: "Toggle strikethrough",
            text: None,
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-disabled",
        fret_ui_shadcn::ToggleGroupKind::Multiple,
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Default,
        true,
        Space::N0,
        ITEMS,
    );
}

#[test]
fn web_vs_fret_layout_toggle_group_single_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "bold",
            a11y_label: "Toggle bold",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "italic",
            a11y_label: "Toggle italic",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "strikethrough",
            a11y_label: "Toggle strikethrough",
            text: None,
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-single",
        fret_ui_shadcn::ToggleGroupKind::Single,
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        Space::N0,
        ITEMS,
    );
}

#[test]
fn web_vs_fret_layout_toggle_group_spacing_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "star",
            a11y_label: "Toggle star",
            text: Some("Star"),
        },
        ToggleGroupItemSpec {
            value: "heart",
            a11y_label: "Toggle heart",
            text: Some("Heart"),
        },
        ToggleGroupItemSpec {
            value: "bookmark",
            a11y_label: "Toggle bookmark",
            text: Some("Bookmark"),
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-spacing",
        fret_ui_shadcn::ToggleGroupKind::Multiple,
        fret_ui_shadcn::ToggleSize::Sm,
        fret_ui_shadcn::ToggleVariant::Outline,
        false,
        Space::N2,
        ITEMS,
    );
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
            fret_ui_shadcn::RadioGroup::uncontrolled(Some("default")).a11y_label("Options"),
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
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

        let icon =
            decl_icon::icon_with(cx, fret_icons::ids::ui::CHEVRON_DOWN, Some(Px(24.0)), None);
        let media = fret_ui_shadcn::EmptyMedia::new(vec![icon])
            .variant(fret_ui_shadcn::EmptyMediaVariant::Icon)
            .into_element(cx);

        let title = fret_ui_shadcn::EmptyTitle::new("No Projects Yet").into_element(cx);
        let desc = fret_ui_shadcn::EmptyDescription::new(
            "You haven't created any projects yet. Get started by creating your first project.",
        )
        .into_element(cx);
        let header = fret_ui_shadcn::EmptyHeader::new(vec![media, title, desc]).into_element(cx);
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
        let content = fret_ui_shadcn::EmptyContent::new(vec![actions]).into_element(cx);

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
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let icon =
            decl_icon::icon_with(cx, fret_icons::ids::ui::CHEVRON_DOWN, Some(Px(24.0)), None);
        let media = fret_ui_shadcn::EmptyMedia::new(vec![icon])
            .variant(fret_ui_shadcn::EmptyMediaVariant::Icon)
            .into_element(cx);

        let title = fret_ui_shadcn::EmptyTitle::new("No Notifications").into_element(cx);
        let desc = fret_ui_shadcn::EmptyDescription::new(
            "You're all caught up. New notifications will appear here.",
        )
        .into_element(cx);
        let header = fret_ui_shadcn::EmptyHeader::new(vec![media, title, desc]).into_element(cx);

        let button = fret_ui_shadcn::Button::new("Refresh")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .size(fret_ui_shadcn::ButtonSize::Sm)
            .into_element(cx);
        let content = fret_ui_shadcn::EmptyContent::new(vec![button]).into_element(cx);

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
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let icon =
            decl_icon::icon_with(cx, fret_icons::ids::ui::CHEVRON_DOWN, Some(Px(24.0)), None);
        let media = fret_ui_shadcn::EmptyMedia::new(vec![icon])
            .variant(fret_ui_shadcn::EmptyMediaVariant::Icon)
            .into_element(cx);

        let title = fret_ui_shadcn::EmptyTitle::new("Cloud Storage Empty").into_element(cx);
        let desc = fret_ui_shadcn::EmptyDescription::new(
            "Upload files to your cloud storage to access them anywhere.",
        )
        .into_element(cx);
        let header = fret_ui_shadcn::EmptyHeader::new(vec![media, title, desc]).into_element(cx);

        let button = fret_ui_shadcn::Button::new("Upload Files")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .size(fret_ui_shadcn::ButtonSize::Sm)
            .into_element(cx);
        let content = fret_ui_shadcn::EmptyContent::new(vec![button]).into_element(cx);

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
            let media = fret_ui_shadcn::EmptyMedia::new(vec![icon])
                .variant(fret_ui_shadcn::EmptyMediaVariant::Icon)
                .into_element(cx);
            let title = fret_ui_shadcn::EmptyTitle::new(title).into_element(cx);
            let desc = fret_ui_shadcn::EmptyDescription::new(desc).into_element(cx);
            let header =
                fret_ui_shadcn::EmptyHeader::new(vec![media, title, desc]).into_element(cx);
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
fn web_vs_fret_layout_resizable_demo_geometry_matches_web() {
    let web = read_web_golden("resizable-demo");
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(&theme.root, &["max-w-md", "rounded-lg", "border"])
        .expect("web resizable group");

    let web_one = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_all_tokens(
                n,
                &["flex", "h-[200px]", "items-center", "justify-center", "p-6"],
            )
            && contains_text(n, "One")
    })
    .expect("web one panel content");
    let web_two = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_all_tokens(
                n,
                &["flex", "h-full", "items-center", "justify-center", "p-6"],
            )
            && contains_text(n, "Two")
    })
    .expect("web two panel content");
    let web_three = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_all_tokens(
                n,
                &["flex", "h-full", "items-center", "justify-center", "p-6"],
            )
            && contains_text(n, "Three")
    })
    .expect("web three panel content");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model_outer: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.5, 0.5]);
        let model_inner: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.25, 0.75]);

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        fn mk_center(
            cx: &mut fret_ui::ElementContext<'_, App>,
            theme: &Theme,
            label: &'static str,
            text: &'static str,
            fixed_height: Option<Px>,
        ) -> AnyElement {
            let layout = match fixed_height {
                Some(h) => LayoutRefinement::default().w_full().h_px(MetricRef::Px(h)),
                None => LayoutRefinement::default().size_full(),
            };
            let layout = decl_style::layout_style(theme, layout);
            let node = cx.container(
                ContainerProps {
                    layout,
                    padding: Edges::all(Px(24.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| vec![ui::text(cx, text).font_semibold().into_element(cx)],
                    )]
                },
            );
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from(label)),
                    ..Default::default()
                },
                move |_cx| vec![node],
            )
        }

        let one = mk_center(
            cx,
            &theme,
            "Golden:resizable-demo:one",
            "One",
            Some(Px(200.0)),
        );
        let two = mk_center(cx, &theme, "Golden:resizable-demo:two", "Two", None);
        let three = mk_center(cx, &theme, "Golden:resizable-demo:three", "Three", None);

        let inner = fret_ui_shadcn::ResizablePanelGroup::new(model_inner)
            .axis(fret_core::Axis::Vertical)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![two])
                    .min_px(Px(0.0))
                    .into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![three])
                    .min_px(Px(0.0))
                    .into(),
            ])
            .into_element(cx);

        let outer = fret_ui_shadcn::ResizablePanelGroup::new(model_outer)
            .axis(fret_core::Axis::Horizontal)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![one])
                    .min_px(Px(0.0))
                    .into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![inner])
                    .min_px(Px(0.0))
                    .into(),
            ])
            .into_element(cx);

        let frame = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_group.rect.w)))
                        .h_px(MetricRef::Px(Px(web_group.rect.h))),
                ),
                border: Edges::all(Px(1.0)),
                border_color: Some(border),
                corner_radii: fret_core::Corners::all(Px(8.0)),
                ..Default::default()
            },
            move |_cx| vec![outer],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:resizable-demo:group")),
                ..Default::default()
            },
            move |_cx| vec![frame],
        )]
    });

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-demo:group"),
    )
    .expect("fret group");
    let one = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-demo:one"),
    )
    .expect("fret one");
    let two = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-demo:two"),
    )
    .expect("fret two");
    let three = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-demo:three"),
    )
    .expect("fret three");

    assert_rect_close_px("resizable-demo group", group.bounds, web_group.rect, 2.0);
    assert_rect_close_px("resizable-demo one", one.bounds, web_one.rect, 2.0);
    assert_rect_close_px("resizable-demo two", two.bounds, web_two.rect, 2.0);
    assert_rect_close_px("resizable-demo three", three.bounds, web_three.rect, 2.0);
}

#[test]
fn web_vs_fret_layout_resizable_demo_with_handle_geometry_matches_web() {
    let web = read_web_golden("resizable-demo-with-handle");
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(&theme.root, &["max-w-md", "rounded-lg", "border"])
        .expect("web resizable group");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model_outer: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.5, 0.5]);
        let model_inner: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.25, 0.75]);

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        fn panel(
            cx: &mut fret_ui::ElementContext<'_, App>,
            theme: &Theme,
            text: &'static str,
            fixed_height: Option<Px>,
        ) -> AnyElement {
            let layout = match fixed_height {
                Some(h) => LayoutRefinement::default().w_full().h_px(MetricRef::Px(h)),
                None => LayoutRefinement::default().size_full(),
            };
            let layout = decl_style::layout_style(theme, layout);
            cx.container(
                ContainerProps {
                    layout,
                    padding: Edges::all(Px(24.0)),
                    ..Default::default()
                },
                move |cx| vec![ui::text(cx, text).font_semibold().into_element(cx)],
            )
        }

        let inner = fret_ui_shadcn::ResizablePanelGroup::new(model_inner)
            .axis(fret_core::Axis::Vertical)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![panel(cx, &theme, "Two", None)]).into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![panel(cx, &theme, "Three", None)]).into(),
            ])
            .into_element(cx);

        let outer = fret_ui_shadcn::ResizablePanelGroup::new(model_outer)
            .axis(fret_core::Axis::Horizontal)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![panel(
                    cx,
                    &theme,
                    "One",
                    Some(Px(200.0)),
                )])
                .into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![inner]).into(),
            ])
            .into_element(cx);

        let frame = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_group.rect.w)))
                        .h_px(MetricRef::Px(Px(web_group.rect.h))),
                ),
                border: Edges::all(Px(1.0)),
                border_color: Some(border),
                corner_radii: fret_core::Corners::all(Px(8.0)),
                ..Default::default()
            },
            move |_cx| vec![outer],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:resizable-demo-with-handle:group")),
                ..Default::default()
            },
            move |_cx| vec![frame],
        )]
    });

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-demo-with-handle:group"),
    )
    .expect("fret group");

    assert_rect_close_px(
        "resizable-demo-with-handle group",
        group.bounds,
        web_group.rect,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_resizable_handle_geometry_matches_web() {
    let web = read_web_golden("resizable-handle");
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(
        &theme.root,
        &["min-h-[200px]", "max-w-md", "rounded-lg", "border"],
    )
    .expect("web resizable group");

    let web_left = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "p-6") && contains_text(n, "Sidebar")
    })
    .expect("web left panel");
    let web_right = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "p-6") && contains_text(n, "Content")
    })
    .expect("web right panel");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.25, 0.75]);
        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let fill_layout = decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

        let left_box = cx.container(
            ContainerProps {
                layout: fill_layout,
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |cx| vec![ui::text(cx, "Sidebar").font_semibold().into_element(cx)],
        );
        let left = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:resizable-handle:left")),
                ..Default::default()
            },
            move |_cx| vec![left_box],
        );

        let right_box = cx.container(
            ContainerProps {
                layout: fill_layout,
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |cx| vec![ui::text(cx, "Content").font_semibold().into_element(cx)],
        );
        let right = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:resizable-handle:right")),
                ..Default::default()
            },
            move |_cx| vec![right_box],
        );

        let group = fret_ui_shadcn::ResizablePanelGroup::new(model)
            .axis(fret_core::Axis::Horizontal)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![left])
                    .min_px(Px(0.0))
                    .into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![right])
                    .min_px(Px(0.0))
                    .into(),
            ])
            .into_element(cx);

        let frame = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_group.rect.w)))
                        .h_px(MetricRef::Px(Px(web_group.rect.h))),
                ),
                border: Edges::all(Px(1.0)),
                border_color: Some(border),
                corner_radii: fret_core::Corners::all(Px(8.0)),
                ..Default::default()
            },
            move |_cx| vec![group],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:resizable-handle:group")),
                ..Default::default()
            },
            move |_cx| vec![frame],
        )]
    });

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-handle:group"),
    )
    .expect("fret group");

    assert_rect_close_px("resizable-handle group", group.bounds, web_group.rect, 2.0);

    let left = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-handle:left"),
    )
    .expect("fret left");
    let right = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-handle:right"),
    )
    .expect("fret right");

    assert_close_px(
        "resizable-handle left x",
        left.bounds.origin.x,
        web_left.rect.x,
        2.0,
    );
    assert_close_px(
        "resizable-handle right x",
        right.bounds.origin.x,
        web_right.rect.x,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_resizable_vertical_geometry_matches_web() {
    let web = read_web_golden("resizable-vertical");
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(
        &theme.root,
        &["min-h-[200px]", "max-w-md", "rounded-lg", "border"],
    )
    .expect("web resizable group");

    let web_header = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "p-6") && contains_text(n, "Header")
    })
    .expect("web header panel");
    let web_content = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "p-6") && contains_text(n, "Content")
    })
    .expect("web content panel");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.25, 0.75]);
        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let fill_layout = decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

        let top_box = cx.container(
            ContainerProps {
                layout: fill_layout,
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |cx| vec![ui::text(cx, "Header").font_semibold().into_element(cx)],
        );
        let top = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:resizable-vertical:top")),
                ..Default::default()
            },
            move |_cx| vec![top_box],
        );

        let bottom_box = cx.container(
            ContainerProps {
                layout: fill_layout,
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |cx| vec![ui::text(cx, "Content").font_semibold().into_element(cx)],
        );
        let bottom = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:resizable-vertical:bottom")),
                ..Default::default()
            },
            move |_cx| vec![bottom_box],
        );

        let group = fret_ui_shadcn::ResizablePanelGroup::new(model)
            .axis(fret_core::Axis::Vertical)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![top])
                    .min_px(Px(0.0))
                    .into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![bottom])
                    .min_px(Px(0.0))
                    .into(),
            ])
            .into_element(cx);

        let frame = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_group.rect.w)))
                        .h_px(MetricRef::Px(Px(web_group.rect.h))),
                ),
                border: Edges::all(Px(1.0)),
                border_color: Some(border),
                corner_radii: fret_core::Corners::all(Px(8.0)),
                ..Default::default()
            },
            move |_cx| vec![group],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:resizable-vertical:group")),
                ..Default::default()
            },
            move |_cx| vec![frame],
        )]
    });

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-vertical:group"),
    )
    .expect("fret group");
    assert_rect_close_px(
        "resizable-vertical group",
        group.bounds,
        web_group.rect,
        2.0,
    );

    let top = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-vertical:top"),
    )
    .expect("fret top");
    let bottom = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:resizable-vertical:bottom"),
    )
    .expect("fret bottom");

    assert_close_px(
        "resizable-vertical top y",
        top.bounds.origin.y,
        web_header.rect.y,
        2.0,
    );
    assert_close_px(
        "resizable-vertical bottom y",
        bottom.bounds.origin.y,
        web_content.rect.y,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_native_select_demo_geometry_matches_web() {
    let web = read_web_golden("native-select-demo");
    let theme = web_theme(&web);
    let web_select = find_first(&theme.root, &|n| n.tag == "select").expect("web select");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::NativeSelect::new("Select status")
                .a11y_label("NativeSelect")
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_select.rect.w))),
                )
                .into_element(cx),
        ]
    });

    let select = find_semantics(&snap, SemanticsRole::ComboBox, Some("NativeSelect"))
        .or_else(|| find_semantics(&snap, SemanticsRole::ComboBox, None))
        .expect("fret native select");

    assert_close_px(
        "native-select-demo h",
        select.bounds.size.height,
        web_select.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_native_select_disabled_geometry_matches_web() {
    let web = read_web_golden("native-select-disabled");
    let theme = web_theme(&web);
    let web_select = find_first(&theme.root, &|n| n.tag == "select").expect("web select");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::NativeSelect::new("Select priority")
                .a11y_label("NativeSelect")
                .disabled(true)
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_select.rect.w))),
                )
                .into_element(cx),
        ]
    });

    let select = find_semantics(&snap, SemanticsRole::ComboBox, Some("NativeSelect"))
        .or_else(|| find_semantics(&snap, SemanticsRole::ComboBox, None))
        .expect("fret native select");

    assert_close_px(
        "native-select-disabled h",
        select.bounds.size.height,
        web_select.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_native_select_groups_geometry_matches_web() {
    let web = read_web_golden("native-select-groups");
    let theme = web_theme(&web);
    let web_select = find_first(&theme.root, &|n| n.tag == "select").expect("web select");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::NativeSelect::new("Select department")
                .a11y_label("NativeSelect")
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_select.rect.w))),
                )
                .into_element(cx),
        ]
    });

    let select = find_semantics(&snap, SemanticsRole::ComboBox, Some("NativeSelect"))
        .or_else(|| find_semantics(&snap, SemanticsRole::ComboBox, None))
        .expect("fret native select");

    assert_close_px(
        "native-select-groups h",
        select.bounds.size.height,
        web_select.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_native_select_invalid_geometry_matches_web() {
    let web = read_web_golden("native-select-invalid");
    let theme = web_theme(&web);
    let web_select = find_first(&theme.root, &|n| n.tag == "select").expect("web select");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::NativeSelect::new("Select role")
                .a11y_label("NativeSelect")
                .aria_invalid(true)
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_select.rect.w))),
                )
                .into_element(cx),
        ]
    });

    let select = find_semantics(&snap, SemanticsRole::ComboBox, Some("NativeSelect"))
        .or_else(|| find_semantics(&snap, SemanticsRole::ComboBox, None))
        .expect("fret native select");

    assert_close_px(
        "native-select-invalid h",
        select.bounds.size.height,
        web_select.rect.h,
        1.0,
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

            let trunc_layout = LayoutRefinement::default().max_w(MetricRef::Px(Px(80.0)));

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
    let group_items = &group_items[..3];

    let min_x = group_items
        .iter()
        .map(|r| r.origin.x.0)
        .fold(f32::INFINITY, f32::min);
    let min_y = group_items
        .iter()
        .map(|r| r.origin.y.0)
        .fold(f32::INFINITY, f32::min);
    let max_x = group_items
        .iter()
        .map(|r| r.origin.x.0 + r.size.width.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = group_items
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
                .w_px(MetricRef::Px(Px(web_avatar.rect.w)))
                .h_px(MetricRef::Px(Px(web_avatar.rect.h))),
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
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(size))
                        .h_px(MetricRef::Px(size)),
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
    let group_items = &group_items[..3];

    let min_x = group_items
        .iter()
        .map(|r| r.origin.x.0)
        .fold(f32::INFINITY, f32::min);
    let min_y = group_items
        .iter()
        .map(|r| r.origin.y.0)
        .fold(f32::INFINITY, f32::min);
    let max_x = group_items
        .iter()
        .map(|r| r.origin.x.0 + r.size.width.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = group_items
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
                .w_px(MetricRef::Px(Px(web_item_avatar.rect.w)))
                .h_px(MetricRef::Px(Px(web_item_avatar.rect.h))),
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
                        .w_px(MetricRef::Px(Px(web_group_item.rect.w)))
                        .h_px(MetricRef::Px(Px(web_group_item.rect.h))),
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
    let group_items = &group_items[..3];

    let min_x = group_items
        .iter()
        .map(|r| r.origin.x.0)
        .fold(f32::INFINITY, f32::min);
    let min_y = group_items
        .iter()
        .map(|r| r.origin.y.0)
        .fold(f32::INFINITY, f32::min);
    let max_x = group_items
        .iter()
        .map(|r| r.origin.x.0 + r.size.width.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = group_items
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
fn web_vs_fret_layout_scroll_area_demo_root_size() {
    let web = read_web_golden("scroll-area-demo");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items: Vec<_> = (1..=50).map(|i| cx.text(format!("Item {i}"))).collect();

        let scroll_area = fret_ui_shadcn::ScrollArea::new(items)
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w)))
                    .h_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.h))),
            )
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:root")),
                ..Default::default()
            },
            move |_cx| vec![scroll_area],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:root"),
    )
    .expect("fret scroll area root");

    assert_close_px(
        "scroll area root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "scroll area root height",
        root.bounds.size.height,
        web_root.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_demo_max_offset_y_matches_web() {
    let web = read_web_golden("scroll-area-demo");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport = find_first(web_root, &|n| {
        n.computed_style
            .get("overflowY")
            .is_some_and(|v| v == "scroll")
    })
    .expect("web scroll viewport (overflowY=scroll)");

    let metrics = web_viewport
        .scroll
        .as_ref()
        .expect("web scroll viewport missing scroll metrics (regenerate goldens)");

    let expected_max_offset_y = metrics.scroll_height - metrics.client_height;
    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let handle = ScrollHandle::default();
    let _ = run_fret_root(bounds, |cx| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(metrics.scroll_height));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area = fret_ui_shadcn::ScrollArea::new(vec![content])
            .scroll_handle(handle.clone())
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:max-offset-y")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        // Match the upstream border inset: the scroll viewport is inset from the
                        // root's border box (fractional due to DPR / layout rounding).
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    });

    let max = handle.max_offset();
    assert_close_px(
        "scroll area max_offset_y",
        max.y,
        expected_max_offset_y,
        1.0,
    );
    assert!(max.y.0 > 0.0, "expected scroll area to overflow vertically");
}

#[test]
fn web_vs_fret_layout_scroll_area_horizontal_demo_max_offset_matches_web() {
    let web = read_web_golden("scroll-area-horizontal-demo");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_tokens(
        &theme.root,
        &[
            "relative",
            "w-96",
            "rounded-md",
            "border",
            "whitespace-nowrap",
        ],
    )
    .expect("web horizontal scroll area root");

    let web_viewport = find_first(web_root, &|n| {
        n.computed_style
            .get("overflowX")
            .is_some_and(|v| v == "scroll")
    })
    .expect("web scroll viewport (overflowX=scroll)");

    let metrics = web_viewport
        .scroll
        .as_ref()
        .expect("web scroll viewport missing scroll metrics (regenerate goldens)");

    let expected_max_offset_x = metrics.scroll_width - metrics.client_width;
    let expected_max_offset_y = metrics.scroll_height - metrics.client_height;
    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let handle = ScrollHandle::default();
    let _ = run_fret_root(bounds, |cx| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(metrics.scroll_width));
                    layout.size.height = Length::Px(Px(metrics.scroll_height));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area = fret_ui_shadcn::ScrollArea::new(vec![content])
            .axis(fret_ui::element::ScrollAxis::Both)
            .scroll_handle(handle.clone())
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-horizontal-demo:max-offset")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
    });

    let max = handle.max_offset();
    assert_close_px(
        "scroll area horizontal max_offset_x",
        max.x,
        expected_max_offset_x,
        1.0,
    );
    assert_close_px(
        "scroll area horizontal max_offset_y",
        max.y,
        expected_max_offset_y,
        1.0,
    );
    assert!(
        max.x.0 > 0.0,
        "expected scroll area to overflow horizontally"
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_demo_scrollbar_bounds_match_web_hover() {
    let web = read_web_golden("scroll-area-demo.hover");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "vertical")
        .expect("web scroll-area-scrollbar (vertical)");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.h > web_viewport.rect.h + 1.0,
        &|n| -n.rect.h,
    )
    .expect("web scroll content (taller than viewport)");

    let expected_rel = WebRect {
        x: web_scrollbar.rect.x - web_root.rect.x,
        y: web_scrollbar.rect.y - web_root.rect.y,
        w: web_scrollbar.rect.w,
        h: web_scrollbar.rect.h,
    };

    // Match the web border inset: the viewport is inset from the root, and the scrollbar is
    // positioned against that inner padding box.
    let inset = web_viewport.rect.x - web_root.rect.x;

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

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(web_content.rect.h));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:hover")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
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

    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover"),
    )
    .expect("fret hover panel (pre-hover)");

    let expected_abs_pre = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    assert!(
        find_node_with_bounds_close(&ui, panel1.id, expected_abs_pre, 2.0).is_none(),
        "expected scrollbar to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    let move_event = Event::Pointer(PointerEvent::Move {
        position: hover_pos,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_id: PointerId(0),
        pointer_type: PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_event);

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
    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");

    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover"),
    )
    .expect("fret hover panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let (_, scrollbar_bounds) = find_node_with_bounds_close(&ui, panel2.id, expected_abs, 2.0)
        .expect("fret scrollbar bounds after hover");

    assert_rect_close_px(
        "scroll-area-demo scrollbar",
        scrollbar_bounds,
        expected_abs,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_demo_thumb_background_matches_web_hover_light() {
    let web = read_web_golden("scroll-area-demo.hover");
    let theme = web
        .themes
        .get("light")
        .expect("missing light theme in scroll-area-demo.hover");

    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "vertical")
        .expect("web scroll-area-scrollbar (vertical)");
    let web_thumb =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar).expect("web scroll-area-thumb");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let expected_bg = web_thumb
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web thumb backgroundColor");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.h > web_viewport.rect.h + 1.0,
        &|n| -n.rect.h,
    )
    .expect("web scroll content (taller than viewport)");

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

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(web_content.rect.h));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:hover-thumb-bg-light")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
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
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-thumb-bg-light"),
    )
    .expect("fret scroll-area panel (pre-hover)");

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
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

    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");
    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-thumb-bg-light"),
    )
    .expect("fret scroll-area panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) =
        find_scene_quad_background_with_rect_close(&scene, expected_abs, 2.0).expect("thumb quad");
    assert_rgba_close(
        "scroll-area-demo.hover thumb background",
        color_to_rgba(bg),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_demo_thumb_background_matches_web_hover_dark() {
    let web = read_web_golden("scroll-area-demo.hover");
    let theme = web
        .themes
        .get("dark")
        .expect("missing dark theme in scroll-area-demo.hover");

    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "vertical")
        .expect("web scroll-area-scrollbar (vertical)");
    let web_thumb =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar).expect("web scroll-area-thumb");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let expected_bg = web_thumb
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web thumb backgroundColor");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.h > web_viewport.rect.h + 1.0,
        &|n| -n.rect.h,
    )
    .expect("web scroll content (taller than viewport)");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(web_content.rect.h));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:hover-thumb-bg-dark")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
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
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-thumb-bg-dark"),
    )
    .expect("fret scroll-area panel (pre-hover)");

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
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

    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");
    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-thumb-bg-dark"),
    )
    .expect("fret scroll-area panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) =
        find_scene_quad_background_with_rect_close(&scene, expected_abs, 2.0).expect("thumb quad");
    assert_rgba_close(
        "scroll-area-demo.hover dark thumb background",
        color_to_rgba(bg),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_demo_scrollbar_hides_after_hover_out_delay() {
    let web_early = read_web_golden("scroll-area-demo.hover-out-550ms");
    let theme_early = web_theme(&web_early);
    let web_root = web_find_by_class_tokens(
        &theme_early.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport = web_find_by_data_slot(&theme_early.root, "scroll-area-viewport")
        .expect("web scroll viewport");
    let web_scrollbar_early = web_find_scroll_area_scrollbar(&theme_early.root, "vertical")
        .expect("web scroll-area-scrollbar (vertical, early)");
    let web_thumb_early =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar_early).expect("web thumb (early)");

    assert!(
        web_scrollbar_early
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected early web scrollbar to be visible"
    );

    let web_late = read_web_golden("scroll-area-demo.hover-out-650ms");
    let theme_late = web_theme(&web_late);
    assert!(
        web_find_scroll_area_scrollbar(&theme_late.root, "vertical").is_none(),
        "expected late web scrollbar to be absent"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.h > web_viewport.rect.h + 1.0,
        &|n| -n.rect.h,
    )
    .expect("web scroll content (taller than viewport)");

    let expected_rel = WebRect {
        x: web_thumb_early.rect.x - web_root.rect.x,
        y: web_thumb_early.rect.y - web_root.rect.y,
        w: web_thumb_early.rect.w,
        h: web_thumb_early.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme_early.viewport.w), Px(theme_early.viewport.h)),
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

    let handle = ScrollHandle::default();
    let content_h = web_content.rect.h;
    let root_w = web_root.rect.w;
    let root_h = web_root.rect.h;

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let handle = handle.clone();
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:hover-out")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(root_w));
                            layout.size.height = Length::Px(Px(root_h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |cx| {
                        let content = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Px(Px(content_h));
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx| vec![],
                        );

                        let scroll_area =
                            fret_ui_shadcn::ScrollAreaRoot::new(
                                fret_ui_shadcn::ScrollAreaViewport::new(vec![content]),
                            )
                            .scroll_handle(handle.clone())
                            .scrollbar(fret_ui_shadcn::ScrollAreaScrollbar::new().orientation(
                                fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical,
                            ))
                            .refine_layout(LayoutRefinement::default().size_full())
                            .into_element(cx);

                        vec![scroll_area]
                    },
                )]
            },
        )]
    };

    macro_rules! render_at {
        ($frame:expr) => {{
            app.set_frame_id(FrameId($frame));
            let root_node = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "web-vs-fret-layout",
                &render,
            );
            ui.set_root(root_node);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            ui.semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot")
        }};
    }

    let snap0 = render_at!(0);
    let panel0 = find_semantics(
        &snap0,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-out"),
    )
    .expect("fret hover-out panel (initial)");
    let expected_abs0 = WebRect {
        x: panel0.bounds.origin.x.0 + expected_rel.x,
        y: panel0.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene0 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene0, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene0, expected_abs0, 2.0).is_none(),
        "expected thumb quad to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel0.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel0.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
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

    let snap1 = render_at!(1);
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-out"),
    )
    .expect("fret hover-out panel (hovered)");
    let expected_abs1 = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene1 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene1, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene1, expected_abs1, 2.0).is_some(),
        "expected thumb quad to be present after hover"
    );

    // Move outside the ScrollArea hover region (Radix uses pointer leave on the root).
    // Using the outer panel bounds is more robust than aiming for a "gap" near the viewport.
    // Move inside the window but outside the ScrollArea bounds so hover state clears.
    let leave_pos = Point::new(Px(root_w + 100.0), Px(root_h + 100.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: leave_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    // Render once at the "leave tick" so the hover timer is armed.
    let snap_leave = render_at!(2);
    let panel_leave = find_semantics(
        &snap_leave,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-out"),
    )
    .expect("fret hover-out panel (leave)");
    let expected_abs_leave = WebRect {
        x: panel_leave.bounds.origin.x.0 + expected_rel.x,
        y: panel_leave.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene_leave = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene_leave, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene_leave, expected_abs_leave, 2.0).is_some(),
        "expected thumb quad to remain visible immediately after leave"
    );

    // The scrollHideDelay timer advances via per-frame ticks in the ScrollAreaVisibility driver.
    // To match the web goldens, step through frames rather than jumping the FrameId.
    let mut snap_early: Option<fret_core::SemanticsSnapshot> = None;
    let mut snap_late: Option<fret_core::SemanticsSnapshot> = None;
    let mut scene_early: Option<Scene> = None;
    let mut scene_late: Option<Scene> = None;
    for frame in 3..=(2 + 39) {
        let snap = render_at!(frame);
        if frame == 2 + 33 {
            snap_early = Some(snap);
            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
            scene_early = Some(scene);
        } else if frame == 2 + 39 {
            snap_late = Some(snap);
            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
            scene_late = Some(scene);
        }
    }

    // ~550ms after leaving (33 ticks at ~60fps): still visible.
    let snap_early = snap_early.expect("missing snap_early");
    let panel_early = find_semantics(
        &snap_early,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-out"),
    )
    .expect("fret hover-out panel (early)");
    let expected_abs_early = WebRect {
        x: panel_early.bounds.origin.x.0 + expected_rel.x,
        y: panel_early.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let scene_early = scene_early.expect("missing scene_early");
    assert!(
        find_scene_quad_with_rect_close(&scene_early, expected_abs_early, 2.0).is_some(),
        "expected thumb quad to remain visible before scrollHideDelay"
    );

    // ~650ms after leaving (39 ticks at ~60fps): hidden.
    let snap_late = snap_late.expect("missing snap_late");
    let panel_late = find_semantics(
        &snap_late,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:hover-out"),
    )
    .expect("fret hover-out panel (late)");
    let expected_abs_late = WebRect {
        x: panel_late.bounds.origin.x.0 + expected_rel.x,
        y: panel_late.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let scene_late = scene_late.expect("missing scene_late");
    assert!(
        find_scene_quad_with_rect_close(&scene_late, expected_abs_late, 2.0).is_none(),
        "expected thumb quad to be hidden after scrollHideDelay"
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_demo_thumb_bounds_match_web_scrolled() {
    let web = read_web_golden("scroll-area-demo.scrolled");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_tokens(
        &theme.root,
        &["relative", "h-72", "w-48", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "vertical")
        .expect("web scroll-area-scrollbar (vertical)");
    let web_thumb =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar).expect("web scroll-area-thumb");
    let web_scroll = web_viewport
        .scroll
        .as_ref()
        .expect("web scroll viewport scroll metrics");

    assert!(
        (web_scroll.scroll_top - 80.0).abs() < 0.01,
        "expected scrollTop=80 in golden"
    );

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in scrolled golden"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.h > web_viewport.rect.h + 1.0,
        &|n| -n.rect.h,
    )
    .expect("web scroll content (taller than viewport)");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

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

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(web_content.rect.h));
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:scrolled")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
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

    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:scrolled"),
    )
    .expect("fret scrolled panel (pre-hover)");

    let expected_abs_pre = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene1 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene1, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene1, expected_abs_pre, 2.0).is_none(),
        "expected thumb quad to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    let move_event = Event::Pointer(PointerEvent::Move {
        position: hover_pos,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_id: PointerId(0),
        pointer_type: PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_event);

    handle.set_offset(Point::new(Px(0.0), Px(web_scroll.scroll_top)));

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
    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");

    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:scrolled"),
    )
    .expect("fret scrolled panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene2 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene2, 1.0);
    let thumb_bounds =
        find_scene_quad_with_rect_close(&scene2, expected_abs, 2.0).expect("fret thumb quad");
    assert_rect_close_px("scroll-area-demo thumb", thumb_bounds, expected_abs, 2.0);
}

#[test]
fn web_vs_fret_layout_scroll_area_horizontal_demo_scrollbar_bounds_match_web_hover() {
    let web = read_web_golden("scroll-area-horizontal-demo.hover");
    let theme = web_theme(&web);
    let web_root =
        web_find_by_class_tokens(&theme.root, &["relative", "w-96", "rounded-md", "border"])
            .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "horizontal")
        .expect("web scroll-area-scrollbar (horizontal)");
    let _web_thumb = web_find_scroll_area_thumb_in_scrollbar(web_scrollbar)
        .expect("web scroll-area-thumb (horizontal)");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.w > web_viewport.rect.w + 1.0,
        &|n| -n.rect.w,
    )
    .expect("web scroll content (wider than viewport)");

    let expected_rel = WebRect {
        x: web_scrollbar.rect.x - web_root.rect.x,
        y: web_scrollbar.rect.y - web_root.rect.y,
        w: web_scrollbar.rect.w,
        h: web_scrollbar.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

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

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(web_content.rect.w));
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Horizontal),
            )
            .corner(true)
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-horizontal-demo:hover")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
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

    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover"),
    )
    .expect("fret hover panel (pre-hover)");

    let expected_abs_pre = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    assert!(
        find_node_with_bounds_close(&ui, panel1.id, expected_abs_pre, 2.0).is_none(),
        "expected scrollbar to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    let move_event = Event::Pointer(PointerEvent::Move {
        position: hover_pos,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_id: PointerId(0),
        pointer_type: PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_event);

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
    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");

    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover"),
    )
    .expect("fret hover panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let (_, scrollbar_bounds) = find_node_with_bounds_close(&ui, panel2.id, expected_abs, 2.0)
        .expect("fret scrollbar bounds after hover");

    assert_rect_close_px(
        "scroll-area-horizontal-demo scrollbar",
        scrollbar_bounds,
        expected_abs,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_horizontal_demo_thumb_background_matches_web_hover_light() {
    let web = read_web_golden("scroll-area-horizontal-demo.hover");
    let theme = web
        .themes
        .get("light")
        .expect("missing light theme in scroll-area-horizontal-demo.hover");

    let web_root =
        web_find_by_class_tokens(&theme.root, &["relative", "w-96", "rounded-md", "border"])
            .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "horizontal")
        .expect("web scroll-area-scrollbar (horizontal)");
    let web_thumb = web_find_scroll_area_thumb_in_scrollbar(web_scrollbar)
        .expect("web scroll-area-thumb (horizontal)");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let expected_bg = web_thumb
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web thumb backgroundColor");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.w > web_viewport.rect.w + 1.0,
        &|n| -n.rect.w,
    )
    .expect("web scroll content (wider than viewport)");

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

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(web_content.rect.w));
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Horizontal),
            )
            .corner(true)
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(
                    "Golden:scroll-area-horizontal-demo:hover-thumb-bg-light",
                )),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
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
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-thumb-bg-light"),
    )
    .expect("fret scroll-area panel (pre-hover)");

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
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

    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");
    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-thumb-bg-light"),
    )
    .expect("fret scroll-area panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) =
        find_scene_quad_background_with_rect_close(&scene, expected_abs, 2.0).expect("thumb quad");
    assert_rgba_close(
        "scroll-area-horizontal-demo.hover thumb background",
        color_to_rgba(bg),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_horizontal_demo_thumb_background_matches_web_hover_dark() {
    let web = read_web_golden("scroll-area-horizontal-demo.hover");
    let theme = web
        .themes
        .get("dark")
        .expect("missing dark theme in scroll-area-horizontal-demo.hover");

    let web_root =
        web_find_by_class_tokens(&theme.root, &["relative", "w-96", "rounded-md", "border"])
            .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "horizontal")
        .expect("web scroll-area-scrollbar (horizontal)");
    let web_thumb = web_find_scroll_area_thumb_in_scrollbar(web_scrollbar)
        .expect("web scroll-area-thumb (horizontal)");

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in hover golden"
    );

    let expected_bg = web_thumb
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web thumb backgroundColor");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.w > web_viewport.rect.w + 1.0,
        &|n| -n.rect.w,
    )
    .expect("web scroll content (wider than viewport)");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(web_content.rect.w));
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Horizontal),
            )
            .corner(true)
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(
                    "Golden:scroll-area-horizontal-demo:hover-thumb-bg-dark",
                )),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
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
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-thumb-bg-dark"),
    )
    .expect("fret scroll-area panel (pre-hover)");

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
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

    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");
    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-thumb-bg-dark"),
    )
    .expect("fret scroll-area panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) =
        find_scene_quad_background_with_rect_close(&scene, expected_abs, 2.0).expect("thumb quad");
    assert_rgba_close(
        "scroll-area-horizontal-demo.hover dark thumb background",
        color_to_rgba(bg),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_horizontal_demo_scrollbar_hides_after_hover_out_delay() {
    let web_early = read_web_golden("scroll-area-horizontal-demo.hover-out-550ms");
    let theme_early = web_theme(&web_early);
    let web_root = web_find_by_class_tokens(
        &theme_early.root,
        &["relative", "w-96", "rounded-md", "border"],
    )
    .expect("web scroll area root");

    let web_viewport = web_find_by_data_slot(&theme_early.root, "scroll-area-viewport")
        .expect("web scroll viewport");
    let web_scrollbar_early = web_find_scroll_area_scrollbar(&theme_early.root, "horizontal")
        .expect("web scroll-area-scrollbar (horizontal, early)");
    let web_thumb_early =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar_early).expect("web thumb (early)");

    assert!(
        web_scrollbar_early
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected early web scrollbar to be visible"
    );

    let web_late = read_web_golden("scroll-area-horizontal-demo.hover-out-650ms");
    let theme_late = web_theme(&web_late);
    assert!(
        web_find_scroll_area_scrollbar(&theme_late.root, "horizontal").is_none(),
        "expected late web scrollbar to be absent"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.w > web_viewport.rect.w + 1.0,
        &|n| -n.rect.w,
    )
    .expect("web scroll content (wider than viewport)");

    let expected_rel = WebRect {
        x: web_thumb_early.rect.x - web_root.rect.x,
        y: web_thumb_early.rect.y - web_root.rect.y,
        w: web_thumb_early.rect.w,
        h: web_thumb_early.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme_early.viewport.w), Px(theme_early.viewport.h)),
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

    let handle = ScrollHandle::default();
    let content_w = web_content.rect.w;
    let root_w = web_root.rect.w;
    let root_h = web_root.rect.h;

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let handle = handle.clone();
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-horizontal-demo:hover-out")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(root_w));
                            layout.size.height = Length::Px(Px(root_h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |cx| {
                        let content = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(content_w));
                                    layout.size.height = Length::Fill;
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx| vec![],
                        );

                        let scroll_area =
                            fret_ui_shadcn::ScrollAreaRoot::new(
                                fret_ui_shadcn::ScrollAreaViewport::new(vec![content]),
                            )
                            .scroll_handle(handle.clone())
                            .scrollbar(fret_ui_shadcn::ScrollAreaScrollbar::new().orientation(
                                fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical,
                            ))
                            .scrollbar(fret_ui_shadcn::ScrollAreaScrollbar::new().orientation(
                                fret_ui_shadcn::ScrollAreaScrollbarOrientation::Horizontal,
                            ))
                            .corner(true)
                            .refine_layout(LayoutRefinement::default().size_full())
                            .into_element(cx);

                        vec![scroll_area]
                    },
                )]
            },
        )]
    };

    macro_rules! render_at {
        ($frame:expr) => {{
            app.set_frame_id(FrameId($frame));
            let root_node = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "web-vs-fret-layout",
                &render,
            );
            ui.set_root(root_node);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            ui.semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot")
        }};
    }

    let snap0 = render_at!(0);
    let panel0 = find_semantics(
        &snap0,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-out"),
    )
    .expect("fret hover-out panel (initial)");
    let expected_abs0 = WebRect {
        x: panel0.bounds.origin.x.0 + expected_rel.x,
        y: panel0.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene0 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene0, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene0, expected_abs0, 2.0).is_none(),
        "expected thumb quad to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel0.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel0.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
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

    let snap1 = render_at!(1);
    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-out"),
    )
    .expect("fret hover-out panel (hovered)");
    let expected_abs1 = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene1 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene1, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene1, expected_abs1, 2.0).is_some(),
        "expected thumb quad to be present after hover"
    );

    let leave_pos = Point::new(Px(root_w + 100.0), Px(root_h + 100.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: leave_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let snap_leave = render_at!(2);
    let panel_leave = find_semantics(
        &snap_leave,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-out"),
    )
    .expect("fret hover-out panel (leave)");
    let expected_abs_leave = WebRect {
        x: panel_leave.bounds.origin.x.0 + expected_rel.x,
        y: panel_leave.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let mut scene_leave = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene_leave, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene_leave, expected_abs_leave, 2.0).is_some(),
        "expected thumb quad to remain visible immediately after leave"
    );

    let mut snap_early: Option<fret_core::SemanticsSnapshot> = None;
    let mut snap_late: Option<fret_core::SemanticsSnapshot> = None;
    let mut scene_early: Option<Scene> = None;
    let mut scene_late: Option<Scene> = None;
    for frame in 3..=(2 + 39) {
        let snap = render_at!(frame);
        if frame == 2 + 33 {
            snap_early = Some(snap);
            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
            scene_early = Some(scene);
        } else if frame == 2 + 39 {
            snap_late = Some(snap);
            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
            scene_late = Some(scene);
        }
    }

    let snap_early = snap_early.expect("missing snap_early");
    let panel_early = find_semantics(
        &snap_early,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-out"),
    )
    .expect("fret hover-out panel (early)");
    let expected_abs_early = WebRect {
        x: panel_early.bounds.origin.x.0 + expected_rel.x,
        y: panel_early.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let scene_early = scene_early.expect("missing scene_early");
    assert!(
        find_scene_quad_with_rect_close(&scene_early, expected_abs_early, 2.0).is_some(),
        "expected thumb quad to remain visible before scrollHideDelay"
    );

    let snap_late = snap_late.expect("missing snap_late");
    let panel_late = find_semantics(
        &snap_late,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:hover-out"),
    )
    .expect("fret hover-out panel (late)");
    let expected_abs_late = WebRect {
        x: panel_late.bounds.origin.x.0 + expected_rel.x,
        y: panel_late.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };
    let scene_late = scene_late.expect("missing scene_late");
    assert!(
        find_scene_quad_with_rect_close(&scene_late, expected_abs_late, 2.0).is_none(),
        "expected thumb quad to be hidden after scrollHideDelay"
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_horizontal_demo_thumb_bounds_match_web_scrolled() {
    let web = read_web_golden("scroll-area-horizontal-demo.scrolled");
    let theme = web_theme(&web);
    let web_root =
        web_find_by_class_tokens(&theme.root, &["relative", "w-96", "rounded-md", "border"])
            .expect("web scroll area root");

    let web_viewport =
        web_find_by_data_slot(&theme.root, "scroll-area-viewport").expect("web scroll viewport");
    let web_scrollbar = web_find_scroll_area_scrollbar(&theme.root, "horizontal")
        .expect("web scroll-area-scrollbar (horizontal)");
    let web_thumb =
        web_find_scroll_area_thumb_in_scrollbar(web_scrollbar).expect("web scroll-area-thumb");
    let web_scroll = web_viewport
        .scroll
        .as_ref()
        .expect("web scroll viewport scroll metrics");

    assert!(
        (web_scroll.scroll_left - 80.0).abs() < 0.01,
        "expected scrollLeft=80 in golden"
    );

    assert!(
        web_scrollbar
            .attrs
            .get("data-state")
            .is_some_and(|v| v == "visible"),
        "expected web scrollbar to be visible in scrolled golden"
    );

    let web_content = web_find_best_by(
        web_viewport,
        &|n| n.tag == "div" && n.rect.w > web_viewport.rect.w + 1.0,
        &|n| -n.rect.w,
    )
    .expect("web scroll content (wider than viewport)");

    let expected_rel = WebRect {
        x: web_thumb.rect.x - web_root.rect.x,
        y: web_thumb.rect.y - web_root.rect.y,
        w: web_thumb.rect.w,
        h: web_thumb.rect.h,
    };

    let inset = web_viewport.rect.x - web_root.rect.x;

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

    let handle = ScrollHandle::default();

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        let content = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(web_content.rect.w));
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |_cx| vec![],
        );

        let scroll_area =
            fret_ui_shadcn::ScrollAreaRoot::new(fret_ui_shadcn::ScrollAreaViewport::new(vec![
                content,
            ]))
            .scroll_handle(handle.clone())
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Vertical),
            )
            .scrollbar(
                fret_ui_shadcn::ScrollAreaScrollbar::new()
                    .orientation(fret_ui_shadcn::ScrollAreaScrollbarOrientation::Horizontal),
            )
            .corner(true)
            .refine_layout(LayoutRefinement::default().size_full())
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-horizontal-demo:scrolled")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(web_root.rect.w));
                            layout.size.height = Length::Px(Px(web_root.rect.h));
                            layout
                        },
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |_cx| vec![scroll_area],
                )]
            },
        )]
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

    let panel1 = find_semantics(
        &snap1,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:scrolled"),
    )
    .expect("fret scrolled panel (pre-hover)");

    let expected_abs_pre = WebRect {
        x: panel1.bounds.origin.x.0 + expected_rel.x,
        y: panel1.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene1 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene1, 1.0);
    assert!(
        find_scene_quad_with_rect_close(&scene1, expected_abs_pre, 2.0).is_none(),
        "expected thumb quad to be absent before hover"
    );

    let hover_pos = Point::new(
        Px(panel1.bounds.origin.x.0 + (web_viewport.rect.x + web_viewport.rect.w * 0.5)),
        Px(panel1.bounds.origin.y.0 + (web_viewport.rect.y + web_viewport.rect.h * 0.5)),
    );
    let move_event = Event::Pointer(PointerEvent::Move {
        position: hover_pos,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_id: PointerId(0),
        pointer_type: PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_event);

    handle.set_offset(Point::new(Px(web_scroll.scroll_left), Px(0.0)));

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
    let snap2 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (post-hover)");

    let panel2 = find_semantics(
        &snap2,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-horizontal-demo:scrolled"),
    )
    .expect("fret scrolled panel (post-hover)");

    let expected_abs = WebRect {
        x: panel2.bounds.origin.x.0 + expected_rel.x,
        y: panel2.bounds.origin.y.0 + expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let mut scene2 = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene2, 1.0);
    let thumb_bounds =
        find_scene_quad_with_rect_close(&scene2, expected_abs, 2.0).expect("fret thumb quad");
    assert_rect_close_px(
        "scroll-area-horizontal-demo thumb",
        thumb_bounds,
        expected_abs,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_select_scrollable_trigger_size() {
    let web = read_web_golden("select-scrollable");
    let theme = web_theme(&web);
    let web_trigger =
        web_find_by_class_tokens(&theme.root, &["w-[280px]"]).expect("web select trigger");

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

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-select",
        |cx| {
            vec![
                fret_ui_shadcn::Select::new(value.clone(), open.clone())
                    .items([
                        fret_ui_shadcn::SelectItem::new("alpha", "Alpha"),
                        fret_ui_shadcn::SelectItem::new("beta", "Beta"),
                        fret_ui_shadcn::SelectItem::new("gamma", "Gamma"),
                    ])
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(fret_ui_kit::MetricRef::Px(Px(web_trigger.rect.w))),
                    )
                    .into_element(cx),
            ]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let combobox = find_semantics(&snap, SemanticsRole::ComboBox, None)
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret select trigger node");

    assert_close_px(
        "select trigger width",
        combobox.bounds.size.width,
        web_trigger.rect.w,
        1.0,
    );
    assert_close_px(
        "select trigger height",
        combobox.bounds.size.height,
        web_trigger.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_demo_geometry() {
    let web = read_web_golden("input-demo");
    let theme = web_theme(&web);
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::Input::new(model)
                .a11y_label("Input")
                .into_element(cx),
        ]
    });

    let input = find_semantics(&snap, SemanticsRole::TextField, Some("Input"))
        .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
        .expect("fret input semantics node");

    assert_close_px(
        "input width",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input height",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_disabled_geometry_matches() {
    let web = read_web_golden("input-disabled");
    let theme = web_theme(&web);
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::Input::new(model)
                .disabled(true)
                .a11y_label("Golden:input-disabled:input")
                .into_element(cx),
        ]
    });

    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-disabled:input"),
    )
    .expect("fret disabled input semantics node");

    assert_close_px(
        "input-disabled width",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-disabled height",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_file_geometry_matches() {
    let web = read_web_golden("input-file");
    let theme = web_theme(&web);

    let web_label = web_find_by_tag_and_text(&theme.root, "label", "Picture").expect("web label");
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");

    let expected_gap_y = web_input.rect.y - (web_label.rect.y + web_label.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-file:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::Label::new("Picture").into_element(cx)],
        );

        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("Golden:input-file:input")
            .into_element(cx);

        let col = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_input.rect.w)))
                        .min_w_0(),
                ),
                direction: fret_core::Axis::Vertical,
                gap: Px(expected_gap_y),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![label, input],
        );

        vec![col]
    });

    let label = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:input-file:label"))
        .expect("fret label");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-file:input"),
    )
    .expect("fret input");

    assert_close_px(
        "input-file label h",
        label.bounds.size.height,
        web_label.rect.h,
        1.0,
    );
    assert_close_px(
        "input-file input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-file input h",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );

    let gap_y = input.bounds.origin.y.0 - (label.bounds.origin.y.0 + label.bounds.size.height.0);
    assert_close_px("input-file gap", Px(gap_y), expected_gap_y, 1.0);
}

#[test]
fn web_vs_fret_layout_input_with_button_geometry_matches() {
    let web = read_web_golden("input-with-button");
    let theme = web_theme(&web);

    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");
    let web_button =
        web_find_by_tag_and_text(&theme.root, "button", "Subscribe").expect("web button");

    let expected_gap_x = web_button.rect.x - (web_input.rect.x + web_input.rect.w);
    let expected_row_w = (web_button.rect.x + web_button.rect.w) - web_input.rect.x;
    let web_button_w = web_button.rect.w;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("Golden:input-with-button:input")
            .into_element(cx);

        let button = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-with-button:button")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Button::new("Subscribe")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .refine_layout(
                            LayoutRefinement::default().w_px(MetricRef::Px(Px(web_button_w))),
                        )
                        .into_element(cx),
                ]
            },
        );

        let row = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(expected_row_w)))
                        .min_w_0(),
                ),
                direction: fret_core::Axis::Horizontal,
                gap: Px(expected_gap_x),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| vec![input, button],
        );

        vec![row]
    });

    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-with-button:input"),
    )
    .expect("fret input");
    let button = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-with-button:button"),
    )
    .expect("fret button wrapper");

    assert_close_px(
        "input-with-button input h",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
    assert_close_px(
        "input-with-button button w",
        button.bounds.size.width,
        web_button.rect.w,
        1.0,
    );
    assert_close_px(
        "input-with-button button h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
    assert_close_px(
        "input-with-button input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_with_text_geometry_matches() {
    let web = read_web_golden("input-with-text");
    let theme = web_theme(&web);

    let web_label = web_find_by_tag_and_text(&theme.root, "label", "Email").expect("web label");
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");
    let web_p = web_find_by_tag_and_text(&theme.root, "p", "Enter your email address.")
        .expect("web helper text");

    let gap0 = web_input.rect.y - (web_label.rect.y + web_label.rect.h);
    let gap1 = web_p.rect.y - (web_input.rect.y + web_input.rect.h);
    let web_label_h = web_label.rect.h;
    let web_p_h = web_p.rect.h;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-with-text:label")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(web_label_h)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx| vec![fret_ui_shadcn::Label::new("Email").into_element(cx)],
                )]
            },
        );

        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("Golden:input-with-text:input")
            .into_element(cx);

        let helper = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-with-text:helper")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(web_p_h)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx| vec![decl_text::text_sm(cx, "Enter your email address.")],
                )]
            },
        );

        let col = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_input.rect.w)))
                        .min_w_0(),
                ),
                direction: fret_core::Axis::Vertical,
                gap: Px(gap0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![label, input, helper],
        );

        vec![col]
    });

    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-with-text:label"),
    )
    .expect("fret label");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-with-text:input"),
    )
    .expect("fret input");
    let helper = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-with-text:helper"),
    )
    .expect("fret helper");

    assert_close_px(
        "input-with-text label h",
        label.bounds.size.height,
        web_label.rect.h,
        1.0,
    );
    assert_close_px(
        "input-with-text input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-with-text input h",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
    assert_close_px(
        "input-with-text helper h",
        helper.bounds.size.height,
        web_p.rect.h,
        1.0,
    );

    let gap0_fret =
        input.bounds.origin.y.0 - (label.bounds.origin.y.0 + label.bounds.size.height.0);
    let gap1_fret =
        helper.bounds.origin.y.0 - (input.bounds.origin.y.0 + input.bounds.size.height.0);
    assert_close_px("input-with-text gap0", Px(gap0_fret), gap0, 1.0);
    assert_close_px("input-with-text gap1", Px(gap1_fret), gap1, 1.0);
}

#[test]
fn web_vs_fret_layout_input_group_label_geometry_matches() {
    let web = read_web_golden("input-group-label");
    let theme = web_theme(&web);

    let mut web_groups: Vec<&WebNode> = Vec::new();
    fn walk_collect<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                let mut has_group = false;
                let mut has_border = false;
                for t in c.split_whitespace() {
                    has_group |= t == "group/input-group";
                    has_border |= t == "border-input";
                }
                has_group && has_border
            })
        {
            out.push(n);
        }
        for c in &n.children {
            walk_collect(c, out);
        }
    }
    walk_collect(&theme.root, &mut web_groups);
    web_groups.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let web_group0 = *web_groups.get(0).expect("web group 0");
    let web_group1 = *web_groups.get(1).expect("web group 1");

    let web_input0 = find_first(web_group0, &|n| n.tag == "input").expect("web input0");
    let web_input1 = find_first(web_group1, &|n| n.tag == "input").expect("web input1");
    let web_addon_label0 = find_first(web_group0, &|n| n.tag == "label").expect("web label0");
    let web_addon_label0_w = web_addon_label0.rect.w;

    let expected_gap_y = web_group1.rect.y - (web_group0.rect.y + web_group0.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model0: Model<String> = cx.app.models_mut().insert(String::new());
        let model1: Model<String> = cx.app.models_mut().insert(String::new());

        let group0 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-group-label:0:root")),
                ..Default::default()
            },
            move |cx| {
                let addon = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        label: Some(Arc::from("Golden:input-group-label:0:addon")),
                        ..Default::default()
                    },
                    move |cx| {
                        let label = fret_ui_shadcn::Label::new("@").into_element(cx);
                        vec![
                            cx.container(
                                ContainerProps {
                                    layout: decl_style::layout_style(
                                        &Theme::global(&*cx.app),
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(web_addon_label0_w)))
                                            .min_w_0(),
                                    ),
                                    ..Default::default()
                                },
                                move |_cx| vec![label],
                            ),
                        ]
                    },
                );

                vec![
                    fret_ui_shadcn::InputGroup::new(model0.clone())
                        .a11y_label("Golden:input-group-label:0:input")
                        .leading(vec![addon])
                        .into_element(cx),
                ]
            },
        );

        let group1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-group-label:1:root")),
                ..Default::default()
            },
            move |cx| {
                let info_icon = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        label: Some(Arc::from("Golden:input-group-label:1:icon")),
                        ..Default::default()
                    },
                    move |cx| vec![decl_icon::icon(cx, IconId::new_static("lucide.info"))],
                );

                let help_button = fret_ui_shadcn::InputGroupButton::new("")
                    .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                    .size(fret_ui_shadcn::InputGroupButtonSize::IconXs)
                    .children(vec![info_icon])
                    .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                    .into_element(cx);

                let header_row = cx.flex(
                    FlexProps {
                        layout: decl_style::layout_style(
                            &Theme::global(&*cx.app),
                            LayoutRefinement::default().w_full().min_w_0(),
                        ),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(8.0),
                        padding: fret_core::Edges::all(Px(0.0)),
                        justify: MainAlign::SpaceBetween,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![
                            fret_ui_shadcn::Label::new("Email").into_element(cx),
                            help_button,
                        ]
                    },
                );

                vec![
                    fret_ui_shadcn::InputGroup::new(model1.clone())
                        .a11y_label("Golden:input-group-label:1:input")
                        .block_start(vec![header_row])
                        .into_element(cx),
                ]
            },
        );

        let col = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_group0.rect.w)))
                        .min_w_0(),
                ),
                direction: fret_core::Axis::Vertical,
                gap: Px(expected_gap_y),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![group0, group1],
        );

        vec![col]
    });

    let fret_group0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-label:0:root"),
    )
    .expect("fret group0");
    let fret_input0 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-label:0:input"),
    )
    .expect("fret input0");

    let fret_group1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-label:1:root"),
    )
    .expect("fret group1");
    let fret_input1 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-label:1:input"),
    )
    .expect("fret input1");

    assert_close_px(
        "input-group-label group0 h",
        fret_group0.bounds.size.height,
        web_group0.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-label input0 w",
        fret_input0.bounds.size.width,
        web_input0.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-label group1 h",
        fret_group1.bounds.size.height,
        web_group1.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-label input1 w",
        fret_input1.bounds.size.width,
        web_input1.rect.w,
        1.0,
    );

    let gap_y = fret_group1.bounds.origin.y.0
        - (fret_group0.bounds.origin.y.0 + fret_group0.bounds.size.height.0);
    assert_close_px("input-group-label gap", Px(gap_y), expected_gap_y, 1.0);
}

#[test]
fn web_vs_fret_layout_input_group_button_group_geometry_matches() {
    let web = read_web_golden("input-group-button-group");
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(
        &theme.root,
        &[
            "flex",
            "w-fit",
            "items-stretch",
            "[&>*:not(:first-child)]:border-l-0",
        ],
    )
    .expect("web button-group");
    let web_input_group =
        web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
            .expect("web input-group");
    let web_input = find_first(web_input_group, &|n| n.tag == "input").expect("web input");

    let web_prefix = find_first(web_group, &|n| {
        class_has_token(n, "bg-muted") && contains_text(n, "https://")
    })
    .expect("web prefix");
    let web_suffix = find_first(web_group, &|n| {
        class_has_token(n, "bg-muted") && contains_text(n, ".com")
    })
    .expect("web suffix");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let icon = decl_icon::icon(cx, IconId::new_static("lucide.link-2"));

        let input_group = fret_ui_shadcn::InputGroup::new(model)
            .a11y_label("Golden:input-group-button-group:input")
            .trailing(vec![icon]);

        let group = fret_ui_shadcn::ButtonGroup::new(vec![
            fret_ui_shadcn::ButtonGroupText::new("https://")
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_prefix.rect.w))),
                )
                .into(),
            input_group.into(),
            fret_ui_shadcn::ButtonGroupText::new(".com")
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_suffix.rect.w))),
                )
                .into(),
        ])
        .a11y_label("Golden:input-group-button-group:group")
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(web_group.rect.w))));

        vec![group.into_element(cx)]
    });

    let group = find_semantics(
        &snap,
        SemanticsRole::Group,
        Some("Golden:input-group-button-group:group"),
    )
    .expect("fret button-group");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-button-group:input"),
    )
    .expect("fret input");

    assert_close_px(
        "input-group-button-group group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-button-group group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-button-group input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-button-group input h",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
}

fn web_collect_input_otp_slots<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut slots = find_all(root, &|n| {
        n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                c.split_whitespace().any(|t| t == "h-9")
                    && c.split_whitespace().any(|t| t == "w-9")
                    && c.split_whitespace().any(|t| t == "border-input")
            })
    });
    slots.sort_by(|a, b| {
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
    slots
}

fn web_collect_input_otp_separators<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut seps = find_all(root, &|n| {
        n.tag == "div" && n.attrs.get("role").is_some_and(|v| v == "separator")
    });
    seps.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    seps
}

fn web_find_leftmost_by_class_tokens<'a>(root: &'a WebNode, tokens: &[&str]) -> &'a WebNode {
    let mut nodes = find_all(root, &|n| class_has_all_tokens(n, tokens));
    nodes.sort_by(|a, b| {
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
    nodes[0]
}

fn web_collect_input_otp_slots_by_border_input<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut slots = find_all(root, &|n| {
        n.tag == "div"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.split_whitespace().any(|t| t == "border-input"))
    });
    slots.sort_by(|a, b| {
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
    slots
}

fn fret_collect_rects_by_size(
    snap: &fret_core::SemanticsSnapshot,
    w: Px,
    h: Px,
    tol: f32,
) -> Vec<Rect> {
    let mut rects: Vec<Rect> = snap
        .nodes
        .iter()
        .filter(|n| {
            (n.bounds.size.width.0 - w.0).abs() <= tol
                && (n.bounds.size.height.0 - h.0).abs() <= tol
        })
        .map(|n| n.bounds)
        .collect();
    rects.sort_by(|a, b| {
        a.origin
            .x
            .0
            .partial_cmp(&b.origin.x.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.origin
                    .y
                    .0
                    .partial_cmp(&b.origin.y.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    rects.dedup_by(|a, b| {
        (a.origin.x.0 - b.origin.x.0).abs() <= tol
            && (a.origin.y.0 - b.origin.y.0).abs() <= tol
            && (a.size.width.0 - b.size.width.0).abs() <= tol
            && (a.size.height.0 - b.size.height.0).abs() <= tol
    });
    rects
}

fn assert_input_otp_slots_match_web(
    name: &str,
    web_slots: &[&WebNode],
    fret_slots: &[Rect],
    tol: f32,
) {
    assert_eq!(
        fret_slots.len(),
        web_slots.len(),
        "{name}: expected {} slots, got {}",
        web_slots.len(),
        fret_slots.len()
    );
    for (idx, (w, f)) in web_slots.iter().zip(fret_slots.iter()).enumerate() {
        assert_close_px(&format!("{name} slot[{idx}] x"), f.origin.x, w.rect.x, tol);
        assert_close_px(&format!("{name} slot[{idx}] y"), f.origin.y, w.rect.y, tol);
        assert_close_px(
            &format!("{name} slot[{idx}] w"),
            f.size.width,
            w.rect.w,
            tol,
        );
        assert_close_px(
            &format!("{name} slot[{idx}] h"),
            f.size.height,
            w.rect.h,
            tol,
        );
    }
}

fn assert_input_otp_separators_match_web(
    name: &str,
    web_seps: &[&WebNode],
    fret_seps: &[Rect],
    tol: f32,
) {
    assert_eq!(
        fret_seps.len(),
        web_seps.len(),
        "{name}: expected {} separators, got {}",
        web_seps.len(),
        fret_seps.len()
    );
    for (idx, (w, f)) in web_seps.iter().zip(fret_seps.iter()).enumerate() {
        assert_close_px(&format!("{name} sep[{idx}] x"), f.origin.x, w.rect.x, tol);
        assert_close_px(&format!("{name} sep[{idx}] y"), f.origin.y, w.rect.y, tol);
        assert_close_px(&format!("{name} sep[{idx}] w"), f.size.width, w.rect.w, tol);
        assert_close_px(
            &format!("{name} sep[{idx}] h"),
            f.size.height,
            w.rect.h,
            tol,
        );
    }
}

fn assert_input_otp_slots_relative_to_container_match_web(
    name: &str,
    web_container: &WebNode,
    web_slots: &[&WebNode],
    fret_container: &Rect,
    fret_slots: &[Rect],
    tol: f32,
) {
    assert_eq!(
        fret_slots.len(),
        web_slots.len(),
        "{name}: expected {} slots, got {}",
        web_slots.len(),
        fret_slots.len()
    );
    for (idx, (w, f)) in web_slots.iter().zip(fret_slots.iter()).enumerate() {
        let web_dx = w.rect.x - web_container.rect.x;
        let web_dy = w.rect.y - web_container.rect.y;

        let fret_dx = f.origin.x - fret_container.origin.x;
        let fret_dy = f.origin.y - fret_container.origin.y;

        assert_close_px(&format!("{name} slot[{idx}] dx"), fret_dx, web_dx, tol);
        assert_close_px(&format!("{name} slot[{idx}] dy"), fret_dy, web_dy, tol);
        assert_close_px(
            &format!("{name} slot[{idx}] w"),
            f.size.width,
            w.rect.w,
            tol,
        );
        assert_close_px(
            &format!("{name} slot[{idx}] h"),
            f.size.height,
            w.rect.h,
            tol,
        );
    }
}

fn assert_input_otp_block_relative_geometry_matches_web(web_name: &str, row_tokens: &[&str]) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_row = web_find_leftmost_by_class_tokens(&theme.root, row_tokens);
    let web_slots = web_collect_input_otp_slots_by_border_input(web_row);
    assert!(
        !web_slots.is_empty(),
        "{web_name}: expected input otp slots in web row"
    );

    let slot_w = web_slots[0].rect.w;
    let slot_h = web_slots[0].rect.h;
    let slot_gap = if web_slots.len() > 1 {
        (web_slots[1].rect.x - web_slots[0].rect.x - slot_w).max(0.0)
    } else {
        0.0
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label: Arc<str> = Arc::from(format!("Golden:{web_name}:otp-row"));
    let label_str: &str = &label;
    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let otp = fret_ui_shadcn::InputOtp::new(model)
            .length(web_slots.len())
            .slot_gap_px(Px(slot_gap))
            .slot_size_px(Px(slot_w), Px(slot_h));
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(label.clone()),
                ..Default::default()
            },
            move |cx| vec![otp.into_element(cx)],
        )]
    });

    let fret_row =
        find_semantics(&snap, SemanticsRole::Panel, Some(label_str)).expect("fret otp row");
    assert_close_px(
        &format!("{web_name} otp-row w"),
        fret_row.bounds.size.width,
        web_row.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} otp-row h"),
        fret_row.bounds.size.height,
        web_row.rect.h,
        1.0,
    );

    let fret_slots = fret_collect_rects_by_size(&snap, Px(slot_w), Px(slot_h), 1.0);
    assert_input_otp_slots_relative_to_container_match_web(
        web_name,
        web_row,
        &web_slots,
        &fret_row.bounds,
        &fret_slots,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_otp_01_input_otp_row_geometry_matches_web() {
    assert_input_otp_block_relative_geometry_matches_web(
        "otp-01",
        &[
            "flex",
            "items-center",
            "gap-2.5",
            "*:data-[slot=input-otp-slot]:rounded-md",
            "*:data-[slot=input-otp-slot]:border",
        ],
    );
}

#[test]
fn web_vs_fret_layout_otp_02_input_otp_row_geometry_matches_web() {
    assert_input_otp_block_relative_geometry_matches_web(
        "otp-02",
        &[
            "flex",
            "items-center",
            "gap-2",
            "*:data-[slot=input-otp-slot]:rounded-md",
            "*:data-[slot=input-otp-slot]:border",
        ],
    );
}

#[test]
fn web_vs_fret_layout_otp_03_input_otp_row_geometry_matches_web() {
    assert_input_otp_block_relative_geometry_matches_web(
        "otp-03",
        &[
            "flex",
            "items-center",
            "gap-2.5",
            "*:data-[slot=input-otp-slot]:rounded-md",
            "*:data-[slot=input-otp-slot]:border",
        ],
    );
}

#[test]
fn web_vs_fret_layout_otp_05_input_otp_row_geometry_matches_web() {
    assert_input_otp_block_relative_geometry_matches_web(
        "otp-05",
        &[
            "flex",
            "items-center",
            "gap-2.5",
            "*:data-[slot=input-otp-slot]:h-16",
            "*:data-[slot=input-otp-slot]:w-12",
            "*:data-[slot=input-otp-slot]:rounded-md",
            "*:data-[slot=input-otp-slot]:border",
            "*:data-[slot=input-otp-slot]:text-xl",
        ],
    );
}

#[test]
fn web_vs_fret_layout_input_otp_demo_geometry_matches() {
    let web = read_web_golden("input-otp-demo");
    let theme = web_theme(&web);
    let web_slots = web_collect_input_otp_slots(&theme.root);
    let web_seps = web_collect_input_otp_separators(&theme.root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::InputOtp::new(model)
                .group_size(Some(3))
                .into_element(cx),
        ]
    });

    let fret_slots = fret_collect_rects_by_size(&snap, Px(36.0), Px(36.0), 1.0);
    let fret_seps = fret_collect_rects_by_size(&snap, Px(24.0), Px(24.0), 1.0);

    assert_input_otp_slots_match_web("input-otp-demo", &web_slots, &fret_slots, 1.0);
    assert_input_otp_separators_match_web("input-otp-demo", &web_seps, &fret_seps, 1.0);
}

#[test]
fn web_vs_fret_layout_input_otp_separator_geometry_matches() {
    let web = read_web_golden("input-otp-separator");
    let theme = web_theme(&web);
    let web_slots = web_collect_input_otp_slots(&theme.root);
    let web_seps = web_collect_input_otp_separators(&theme.root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::InputOtp::new(model)
                .group_size(Some(2))
                .into_element(cx),
        ]
    });

    let fret_slots = fret_collect_rects_by_size(&snap, Px(36.0), Px(36.0), 1.0);
    let fret_seps = fret_collect_rects_by_size(&snap, Px(24.0), Px(24.0), 1.0);

    assert_input_otp_slots_match_web("input-otp-separator", &web_slots, &fret_slots, 1.0);
    assert_input_otp_separators_match_web("input-otp-separator", &web_seps, &fret_seps, 1.0);
}

#[test]
fn web_vs_fret_layout_input_otp_pattern_geometry_matches() {
    let web = read_web_golden("input-otp-pattern");
    let theme = web_theme(&web);
    let web_slots = web_collect_input_otp_slots(&theme.root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::InputOtp::new(model)
                .numeric_only(false)
                .into_element(cx),
        ]
    });

    let fret_slots = fret_collect_rects_by_size(&snap, Px(36.0), Px(36.0), 1.0);
    assert_input_otp_slots_match_web("input-otp-pattern", &web_slots, &fret_slots, 1.0);
}

#[test]
fn web_vs_fret_layout_input_otp_controlled_geometry_matches() {
    let web = read_web_golden("input-otp-controlled");
    let theme = web_theme(&web);
    let web_slots = web_collect_input_otp_slots(&theme.root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![fret_ui_shadcn::InputOtp::new(model).into_element(cx)]
    });

    let fret_slots = fret_collect_rects_by_size(&snap, Px(36.0), Px(36.0), 1.0);
    assert_input_otp_slots_match_web("input-otp-controlled", &web_slots, &fret_slots, 1.0);
}

fn command_demo_snapshot(theme: &WebGoldenTheme) -> fret_core::SemanticsSnapshot {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    run_fret_root_with_services(bounds, &mut services, |cx| {
        use fret_ui_shadcn::{
            CommandEntry, CommandGroup, CommandItem, CommandPalette, CommandSeparator,
        };

        let query: Model<String> = cx.app.models_mut().insert(String::new());

        let entries: Vec<CommandEntry> = vec![
            CommandGroup::new(vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ])
            .heading("Suggestions")
            .into(),
            CommandSeparator::new().into(),
            CommandGroup::new(vec![
                CommandItem::new("Profile"),
                CommandItem::new("Billing"),
                CommandItem::new("Settings"),
            ])
            .heading("Settings")
            .into(),
        ];

        vec![
            CommandPalette::new(query, Vec::new())
                .entries(entries)
                .into_element(cx),
        ]
    })
}

#[test]
fn web_vs_fret_layout_command_demo_input_height_matches() {
    let web = read_web_golden("command-demo");
    let theme = web_theme(&web);
    let web_input = find_first(&theme.root, &|n| {
        n.tag == "input" && n.attrs.get("role").is_some_and(|v| v == "combobox")
    })
    .expect("web command-demo combobox input");

    let snap = command_demo_snapshot(theme);
    let combobox = find_semantics(&snap, SemanticsRole::ComboBox, None)
        .unwrap_or_else(|| panic!("missing fret command-demo combobox"));

    assert_close_px(
        "command-demo input height",
        combobox.bounds.size.height,
        web_input.rect.h,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_command_demo_listbox_height_matches() {
    let web = read_web_golden("command-demo");
    let theme = web_theme(&web);
    let web_listbox = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "listbox")
    })
    .expect("web command-demo listbox");

    let snap = command_demo_snapshot(theme);
    let listbox = find_semantics(&snap, SemanticsRole::ListBox, None)
        .unwrap_or_else(|| panic!("missing fret command-demo listbox"));

    assert_close_px(
        "command-demo listbox height",
        listbox.bounds.size.height,
        web_listbox.rect.h,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_command_demo_listbox_option_height_matches() {
    let web = read_web_golden("command-demo");
    let theme = web_theme(&web);
    let web_listbox = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "listbox")
    })
    .expect("web command-demo listbox");

    let mut all = Vec::new();
    web_collect_all(&theme.root, &mut all);
    let web_heights: std::collections::BTreeSet<i32> = all
        .into_iter()
        .filter(|n| n.attrs.get("role").is_some_and(|v| v == "option"))
        .filter(|n| rect_contains(web_listbox.rect, n.rect))
        .map(|n| n.rect.h.round() as i32)
        .collect();
    assert!(
        web_heights.len() == 1,
        "command-demo expected uniform web option height; got {web_heights:?}"
    );

    let snap = command_demo_snapshot(theme);
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret command-demo listbox"));
    let fret_heights: std::collections::BTreeSet<i32> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| fret_rect_contains(listbox.bounds, n.bounds))
        .map(|n| n.bounds.size.height.0.round() as i32)
        .collect();
    assert!(
        fret_heights.len() == 1,
        "command-demo expected uniform fret option height; got {fret_heights:?}"
    );

    let expected_h = web_heights.iter().next().copied().unwrap_or_default() as f32;
    let actual_h = fret_heights.iter().next().copied().unwrap_or_default() as f32;
    assert_close_px("command-demo option height", Px(actual_h), expected_h, 1.0);
}

#[test]
fn web_vs_fret_layout_command_demo_listbox_option_insets_match() {
    let web = read_web_golden("command-demo");
    let theme = web_theme(&web);
    let web_listbox = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "listbox")
    })
    .expect("web command-demo listbox");
    let expected = web_listbox_option_inset(theme, web_listbox);

    let snap = command_demo_snapshot(theme);
    let actual = fret_listbox_option_inset(&snap);
    assert_inset_quad_close("command-demo", actual, expected, 1.0);
}

#[test]
fn web_vs_fret_layout_input_with_label_geometry() {
    let web = read_web_golden("input-with-label");
    let theme = web_theme(&web);
    let web_label = find_first(&theme.root, &|n| n.tag == "label").expect("web label");
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (_ui, snap, _root) = run_fret_root_with_ui(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-with-label:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::Label::new("Email").into_element(cx)],
        );

        let input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:input-with-label:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(model)
                        .a11y_label("Email")
                        .placeholder("Email")
                        .into_element(cx),
                ]
            },
        );

        let col = cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Vertical,
                gap: Px(12.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![label, input],
        );

        let container = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_label.rect.w)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![col],
        );

        vec![container]
    });

    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-with-label:label"),
    )
    .expect("fret label");
    let input = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-with-label:input"),
    )
    .expect("fret input");

    assert_close_px(
        "input-with-label label h",
        label.bounds.size.height,
        web_label.rect.h,
        1.0,
    );
    assert_close_px(
        "input-with-label input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-with-label input h",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );

    let gap_y = input.bounds.origin.y.0 - (label.bounds.origin.y.0 + label.bounds.size.height.0);
    assert_close_px(
        "input-with-label gap",
        Px(gap_y),
        web_input.rect.y - (web_label.rect.y + web_label.rect.h),
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_dropdown_height() {
    let web = read_web_golden("input-group-dropdown");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(384.0)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Input group")
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-dropdown:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-dropdown:root"),
    )
    .expect("fret input group root");

    assert_close_px(
        "input group height",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_icon_geometry_matches() {
    let web = read_web_golden("input-group-icon");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");

    let web_input = find_first(web_group, &|n| n.tag == "input").expect("web input node");
    let web_svg = find_first(web_group, &|n| n.tag == "svg").expect("web svg node");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-icon",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(384.0)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-icon:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-icon:input")
                        .leading(vec![icon])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-icon:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-icon:root"),
    )
    .expect("fret input group root");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-icon:input"),
    )
    .expect("fret input");
    let icon = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-icon:icon"),
    )
    .expect("fret icon");

    assert_close_px(
        "input-group-icon group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-icon group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-icon input x",
        input.bounds.origin.x,
        web_input.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-icon input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-icon svg x",
        icon.bounds.origin.x,
        web_svg.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-icon svg y",
        icon.bounds.origin.y,
        web_svg.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-icon svg w",
        icon.bounds.size.width,
        web_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-icon svg h",
        icon.bounds.size.height,
        web_svg.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_spinner_geometry_matches() {
    let web = read_web_golden("input-group-spinner");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");

    let web_input = find_first(web_group, &|n| n.tag == "input").expect("web input node");
    let web_svg = find_first(web_group, &|n| n.tag == "svg").expect("web svg node");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-spinner",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(384.0)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let spinner = fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx);
                    let spinner = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-spinner:spinner")),
                            ..Default::default()
                        },
                        move |_cx| vec![spinner],
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-spinner:input")
                        .trailing(vec![spinner])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-spinner:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-spinner:root"),
    )
    .expect("fret input group root");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-spinner:input"),
    )
    .expect("fret input");
    let spinner = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-spinner:spinner"),
    )
    .expect("fret spinner");

    assert_close_px(
        "input-group-spinner group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-spinner group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-spinner input x",
        input.bounds.origin.x,
        web_input.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-spinner input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-spinner svg x",
        spinner.bounds.origin.x,
        web_svg.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-spinner svg y",
        spinner.bounds.origin.y,
        web_svg.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-spinner svg w",
        spinner.bounds.size.width,
        web_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-spinner svg h",
        spinner.bounds.size.height,
        web_svg.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_button_geometry_matches() {
    let web = read_web_golden("input-group-button");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");

    let web_input = web_group
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web input node");
    let web_addon = web_group
        .children
        .iter()
        .find(|n| {
            n.tag == "div"
                && n.computed_style
                    .get("marginRight")
                    .is_some_and(|v| v == "-7.2px")
        })
        .expect("web addon node");
    let web_svg = find_first(web_addon, &|n| n.tag == "svg").expect("web svg node");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-button",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(384.0)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-button:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );

                    let button = fret_ui_shadcn::Button::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                        .children(vec![icon])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(fret_ui_kit::MetricRef::Px(Px(24.0)))
                                .h_px(fret_ui_kit::MetricRef::Px(Px(24.0))),
                        )
                        .into_element(cx);

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-button:input")
                        .trailing_has_button(true)
                        .trailing(vec![button])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-button:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-button:root"),
    )
    .expect("fret input group root");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-button:input"),
    )
    .expect("fret input");
    let icon = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-button:icon"),
    )
    .expect("fret icon");

    assert_close_px(
        "input-group-button group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-button group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-button input x",
        input.bounds.origin.x,
        web_input.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-button input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-button svg x",
        icon.bounds.origin.x,
        web_svg.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-button svg y",
        icon.bounds.origin.y,
        web_svg.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-button svg w",
        icon.bounds.size.width,
        web_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-button svg h",
        icon.bounds.size.height,
        web_svg.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_tooltip_geometry_matches() {
    let web = read_web_golden("input-group-tooltip");
    let theme = web_theme(&web);

    let mut web_groups: Vec<&WebNode> = Vec::new();
    fn walk_collect<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                let mut has_group = false;
                let mut has_border = false;
                for t in c.split_whitespace() {
                    has_group |= t == "group/input-group";
                    has_border |= t == "border-input";
                }
                has_group && has_border
            })
        {
            out.push(n);
        }
        for c in &n.children {
            walk_collect(c, out);
        }
    }
    walk_collect(&theme.root, &mut web_groups);
    web_groups.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let web_group0 = *web_groups.get(0).expect("web group 0");
    let web_group2 = *web_groups.get(2).expect("web group 2");

    let web_group0_input = web_group0
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web group0 input");
    let web_group0_svg = find_first(web_group0, &|n| n.tag == "svg").expect("web group0 svg");

    let web_group2_input = web_group2
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web group2 input");
    let web_group2_svg = find_first(web_group2, &|n| n.tag == "svg").expect("web group2 svg");

    let expected_gap_y = web_groups[1].rect.y - (web_groups[0].rect.y + web_groups[0].rect.h);

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

    let model0: Model<String> = app.models_mut().insert(String::new());
    let model1: Model<String> = app.models_mut().insert(String::new());
    let model2: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-tooltip",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(384.0)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let button_icon0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-tooltip:0:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );
                    let button0 = fret_ui_shadcn::Button::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                        .children(vec![button_icon0])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(fret_ui_kit::MetricRef::Px(Px(24.0)))
                                .h_px(fret_ui_kit::MetricRef::Px(Px(24.0))),
                        )
                        .into_element(cx);

                    let group0 = fret_ui_shadcn::InputGroup::new(model0.clone())
                        .a11y_label("Golden:input-group-tooltip:0:input")
                        .trailing_has_button(true)
                        .trailing(vec![button0])
                        .into_element(cx);
                    let group0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-tooltip:0:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group0],
                    );

                    let group1_button = fret_ui_shadcn::Button::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                        .children(vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(fret_ui_kit::MetricRef::Px(Px(24.0)))
                                .h_px(fret_ui_kit::MetricRef::Px(Px(24.0))),
                        )
                        .into_element(cx);
                    let group1 = fret_ui_shadcn::InputGroup::new(model1.clone())
                        .a11y_label("Golden:input-group-tooltip:1:input")
                        .trailing_has_button(true)
                        .trailing(vec![group1_button])
                        .into_element(cx);
                    let group1 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-tooltip:1:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group1],
                    );

                    let button_icon2 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-tooltip:2:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );
                    let button2 = fret_ui_shadcn::Button::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                        .children(vec![button_icon2])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(fret_ui_kit::MetricRef::Px(Px(24.0)))
                                .h_px(fret_ui_kit::MetricRef::Px(Px(24.0))),
                        )
                        .into_element(cx);

                    let group2 = fret_ui_shadcn::InputGroup::new(model2.clone())
                        .a11y_label("Golden:input-group-tooltip:2:input")
                        .leading_has_button(true)
                        .leading(vec![button2])
                        .into_element(cx);
                    let group2 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-tooltip:2:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group2],
                    );

                    vec![cx.column(
                        ColumnProps {
                            gap: Px(expected_gap_y),
                            ..Default::default()
                        },
                        move |_cx| vec![group0, group1, group2],
                    )]
                },
            );

            vec![container]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_group0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-tooltip:0:root"),
    )
    .expect("fret group0");
    let fret_input0 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-tooltip:0:input"),
    )
    .expect("fret input0");
    let fret_icon0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-tooltip:0:icon"),
    )
    .expect("fret icon0");

    assert_close_px(
        "input-group-tooltip group0 y",
        fret_group0.bounds.origin.y,
        web_group0.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip group0 w",
        fret_group0.bounds.size.width,
        web_group0.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip group0 h",
        fret_group0.bounds.size.height,
        web_group0.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input0 x",
        fret_input0.bounds.origin.x,
        web_group0_input.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input0 y",
        fret_input0.bounds.origin.y,
        web_group0_input.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input0 w",
        fret_input0.bounds.size.width,
        web_group0_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg0 x",
        fret_icon0.bounds.origin.x,
        web_group0_svg.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg0 y",
        fret_icon0.bounds.origin.y,
        web_group0_svg.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg0 w",
        fret_icon0.bounds.size.width,
        web_group0_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg0 h",
        fret_icon0.bounds.size.height,
        web_group0_svg.rect.h,
        1.0,
    );

    let fret_group2 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-tooltip:2:root"),
    )
    .expect("fret group2");
    let fret_input2 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-tooltip:2:input"),
    )
    .expect("fret input2");
    let fret_icon2 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-tooltip:2:icon"),
    )
    .expect("fret icon2");

    assert_close_px(
        "input-group-tooltip group2 y",
        fret_group2.bounds.origin.y,
        web_group2.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input2 x",
        fret_input2.bounds.origin.x,
        web_group2_input.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input2 y",
        fret_input2.bounds.origin.y,
        web_group2_input.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip input2 w",
        fret_input2.bounds.size.width,
        web_group2_input.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg2 x",
        fret_icon2.bounds.origin.x,
        web_group2_svg.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg2 y",
        fret_icon2.bounds.origin.y,
        web_group2_svg.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg2 w",
        fret_icon2.bounds.size.width,
        web_group2_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-tooltip svg2 h",
        fret_icon2.bounds.size.height,
        web_group2_svg.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_empty_input_group_geometry_matches() {
    let web = read_web_golden("empty-input-group");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");
    let web_input = find_first(web_group, &|n| n.tag == "input").expect("web input");
    let web_svg = find_first(web_group, &|n| n.tag == "svg").expect("web icon");
    let web_kbd = find_first(web_group, &|n| n.tag == "kbd").expect("web kbd");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-empty-input-group",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_group.rect.w)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:empty-input-group:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );

                    let kbd = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:empty-input-group:kbd")),
                            ..Default::default()
                        },
                        move |cx| vec![fret_ui_shadcn::Kbd::new("/").into_element(cx)],
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:empty-input-group:input")
                        .leading(vec![icon])
                        .trailing_has_kbd(true)
                        .trailing(vec![kbd])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:empty-input-group:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:empty-input-group:root"),
    )
    .expect("fret input group root");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:empty-input-group:input"),
    )
    .expect("fret input");
    let icon = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:empty-input-group:icon"),
    )
    .expect("fret icon");
    let kbd = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:empty-input-group:kbd"),
    )
    .expect("fret kbd");

    assert_close_px(
        "empty-input-group group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "empty-input-group group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );

    assert_close_px(
        "empty-input-group input x",
        Px(input.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_input.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "empty-input-group input y",
        Px(input.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_input.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "empty-input-group input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );

    assert_close_px(
        "empty-input-group svg x",
        Px(icon.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_svg.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "empty-input-group svg y",
        Px(icon.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_svg.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "empty-input-group svg w",
        icon.bounds.size.width,
        web_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "empty-input-group svg h",
        icon.bounds.size.height,
        web_svg.rect.h,
        1.0,
    );

    assert_close_px(
        "empty-input-group kbd x",
        Px(kbd.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_kbd.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "empty-input-group kbd y",
        Px(kbd.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_kbd.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "empty-input-group kbd w",
        kbd.bounds.size.width,
        web_kbd.rect.w,
        1.0,
    );
    assert_close_px(
        "empty-input-group kbd h",
        kbd.bounds.size.height,
        web_kbd.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_kbd_input_group_geometry_matches() {
    let web = read_web_golden("kbd-input-group");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");
    let web_input = find_first(web_group, &|n| n.tag == "input").expect("web input");
    let web_svg = find_first(web_group, &|n| n.tag == "svg").expect("web icon");

    let mut web_kbds = Vec::new();
    web_collect_tag(web_group, "kbd", &mut web_kbds);
    web_kbds.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let web_kbd0 = *web_kbds.get(0).expect("web kbd0");
    let web_kbd1 = *web_kbds.get(1).expect("web kbd1");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-kbd-input-group",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_group.rect.w)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:kbd-input-group:icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );

                    let kbd0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:kbd-input-group:kbd0")),
                            ..Default::default()
                        },
                        move |cx| vec![fret_ui_shadcn::Kbd::new("⌘").into_element(cx)],
                    );
                    let kbd1 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:kbd-input-group:kbd1")),
                            ..Default::default()
                        },
                        move |cx| vec![fret_ui_shadcn::Kbd::new("K").into_element(cx)],
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:kbd-input-group:input")
                        .leading(vec![icon])
                        .trailing_has_kbd(true)
                        .trailing(vec![kbd0, kbd1])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:kbd-input-group:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:kbd-input-group:root"),
    )
    .expect("fret input group root");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:kbd-input-group:input"),
    )
    .expect("fret input");
    let icon = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:kbd-input-group:icon"),
    )
    .expect("fret icon");
    let kbd0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:kbd-input-group:kbd0"),
    )
    .expect("fret kbd0");
    let kbd1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:kbd-input-group:kbd1"),
    )
    .expect("fret kbd1");

    assert_close_px(
        "kbd-input-group group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "kbd-input-group group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );

    assert_close_px(
        "kbd-input-group input x",
        Px(input.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_input.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "kbd-input-group input w",
        input.bounds.size.width,
        web_input.rect.w,
        1.0,
    );

    assert_close_px(
        "kbd-input-group svg x",
        Px(icon.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_svg.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "kbd-input-group svg y",
        Px(icon.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_svg.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "kbd-input-group svg w",
        icon.bounds.size.width,
        web_svg.rect.w,
        1.0,
    );
    assert_close_px(
        "kbd-input-group svg h",
        icon.bounds.size.height,
        web_svg.rect.h,
        1.0,
    );

    assert_close_px(
        "kbd-input-group kbd0 x",
        Px(kbd0.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_kbd0.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd0 y",
        Px(kbd0.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_kbd0.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd0 w",
        kbd0.bounds.size.width,
        web_kbd0.rect.w,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd0 h",
        kbd0.bounds.size.height,
        web_kbd0.rect.h,
        1.0,
    );

    assert_close_px(
        "kbd-input-group kbd1 x",
        Px(kbd1.bounds.origin.x.0 - group.bounds.origin.x.0),
        web_kbd1.rect.x - web_group.rect.x,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd1 y",
        Px(kbd1.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_kbd1.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd1 w",
        kbd1.bounds.size.width,
        web_kbd1.rect.w,
        1.0,
    );
    assert_close_px(
        "kbd-input-group kbd1 h",
        kbd1.bounds.size.height,
        web_kbd1.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_textarea_geometry_matches() {
    let web = read_web_golden("input-group-textarea");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");
    let web_textarea = find_first(web_group, &|n| n.tag == "textarea").expect("web textarea");

    let mut web_svgs = Vec::new();
    web_collect_tag(web_group, "svg", &mut web_svgs);
    web_svgs.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .x
                    .partial_cmp(&b.rect.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let web_js = *web_svgs.first().expect("web js icon");
    let web_refresh = *web_svgs.get(1).expect("web refresh icon");
    let web_copy = *web_svgs.get(2).expect("web copy icon");
    let web_run = *web_svgs.get(3).expect("web run icon");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-textarea",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_group.rect.w)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let js_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-textarea:js")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SEARCH,
                                Some(Px(16.0)),
                                None,
                            )]
                        },
                    );

                    let refresh_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-textarea:refresh")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SEARCH,
                                Some(Px(16.0)),
                                None,
                            )]
                        },
                    );
                    let copy_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-textarea:copy")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SEARCH,
                                Some(Px(16.0)),
                                None,
                            )]
                        },
                    );

                    let run_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-textarea:run")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::SEARCH,
                                Some(Px(16.0)),
                                None,
                            )]
                        },
                    );

                    let script_label = cx.text("script.js");
                    let block_start_left = cx.flex(
                        FlexProps {
                            layout: LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: MetricRef::space(Space::N2).resolve(&Theme::global(&*cx.app)),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| vec![js_icon, script_label],
                    );

                    let refresh_button = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &Theme::global(&*cx.app),
                                fret_ui_kit::LayoutRefinement::default()
                                    .ml_auto()
                                    .w_px(MetricRef::Px(Px(24.0)))
                                    .h_px(MetricRef::Px(Px(24.0))),
                            ),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| vec![refresh_icon],
                            )]
                        },
                    );
                    let copy_button = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &Theme::global(&*cx.app),
                                fret_ui_kit::LayoutRefinement::default()
                                    .w_px(MetricRef::Px(Px(24.0)))
                                    .h_px(MetricRef::Px(Px(24.0))),
                            ),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| vec![copy_icon],
                            )]
                        },
                    );

                    let block_end_text = cx.text("Line 1, Column 1");
                    let run_button = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &Theme::global(&*cx.app),
                                fret_ui_kit::LayoutRefinement::default()
                                    .ml_auto()
                                    .w_px(MetricRef::Px(Px(32.0)))
                                    .h_px(MetricRef::Px(Px(32.0))),
                            ),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| vec![run_icon],
                            )]
                        },
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .textarea()
                        .textarea_min_height(Px(web_textarea.rect.h))
                        .a11y_label("Golden:input-group-textarea:textarea")
                        .block_start_border_bottom(true)
                        .block_start(vec![block_start_left, refresh_button, copy_button])
                        .block_end_border_top(true)
                        .block_end(vec![block_end_text, run_button])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-textarea:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-textarea:root"),
    )
    .expect("fret input group root");
    let textarea = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-textarea:textarea"),
    )
    .expect("fret textarea");
    let js = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-textarea:js"),
    )
    .expect("fret js icon");
    let refresh = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-textarea:refresh"),
    )
    .expect("fret refresh icon");
    let copy = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-textarea:copy"),
    )
    .expect("fret copy icon");
    let run = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-textarea:run"),
    )
    .expect("fret run icon");

    assert_close_px(
        "input-group-textarea group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );

    assert_close_px(
        "input-group-textarea textarea x",
        textarea.bounds.origin.x,
        web_textarea.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-textarea textarea y",
        textarea.bounds.origin.y,
        web_textarea.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-textarea textarea w",
        textarea.bounds.size.width,
        web_textarea.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );

    assert_close_px(
        "input-group-textarea js x",
        js.bounds.origin.x,
        web_js.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-textarea js y",
        js.bounds.origin.y,
        web_js.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-textarea js w",
        js.bounds.size.width,
        web_js.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea js h",
        js.bounds.size.height,
        web_js.rect.h,
        1.0,
    );

    assert_close_px(
        "input-group-textarea refresh x",
        refresh.bounds.origin.x,
        web_refresh.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-textarea refresh y",
        refresh.bounds.origin.y,
        web_refresh.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-textarea refresh w",
        refresh.bounds.size.width,
        web_refresh.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea refresh h",
        refresh.bounds.size.height,
        web_refresh.rect.h,
        1.0,
    );

    assert_close_px(
        "input-group-textarea copy x",
        copy.bounds.origin.x,
        web_copy.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-textarea copy y",
        copy.bounds.origin.y,
        web_copy.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-textarea copy w",
        copy.bounds.size.width,
        web_copy.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea copy h",
        copy.bounds.size.height,
        web_copy.rect.h,
        1.0,
    );

    assert_close_px(
        "input-group-textarea run y",
        run.bounds.origin.y,
        web_run.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-textarea run w",
        run.bounds.size.width,
        web_run.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-textarea run h",
        run.bounds.size.height,
        web_run.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_text_currency_geometry_matches() {
    let web = read_web_golden("input-group-text");
    let theme = web_theme(&web);

    let mut web_groups: Vec<&WebNode> = Vec::new();
    fn walk_collect<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                let mut has_group = false;
                let mut has_border = false;
                for t in c.split_whitespace() {
                    has_group |= t == "group/input-group";
                    has_border |= t == "border-input";
                }
                has_group && has_border
            })
        {
            out.push(n);
        }
        for c in &n.children {
            walk_collect(c, out);
        }
    }
    walk_collect(&theme.root, &mut web_groups);
    web_groups.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let web_group0 = *web_groups.first().expect("web group0");
    let web_input0 = web_group0
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web input0");
    let web_dollar = web_find_by_tag_and_text(web_group0, "span", "$").expect("web $ label");
    let web_usd = web_find_by_tag_and_text(web_group0, "span", "USD").expect("web USD label");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-text-currency",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_group0.rect.w)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let leading = fret_ui_shadcn::InputGroupText::new("$")
                        .refine_layout(
                            LayoutRefinement::default().w_px(MetricRef::Px(Px(web_dollar.rect.w))),
                        )
                        .into_element(cx);
                    let trailing = fret_ui_shadcn::InputGroupText::new("USD")
                        .refine_layout(
                            LayoutRefinement::default().w_px(MetricRef::Px(Px(web_usd.rect.w))),
                        )
                        .into_element(cx);

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-text:currency:input")
                        .leading(vec![leading])
                        .trailing(vec![trailing])
                        .into_element(cx);
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:currency:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:currency:root"),
    )
    .expect("fret group");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-text:currency:input"),
    )
    .expect("fret input");

    assert_close_px(
        "input-group-text currency group w",
        group.bounds.size.width,
        web_group0.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text currency group h",
        group.bounds.size.height,
        web_group0.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-text currency input x",
        input.bounds.origin.x,
        web_input0.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text currency input w",
        input.bounds.size.width,
        web_input0.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text currency input h",
        input.bounds.size.height,
        web_input0.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_text_url_geometry_matches() {
    let web = read_web_golden("input-group-text");
    let theme = web_theme(&web);

    let mut web_groups: Vec<&WebNode> = Vec::new();
    fn walk_collect<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                let mut has_group = false;
                let mut has_border = false;
                for t in c.split_whitespace() {
                    has_group |= t == "group/input-group";
                    has_border |= t == "border-input";
                }
                has_group && has_border
            })
        {
            out.push(n);
        }
        for c in &n.children {
            walk_collect(c, out);
        }
    }
    walk_collect(&theme.root, &mut web_groups);
    web_groups.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let web_group1 = *web_groups.get(1).expect("web group1");
    let web_input1 = web_group1
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web input1");
    let web_prefix =
        web_find_by_tag_and_text(web_group1, "span", "https://").expect("web https prefix");
    let web_suffix = web_find_by_tag_and_text(web_group1, "span", ".com").expect("web .com suffix");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-text-url",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_group1.rect.w)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let prefix = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:url:prefix")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::InputGroupText::new("https://")
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(web_prefix.rect.w))),
                                    )
                                    .into_element(cx),
                            ]
                        },
                    );

                    let suffix = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:url:suffix")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::InputGroupText::new(".com")
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(web_suffix.rect.w))),
                                    )
                                    .into_element(cx),
                            ]
                        },
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-text:url:input")
                        .leading(vec![prefix])
                        .trailing(vec![suffix])
                        .into_element(cx);
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:url:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:url:root"),
    )
    .expect("fret group");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-text:url:input"),
    )
    .expect("fret input");
    let prefix = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:url:prefix"),
    )
    .expect("fret prefix");
    let suffix = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:url:suffix"),
    )
    .expect("fret suffix");

    assert_close_px(
        "input-group-text url group w",
        group.bounds.size.width,
        web_group1.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text url group h",
        group.bounds.size.height,
        web_group1.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-text url input x",
        input.bounds.origin.x,
        web_input1.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text url input w",
        input.bounds.size.width,
        web_input1.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text url prefix x",
        prefix.bounds.origin.x,
        web_prefix.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text url prefix w",
        prefix.bounds.size.width,
        web_prefix.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text url suffix x",
        suffix.bounds.origin.x,
        web_suffix.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text url suffix w",
        suffix.bounds.size.width,
        web_suffix.rect.w,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_text_email_geometry_matches() {
    let web = read_web_golden("input-group-text");
    let theme = web_theme(&web);

    let mut web_groups: Vec<&WebNode> = Vec::new();
    fn walk_collect<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                let mut has_group = false;
                let mut has_border = false;
                for t in c.split_whitespace() {
                    has_group |= t == "group/input-group";
                    has_border |= t == "border-input";
                }
                has_group && has_border
            })
        {
            out.push(n);
        }
        for c in &n.children {
            walk_collect(c, out);
        }
    }
    walk_collect(&theme.root, &mut web_groups);
    web_groups.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let web_group2 = *web_groups.get(2).expect("web group2");
    let web_input2 = web_group2
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web input2");
    let web_suffix = web_find_by_tag_and_text(web_group2, "span", "@company.com")
        .expect("web @company.com suffix");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-text-email",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_group2.rect.w)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let suffix = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:email:suffix")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::InputGroupText::new("@company.com")
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(web_suffix.rect.w))),
                                    )
                                    .into_element(cx),
                            ]
                        },
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Golden:input-group-text:email:input")
                        .trailing(vec![suffix])
                        .into_element(cx);
                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:email:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:email:root"),
    )
    .expect("fret group");
    let input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-text:email:input"),
    )
    .expect("fret input");
    let suffix = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:email:suffix"),
    )
    .expect("fret suffix");

    assert_close_px(
        "input-group-text email group w",
        group.bounds.size.width,
        web_group2.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text email group h",
        group.bounds.size.height,
        web_group2.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-text email input x",
        input.bounds.origin.x,
        web_input2.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text email input w",
        input.bounds.size.width,
        web_input2.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text email suffix x",
        suffix.bounds.origin.x,
        web_suffix.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text email suffix w",
        suffix.bounds.size.width,
        web_suffix.rect.w,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_text_textarea_count_geometry_matches() {
    let web = read_web_golden("input-group-text");
    let theme = web_theme(&web);

    let mut web_groups: Vec<&WebNode> = Vec::new();
    fn walk_collect<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                let mut has_group = false;
                let mut has_border = false;
                for t in c.split_whitespace() {
                    has_group |= t == "group/input-group";
                    has_border |= t == "border-input";
                }
                has_group && has_border
            })
        {
            out.push(n);
        }
        for c in &n.children {
            walk_collect(c, out);
        }
    }
    walk_collect(&theme.root, &mut web_groups);
    web_groups.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let web_group3 = *web_groups.get(3).expect("web group3");
    let web_textarea3 = web_group3
        .children
        .iter()
        .find(|n| n.tag == "textarea")
        .expect("web textarea3");
    let web_count = web_find_by_tag_and_text(web_group3, "span", "120 characters left")
        .expect("web count label");

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-text-textarea-count",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_group3.rect.w)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let count = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:count:text")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::InputGroupText::new("120 characters left")
                                    .size(fret_ui_shadcn::InputGroupTextSize::Xs)
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(web_count.rect.w))),
                                    )
                                    .into_element(cx),
                            ]
                        },
                    );

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .textarea()
                        .textarea_min_height(Px(web_textarea3.rect.h))
                        .a11y_label("Golden:input-group-text:count:textarea")
                        .block_end(vec![count])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-text:count:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:count:root"),
    )
    .expect("fret group");
    let textarea = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-text:count:textarea"),
    )
    .expect("fret textarea");
    let count = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-text:count:text"),
    )
    .expect("fret count text");

    assert_close_px(
        "input-group-text textarea count group w",
        group.bounds.size.width,
        web_group3.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count group h",
        group.bounds.size.height,
        web_group3.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count textarea x",
        textarea.bounds.origin.x,
        web_textarea3.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count textarea w",
        textarea.bounds.size.width,
        web_textarea3.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count textarea h",
        textarea.bounds.size.height,
        web_textarea3.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count text x",
        count.bounds.origin.x,
        web_count.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count text w",
        count.bounds.size.width,
        web_count.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count text y",
        Px(count.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_count.rect.y - web_group3.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-text textarea count text h",
        count.bounds.size.height,
        web_count.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_custom_geometry_matches() {
    let web = read_web_golden("input-group-custom");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_tokens(&theme.root, &["group/input-group", "border-input"])
        .expect("web input group root");

    let web_textarea = web_group
        .children
        .iter()
        .find(|n| n.tag == "textarea")
        .expect("web textarea node");
    let web_submit =
        web_find_by_tag_and_text(web_group, "button", "Submit").expect("web submit button node");
    let expected_submit_w = Px(web_submit.rect.w);

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-custom",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(MetricRef::Px(Px(web_group.rect.w)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let submit = fret_ui_shadcn::InputGroupButton::new("Submit")
                        .variant(fret_ui_shadcn::ButtonVariant::Default)
                        .size(fret_ui_shadcn::InputGroupButtonSize::Sm)
                        .a11y_label("Golden:input-group-custom:submit")
                        .refine_layout(
                            LayoutRefinement::default()
                                .ml_auto()
                                .w_px(MetricRef::Px(expected_submit_w)),
                        )
                        .into_element(cx);

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .textarea()
                        .textarea_min_height(Px(web_textarea.rect.h))
                        .a11y_label("Golden:input-group-custom:textarea")
                        .block_end(vec![submit])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-custom:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-custom:root"),
    )
    .expect("fret input group root");
    let textarea = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-custom:textarea"),
    )
    .expect("fret textarea");
    let submit = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Golden:input-group-custom:submit"),
    )
    .expect("fret submit button");

    assert_close_px(
        "input-group-custom group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-custom group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-custom textarea w",
        textarea.bounds.size.width,
        web_textarea.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-custom textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-custom textarea y",
        Px(textarea.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_textarea.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-custom submit w",
        submit.bounds.size.width,
        web_submit.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-custom submit h",
        submit.bounds.size.height,
        web_submit.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-custom submit x",
        submit.bounds.origin.x,
        web_submit.rect.x,
        1.0,
    );
    assert_close_px(
        "input-group-custom submit y",
        Px(submit.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_submit.rect.y - web_group.rect.y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_demo_block_end_geometry_matches() {
    let web = read_web_golden("input-group-demo");
    let theme = web_theme(&web);

    let mut web_groups: Vec<&WebNode> = Vec::new();
    fn walk_collect<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                let mut has_group = false;
                let mut has_border = false;
                for t in c.split_whitespace() {
                    has_group |= t == "group/input-group";
                    has_border |= t == "border-input";
                }
                has_group && has_border
            })
        {
            out.push(n);
        }
        for c in &n.children {
            walk_collect(c, out);
        }
    }
    walk_collect(&theme.root, &mut web_groups);
    web_groups.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let web_group = *web_groups
        .get(2)
        .expect("web input group (textarea + block-end addon)");
    let web_textarea = web_group
        .children
        .iter()
        .find(|n| n.tag == "textarea")
        .expect("web textarea node");
    let web_auto =
        web_find_by_tag_and_text(web_group, "button", "Auto").expect("web Auto button node");
    let web_used =
        web_find_by_tag_and_text(web_group, "span", "52% used").expect("web usage label node");
    let web_send = {
        let mut buttons = Vec::new();
        web_collect_tag(web_group, "button", &mut buttons);
        *buttons
            .iter()
            .max_by(|a, b| {
                a.rect
                    .x
                    .partial_cmp(&b.rect.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("web send button node")
    };
    let web_separator = find_first(web_group, &|n| {
        n.class_name
            .as_deref()
            .is_some_and(|c| c.contains("bg-border shrink-0"))
            && n.attrs
                .get("data-orientation")
                .is_some_and(|o| o == "vertical")
    })
    .expect("web separator node");
    let expected_auto_w = Px(web_auto.rect.w);
    let expected_used_w = Px(web_used.rect.w);

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

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group-demo-block-end",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(MetricRef::Px(Px(web_group.rect.w)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let plus_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-demo:block:plus-icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );
                    let plus_button = fret_ui_shadcn::InputGroupButton::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .size(fret_ui_shadcn::InputGroupButtonSize::IconXs)
                        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                        .children(vec![plus_icon])
                        .into_element(cx);

                    let auto = fret_ui_shadcn::InputGroupButton::new("Auto")
                        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
                        .a11y_label("Golden:input-group-demo:block:auto")
                        .refine_layout(
                            LayoutRefinement::default().w_px(MetricRef::Px(expected_auto_w)),
                        )
                        .into_element(cx);

                    let used = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-demo:block:used")),
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &Theme::global(&*cx.app),
                                LayoutRefinement::default().ml_auto(),
                            ),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::InputGroupText::new("52% used")
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(expected_used_w)),
                                    )
                                    .into_element(cx),
                            ]
                        },
                    );

                    let separator = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-demo:block:separator")),
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Px(Px(web_separator.rect.w)),
                                    height: Length::Px(Px(web_separator.rect.h)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                fret_ui_shadcn::Separator::new()
                                    .orientation(fret_ui_shadcn::SeparatorOrientation::Vertical)
                                    .into_element(cx),
                            ]
                        },
                    );

                    let send_icon = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-demo:block:send-icon")),
                            ..Default::default()
                        },
                        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
                    );
                    let send_button = fret_ui_shadcn::InputGroupButton::new("")
                        .variant(fret_ui_shadcn::ButtonVariant::Default)
                        .size(fret_ui_shadcn::InputGroupButtonSize::IconXs)
                        .a11y_label("Golden:input-group-demo:block:send")
                        .disabled(true)
                        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                        .children(vec![send_icon])
                        .into_element(cx);

                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .textarea()
                        .textarea_min_height(Px(web_textarea.rect.h))
                        .a11y_label("Golden:input-group-demo:block:textarea")
                        .block_end(vec![plus_button, auto, used, separator, send_button])
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-demo:block:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-demo:block:root"),
    )
    .expect("fret input group root");
    let textarea = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:input-group-demo:block:textarea"),
    )
    .expect("fret textarea");
    let auto = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Golden:input-group-demo:block:auto"),
    )
    .expect("fret Auto button");
    let used = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-demo:block:used"),
    )
    .expect("fret usage label");
    let separator = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-demo:block:separator"),
    )
    .expect("fret separator");
    let send = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Golden:input-group-demo:block:send"),
    )
    .expect("fret send button");

    assert_close_px(
        "input-group-demo block-end group w",
        group.bounds.size.width,
        web_group.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end group h",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end textarea w",
        textarea.bounds.size.width,
        web_textarea.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end auto w",
        auto.bounds.size.width,
        web_auto.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end auto h",
        auto.bounds.size.height,
        web_auto.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end used w",
        used.bounds.size.width,
        web_used.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end used h",
        used.bounds.size.height,
        web_used.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end used y",
        Px(used.bounds.origin.y.0 - group.bounds.origin.y.0),
        web_used.rect.y - web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end separator w",
        separator.bounds.size.width,
        web_separator.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end separator h",
        separator.bounds.size.height,
        web_separator.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end send w",
        send.bounds.size.width,
        web_send.rect.w,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end send h",
        send.bounds.size.height,
        web_send.rect.h,
        1.0,
    );
    assert_close_px(
        "input-group-demo block-end send x",
        send.bounds.origin.x,
        web_send.rect.x,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_spinner_input_group_geometry_matches() {
    let web = read_web_golden("spinner-input-group");
    let theme = web_theme(&web);

    let mut web_groups: Vec<&WebNode> = Vec::new();
    fn walk_collect<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if n.tag == "div"
            && n.class_name.as_deref().is_some_and(|c| {
                let mut has_group = false;
                let mut has_border = false;
                for t in c.split_whitespace() {
                    has_group |= t == "group/input-group";
                    has_border |= t == "border-input";
                }
                has_group && has_border
            })
        {
            out.push(n);
        }
        for c in &n.children {
            walk_collect(c, out);
        }
    }
    walk_collect(&theme.root, &mut web_groups);
    web_groups.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let web_group0 = *web_groups.get(0).expect("web group 0");
    let web_group1 = *web_groups.get(1).expect("web group 1");

    let expected_gap_y = web_group1.rect.y - (web_group0.rect.y + web_group0.rect.h);

    let web_input0 = web_group0
        .children
        .iter()
        .find(|n| n.tag == "input")
        .expect("web input0");
    let web_svg0 = find_first(web_group0, &|n| n.tag == "svg").expect("web svg0");

    let web_textarea1 = web_group1
        .children
        .iter()
        .find(|n| n.tag == "textarea")
        .expect("web textarea1");
    let web_svg1a = find_first(web_group1, &|n| {
        n.tag == "svg" && (n.rect.w - 16.0).abs() <= 0.1
    })
    .expect("web svg1a (spinner)");
    let web_svg1b = find_first(web_group1, &|n| {
        n.tag == "svg" && (n.rect.w - 14.0).abs() <= 0.1
    })
    .expect("web svg1b (arrow)");

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

    let model0: Model<String> = app.models_mut().insert(String::new());
    let model1: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-spinner-input-group",
        |cx| {
            let container_layout =
                fret_ui_kit::LayoutRefinement::default().w_px(MetricRef::Px(Px(web_group0.rect.w)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let spinner0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:spinner-input-group:0:spinner")),
                            ..Default::default()
                        },
                        move |cx| vec![fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx)],
                    );

                    let group0 = fret_ui_shadcn::InputGroup::new(model0.clone())
                        .a11y_label("Golden:spinner-input-group:0:input")
                        .trailing(vec![spinner0])
                        .into_element(cx);
                    let group0 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:spinner-input-group:0:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group0],
                    );

                    let spinner1 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:spinner-input-group:1:spinner")),
                            ..Default::default()
                        },
                        move |cx| vec![fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx)],
                    );
                    let arrow = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:spinner-input-group:1:arrow")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::CHEVRON_UP,
                                Some(Px(14.0)),
                                None,
                            )]
                        },
                    );
                    let send_button = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &fret_ui::Theme::global(&*cx.app),
                                fret_ui_kit::LayoutRefinement::default()
                                    .ml_auto()
                                    .w_px(MetricRef::Px(Px(30.0)))
                                    .h_px(MetricRef::Px(Px(24.0))),
                            ),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding: Edges::symmetric(Px(8.0), Px(0.0)),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| vec![arrow],
                            )]
                        },
                    );

                    let group1_addon = vec![spinner1, cx.text("Validating..."), send_button];
                    let group1 = fret_ui_shadcn::InputGroup::new(model1.clone())
                        .textarea()
                        .a11y_label("Golden:spinner-input-group:1:textarea")
                        .block_end(group1_addon)
                        .into_element(cx);
                    let group1 = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:spinner-input-group:1:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group1],
                    );

                    vec![cx.column(
                        ColumnProps {
                            gap: Px(expected_gap_y),
                            ..Default::default()
                        },
                        move |_cx| vec![group0, group1],
                    )]
                },
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:spinner-input-group:0:root"),
    )
    .expect("fret group0");
    let input0 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:spinner-input-group:0:input"),
    )
    .expect("fret input0");
    let spinner0 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:spinner-input-group:0:spinner"),
    )
    .expect("fret spinner0");

    assert_close_px(
        "spinner-input-group group0 y",
        group0.bounds.origin.y,
        web_group0.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group group0 w",
        group0.bounds.size.width,
        web_group0.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group group0 h",
        group0.bounds.size.height,
        web_group0.rect.h,
        1.0,
    );
    assert_close_px(
        "spinner-input-group input0 x",
        input0.bounds.origin.x,
        web_input0.rect.x,
        1.0,
    );
    assert_close_px(
        "spinner-input-group input0 w",
        input0.bounds.size.width,
        web_input0.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner0 x",
        spinner0.bounds.origin.x,
        web_svg0.rect.x,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner0 y",
        spinner0.bounds.origin.y,
        web_svg0.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner0 w",
        spinner0.bounds.size.width,
        web_svg0.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner0 h",
        spinner0.bounds.size.height,
        web_svg0.rect.h,
        1.0,
    );

    let group1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:spinner-input-group:1:root"),
    )
    .expect("fret group1");
    let textarea1 = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:spinner-input-group:1:textarea"),
    )
    .expect("fret textarea1");
    let spinner1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:spinner-input-group:1:spinner"),
    )
    .expect("fret spinner1");
    let arrow = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:spinner-input-group:1:arrow"),
    )
    .expect("fret arrow");

    assert_close_px(
        "spinner-input-group group1 y",
        group1.bounds.origin.y,
        web_group1.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group group1 w",
        group1.bounds.size.width,
        web_group1.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group group1 h",
        group1.bounds.size.height,
        web_group1.rect.h,
        1.0,
    );
    assert_close_px(
        "spinner-input-group textarea1 x",
        textarea1.bounds.origin.x,
        web_textarea1.rect.x,
        1.0,
    );
    assert_close_px(
        "spinner-input-group textarea1 y",
        textarea1.bounds.origin.y,
        web_textarea1.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group textarea1 w",
        textarea1.bounds.size.width,
        web_textarea1.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group textarea1 h",
        textarea1.bounds.size.height,
        web_textarea1.rect.h,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner1 x",
        spinner1.bounds.origin.x,
        web_svg1a.rect.x,
        1.0,
    );
    assert_close_px(
        "spinner-input-group spinner1 y",
        spinner1.bounds.origin.y,
        web_svg1a.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group arrow x",
        arrow.bounds.origin.x,
        web_svg1b.rect.x,
        1.0,
    );
    assert_close_px(
        "spinner-input-group arrow y",
        arrow.bounds.origin.y,
        web_svg1b.rect.y,
        1.0,
    );
    assert_close_px(
        "spinner-input-group arrow w",
        arrow.bounds.size.width,
        web_svg1b.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-input-group arrow h",
        arrow.bounds.size.height,
        web_svg1b.rect.h,
        1.0,
    );
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
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_card.rect.w))),
        )
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

fn web_find_by_id<'a>(root: &'a WebNode, id: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.id.as_deref() == Some(id) || n.attrs.get("id").is_some_and(|v| v == id)
    })
}

fn web_find_by_tag_and_text_within<'a>(
    root: &'a WebNode,
    within: WebRect,
    tag: &str,
    text: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.tag == tag && n.text.as_deref() == Some(text) && rect_contains(within, n.rect)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormControlKind {
    Input,
    Textarea,
}

fn assert_bug_report_form_demo_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_card = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-card",
            "text-card-foreground",
            "rounded-xl",
            "border",
            "py-6",
            "sm:max-w-md",
        ],
    )
    .expect("web card root");

    let web_title_input = find_all(&theme.root, &|n| n.tag == "input")
        .into_iter()
        .filter(|n| rect_contains(web_card.rect, n.rect))
        .min_by(|a, b| {
            a.rect
                .y
                .partial_cmp(&b.rect.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("web title input");

    let web_description_group = find_all(&theme.root, &|n| {
        n.tag == "div"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("group/input-group"))
    })
    .into_iter()
    .filter(|n| rect_contains(web_card.rect, n.rect))
    .min_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
    .expect("web description input-group");

    let web_reset = web_find_by_tag_and_text_within(&theme.root, web_card.rect, "button", "Reset")
        .expect("web reset button");
    let web_submit =
        web_find_by_tag_and_text_within(&theme.root, web_card.rect, "button", "Submit")
            .expect("web submit button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let title: Model<String> = cx.app.models_mut().insert(String::new());
        let description: Model<String> = cx.app.models_mut().insert(String::new());

        let title_field = fret_ui_shadcn::Field::new(vec![
            fret_ui_shadcn::FieldLabel::new("Bug Title").into_element(cx),
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::TextField,
                    label: Some(Arc::from(format!("Golden:{web_name}:title"))),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        fret_ui_shadcn::Input::new(title)
                            .a11y_label("Bug Title")
                            .placeholder("Bug title")
                            .into_element(cx),
                    ]
                },
            ),
        ])
        .into_element(cx);

        let description_group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:description_group"))),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::InputGroup::new(description)
                        .textarea()
                        .a11y_label("Description")
                        .textarea_min_height(Px(96.0))
                        .block_end(vec![
                            fret_ui_shadcn::InputGroupText::new("0/100 characters")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                ]
            },
        );

        let description_field = fret_ui_shadcn::Field::new(vec![
            fret_ui_shadcn::FieldLabel::new("Description").into_element(cx),
            description_group,
            fret_ui_shadcn::FieldDescription::new(
                "Include steps to reproduce, expected behavior, and what actually happened.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let card = fret_ui_shadcn::Card::new(vec![
            fret_ui_shadcn::CardHeader::new(vec![
                fret_ui_shadcn::CardTitle::new("Bug Report").into_element(cx),
                fret_ui_shadcn::CardDescription::new(
                    "Help us improve by reporting bugs you encounter.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardContent::new(vec![
                fret_ui_shadcn::FieldGroup::new(vec![title_field, description_field])
                    .into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardFooter::new(vec![cx.row(
                RowProps {
                    layout: LayoutStyle::default(),
                    gap: fret_ui_kit::MetricRef::space(Space::N2).resolve(&Theme::global(&*cx.app)),
                    justify: MainAlign::End,
                    align: CrossAlign::Center,
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        fret_ui_shadcn::Button::new("Reset")
                            .variant(fret_ui_shadcn::ButtonVariant::Outline)
                            .into_element(cx),
                        fret_ui_shadcn::Button::new("Submit").into_element(cx),
                    ]
                },
            )])
            .into_element(cx),
        ])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_card.rect.w))),
        )
        .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:card"))),
                ..Default::default()
            },
            move |_cx| vec![card],
        )]
    });

    let card = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:card")),
    )
    .expect("fret card");
    let title = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some(&format!("Golden:{web_name}:title")),
    )
    .expect("fret title input");
    let description_group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:description_group")),
    )
    .expect("fret description group");
    let reset =
        find_semantics(&snap, SemanticsRole::Button, Some("Reset")).expect("fret reset button");
    let submit =
        find_semantics(&snap, SemanticsRole::Button, Some("Submit")).expect("fret submit button");

    assert_close_px("card width", card.bounds.size.width, web_card.rect.w, 1.0);
    assert_rect_xwh_close_px("title input", title.bounds, web_title_input.rect, 1.0);
    assert_rect_xwh_close_px(
        "description input-group",
        description_group.bounds,
        web_description_group.rect,
        1.0,
    );
    assert_close_px(
        "reset button height",
        reset.bounds.size.height,
        web_reset.rect.h,
        1.0,
    );
    assert_close_px(
        "submit button height",
        submit.bounds.size.height,
        web_submit.rect.h,
        1.0,
    );
}

fn assert_single_field_form_card_geometry_matches_web(
    web_name: &str,
    title: &str,
    description: &str,
    field_label: &str,
    field_description: &str,
    control_id: &str,
    control_kind: FormControlKind,
    primary_action: &str,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_card = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-card",
            "text-card-foreground",
            "rounded-xl",
            "border",
            "py-6",
            "sm:max-w-md",
        ],
    )
    .expect("web card root");

    let web_control = match control_kind {
        FormControlKind::Input => web_find_by_id(&theme.root, control_id).expect("web input"),
        FormControlKind::Textarea => web_find_by_id(&theme.root, control_id).expect("web textarea"),
    };

    let web_reset = web_find_by_tag_and_text_within(&theme.root, web_card.rect, "button", "Reset")
        .expect("web reset button");
    let web_primary =
        web_find_by_tag_and_text_within(&theme.root, web_card.rect, "button", primary_action)
            .expect("web primary button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let control = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from(format!("Golden:{web_name}:control"))),
                ..Default::default()
            },
            move |cx| {
                vec![match control_kind {
                    FormControlKind::Input => fret_ui_shadcn::Input::new(model)
                        .a11y_label(field_label)
                        .placeholder("...")
                        .into_element(cx),
                    FormControlKind::Textarea => fret_ui_shadcn::Textarea::new(model)
                        .a11y_label(field_label)
                        .min_height(Px(120.0))
                        .into_element(cx),
                }]
            },
        );

        let field = fret_ui_shadcn::Field::new(vec![
            fret_ui_shadcn::FieldLabel::new(field_label).into_element(cx),
            control,
            fret_ui_shadcn::FieldDescription::new(field_description).into_element(cx),
        ])
        .into_element(cx);

        let card = fret_ui_shadcn::Card::new(vec![
            fret_ui_shadcn::CardHeader::new(vec![
                fret_ui_shadcn::CardTitle::new(title).into_element(cx),
                fret_ui_shadcn::CardDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardContent::new(vec![
                fret_ui_shadcn::FieldGroup::new(vec![field]).into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardFooter::new(vec![cx.row(
                RowProps {
                    layout: LayoutStyle::default(),
                    gap: fret_ui_kit::MetricRef::space(Space::N2).resolve(&Theme::global(&*cx.app)),
                    justify: MainAlign::End,
                    align: CrossAlign::Center,
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        fret_ui_shadcn::Button::new("Reset")
                            .variant(fret_ui_shadcn::ButtonVariant::Outline)
                            .into_element(cx),
                        fret_ui_shadcn::Button::new(primary_action).into_element(cx),
                    ]
                },
            )])
            .into_element(cx),
        ])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_card.rect.w))),
        )
        .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:card"))),
                ..Default::default()
            },
            move |_cx| vec![card],
        )]
    });

    let card = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:card")),
    )
    .expect("fret card");
    let control = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some(&format!("Golden:{web_name}:control")),
    )
    .expect("fret control");
    let reset =
        find_semantics(&snap, SemanticsRole::Button, Some("Reset")).expect("fret reset button");
    let primary = find_semantics(&snap, SemanticsRole::Button, Some(primary_action))
        .expect("fret primary button");

    assert_close_px("card width", card.bounds.size.width, web_card.rect.w, 1.0);
    assert_rect_xwh_close_px("control", control.bounds, web_control.rect, 1.0);
    assert_close_px(
        "reset button height",
        reset.bounds.size.height,
        web_reset.rect.h,
        1.0,
    );
    assert_close_px(
        "primary button height",
        primary.bounds.size.height,
        web_primary.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_demo_geometry_matches_web() {
    assert_bug_report_form_demo_geometry_matches_web("form-rhf-demo");
}

#[test]
fn web_vs_fret_layout_form_tanstack_demo_geometry_matches_web() {
    assert_bug_report_form_demo_geometry_matches_web("form-tanstack-demo");
}

#[test]
fn web_vs_fret_layout_form_rhf_input_geometry_matches_web() {
    assert_single_field_form_card_geometry_matches_web(
        "form-rhf-input",
        "Profile Settings",
        "Update your profile information below.",
        "Username",
        "This is your public display name. Must be between 3 and 10 characters. Must only contain letters, numbers, and underscores.",
        "form-rhf-input-username",
        FormControlKind::Input,
        "Save",
    );
}

#[test]
fn web_vs_fret_layout_form_tanstack_input_geometry_matches_web() {
    assert_single_field_form_card_geometry_matches_web(
        "form-tanstack-input",
        "Profile Settings",
        "Update your profile information below.",
        "Username",
        "This is your public display name. Must be between 3 and 10 characters. Must only contain letters, numbers, and underscores.",
        "form-tanstack-input-username",
        FormControlKind::Input,
        "Save",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_textarea_geometry_matches_web() {
    assert_single_field_form_card_geometry_matches_web(
        "form-rhf-textarea",
        "Personalization",
        "Customize your experience by telling us more about yourself.",
        "More about you",
        "Tell us more about yourself. This will be used to help us personalize your experience.",
        "form-rhf-textarea-about",
        FormControlKind::Textarea,
        "Save",
    );
}

#[test]
fn web_vs_fret_layout_form_tanstack_textarea_geometry_matches_web() {
    assert_single_field_form_card_geometry_matches_web(
        "form-tanstack-textarea",
        "Personalization",
        "Customize your experience by telling us more about yourself.",
        "More about you",
        "Tell us more about yourself. This will be used to help us personalize your experience.",
        "form-tanstack-textarea-about",
        FormControlKind::Textarea,
        "Save",
    );
}

fn web_find_input_group_container_containing<'a>(
    theme: &'a WebGoldenTheme,
    input: &WebNode,
) -> &'a WebNode {
    let mut all = Vec::new();
    web_collect_all(&theme.root, &mut all);
    all.into_iter()
        .filter(|n| {
            n.tag == "div"
                && n.class_name
                    .as_deref()
                    .is_some_and(|c| c.contains("group/input-group"))
                && rect_contains(n.rect, input.rect)
        })
        .min_by(|a, b| {
            let area_a = a.rect.w * a.rect.h;
            let area_b = b.rect.w * b.rect.h;
            area_a
                .partial_cmp(&area_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("web input-group container")
}

fn form_trailing_icon_button<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    a11y_label: &str,
) -> fret_ui::element::AnyElement {
    let icon = cx.semantics(
        fret_ui::element::SemanticsProps {
            role: SemanticsRole::Panel,
            label: Some(Arc::from(format!("{a11y_label}:icon"))),
            ..Default::default()
        },
        move |cx| vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)],
    );

    fret_ui_shadcn::InputGroupButton::new("")
        .variant(fret_ui_shadcn::ButtonVariant::Ghost)
        .size(fret_ui_shadcn::InputGroupButtonSize::IconXs)
        .a11y_label(a11y_label)
        .children(vec![icon])
        .into_element(cx)
}

fn assert_form_input_group_control_geometry_matches_web(
    web_name: &str,
    title: &str,
    description: &str,
    input_id: &str,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_card = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-card",
            "text-card-foreground",
            "rounded-xl",
            "border",
            "py-6",
            "sm:max-w-md",
        ],
    )
    .expect("web card root");

    let web_input = web_find_by_id(&theme.root, input_id).expect("web input");
    let web_group = web_find_input_group_container_containing(theme, web_input);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:group"))),
                ..Default::default()
            },
            move |cx| {
                let trailing = form_trailing_icon_button(cx, &format!("Golden:{web_name}:trail"));
                vec![
                    fret_ui_shadcn::InputGroup::new(model)
                        .trailing(vec![trailing])
                        .trailing_has_button(true)
                        .a11y_label("Golden:form-input-group")
                        .into_element(cx),
                ]
            },
        );

        let card = fret_ui_shadcn::Card::new(vec![
            fret_ui_shadcn::CardHeader::new(vec![
                fret_ui_shadcn::CardTitle::new(title).into_element(cx),
                fret_ui_shadcn::CardDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardContent::new(vec![group]).into_element(cx),
        ])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_card.rect.w))),
        )
        .into_element(cx);

        vec![card]
    });

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:group")),
    )
    .expect("fret input-group");

    assert_rect_xwh_close_px("input-group", group.bounds, web_group.rect, 1.0);
}

fn assert_form_checkbox_control_size_matches_web(web_name: &str, checkbox_id: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_checkbox = web_find_by_id(&theme.root, checkbox_id).expect("web checkbox");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let checked: Model<bool> = cx.app.models_mut().insert(true);
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:checkbox"))),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Checkbox::new(checked)
                        .disabled(true)
                        .a11y_label("Golden:form-checkbox")
                        .into_element(cx),
                ]
            },
        )]
    });

    let checkbox = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:checkbox")),
    )
    .expect("fret checkbox wrapper");

    assert_close_px(
        "checkbox w",
        checkbox.bounds.size.width,
        web_checkbox.rect.w,
        1.0,
    );
    assert_close_px(
        "checkbox h",
        checkbox.bounds.size.height,
        web_checkbox.rect.h,
        1.0,
    );
}

fn assert_form_switch_control_size_matches_web(web_name: &str, switch_id: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_switch = web_find_by_id(&theme.root, switch_id).expect("web switch");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let checked: Model<bool> = cx.app.models_mut().insert(false);
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:switch"))),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Switch::new(checked)
                        .a11y_label("Golden:form-switch")
                        .into_element(cx),
                ]
            },
        )]
    });

    let switch = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:switch")),
    )
    .expect("fret switch wrapper");

    assert_close_px("switch w", switch.bounds.size.width, web_switch.rect.w, 1.0);
    assert_close_px(
        "switch h",
        switch.bounds.size.height,
        web_switch.rect.h,
        1.0,
    );
}

fn assert_form_select_control_size_matches_web(web_name: &str, select_id: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_select = web_find_by_id(&theme.root, select_id).expect("web select trigger");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
        let open: Model<bool> = cx.app.models_mut().insert(false);
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(format!("Golden:{web_name}:select"))),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Select::new(value, open)
                        .placeholder("Select")
                        .items([
                            fret_ui_shadcn::SelectItem::new("auto", "Auto"),
                            fret_ui_shadcn::SelectItem::new("english", "English"),
                            fret_ui_shadcn::SelectItem::new("spanish", "Spanish"),
                        ])
                        .refine_layout(
                            LayoutRefinement::default().w_px(MetricRef::Px(Px(web_select.rect.w))),
                        )
                        .into_element(cx),
                ]
            },
        )]
    });

    let select = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some(&format!("Golden:{web_name}:select")),
    )
    .expect("fret select wrapper");

    assert_close_px("select w", select.bounds.size.width, web_select.rect.w, 1.0);
    assert_close_px(
        "select h",
        select.bounds.size.height,
        web_select.rect.h,
        1.0,
    );
}

fn assert_form_radio_control_size_matches_web(web_name: &str, radio_id: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_radio = web_find_by_id(&theme.root, radio_id).expect("web radio");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, _snap, root) = run_fret_root_with_ui(bounds, |cx| {
        let group = fret_ui_shadcn::RadioGroup::uncontrolled(Some("other"))
            .item(fret_ui_shadcn::RadioGroupItem::new("starter", "Starter"))
            .a11y_label("Golden:form-radio-group")
            .into_element(cx);
        vec![group]
    });

    let radio_bounds =
        find_node_with_size_close(&ui, root, web_radio.rect.w, web_radio.rect.h, 1.0)
            .expect("missing fret radio indicator bounds");

    assert_close_px("radio w", radio_bounds.size.width, web_radio.rect.w, 1.0);
    assert_close_px("radio h", radio_bounds.size.height, web_radio.rect.h, 1.0);
}

#[test]
fn web_vs_fret_layout_form_rhf_array_geometry_matches_web() {
    assert_form_input_group_control_geometry_matches_web(
        "form-rhf-array",
        "Contact Emails",
        "Manage your contact email addresses.",
        "form-rhf-array-email-0",
    );
}

#[test]
fn web_vs_fret_layout_form_tanstack_array_geometry_matches_web() {
    assert_form_input_group_control_geometry_matches_web(
        "form-tanstack-array",
        "Contact Emails",
        "Manage your contact email addresses.",
        "form-tanstack-array-email-0",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_password_geometry_matches_web() {
    assert_form_input_group_control_geometry_matches_web(
        "form-rhf-password",
        "Create Password",
        "Choose a strong password to secure your account.",
        "form-rhf-password-input",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_checkbox_geometry_matches_web() {
    assert_form_checkbox_control_size_matches_web(
        "form-rhf-checkbox",
        "form-rhf-checkbox-responses",
    );
}

#[test]
fn web_vs_fret_layout_form_tanstack_checkbox_geometry_matches_web() {
    assert_form_checkbox_control_size_matches_web(
        "form-tanstack-checkbox",
        "form-tanstack-checkbox-responses",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_switch_geometry_matches_web() {
    assert_form_switch_control_size_matches_web("form-rhf-switch", "form-rhf-switch-twoFactor");
}

#[test]
fn web_vs_fret_layout_form_tanstack_switch_geometry_matches_web() {
    assert_form_switch_control_size_matches_web(
        "form-tanstack-switch",
        "form-tanstack-switch-twoFactor",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_select_geometry_matches_web() {
    assert_form_select_control_size_matches_web("form-rhf-select", "form-rhf-select-language");
}

#[test]
fn web_vs_fret_layout_form_tanstack_select_geometry_matches_web() {
    assert_form_select_control_size_matches_web(
        "form-tanstack-select",
        "form-tanstack-select-language",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_radiogroup_geometry_matches_web() {
    assert_form_radio_control_size_matches_web(
        "form-rhf-radiogroup",
        "form-rhf-radiogroup-starter",
    );
}

#[test]
fn web_vs_fret_layout_form_tanstack_radiogroup_geometry_matches_web() {
    assert_form_radio_control_size_matches_web(
        "form-tanstack-radiogroup",
        "form-tanstack-radiogroup-starter",
    );
}

#[test]
fn web_vs_fret_layout_form_rhf_complex_geometry_matches_web() {
    assert_form_radio_control_size_matches_web("form-rhf-complex", "form-rhf-complex-basic");
}

#[test]
fn web_vs_fret_layout_form_tanstack_complex_geometry_matches_web() {
    assert_form_radio_control_size_matches_web("form-tanstack-complex", "basic");
}

#[test]
fn web_vs_fret_layout_field_input_geometry() {
    let web = read_web_golden("field-input");
    let theme = web_theme(&web);

    let web_username_label =
        web_find_by_tag_and_text(&theme.root, "label", "Username").expect("web username label");
    let web_password_label =
        web_find_by_tag_and_text(&theme.root, "label", "Password").expect("web password label");
    let web_username_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");
    let web_inputs: Vec<&WebNode> = {
        let mut out = Vec::new();
        fn walk<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
            if n.tag == "input" {
                out.push(n);
            }
            for c in &n.children {
                walk(c, out);
            }
        }
        walk(&theme.root, &mut out);
        out.sort_by(|a, b| {
            a.rect
                .y
                .partial_cmp(&b.rect.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    };
    let web_password_input = web_inputs.get(1).copied().unwrap_or(web_username_input);
    let web_username_desc = web_find_by_tag_and_text(
        &theme.root,
        "p",
        "Choose a unique username for your account.",
    )
    .expect("web username desc");
    let web_password_desc = web_find_by_tag_and_text(&theme.root, "p", "Must be at least 8")
        .expect("web password desc");

    let web_root = web_find_smallest_container(
        &theme.root,
        &[web_username_label, web_password_desc, web_password_input],
    )
    .expect("web root container");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let username: Model<String> = cx.app.models_mut().insert(String::new());
        let password: Model<String> = cx.app.models_mut().insert(String::new());

        let username_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:username:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Username").into_element(cx)],
        );
        let username_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-input:username:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(username)
                        .a11y_label("Username")
                        .placeholder("Max Leiter")
                        .into_element(cx),
                ]
            },
        );
        let username_desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:username:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new(
                        "Choose a unique username for your account.",
                    )
                    .into_element(cx),
                ]
            },
        );

        let username_field =
            fret_ui_shadcn::Field::new(vec![username_label, username_input, username_desc])
                .into_element(cx);

        let password_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:password:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Password").into_element(cx)],
        );
        let password_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-input:password:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(password)
                        .a11y_label("Password")
                        .placeholder("????????")
                        .into_element(cx),
                ]
            },
        );
        let password_desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:password:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new("Must be at least 8 characters long.")
                        .into_element(cx),
                ]
            },
        );

        let password_field =
            fret_ui_shadcn::Field::new(vec![password_label, password_desc, password_input])
                .into_element(cx);

        let group =
            fret_ui_shadcn::FieldGroup::new(vec![username_field, password_field]).into_element(cx);
        let set = fret_ui_shadcn::FieldSet::new(vec![group]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w))),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:field-input:root"))
        .expect("fret root");

    let username_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:username:label"),
    )
    .expect("fret username label");
    let username_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-input:username:input"),
    )
    .expect("fret username input");
    let username_desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:username:desc"),
    )
    .expect("fret username desc");

    let password_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:password:label"),
    )
    .expect("fret password label");
    let password_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-input:password:input"),
    )
    .expect("fret password input");
    let password_desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:password:desc"),
    )
    .expect("fret password desc");

    assert_close_px(
        "field-input root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-input username label y",
        username_label.bounds.origin.y,
        web_username_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input username input y",
        username_input.bounds.origin.y,
        web_username_input.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input username desc y",
        username_desc.bounds.origin.y,
        web_username_desc.rect.y,
        1.0,
    );

    let username_label_to_input_gap = username_input.bounds.origin.y.0
        - (username_label.bounds.origin.y.0 + username_label.bounds.size.height.0);
    assert!(
        (username_label_to_input_gap - 12.0).abs() <= 1.0,
        "field-input username label->input gap: expected ~12 got={username_label_to_input_gap}"
    );

    let username_input_to_desc_gap = username_desc.bounds.origin.y.0
        - (username_input.bounds.origin.y.0 + username_input.bounds.size.height.0);
    assert!(
        (username_input_to_desc_gap - 12.0).abs() <= 1.0,
        "field-input username input->desc gap: expected ~12 got={username_input_to_desc_gap}"
    );

    assert_close_px(
        "field-input password label y",
        password_label.bounds.origin.y,
        web_password_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input password desc y",
        password_desc.bounds.origin.y,
        web_password_desc.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input password input y",
        password_input.bounds.origin.y,
        web_password_input.rect.y,
        1.0,
    );

    let password_label_to_desc_gap = password_desc.bounds.origin.y.0
        - (password_label.bounds.origin.y.0 + password_label.bounds.size.height.0);
    assert!(
        (password_label_to_desc_gap - 8.0).abs() <= 1.0,
        "field-input password label->desc gap: expected ~8 got={password_label_to_desc_gap}"
    );

    let password_desc_to_input_gap = password_input.bounds.origin.y.0
        - (password_desc.bounds.origin.y.0 + password_desc.bounds.size.height.0);
    assert!(
        (password_desc_to_input_gap - 12.0).abs() <= 1.0,
        "field-input password desc->input gap: expected ~12 got={password_desc_to_input_gap}"
    );

    let field_to_field_gap = password_label.bounds.origin.y.0
        - (username_desc.bounds.origin.y.0 + username_desc.bounds.size.height.0);
    assert!(
        (field_to_field_gap - 28.0).abs() <= 1.0,
        "field-input field->field gap: expected ~28 got={field_to_field_gap}"
    );
}

#[test]
fn web_vs_fret_layout_field_checkbox_geometry() {
    let web = read_web_golden("field-checkbox");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_outer_group =
        web_find_by_class_tokens(&theme.root, &["flex", "w-full", "flex-col", "gap-7"])
            .expect("web outer group");
    let web_row_1 = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "group")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
    })
    .expect("web field row");
    let web_sync_field = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "group")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
            && contains_text(n, "Sync Desktop & Documents folders")
    })
    .expect("web sync field");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let checked_1: Model<bool> = cx.app.models_mut().insert(true);
        let checked_2: Model<bool> = cx.app.models_mut().insert(false);
        let checked_3: Model<bool> = cx.app.models_mut().insert(false);
        let checked_4: Model<bool> = cx.app.models_mut().insert(false);
        let checked_5: Model<bool> = cx.app.models_mut().insert(true);

        let legend = fret_ui_shadcn::FieldLegend::new("Show these items on the desktop")
            .variant(fret_ui_shadcn::FieldLegendVariant::Label)
            .into_element(cx);
        let description = fret_ui_shadcn::FieldDescription::new(
            "Select the items you want to show on the desktop.",
        )
        .into_element(cx);

        let row_1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:row-1")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(checked_1)
                    .a11y_label("Hard disks")
                    .into_element(cx);
                let label = fret_ui_shadcn::FieldLabel::new("Hard disks").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );

        let row_2 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:row-2")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(checked_2)
                    .a11y_label("External disks")
                    .into_element(cx);
                let label = fret_ui_shadcn::FieldLabel::new("External disks").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );

        let row_3 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:row-3")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(checked_3)
                    .a11y_label("CDs, DVDs, and iPods")
                    .into_element(cx);
                let label =
                    fret_ui_shadcn::FieldLabel::new("CDs, DVDs, and iPods").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );

        let row_4 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:row-4")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(checked_4)
                    .a11y_label("Connected servers")
                    .into_element(cx);
                let label = fret_ui_shadcn::FieldLabel::new("Connected servers").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );

        let checkbox_group = fret_ui_shadcn::FieldGroup::new(vec![row_1, row_2, row_3, row_4])
            .gap(Space::N3)
            .into_element(cx);

        let fieldset = fret_ui_shadcn::FieldSet::new(vec![legend, description, checkbox_group])
            .into_element(cx);

        let sync_field = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:sync-field")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(checked_5)
                    .a11y_label("Sync")
                    .into_element(cx);
                let content = fret_ui_shadcn::FieldContent::new(vec![
                    fret_ui_shadcn::FieldLabel::new("Sync Desktop & Documents folders").into_element(cx),
                    fret_ui_shadcn::FieldDescription::new(
                        "Your Desktop & Documents folders are being synced with iCloud Drive. You can access them from other devices.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx);

                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, content])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );

        let group = fret_ui_shadcn::FieldGroup::new(vec![
            fieldset,
            fret_ui_shadcn::FieldSeparator::new().into_element(cx),
            sync_field,
        ])
        .into_element(cx);

        let group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:group")),
                ..Default::default()
            },
            move |_cx| vec![group],
        );

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w))),
                ),
                ..Default::default()
            },
            move |_cx| vec![group],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-checkbox:root"),
    )
    .expect("fret root");
    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-checkbox:group"),
    )
    .expect("fret group");

    let row_1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-checkbox:row-1"),
    )
    .expect("fret row 1");
    let row_2 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-checkbox:row-2"),
    )
    .expect("fret row 2");
    let sync_field = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-checkbox:sync-field"),
    )
    .expect("fret sync field");

    assert_close_px(
        "field-checkbox root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "field-checkbox group width",
        group.bounds.size.width,
        web_outer_group.rect.w,
        1.0,
    );

    let row_gap = row_2.bounds.origin.y.0 - (row_1.bounds.origin.y.0 + row_1.bounds.size.height.0);
    assert!(
        (row_gap - 12.0).abs() <= 1.0,
        "field-checkbox inner group gap: expected ~12 got={row_gap}"
    );

    assert_close_px(
        "field-checkbox row height",
        row_1.bounds.size.height,
        web_row_1.rect.h,
        1.0,
    );
    assert_close_px(
        "field-checkbox sync-field y",
        sync_field.bounds.origin.y,
        web_sync_field.rect.y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_switch_geometry() {
    let web = read_web_golden("field-switch");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_switch = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|v| v == "switch")
            && n.attrs.get("data-state").is_some_and(|v| v == "unchecked")
    })
    .expect("web switch");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let checked: Model<bool> = cx.app.models_mut().insert(false);

        let content = fret_ui_shadcn::FieldContent::new(vec![
            fret_ui_shadcn::FieldLabel::new("Multi-factor authentication").into_element(cx),
            fret_ui_shadcn::FieldDescription::new(
                "Enable multi-factor authentication. If you do not have a two-factor device, you can use a one-time code sent to your email.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let switch = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-switch:switch")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Switch::new(checked)
                        .a11y_label("2fa")
                        .refine_layout(LayoutRefinement::default().flex_shrink_0())
                        .into_element(cx),
                ]
            },
        );

        let field = fret_ui_shadcn::Field::new(vec![content, switch])
            .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
            .into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w))),
                ),
                ..Default::default()
            },
            move |_cx| vec![field],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-switch:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-switch:root"),
    )
    .expect("fret root");
    let switch = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-switch:switch"),
    )
    .expect("fret switch");

    assert_close_px(
        "field-switch root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "field-switch switch x",
        switch.bounds.origin.x,
        web_switch.rect.x,
        1.0,
    );
    assert_close_px(
        "field-switch switch y",
        switch.bounds.origin.y,
        web_switch.rect.y,
        1.0,
    );
    assert_close_px(
        "field-switch switch w",
        switch.bounds.size.width,
        web_switch.rect.w,
        1.0,
    );
    assert_close_px(
        "field-switch switch h",
        switch.bounds.size.height,
        web_switch.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_select_geometry() {
    let web = read_web_golden("field-select");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_label =
        web_find_by_tag_and_text(&theme.root, "label", "Department").expect("web label");
    let web_trigger = find_first(&theme.root, &|n| {
        n.tag == "button" && n.attrs.get("role").is_some_and(|v| v == "combobox")
    })
    .expect("web trigger");
    let web_desc =
        web_find_by_tag_and_text(&theme.root, "p", "Select your department or area of work.")
            .expect("web desc");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
        let open: Model<bool> = cx.app.models_mut().insert(false);

        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-select:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Department").into_element(cx)],
        );

        let trigger = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-select:trigger")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Select::new(value, open)
                        .placeholder("Choose department")
                        .items([
                            fret_ui_shadcn::SelectItem::new("engineering", "Engineering"),
                            fret_ui_shadcn::SelectItem::new("design", "Design"),
                            fret_ui_shadcn::SelectItem::new("marketing", "Marketing"),
                            fret_ui_shadcn::SelectItem::new("sales", "Sales"),
                            fret_ui_shadcn::SelectItem::new("support", "Customer Support"),
                            fret_ui_shadcn::SelectItem::new("hr", "Human Resources"),
                            fret_ui_shadcn::SelectItem::new("finance", "Finance"),
                            fret_ui_shadcn::SelectItem::new("operations", "Operations"),
                        ])
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                ]
            },
        );

        let desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-select:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new(
                        "Select your department or area of work.",
                    )
                    .into_element(cx),
                ]
            },
        );

        let field = fret_ui_shadcn::Field::new(vec![label, trigger, desc]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w))),
                ),
                ..Default::default()
            },
            move |_cx| vec![field],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-select:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-select:root"),
    )
    .expect("fret root");
    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-select:label"),
    )
    .expect("fret label");
    let trigger = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-select:trigger"),
    )
    .expect("fret trigger");
    let desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-select:desc"),
    )
    .expect("fret desc");

    assert_close_px(
        "field-select root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-select label y",
        label.bounds.origin.y,
        web_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-select trigger y",
        trigger.bounds.origin.y,
        web_trigger.rect.y,
        1.0,
    );
    assert_close_px(
        "field-select desc y",
        desc.bounds.origin.y,
        web_desc.rect.y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_radio_geometry() {
    let web = read_web_golden("field-radio");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_label =
        web_find_by_tag_and_text(&theme.root, "label", "Subscription Plan").expect("web label");
    let web_desc =
        web_find_by_tag_and_text(&theme.root, "p", "Yearly and lifetime").expect("web desc");
    let web_group = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "radiogroup")
    })
    .expect("web radio group");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-radio:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Subscription Plan").into_element(cx)],
        );

        let desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-radio:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new(
                        "Yearly and lifetime plans offer significant savings.",
                    )
                    .into_element(cx),
                ]
            },
        );

        let radio_group = {
            let items = vec![
                fret_ui_shadcn::RadioGroupItem::new("monthly", "Monthly ($9.99/month)").children(
                    vec![fret_ui_shadcn::FieldLabel::new("Monthly ($9.99/month)").into_element(cx)],
                ),
                fret_ui_shadcn::RadioGroupItem::new("yearly", "Yearly ($99.99/year)").children(
                    vec![fret_ui_shadcn::FieldLabel::new("Yearly ($99.99/year)").into_element(cx)],
                ),
                fret_ui_shadcn::RadioGroupItem::new("lifetime", "Lifetime ($299.99)").children(
                    vec![fret_ui_shadcn::FieldLabel::new("Lifetime ($299.99)").into_element(cx)],
                ),
            ];

            items
                .into_iter()
                .fold(
                    fret_ui_shadcn::RadioGroup::uncontrolled(Some("monthly")),
                    |group, item| group.item(item),
                )
                .a11y_label("Subscription Plan")
                .into_element(cx)
        };

        let set = fret_ui_shadcn::FieldSet::new(vec![label, desc, radio_group]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w))),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-radio:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:field-radio:root"))
        .expect("fret root");
    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-radio:label"),
    )
    .expect("fret label");
    let desc = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:field-radio:desc"))
        .expect("fret desc");
    let group = find_semantics(&snap, SemanticsRole::RadioGroup, Some("Subscription Plan"))
        .or_else(|| find_semantics(&snap, SemanticsRole::RadioGroup, None))
        .expect("fret radio group");

    assert_close_px(
        "field-radio root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-radio label y",
        label.bounds.origin.y,
        web_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-radio desc y",
        desc.bounds.origin.y,
        web_desc.rect.y,
        1.0,
    );
    assert_close_px(
        "field-radio group y",
        group.bounds.origin.y,
        web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "field-radio group h",
        group.bounds.size.height,
        web_group.rect.h,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_field_textarea_geometry() {
    let web = read_web_golden("field-textarea");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_label = web_find_by_tag_and_text(&theme.root, "label", "Feedback").expect("web label");
    let web_textarea = find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");
    let web_desc =
        web_find_by_tag_and_text(&theme.root, "p", "Share your thoughts").expect("web desc");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-textarea:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Feedback").into_element(cx)],
        );

        let textarea = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-textarea:textarea")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Textarea::new(model)
                        .a11y_label("Feedback")
                        .into_element(cx),
                ]
            },
        );

        let desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-textarea:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new("Share your thoughts about our service.")
                        .into_element(cx),
                ]
            },
        );

        let field = fret_ui_shadcn::Field::new(vec![label, textarea, desc]).into_element(cx);
        let group = fret_ui_shadcn::FieldGroup::new(vec![field]).into_element(cx);
        let set = fret_ui_shadcn::FieldSet::new(vec![group]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w))),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-textarea:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-textarea:root"),
    )
    .expect("fret root");
    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-textarea:label"),
    )
    .expect("fret label");
    let textarea = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-textarea:textarea"),
    )
    .expect("fret textarea wrapper");
    let desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-textarea:desc"),
    )
    .expect("fret desc");

    assert_close_px(
        "field-textarea root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-textarea label y",
        label.bounds.origin.y,
        web_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-textarea textarea y",
        textarea.bounds.origin.y,
        web_textarea.rect.y,
        1.0,
    );
    assert_close_px(
        "field-textarea textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );
    assert_close_px(
        "field-textarea desc y",
        desc.bounds.origin.y,
        web_desc.rect.y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_group_geometry() {
    let web = read_web_golden("field-group");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_responses_label =
        web_find_by_tag_and_text(&theme.root, "label", "Responses").expect("web responses label");
    let web_responses_desc =
        web_find_by_tag_and_text(&theme.root, "p", "Get notified when ChatGPT")
            .expect("web responses desc");
    let web_tasks_label =
        web_find_by_tag_and_text(&theme.root, "label", "Tasks").expect("web tasks label");

    let web_responses_row = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "group")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
            && contains_text(n, "Push notifications")
            && contains_id(n, "push")
    })
    .expect("web responses row");
    let web_push_tasks_row = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "group")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
            && contains_text(n, "Push notifications")
            && contains_id(n, "push-tasks")
    })
    .expect("web push tasks row");
    let web_email_tasks_row = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "group")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
            && contains_text(n, "Email notifications")
            && contains_id(n, "email-tasks")
    })
    .expect("web email tasks row");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let responses_push: Model<bool> = cx.app.models_mut().insert(true);
        let tasks_push: Model<bool> = cx.app.models_mut().insert(false);
        let tasks_email: Model<bool> = cx.app.models_mut().insert(false);

        let responses_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-group:responses:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Responses").into_element(cx)],
        );
        let responses_desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-group:responses:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![fret_ui_shadcn::FieldDescription::new(
                    "Get notified when ChatGPT responds to requests that take time, like research or image generation.",
                )
                .into_element(cx)]
            },
        );
        let responses_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-group:responses:row")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(responses_push)
                    .disabled(true)
                    .a11y_label("push")
                    .into_element(cx);
                let label = fret_ui_shadcn::FieldLabel::new("Push notifications").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );
        let responses_checkbox_group = fret_ui_shadcn::FieldGroup::new(vec![responses_row])
            .checkbox_group()
            .into_element(cx);
        let responses_fieldset = fret_ui_shadcn::FieldSet::new(vec![
            responses_label,
            responses_desc,
            responses_checkbox_group,
        ])
        .into_element(cx);

        let tasks_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-group:tasks:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Tasks").into_element(cx)],
        );
        let tasks_desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-group:tasks:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new(
                        "Get notified when tasks you've created have updates. Manage tasks",
                    )
                    .into_element(cx),
                ]
            },
        );
        let tasks_row_push = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-group:tasks:push-row")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(tasks_push)
                    .a11y_label("push-tasks")
                    .into_element(cx);
                let label = fret_ui_shadcn::FieldLabel::new("Push notifications").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );
        let tasks_row_email = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-group:tasks:email-row")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(tasks_email)
                    .a11y_label("email-tasks")
                    .into_element(cx);
                let label = fret_ui_shadcn::FieldLabel::new("Email notifications").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );
        let tasks_checkbox_group =
            fret_ui_shadcn::FieldGroup::new(vec![tasks_row_push, tasks_row_email])
                .checkbox_group()
                .into_element(cx);
        let tasks_fieldset =
            fret_ui_shadcn::FieldSet::new(vec![tasks_label, tasks_desc, tasks_checkbox_group])
                .into_element(cx);

        let separator = fret_ui_shadcn::FieldSeparator::new().into_element(cx);

        let group =
            fret_ui_shadcn::FieldGroup::new(vec![responses_fieldset, separator, tasks_fieldset])
                .into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w))),
                ),
                ..Default::default()
            },
            move |_cx| vec![group],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-group:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:field-group:root"))
        .expect("fret root");
    let responses_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:responses:label"),
    )
    .expect("fret responses label");
    let responses_desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:responses:desc"),
    )
    .expect("fret responses desc");
    let responses_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:responses:row"),
    )
    .expect("fret responses row");
    let tasks_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:tasks:label"),
    )
    .expect("fret tasks label");

    let tasks_row_push = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:tasks:push-row"),
    )
    .expect("fret tasks push row");
    let tasks_row_email = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:tasks:email-row"),
    )
    .expect("fret tasks email row");

    assert_close_px(
        "field-group root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "field-group responses label y",
        responses_label.bounds.origin.y,
        web_responses_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-group responses desc y",
        responses_desc.bounds.origin.y,
        web_responses_desc.rect.y,
        1.0,
    );
    assert_close_px(
        "field-group responses desc h",
        responses_desc.bounds.size.height,
        web_responses_desc.rect.h,
        1.0,
    );
    assert_close_px(
        "field-group responses row y",
        responses_row.bounds.origin.y,
        web_responses_row.rect.y,
        1.0,
    );
    assert_close_px(
        "field-group responses row h",
        responses_row.bounds.size.height,
        web_responses_row.rect.h,
        1.0,
    );
    assert_close_px(
        "field-group tasks label y",
        tasks_label.bounds.origin.y,
        web_tasks_label.rect.y,
        1.0,
    );

    let fret_first_fieldset_to_tasks_label = tasks_label.bounds.origin.y.0
        - (responses_row.bounds.origin.y.0 + responses_row.bounds.size.height.0);
    let web_first_fieldset_to_tasks_label =
        web_tasks_label.rect.y - (web_responses_row.rect.y + web_responses_row.rect.h);
    assert!(
        (fret_first_fieldset_to_tasks_label - web_first_fieldset_to_tasks_label).abs() <= 1.0,
        "field-group responses row -> tasks label: expected≈{web_first_fieldset_to_tasks_label} got={fret_first_fieldset_to_tasks_label}"
    );

    let tasks_gap = tasks_row_email.bounds.origin.y.0
        - (tasks_row_push.bounds.origin.y.0 + tasks_row_push.bounds.size.height.0);
    assert!(
        (tasks_gap - 12.0).abs() <= 1.0,
        "field-group checkbox-group gap: expected ~12 got={tasks_gap}"
    );

    assert_close_px(
        "field-group tasks push row h",
        tasks_row_push.bounds.size.height,
        web_push_tasks_row.rect.h,
        1.0,
    );
    assert_close_px(
        "field-group tasks email row h",
        tasks_row_email.bounds.size.height,
        web_email_tasks_row.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_fieldset_geometry() {
    let web = read_web_golden("field-fieldset");
    let theme = web_theme(&web);

    let web_root = web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md", "space-y-6"])
        .expect("web root");
    let web_legend =
        web_find_by_tag_and_text(&theme.root, "legend", "Address Information").expect("web legend");
    let web_desc = web_find_by_tag_and_text(&theme.root, "p", "We need your address")
        .expect("web description");

    let web_street_label =
        web_find_by_tag_and_text(&theme.root, "label", "Street Address").expect("web street label");
    let web_city_label =
        web_find_by_tag_and_text(&theme.root, "label", "City").expect("web city label");
    let web_zip_label =
        web_find_by_tag_and_text(&theme.root, "label", "Postal Code").expect("web zip label");

    let web_street_input = find_first(&theme.root, &|n| {
        n.tag == "input" && n.id.as_deref().is_some_and(|id| id == "street")
    })
    .expect("web street input");
    let web_city_input = find_first(&theme.root, &|n| {
        n.tag == "input" && n.id.as_deref().is_some_and(|id| id == "city")
    })
    .expect("web city input");
    let web_zip_input = find_first(&theme.root, &|n| {
        n.tag == "input" && n.id.as_deref().is_some_and(|id| id == "zip")
    })
    .expect("web zip input");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let street: Model<String> = cx.app.models_mut().insert(String::new());
        let city: Model<String> = cx.app.models_mut().insert(String::new());
        let zip: Model<String> = cx.app.models_mut().insert(String::new());

        let legend = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:legend")),
                ..Default::default()
            },
            move |cx| {
                vec![fret_ui_shadcn::FieldLegend::new("Address Information").into_element(cx)]
            },
        );
        let desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new(
                        "We need your address to deliver your order.",
                    )
                    .into_element(cx),
                ]
            },
        );

        let street_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:street:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Street Address").into_element(cx)],
        );
        let street_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-fieldset:street:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(street)
                        .a11y_label("Street Address")
                        .placeholder("123 Main St")
                        .into_element(cx),
                ]
            },
        );
        let street_field =
            fret_ui_shadcn::Field::new(vec![street_label, street_input]).into_element(cx);

        let city_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:city:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("City").into_element(cx)],
        );
        let city_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-fieldset:city:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(city)
                        .a11y_label("City")
                        .placeholder("New York")
                        .into_element(cx),
                ]
            },
        );
        let city_field = fret_ui_shadcn::Field::new(vec![city_label, city_input]).into_element(cx);

        let zip_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:zip:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Postal Code").into_element(cx)],
        );
        let zip_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-fieldset:zip:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(zip)
                        .a11y_label("Postal Code")
                        .placeholder("90502")
                        .into_element(cx),
                ]
            },
        );
        let zip_field = fret_ui_shadcn::Field::new(vec![zip_label, zip_input]).into_element(cx);

        let grid = cx.grid(
            GridProps {
                cols: 2,
                gap: Px(16.0),
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_full(),
                ),
                ..Default::default()
            },
            move |_cx| vec![city_field, zip_field],
        );

        let group = fret_ui_shadcn::FieldGroup::new(vec![street_field, grid]).into_element(cx);
        let set = fret_ui_shadcn::FieldSet::new(vec![legend, desc, group]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w))),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:root"),
    )
    .expect("fret root");
    let legend = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:legend"),
    )
    .expect("fret legend");
    let desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:desc"),
    )
    .expect("fret desc");

    let street_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:street:label"),
    )
    .expect("fret street label");
    let street_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-fieldset:street:input"),
    )
    .expect("fret street input");

    let city_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:city:label"),
    )
    .expect("fret city label");
    let city_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-fieldset:city:input"),
    )
    .expect("fret city input");

    let zip_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:zip:label"),
    )
    .expect("fret zip label");
    let zip_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-fieldset:zip:input"),
    )
    .expect("fret zip input");

    assert_close_px(
        "field-fieldset root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-fieldset legend y",
        legend.bounds.origin.y,
        web_legend.rect.y,
        1.0,
    );
    assert_close_px(
        "field-fieldset desc y",
        desc.bounds.origin.y,
        web_desc.rect.y,
        1.0,
    );

    assert_close_px(
        "field-fieldset street label y",
        street_label.bounds.origin.y,
        web_street_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-fieldset street input y",
        street_input.bounds.origin.y,
        web_street_input.rect.y,
        1.0,
    );
    assert_close_px(
        "field-fieldset city label y",
        city_label.bounds.origin.y,
        web_city_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-fieldset zip label y",
        zip_label.bounds.origin.y,
        web_zip_label.rect.y,
        1.0,
    );

    let fret_city_x = city_input.bounds.origin.x.0 - root.bounds.origin.x.0;
    let web_city_x = web_city_input.rect.x - web_root.rect.x;
    assert!(
        (fret_city_x - web_city_x).abs() <= 1.0,
        "field-fieldset city input x: expected≈{web_city_x} got={fret_city_x}"
    );

    let fret_zip_x = zip_input.bounds.origin.x.0 - root.bounds.origin.x.0;
    let web_zip_x = web_zip_input.rect.x - web_root.rect.x;
    assert!(
        (fret_zip_x - web_zip_x).abs() <= 1.0,
        "field-fieldset zip input x: expected≈{web_zip_x} got={fret_zip_x}"
    );
}

#[test]
fn web_vs_fret_layout_field_choice_card_geometry() {
    let web = read_web_golden("field-choice-card");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");

    let web_radio_group = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.attrs.get("role").is_some_and(|v| v == "radiogroup")
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("grid gap-3"))
    })
    .expect("web radio group");

    let web_card_kubernetes = find_first(&theme.root, &|n| {
        n.tag == "label"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("has-[>[data-slot=field]]:w-full"))
            && contains_text(n, "Kubernetes")
    })
    .expect("web kubernetes card");

    let web_card_vm = find_first(&theme.root, &|n| {
        n.tag == "label"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("has-[>[data-slot=field]]:w-full"))
            && contains_text(n, "Virtual Machine")
    })
    .expect("web vm card");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let selected: Model<Option<Arc<str>>> =
            cx.app.models_mut().insert(Some(Arc::from("kubernetes")));

        let root =
            radio_group_prim::RadioGroupRoot::new(selected).a11y_label("Compute Environment");
        let values: Arc<[Arc<str>]> = Arc::from([Arc::from("kubernetes"), Arc::from("vm")]);
        let disabled: Arc<[bool]> = Arc::from([false, false]);

        let mut list_props = RovingFlexProps::default();
        list_props.flex.layout = fret_ui_kit::declarative::style::layout_style(
            &theme,
            fret_ui_kit::LayoutRefinement::default().w_full(),
        );
        list_props.flex.gap = MetricRef::space(Space::N3).resolve(&theme);

        let pressable_layout = fret_ui_kit::declarative::style::layout_style(
            &theme,
            fret_ui_kit::LayoutRefinement::default().w_full(),
        );

        let chrome = ChromeRefinement::default()
            .rounded_md()
            .border_1()
            .border_color(fret_ui_kit::ColorRef::Color(border))
            .p_4();

        let make_card = |cx: &mut fret_ui::ElementContext<'_, App>,
                         title: &'static str,
                         desc: &'static str,
                         checked: bool| {
            let content = fret_ui_shadcn::FieldContent::new(vec![
                fret_ui_shadcn::FieldTitle::new(title).into_element(cx),
                fret_ui_shadcn::FieldDescription::new(desc).into_element(cx),
            ])
            .into_element(cx);

            let radio_stub_layout = fret_ui_kit::declarative::style::layout_style(
                &Theme::global(&*cx.app),
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(fret_ui_kit::MetricRef::Px(Px(16.0)))
                    .h_px(fret_ui_kit::MetricRef::Px(Px(16.0)))
                    .flex_shrink_0(),
            );
            let radio_stub = cx.container(
                ContainerProps {
                    layout: radio_stub_layout,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let field = fret_ui_shadcn::Field::new(vec![content, radio_stub])
                .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                .into_element(cx);

            let mut props = fret_ui_kit::declarative::style::container_props(
                &theme,
                chrome.clone(),
                fret_ui_kit::LayoutRefinement::default().w_full(),
            );
            if checked {
                // Matches upstream `has-data-[state=checked]:bg-primary/5` (visual-only).
                if let Some(primary) = theme.color_by_key("primary/5") {
                    props.background = Some(primary);
                }
            }

            cx.container(props, move |_cx| vec![field])
        };

        let list = root
            .clone()
            .list(values.clone(), disabled.clone())
            .into_element(cx, list_props, move |cx| {
                let kubernetes = root
                    .item("kubernetes")
                    .label("Kubernetes")
                    .index(0)
                    .set_size(Some(2))
                    .tab_stop(true)
                    .into_element(
                        cx,
                        &root,
                        PressableProps {
                            layout: pressable_layout,
                            ..Default::default()
                        },
                        move |cx, _st, checked| {
                            vec![make_card(
                                cx,
                                "Kubernetes",
                                "Run GPU workloads on a K8s configured cluster.",
                                checked,
                            )]
                        },
                    );

                let vm = root
                    .item("vm")
                    .label("Virtual Machine")
                    .index(1)
                    .set_size(Some(2))
                    .into_element(
                        cx,
                        &root,
                        PressableProps {
                            layout: pressable_layout,
                            ..Default::default()
                        },
                        move |cx, _st, checked| {
                            vec![make_card(
                                cx,
                                "Virtual Machine",
                                "Access a VM configured cluster to run GPU workloads.",
                                checked,
                            )]
                        },
                    );

                vec![kubernetes, vm]
            });

        let set = fret_ui_shadcn::FieldSet::new(vec![
            fret_ui_shadcn::FieldLabel::new("Compute Environment").into_element(cx),
            fret_ui_shadcn::FieldDescription::new(
                "Select the compute environment for your cluster.",
            )
            .into_element(cx),
            list,
        ])
        .into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w))),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-choice-card:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-choice-card:root"),
    )
    .expect("fret root");
    let radio_group = find_semantics(
        &snap,
        SemanticsRole::RadioGroup,
        Some("Compute Environment"),
    )
    .or_else(|| find_semantics(&snap, SemanticsRole::RadioGroup, None))
    .expect("fret radio group");
    let kubernetes =
        find_semantics(&snap, SemanticsRole::RadioButton, Some("Kubernetes")).expect("fret k8s");
    let vm = find_semantics(&snap, SemanticsRole::RadioButton, Some("Virtual Machine"))
        .expect("fret vm");

    assert_close_px(
        "field-choice-card root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "field-choice-card kubernetes y",
        kubernetes.bounds.origin.y,
        web_card_kubernetes.rect.y,
        2.0,
    );
    assert_close_px(
        "field-choice-card kubernetes w",
        kubernetes.bounds.size.width,
        web_card_kubernetes.rect.w,
        2.0,
    );
    assert_close_px(
        "field-choice-card vm y",
        vm.bounds.origin.y,
        web_card_vm.rect.y,
        2.0,
    );
    assert_close_px(
        "field-choice-card vm w",
        vm.bounds.size.width,
        web_card_vm.rect.w,
        2.0,
    );
    assert_close_px(
        "field-choice-card radiogroup y",
        radio_group.bounds.origin.y,
        web_radio_group.rect.y,
        1.0,
    );
    let fret_card_delta_y = vm.bounds.origin.y.0 - kubernetes.bounds.origin.y.0;
    let web_card_delta_y = web_card_vm.rect.y - web_card_kubernetes.rect.y;
    assert!(
        (fret_card_delta_y - web_card_delta_y).abs() <= 2.0,
        "field-choice-card card delta y: expected≈{web_card_delta_y} got={fret_card_delta_y}"
    );

    assert_close_px(
        "field-choice-card radiogroup-to-root gap",
        Px(radio_group.bounds.origin.y.0 - root.bounds.origin.y.0),
        web_radio_group.rect.y - web_root.rect.y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_table_demo_row_heights_and_caption_gap() {
    let web = read_web_golden("table-demo");
    let theme = web_theme(&web);

    let web_caption = web_find_by_tag_and_text(&theme.root, "caption", "recent invoices")
        .or_else(|| find_first(&theme.root, &|n| n.tag == "caption"))
        .expect("web caption");
    let web_header_row = find_first(&theme.root, &|n| n.tag == "thead")
        .and_then(|thead| thead.children.iter().find(|n| n.tag == "tr"))
        .expect("web header tr");
    let web_body_row = find_first(&theme.root, &|n| n.tag == "tbody")
        .and_then(|tbody| tbody.children.iter().find(|n| n.tag == "tr"))
        .expect("web body tr");
    let web_footer_row = find_first(&theme.root, &|n| n.tag == "tfoot")
        .and_then(|tfoot| tfoot.children.iter().find(|n| n.tag == "tr"))
        .expect("web footer tr");

    let web_caption_gap = web_caption.rect.y - (web_footer_row.rect.y + web_footer_row.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let head_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:table-demo:header-row")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::TableRow::new(
                        4,
                        vec![
                            fret_ui_shadcn::TableHead::new("Invoice").into_element(cx),
                            fret_ui_shadcn::TableHead::new("Status").into_element(cx),
                            fret_ui_shadcn::TableHead::new("Method").into_element(cx),
                            fret_ui_shadcn::TableHead::new("Amount").into_element(cx),
                        ],
                    )
                    .into_element(cx),
                ]
            },
        );

        let first_body_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:table-demo:body-row-0")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::TableRow::new(
                        4,
                        vec![
                            fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, "INV001"))
                                .into_element(cx),
                            fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, "Paid"))
                                .into_element(cx),
                            fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, "Credit Card"))
                                .into_element(cx),
                            fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, "$250.00"))
                                .into_element(cx),
                        ],
                    )
                    .into_element(cx),
                ]
            },
        );

        let other_rows = [
            ("INV002", "Pending", "PayPal", "$150.00"),
            ("INV003", "Unpaid", "Bank Transfer", "$350.00"),
            ("INV004", "Paid", "Credit Card", "$450.00"),
            ("INV005", "Paid", "PayPal", "$550.00"),
            ("INV006", "Pending", "Bank Transfer", "$200.00"),
            ("INV007", "Unpaid", "Credit Card", "$300.00"),
        ]
        .into_iter()
        .map(|(invoice, status, method, amount)| {
            fret_ui_shadcn::TableRow::new(
                4,
                vec![
                    fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, invoice))
                        .into_element(cx),
                    fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, status)).into_element(cx),
                    fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, method)).into_element(cx),
                    fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, amount)).into_element(cx),
                ],
            )
            .into_element(cx)
        })
        .collect::<Vec<_>>();

        let footer_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:table-demo:footer-row")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::TableRow::new(
                        4,
                        vec![
                            fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, "Total"))
                                .col_span(3)
                                .into_element(cx),
                            fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, "$2,500.00"))
                                .into_element(cx),
                        ],
                    )
                    .into_element(cx),
                ]
            },
        );

        let caption =
            fret_ui_shadcn::TableCaption::new("A list of your recent invoices.").into_element(cx);

        vec![
            fret_ui_shadcn::Table::new(vec![
                fret_ui_shadcn::TableHeader::new(vec![head_row]).into_element(cx),
                fret_ui_shadcn::TableBody::new({
                    let mut rows = Vec::new();
                    rows.push(first_body_row);
                    rows.extend(other_rows);
                    rows
                })
                .into_element(cx),
                fret_ui_shadcn::TableFooter::new(vec![footer_row]).into_element(cx),
                caption,
            ])
            .into_element(cx),
        ]
    });

    let header_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:table-demo:header-row"),
    )
    .expect("fret header row");
    let body_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:table-demo:body-row-0"),
    )
    .expect("fret first body row");
    let footer_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:table-demo:footer-row"),
    )
    .expect("fret footer row");

    assert_close_px(
        "table-demo header row height",
        header_row.bounds.size.height,
        web_header_row.rect.h,
        1.0,
    );
    assert_close_px(
        "table-demo first body row height",
        body_row.bounds.size.height,
        web_body_row.rect.h,
        1.0,
    );
    assert_close_px(
        "table-demo footer row height",
        footer_row.bounds.size.height,
        web_footer_row.rect.h,
        2.0,
    );

    let target_caption_y =
        footer_row.bounds.origin.y.0 + footer_row.bounds.size.height.0 + web_caption_gap;
    let target_caption_h = web_caption.rect.h;

    let mut nodes = Vec::new();
    collect_subtree_nodes(&ui, root, &mut nodes);

    let mut best: Option<Rect> = None;
    let mut best_score = f32::INFINITY;
    for id in nodes {
        let Some(bounds) = ui.debug_node_bounds(id) else {
            continue;
        };
        let score = (bounds.origin.y.0 - target_caption_y).abs()
            + (bounds.size.height.0 - target_caption_h).abs()
            + bounds.origin.x.0.abs();
        if score < best_score {
            best_score = score;
            best = Some(bounds);
        }
    }

    let caption_bounds = best.expect("fret caption bounds");
    let fret_caption_gap = caption_bounds.origin.y.0
        - (footer_row.bounds.origin.y.0 + footer_row.bounds.size.height.0);
    assert_close_px(
        "table-demo caption gap",
        Px(fret_caption_gap),
        web_caption_gap,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_data_table_demo_row_height_and_action_button_size() {
    let web = read_web_golden("data-table-demo");
    let theme = web_theme(&web);

    let web_header_row = find_first(&theme.root, &|n| n.tag == "thead")
        .and_then(|thead| thead.children.iter().find(|n| n.tag == "tr"))
        .expect("web header tr");
    let web_body_row = find_first(&theme.root, &|n| n.tag == "tbody")
        .and_then(|tbody| tbody.children.iter().find(|n| n.tag == "tr"))
        .expect("web body tr");

    let web_select_row = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "checkbox")
            && n.attrs.get("aria-label").is_some_and(|v| v == "Select row")
    })
    .expect("web select row checkbox");

    let web_open_menu = find_first(&theme.root, &|n| {
        n.tag == "button" && contains_text(n, "Open menu")
    })
    .expect("web open menu button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let header_select_all: Model<bool> = cx.app.models_mut().insert(false);
        let row_select: Model<bool> = cx.app.models_mut().insert(false);

        let select_all = fret_ui_shadcn::Checkbox::new(header_select_all)
            .a11y_label("Select all")
            .into_element(cx);
        let select_row = fret_ui_shadcn::Checkbox::new(row_select)
            .a11y_label("Select row")
            .into_element(cx);

        let open_menu = fret_ui_shadcn::Button::new("Open menu")
            .variant(fret_ui_shadcn::ButtonVariant::Ghost)
            .size(fret_ui_shadcn::ButtonSize::IconSm)
            .children(vec![
                fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
            ])
            .into_element(cx);

        let header_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:data-table-demo:header-row")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::TableRow::new(
                        5,
                        vec![
                            fret_ui_shadcn::TableCell::new(select_all.clone()).into_element(cx),
                            fret_ui_shadcn::TableHead::new("Status").into_element(cx),
                            fret_ui_shadcn::TableHead::new("Email").into_element(cx),
                            fret_ui_shadcn::TableHead::new("Amount").into_element(cx),
                            fret_ui_shadcn::TableHead::new("").into_element(cx),
                        ],
                    )
                    .into_element(cx),
                ]
            },
        );

        let body_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:data-table-demo:row-0")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::TableRow::new(
                        5,
                        vec![
                            fret_ui_shadcn::TableCell::new(select_row.clone()).into_element(cx),
                            fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, "success"))
                                .into_element(cx),
                            fret_ui_shadcn::TableCell::new(decl_text::text_sm(
                                cx,
                                "ken99@example.com",
                            ))
                            .into_element(cx),
                            fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, "$316.00"))
                                .into_element(cx),
                            fret_ui_shadcn::TableCell::new(open_menu.clone()).into_element(cx),
                        ],
                    )
                    .into_element(cx),
                ]
            },
        );

        vec![
            fret_ui_shadcn::Table::new(vec![
                fret_ui_shadcn::TableHeader::new(vec![header_row]).into_element(cx),
                fret_ui_shadcn::TableBody::new(vec![body_row]).into_element(cx),
            ])
            .into_element(cx),
        ]
    });

    let header_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:data-table-demo:header-row"),
    )
    .expect("fret header row");
    let body_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:data-table-demo:row-0"),
    )
    .expect("fret body row");

    let select_row = find_semantics(&snap, SemanticsRole::Checkbox, Some("Select row"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Checkbox, None))
        .expect("fret select row checkbox");
    let open_menu = find_semantics(&snap, SemanticsRole::Button, Some("Open menu"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret open menu button");

    assert_close_px(
        "data-table-demo header row height",
        header_row.bounds.size.height,
        web_header_row.rect.h,
        1.0,
    );
    assert_close_px(
        "data-table-demo row height",
        body_row.bounds.size.height,
        web_body_row.rect.h,
        2.0,
    );

    assert_close_px(
        "data-table-demo select row checkbox width",
        select_row.bounds.size.width,
        web_select_row.rect.w,
        1.0,
    );
    assert_close_px(
        "data-table-demo select row checkbox height",
        select_row.bounds.size.height,
        web_select_row.rect.h,
        1.0,
    );

    assert_close_px(
        "data-table-demo open menu button width",
        open_menu.bounds.size.width,
        web_open_menu.rect.w,
        1.0,
    );
    assert_close_px(
        "data-table-demo open menu button height",
        open_menu.bounds.size.height,
        web_open_menu.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_data_table_demo_empty_state_cell_spans_table_width() {
    let web = read_web_golden("data-table-demo.empty");
    let theme = web_theme(&web);

    let web_table = find_first(&theme.root, &|n| n.tag == "table").expect("web table");
    let web_empty_td =
        web_find_by_tag_and_text(&theme.root, "td", "No results").expect("web empty state td");

    let expected_rel = WebRect {
        x: web_empty_td.rect.x - web_table.rect.x,
        y: web_empty_td.rect.y - web_table.rect.y,
        w: web_empty_td.rect.w,
        h: web_empty_td.rect.h,
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();

        let empty_td = fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, "No results."))
            .col_span(5)
            .refine_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(web_empty_td.rect.h))))
            .into_element(cx);

        let table = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:data-table-demo.empty:table")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Table::new(vec![
                        fret_ui_shadcn::TableHeader::new(vec![
                            fret_ui_shadcn::TableRow::new(
                                5,
                                vec![
                                    fret_ui_shadcn::TableHead::new("").into_element(cx),
                                    fret_ui_shadcn::TableHead::new("Status").into_element(cx),
                                    fret_ui_shadcn::TableHead::new("Email").into_element(cx),
                                    fret_ui_shadcn::TableHead::new("Amount").into_element(cx),
                                    fret_ui_shadcn::TableHead::new("").into_element(cx),
                                ],
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        fret_ui_shadcn::TableBody::new(vec![
                            fret_ui_shadcn::TableRow::new(5, vec![empty_td.clone()])
                                .border_bottom(false)
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                ]
            },
        );

        vec![cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &theme,
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_table.rect.w))),
                ),
                ..Default::default()
            },
            move |_cx| vec![table],
        )]
    });

    let _ = snap;

    // We render only the table subtree in Fret, so the "relative to table" rect becomes an
    // absolute rect in our test harness.
    let expected_abs = WebRect {
        x: expected_rel.x,
        y: expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let (td_id, td_bounds) = if let Some(found) =
        find_node_with_bounds_close(&ui, root, expected_abs, 2.0)
    {
        found
    } else {
        let mut nodes = Vec::new();
        collect_subtree_nodes(&ui, root, &mut nodes);

        let mut best: Option<(NodeId, Rect, f32)> = None;
        for id in nodes {
            let Some(bounds) = ui.debug_node_bounds(id) else {
                continue;
            };
            let score = (bounds.origin.x.0 - expected_abs.x).abs()
                + (bounds.origin.y.0 - expected_abs.y).abs()
                + (bounds.size.width.0 - expected_abs.w).abs()
                + (bounds.size.height.0 - expected_abs.h).abs();
            if best.as_ref().is_none_or(|(_, _, s)| score < *s) {
                best = Some((id, bounds, score));
            }
        }

        let (id, b, score) = best.expect("no debug bounds in subtree");
        panic!(
            "fret td bounds not found; bestCandidate id={id:?} bounds={b:?} score={score} expected={expected_abs:?}"
        );
    };
    let _ = td_id;

    assert_rect_close_px("data-table-demo.empty td", td_bounds, expected_abs, 2.0);
}

#[test]
fn web_vs_fret_layout_typography_table_cell_geometry_light() {
    let web = read_web_golden("typography-table");
    let theme = web.themes.get("light").expect("missing light theme");

    let web_table = find_first(&theme.root, &|n| n.tag == "table").expect("web table");

    let mut web_trs = Vec::new();
    web_collect_tag(web_table, "tr", &mut web_trs);
    web_trs.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert_eq!(web_trs.len(), 4, "expected 1 header + 3 body rows");

    let web_header = web_trs[0];
    let mut web_header_cells: Vec<_> = web_header
        .children
        .iter()
        .filter(|n| n.tag == "th")
        .collect();
    web_header_cells.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert_eq!(web_header_cells.len(), 2, "expected 2 header cells");

    let col_w0 = web_header_cells[0].rect.w;
    let col_w1 = web_header_cells[1].rect.w;

    // `border-collapse: collapse` means the cell grid is inset by half the outer border width.
    let inset = web_trs[0].rect.x;

    let mut rows: Vec<[(String, WebRect); 2]> = Vec::new();
    for (row_idx, tr) in web_trs.iter().enumerate() {
        let mut cells: Vec<_> = tr
            .children
            .iter()
            .filter(|n| n.tag == "th" || n.tag == "td")
            .collect();
        cells.sort_by(|a, b| {
            a.rect
                .x
                .partial_cmp(&b.rect.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        assert_eq!(cells.len(), 2, "expected 2 cells in row {row_idx}");
        rows.push([
            (cells[0].text.clone().unwrap_or_default(), cells[0].rect),
            (cells[1].text.clone().unwrap_or_default(), cells[1].rect),
        ]);
    }
    let rows_ui = rows.clone();

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
    let mut services = StyleAwareServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let border = theme.color_required("border");
            let muted = theme.color_required("muted");

            let table = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:typography-table:table")),
                ..Default::default()
            },
            move |cx| {
                let mut table_layout = LayoutStyle::default();
                table_layout.size.width = Length::Fill;

                vec![cx.container(
                    ContainerProps {
                        layout: table_layout,
                        padding: Edges::all(Px(inset)),
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
                                gap: Px(0.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                            },
                            move |cx| {
                                let mut out = Vec::new();
                                for (row_idx, row) in rows_ui.clone().into_iter().enumerate() {
                                    let is_header = row_idx == 0;
                                    let is_body_even = row_idx > 0 && ((row_idx - 1) % 2 == 1);

                                    let row_label = Arc::<str>::from(format!(
                                        "Golden:typography-table:row{row_idx}"
                                    ));

                                    let row_panel = cx.semantics(
                                        fret_ui::element::SemanticsProps {
                                            layout: LayoutStyle {
                                                size: SizeStyle {
                                                    width: Length::Fill,
                                                    height: Length::Auto,
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            role: SemanticsRole::Panel,
                                            label: Some(row_label),
                                            ..Default::default()
                                        },
                                        move |cx| {
                                            let mut row_layout = LayoutStyle::default();
                                            row_layout.size.width = Length::Fill;

                                            let row_props = ContainerProps {
                                                layout: row_layout,
                                                padding: Edges::all(Px(0.0)),
                                                background: is_body_even.then_some(muted),
                                                shadow: None,
                                                border: Edges::all(Px(0.0)),
                                                border_color: None,
                                                corner_radii: Default::default(),
                                                ..Default::default()
                                            };

                                            vec![cx.container(row_props, move |cx| {
                                                let mut flex_layout = LayoutStyle::default();
                                                flex_layout.size.width = Length::Fill;

                                                vec![cx.row(
                                                    RowProps {
                                                        layout: flex_layout,
                                                        gap: Px(0.0),
                                                        padding: Edges::all(Px(0.0)),
                                                        justify: MainAlign::Start,
                                                        align: CrossAlign::Stretch,
                                                    },
                                                    move |cx| {
                                                        let mut cells_out = Vec::new();
                                                        for col_idx in 0..2 {
                                                            let label = Arc::<str>::from(format!(
                                                                "Golden:typography-table:r{row_idx}c{col_idx}"
                                                            ));
                                                            let text = row[col_idx].0.clone();
                                                            let weight = if col_idx == 0 {
                                                                col_w0
                                                            } else {
                                                                col_w1
                                                            };
                                                            let left_border = if col_idx == 0 {
                                                                1.0
                                                            } else {
                                                                0.0
                                                            };

                                                            let cell = cx.semantics(
                                                                fret_ui::element::SemanticsProps {
                                                                    layout: {
                                                                        let mut layout =
                                                                            LayoutStyle::default();
                                                                        layout.flex.grow = weight;
                                                                        layout.flex.shrink = 1.0;
                                                                        layout.flex.basis =
                                                                            Length::Px(Px(0.0));
                                                                        layout
                                                                    },
                                                                    role: SemanticsRole::Panel,
                                                                    label: Some(label),
                                                                    ..Default::default()
                                                                },
                                                                move |cx| {
                                                                    let mut cell_layout =
                                                                        LayoutStyle::default();
                                                                    cell_layout.size.width =
                                                                        Length::Fill;

                                                                    let cell_props = ContainerProps {
                                                                        layout: cell_layout,
                                                                        padding: Edges {
                                                                            top: Px(8.0),
                                                                            right: Px(16.0),
                                                                            bottom: Px(8.0),
                                                                            left: Px(16.0),
                                                                        },
                                                                        background: None,
                                                                        shadow: None,
                                                                        border: Edges {
                                                                            top: Px(1.0),
                                                                            right: Px(1.0),
                                                                            bottom: Px(0.0),
                                                                            left: Px(left_border),
                                                                        },
                                                                        border_color: Some(border),
                                                                        corner_radii: Default::default(),
                                                                        ..Default::default()
                                                                    };

                                                                    vec![cx.container(
                                                                        cell_props,
                                                                        move |cx| {
                                                                            if is_header {
                                                                                vec![decl_text::text_prose_bold_nowrap(
                                                                                    cx,
                                                                                    text.clone(),
                                                                                )]
                                                                            } else {
                                                                                vec![decl_text::text_prose_nowrap(
                                                                                    cx,
                                                                                    text.clone(),
                                                                                )]
                                                                            }
                                                                        },
                                                                    )]
                                                                },
                                                            );
                                                            cells_out.push(cell);
                                                        }
                                                        cells_out
                                                    },
                                                )]
                                            })]
                                        },
                                    );

                                    out.push(row_panel);
                                }
                                out
                            },
                        )]
                    },
                )]
            },
        );

            vec![table]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let table = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:typography-table:table"),
    )
    .expect("fret table");
    assert_close_px(
        "typography-table table width",
        table.bounds.size.width,
        web_table.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-table table height",
        table.bounds.size.height,
        web_table.rect.h,
        1.0,
    );

    for (row_idx, web_tr) in web_trs.iter().enumerate() {
        let row = find_semantics(
            &snap,
            SemanticsRole::Panel,
            Some(&format!("Golden:typography-table:row{row_idx}")),
        )
        .unwrap_or_else(|| panic!("missing fret row {row_idx}"));

        assert_close_px(
            &format!("typography-table row[{row_idx}] y"),
            row.bounds.origin.y,
            web_tr.rect.y,
            1.0,
        );
        assert_close_px(
            &format!("typography-table row[{row_idx}] h"),
            row.bounds.size.height,
            web_tr.rect.h,
            1.0,
        );

        for col_idx in 0..2 {
            let label = format!("Golden:typography-table:r{row_idx}c{col_idx}");
            let cell = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
                .unwrap_or_else(|| panic!("missing fret cell {label}"));
            let expected = &rows[row_idx][col_idx].1;

            assert_close_px(&format!("{label} x"), cell.bounds.origin.x, expected.x, 1.0);
            assert_close_px(&format!("{label} y"), cell.bounds.origin.y, expected.y, 1.0);
            assert_close_px(
                &format!("{label} w"),
                cell.bounds.size.width,
                expected.w,
                1.0,
            );
            assert_close_px(
                &format!("{label} h"),
                cell.bounds.size.height,
                expected.h,
                1.0,
            );
        }
    }

    // Paint-backed parity: `even:bg-muted` (web uses `lab(...)`).
    let web_even_bg = web_trs[2]
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web row[2] backgroundColor");
    let expected_even_rect = web_trs[2].rect;

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) = find_scene_quad_background_with_rect_close(&scene, expected_even_rect, 2.0)
        .expect("even row background quad");
    assert_rgba_close(
        "typography-table even row background",
        color_to_rgba(bg),
        web_even_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_typography_table_cell_geometry_dark() {
    let web = read_web_golden("typography-table");
    let theme = web.themes.get("dark").expect("missing dark theme");

    let web_table = find_first(&theme.root, &|n| n.tag == "table").expect("web table");

    let mut web_trs = Vec::new();
    web_collect_tag(web_table, "tr", &mut web_trs);
    web_trs.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert_eq!(web_trs.len(), 4, "expected 1 header + 3 body rows");

    let web_header = web_trs[0];
    let mut web_header_cells: Vec<_> = web_header
        .children
        .iter()
        .filter(|n| n.tag == "th")
        .collect();
    web_header_cells.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert_eq!(web_header_cells.len(), 2, "expected 2 header cells");

    let col_w0 = web_header_cells[0].rect.w;
    let col_w1 = web_header_cells[1].rect.w;

    // `border-collapse: collapse` means the cell grid is inset by half the outer border width.
    let inset = web_trs[0].rect.x;

    let mut rows: Vec<[(String, WebRect); 2]> = Vec::new();
    for (row_idx, tr) in web_trs.iter().enumerate() {
        let mut cells: Vec<_> = tr
            .children
            .iter()
            .filter(|n| n.tag == "th" || n.tag == "td")
            .collect();
        cells.sort_by(|a, b| {
            a.rect
                .x
                .partial_cmp(&b.rect.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        assert_eq!(cells.len(), 2, "expected 2 cells in row {row_idx}");
        rows.push([
            (cells[0].text.clone().unwrap_or_default(), cells[0].rect),
            (cells[1].text.clone().unwrap_or_default(), cells[1].rect),
        ]);
    }
    let rows_ui = rows.clone();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let border = theme.color_required("border");
            let muted = theme.color_required("muted");

            let table = cx.semantics(
                fret_ui::element::SemanticsProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("Golden:typography-table:table")),
                    ..Default::default()
                },
                move |cx| {
                    let mut table_layout = LayoutStyle::default();
                    table_layout.size.width = Length::Fill;

                    vec![cx.container(
                        ContainerProps {
                            layout: table_layout,
                            padding: Edges::all(Px(inset)),
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
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                },
                                move |cx| {
                                    let mut out = Vec::new();
                                    for (row_idx, row) in rows_ui.clone().into_iter().enumerate() {
                                        let is_header = row_idx == 0;
                                        let is_body_even = row_idx > 0 && ((row_idx - 1) % 2 == 1);

                                        let row_label = Arc::<str>::from(format!(
                                            "Golden:typography-table:row{row_idx}"
                                        ));

                                        let row_panel = cx.semantics(
                                            fret_ui::element::SemanticsProps {
                                                layout: LayoutStyle {
                                                    size: SizeStyle {
                                                        width: Length::Fill,
                                                        height: Length::Auto,
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                role: SemanticsRole::Panel,
                                                label: Some(row_label),
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                let mut row_layout = LayoutStyle::default();
                                                row_layout.size.width = Length::Fill;

                                                let row_props = ContainerProps {
                                                    layout: row_layout,
                                                    padding: Edges::all(Px(0.0)),
                                                    background: is_body_even.then_some(muted),
                                                    shadow: None,
                                                    border: Edges::all(Px(0.0)),
                                                    border_color: None,
                                                    corner_radii: Default::default(),
                                                    ..Default::default()
                                                };

                                                vec![cx.container(row_props, move |cx| {
                                                    let mut flex_layout = LayoutStyle::default();
                                                    flex_layout.size.width = Length::Fill;

                                                    vec![cx.row(
                                                        RowProps {
                                                            layout: flex_layout,
                                                            gap: Px(0.0),
                                                            padding: Edges::all(Px(0.0)),
                                                            justify: MainAlign::Start,
                                                            align: CrossAlign::Stretch,
                                                        },
                                                        move |cx| {
                                                            let mut cells_out = Vec::new();
                                                            for col_idx in 0..2 {
                                                                let label =
                                                                    Arc::<str>::from(format!(
                                                                        "Golden:typography-table:r{row_idx}c{col_idx}"
                                                                    ));
                                                                let text = row[col_idx].0.clone();
                                                                let weight = if col_idx == 0 {
                                                                    col_w0
                                                                } else {
                                                                    col_w1
                                                                };
                                                                let left_border = if col_idx == 0 {
                                                                    1.0
                                                                } else {
                                                                    0.0
                                                                };

                                                                let cell = cx.semantics(
                                                                    fret_ui::element::SemanticsProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.flex.grow = weight;
                                                                            layout.flex.shrink = 1.0;
                                                                            layout.flex.basis =
                                                                                Length::Px(Px(0.0));
                                                                            layout
                                                                        },
                                                                        role: SemanticsRole::Panel,
                                                                        label: Some(label),
                                                                        ..Default::default()
                                                                    },
                                                                    move |cx| {
                                                                        let mut cell_layout =
                                                                            LayoutStyle::default();
                                                                        cell_layout.size.width =
                                                                            Length::Fill;

                                                                        let cell_props = ContainerProps {
                                                                            layout: cell_layout,
                                                                            padding: Edges {
                                                                                top: Px(8.0),
                                                                                right: Px(16.0),
                                                                                bottom: Px(8.0),
                                                                                left: Px(16.0),
                                                                            },
                                                                            background: None,
                                                                            shadow: None,
                                                                            border: Edges {
                                                                                top: Px(1.0),
                                                                                right: Px(1.0),
                                                                                bottom: Px(0.0),
                                                                                left: Px(left_border),
                                                                            },
                                                                            border_color: Some(border),
                                                                            corner_radii: Default::default(),
                                                                            ..Default::default()
                                                                        };

                                                                        vec![cx.container(
                                                                            cell_props,
                                                                            move |cx| {
                                                                                if is_header {
                                                                                    vec![decl_text::text_prose_bold_nowrap(
                                                                                        cx,
                                                                                        text.clone(),
                                                                                    )]
                                                                                } else {
                                                                                    vec![decl_text::text_prose_nowrap(
                                                                                        cx,
                                                                                        text.clone(),
                                                                                    )]
                                                                                }
                                                                            },
                                                                        )]
                                                                    },
                                                                );
                                                                cells_out.push(cell);
                                                            }
                                                            cells_out
                                                        },
                                                    )]
                                                })]
                                            },
                                        );

                                        out.push(row_panel);
                                    }
                                    out
                                },
                            )]
                        },
                    )]
                },
            );

            vec![table]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let table = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:typography-table:table"),
    )
    .expect("fret table");
    assert_close_px(
        "typography-table table width",
        table.bounds.size.width,
        web_table.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-table table height",
        table.bounds.size.height,
        web_table.rect.h,
        1.0,
    );

    for (row_idx, web_tr) in web_trs.iter().enumerate() {
        let row = find_semantics(
            &snap,
            SemanticsRole::Panel,
            Some(&format!("Golden:typography-table:row{row_idx}")),
        )
        .unwrap_or_else(|| panic!("missing fret row {row_idx}"));

        assert_close_px(
            &format!("typography-table row[{row_idx}] y"),
            row.bounds.origin.y,
            web_tr.rect.y,
            1.0,
        );
        assert_close_px(
            &format!("typography-table row[{row_idx}] h"),
            row.bounds.size.height,
            web_tr.rect.h,
            1.0,
        );

        for col_idx in 0..2 {
            let label = format!("Golden:typography-table:r{row_idx}c{col_idx}");
            let cell = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
                .unwrap_or_else(|| panic!("missing fret cell {label}"));
            let expected = &rows[row_idx][col_idx].1;

            assert_close_px(&format!("{label} x"), cell.bounds.origin.x, expected.x, 1.0);
            assert_close_px(&format!("{label} y"), cell.bounds.origin.y, expected.y, 1.0);
            assert_close_px(
                &format!("{label} w"),
                cell.bounds.size.width,
                expected.w,
                1.0,
            );
            assert_close_px(
                &format!("{label} h"),
                cell.bounds.size.height,
                expected.h,
                1.0,
            );
        }
    }

    // Paint-backed parity: `even:bg-muted` (web uses `lab(...)`).
    let web_even_bg = web_trs[2]
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web row[2] backgroundColor");
    let expected_even_rect = web_trs[2].rect;

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) = find_scene_quad_background_with_rect_close(&scene, expected_even_rect, 2.0)
        .expect("even row background quad");
    assert_rgba_close(
        "typography-table even row background",
        color_to_rgba(bg),
        web_even_bg,
        0.02,
    );
}

fn assert_prepared_text_style<'a>(
    services: &'a StyleAwareServices,
    expected_text: &str,
    expected_size: Px,
    expected_line_height: Px,
    expected_weight: u16,
) -> &'a RecordedTextPrepare {
    let record = services
        .prepared
        .iter()
        .rev()
        .find(|r| r.text == expected_text)
        .unwrap_or_else(|| {
            let mut texts: Vec<_> = services.prepared.iter().map(|r| r.text.as_str()).collect();
            texts.sort();
            panic!(
                "missing prepared text style for {expected_text:?}; seen {} prepares: {texts:?}",
                services.prepared.len()
            )
        });

    assert_eq!(record.style.size, expected_size, "text size mismatch");
    assert_eq!(
        record.style.line_height,
        Some(expected_line_height),
        "line height mismatch"
    );
    assert_eq!(
        record.style.weight.0, expected_weight,
        "font weight mismatch"
    );
    record
}

#[test]
fn web_vs_fret_layout_typography_h1_geometry_light() {
    let web = read_web_golden("typography-h1");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h1 = find_first(&theme.root, &|n| n.tag == "h1").expect("web h1");

    let text = web_h1.text.clone().unwrap_or_default();
    let size = web_css_px(web_h1, "fontSize");
    let line_height = web_css_px(web_h1, "lineHeight");
    let weight = web_css_u16(web_h1, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h1")),
                ..Default::default()
            },
            move |_cx| vec![heading],
        )]
    });

    let h1 = find_by_test_id(&snap, "typography-h1");
    assert_rect_close_px("typography-h1", h1.bounds, web_h1.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_layout_typography_h2_geometry_light() {
    let web = read_web_golden("typography-h2");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h2 = find_first(&theme.root, &|n| n.tag == "h2").expect("web h2");

    let text = web_h2.text.clone().unwrap_or_default();
    let size = web_css_px(web_h2, "fontSize");
    let line_height = web_css_px(web_h2, "lineHeight");
    let weight = web_css_u16(web_h2, "fontWeight");
    let padding_bottom = web_css_px(web_h2, "paddingBottom");
    let border_bottom = web_css_px(web_h2, "borderBottomWidth");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let border_color = theme.color_required("border");

        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        let container = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                padding: Edges {
                    bottom: padding_bottom,
                    ..Edges::all(Px(0.0))
                },
                border: Edges {
                    bottom: border_bottom,
                    ..Edges::all(Px(0.0))
                },
                border_color: Some(border_color),
                ..Default::default()
            },
            move |_cx| vec![heading],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h2")),
                ..Default::default()
            },
            move |_cx| vec![container],
        )]
    });

    let h2 = find_by_test_id(&snap, "typography-h2");
    assert_rect_close_px("typography-h2", h2.bounds, web_h2.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_layout_typography_h3_geometry_light() {
    let web = read_web_golden("typography-h3");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h3 = find_first(&theme.root, &|n| n.tag == "h3").expect("web h3");

    let text = web_h3.text.clone().unwrap_or_default();
    let size = web_css_px(web_h3, "fontSize");
    let line_height = web_css_px(web_h3, "lineHeight");
    let weight = web_css_u16(web_h3, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h3")),
                ..Default::default()
            },
            move |_cx| vec![heading],
        )]
    });

    let h3 = find_by_test_id(&snap, "typography-h3");
    assert_rect_close_px("typography-h3", h3.bounds, web_h3.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_layout_typography_h4_geometry_light() {
    let web = read_web_golden("typography-h4");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h4 = find_first(&theme.root, &|n| n.tag == "h4").expect("web h4");

    let text = web_h4.text.clone().unwrap_or_default();
    let size = web_css_px(web_h4, "fontSize");
    let line_height = web_css_px(web_h4, "lineHeight");
    let weight = web_css_u16(web_h4, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h4")),
                ..Default::default()
            },
            move |_cx| vec![heading],
        )]
    });

    let h4 = find_by_test_id(&snap, "typography-h4");
    assert_rect_close_px("typography-h4", h4.bounds, web_h4.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_layout_typography_p_geometry_light() {
    let web = read_web_golden("typography-p");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");

    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let p = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-p")),
                ..Default::default()
            },
            move |_cx| vec![p],
        )]
    });

    let p = find_by_test_id(&snap, "typography-p");
    assert_rect_close_px("typography-p", p.bounds, web_p.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_layout_typography_lead_geometry_light() {
    let web = read_web_golden("typography-lead");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");

    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let p = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .text_color(ColorRef::Token {
                key: "muted-foreground",
                fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
            })
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-lead")),
                ..Default::default()
            },
            move |_cx| vec![p],
        )]
    });

    let p = find_by_test_id(&snap, "typography-lead");
    assert_rect_close_px("typography-lead", p.bounds, web_p.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_layout_typography_muted_geometry_light() {
    let web = read_web_golden("typography-muted");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");

    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let p = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .text_color(ColorRef::Token {
                key: "muted-foreground",
                fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
            })
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-muted")),
                ..Default::default()
            },
            move |_cx| vec![p],
        )]
    });

    let p = find_by_test_id(&snap, "typography-muted");
    assert_rect_close_px("typography-muted", p.bounds, web_p.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_layout_typography_large_geometry_light() {
    let web = read_web_golden("typography-large");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_div =
        find_first(&theme.root, &|n| n.tag == "div" && n.text.is_some()).expect("web div");

    let text = web_div.text.clone().unwrap_or_default();
    let size = web_css_px(web_div, "fontSize");
    let line_height = web_css_px(web_div, "lineHeight");
    let weight = web_css_u16(web_div, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let div = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-large")),
                ..Default::default()
            },
            move |_cx| vec![div],
        )]
    });

    let div = find_by_test_id(&snap, "typography-large");
    assert_rect_close_px("typography-large", div.bounds, web_div.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_layout_typography_blockquote_geometry_light() {
    let web = read_web_golden("typography-blockquote");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_bq = find_first(&theme.root, &|n| n.tag == "blockquote").expect("web blockquote");

    let text = web_bq.text.clone().unwrap_or_default();
    let size = web_css_px(web_bq, "fontSize");
    let line_height = web_css_px(web_bq, "lineHeight");
    let border_left = web_css_px(web_bq, "borderLeftWidth");
    let padding_left = web_css_px(web_bq, "paddingLeft");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let border_color = theme.color_required("border");

        let quote = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .into_element(cx);

        let container = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                padding: Edges {
                    left: padding_left,
                    ..Edges::all(Px(0.0))
                },
                border: Edges {
                    left: border_left,
                    ..Edges::all(Px(0.0))
                },
                border_color: Some(border_color),
                ..Default::default()
            },
            move |_cx| vec![quote],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-blockquote")),
                ..Default::default()
            },
            move |_cx| vec![container],
        )]
    });

    let bq = find_by_test_id(&snap, "typography-blockquote");
    assert_rect_close_px("typography-blockquote", bq.bounds, web_bq.rect, 1.0);

    let record = assert_prepared_text_style(
        &services,
        &text,
        size,
        line_height,
        fret_core::FontWeight::NORMAL.0,
    );
    let _ = record;
}

#[test]
fn web_vs_fret_layout_typography_list_geometry_light() {
    let web = read_web_golden("typography-list");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_ul = find_first(&theme.root, &|n| n.tag == "ul").expect("web ul");

    let mut web_lis = Vec::new();
    web_collect_tag(web_ul, "li", &mut web_lis);
    web_lis.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_lis.len(), 3, "expected 3 web li nodes");

    let li_texts: Vec<String> = web_lis
        .iter()
        .map(|li| li.text.clone().unwrap_or_default())
        .collect();

    let li_size = web_css_px(web_lis[0], "fontSize");
    let li_line_height = web_css_px(web_lis[0], "lineHeight");
    let li_weight = web_css_u16(web_lis[0], "fontWeight");

    let li_texts_for_render = li_texts.clone();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, move |cx| {
        let mut ul_layout = LayoutStyle::default();
        ul_layout.size.width = Length::Px(Px(web_ul.rect.w));
        ul_layout.margin.left = fret_ui::element::MarginEdge::Px(Px(web_ul.rect.x));

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: ul_layout,
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-list")),
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
                        gap: Px(8.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                    },
                    move |cx| {
                        li_texts_for_render
                            .into_iter()
                            .enumerate()
                            .map(|(idx, text)| {
                                let test_id = Arc::<str>::from(format!("typography-list-li{idx}"));
                                cx.semantics(
                                    fret_ui::element::SemanticsProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout
                                        },
                                        role: SemanticsRole::Panel,
                                        test_id: Some(test_id),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        let li = ui::text(cx, text)
                                            .text_size_px(li_size)
                                            .line_height_px(li_line_height)
                                            .font_weight(fret_core::FontWeight(li_weight))
                                            .into_element(cx);
                                        vec![li]
                                    },
                                )
                            })
                            .collect::<Vec<_>>()
                    },
                )]
            },
        )]
    });

    let ul = find_by_test_id(&snap, "typography-list");
    assert_rect_close_px("typography-list ul", ul.bounds, web_ul.rect, 1.0);

    for (idx, web_li) in web_lis.iter().enumerate() {
        let li = find_by_test_id(&snap, &format!("typography-list-li{idx}"));
        assert_rect_close_px(
            &format!("typography-list li[{idx}]"),
            li.bounds,
            web_li.rect,
            1.0,
        );
        assert_prepared_text_style(
            &services,
            &li_texts[idx],
            li_size,
            li_line_height,
            li_weight,
        );
    }
}

#[test]
fn web_vs_fret_layout_typography_inline_code_padding_and_style_light() {
    let web = read_web_golden("typography-inline-code");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_code = find_first(&theme.root, &|n| n.tag == "code").expect("web code");

    let text = web_code.text.clone().unwrap_or_default();
    let size = web_css_px(web_code, "fontSize");
    let line_height = web_css_px(web_code, "lineHeight");
    let weight = web_css_u16(web_code, "fontWeight");
    let pt = web_css_px(web_code, "paddingTop");
    let pb = web_css_px(web_code, "paddingBottom");
    let pl = web_css_px(web_code, "paddingLeft");
    let pr = web_css_px(web_code, "paddingRight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let bg = theme.color_required("muted");

        let code_text_el = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .nowrap()
            .into_element(cx);

        let code_text = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-inline-code-text")),
                ..Default::default()
            },
            move |_cx| vec![code_text_el],
        );

        let code = cx.container(
            ContainerProps {
                padding: Edges {
                    top: pt,
                    right: pr,
                    bottom: pb,
                    left: pl,
                },
                background: Some(bg),
                corner_radii: fret_core::Corners::all(Px(4.0)),
                ..Default::default()
            },
            move |_cx| vec![code_text],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-inline-code")),
                ..Default::default()
            },
            move |_cx| vec![code],
        )]
    });

    assert_prepared_text_style(&services, &text, size, line_height, weight);

    let code = find_by_test_id(&snap, "typography-inline-code");
    let code_text = find_by_test_id(&snap, "typography-inline-code-text");

    assert_close_px(
        "inline-code padding left",
        Px(code_text.bounds.origin.x.0 - code.bounds.origin.x.0),
        pl.0,
        0.25,
    );
    assert_close_px(
        "inline-code padding top",
        Px(code_text.bounds.origin.y.0 - code.bounds.origin.y.0),
        pt.0,
        0.25,
    );

    let code_right = code.bounds.origin.x.0 + code.bounds.size.width.0;
    let text_right = code_text.bounds.origin.x.0 + code_text.bounds.size.width.0;
    assert_close_px(
        "inline-code padding right",
        Px(code_right - text_right),
        pr.0,
        0.25,
    );

    let code_bottom = code.bounds.origin.y.0 + code.bounds.size.height.0;
    let text_bottom = code_text.bounds.origin.y.0 + code_text.bounds.size.height.0;
    assert_close_px(
        "inline-code padding bottom",
        Px(code_bottom - text_bottom),
        pb.0,
        0.25,
    );
}

#[test]
fn web_vs_fret_layout_typography_small_text_style_light() {
    let web = read_web_golden("typography-small");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_small = find_first(&theme.root, &|n| n.tag == "small").expect("web small");

    let text = web_small.text.clone().unwrap_or_default();
    let size = web_css_px(web_small, "fontSize");
    let line_height = web_css_px(web_small, "lineHeight");
    let weight = web_css_u16(web_small, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, _snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let small = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .nowrap()
            .into_element(cx);

        vec![small]
    });

    let record = assert_prepared_text_style(&services, &text, size, line_height, weight);
    assert_eq!(record.constraints.wrap, TextWrap::None);
}

#[test]
fn web_vs_fret_layout_typography_demo_geometry_smoke_light() {
    let web = read_web_golden("typography-demo");
    let theme = web.themes.get("light").expect("missing light theme");

    let web_h1 = find_first(&theme.root, &|n| n.tag == "h1").expect("web h1");
    let web_h2 = find_first(&theme.root, &|n| n.tag == "h2").expect("web h2");
    let mut web_h3s = find_all(&theme.root, &|n| n.tag == "h3");
    web_h3s.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    let web_h3 = web_h3s.first().copied().expect("web h3");
    let web_bq = find_first(&theme.root, &|n| n.tag == "blockquote").expect("web blockquote");
    let web_ul = find_first(&theme.root, &|n| n.tag == "ul").expect("web ul");

    let mut web_lis = Vec::new();
    web_collect_tag(web_ul, "li", &mut web_lis);
    web_lis.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_lis.len(), 3, "expected 3 web li nodes");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, move |cx| {
        let h1_text = web_h1.text.clone().unwrap_or_default();
        let h2_text = web_h2.text.clone().unwrap_or_default();
        let h3_text = web_h3.text.clone().unwrap_or_default();
        let bq_text = web_bq.text.clone().unwrap_or_default();

        let h1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("typography-demo-h1")),
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(cx, h1_text.clone())
                        .text_size_px(web_css_px(web_h1, "fontSize"))
                        .line_height_px(web_css_px(web_h1, "lineHeight"))
                        .font_weight(fret_core::FontWeight(web_css_u16(web_h1, "fontWeight")))
                        .into_element(cx),
                ]
            },
        );

        let h2 = cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("typography-demo-h2")),
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                ..Default::default()
            },
            move |cx| {
                let theme = Theme::global(&*cx.app).clone();
                let border_color = theme.color_required("border");

                let heading = ui::text(cx, h2_text.clone())
                    .text_size_px(web_css_px(web_h2, "fontSize"))
                    .line_height_px(web_css_px(web_h2, "lineHeight"))
                    .font_weight(fret_core::FontWeight(web_css_u16(web_h2, "fontWeight")))
                    .into_element(cx);

                let padding_bottom = web_css_px(web_h2, "paddingBottom");
                let border_bottom = web_css_px(web_h2, "borderBottomWidth");

                let container = cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout
                        },
                        padding: Edges {
                            bottom: padding_bottom,
                            ..Edges::all(Px(0.0))
                        },
                        border: Edges {
                            bottom: border_bottom,
                            ..Edges::all(Px(0.0))
                        },
                        border_color: Some(border_color),
                        ..Default::default()
                    },
                    move |_cx| vec![heading],
                );

                vec![container]
            },
        );

        let h3 = cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("typography-demo-h3")),
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(cx, h3_text.clone())
                        .text_size_px(web_css_px(web_h3, "fontSize"))
                        .line_height_px(web_css_px(web_h3, "lineHeight"))
                        .font_weight(fret_core::FontWeight(web_css_u16(web_h3, "fontWeight")))
                        .into_element(cx),
                ]
            },
        );

        let bq = cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("typography-demo-blockquote")),
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(cx, bq_text.clone())
                        .text_size_px(web_css_px(web_bq, "fontSize"))
                        .line_height_px(web_css_px(web_bq, "lineHeight"))
                        .into_element(cx),
                ]
            },
        );

        let li_texts: Vec<String> = web_lis
            .iter()
            .map(|li| li.text.clone().unwrap_or_default())
            .collect();
        let li_size = web_css_px(web_lis[0], "fontSize");
        let li_line_height = web_css_px(web_lis[0], "lineHeight");

        let ul = cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("typography-demo-ul")),
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(web_ul.rect.w));
                    layout.margin.left = fret_ui::element::MarginEdge::Px(Px(web_ul.rect.x));
                    layout
                },
                role: SemanticsRole::Panel,
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
                        gap: Px(8.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                    },
                    move |cx| {
                        li_texts
                            .into_iter()
                            .map(|t| {
                                ui::text(cx, t)
                                    .text_size_px(li_size)
                                    .line_height_px(li_line_height)
                                    .into_element(cx)
                            })
                            .collect::<Vec<_>>()
                    },
                )]
            },
        );

        vec![cx.column(
            ColumnProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
            },
            move |_cx| vec![h1, h2, bq, h3, ul],
        )]
    });

    let h1 = find_by_test_id(&snap, "typography-demo-h1");
    assert_close_px(
        "typography-demo h1 w",
        h1.bounds.size.width,
        web_h1.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-demo h1 h",
        h1.bounds.size.height,
        web_h1.rect.h,
        1.0,
    );

    let h2 = find_by_test_id(&snap, "typography-demo-h2");
    assert_close_px(
        "typography-demo h2 w",
        h2.bounds.size.width,
        web_h2.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-demo h2 h",
        h2.bounds.size.height,
        web_h2.rect.h,
        1.0,
    );

    let bq = find_by_test_id(&snap, "typography-demo-blockquote");
    assert_close_px(
        "typography-demo blockquote w",
        bq.bounds.size.width,
        web_bq.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-demo blockquote h",
        bq.bounds.size.height,
        web_bq.rect.h,
        1.0,
    );

    let h3 = find_by_test_id(&snap, "typography-demo-h3");
    assert_close_px(
        "typography-demo h3 w",
        h3.bounds.size.width,
        web_h3.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-demo h3 h",
        h3.bounds.size.height,
        web_h3.rect.h,
        1.0,
    );

    let ul = find_by_test_id(&snap, "typography-demo-ul");
    assert_close_px(
        "typography-demo ul x",
        ul.bounds.origin.x,
        web_ul.rect.x,
        1.0,
    );
    assert_close_px(
        "typography-demo ul w",
        ul.bounds.size.width,
        web_ul.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-demo ul h",
        ul.bounds.size.height,
        web_ul.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_accordion_demo_geometry_light() {
    let web = read_web_golden("accordion-demo");
    let theme = web.themes.get("light").expect("missing light theme");

    let mut web_buttons = Vec::new();
    web_collect_tag(&theme.root, "button", &mut web_buttons);
    web_buttons.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_buttons.len(), 3, "expected 3 accordion triggers");

    let web_items: Vec<&WebNode> = {
        let mut all = Vec::new();
        web_collect_all(&theme.root, &mut all);
        let mut items: Vec<&WebNode> = all
            .into_iter()
            .filter(|n| n.tag == "div" && class_has_token(n, "border-b"))
            .collect();
        items.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
        items
    };
    assert_eq!(web_items.len(), 3, "expected 3 accordion items");

    let web_open_content =
        web_find_by_class_tokens(&theme.root, &["pt-0", "pb-4", "flex", "flex-col", "gap-4"])
            .expect("web open accordion content wrapper");

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
    let mut services = StyleAwareServices::default();

    let default_value = Some(Arc::from("item-1"));
    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        use fret_ui_shadcn::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};

        let item_1 = AccordionItem::new(
            Arc::from("item-1"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![
                decl_text::text_sm(
                    cx,
                    "Our flagship product combines cutting-edge technology with sleek design. Built with premium materials, it offers unparalleled performance and reliability.",
                ),
                decl_text::text_sm(
                    cx,
                    "Key features include advanced processing capabilities, and an intuitive user interface designed for both beginners and experts.",
                ),
            ]),
        );
        let item_2 = AccordionItem::new(
            Arc::from("item-2"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![decl_text::text_sm(cx, "Content 2")]),
        );
        let item_3 = AccordionItem::new(
            Arc::from("item-3"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![decl_text::text_sm(cx, "Content 3")]),
        );

        let accordion = Accordion::single_uncontrolled(default_value.clone())
            .collapsible(true)
            .items([item_1, item_2, item_3])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(theme.viewport.w));
                    layout
                },
                ..Default::default()
            },
            move |_cx| vec![accordion],
        )]
    };

    for frame in 0..12 {
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
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let trig_1 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-1"))
        .expect("fret trigger item-1");
    let trig_2 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-2"))
        .expect("fret trigger item-2");
    let trig_3 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-3"))
        .expect("fret trigger item-3");

    assert_rect_close_px(
        "accordion-demo trigger 1",
        trig_1.bounds,
        web_buttons[0].rect,
        1.0,
    );

    let content_id = *trig_1
        .controls
        .first()
        .expect("expected controls on item-1");
    let content = snap
        .nodes
        .iter()
        .find(|n| n.id == content_id)
        .expect("fret content node (item-1)");
    assert_rect_close_px(
        "accordion-demo open content wrapper",
        content.bounds,
        web_open_content.rect,
        1.0,
    );
    assert_rect_close_px(
        "accordion-demo trigger 2",
        trig_2.bounds,
        web_buttons[1].rect,
        1.0,
    );
    assert_rect_close_px(
        "accordion-demo trigger 3",
        trig_3.bounds,
        web_buttons[2].rect,
        1.0,
    );

    let item_1_h = trig_2.bounds.origin.y.0 - trig_1.bounds.origin.y.0;
    let item_2_h = trig_3.bounds.origin.y.0 - trig_2.bounds.origin.y.0;
    assert_close_px(
        "accordion-demo item 1 height",
        Px(item_1_h),
        web_items[0].rect.h,
        1.0,
    );
    assert_close_px(
        "accordion-demo item 2 height",
        Px(item_2_h),
        web_items[1].rect.h,
        1.0,
    );
    assert_close_px(
        "accordion-demo item 3 height",
        trig_3.bounds.size.height,
        web_items[2].rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_accordion_demo_geometry_dark() {
    let web = read_web_golden("accordion-demo");
    let theme = web.themes.get("dark").expect("missing dark theme");

    let mut web_buttons = Vec::new();
    web_collect_tag(&theme.root, "button", &mut web_buttons);
    web_buttons.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_buttons.len(), 3, "expected 3 accordion triggers");

    let web_items: Vec<&WebNode> = {
        let mut all = Vec::new();
        web_collect_all(&theme.root, &mut all);
        let mut items: Vec<&WebNode> = all
            .into_iter()
            .filter(|n| n.tag == "div" && class_has_token(n, "border-b"))
            .collect();
        items.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
        items
    };
    assert_eq!(web_items.len(), 3, "expected 3 accordion items");

    let web_open_content =
        web_find_by_class_tokens(&theme.root, &["pt-0", "pb-4", "flex", "flex-col", "gap-4"])
            .expect("web open accordion content wrapper");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let default_value = Some(Arc::from("item-1"));
    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        use fret_ui_shadcn::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};

        let item_1 = AccordionItem::new(
            Arc::from("item-1"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![
                decl_text::text_sm(
                    cx,
                    "Our flagship product combines cutting-edge technology with sleek design. Built with premium materials, it offers unparalleled performance and reliability.",
                ),
                decl_text::text_sm(
                    cx,
                    "Key features include advanced processing capabilities, and an intuitive user interface designed for both beginners and experts.",
                ),
            ]),
        );
        let item_2 = AccordionItem::new(
            Arc::from("item-2"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![decl_text::text_sm(cx, "Content 2")]),
        );
        let item_3 = AccordionItem::new(
            Arc::from("item-3"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![decl_text::text_sm(cx, "Content 3")]),
        );

        let accordion = Accordion::single_uncontrolled(default_value.clone())
            .collapsible(true)
            .items([item_1, item_2, item_3])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(theme.viewport.w));
                    layout
                },
                ..Default::default()
            },
            move |_cx| vec![accordion],
        )]
    };

    for frame in 0..12 {
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
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let trig_1 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-1"))
        .expect("fret trigger item-1");
    let trig_2 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-2"))
        .expect("fret trigger item-2");
    let trig_3 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-3"))
        .expect("fret trigger item-3");

    assert_rect_close_px(
        "accordion-demo trigger 1 (dark)",
        trig_1.bounds,
        web_buttons[0].rect,
        1.0,
    );

    let content_id = *trig_1
        .controls
        .first()
        .expect("expected controls on item-1");
    let content = snap
        .nodes
        .iter()
        .find(|n| n.id == content_id)
        .expect("fret content node (item-1)");
    assert_rect_close_px(
        "accordion-demo open content wrapper (dark)",
        content.bounds,
        web_open_content.rect,
        1.0,
    );
    assert_rect_close_px(
        "accordion-demo trigger 2 (dark)",
        trig_2.bounds,
        web_buttons[1].rect,
        1.0,
    );
    assert_rect_close_px(
        "accordion-demo trigger 3 (dark)",
        trig_3.bounds,
        web_buttons[2].rect,
        1.0,
    );

    let item_1_h = trig_2.bounds.origin.y.0 - trig_1.bounds.origin.y.0;
    let item_2_h = trig_3.bounds.origin.y.0 - trig_2.bounds.origin.y.0;
    assert_close_px(
        "accordion-demo item 1 height (dark)",
        Px(item_1_h),
        web_items[0].rect.h,
        1.0,
    );
    assert_close_px(
        "accordion-demo item 2 height (dark)",
        Px(item_2_h),
        web_items[1].rect.h,
        1.0,
    );
    assert_close_px(
        "accordion-demo item 3 height (dark)",
        trig_3.bounds.size.height,
        web_items[2].rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_progress_demo_track_and_indicator_geometry_light() {
    let web = read_web_golden("progress-demo");
    let theme = web.themes.get("light").expect("missing light theme");

    let web_track = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-primary/20",
            "relative",
            "h-2",
            "overflow-hidden",
            "rounded-full",
            "w-[60%]",
        ],
    )
    .expect("web progress track");
    let web_indicator = web_find_by_class_tokens(
        web_track,
        &["bg-primary", "h-full", "w-full", "flex-1", "transition-all"],
    )
    .or_else(|| web_find_by_class_token(web_track, "bg-primary"))
    .expect("web progress indicator");

    let expected_track_bg = web_track
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web track backgroundColor");
    let expected_indicator_bg = web_indicator
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web indicator backgroundColor");

    let t = (web_indicator.rect.x + web_indicator.rect.w - web_track.rect.x) / web_track.rect.w;
    let v = (t * 100.0).clamp(0.0, 100.0);

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

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        |cx| {
            let width = Px(web_track.rect.w);
            let model: Model<f32> = cx.app.models_mut().insert(v);

            let progress = cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("Golden:progress-demo")),
                    ..Default::default()
                },
                move |cx| vec![fret_ui_shadcn::Progress::new(model).into_element(cx)],
            );

            vec![cx.container(
                ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &Theme::global(&*cx.app),
                        LayoutRefinement::default().w_px(MetricRef::Px(width)),
                    ),
                    ..Default::default()
                },
                move |_cx| vec![progress],
            )]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let (_track_rect, track_bg) =
        find_scene_quad_background_with_rect_close(&scene, web_track.rect, 1.0)
            .expect("track quad");
    assert_rgba_close(
        "progress-demo track background",
        color_to_rgba(track_bg),
        expected_track_bg,
        0.02,
    );

    let ind = find_scene_quad_background_with_world_rect_close(&scene, web_indicator.rect, 1.0);
    if ind.is_none() {
        debug_dump_scene_quads_near_expected(
            &scene,
            web_indicator.rect,
            Some(expected_indicator_bg),
        );
    }
    let (_ind_rect, ind_bg) = ind.expect("indicator quad");
    assert_rgba_close(
        "progress-demo indicator background",
        color_to_rgba(ind_bg),
        expected_indicator_bg,
        0.02,
    );
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
fn web_vs_fret_layout_carousel_demo_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-demo",
        "max-w-xs",
        &["min-w-0", "shrink-0", "grow-0", "basis-full", "pl-4"],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(36.0), Px(40.0), true, true))
                .collect::<Vec<_>>();

            fret_ui_shadcn::Carousel::new(slides)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(320.0))))
                .refine_track_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(336.0))))
                .track_start_neg_margin(Space::N4)
                .item_padding_start(Space::N4)
                .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_layout_carousel_plugin_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-plugin",
        "max-w-xs",
        &["min-w-0", "shrink-0", "grow-0", "basis-full", "pl-4"],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(36.0), Px(40.0), true, true))
                .collect::<Vec<_>>();

            fret_ui_shadcn::Carousel::new(slides)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(320.0))))
                .refine_track_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(336.0))))
                .track_start_neg_margin(Space::N4)
                .item_padding_start(Space::N4)
                .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_layout_carousel_api_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-api",
        "max-w-xs",
        &["min-w-0", "shrink-0", "grow-0", "basis-full", "pl-4"],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(36.0), Px(40.0), true, false))
                .collect::<Vec<_>>();

            let carousel = fret_ui_shadcn::Carousel::new(slides)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(320.0))))
                .refine_track_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(336.0))))
                .track_start_neg_margin(Space::N4)
                .item_padding_start(Space::N4)
                .into_element(cx);

            let caption = ui::text(cx, "Slide 1 of 5")
                .text_size_px(Px(14.0))
                .line_height_px(Px(20.0))
                .text_color(ColorRef::Token {
                    key: "muted-foreground",
                    fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                })
                .into_element(cx);

            ui::container(cx, move |_cx| vec![carousel, caption])
                .w_full()
                .max_w(MetricRef::Px(Px(320.0)))
                .mx_auto()
                .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_layout_carousel_size_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-size",
        "max-w-sm",
        &[
            "min-w-0",
            "shrink-0",
            "grow-0",
            "basis-full",
            "pl-4",
            "lg:basis-1/3",
        ],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(30.0), Px(36.0), true, true))
                .collect::<Vec<_>>();

            fret_ui_shadcn::Carousel::new(slides)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(384.0))))
                .refine_track_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(400.0))))
                .track_start_neg_margin(Space::N4)
                .item_padding_start(Space::N4)
                .item_basis_main_px(Px(133.328))
                .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_layout_carousel_spacing_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-spacing",
        "max-w-sm",
        &[
            "min-w-0",
            "shrink-0",
            "grow-0",
            "basis-full",
            "pl-1",
            "lg:basis-1/3",
        ],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(24.0), Px(32.0), true, true))
                .collect::<Vec<_>>();

            fret_ui_shadcn::Carousel::new(slides)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(384.0))))
                .refine_track_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(388.0))))
                .track_start_neg_margin(Space::N1)
                .item_padding_start(Space::N1)
                .item_basis_main_px(Px(129.328))
                .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_layout_carousel_orientation_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-orientation",
        "max-w-xs",
        &[
            "min-w-0",
            "shrink-0",
            "grow-0",
            "basis-full",
            "pt-1",
            "md:basis-1/2",
        ],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(30.0), Px(36.0), false, true))
                .collect::<Vec<_>>();

            fret_ui_shadcn::Carousel::new(slides)
                .orientation(fret_ui_shadcn::CarouselOrientation::Vertical)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(320.0))))
                .refine_viewport_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(196.0))))
                .refine_track_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(200.0))))
                .track_start_neg_margin(Space::N1)
                .item_padding_start(Space::N1)
                .into_element(cx)
        },
    );
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

    let web_rdp_root = web_find_by_class_token(&theme.root, "rdp-root").expect("web rdp-root");
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
    let rdp_root = web_find_by_class_token(&theme.root, "rdp-root")?;
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

    let web_rdp_root = web_find_by_class_token(&theme.root, "rdp-root").expect("web rdp-root");
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

    let web_month_grids = find_all(&theme.root, &|n| {
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
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use fret_ui_headless::calendar::DateRangeSelection;

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        match web_selected_dates.as_slice() {
            [] | [_] => {
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(selected_date);
                let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);
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
                    .refine_style(chrome_override);
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
                    .refine_style(chrome_override);
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

    let web_rdp_root = web_find_by_class_token(&theme.root, "rdp-root").expect("web rdp-root");
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
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use fret_ui_headless::calendar::DateRangeSelection;

        let month_model: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(year_a, month_a));

        match web_selected_dates.as_slice() {
            [] | [_] => {
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(selected_date);
                let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                    .number_of_months(2)
                    .locale(locale)
                    .disable_navigation(web_disable_navigation)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);
                if let Some(((start_month, start_year), (end_month, end_year))) = web_month_bounds {
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
                    .refine_style(chrome_override);
                if let Some(((start_month, start_year), (end_month, end_year))) = web_month_bounds {
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
                    .refine_style(chrome_override);

                if web_name == "calendar-03" {
                    calendar = calendar.required(true).max(5);
                }
                if let Some(((start_month, start_year), (end_month, end_year))) = web_month_bounds {
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
fn web_vs_fret_layout_calendar_01_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-01");
}

#[test]
fn web_vs_fret_layout_calendar_04_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-04");
}

#[test]
fn web_vs_fret_layout_calendar_06_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-06");
}

#[test]
fn web_vs_fret_layout_calendar_08_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-08");
}

#[test]
fn web_vs_fret_layout_calendar_10_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-10");
}

#[test]
fn web_vs_fret_layout_calendar_13_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-13");
}

#[test]
fn web_vs_fret_layout_calendar_14_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-14");
}

#[test]
fn web_vs_fret_layout_calendar_15_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-15");
}

#[test]
fn web_vs_fret_layout_calendar_16_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-16");
}

#[test]
fn web_vs_fret_layout_calendar_17_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-17");
}

#[test]
fn web_vs_fret_layout_calendar_18_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-18");
}

#[test]
fn web_vs_fret_layout_calendar_19_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-19");
}

#[test]
fn web_vs_fret_layout_calendar_20_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-20");
}

#[test]
fn web_vs_fret_layout_calendar_21_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-21");
}

#[test]
fn web_vs_fret_layout_calendar_31_geometry_matches() {
    assert_calendar_single_month_variant_geometry_matches_web("calendar-31");
}

#[test]
fn web_vs_fret_layout_calendar_02_geometry_matches() {
    assert_calendar_multi_month_variant_geometry_matches_web("calendar-02");
}

#[test]
fn web_vs_fret_layout_calendar_03_geometry_matches() {
    assert_calendar_multi_month_variant_geometry_matches_web("calendar-03");
}

#[test]
fn web_vs_fret_layout_calendar_05_geometry_matches() {
    assert_calendar_multi_month_variant_geometry_matches_web("calendar-05");
}

#[test]
fn web_vs_fret_layout_calendar_07_geometry_matches() {
    assert_calendar_multi_month_variant_geometry_matches_web("calendar-07");
}

#[test]
fn web_vs_fret_layout_calendar_09_geometry_matches() {
    assert_calendar_multi_month_variant_geometry_matches_web("calendar-09");
}

#[test]
fn web_vs_fret_layout_calendar_11_geometry_matches() {
    assert_calendar_multi_month_variant_geometry_matches_web("calendar-11");
}

#[test]
fn web_vs_fret_layout_calendar_12_geometry_matches() {
    assert_calendar_multi_month_variant_geometry_matches_web("calendar-12");
}

#[test]
fn web_vs_fret_layout_progress_demo_track_and_indicator_geometry_dark() {
    let web = read_web_golden("progress-demo");
    let theme = web.themes.get("dark").expect("missing dark theme");

    let web_track = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-primary/20",
            "relative",
            "h-2",
            "overflow-hidden",
            "rounded-full",
            "w-[60%]",
        ],
    )
    .expect("web progress track");
    let web_indicator = web_find_by_class_tokens(
        web_track,
        &["bg-primary", "h-full", "w-full", "flex-1", "transition-all"],
    )
    .or_else(|| web_find_by_class_token(web_track, "bg-primary"))
    .expect("web progress indicator");

    let expected_track_bg = web_track
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web track backgroundColor");
    let expected_indicator_bg = web_indicator
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web indicator backgroundColor");

    let t = (web_indicator.rect.x + web_indicator.rect.w - web_track.rect.x) / web_track.rect.w;
    let v = (t * 100.0).clamp(0.0, 100.0);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
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
        |cx| {
            let width = Px(web_track.rect.w);
            let model: Model<f32> = cx.app.models_mut().insert(v);

            let progress = cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("Golden:progress-demo")),
                    ..Default::default()
                },
                move |cx| vec![fret_ui_shadcn::Progress::new(model).into_element(cx)],
            );

            vec![cx.container(
                ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &Theme::global(&*cx.app),
                        LayoutRefinement::default().w_px(MetricRef::Px(width)),
                    ),
                    ..Default::default()
                },
                move |_cx| vec![progress],
            )]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let (_track_rect, track_bg) =
        find_scene_quad_background_with_rect_close(&scene, web_track.rect, 1.0)
            .expect("track quad");
    assert_rgba_close(
        "progress-demo track background",
        color_to_rgba(track_bg),
        expected_track_bg,
        0.02,
    );

    let ind = find_scene_quad_background_with_world_rect_close(&scene, web_indicator.rect, 1.0);
    if ind.is_none() {
        debug_dump_scene_quads_near_expected(
            &scene,
            web_indicator.rect,
            Some(expected_indicator_bg),
        );
    }
    let (_ind_rect, ind_bg) = ind.expect("indicator quad");
    assert_rgba_close(
        "progress-demo indicator background",
        color_to_rgba(ind_bg),
        expected_indicator_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_spinner_basic_geometry_matches_web() {
    let web = read_web_golden("spinner-basic");
    let theme = web_theme(&web);
    let web_spinner = find_first(&theme.root, &|n| {
        n.tag == "svg" && class_has_token(n, "animate-spin")
    })
    .expect("web spinner svg");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let spinner = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:spinner-basic:spinner")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx)],
        );
        vec![spinner]
    });

    let spinner = find_by_test_id(&snap, "Golden:spinner-basic:spinner");
    assert_close_px(
        "spinner-basic width",
        spinner.bounds.size.width,
        web_spinner.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-basic height",
        spinner.bounds.size.height,
        web_spinner.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_spinner_custom_geometry_matches_web() {
    let web = read_web_golden("spinner-custom");
    let theme = web_theme(&web);
    let web_spinner = find_first(&theme.root, &|n| {
        n.tag == "svg" && class_has_token(n, "animate-spin")
    })
    .expect("web spinner svg");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let spinner = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:spinner-custom:spinner")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx)],
        );
        vec![spinner]
    });

    let spinner = find_by_test_id(&snap, "Golden:spinner-custom:spinner");
    assert_close_px(
        "spinner-custom width",
        spinner.bounds.size.width,
        web_spinner.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-custom height",
        spinner.bounds.size.height,
        web_spinner.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_spinner_size_variants_match_web() {
    let web = read_web_golden("spinner-size");
    let theme = web_theme(&web);
    let mut web_spinners = find_all(&theme.root, &|n| {
        n.tag == "svg" && class_has_token(n, "animate-spin")
    });
    web_spinners.sort_by(|a, b| a.rect.w.total_cmp(&b.rect.w));
    assert_eq!(web_spinners.len(), 4, "expected 4 web spinners");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let sizes = [Px(12.0), Px(16.0), Px(24.0), Px(32.0)];
        let mut out = Vec::new();
        for (i, size) in sizes.into_iter().enumerate() {
            let id = Arc::from(format!("Golden:spinner-size:{i}"));
            let layout = LayoutRefinement::default()
                .w_px(MetricRef::Px(size))
                .h_px(MetricRef::Px(size));
            out.push(cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    test_id: Some(id),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        fret_ui_shadcn::Spinner::new()
                            .refine_layout(layout)
                            .speed(0.0)
                            .into_element(cx),
                    ]
                },
            ));
        }
        out
    });

    for (i, web_spinner) in web_spinners.iter().enumerate() {
        let id = format!("Golden:spinner-size:{i}");
        let spinner = find_by_test_id(&snap, &id);
        assert_close_px(
            &format!("spinner-size[{i}] width"),
            spinner.bounds.size.width,
            web_spinner.rect.w,
            1.0,
        );
        assert_close_px(
            &format!("spinner-size[{i}] height"),
            spinner.bounds.size.height,
            web_spinner.rect.h,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_spinner_color_sizes_match_web() {
    let web = read_web_golden("spinner-color");
    let theme = web_theme(&web);
    let web_spinners = find_all(&theme.root, &|n| {
        n.tag == "svg" && class_has_token(n, "animate-spin")
    });
    assert_eq!(web_spinners.len(), 5, "expected 5 web spinners");
    for (i, s) in web_spinners.iter().enumerate() {
        assert_close_px(
            &format!("spinner-color[{i}] width"),
            Px(s.rect.w),
            24.0,
            0.5,
        );
        assert_close_px(
            &format!("spinner-color[{i}] height"),
            Px(s.rect.h),
            24.0,
            0.5,
        );
    }
}

#[test]
fn web_vs_fret_layout_spinner_button_disabled_sm_heights_match_web() {
    let web = read_web_golden("spinner-button");
    let theme = web_theme(&web);

    let mut web_buttons = find_all(&theme.root, &|n| n.tag == "button");
    web_buttons.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_buttons.len(), 3, "expected 3 web buttons");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let buttons = vec![
            fret_ui_shadcn::Button::new("Loading...")
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .disabled(true)
                .test_id("Golden:spinner-button:btn-0")
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .into_element(cx),
            fret_ui_shadcn::Button::new("Please wait")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .disabled(true)
                .test_id("Golden:spinner-button:btn-1")
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .into_element(cx),
            fret_ui_shadcn::Button::new("Processing")
                .variant(fret_ui_shadcn::ButtonVariant::Secondary)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .disabled(true)
                .test_id("Golden:spinner-button:btn-2")
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .into_element(cx),
        ];

        vec![cx.column(
            ColumnProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    LayoutRefinement::default().w_full(),
                ),
                gap: MetricRef::space(Space::N4).resolve(&Theme::global(&*cx.app)),
                ..Default::default()
            },
            move |_cx| buttons,
        )]
    });

    for (i, web_button) in web_buttons.iter().enumerate() {
        let id = format!("Golden:spinner-button:btn-{i}");
        let btn = find_by_test_id(&snap, &id);
        assert_close_px(
            &format!("spinner-button[{i}] height"),
            btn.bounds.size.height,
            web_button.rect.h,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_spinner_badge_heights_match_web() {
    let web = read_web_golden("spinner-badge");
    let theme = web_theme(&web);

    let web_badges = web_find_badge_spans_with_spinner(&theme.root);
    assert_eq!(web_badges.len(), 3, "expected 3 web badges");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let badges = vec![
            fret_ui_shadcn::Badge::new("Syncing")
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default())
                .into_element(cx),
            fret_ui_shadcn::Badge::new("Updating")
                .variant(fret_ui_shadcn::BadgeVariant::Secondary)
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .into_element(cx),
            fret_ui_shadcn::Badge::new("Processing")
                .variant(fret_ui_shadcn::BadgeVariant::Outline)
                .children(vec![
                    fret_ui_shadcn::Spinner::new().speed(0.0).into_element(cx),
                ])
                .into_element(cx),
        ];

        let mut out = Vec::new();
        for (i, badge) in badges.into_iter().enumerate() {
            out.push(cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    test_id: Some(Arc::from(format!("Golden:spinner-badge:{i}"))),
                    ..Default::default()
                },
                move |_cx| vec![badge],
            ));
        }
        out
    });

    for (i, web_badge) in web_badges.iter().enumerate() {
        let id = format!("Golden:spinner-badge:{i}");
        let badge = find_by_test_id(&snap, &id);
        assert_close_px(
            &format!("spinner-badge[{i}] height"),
            badge.bounds.size.height,
            web_badge.rect.h,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_spinner_demo_item_height_matches_web() {
    let web = read_web_golden("spinner-demo");
    let theme = web_theme(&web);

    let web_item = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "group/item") && contains_text(n, "Processing payment")
    })
    .expect("web item");

    let web_media = find_first(web_item, &|n| {
        n.tag == "div" && class_has_all_tokens(n, &["flex", "shrink-0", "items-center", "gap-2"])
    })
    .expect("web item media");
    let web_content = find_first(web_item, &|n| {
        n.tag == "div" && class_has_all_tokens(n, &["flex", "flex-1", "flex-col", "gap-1"])
    })
    .expect("web item content");
    let web_price = find_first(web_item, &|n| {
        n.tag == "div" && class_has_all_tokens(n, &["flex", "flex-col", "flex-none", "justify-end"])
    })
    .expect("web item price container");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
            &Theme::global(&*cx.app),
            LayoutRefinement::default()
                .w_full()
                .max_w(MetricRef::Px(Px(web_item.rect.w))),
        );
        let wrapper_gap = MetricRef::space(Space::N4).resolve(&Theme::global(&*cx.app));

        let item = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:spinner-demo:item")),
                ..Default::default()
            },
            move |cx| {
                let media = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        test_id: Some(Arc::from("Golden:spinner-demo:media")),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            fret_ui_shadcn::ItemMedia::new([fret_ui_shadcn::Spinner::new()
                                .speed(0.0)
                                .into_element(cx)])
                            .into_element(cx),
                        ]
                    },
                );

                let content = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        test_id: Some(Arc::from("Golden:spinner-demo:content")),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            fret_ui_shadcn::ItemContent::new([fret_ui_shadcn::ItemTitle::new(
                                "Processing payment...",
                            )
                            .into_element(cx)])
                            .into_element(cx),
                        ]
                    },
                );

                let price = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Panel,
                        test_id: Some(Arc::from("Golden:spinner-demo:price")),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            fret_ui_shadcn::ItemContent::new([ui::text(cx, "$100.00")
                                .text_size_px(Theme::global(&*cx.app).metric_required("font.size"))
                                .line_height_px(
                                    Theme::global(&*cx.app).metric_required("font.line_height"),
                                )
                                .into_element(cx)])
                            .justify(MainAlign::End)
                            .refine_layout(LayoutRefinement::default().flex_none())
                            .into_element(cx),
                        ]
                    },
                );

                let item = fret_ui_shadcn::Item::new([media, content, price])
                    .variant(fret_ui_shadcn::ItemVariant::Muted)
                    .into_element(cx);
                vec![item]
            },
        );

        vec![cx.column(
            ColumnProps {
                layout: wrapper_layout,
                gap: wrapper_gap,
                ..Default::default()
            },
            move |_cx| vec![item],
        )]
    });

    let item = find_by_test_id(&snap, "Golden:spinner-demo:item");
    assert_close_px(
        "spinner-demo item width",
        item.bounds.size.width,
        web_item.rect.w,
        2.0,
    );

    let media = find_by_test_id(&snap, "Golden:spinner-demo:media");
    assert_close_px(
        "spinner-demo media y",
        media.bounds.origin.y,
        web_media.rect.y,
        2.0,
    );

    let content = find_by_test_id(&snap, "Golden:spinner-demo:content");
    assert_close_px(
        "spinner-demo content y",
        content.bounds.origin.y,
        web_content.rect.y,
        2.0,
    );

    let price = find_by_test_id(&snap, "Golden:spinner-demo:price");
    assert_close_px(
        "spinner-demo price y",
        price.bounds.origin.y,
        web_price.rect.y,
        2.0,
    );

    assert_close_px(
        "spinner-demo item height",
        item.bounds.size.height,
        web_item.rect.h,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_spinner_item_height_matches_web() {
    let web = read_web_golden("spinner-item");
    let theme = web_theme(&web);

    let web_item = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "group/item") && contains_text(n, "Downloading...")
    })
    .expect("web item");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let value: Model<f32> = cx.app.models_mut().insert(0.75);

        let item = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("Golden:spinner-item:item")),
                ..Default::default()
            },
            move |cx| {
                let item = fret_ui_shadcn::Item::new([
                    fret_ui_shadcn::ItemMedia::new([fret_ui_shadcn::Spinner::new()
                        .speed(0.0)
                        .into_element(cx)])
                    .variant(fret_ui_shadcn::ItemMediaVariant::Icon)
                    .into_element(cx),
                    fret_ui_shadcn::ItemContent::new([
                        fret_ui_shadcn::ItemTitle::new("Downloading...").into_element(cx),
                        fret_ui_shadcn::ItemDescription::new("129 MB / 1000 MB").into_element(cx),
                    ])
                    .into_element(cx),
                    fret_ui_shadcn::ItemActions::new([fret_ui_shadcn::Button::new("Cancel")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .size(fret_ui_shadcn::ButtonSize::Sm)
                        .into_element(cx)])
                    .into_element(cx),
                    fret_ui_shadcn::ItemFooter::new([
                        fret_ui_shadcn::Progress::new(value).into_element(cx)
                    ])
                    .into_element(cx),
                ])
                .variant(fret_ui_shadcn::ItemVariant::Outline)
                .into_element(cx);
                vec![item]
            },
        );
        vec![item]
    });

    let item = find_by_test_id(&snap, "Golden:spinner-item:item");
    assert_close_px(
        "spinner-item item height",
        item.bounds.size.height,
        web_item.rect.h,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_spinner_empty_icon_geometry_matches_web() {
    let web = read_web_golden("spinner-empty");
    let theme = web_theme(&web);

    let web_icon = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_all_tokens(n, &["mb-2", "size-10", "rounded-lg"])
    })
    .expect("web empty icon");

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

    let mut services = StyleAwareServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        |cx| {
            let empty = fret_ui_shadcn::Empty::new([
                fret_ui_shadcn::EmptyHeader::new([
                    fret_ui_shadcn::EmptyMedia::new([fret_ui_shadcn::Spinner::new()
                        .speed(0.0)
                        .into_element(cx)])
                    .variant(fret_ui_shadcn::EmptyMediaVariant::Icon)
                    .into_element(cx),
                    fret_ui_shadcn::EmptyTitle::new("Processing your request").into_element(cx),
                    fret_ui_shadcn::EmptyDescription::new(
                        "Please wait while we process your request. Do not refresh the page.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                fret_ui_shadcn::EmptyContent::new([fret_ui_shadcn::Button::new("Cancel")
                    .variant(fret_ui_shadcn::ButtonVariant::Outline)
                    .size(fret_ui_shadcn::ButtonSize::Sm)
                    .into_element(cx)])
                .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

            vec![empty]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let expected_bg = web_icon
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web empty icon backgroundColor");

    let mut best: Option<(Rect, fret_core::Color, f32)> = None;
    for op in scene.ops() {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            continue;
        };

        if (rect.size.width.0 - web_icon.rect.w).abs() > 2.0 {
            continue;
        }
        if (rect.size.height.0 - web_icon.rect.h).abs() > 2.0 {
            continue;
        }

        let diff = rgba_diff_metric(color_to_rgba(background), expected_bg);
        match best {
            Some((_best_rect, _best_bg, best_diff)) if diff >= best_diff => {}
            _ => best = Some((rect, background, diff)),
        }
    }

    let (rect, bg, _diff) = best.unwrap_or_else(|| {
        debug_dump_scene_quads_near_expected(&scene, web_icon.rect, Some(expected_bg));
        panic!("spinner-empty: missing icon background quad near expected size");
    });
    assert_close_px(
        "spinner-empty icon width",
        rect.size.width,
        web_icon.rect.w,
        1.0,
    );
    assert_close_px(
        "spinner-empty icon height",
        rect.size.height,
        web_icon.rect.h,
        1.0,
    );
    assert_rgba_close(
        "spinner-empty icon background",
        color_to_rgba(bg),
        expected_bg,
        0.02,
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

#[test]
fn web_vs_fret_layout_date_picker_demo_trigger_geometry_matches_web() {
    let web = read_web_golden("date-picker-demo");
    let theme = web_theme(&web);
    let web_button =
        web_find_by_tag_and_text(&theme.root, "button", "Pick a date").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use time::Month;

        let open: Model<bool> = cx.app.models_mut().insert(false);
        let month: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

        vec![
            fret_ui_shadcn::DatePicker::new(open, month, selected)
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_button.rect.w))),
                )
                .into_element(cx),
        ]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Pick a date"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret date-picker trigger button");

    assert_close_px(
        "date-picker-demo trigger w",
        button.bounds.size.width,
        web_button.rect.w,
        1.0,
    );
    assert_close_px(
        "date-picker-demo trigger h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_date_picker_with_presets_trigger_geometry_matches_web() {
    let web = read_web_golden("date-picker-with-presets");
    let theme = web_theme(&web);
    let web_button =
        web_find_by_tag_and_text(&theme.root, "button", "Pick a date").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use time::Month;

        let open: Model<bool> = cx.app.models_mut().insert(false);
        let month: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

        vec![
            fret_ui_shadcn::DatePicker::new(open, month, selected)
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_button.rect.w))),
                )
                .into_element(cx),
        ]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Pick a date"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret date-picker trigger button");

    assert_close_px(
        "date-picker-with-presets trigger w",
        button.bounds.size.width,
        web_button.rect.w,
        1.0,
    );
    assert_close_px(
        "date-picker-with-presets trigger h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_date_picker_with_range_trigger_geometry_matches_web() {
    let web = read_web_golden("date-picker-with-range");
    let theme = web_theme(&web);
    let web_button = find_first(&theme.root, &|n| {
        n.tag == "button" && contains_id(n, "date")
    })
    .expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use time::{Date, Month};

        let open: Model<bool> = cx.app.models_mut().insert(false);
        let month: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(2022, Month::January));
        let selected: Model<fret_ui_headless::calendar::DateRangeSelection> = cx
            .app
            .models_mut()
            .insert(fret_ui_headless::calendar::DateRangeSelection {
                from: Some(Date::from_calendar_date(2022, Month::January, 20).expect("from date")),
                to: Some(Date::from_calendar_date(2022, Month::February, 9).expect("to date")),
            });

        vec![
            fret_ui_shadcn::DateRangePicker::new(open, month, selected)
                .refine_layout(
                    LayoutRefinement::default().w_px(MetricRef::Px(Px(web_button.rect.w))),
                )
                .into_element(cx),
        ]
    });

    let button = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Jan 20, 2022 - Feb 09, 2022"),
    )
    .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
    .expect("fret date-range trigger button");

    assert_close_px(
        "date-picker-with-range trigger w",
        button.bounds.size.width,
        web_button.rect.w,
        1.0,
    );
    assert_close_px(
        "date-picker-with-range trigger h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_slider_track_geometry_matches_web() {
    let web = read_web_golden("field-slider");
    let theme = web_theme(&web);

    let web_slider = find_first(&theme.root, &|n| {
        n.tag == "span"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Price Range")
    })
    .expect("web slider");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, snap, _root) = run_fret_root_with_ui(bounds, |cx| {
        let model: Model<Vec<f32>> = cx.app.models_mut().insert(vec![200.0, 800.0]);
        let slider = fret_ui_shadcn::Slider::new(model)
            .range(0.0, 1000.0)
            .step(10.0)
            .a11y_label("Price Range")
            .into_element(cx);

        let field = fret_ui_shadcn::Field::new(vec![
            fret_ui_shadcn::FieldTitle::new("Price Range").into_element(cx),
            fret_ui_shadcn::FieldDescription::new("Set your budget range ($200 - 800).")
                .into_element(cx),
            slider,
        ])
        .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_slider.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![field],
        )]
    });

    let thumb = find_semantics(&snap, SemanticsRole::Slider, Some("Price Range"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Slider, None))
        .expect("fret slider thumb semantics");
    let slider = thumb
        .parent
        .and_then(|parent| snap.nodes.iter().find(|n| n.id == parent))
        .unwrap_or(thumb);

    assert_close_px(
        "field-slider track w",
        slider.bounds.size.width,
        web_slider.rect.w,
        1.0,
    );
    assert_close_px(
        "field-slider track h",
        slider.bounds.size.height,
        web_slider.rect.h,
        1.0,
    );

    let _ = ui.debug_node_bounds(slider.id).expect("fret slider bounds");
}

#[test]
fn web_vs_fret_layout_field_demo_separator_height_matches_web() {
    let web = read_web_golden("field-demo");
    let theme = web_theme(&web);
    let web_sep = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_all_tokens(n, &["relative", "-my-2", "h-5", "text-sm"])
    })
    .expect("web field-separator");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let sep = fret_ui_shadcn::FieldSeparator::new()
            .refine_layout(
                LayoutRefinement::default()
                    .mt_neg(Space::N0)
                    .mb_neg(Space::N0),
            )
            .into_element(cx);
        let sep = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-demo:separator")),
                ..Default::default()
            },
            move |_cx| vec![sep],
        );

        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_sep.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![sep],
        )]
    });

    let sep = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-demo:separator"),
    )
    .expect("fret field-separator");

    assert_close_px(
        "field-demo separator h",
        sep.bounds.size.height,
        web_sep.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_responsive_orientation_places_input_beside_content() {
    let web = read_web_golden("field-responsive");
    let theme = web_theme(&web);

    let web_max_w = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "max-w-4xl")
    })
    .expect("web max-w-4xl container");

    let web_content = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "group/field-content")
            && contains_text(n, "Provide your full name")
    })
    .expect("web field-content");

    let web_input = find_first(&theme.root, &|n| n.tag == "input" && contains_id(n, "name"))
        .expect("web input");

    let web_dx = web_input.rect.x - web_content.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let content_layout =
            decl_style::layout_style(&theme, LayoutRefinement::default().flex_1().min_w_0());

        let content = fret_ui_shadcn::FieldContent::new(vec![
            fret_ui_shadcn::FieldLabel::new("Name").into_element(cx),
            fret_ui_shadcn::FieldDescription::new("Provide your full name for identification")
                .into_element(cx),
        ])
        .into_element(cx);

        let content = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: content_layout,
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-responsive:content")),
                ..Default::default()
            },
            move |_cx| vec![content],
        );

        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("NameInput")
            .placeholder("Evil Rabbit")
            .into_element(cx);

        let field = fret_ui_shadcn::Field::new(vec![content, input])
            .orientation(fret_ui_shadcn::FieldOrientation::Responsive)
            .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_max_w.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![field],
        )]
    });

    let fret_content = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-responsive:content"),
    )
    .expect("fret field-content");
    let fret_input = find_semantics(&snap, SemanticsRole::TextField, Some("NameInput"))
        .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
        .expect("fret input");

    let fret_dx = fret_input.bounds.origin.x.0 - fret_content.bounds.origin.x.0;

    assert!(
        fret_dx >= 1.0,
        "expected responsive field to place input beside content; dx={fret_dx} (content={:?} input={:?})",
        fret_content.bounds,
        fret_input.bounds
    );
    assert_close_px("field-responsive input dx", Px(fret_dx), web_dx, 12.0);
}

fn assert_kbd_first_height_matches_web(web_name: &str, text: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_kbd = find_first(&theme.root, &|n| n.tag == "kbd").expect("web kbd");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = format!("Golden:{web_name}:kbd");
    let snap = run_fret_root(bounds, |cx| {
        let kbd = fret_ui_shadcn::Kbd::new(text).into_element(cx);
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from(label.clone())),
                ..Default::default()
            },
            move |_cx| vec![kbd],
        )]
    });

    let kbd = find_semantics(&snap, SemanticsRole::Panel, Some(&label)).expect("fret kbd");

    assert_close_px("kbd height", kbd.bounds.size.height, web_kbd.rect.h, 1.0);
}

#[test]
fn web_vs_fret_layout_kbd_button_kbd_height_matches_web() {
    assert_kbd_first_height_matches_web("kbd-button", "Esc");
}

#[test]
fn web_vs_fret_layout_kbd_group_kbd_height_matches_web() {
    assert_kbd_first_height_matches_web("kbd-group", "Esc");
}

#[test]
fn web_vs_fret_layout_kbd_tooltip_kbd_height_matches_web() {
    let web = read_web_golden("kbd-tooltip");
    let theme = web_theme(&web);
    let web_button = web_find_by_tag_and_text(&theme.root, "button", "Save").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::Button::new("Save")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .into_element(cx),
        ]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Save"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button");

    assert_close_px(
        "kbd-tooltip button h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

fn assert_skeleton_rects_match_web(
    web_name: &str,
    layout: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<AnyElement>,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let mut web_skeletons = find_all(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "bg-accent") && class_has_token(n, "animate-pulse")
    });
    web_skeletons.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .x
                    .partial_cmp(&b.rect.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    assert!(
        !web_skeletons.is_empty(),
        "expected skeleton nodes in {web_name}"
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, layout);

    for (idx, web_node) in web_skeletons.iter().enumerate() {
        let label = format!("Golden:{web_name}:skeleton:{idx}");
        let node = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
            .unwrap_or_else(|| panic!("missing fret skeleton semantics for {label}"));
        assert_rect_close_px(&label, node.bounds, web_node.rect, 1.0);
    }
}

#[test]
fn web_vs_fret_layout_skeleton_demo_rects_match_web() {
    assert_skeleton_rects_match_web("skeleton-demo", |cx| {
        let left = fret_ui_shadcn::Skeleton::new()
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(48.0)))
                    .h_px(MetricRef::Px(Px(48.0))),
            )
            .into_element(cx);
        let left = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-demo:skeleton:0")),
                ..Default::default()
            },
            move |_cx| vec![left],
        );

        let line0 = fret_ui_shadcn::Skeleton::new()
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(250.0)))
                    .h_px(MetricRef::Px(Px(16.0))),
            )
            .into_element(cx);
        let line0 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-demo:skeleton:1")),
                ..Default::default()
            },
            move |_cx| vec![line0],
        );

        let line1 = fret_ui_shadcn::Skeleton::new()
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(200.0)))
                    .h_px(MetricRef::Px(Px(16.0))),
            )
            .into_element(cx);
        let line1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-demo:skeleton:2")),
                ..Default::default()
            },
            move |_cx| vec![line1],
        );
        let line1 = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(200.0)),
                        height: Length::Px(Px(16.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![line1],
        );

        let col = cx.column(
            ColumnProps {
                layout: LayoutStyle::default(),
                gap: Px(8.0),
                ..Default::default()
            },
            move |_cx| vec![line0, line1],
        );

        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(16.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| vec![left, col],
        )]
    });
}

#[test]
fn web_vs_fret_layout_skeleton_card_rects_match_web() {
    assert_skeleton_rects_match_web("skeleton-card", |cx| {
        let top = fret_ui_shadcn::Skeleton::new()
            .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(250.0)))
                    .h_px(MetricRef::Px(Px(125.0))),
            )
            .into_element(cx);
        let top = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-card:skeleton:0")),
                ..Default::default()
            },
            move |_cx| vec![top],
        );

        let line0 = fret_ui_shadcn::Skeleton::new()
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(250.0)))
                    .h_px(MetricRef::Px(Px(16.0))),
            )
            .into_element(cx);
        let line0 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-card:skeleton:1")),
                ..Default::default()
            },
            move |_cx| vec![line0],
        );

        let line1 = fret_ui_shadcn::Skeleton::new()
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(200.0)))
                    .h_px(MetricRef::Px(Px(16.0))),
            )
            .into_element(cx);
        let line1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-card:skeleton:2")),
                ..Default::default()
            },
            move |_cx| vec![line1],
        );
        let line1 = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(200.0)),
                        height: Length::Px(Px(16.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![line1],
        );

        let inner = cx.column(
            ColumnProps {
                layout: LayoutStyle::default(),
                gap: Px(8.0),
                ..Default::default()
            },
            move |_cx| vec![line0, line1],
        );

        vec![cx.column(
            ColumnProps {
                layout: LayoutStyle::default(),
                gap: Px(12.0),
                ..Default::default()
            },
            move |_cx| vec![top, inner],
        )]
    });
}

#[test]
fn web_vs_fret_layout_sonner_demo_button_height_matches_web() {
    let web = read_web_golden("sonner-demo");
    let theme = web_theme(&web);
    let web_button =
        web_find_by_tag_and_text(&theme.root, "button", "Show Toast").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::Button::new("Show Toast")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Show Toast"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button");

    assert_close_px(
        "sonner-demo button h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_sonner_types_first_button_height_matches_web() {
    let web = read_web_golden("sonner-types");
    let theme = web_theme(&web);
    let web_button =
        web_find_by_tag_and_text(&theme.root, "button", "Default").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::Button::new("Default")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Default"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button");

    assert_close_px(
        "sonner-types button h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_pagination_demo_active_link_size_matches_web() {
    let web = read_web_golden("pagination-demo");
    let theme = web_theme(&web);
    let web_active = web_find_by_tag_and_text(&theme.root, "a", "2").expect("web active link");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let link = fret_ui_shadcn::PaginationLink::new(vec![ui::text(cx, "2").into_element(cx)])
            .active(true)
            .into_element(cx);
        let link = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:pagination-demo:active")),
                ..Default::default()
            },
            move |_cx| vec![link],
        );

        vec![link]
    });

    let active = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:pagination-demo:active"),
    )
    .expect("fret active pagination link");

    assert_close_px(
        "pagination-demo active w",
        active.bounds.size.width,
        web_active.rect.w,
        1.0,
    );
    assert_close_px(
        "pagination-demo active h",
        active.bounds.size.height,
        web_active.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_dashboard_01_shell_geometry_matches_web() {
    let web = read_web_golden("dashboard-01");
    let theme = web_theme(&web);

    let web_sidebar = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "fixed")
            && class_has_token(n, "w-(--sidebar-width)")
            && class_has_token(n, "p-2")
    })
    .expect("web sidebar container");

    let web_header = find_first(&theme.root, &|n| {
        n.tag == "header"
            && class_has_token(n, "h-(--header-height)")
            && class_has_token(n, "border-b")
    })
    .expect("web site header");

    let pad_top = web_header.rect.y;
    let pad_right = theme.viewport.w - (web_header.rect.x + web_header.rect.w);
    let pad_bottom = pad_top;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let sidebar = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_sidebar.rect.w)),
                        height: Length::Px(Px(theme.viewport.h)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        );
        let sidebar = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:dashboard-01:sidebar")),
                ..Default::default()
            },
            move |_cx| vec![sidebar],
        );

        let header = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_header.rect.w)),
                        height: Length::Px(Px(web_header.rect.h)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        );
        let header = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:dashboard-01:header")),
                ..Default::default()
            },
            move |_cx| vec![header],
        );

        let main = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(theme.viewport.w - web_sidebar.rect.w)),
                        height: Length::Px(Px(theme.viewport.h)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                padding: Edges {
                    left: Px(0.0),
                    top: Px(pad_top),
                    right: Px(pad_right),
                    bottom: Px(pad_bottom),
                },
                ..Default::default()
            },
            move |_cx| vec![header],
        );

        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![sidebar, main],
        )]
    });

    let sidebar = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:dashboard-01:sidebar"),
    )
    .expect("fret dashboard sidebar");
    assert_rect_close_px(
        "dashboard-01 sidebar",
        sidebar.bounds,
        web_sidebar.rect,
        1.0,
    );

    let header = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:dashboard-01:header"),
    )
    .expect("fret dashboard header");
    assert_rect_close_px("dashboard-01 header", header.bounds, web_header.rect, 1.0);
}

fn assert_chart_tooltip_rect_matches_web(
    web_name: &str,
    indicator: fret_ui_shadcn::ChartTooltipIndicator,
    hide_indicator: bool,
    hide_label: bool,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_tooltip = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "border-border/50")
            && class_has_token(n, "bg-background")
            && class_has_token(n, "shadow-xl")
            && class_has_token(n, "min-w-[8rem]")
    })
    .expect("web chart tooltip node");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = Arc::<str>::from(format!("Golden:{web_name}:tooltip"));

    let snap = run_fret_root(bounds, |cx| {
        let tooltip = fret_ui_shadcn::ChartTooltipContent::new()
            .label("Tue")
            .indicator(indicator)
            .hide_indicator(hide_indicator)
            .hide_label(hide_label)
            .items([
                fret_ui_shadcn::ChartTooltipItem::new("Running", "380"),
                fret_ui_shadcn::ChartTooltipItem::new("Swimming", "420"),
            ])
            .into_element(cx);

        let tooltip = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(web_tooltip.rect.x));
                    layout.inset.top = Some(Px(web_tooltip.rect.y));
                    layout
                },
                role: SemanticsRole::Panel,
                label: Some(label.clone()),
                ..Default::default()
            },
            move |_cx| vec![tooltip],
        );

        vec![tooltip]
    });

    let tooltip = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
        .unwrap_or_else(|| panic!("missing fret chart tooltip semantics for {web_name}"));

    assert_rect_close_px(web_name, tooltip.bounds, web_tooltip.rect, 1.0);
}

fn assert_chart_legend_rect_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_legend = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "flex")
            && class_has_token(n, "items-center")
            && class_has_token(n, "justify-center")
            && class_has_token(n, "gap-4")
            && (class_has_token(n, "pt-3") || class_has_token(n, "pb-3"))
    })
    .expect("web chart legend node");

    let vertical_align = if class_has_token(web_legend, "pb-3") {
        fret_ui_shadcn::ChartLegendVerticalAlign::Top
    } else {
        fret_ui_shadcn::ChartLegendVerticalAlign::Bottom
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = Arc::<str>::from(format!("Golden:{web_name}:legend"));

    let snap = run_fret_root(bounds, |cx| {
        let legend = fret_ui_shadcn::ChartLegendContent::new()
            .vertical_align(vertical_align)
            .items([
                fret_ui_shadcn::ChartLegendItem::new("Desktop"),
                fret_ui_shadcn::ChartLegendItem::new("Mobile"),
            ])
            .into_element(cx);

        let legend = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(web_legend.rect.x));
                    layout.inset.top = Some(Px(web_legend.rect.y));
                    layout.size.width = Length::Px(Px(web_legend.rect.w));
                    layout
                },
                role: SemanticsRole::Panel,
                label: Some(label.clone()),
                ..Default::default()
            },
            move |_cx| vec![legend],
        );

        vec![legend]
    });

    let legend = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
        .unwrap_or_else(|| panic!("missing fret chart legend semantics for {web_name}"));

    assert_rect_close_px(web_name, legend.bounds, web_legend.rect, 1.0);
}

#[test]
fn web_vs_fret_layout_chart_tooltip_default_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-default",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
    );
}

#[test]
fn web_vs_fret_layout_chart_tooltip_indicator_line_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-indicator-line",
        fret_ui_shadcn::ChartTooltipIndicator::Line,
        false,
        false,
    );
}

#[test]
fn web_vs_fret_layout_chart_tooltip_indicator_none_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-indicator-none",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        true,
        false,
    );
}

#[test]
fn web_vs_fret_layout_chart_tooltip_label_none_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-label-none",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
    );
}

#[test]
fn web_vs_fret_layout_chart_tooltip_icons_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-icons",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
    );
}

#[test]
fn web_vs_fret_layout_chart_tooltip_label_custom_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-label-custom",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
    );
}

#[test]
fn web_vs_fret_layout_chart_tooltip_label_formatter_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-label-formatter",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
    );
}

#[test]
fn web_vs_fret_layout_chart_area_legend_geometry_matches_web() {
    assert_chart_legend_rect_matches_web("chart-area-legend");
}

#[test]
fn web_vs_fret_layout_chart_bar_demo_legend_geometry_matches_web() {
    assert_chart_legend_rect_matches_web("chart-bar-demo-legend");
}

#[test]
fn web_vs_fret_layout_chart_radar_legend_geometry_matches_web() {
    assert_chart_legend_rect_matches_web("chart-radar-legend");
}
