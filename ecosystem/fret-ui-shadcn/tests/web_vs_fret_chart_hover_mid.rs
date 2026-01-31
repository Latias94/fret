use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::tree::UiTree;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod chart_test_data;
use chart_test_data::CHART_INTERACTIVE_DESKTOP;

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
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebNode {
    tag: String,
    #[serde(default)]
    #[serde(rename = "className")]
    class_name: Option<String>,
    rect: WebRect,
    #[serde(default)]
    children: Vec<WebNode>,
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
    golden.themes.get("light").expect("missing light theme")
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

fn class_has_token(node: &WebNode, token: &str) -> bool {
    node.class_name
        .as_deref()
        .is_some_and(|class| class.split_whitespace().any(|t| t == token))
}

fn web_find_chart_tooltip_panel<'a>(root: &'a WebNode) -> &'a WebNode {
    find_first(root, &|n| {
        n.tag == "div"
            && class_has_token(n, "border-border/50")
            && class_has_token(n, "bg-background")
            && class_has_token(n, "shadow-xl")
            && class_has_token(n, "min-w-[8rem]")
            && class_has_token(n, "w-[150px]")
    })
    .expect("web chart tooltip panel")
}

fn web_find_chart_container<'a>(root: &'a WebNode) -> &'a WebNode {
    find_first(root, &|n| {
        n.tag == "div"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-cartesian"))
    })
    .expect("web chart container")
}

fn web_find_chart_grid<'a>(root: &'a WebNode) -> &'a WebNode {
    find_first(root, &|n| {
        n.tag == "g"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-cartesian-grid"))
    })
    .expect("web chart grid")
}

fn web_find_chart_svg<'a>(root: &'a WebNode) -> &'a WebNode {
    find_first(root, &|n| {
        n.tag == "svg" && n.class_name.as_deref() == Some("recharts-surface")
    })
    .expect("web chart svg")
}

fn web_find_chart_tooltip_cursor<'a>(root: &'a WebNode) -> &'a WebNode {
    find_first(root, &|n| {
        n.tag == "path"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-tooltip-cursor"))
    })
    .expect("web chart tooltip cursor")
}

fn web_find_chart_active_dot_circle<'a>(root: &'a WebNode) -> &'a WebNode {
    fn walk<'a>(node: &'a WebNode, active_dot_layer: bool, out: &mut Option<&'a WebNode>) {
        if out.is_some() {
            return;
        }

        let active_dot_layer = active_dot_layer
            || node
                .class_name
                .as_deref()
                .is_some_and(|c| c.contains("recharts-active-dot"));

        if active_dot_layer
            && node.tag == "circle"
            && node.class_name.as_deref() == Some("recharts-dot")
        {
            *out = Some(node);
            return;
        }

        for child in &node.children {
            walk(child, active_dot_layer, out);
            if out.is_some() {
                return;
            }
        }
    }

    let mut out = None;
    walk(root, false, &mut out);
    out.expect("web chart active dot circle")
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

fn run_fret_root(
    bounds: Rect,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<AnyElement>,
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
        "web-vs-fret-chart-hover-mid",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot")
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

fn assert_close_px(label: &str, a: Px, b: f32, tol: f32) {
    let delta = (a.0 - b).abs();
    assert!(
        delta <= tol,
        "{label}: expected ~{b:.3}px, got {:.3}px (Δ={delta:.3}px, tol={tol:.3}px)",
        a.0
    );
}

fn assert_rect_close_px(label: &str, actual: Rect, expected: WebRect, tol: f32) {
    assert_close_px(&format!("{label} x"), actual.origin.x, expected.x, tol);
    assert_close_px(&format!("{label} y"), actual.origin.y, expected.y, tol);
    assert_close_px(&format!("{label} w"), actual.size.width, expected.w, tol);
    assert_close_px(&format!("{label} h"), actual.size.height, expected.h, tol);
}

fn assert_chart_hover_tooltip_size_matches_web(web_name: &str, tooltip_label: &str, value: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_tooltip = web_find_chart_tooltip_panel(&theme.root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let semantics_label = Arc::<str>::from(format!("Golden:{web_name}:tooltip"));
    let semantics_label_in_tree = semantics_label.clone();
    let value = Arc::<str>::from(value);
    let tooltip_label = Arc::<str>::from(tooltip_label);
    let web_w = Px(web_tooltip.rect.w);

    let snap = run_fret_root(bounds, move |cx| {
        let tooltip = fret_ui_shadcn::ChartTooltipContent::new()
            .label(tooltip_label.clone())
            .fixed_width_border_box(web_w)
            .items([fret_ui_shadcn::ChartTooltipItem::new(
                "Page Views",
                value.clone(),
            )])
            .into_element(cx);

        let tooltip = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(0.0));
                    layout.inset.top = Some(Px(0.0));
                    layout
                },
                role: SemanticsRole::Panel,
                label: Some(semantics_label_in_tree.clone()),
                ..Default::default()
            },
            move |_cx| vec![tooltip],
        );

        vec![tooltip]
    });

    let tooltip = find_semantics(&snap, SemanticsRole::Panel, Some(&semantics_label))
        .unwrap_or_else(|| panic!("missing fret chart tooltip semantics for {web_name}"));

    assert_close_px(
        &format!("{web_name} tooltip w"),
        tooltip.bounds.size.width,
        web_tooltip.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} tooltip h"),
        tooltip.bounds.size.height,
        web_tooltip.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_chart_line_interactive_hover_mid_tooltip_size_matches_web() {
    assert_chart_hover_tooltip_size_matches_web(
        "chart-line-interactive.hover-mid",
        "May 16, 2024",
        "338",
    );
}

