use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, KeyCode, Modifiers, MouseButton, MouseButtons, Point,
    PointerEvent, PointerType, Px, Rect, Scene, SceneOp, SemanticsRole, Size as CoreSize,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::elements::{GlobalElementId, bounds_for_element, with_element_cx};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use serde::Deserialize;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

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
    #[allow(dead_code)]
    tag: String,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    computed_style: BTreeMap<String, String>,
    #[allow(dead_code)]
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

fn web_golden_path(file_name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(file_name)
}

fn read_web_golden_open(name: &str) -> WebGolden {
    let path = web_golden_path(&format!("{name}.open.json"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web open golden: {}\nerror: {err}\n\nGenerate it via:\n  pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts {name} --modes=open --update --baseUrl=http://localhost:4020\n\nDocs:\n  docs/shadcn-web-goldens.md",
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

fn find_portal_by_role<'a>(theme: &'a WebGoldenTheme, role: &str) -> Option<&'a WebNode> {
    theme
        .portals
        .iter()
        .find(|n| n.attrs.get("role").is_some_and(|v| v == role))
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

fn find_by_data_slot_and_state<'a>(
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

fn parse_px(s: &str) -> Option<f32> {
    let s = s.trim();
    let v = s.strip_suffix("px").unwrap_or(s);
    v.parse::<f32>().ok()
}

fn srgb_f32_to_linear(c_srgb: f32) -> f32 {
    if c_srgb <= 0.04045 {
        c_srgb / 12.92
    } else {
        ((c_srgb + 0.055) / 1.055).powf(2.4)
    }
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

    let r_srgb: f32 = parts[0].parse::<f32>().ok()? / 255.0;
    let g_srgb: f32 = parts[1].parse::<f32>().ok()? / 255.0;
    let b_srgb: f32 = parts[2].parse::<f32>().ok()? / 255.0;
    let a: f32 = if inner.1 {
        parts
            .get(3)
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(1.0)
    } else {
        1.0
    };

    Some(Rgba {
        r: srgb_f32_to_linear(r_srgb.clamp(0.0, 1.0)),
        g: srgb_f32_to_linear(g_srgb.clamp(0.0, 1.0)),
        b: srgb_f32_to_linear(b_srgb.clamp(0.0, 1.0)),
        a,
    })
}

fn parse_oklch(s: &str) -> Option<Rgba> {
    let s = s.trim();
    let inner = s.strip_prefix("oklch(")?.strip_suffix(')')?.trim();

    let (main, alpha_part) = if let Some((l, r)) = inner.split_once('/') {
        (l.trim(), Some(r.trim()))
    } else {
        (inner, None)
    };

    let parts: Vec<&str> = main
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 3 {
        return None;
    }

    let l: f32 = parts[0].trim_end_matches('%').parse().ok()?;
    let c: f32 = parts[1].parse().ok()?;
    let h_raw = parts[2].trim();
    let h: f32 = h_raw
        .trim_end_matches("deg")
        .trim_end_matches("rad")
        .trim_end_matches("turn")
        .parse()
        .ok()?;

    let alpha = if let Some(a) = alpha_part {
        if let Some(pct) = a.trim_end_matches('%').parse::<f32>().ok()
            && a.trim_end().ends_with('%')
        {
            (pct / 100.0).clamp(0.0, 1.0)
        } else {
            a.parse::<f32>().ok()?.clamp(0.0, 1.0)
        }
    } else {
        1.0
    };

    // Assume degrees when unitless. (Our extracted goldens currently use unitless degrees.)
    let h_rad = h.to_radians();
    let a_ = c * h_rad.cos();
    let b_ = c * h_rad.sin();

    // OKLab -> linear sRGB (Björn Ottosson).
    let l_ = l + 0.396_337_78 * a_ + 0.215_803_76 * b_;
    let m_ = l - 0.105_561_346 * a_ - 0.063_854_17 * b_;
    let s_ = l - 0.089_484_18 * a_ - 1.291_485_5 * b_;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    let r = 4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_94 * s3;
    let g = -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_4 * s3;
    let b = -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3;

    Some(Rgba {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: alpha,
    })
}

fn parse_lab(s: &str) -> Option<Rgba> {
    let s = s.trim();
    let inner = s.strip_prefix("lab(")?.strip_suffix(')')?.trim();

    let (main, alpha_part) = if let Some((l, r)) = inner.split_once('/') {
        (l.trim(), Some(r.trim()))
    } else {
        (inner, None)
    };

    let parts: Vec<&str> = main
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 3 {
        return None;
    }

    let l_star: f32 = parts[0].trim_end_matches('%').parse().ok()?;
    let a_star: f32 = parts[1].parse().ok()?;
    let b_star: f32 = parts[2].parse().ok()?;

    let alpha = if let Some(a) = alpha_part {
        if let Some(pct) = a.trim_end_matches('%').parse::<f32>().ok()
            && a.trim_end().ends_with('%')
        {
            (pct / 100.0).clamp(0.0, 1.0)
        } else {
            a.parse::<f32>().ok()?.clamp(0.0, 1.0)
        }
    } else {
        1.0
    };

    // Lab(D50) conversion: match existing web_vs_fret_button.rs behavior (Bradford D50->D65).
    let fy = (l_star + 16.0) / 116.0;
    let fx = fy + a_star / 500.0;
    let fz = fy - b_star / 200.0;

    let eps = 216.0 / 24_389.0;
    let kappa = 24_389.0 / 27.0;

    let fx3 = fx * fx * fx;
    let fz3 = fz * fz * fz;
    let xr = if fx3 > eps {
        fx3
    } else {
        (116.0 * fx - 16.0) / kappa
    };
    let yr = if l_star > kappa * eps {
        fy * fy * fy
    } else {
        l_star / kappa
    };
    let zr = if fz3 > eps {
        fz3
    } else {
        (116.0 * fz - 16.0) / kappa
    };

    // D50 whitepoint.
    let x_d50 = xr * 0.96422;
    let y_d50 = yr * 1.00000;
    let z_d50 = zr * 0.82521;

    // Bradford adaptation D50->D65.
    let x = 0.955_576_6 * x_d50 + -0.023_039_3 * y_d50 + 0.063_163_6 * z_d50;
    let y = -0.028_289_5 * x_d50 + 1.009_941_6 * y_d50 + 0.021_007_7 * z_d50;
    let z = 0.012_298_2 * x_d50 + -0.020_483 * y_d50 + 1.329_909_8 * z_d50;

    // XYZ(D65) -> linear sRGB.
    let r = 3.240_454_2 * x + -1.537_138_5 * y + -0.498_531_4 * z;
    let g = -0.969_266 * x + 1.876_010_8 * y + 0.041_556 * z;
    let b = 0.055_643_4 * x + -0.204_025_9 * y + 1.057_225_2 * z;

    Some(Rgba {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: alpha,
    })
}

fn parse_css_color(s: &str) -> Option<Rgba> {
    parse_rgb(s)
        .or_else(|| parse_lab(s))
        .or_else(|| parse_oklch(s))
}

fn color_to_rgba(c: fret_core::Color) -> Rgba {
    Rgba {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

fn web_border_widths_px(node: &WebNode) -> Option<[f32; 4]> {
    Some([
        node.computed_style
            .get("borderTopWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
        node.computed_style
            .get("borderRightWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
        node.computed_style
            .get("borderBottomWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
        node.computed_style
            .get("borderLeftWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
    ])
}

fn web_corner_radii_effective_px(node: &WebNode) -> Option<[f32; 4]> {
    let max = node.rect.w.min(node.rect.h) * 0.5;
    let radius = |key: &str| {
        node.computed_style
            .get(key)
            .map(String::as_str)
            .and_then(parse_px)
            .map(|v| v.min(max))
    };

    Some([
        radius("borderTopLeftRadius")?,
        radius("borderTopRightRadius")?,
        radius("borderBottomRightRadius")?,
        radius("borderBottomLeftRadius")?,
    ])
}

#[derive(Debug, Clone, Copy)]
struct PaintedQuad {
    #[allow(dead_code)]
    rect: Rect,
    background: fret_core::Color,
    border: [f32; 4],
    border_color: fret_core::Color,
    corners: [f32; 4],
}

fn rect_contains(outer: Rect, inner: Rect) -> bool {
    let eps = 0.01;
    inner.origin.x.0 + eps >= outer.origin.x.0
        && inner.origin.y.0 + eps >= outer.origin.y.0
        && inner.origin.x.0 + inner.size.width.0 <= outer.origin.x.0 + outer.size.width.0 + eps
        && inner.origin.y.0 + inner.size.height.0 <= outer.origin.y.0 + outer.size.height.0 + eps
}

fn has_border(border: &[f32; 4]) -> bool {
    border.iter().any(|v| *v > 0.01)
}

fn rect_area(rect: Rect) -> f32 {
    rect.size.width.0 * rect.size.height.0
}

fn find_best_chrome_quad(scene: &Scene, target: Rect) -> Option<PaintedQuad> {
    let mut best_containing_border: Option<PaintedQuad> = None;
    let mut best_containing_border_area = f32::INFINITY;
    let mut best_containing_background: Option<PaintedQuad> = None;
    let mut best_containing_background_area = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_color,
            ..
        } = *op
        else {
            continue;
        };

        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        let has_background = background.a > 0.01;
        if !has_background && !has_border(&border) {
            continue;
        }

        if rect_contains(rect, target) {
            let area = rect_area(rect);
            let quad = PaintedQuad {
                rect,
                background,
                border,
                border_color,
                corners: [
                    corner_radii.top_left.0,
                    corner_radii.top_right.0,
                    corner_radii.bottom_right.0,
                    corner_radii.bottom_left.0,
                ],
            };
            if has_border(&border) {
                if area < best_containing_border_area {
                    best_containing_border_area = area;
                    best_containing_border = Some(quad);
                }
            } else if area < best_containing_background_area {
                best_containing_background_area = area;
                best_containing_background = Some(quad);
            }
        }
    }

    if best_containing_border.is_some() || best_containing_background.is_some() {
        return best_containing_border.or(best_containing_background);
    }

    // Fallback: if containment matching fails (e.g. semantics bounds already include the border),
    // use a best-effort score match.
    let mut best_border: Option<PaintedQuad> = None;
    let mut best_border_score = f32::INFINITY;
    let mut best_background: Option<PaintedQuad> = None;
    let mut best_background_score = f32::INFINITY;
    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_color,
            ..
        } = *op
        else {
            continue;
        };

        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        let has_background = background.a > 0.01;
        if !has_background && !has_border(&border) {
            continue;
        }

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        let quad = PaintedQuad {
            rect,
            background,
            border,
            border_color,
            corners: [
                corner_radii.top_left.0,
                corner_radii.top_right.0,
                corner_radii.bottom_right.0,
                corner_radii.bottom_left.0,
            ],
        };

        if has_border(&border) {
            if score < best_border_score {
                best_border_score = score;
                best_border = Some(quad);
            }
        } else if score < best_background_score {
            best_background_score = score;
            best_background = Some(quad);
        }
    }

    best_border.or(best_background)
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
        "web-vs-fret-overlay-chrome",
        render,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn paint_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    bounds: Rect,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
    (snap, scene)
}

fn assert_close(label: &str, actual: f32, expected: f32, tol: f32) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected＞{expected} (㊣{tol}) got={actual} (忖={delta})"
    );
}

fn bounds_center(r: Rect) -> Point {
    Point::new(
        Px(r.origin.x.0 + r.size.width.0 * 0.5),
        Px(r.origin.y.0 + r.size.height.0 * 0.5),
    )
}

fn right_click_center(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    center: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: center,
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
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

fn left_click_center(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    center: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

fn assert_overlay_chrome_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = find_portal_by_role(theme, web_portal_role).expect("web portal root by role");
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

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
        |cx| vec![build_frame1(cx, &open)],
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let build_frame2 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_frame2(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = largest_semantics_node(&snap, fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let mut quad =
        find_best_chrome_quad(&scene, overlay.bounds).expect("painted quad for overlay panel");
    if has_border(&web_border) && !has_border(&quad.border) {
        quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
            .unwrap_or_else(|| panic!("painted border quad for overlay panel ({web_name})"));
    }
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn assert_click_overlay_chrome_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_role: SemanticsRole,
    fret_trigger_role: SemanticsRole,
    fret_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = find_portal_by_role(theme, web_portal_role).expect("web portal root by role");
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_trigger_role && n.label.as_deref() == Some(fret_trigger_label))
        .unwrap_or_else(|| {
            panic!(
                "missing trigger semantics node: {fret_trigger_role:?} label={fret_trigger_label:?} for {web_name}"
            )
        });
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let build_frame2 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_frame2(cx)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = largest_semantics_node(&snap, fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let quad =
        find_best_chrome_quad(&scene, overlay.bounds).expect("painted quad for overlay panel");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn assert_context_menu_chrome_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_role: SemanticsRole,
    trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = find_portal_by_role(theme, web_portal_role).expect("web portal root by role");
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let build_frame2 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_frame2(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let quad =
        find_best_chrome_quad(&scene, overlay.bounds).expect("painted quad for overlay panel");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn assert_navigation_menu_content_chrome_matches(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    open_value: &str,
    trigger_label: &str,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_content = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let web_border = web_border_widths_px(web_content).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_content).expect("web radius px");

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &model, &root_id_out)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &model, &root_id_out)],
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = with_element_cx(
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
    .unwrap_or_else(|| panic!("missing fret navigation-menu content id for {open_value}"));

    let target = bounds_for_element(&mut app, window, content_id).unwrap_or_else(|| {
        panic!("missing fret bounds for navigation-menu content id {content_id:?}")
    });

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad =
        find_best_chrome_quad(&scene, target).expect("painted quad for navigation-menu content");

    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn find_best_chrome_quad_by_size(
    scene: &Scene,
    expected_w: f32,
    expected_h: f32,
    expected_border: [f32; 4],
) -> Option<PaintedQuad> {
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_color,
            ..
        } = *op
        else {
            continue;
        };

        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        let has_background = background.a > 0.01;
        if !has_background && !has_border(&border) {
            continue;
        }
        if has_border(&expected_border) && !has_border(&border) {
            continue;
        }

        let w = rect.size.width.0;
        let h = rect.size.height.0;
        let score = (w - expected_w).abs() + (h - expected_h).abs();
        if score < best_score {
            best_score = score;
            best = Some(PaintedQuad {
                rect,
                background,
                border,
                border_color,
                corners: [
                    corner_radii.top_left.0,
                    corner_radii.top_right.0,
                    corner_radii.bottom_right.0,
                    corner_radii.bottom_left.0,
                ],
            });
        }
    }

    best
}

fn assert_overlay_chrome_matches_by_portal_slot(
    web_name: &str,
    web_portal_slot: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = theme
        .portals
        .iter()
        .find(|n| {
            n.attrs
                .get("data-slot")
                .is_some_and(|v| v == web_portal_slot)
        })
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");
    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

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
        |cx| vec![build_frame1(cx, &open)],
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let build_frame2 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_frame2(cx, &open)],
    );

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .expect("painted quad for overlay panel");

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn largest_semantics_node<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes.iter().filter(|n| n.role == role).max_by(|a, b| {
        let a_area = a.bounds.size.width.0 * a.bounds.size.height.0;
        let b_area = b.bounds.size.width.0 * b.bounds.size.height.0;
        a_area
            .partial_cmp(&b_area)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn hover_open_at(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    position: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
}

fn assert_hover_overlay_chrome_matches(
    web_name: &str,
    web_portal_slot: &str,
    fret_role: SemanticsRole,
    fret_trigger_label: &str,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = theme
        .portals
        .iter()
        .find(|n| {
            n.attrs
                .get("data-slot")
                .is_some_and(|v| v == web_portal_slot)
        })
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let trigger_id_out: std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &trigger_id_out)],
    );

    let frame1_snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger_semantics = frame1_snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(fret_trigger_label))
        .unwrap_or_else(|| {
            panic!(
                "missing trigger semantics node: Button label={fret_trigger_label:?} for {web_name}"
            )
        });
    let trigger_center = Point::new(
        Px(trigger_semantics.bounds.origin.x.0 + trigger_semantics.bounds.size.width.0 * 0.5),
        Px(trigger_semantics.bounds.origin.y.0 + trigger_semantics.bounds.size.height.0 * 0.5),
    );

    let trigger_element = trigger_id_out.get().expect("trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("trigger node");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::KeyA,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.set_focus(Some(trigger_node));
    hover_open_at(&mut ui, &mut app, &mut services, trigger_center);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &trigger_id_out)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let overlay = largest_semantics_node(&snap, fret_role).unwrap_or_else(|| {
        let mut roles: Vec<String> = snap.nodes.iter().map(|n| format!("{:?}", n.role)).collect();
        roles.sort();
        roles.dedup();
        panic!("missing fret semantics node: {fret_role:?}; roles={roles:?}");
    });

    let quad = find_best_chrome_quad(&scene, overlay.bounds).expect("painted quad for overlay");

    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_dialog_demo_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_overlay_chrome_matches(
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
fn web_vs_fret_dialog_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_overlay_chrome_matches_by_portal_slot("dialog-demo", "dialog-content", |cx, open| {
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
    });
}

#[test]
fn web_vs_fret_command_dialog_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    assert_overlay_chrome_matches(
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
fn web_vs_fret_alert_dialog_demo_panel_chrome_matches() {
    use fret_ui_shadcn::{AlertDialog, AlertDialogContent, Button, ButtonVariant};

    assert_overlay_chrome_matches(
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
fn web_vs_fret_sheet_demo_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_overlay_chrome_matches("sheet-demo", "dialog", SemanticsRole::Dialog, |cx, open| {
        Sheet::new(open.clone()).into_element(
            cx,
            |cx| {
                Button::new("Open")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
        )
    });
}

#[test]
fn web_vs_fret_sheet_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_overlay_chrome_matches_by_portal_slot("sheet-demo", "sheet-content", |cx, open| {
        Sheet::new(open.clone()).into_element(
            cx,
            |cx| {
                Button::new("Open")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
        )
    });
}

#[test]
fn web_vs_fret_sheet_side_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_overlay_chrome_matches("sheet-side", "dialog", SemanticsRole::Dialog, |cx, open| {
        Sheet::new(open.clone()).side(SheetSide::Top).into_element(
            cx,
            |cx| {
                Button::new("top")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
        )
    });
}

#[test]
fn web_vs_fret_sheet_side_right_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_overlay_chrome_matches(
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
fn web_vs_fret_sheet_side_bottom_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_overlay_chrome_matches(
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
fn web_vs_fret_sheet_side_left_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_overlay_chrome_matches(
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

#[test]
fn web_vs_fret_popover_panel_chrome_matches() {
    assert_overlay_chrome_matches(
        "popover-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            fret_ui_shadcn::Popover::new(open.clone()).into_element(
                cx,
                |cx| fret_ui_shadcn::Button::new("Open").into_element(cx),
                |cx| fret_ui_shadcn::PopoverContent::new(Vec::new()).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_panel_chrome_matches() {
    assert_overlay_chrome_matches(
        "dropdown-menu-demo",
        "menu",
        SemanticsRole::Menu,
        |cx, open| {
            fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
                cx,
                |cx| fret_ui_shadcn::Button::new("Open").into_element(cx),
                |_cx| {
                    vec![fret_ui_shadcn::DropdownMenuEntry::Item(
                        fret_ui_shadcn::DropdownMenuItem::new("Alpha"),
                    )]
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_select_panel_chrome_matches() {
    assert_overlay_chrome_matches(
        "select-scrollable",
        "listbox",
        SemanticsRole::ListBox,
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .item(fret_ui_shadcn::SelectItem::new("one", "One"))
                .item(fret_ui_shadcn::SelectItem::new("two", "Two"))
                .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_tooltip_panel_chrome_matches() {
    assert_hover_overlay_chrome_matches(
        "tooltip-demo",
        "tooltip-content",
        SemanticsRole::Tooltip,
        "Hover",
        |cx, trigger| {
            let trigger_el = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx);
            trigger.set(Some(trigger_el.id));

            let content_el = fret_ui_shadcn::TooltipContent::new(vec![cx.text("Add to library")])
                .into_element(cx);

            fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_hover_card_panel_chrome_matches() {
    assert_overlay_chrome_matches_by_portal_slot(
        "hover-card-demo",
        "hover-card-content",
        |cx, open| {
            let trigger_el = fret_ui_shadcn::Button::new("@nextjs")
                .variant(fret_ui_shadcn::ButtonVariant::Link)
                .into_element(cx);
            let content_el =
                fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")]).into_element(cx);

            fret_ui_shadcn::HoverCard::new(trigger_el, content_el)
                .open(Some(open.clone()))
                .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_context_menu_panel_chrome_matches() {
    assert_context_menu_chrome_matches(
        "context-menu-demo",
        "menu",
        SemanticsRole::Menu,
        "Right click here",
        |cx, open| {
            fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
                cx,
                |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                |_cx| {
                    vec![fret_ui_shadcn::ContextMenuEntry::Item(
                        fret_ui_shadcn::ContextMenuItem::new("Copy"),
                    )]
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_menubar_panel_chrome_matches() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    assert_click_overlay_chrome_matches(
        "menubar-demo",
        "menu",
        SemanticsRole::Menu,
        SemanticsRole::MenuItem,
        "File",
        |cx| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Item(MenubarItem::new("New Tab")),
                MenubarEntry::Item(MenubarItem::new("New Window")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Share")),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_panel_chrome_matches() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_content_chrome_matches(
        "navigation-menu-demo",
        "navigation-menu-content",
        "open",
        "home",
        "Home",
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
