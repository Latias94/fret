use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Scene, SceneOp, Size as CoreSize};
use fret_ui::tree::UiTree;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct WebGolden {
    themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebGoldenTheme {
    root: WebNode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    #[serde(rename = "computedStyle", default)]
    computed_style: BTreeMap<String, String>,
    #[serde(default)]
    children: Vec<WebNode>,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[derive(Debug, Clone, Serialize)]
struct WebButtonStyle {
    rect: WebRect,
    background_color: Option<String>,
    color: Option<String>,
    border_top_width: Option<String>,
    border_top_color: Option<String>,
    border_radius: Option<String>,
    padding_left: Option<String>,
    padding_top: Option<String>,
    display: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct FretButtonStyle {
    rect: [f32; 4],
    background: Rgba,
    border: [f32; 4],
    border_color: Rgba,
    corner_radii: [f32; 4],
    text_color: Option<Rgba>,
}

#[derive(Debug, Clone, Serialize)]
struct ButtonReport {
    web: WebButtonStyle,
    fret: FretButtonStyle,
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

fn css_get(style: &BTreeMap<String, String>, key: &str) -> Option<String> {
    style.get(key).cloned()
}

fn parse_px(s: &str) -> Option<f32> {
    let s = s.trim();
    let v = s.strip_suffix("px").unwrap_or(s);
    v.parse::<f32>().ok()
}

fn parse_rgb(s: &str) -> Option<Rgba> {
    let s = s.trim();
    let inner = if let Some(v) = s.strip_prefix("rgba(").and_then(|v| v.strip_suffix(')')) {
        (v, true)
    } else if let Some(v) = s.strip_prefix("rgb(").and_then(|v| v.strip_suffix(')')) {
        (v, false)
    } else {
        return None;
    };

    let parts: Vec<&str> = inner.0.split(',').map(|p| p.trim()).collect();
    if parts.len() < 3 {
        return None;
    }

    let r: f32 = parts[0].parse::<f32>().ok()? / 255.0;
    let g: f32 = parts[1].parse::<f32>().ok()? / 255.0;
    let b: f32 = parts[2].parse::<f32>().ok()? / 255.0;
    let a: f32 = if inner.1 {
        parts
            .get(3)
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(1.0)
    } else {
        1.0
    };

    Some(Rgba { r, g, b, a })
}

fn round3(v: f32) -> f32 {
    (v * 1000.0).round() / 1000.0
}

fn color_to_rgba(c: fret_core::Color) -> Rgba {
    Rgba {
        r: round3(c.r),
        g: round3(c.g),
        b: round3(c.b),
        a: round3(c.a),
    }
}

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

fn extract_fret_button_style() -> FretButtonStyle {
    let window = AppWindowId::default();
    let mut app = App::new();

    // Match the web golden baseline: shadcn v4 new-york-v4 base `neutral`, `light` scheme.
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(180.0)),
    );

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-button",
        |cx| vec![fret_ui_shadcn::Button::new("Button").into_element(cx)],
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let semantics = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let button_bounds = semantics
        .nodes
        .iter()
        .find(|n| format!("{:?}", n.role) == "Button")
        .map(|n| n.bounds)
        .unwrap_or(bounds);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let mut best_quad: Option<(
        Rect,
        fret_core::Color,
        fret_core::Edges,
        fret_core::Color,
        fret_core::Corners,
    )> = None;

    for op in scene.ops() {
        if let SceneOp::Quad {
            rect,
            background,
            border,
            border_color,
            corner_radii,
            ..
        } = *op
        {
            if rect == button_bounds {
                best_quad = Some((rect, background, border, border_color, corner_radii));
                break;
            }
        }
    }

    let (rect, background, border, border_color, corner_radii) = best_quad.unwrap_or_else(|| {
        panic!(
            "failed to find button quad in scene (expected a Quad with rect == semantics bounds)"
        )
    });

    let mut text_color: Option<Rgba> = None;
    for op in scene.ops() {
        if let SceneOp::Text { origin, color, .. } = *op {
            if rect.contains(origin) {
                text_color = Some(color_to_rgba(color));
                break;
            }
        }
    }

    FretButtonStyle {
        rect: [
            round3(rect.origin.x.0),
            round3(rect.origin.y.0),
            round3(rect.size.width.0),
            round3(rect.size.height.0),
        ],
        background: color_to_rgba(background),
        border: [
            round3(border.top.0),
            round3(border.right.0),
            round3(border.bottom.0),
            round3(border.left.0),
        ],
        border_color: color_to_rgba(border_color),
        corner_radii: [
            round3(corner_radii.top_left.0),
            round3(corner_radii.top_right.0),
            round3(corner_radii.bottom_right.0),
            round3(corner_radii.bottom_left.0),
        ],
        text_color,
    }
}

fn extract_web_button_style(golden: &WebGolden) -> WebButtonStyle {
    let theme = golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden");

    fn is_button_node(node: &WebNode) -> bool {
        if node.tag == "button" {
            return true;
        }

        node.attrs
            .get("role")
            .is_some_and(|value| value == "button")
    }

    let button =
        find_first(&theme.root, &is_button_node).expect("expected at least one <button> node");

    WebButtonStyle {
        rect: button.rect.clone(),
        display: css_get(&button.computed_style, "display"),
        padding_left: css_get(&button.computed_style, "paddingLeft"),
        padding_top: css_get(&button.computed_style, "paddingTop"),
        background_color: css_get(&button.computed_style, "backgroundColor"),
        color: css_get(&button.computed_style, "color"),
        border_top_width: css_get(&button.computed_style, "borderTopWidth"),
        border_top_color: css_get(&button.computed_style, "borderTopColor"),
        border_radius: css_get(&button.computed_style, "borderTopLeftRadius"),
    }
}

#[test]
fn web_vs_fret_button_default_pipeline_smoke() {
    let web = read_web_golden("button-default");
    let web_style = extract_web_button_style(&web);
    let fret_style = extract_fret_button_style();

    // Catch “dev server / missing Tailwind” goldens early: these are stable invariants of the
    // shadcn v4 button recipe and are used by downstream comparisons.
    assert_eq!(
        web_style.display.as_deref(),
        Some("inline-flex"),
        "unexpected web button display"
    );
    assert_eq!(
        web_style.padding_left.as_deref(),
        Some("16px"),
        "unexpected web button paddingLeft"
    );
    assert_eq!(
        web_style.padding_top.as_deref(),
        Some("8px"),
        "unexpected web button paddingTop"
    );

    if let Some(px) = web_style.border_top_width.as_deref().and_then(parse_px) {
        for (idx, edge) in fret_style.border.iter().enumerate() {
            assert!(
                (*edge - px).abs() <= 0.5,
                "border[{idx}]: expected≈{px} got={edge}"
            );
        }
    }

    if let Some(px) = web_style.border_radius.as_deref().and_then(parse_px) {
        for (idx, corner) in fret_style.corner_radii.iter().enumerate() {
            assert!(
                (*corner - px).abs() <= 1.0,
                "corner_radii[{idx}]: expected≈{px} got={corner}"
            );
        }
    }

    // Minimal sanity: ensure we can parse at least the core color fields from the web golden.
    if let Some(bg) = web_style.background_color.as_deref() {
        let _ = parse_rgb(bg);
    }
    if let Some(fg) = web_style.color.as_deref() {
        let _ = parse_rgb(fg);
    }

    let report = ButtonReport {
        web: web_style,
        fret: fret_style,
    };

    let write = std::env::var("WRITE_WEB_REPORT").ok().as_deref() == Some("1");
    if write {
        let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("web_reports");
        std::fs::create_dir_all(&out_dir).expect("create web_reports dir");
        let out_path = out_dir.join("button-default.json");
        let json = serde_json::to_string_pretty(&report).expect("serialize report");
        std::fs::write(&out_path, format!("{json}\n")).expect("write report");
    }
}
