use fret_app::App;
use fret_core::{
    AppWindowId, Edges, Event, FrameId, ImageId, Modifiers, MouseButtons, NodeId, Point,
    PointerEvent, PointerId, PointerType, Px, Rect, Scene, SceneOp, SemanticsRole,
    Size as CoreSize, TextOverflow, TextWrap,
};
use fret_runtime::Model;
use fret_ui::Theme;
use fret_ui::element::{
    ColumnProps, ContainerProps, CrossAlign, FlexProps, GridProps, LayoutStyle, Length, MainAlign,
    PressableProps, RovingFlexProps, SizeStyle, TextProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::tree::UiTree;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::primitives::radio_group as radio_group_prim;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Radius, Space};
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
    offset_width: f32,
    #[serde(rename = "offsetHeight")]
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

fn web_collect_all<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
    out.push(node);
    for child in &node.children {
        web_collect_all(child, out);
    }
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

#[derive(Default)]
struct StyleAwareServices;

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

    let snap = run_fret_root(bounds, |cx| {
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

    let web_caption_gap =
        web_caption.rect.y - (web_footer_row.rect.y + web_footer_row.rect.h);

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
                    fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, invoice)).into_element(cx),
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

    let target_caption_y = footer_row.bounds.origin.y.0 + footer_row.bounds.size.height.0
        + web_caption_gap;
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
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Select row")
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
            .children(vec![fret_ui_shadcn::Spinner::new()
                .speed(0.0)
                .into_element(cx)])
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
                            fret_ui_shadcn::TableCell::new(decl_text::text_sm(cx, "ken99@example.com"))
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
