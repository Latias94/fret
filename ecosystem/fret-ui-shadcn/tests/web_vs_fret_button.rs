use fret_app::App;
use fret_core::{
    AppWindowId, Event, Modifiers, MouseButton, Point, PointerEvent, PointerType, Px, Rect, Scene,
    SceneOp, Size as CoreSize,
};
use fret_ui::tree::UiTree;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod css_color;
use css_color::{Rgba, color_to_rgba, parse_css_color};

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
    #[serde(default)]
    active: bool,
    rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    computed_style: BTreeMap<String, String>,
    #[serde(default)]
    children: Vec<WebNode>,
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
    opacity: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct FretButtonStyle {
    rect: [f32; 4],
    background: Rgba,
    opacity: f32,
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

fn parse_f32(s: &str) -> Option<f32> {
    s.trim().parse::<f32>().ok()
}

fn round3(v: f32) -> f32 {
    (v * 1000.0).round() / 1000.0
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

fn maybe_write_report(golden_name: &str, report: &ButtonReport) {
    let write = std::env::var("WRITE_WEB_REPORT").ok().as_deref() == Some("1");
    if !write {
        return;
    }

    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("web_reports");
    std::fs::create_dir_all(&out_dir).expect("create web_reports dir");
    let out_path = out_dir.join(format!("{golden_name}.json"));
    let json = serde_json::to_string_pretty(&report).expect("serialize report");
    std::fs::write(&out_path, format!("{json}\n")).expect("write report");
}

fn assert_button_styles_match_web(
    golden_name: &str,
    web_style: WebButtonStyle,
    fret_style: FretButtonStyle,
) {
    // Catch “dev server / missing Tailwind” goldens early: these are stable invariants of the
    // shadcn v4 button recipe and are used by downstream comparisons.
    if golden_name != "button-link" {
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
    }

    if let Some(px) = web_style.border_top_width.as_deref().and_then(parse_px) {
        for (idx, edge) in fret_style.border.iter().enumerate() {
            assert!(
                (*edge - px).abs() <= 0.5,
                "{golden_name} border[{idx}]: expected≈{px} got={edge}"
            );
        }
    }

    if let Some(px) = web_style.border_radius.as_deref().and_then(parse_px) {
        for (idx, corner) in fret_style.corner_radii.iter().enumerate() {
            assert!(
                (*corner - px).abs() <= 1.0,
                "{golden_name} corner_radii[{idx}]: expected≈{px} got={corner}"
            );
        }
    }

    let opacity = web_style
        .opacity
        .as_deref()
        .and_then(parse_f32)
        .unwrap_or(1.0);
    assert!(
        (fret_style.opacity - opacity).abs() <= 0.01,
        "{golden_name} opacity: expected≈{opacity} got={}",
        fret_style.opacity
    );

    if let Some(web_bg) = web_style
        .background_color
        .as_deref()
        .and_then(parse_css_color)
    {
        assert_rgba_close(
            &format!("{golden_name} background"),
            fret_style.background,
            web_bg,
            0.02,
        );
    }

    let border_px = web_style
        .border_top_width
        .as_deref()
        .and_then(parse_px)
        .unwrap_or(0.0);
    if border_px > 0.0 {
        if let Some(web_border) = web_style
            .border_top_color
            .as_deref()
            .and_then(parse_css_color)
        {
            assert_rgba_close(
                &format!("{golden_name} border_color"),
                fret_style.border_color,
                web_border,
                0.02,
            );
        }
    }

    if let Some(web_fg) = web_style.color.as_deref().and_then(parse_css_color) {
        let fret_fg = fret_style
            .text_color
            .unwrap_or_else(|| panic!("{golden_name} missing fret text color"));
        assert_rgba_close(&format!("{golden_name} text_color"), fret_fg, web_fg, 0.02);
    }

    maybe_write_report(
        golden_name,
        &ButtonReport {
            web: web_style,
            fret: fret_style,
        },
    );
}

fn assert_button_variant_matches_web(golden_name: &str, variant: fret_ui_shadcn::ButtonVariant) {
    let web = read_web_golden(golden_name);
    let web_style = extract_web_button_style(&web);
    let fret_style = extract_fret_button_style(variant);

    assert_button_styles_match_web(golden_name, web_style, fret_style);
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

impl fret_core::MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

fn find_button_quad_style(
    scene: &Scene,
    button_bounds: Rect,
) -> (
    Rect,
    fret_core::Color,
    f32,
    fret_core::Edges,
    fret_core::Color,
    fret_core::Corners,
) {
    let mut opacity_stack: Vec<f32> = Vec::new();
    let mut opacity_prod = 1.0f32;

    for op in scene.ops() {
        match *op {
            SceneOp::PushOpacity { opacity } => {
                opacity_stack.push(opacity);
                opacity_prod *= opacity;
            }
            SceneOp::PopOpacity => {
                if let Some(top) = opacity_stack.pop() {
                    opacity_prod /= top;
                }
            }
            SceneOp::Quad {
                rect,
                background,
                border,
                border_paint,
                corner_radii,
                ..
            } if rect == button_bounds => {
                let background = match background {
                    fret_core::Paint::Solid(c) => c,
                    _ => fret_core::Color::TRANSPARENT,
                };
                let border_color = match border_paint {
                    fret_core::Paint::Solid(c) => c,
                    _ => fret_core::Color::TRANSPARENT,
                };
                return (
                    rect,
                    background,
                    opacity_prod,
                    border,
                    border_color,
                    corner_radii,
                );
            }
            _ => {}
        }
    }

    panic!("failed to find button quad in scene (expected a Quad with rect == semantics bounds)");
}

fn extract_fret_button_style(variant: fret_ui_shadcn::ButtonVariant) -> FretButtonStyle {
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
        |cx| {
            vec![
                fret_ui_shadcn::Button::new("Button")
                    .variant(variant)
                    .into_element(cx),
            ]
        },
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

    let (rect, background, opacity, border, border_color, corner_radii) =
        find_button_quad_style(&scene, button_bounds);

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
        opacity,
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

fn extract_fret_button_style_pressed(variant: fret_ui_shadcn::ButtonVariant) -> FretButtonStyle {
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
        "web-vs-fret-button-pressed",
        |cx| {
            vec![
                fret_ui_shadcn::Button::new("Button")
                    .variant(variant)
                    .into_element(cx),
            ]
        },
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

    let center = Point::new(
        Px(button_bounds.origin.x.0 + button_bounds.size.width.0 * 0.5),
        Px(button_bounds.origin.y.0 + button_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: center,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    // Re-render after the interaction so pressable state is reflected in chrome props.
    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-button-pressed",
        |cx| {
            vec![
                fret_ui_shadcn::Button::new("Button")
                    .variant(variant)
                    .into_element(cx),
            ]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let (rect, background, opacity, border, border_color, corner_radii) =
        find_button_quad_style(&scene, button_bounds);

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
        opacity,
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

fn extract_fret_button_style_hovered(variant: fret_ui_shadcn::ButtonVariant) -> FretButtonStyle {
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
        "web-vs-fret-button-hover",
        |cx| {
            vec![
                fret_ui_shadcn::Button::new("Button")
                    .variant(variant)
                    .into_element(cx),
            ]
        },
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

    let center = Point::new(
        Px(button_bounds.origin.x.0 + button_bounds.size.width.0 * 0.5),
        Px(button_bounds.origin.y.0 + button_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: center,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    // Re-render so hover state is reflected in chrome props.
    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-button-hover",
        |cx| {
            vec![
                fret_ui_shadcn::Button::new("Button")
                    .variant(variant)
                    .into_element(cx),
            ]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let (rect, background, opacity, border, border_color, corner_radii) =
        find_button_quad_style(&scene, button_bounds);

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
        opacity,
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

fn extract_fret_button_style_disabled(variant: fret_ui_shadcn::ButtonVariant) -> FretButtonStyle {
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
        "web-vs-fret-button-disabled",
        |cx| {
            vec![
                fret_ui_shadcn::Button::new("Button")
                    .disabled(true)
                    .variant(variant)
                    .into_element(cx),
            ]
        },
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

    let (rect, background, opacity, border, border_color, corner_radii) =
        find_button_quad_style(&scene, button_bounds);

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
        opacity,
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
        opacity: css_get(&button.computed_style, "opacity"),
    }
}

#[test]
fn web_vs_fret_button_default_pipeline_smoke() {
    assert_button_variant_matches_web("button-default", fret_ui_shadcn::ButtonVariant::Default);
}

#[test]
fn web_vs_fret_button_default_pressed_matches_web() {
    let web = read_web_golden("button-default.pressed");
    let web_style = extract_web_button_style(&web);
    let fret_style = extract_fret_button_style_pressed(fret_ui_shadcn::ButtonVariant::Default);
    assert_button_styles_match_web("button-default.pressed", web_style, fret_style);
}

#[test]
fn web_vs_fret_button_default_hover_matches_web() {
    let web = read_web_golden("button-default.hover");
    let web_style = extract_web_button_style(&web);
    let fret_style = extract_fret_button_style_hovered(fret_ui_shadcn::ButtonVariant::Default);
    assert_button_styles_match_web("button-default.hover", web_style, fret_style);
}

#[test]
fn web_vs_fret_button_default_disabled_matches_web() {
    let web = read_web_golden("button-default.disabled");
    let web_style = extract_web_button_style(&web);
    let fret_style = extract_fret_button_style_disabled(fret_ui_shadcn::ButtonVariant::Default);
    assert_button_styles_match_web("button-default.disabled", web_style, fret_style);
}

#[test]
fn web_vs_fret_button_destructive_matches_web() {
    assert_button_variant_matches_web(
        "button-destructive",
        fret_ui_shadcn::ButtonVariant::Destructive,
    );
}

#[test]
fn web_vs_fret_button_outline_matches_web() {
    assert_button_variant_matches_web("button-outline", fret_ui_shadcn::ButtonVariant::Outline);
}

#[test]
fn web_vs_fret_button_secondary_matches_web() {
    assert_button_variant_matches_web("button-secondary", fret_ui_shadcn::ButtonVariant::Secondary);
}

#[test]
fn web_vs_fret_button_ghost_matches_web() {
    assert_button_variant_matches_web("button-ghost", fret_ui_shadcn::ButtonVariant::Ghost);
}

#[test]
fn web_vs_fret_button_link_matches_web() {
    assert_button_variant_matches_web("button-link", fret_ui_shadcn::ButtonVariant::Link);
}