#[test]
fn web_vs_fret_chart_bar_interactive_hover_mid_tooltip_size_matches_web() {
    assert_chart_hover_tooltip_size_matches_web(
        "chart-bar-interactive.hover-mid",
        "May 16, 2024",
        "338",
    );
}

#[test]
fn web_vs_fret_chart_line_interactive_hover_mid_cursor_rect_matches_web() {
    let web_name = "chart-line-interactive.hover-mid";
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let chart = web_find_chart_container(&theme.root);
    let plot = web_find_chart_grid(chart).rect;
    let svg = web_find_chart_svg(&theme.root).rect;
    let cursor = web_find_chart_tooltip_cursor(&theme.root).rect;

    let plot = Rect::new(
        Point::new(Px(plot.x), Px(plot.y)),
        CoreSize::new(Px(plot.w), Px(plot.h)),
    );

    let hover_x = svg.x + svg.w * 0.5;
    let n = CHART_INTERACTIVE_DESKTOP.len();
    let step_x = if n > 1 {
        plot.size.width.0 / (n as f32 - 1.0)
    } else {
        0.0
    };
    let idx = if step_x > 0.0 {
        ((hover_x - plot.origin.x.0) / step_x).round()
    } else {
        0.0
    };
    let idx = idx.clamp(0.0, (n.saturating_sub(1)) as f32).round() as usize;
    let expected_x = plot.origin.x.0 + idx as f32 * step_x;

    assert_rect_close_px(
        &format!("{web_name} cursor"),
        Rect::new(
            Point::new(Px(expected_x), Px(plot.origin.y.0)),
            CoreSize::new(Px(0.0), Px(plot.size.height.0)),
        ),
        cursor,
        1.0,
    );
}

#[test]
fn web_vs_fret_chart_bar_interactive_hover_mid_cursor_rect_matches_web() {
    let web_name = "chart-bar-interactive.hover-mid";
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let chart = web_find_chart_container(&theme.root);
    let plot = web_find_chart_grid(chart).rect;
    let svg = web_find_chart_svg(&theme.root).rect;
    let cursor = web_find_chart_tooltip_cursor(&theme.root).rect;

    let plot = Rect::new(
        Point::new(Px(plot.x), Px(plot.y)),
        CoreSize::new(Px(plot.w), Px(plot.h)),
    );

    let hover_x = svg.x + svg.w * 0.5;
    let n = CHART_INTERACTIVE_DESKTOP.len();
    let step = if n > 0 {
        plot.size.width.0 / n as f32
    } else {
        0.0
    };
    let idx = if step > 0.0 {
        ((hover_x - plot.origin.x.0) / step - 0.5).round()
    } else {
        0.0
    };
    let idx = idx.clamp(0.0, (n.saturating_sub(1)) as f32).round() as usize;

    let expected_x = plot.origin.x.0 + idx as f32 * step;
    let expected = Rect::new(
        Point::new(Px(expected_x), Px(plot.origin.y.0 + 0.5)),
        CoreSize::new(Px(step), Px((plot.size.height.0 - 1.0).max(0.0))),
    );

    assert_rect_close_px(&format!("{web_name} cursor"), expected, cursor, 1.0);
}

#[test]
fn web_vs_fret_chart_line_interactive_hover_mid_active_dot_rect_matches_web() {
    let web_name = "chart-line-interactive.hover-mid";
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let chart = web_find_chart_container(&theme.root);
    let plot = web_find_chart_grid(chart).rect;
    let svg = web_find_chart_svg(&theme.root).rect;
    let dot = web_find_chart_active_dot_circle(chart);

    let plot = Rect::new(
        Point::new(Px(plot.x), Px(plot.y)),
        CoreSize::new(Px(plot.w), Px(plot.h)),
    );

    let hover_x = svg.x + svg.w * 0.5;
    let n = CHART_INTERACTIVE_DESKTOP.len();
    let step_x = if n > 1 {
        plot.size.width.0 / (n as f32 - 1.0)
    } else {
        0.0
    };
    let idx = if step_x > 0.0 {
        ((hover_x - plot.origin.x.0) / step_x).round()
    } else {
        0.0
    };
    let idx = idx.clamp(0.0, (n.saturating_sub(1)) as f32).round() as usize;

    assert_eq!(
        CHART_INTERACTIVE_DESKTOP[idx], 338.0,
        "{web_name}: expected the hover-mid point to match the tooltip value"
    );

    let domain_max = fret_ui_shadcn::recharts_geometry::nice_domain_max_for_values(
        &CHART_INTERACTIVE_DESKTOP,
        5,
    );
    let expected = fret_ui_shadcn::recharts_geometry::line_dot_rect(
        plot,
        &CHART_INTERACTIVE_DESKTOP,
        domain_max,
        idx,
        4.0,
    )
    .expect("expected dot rect");

    assert_rect_close_px(&format!("{web_name} active-dot"), expected, dot.rect, 1.0);
}
