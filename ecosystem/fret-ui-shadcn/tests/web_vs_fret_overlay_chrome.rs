use fret_app::App;
use fret_core::{
    AppWindowId, Color, Event, FrameId, KeyCode, Modifiers, MouseButton, MouseButtons, Point,
    PointerEvent, PointerType, Px, Rect, Scene, SceneOp, SemanticsRole, Size as CoreSize,
    Transform2D,
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

mod css_color;
use css_color::{color_to_rgba, parse_css_color};

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
    #[allow(dead_code)]
    tag: String,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    #[serde(default)]
    active: bool,
    rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    computed_style: BTreeMap<String, String>,
    #[serde(default)]
    text: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    children: Vec<WebNode>,
}

fn web_theme_named<'a>(golden: &'a WebGolden, name: &str) -> &'a WebGoldenTheme {
    golden
        .themes
        .get(name)
        .unwrap_or_else(|| panic!("missing {name} theme in web golden"))
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

fn find_portal_by_slot<'a>(theme: &'a WebGoldenTheme, slot: &str) -> Option<&'a WebNode> {
    theme
        .portals
        .iter()
        .find(|n| n.attrs.get("data-slot").is_some_and(|v| v == slot))
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

fn split_box_shadow_layers(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0_u32;
    let mut start = 0_usize;
    for (idx, ch) in s.char_indices() {
        match ch {
            '(' => depth = depth.saturating_add(1),
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                out.push(s[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        out.push(s[start..].trim());
    }
    out.into_iter().filter(|p| !p.is_empty()).collect()
}

fn parse_box_shadow_layer(layer: &str) -> Option<(String, f32, f32, f32, f32)> {
    let layer = layer.trim();
    if layer.is_empty() || layer == "none" {
        return None;
    }

    let (color, rest) = if layer.starts_with('#') {
        let mut it = layer.splitn(2, char::is_whitespace);
        let color = it.next()?.trim().to_string();
        (color, it.next().unwrap_or("").trim())
    } else if let Some(paren) = layer.find('(') {
        let mut depth = 0_u32;
        let mut end = None;
        for (idx, ch) in layer.char_indices().skip(paren) {
            match ch {
                '(' => depth = depth.saturating_add(1),
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = Some(idx);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end?;
        let color = layer[..=end].trim().to_string();
        (color, layer[end + 1..].trim())
    } else {
        let mut it = layer.splitn(2, char::is_whitespace);
        let color = it.next()?.trim().to_string();
        (color, it.next().unwrap_or("").trim())
    };

    let parts: Vec<&str> = rest.split_whitespace().filter(|p| !p.is_empty()).collect();
    if parts.len() < 4 {
        return None;
    }
    let x = parse_px(parts[0])?;
    let y = parse_px(parts[1])?;
    let blur = parse_px(parts[2])?;
    let spread = parse_px(parts[3])?;
    Some((color, x, y, blur, spread))
}

#[derive(Debug, Clone, Copy)]
struct ShadowInsets {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

fn shadow_insets_for_rect(panel: Rect, shadow: Rect) -> ShadowInsets {
    let panel_right = panel.origin.x.0 + panel.size.width.0;
    let panel_bottom = panel.origin.y.0 + panel.size.height.0;
    let shadow_right = shadow.origin.x.0 + shadow.size.width.0;
    let shadow_bottom = shadow.origin.y.0 + shadow.size.height.0;

    ShadowInsets {
        left: shadow.origin.x.0 - panel.origin.x.0,
        top: shadow.origin.y.0 - panel.origin.y.0,
        right: shadow_right - panel_right,
        bottom: shadow_bottom - panel_bottom,
    }
}

fn shadow_insets_for_box_shadow_layer(x: f32, y: f32, blur: f32, spread: f32) -> ShadowInsets {
    let delta = spread + blur;
    ShadowInsets {
        left: x - delta,
        top: y - delta,
        right: x + delta,
        bottom: y + delta,
    }
}

fn shadow_insets_score(a: ShadowInsets, b: ShadowInsets) -> f32 {
    (a.left - b.left).abs()
        + (a.top - b.top).abs()
        + (a.right - b.right).abs()
        + (a.bottom - b.bottom).abs()
}

fn rect_intersection_area(a: Rect, b: Rect) -> f32 {
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
        // Ignore drop-shadow layers when looking for "surface chrome".
        if !has_border(&border) && background.a < 0.5 {
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
        // Ignore drop-shadow layers when looking for "surface chrome".
        if !has_border(&border) && background.a < 0.5 {
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
                    ' ' => 0.28,
                    '(' | ')' => 0.28,
                    'i' | 'l' | 'I' | 't' | 'f' | 'j' | 'r' => 0.32,
                    'm' | 'w' | 'M' | 'W' => 0.75,
                    'o' | 'O' | 'p' | 'P' => 0.62,
                    'A'..='Z' => 0.62,
                    'a'..='z' => 0.56,
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
                baseline: Px((line_height.0 * 0.8).max(0.0)),
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

fn setup_app_with_shadcn_theme_scheme(
    app: &mut App,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        scheme,
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

fn bounds_for_viewport(viewport: WebViewport) -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(viewport.w), Px(viewport.h)),
    )
}

fn assert_close(label: &str, actual: f32, expected: f32, tol: f32) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected＞{expected} (㊣{tol}) got={actual} (忖={delta})"
    );
}

fn assert_rgba_close(label: &str, actual: css_color::Rgba, expected: css_color::Rgba, tol: f32) {
    assert_close(&format!("{label}.r"), actual.r, expected.r, tol);
    assert_close(&format!("{label}.g"), actual.g, expected.g, tol);
    assert_close(&format!("{label}.b"), actual.b, expected.b, tol);
    assert_close(&format!("{label}.a"), actual.a, expected.a, tol);
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
            is_click: true,
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
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

fn dispatch_key_down(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    key: KeyCode,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::KeyDown {
            key,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
}

fn dispatch_key_up(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    key: KeyCode,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::KeyUp {
            key,
            modifiers: Modifiers::default(),
        },
    );
}

fn dispatch_key_press(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    key: KeyCode,
) {
    dispatch_key_down(ui, app, services, key);
    dispatch_key_up(ui, app, services, key);
}

fn leftish_text_probe_point(bounds: Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + 40.0),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
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

fn assert_click_overlay_surface_colors_match_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_trigger_role: SemanticsRole,
    fret_trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
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

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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

    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
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
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
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
    let _trigger_bounds = trigger.bounds;
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

fn assert_navigation_menu_content_surface_colors_match(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    open_value: &str,
    trigger_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_content = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let web_background = web_content
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_content).expect("web border widths px");
    let web_border_color = web_content
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
    let _trigger_bounds = trigger.bounds;
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
        "web-vs-fret-nav-menu-surface-colors",
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

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
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
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_navigation_menu_content_shadow_insets_match(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    open_value: &str,
    trigger_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_content = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let expected = web_drop_shadow_insets(web_content);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
    let _trigger_bounds = trigger.bounds;
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
        "web-vs-fret-nav-menu-shadow-insets",
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

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn assert_navigation_menu_viewport_shadow_insets_match(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    trigger_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_viewport = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let expected = web_drop_shadow_insets(web_viewport);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
    let _trigger_bounds = trigger.bounds;
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
    let panel_id = with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-panel-id",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("missing fret navigation-menu viewport panel id");

    let target = bounds_for_element(&mut app, window, panel_id).unwrap_or_else(|| {
        panic!("missing fret bounds for navigation-menu viewport panel id {panel_id:?}")
    });

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad(&scene, target)
        .expect("painted quad for navigation-menu viewport panel");

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn assert_navigation_menu_viewport_surface_colors_match(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    trigger_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_viewport = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let web_background = web_viewport
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_viewport).expect("web border widths px");
    let web_border_color = web_viewport
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
    let panel_id = with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-surface-colors",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("missing fret navigation-menu viewport panel id");

    let target = bounds_for_element(&mut app, window, panel_id).unwrap_or_else(|| {
        panic!("missing fret bounds for navigation-menu viewport panel id {panel_id:?}")
    });

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad(&scene, target)
        .expect("painted quad for navigation-menu viewport panel");

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_background.a"),
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
            &format!("{web_name} {web_theme_name} viewport_border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_navigation_menu_indicator_shadow_insets_match(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    trigger_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_indicator = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let web_diamond = find_first(web_indicator, &|n| {
        let box_shadow = n
            .computed_style
            .get("boxShadow")
            .map(String::as_str)
            .unwrap_or("");
        !box_shadow.is_empty() && box_shadow != "none"
    })
    .expect("missing web indicator diamond node (expected non-empty boxShadow)");

    let expected = web_drop_shadow_insets(web_diamond);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let root_id = root_id_out.get().expect("navigation menu root id");
    let diamond_id = with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-indicator-diamond-id",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_indicator_diamond_id(
                cx, root_id,
            )
        },
    )
    .expect("missing fret navigation-menu indicator diamond id");
    let diamond_bounds = bounds_for_element(&mut app, window, diamond_id).unwrap_or_else(|| {
        panic!("missing fret bounds for navigation-menu indicator diamond id {diamond_id:?}")
    });
    let near = bounds_center(diamond_bounds);
    let panel_rect = find_best_solid_quad_near_point(&scene, near)
        .expect("painted quad for navigation-menu indicator diamond");

    let candidates = fret_drop_shadow_insets_candidates(&scene, panel_rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
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
        // Ignore drop-shadow layers when looking for "surface chrome".
        if !has_border(&border) && background.a < 0.5 {
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

fn find_best_solid_quad_near_point(scene: &Scene, near: Point) -> Option<Rect> {
    let mut best: Option<Rect> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            background,
            border,
            ..
        } = *op
        else {
            continue;
        };

        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        if has_border(&border) {
            continue;
        }
        if background.a < 0.01 {
            continue;
        }

        let w = rect.size.width.0;
        let h = rect.size.height.0;
        if !(6.0..=20.0).contains(&w) || !(6.0..=20.0).contains(&h) {
            continue;
        }

        let cx = rect.origin.x.0 + w * 0.5;
        let cy = rect.origin.y.0 + h * 0.5;
        let dist_score = (cx - near.x.0).abs() + (cy - near.y.0).abs();
        let area_score = (w * h) * 0.02;
        let score = dist_score + area_score;

        if score < best_score {
            best_score = score;
            best = Some(rect);
        }
    }

    best
}

fn find_best_solid_quad_within_matching_bg(
    scene: &Scene,
    within: Rect,
    expected_bg: css_color::Rgba,
) -> Option<PaintedQuad> {
    let mut best_raw: Option<PaintedQuad> = None;
    let mut best_raw_score = f32::INFINITY;
    let mut best_tx: Option<PaintedQuad> = None;
    let mut best_tx_score = f32::INFINITY;

    scene_walk(scene, |st, op| {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_color,
            ..
        } = *op
        else {
            return;
        };

        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        if has_border(&border) {
            return;
        }
        let background = color_with_opacity(background, st.opacity);
        let border_color = color_with_opacity(border_color, st.opacity);
        if background.a <= 0.01 {
            return;
        }

        let rect_raw = rect;
        let rect_tx = transform_rect_bounds(st.transform, rect);

        let bg = color_to_rgba(background);
        let score = (bg.r - expected_bg.r).abs()
            + (bg.g - expected_bg.g).abs()
            + (bg.b - expected_bg.b).abs()
            + (bg.a - expected_bg.a).abs();

        if rect_contains(within, rect_tx) && score < best_tx_score {
            best_tx_score = score;
            best_tx = Some(PaintedQuad {
                rect: rect_tx,
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
        if rect_contains(within, rect_raw) && score < best_raw_score {
            best_raw_score = score;
            best_raw = Some(PaintedQuad {
                rect: rect_raw,
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
    });

    best_tx.or(best_raw)
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

fn assert_overlay_chrome_matches_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
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

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(web_w.max(640.0)), Px(web_h.max(480.0))),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .expect("painted quad for overlay panel");

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
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
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
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

fn assert_overlay_surface_colors_match(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));

    let bounds = web
        .themes
        .get(web_theme_name)
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(640.0), Px(480.0)),
            )
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
    for tick in 0..settle_frames.max(1) {
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames.max(1),
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = largest_semantics_node(&snap, fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));
    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
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
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn web_drop_shadow_insets(node: &WebNode) -> Vec<ShadowInsets> {
    let box_shadow = node
        .computed_style
        .get("boxShadow")
        .map(String::as_str)
        .unwrap_or("");
    if box_shadow.is_empty() || box_shadow == "none" {
        return Vec::new();
    }

    let mut out = Vec::new();
    for layer in split_box_shadow_layers(box_shadow) {
        let Some((color, x, y, blur, spread)) = parse_box_shadow_layer(layer) else {
            continue;
        };
        if let Some(rgba) = parse_css_color(&color)
            && rgba.a <= 0.01
        {
            continue;
        }
        if x.abs() <= 0.01 && y.abs() <= 0.01 && blur.abs() <= 0.01 && spread.abs() <= 0.01 {
            continue;
        }
        out.push(shadow_insets_for_box_shadow_layer(x, y, blur, spread));
    }
    out
}

fn find_by_data_slot<'a>(node: &'a WebNode, slot: &str) -> Option<&'a WebNode> {
    find_first(node, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
    })
}

fn web_find_highlighted_menu_item_background(theme: &WebGoldenTheme) -> css_color::Rgba {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        let is_menuitem = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "menuitem");
        let is_item_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str().ends_with("-item"));
        if is_menuitem && is_item_slot {
            if let Some(bg) = node
                .computed_style
                .get("backgroundColor")
                .map(String::as_str)
                .and_then(parse_css_color)
                && bg.a > 0.01
            {
                out.push(node);
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    for portal in &theme.portals {
        collect(portal, &mut candidates);
    }
    let highlighted = candidates
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .expect("web highlighted menuitem (data-slot ends_with '-item')");
    highlighted
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web highlighted menuitem backgroundColor")
}

#[derive(Debug, Clone, Copy)]
struct WebHighlightedNodeChrome {
    bg: css_color::Rgba,
    fg: css_color::Rgba,
}

fn web_find_active_element<'a>(theme: &'a WebGoldenTheme) -> &'a WebNode {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if node.active {
            out.push(node);
        }
        for child in &node.children {
            collect(child, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    collect(&theme.root, &mut candidates);
    for portal in &theme.portals {
        collect(portal, &mut candidates);
    }
    for wrapper in &theme.portal_wrappers {
        collect(wrapper, &mut candidates);
    }

    candidates
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .expect("web activeElement")
}

fn web_find_active_element_chrome(theme: &WebGoldenTheme) -> WebHighlightedNodeChrome {
    let active = web_find_active_element(theme);

    let bg = active
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web active element backgroundColor");
    let fg = active
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web active element color");

    WebHighlightedNodeChrome { bg, fg }
}

fn web_find_highlighted_listbox_option_chrome(
    theme: &WebGoldenTheme,
    item_slot: &str,
) -> WebHighlightedNodeChrome {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(node: &'a WebNode, item_slot: &str, out: &mut Vec<&'a WebNode>) {
        let is_option = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "option");
        let is_item_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == item_slot);
        if is_option && is_item_slot {
            if let Some(bg) = node
                .computed_style
                .get("backgroundColor")
                .map(String::as_str)
                .and_then(parse_css_color)
                && bg.a > 0.01
            {
                out.push(node);
            }
        }
        for child in &node.children {
            collect(child, item_slot, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    for portal in &theme.portals {
        collect(portal, item_slot, &mut candidates);
    }
    let highlighted = candidates
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .unwrap_or_else(|| panic!("web highlighted option (data-slot={item_slot})"));

    let bg = highlighted
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web highlighted option backgroundColor");
    let fg = highlighted
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web highlighted option color");

    WebHighlightedNodeChrome { bg, fg }
}

fn rect_contains_point_with_margin(rect: Rect, point: Point, margin_px: f32) -> bool {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = rect.origin.x.0 + rect.size.width.0;
    let y1 = rect.origin.y.0 + rect.size.height.0;
    point.x.0 >= x0 - margin_px
        && point.x.0 <= x1 + margin_px
        && point.y.0 >= y0 - margin_px
        && point.y.0 <= y1 + margin_px
}

#[derive(Clone, Copy)]
struct SceneWalkState {
    transform: Transform2D,
    opacity: f32,
}

fn scene_walk(scene: &Scene, mut f: impl FnMut(SceneWalkState, &SceneOp)) {
    let mut transform_stack: Vec<Transform2D> = Vec::new();
    let mut opacity_stack: Vec<f32> = Vec::new();
    let mut st = SceneWalkState {
        transform: Transform2D::IDENTITY,
        opacity: 1.0,
    };

    for op in scene.ops() {
        match *op {
            SceneOp::PushTransform { transform } => {
                transform_stack.push(st.transform);
                st.transform = st.transform.compose(transform);
            }
            SceneOp::PopTransform => {
                st.transform = transform_stack.pop().unwrap_or(Transform2D::IDENTITY);
            }
            SceneOp::PushOpacity { opacity } => {
                opacity_stack.push(st.opacity);
                st.opacity *= opacity;
            }
            SceneOp::PopOpacity => {
                st.opacity = opacity_stack.pop().unwrap_or(1.0);
            }
            _ => f(st, op),
        }
    }
}

fn transform_rect_bounds(transform: Transform2D, rect: Rect) -> Rect {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;

    let p0 = transform.apply_point(Point::new(Px(x0), Px(y0)));
    let p1 = transform.apply_point(Point::new(Px(x1), Px(y0)));
    let p2 = transform.apply_point(Point::new(Px(x0), Px(y1)));
    let p3 = transform.apply_point(Point::new(Px(x1), Px(y1)));

    let min_x = p0.x.0.min(p1.x.0).min(p2.x.0).min(p3.x.0);
    let max_x = p0.x.0.max(p1.x.0).max(p2.x.0).max(p3.x.0);
    let min_y = p0.y.0.min(p1.y.0).min(p2.y.0).min(p3.y.0);
    let max_y = p0.y.0.max(p1.y.0).max(p2.y.0).max(p3.y.0);

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        CoreSize::new(Px(max_x - min_x), Px(max_y - min_y)),
    )
}

fn color_with_opacity(color: Color, opacity: f32) -> Color {
    Color {
        a: (color.a * opacity).clamp(0.0, 1.0),
        ..color
    }
}

fn find_best_text_color_near(
    scene: &Scene,
    search_within: Rect,
    near: Point,
) -> Option<css_color::Rgba> {
    let mut best_raw: Option<css_color::Rgba> = None;
    let mut best_raw_score = f32::INFINITY;
    let mut best_tx: Option<css_color::Rgba> = None;
    let mut best_tx_score = f32::INFINITY;

    scene_walk(scene, |st, op| {
        let SceneOp::Text { origin, color, .. } = *op else {
            return;
        };
        let raw_origin = origin;
        let tx_origin = st.transform.apply_point(origin);
        let rgba = color_to_rgba(color_with_opacity(color, st.opacity));
        if rgba.a <= 0.01 {
            return;
        }

        if rect_contains_point_with_margin(search_within, tx_origin, 10.0) {
            let dist_score = (tx_origin.x.0 - near.x.0).abs() + (tx_origin.y.0 - near.y.0).abs();
            if dist_score < best_tx_score {
                best_tx_score = dist_score;
                best_tx = Some(rgba);
            }
        }
        if rect_contains_point_with_margin(search_within, raw_origin, 10.0) {
            let dist_score = (raw_origin.x.0 - near.x.0).abs() + (raw_origin.y.0 - near.y.0).abs();
            if dist_score < best_raw_score {
                best_raw_score = dist_score;
                best_raw = Some(rgba);
            }
        }
    });

    best_tx.or(best_raw)
}

fn fret_drop_shadow_insets_candidates(scene: &Scene, panel_rect: Rect) -> Vec<ShadowInsets> {
    let panel_area = rect_area(panel_rect).max(1.0);
    let mut out = Vec::new();

    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            background,
            border,
            ..
        } = *op
        else {
            continue;
        };

        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        if has_border(&border) {
            continue;
        }
        // `shadow-lg` can push the outermost layer alpha below 0.01 (e.g. 0.1 / 16 = 0.00625),
        // but we still need to capture the full footprint for 1:1 `box-shadow` geometry gates.
        if background.a <= 0.001 || background.a >= 0.95 {
            continue;
        }
        if rect_intersection_area(rect, panel_rect) / panel_area <= 0.10 {
            continue;
        }

        out.push(shadow_insets_for_rect(panel_rect, rect));
    }

    out
}

fn assert_shadow_insets_match(
    web_name: &str,
    web_theme_name: &str,
    expected: &[ShadowInsets],
    candidates: &[ShadowInsets],
) {
    if expected.is_empty() {
        return;
    }
    assert!(
        candidates.len() >= expected.len(),
        "{web_name} {web_theme_name}: not enough shadow candidates (expected ≥{}, got {})",
        expected.len(),
        candidates.len()
    );

    let chosen: Vec<ShadowInsets> = match expected.len() {
        1 => {
            let exp = expected[0];
            let mut best = candidates[0];
            let mut best_score = f32::INFINITY;
            for cand in candidates {
                let score = shadow_insets_score(*cand, exp);
                if score < best_score {
                    best_score = score;
                    best = *cand;
                }
            }
            vec![best]
        }
        2 => {
            let exp0 = expected[0];
            let exp1 = expected[1];
            let mut best0 = candidates[0];
            let mut best1 = candidates[1];
            let mut best_score = f32::INFINITY;

            for (i, cand0) in candidates.iter().enumerate() {
                for (j, cand1) in candidates.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let score =
                        shadow_insets_score(*cand0, exp0) + shadow_insets_score(*cand1, exp1);
                    if score < best_score {
                        best_score = score;
                        best0 = *cand0;
                        best1 = *cand1;
                    }
                }
            }

            vec![best0, best1]
        }
        n => panic!("{web_name} {web_theme_name}: unsupported shadow layer count {n}"),
    };

    let tol = 1.0;
    for (idx, (exp, act)) in expected.iter().zip(chosen.iter()).enumerate() {
        assert_close(
            &format!("{web_name} {web_theme_name} shadow[{idx}] left"),
            act.left,
            exp.left,
            tol,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} shadow[{idx}] top"),
            act.top,
            exp.top,
            tol,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} shadow[{idx}] right"),
            act.right,
            exp.right,
            tol,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} shadow[{idx}] bottom"),
            act.bottom,
            exp.bottom,
            tol,
        );
    }
}

fn assert_overlay_shadow_insets_match(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let expected = web_drop_shadow_insets(web_portal);

    let bounds = web
        .themes
        .get(web_theme_name)
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(640.0), Px(480.0)),
            )
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
    for tick in 0..settle_frames.max(1) {
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames.max(1),
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = largest_semantics_node(&snap, fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));
    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_trigger_role: SemanticsRole,
    fret_trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let expected = web_drop_shadow_insets(web_portal);
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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

    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn assert_context_menu_shadow_insets_match(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let expected = web_drop_shadow_insets(web_portal);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = web
        .themes
        .get(web_theme_name)
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(640.0), Px(480.0)),
            )
        });

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
    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
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

fn assert_hover_overlay_surface_colors_match_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    fret_trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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

    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
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

    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .or_else(|| find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border))
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
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
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    hover_open_at(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(submenu_trigger.bounds),
    );

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
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
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    ui.set_focus(Some(submenu_trigger.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
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
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let expected = web_drop_shadow_insets(web_portal);
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    ui.set_focus(Some(submenu_trigger.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
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
fn web_vs_fret_dialog_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_overlay_chrome_matches_by_portal_slot_theme(
        "dialog-demo",
        "dialog-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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
fn web_vs_fret_sheet_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_overlay_surface_colors_match(
        "sheet-demo",
        "sheet-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_500 + 2,
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
fn web_vs_fret_popover_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ButtonVariant, Popover, PopoverContent};

    assert_overlay_surface_colors_match(
        "popover-demo",
        "popover-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            Popover::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open popover")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    PopoverContent::new(Vec::new())
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(fret_ui_kit::MetricRef::Px(Px(320.0)))
                                .h_px(fret_ui_kit::MetricRef::Px(Px(245.33334))),
                        )
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_popover_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ButtonVariant, Popover, PopoverContent};

    assert_overlay_surface_colors_match(
        "popover-demo",
        "popover-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            Popover::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open popover")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    PopoverContent::new(Vec::new())
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(fret_ui_kit::MetricRef::Px(Px(320.0)))
                                .h_px(fret_ui_kit::MetricRef::Px(Px(245.33334))),
                        )
                        .into_element(cx)
                },
            )
        },
    );
}

fn build_shadcn_dropdown_menu_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
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
                            .trailing(DropdownMenuShortcut::new("⇧⌘P").into_element(cx)),
                    ),
                ]
            },
        )
}

#[test]
fn web_vs_fret_dropdown_menu_demo_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

fn assert_dropdown_menu_demo_highlighted_item_background_matches_web(web_theme_name: &str) {
    let web = read_web_golden_open("dropdown-menu-demo.highlight-first");
    let theme = web_theme_named(&web, web_theme_name);
    let expected_bg = web_find_highlighted_menu_item_background(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    let scheme = match web_theme_name {
        "dark" => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        _ => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    };
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let item = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Profile"))
        .expect("dropdown-menu hovered item semantics (Profile)");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: bounds_center(item.bounds),
            buttons: MouseButtons::default(),
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
        FrameId(2 + settle_frames),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let item = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Profile"))
        .expect("dropdown-menu hovered item semantics (Profile)");
    let quad = find_best_solid_quad_within_matching_bg(&scene, item.bounds, expected_bg)
        .unwrap_or_else(|| {
            panic!("painted quad for dropdown-menu highlighted menuitem background")
        });
    assert_rgba_close(
        &format!("dropdown-menu-demo {web_theme_name} highlighted menuitem background"),
        color_to_rgba(quad.background),
        expected_bg,
        0.03,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_highlighted_item_background_matches_web() {
    assert_dropdown_menu_demo_highlighted_item_background_matches_web("light");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_highlighted_item_background_matches_web_dark() {
    assert_dropdown_menu_demo_highlighted_item_background_matches_web("dark");
}

fn assert_dropdown_menu_demo_focused_item_chrome_matches_web(web_theme_name: &str) {
    let web = read_web_golden_open("dropdown-menu-demo.focus-first");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    let scheme = match web_theme_name {
        "dark" => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        _ => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    };
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );

    let (snap, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Open"))
        .expect("dropdown-menu trigger semantics (Open)");
    ui.set_focus(Some(trigger.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let focused = snap
        .nodes
        .iter()
        .find(|n| n.flags.focused)
        .expect("focused semantics node");
    assert_eq!(
        focused.role,
        SemanticsRole::MenuItem,
        "expected focused node to be a menu item after ArrowDown open",
    );

    let quad = find_best_solid_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("dropdown-menu-demo {web_theme_name}: focused item background quad")
        });
    assert_rgba_close(
        &format!("dropdown-menu-demo {web_theme_name} focused item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| panic!("dropdown-menu-demo {web_theme_name}: focused item text color"));
    assert_rgba_close(
        &format!("dropdown-menu-demo {web_theme_name} focused item text color"),
        text,
        expected.fg,
        0.03,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_focused_item_chrome_matches_web() {
    assert_dropdown_menu_demo_focused_item_chrome_matches_web("light");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_focused_item_chrome_matches_web_dark() {
    assert_dropdown_menu_demo_focused_item_chrome_matches_web("dark");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "dropdown-menu-demo.vp1440x320",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "dropdown-menu-demo.vp1440x320",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "dropdown-menu-demo.vp1440x240",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "dropdown-menu-demo.vp1440x240",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "dropdown-menu-demo.vp1440x320",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "dropdown-menu-demo.vp1440x320",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "dropdown-menu-demo.vp1440x240",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "dropdown-menu-demo.vp1440x240",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}

fn build_shadcn_button_group_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui::element::{ContainerProps, LayoutStyle, Length};
    use fret_ui_shadcn::{
        Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
        DropdownMenuGroup, DropdownMenuItem, DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
    };

    fn icon_stub(cx: &mut ElementContext<'_, App>) -> AnyElement {
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(16.0));
                    layout.size.height = Length::Px(Px(16.0));
                    layout
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }

    #[derive(Default)]
    struct Models {
        label_value: Option<Model<Option<Arc<str>>>>,
    }

    let existing = cx.with_state(Models::default, |st| st.label_value.as_ref().cloned());
    let label_value = if let Some(existing) = existing {
        existing
    } else {
        let model: Model<Option<Arc<str>>> =
            cx.app.models_mut().insert(Some(Arc::from("personal")));
        cx.with_state(Models::default, |st| st.label_value = Some(model.clone()));
        model
    };

    DropdownMenu::new(open.clone())
        .align(DropdownMenuAlign::End)
        .min_width(Px(208.0))
        .into_element(
            cx,
            |cx| {
                Button::new("More Options")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Icon)
                    .children([icon_stub(cx)])
                    .into_element(cx)
            },
            |cx| {
                vec![
                    DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Mark as Read").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Archive").leading(icon_stub(cx)),
                        ),
                    ])),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Snooze").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Add to Calendar").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Add to List").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Label As...")
                                .leading(icon_stub(cx))
                                .submenu(vec![DropdownMenuEntry::RadioGroup(
                                    DropdownMenuRadioGroup::new(label_value.clone())
                                        .item(DropdownMenuRadioItemSpec::new(
                                            "personal", "Personal",
                                        ))
                                        .item(DropdownMenuRadioItemSpec::new("work", "Work"))
                                        .item(DropdownMenuRadioItemSpec::new("other", "Other")),
                                )]),
                        ),
                    ])),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Trash")
                                .leading(icon_stub(cx))
                                .variant(
                                fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                            ),
                        ),
                    ])),
                ]
            },
        )
}

#[test]
fn web_vs_fret_button_group_demo_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "button-group-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_button_group_demo,
    );
}

#[test]
fn web_vs_fret_button_group_demo_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "button-group-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_button_group_demo,
    );
}

#[test]
fn web_vs_fret_button_group_demo_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "button-group-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_button_group_demo,
    );
}

#[test]
fn web_vs_fret_button_group_demo_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "button-group-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_button_group_demo,
    );
}

#[test]
fn web_vs_fret_button_group_demo_submenu_kbd_surface_colors_match_web() {
    let web = read_web_golden_open("button-group-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "button-group-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Label As...",
        build_shadcn_button_group_demo,
    );
}

#[test]
fn web_vs_fret_button_group_demo_submenu_kbd_surface_colors_match_web_dark() {
    let web = read_web_golden_open("button-group-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "button-group-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Label As...",
        build_shadcn_button_group_demo,
    );
}

#[test]
fn web_vs_fret_button_group_demo_submenu_kbd_shadow_matches_web() {
    let web = read_web_golden_open("button-group-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "button-group-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Label As...",
        build_shadcn_button_group_demo,
    );
}

#[test]
fn web_vs_fret_button_group_demo_submenu_kbd_shadow_matches_web_dark() {
    let web = read_web_golden_open("button-group-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "button-group-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Label As...",
        build_shadcn_button_group_demo,
    );
}

fn build_shadcn_combobox_dropdown_menu_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui::element::{CrossAlign, LayoutStyle, Length, MainAlign, RowProps};
    use fret_ui_kit::declarative::icon as decl_icon;
    use fret_ui_shadcn::{
        Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
        DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuShortcut,
    };

    let button = Button::new("More")
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .children([decl_icon::icon(cx, fret_icons::ids::ui::MORE_HORIZONTAL)]);

    let dropdown = DropdownMenu::new(open.clone())
        .align(DropdownMenuAlign::End)
        .min_width(Px(200.0))
        .into_element(
            cx,
            |cx| button.clone().into_element(cx),
            |cx| {
                vec![
                    DropdownMenuEntry::Label(DropdownMenuLabel::new("Actions")),
                    DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Assign to...")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Set due date...")),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Apply label").submenu(
                            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("feature"))],
                        )),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Delete")
                                .variant(
                                    fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                )
                                .trailing(DropdownMenuShortcut::new("⌘⌫").into_element(cx)),
                        ),
                    ])),
                ]
            },
        );

    cx.row(
        RowProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout
            },
            gap: Px(0.0),
            padding: fret_core::Edges {
                top: Px(12.0),
                right: Px(16.0),
                bottom: Px(12.0),
                left: Px(16.0),
            },
            justify: MainAlign::End,
            align: CrossAlign::Start,
        },
        |_cx| vec![dropdown],
    )
}

#[test]
fn web_vs_fret_combobox_dropdown_menu_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "combobox-dropdown-menu",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_combobox_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_combobox_dropdown_menu_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "combobox-dropdown-menu",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_combobox_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_combobox_dropdown_menu_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "combobox-dropdown-menu",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_combobox_dropdown_menu_demo,
    );
}

#[test]
fn web_vs_fret_combobox_dropdown_menu_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "combobox-dropdown-menu",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_combobox_dropdown_menu_demo,
    );
}

fn build_shadcn_breadcrumb_dropdown_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui::element::PressableProps;
    use fret_ui_shadcn::breadcrumb::primitives as bc;
    use fret_ui_shadcn::{DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem};

    let dropdown = DropdownMenu::new(open.clone()).align(DropdownMenuAlign::Start);

    bc::Breadcrumb::new().into_element(cx, |cx| {
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
                            let theme = fret_ui::Theme::global(&*cx.app).clone();
                            let muted = theme.color_required("muted-foreground");

                            let mut props = PressableProps::default();
                            props.a11y.role = Some(SemanticsRole::Button);
                            props.a11y.label = Some(Arc::from("Components"));

                            cx.pressable(props, move |cx, _st| {
                                vec![
                                    cx.text("Components"),
                                    fret_ui_kit::declarative::icon::icon_with(
                                        cx,
                                        fret_icons::ids::ui::CHEVRON_DOWN,
                                        Some(Px(14.0)),
                                        Some(fret_ui_kit::ColorRef::Color(muted)),
                                    ),
                                ]
                            })
                        },
                        |_cx| {
                            vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Documentation")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Themes")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                            ]
                        },
                    )]
                }),
                bc::BreadcrumbSeparator::new()
                    .kind(bc::BreadcrumbSeparatorKind::Slash)
                    .into_element(cx),
                bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                }),
            ]
        })]
    })
}

#[test]
fn web_vs_fret_breadcrumb_dropdown_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "breadcrumb-dropdown",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_dropdown_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_dropdown_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "breadcrumb-dropdown",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_dropdown_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_dropdown_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "breadcrumb-dropdown",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_dropdown_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_dropdown_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "breadcrumb-dropdown",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_dropdown_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_dropdown_small_viewport_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "breadcrumb-dropdown.vp1440x320",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_dropdown_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_dropdown_small_viewport_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "breadcrumb-dropdown.vp1440x320",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_dropdown_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_dropdown_small_viewport_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "breadcrumb-dropdown.vp1440x320",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_dropdown_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_dropdown_small_viewport_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "breadcrumb-dropdown.vp1440x320",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_dropdown_demo,
    );
}

fn build_shadcn_breadcrumb_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui::element::PressableProps;
    use fret_ui_shadcn::breadcrumb::primitives as bc;
    use fret_ui_shadcn::{DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem};

    let dropdown = DropdownMenu::new(open.clone()).align(DropdownMenuAlign::Start);

    bc::Breadcrumb::new().into_element(cx, |cx| {
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
                            props.a11y.label = Some(Arc::from("Toggle menu"));

                            cx.pressable(props, move |cx, _st| {
                                vec![
                                    bc::BreadcrumbEllipsis::new()
                                        .size(Px(16.0))
                                        .into_element(cx),
                                ]
                            })
                        },
                        |_cx| {
                            vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Documentation")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Themes")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
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
    })
}

#[test]
fn web_vs_fret_breadcrumb_demo_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "breadcrumb-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_demo_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "breadcrumb-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_demo_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "breadcrumb-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_demo_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "breadcrumb-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_demo_small_viewport_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "breadcrumb-demo.vp1440x320",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_demo_small_viewport_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "breadcrumb-demo.vp1440x320",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_demo_small_viewport_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "breadcrumb-demo.vp1440x320",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_demo,
    );
}

#[test]
fn web_vs_fret_breadcrumb_demo_small_viewport_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "breadcrumb-demo.vp1440x320",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_breadcrumb_demo,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_checkboxes_surface_colors_match_web() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuEntry,
        DropdownMenuLabel,
    };

    assert_overlay_surface_colors_match(
        "dropdown-menu-checkboxes",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_status_bar: Option<Model<bool>>,
                checked_activity_bar: Option<Model<bool>>,
                checked_panel: Option<Model<bool>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_status_bar.as_ref(),
                    st.checked_activity_bar.as_ref(),
                    st.checked_panel.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_status_bar, checked_activity_bar, checked_panel) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_status_bar = cx.app.models_mut().insert(true);
                    let checked_activity_bar = cx.app.models_mut().insert(false);
                    let checked_panel = cx.app.models_mut().insert(false);

                    cx.with_state(Models::default, |st| {
                        st.checked_status_bar = Some(checked_status_bar.clone());
                        st.checked_activity_bar = Some(checked_activity_bar.clone());
                        st.checked_panel = Some(checked_panel.clone());
                    });

                    (checked_status_bar, checked_activity_bar, checked_panel)
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
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("Appearance")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                                checked_status_bar,
                                "Status Bar",
                            )),
                            DropdownMenuEntry::CheckboxItem(
                                DropdownMenuCheckboxItem::new(checked_activity_bar, "Activity Bar")
                                    .disabled(true),
                            ),
                            DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                                checked_panel,
                                "Panel",
                            )),
                        ]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_checkboxes_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuEntry,
        DropdownMenuLabel,
    };

    assert_overlay_surface_colors_match(
        "dropdown-menu-checkboxes",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_status_bar: Option<Model<bool>>,
                checked_activity_bar: Option<Model<bool>>,
                checked_panel: Option<Model<bool>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_status_bar.as_ref(),
                    st.checked_activity_bar.as_ref(),
                    st.checked_panel.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_status_bar, checked_activity_bar, checked_panel) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_status_bar = cx.app.models_mut().insert(true);
                    let checked_activity_bar = cx.app.models_mut().insert(false);
                    let checked_panel = cx.app.models_mut().insert(false);

                    cx.with_state(Models::default, |st| {
                        st.checked_status_bar = Some(checked_status_bar.clone());
                        st.checked_activity_bar = Some(checked_activity_bar.clone());
                        st.checked_panel = Some(checked_panel.clone());
                    });

                    (checked_status_bar, checked_activity_bar, checked_panel)
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
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("Appearance")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                                checked_status_bar,
                                "Status Bar",
                            )),
                            DropdownMenuEntry::CheckboxItem(
                                DropdownMenuCheckboxItem::new(checked_activity_bar, "Activity Bar")
                                    .disabled(true),
                            ),
                            DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                                checked_panel,
                                "Panel",
                            )),
                        ]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_checkboxes_shadow_matches_web() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuEntry,
        DropdownMenuLabel,
    };

    assert_overlay_shadow_insets_match(
        "dropdown-menu-checkboxes",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_status_bar: Option<Model<bool>>,
                checked_activity_bar: Option<Model<bool>>,
                checked_panel: Option<Model<bool>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_status_bar.as_ref(),
                    st.checked_activity_bar.as_ref(),
                    st.checked_panel.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_status_bar, checked_activity_bar, checked_panel) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_status_bar = cx.app.models_mut().insert(true);
                    let checked_activity_bar = cx.app.models_mut().insert(false);
                    let checked_panel = cx.app.models_mut().insert(false);

                    cx.with_state(Models::default, |st| {
                        st.checked_status_bar = Some(checked_status_bar.clone());
                        st.checked_activity_bar = Some(checked_activity_bar.clone());
                        st.checked_panel = Some(checked_panel.clone());
                    });

                    (checked_status_bar, checked_activity_bar, checked_panel)
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
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("Appearance")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                                checked_status_bar,
                                "Status Bar",
                            )),
                            DropdownMenuEntry::CheckboxItem(
                                DropdownMenuCheckboxItem::new(checked_activity_bar, "Activity Bar")
                                    .disabled(true),
                            ),
                            DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                                checked_panel,
                                "Panel",
                            )),
                        ]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_checkboxes_shadow_matches_web_dark() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuEntry,
        DropdownMenuLabel,
    };

    assert_overlay_shadow_insets_match(
        "dropdown-menu-checkboxes",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_status_bar: Option<Model<bool>>,
                checked_activity_bar: Option<Model<bool>>,
                checked_panel: Option<Model<bool>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_status_bar.as_ref(),
                    st.checked_activity_bar.as_ref(),
                    st.checked_panel.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_status_bar, checked_activity_bar, checked_panel) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_status_bar = cx.app.models_mut().insert(true);
                    let checked_activity_bar = cx.app.models_mut().insert(false);
                    let checked_panel = cx.app.models_mut().insert(false);

                    cx.with_state(Models::default, |st| {
                        st.checked_status_bar = Some(checked_status_bar.clone());
                        st.checked_activity_bar = Some(checked_activity_bar.clone());
                        st.checked_panel = Some(checked_panel.clone());
                    });

                    (checked_status_bar, checked_activity_bar, checked_panel)
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
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("Appearance")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                                checked_status_bar,
                                "Status Bar",
                            )),
                            DropdownMenuEntry::CheckboxItem(
                                DropdownMenuCheckboxItem::new(checked_activity_bar, "Activity Bar")
                                    .disabled(true),
                            ),
                            DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                                checked_panel,
                                "Panel",
                            )),
                        ]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_radio_group_surface_colors_match_web() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuLabel,
        DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
    };

    assert_overlay_surface_colors_match(
        "dropdown-menu-radio-group",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                position: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| st.position.as_ref().cloned());
            let position = if let Some(existing) = existing {
                existing
            } else {
                let position = cx.app.models_mut().insert(Some(Arc::from("bottom")));
                cx.with_state(Models::default, |st| st.position = Some(position.clone()));
                position
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
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("Panel Position")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::RadioGroup(
                                DropdownMenuRadioGroup::new(position)
                                    .item(DropdownMenuRadioItemSpec::new("top", "Top"))
                                    .item(DropdownMenuRadioItemSpec::new("bottom", "Bottom"))
                                    .item(DropdownMenuRadioItemSpec::new("right", "Right")),
                            ),
                        ]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_radio_group_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuLabel,
        DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
    };

    assert_overlay_surface_colors_match(
        "dropdown-menu-radio-group",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                position: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| st.position.as_ref().cloned());
            let position = if let Some(existing) = existing {
                existing
            } else {
                let position = cx.app.models_mut().insert(Some(Arc::from("bottom")));
                cx.with_state(Models::default, |st| st.position = Some(position.clone()));
                position
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
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("Panel Position")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::RadioGroup(
                                DropdownMenuRadioGroup::new(position)
                                    .item(DropdownMenuRadioItemSpec::new("top", "Top"))
                                    .item(DropdownMenuRadioItemSpec::new("bottom", "Bottom"))
                                    .item(DropdownMenuRadioItemSpec::new("right", "Right")),
                            ),
                        ]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_radio_group_shadow_matches_web() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuLabel,
        DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
    };

    assert_overlay_shadow_insets_match(
        "dropdown-menu-radio-group",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                position: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| st.position.as_ref().cloned());
            let position = if let Some(existing) = existing {
                existing
            } else {
                let position = cx.app.models_mut().insert(Some(Arc::from("bottom")));
                cx.with_state(Models::default, |st| st.position = Some(position.clone()));
                position
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
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("Panel Position")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::RadioGroup(
                                DropdownMenuRadioGroup::new(position)
                                    .item(DropdownMenuRadioItemSpec::new("top", "Top"))
                                    .item(DropdownMenuRadioItemSpec::new("bottom", "Bottom"))
                                    .item(DropdownMenuRadioItemSpec::new("right", "Right")),
                            ),
                        ]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_radio_group_shadow_matches_web_dark() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuLabel,
        DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
    };

    assert_overlay_shadow_insets_match(
        "dropdown-menu-radio-group",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                position: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| st.position.as_ref().cloned());
            let position = if let Some(existing) = existing {
                existing
            } else {
                let position = cx.app.models_mut().insert(Some(Arc::from("bottom")));
                cx.with_state(Models::default, |st| st.position = Some(position.clone()));
                position
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
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("Panel Position")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::RadioGroup(
                                DropdownMenuRadioGroup::new(position)
                                    .item(DropdownMenuRadioItemSpec::new("top", "Top"))
                                    .item(DropdownMenuRadioItemSpec::new("bottom", "Bottom"))
                                    .item(DropdownMenuRadioItemSpec::new("right", "Right")),
                            ),
                        ]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
        "dropdown-menu-demo.submenu",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
        "dropdown-menu-demo.submenu",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_shadow_matches_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_small_viewport_shadow_matches_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_small_viewport_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_small_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_small_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_shadow_matches_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
            DropdownMenu::new(open.clone())
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
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{ContextMenu, ContextMenuEntry, ContextMenuItem};

    assert_overlay_surface_colors_match(
        "context-menu-demo",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Copy"))],
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{ContextMenu, ContextMenuEntry, ContextMenuItem};

    assert_overlay_surface_colors_match(
        "context-menu-demo",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Copy"))],
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_small_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{ContextMenu, ContextMenuEntry, ContextMenuItem};

    assert_overlay_surface_colors_match(
        "context-menu-demo.vp1440x320",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Copy"))],
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_small_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{ContextMenu, ContextMenuEntry, ContextMenuItem};

    assert_overlay_surface_colors_match(
        "context-menu-demo.vp1440x320",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Copy"))],
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{ContextMenu, ContextMenuEntry, ContextMenuItem};

    assert_overlay_surface_colors_match(
        "context-menu-demo.vp1440x240",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Copy"))],
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{ContextMenu, ContextMenuEntry, ContextMenuItem};

    assert_overlay_surface_colors_match(
        "context-menu-demo.vp1440x240",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Copy"))],
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
        "context-menu-demo.submenu",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
        "context-menu-demo.submenu",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_shadow_matches_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_small_viewport_shadow_matches_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp1440x320",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_small_viewport_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp1440x320",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_small_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp1440x320",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_small_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp1440x320",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_shadow_matches_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp1440x240",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp1440x240",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp1440x240",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp1440x240",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| Button::new("Right click here").into_element(cx),
                    |_cx| {
                        vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}

#[test]
fn web_vs_fret_drawer_demo_surface_colors_match_web_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent};

    assert_overlay_surface_colors_match(
        "drawer-demo.vp1440x240",
        "drawer-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_500 + 2,
        |cx, open| {
            Drawer::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Drawer")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| DrawerContent::new(vec![cx.text("Drawer content")]).into_element(cx),
            )
        },
    );
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
        build_shadcn_select_scrollable_demo,
    );
}

fn build_shadcn_select_scrollable_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    fret_ui_shadcn::Select::new(value, open.clone())
        .a11y_label("Select")
        .item(fret_ui_shadcn::SelectItem::new("one", "One"))
        .item(fret_ui_shadcn::SelectItem::new("two", "Two"))
        .into_element(cx)
}

#[test]
fn web_vs_fret_select_scrollable_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "select-scrollable",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "select-scrollable",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "select-scrollable",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "select-scrollable",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "select-scrollable.vp1440x450",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "select-scrollable.vp1440x450",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "select-scrollable.vp1440x450",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "select-scrollable.vp1440x450",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "select-scrollable.vp1440x240",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "select-scrollable.vp1440x240",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "select-scrollable.vp1440x240",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "select-scrollable.vp1440x240",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}

fn hover_first_listbox_option(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
) {
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let listbox = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBox)
        .max_by(|a, b| {
            rect_area(ui.debug_node_bounds(a.id).unwrap_or(a.bounds))
                .total_cmp(&rect_area(ui.debug_node_bounds(b.id).unwrap_or(b.bounds)))
        })
        .expect("listbox");
    let listbox_bounds = ui.debug_node_bounds(listbox.id).unwrap_or(listbox.bounds);
    let mut option_candidates: Vec<(Rect, &fret_core::SemanticsNode)> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .map(|n| (ui.debug_node_bounds(n.id).unwrap_or(n.bounds), n))
        .collect();
    option_candidates.sort_by(|(a, _), (b, _)| {
        a.origin
            .y
            .0
            .total_cmp(&b.origin.y.0)
            .then_with(|| a.origin.x.0.total_cmp(&b.origin.x.0))
    });

    let option = option_candidates
        .iter()
        .find(|(bounds, _)| rect_contains(listbox_bounds, *bounds))
        .map(|(_, n)| *n)
        .unwrap_or_else(|| {
            let samples: Vec<Rect> = option_candidates.iter().take(8).map(|(b, _)| *b).collect();
            panic!(
                "listbox option\n  listbox_bounds={listbox_bounds:?}\n  first_option_bounds={samples:?}"
            )
        });
    let option_bounds = ui.debug_node_bounds(option.id).unwrap_or(option.bounds);

    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: bounds_center(option_bounds),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
}

fn build_shadcn_select_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let entries: Vec<SelectEntry> = vec![
        SelectGroup::new(vec![
            SelectLabel::new("Fruits").into(),
            SelectItem::new("apple", "Apple").into(),
            SelectItem::new("banana", "Banana").into(),
            SelectItem::new("blueberry", "Blueberry").into(),
            SelectItem::new("grapes", "Grapes").into(),
            SelectItem::new("pineapple", "Pineapple").into(),
        ])
        .into(),
    ];

    fret_ui_shadcn::Select::new(value, open.clone())
        .a11y_label("Select")
        .placeholder("Select a fruit")
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default().w_px(fret_ui_kit::MetricRef::Px(Px(180.0))),
        )
        .entries(entries)
        .into_element(cx)
}

fn build_shadcn_select_scrollable_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
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

    fret_ui_shadcn::Select::new(value, open.clone())
        .a11y_label("Select")
        .placeholder("Select a timezone")
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default().w_px(fret_ui_kit::MetricRef::Px(Px(280.0))),
        )
        .entries(entries)
        .into_element(cx)
}

fn build_shadcn_combobox_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Combobox, ComboboxItem};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let items = vec![
        ComboboxItem::new("apple", "Apple"),
        ComboboxItem::new("banana", "Banana"),
        ComboboxItem::new("blueberry", "Blueberry"),
        ComboboxItem::new("grapes", "Grapes"),
        ComboboxItem::new("pineapple", "Pineapple"),
    ];

    Combobox::new(value, open.clone())
        .a11y_label("Select a fruit")
        .width(Px(200.0))
        .items(items)
        .into_element(cx)
}

fn assert_listbox_highlighted_option_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    web_option_slot: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_listbox_option_chrome(theme, web_option_slot);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();
    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx, &open)],
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build(cx, &open)],
        );
    }

    hover_first_listbox_option(&mut ui, &mut app, &mut services);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2 + settle_frames),
        true,
        |cx| vec![build(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let listbox = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBox)
        .max_by(|a, b| {
            rect_area(ui.debug_node_bounds(a.id).unwrap_or(a.bounds))
                .total_cmp(&rect_area(ui.debug_node_bounds(b.id).unwrap_or(b.bounds)))
        })
        .expect("listbox");
    let listbox_bounds = ui.debug_node_bounds(listbox.id).unwrap_or(listbox.bounds);
    let option = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| {
            rect_contains(
                listbox_bounds,
                ui.debug_node_bounds(n.id).unwrap_or(n.bounds),
            )
        })
        .min_by(|a, b| {
            let a_bounds = ui.debug_node_bounds(a.id).unwrap_or(a.bounds);
            let b_bounds = ui.debug_node_bounds(b.id).unwrap_or(b.bounds);
            a_bounds
                .origin
                .y
                .0
                .total_cmp(&b_bounds.origin.y.0)
                .then_with(|| a_bounds.origin.x.0.total_cmp(&b_bounds.origin.x.0))
        })
        .expect("listbox option");
    let option_bounds = ui.debug_node_bounds(option.id).unwrap_or(option.bounds);

    let quad = find_best_solid_quad_within_matching_bg(&scene, option_bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted option background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted option background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(&scene, listbox_bounds, bounds_center(option_bounds))
        .unwrap_or_else(|| {
            let mut total_text = 0usize;
            let mut samples_raw: Vec<(f32, f32)> = Vec::new();
            let mut samples_tx: Vec<(f32, f32)> = Vec::new();
            scene_walk(&scene, |st, op| {
                let SceneOp::Text { origin, .. } = *op else {
                    return;
                };
                total_text += 1;
                if samples_raw.len() < 16 {
                    samples_raw.push((origin.x.0, origin.y.0));
                }
                if samples_tx.len() < 16 {
                    let p = st.transform.apply_point(origin);
                    samples_tx.push((p.x.0, p.y.0));
                }
            });
            panic!(
                "{web_name} {web_theme_name}: highlighted option text color (no text ops near)\n  total_text_ops={total_text}\n  sample_origins_raw={samples_raw:?}\n  sample_origins_tx={samples_tx:?}\n  listbox_bounds={listbox_bounds:?}\n  option_bounds={option_bounds:?}",
            )
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted option text color"),
        text,
        expected.fg,
        0.03,
    );
}

#[test]
fn web_vs_fret_select_demo_highlighted_option_chrome_matches_web() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-demo.highlight-first",
        "light",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_demo_page,
    );
}

#[test]
fn web_vs_fret_select_demo_highlighted_option_chrome_matches_web_dark() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-demo.highlight-first",
        "dark",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_demo_page,
    );
}

#[test]
fn web_vs_fret_select_scrollable_highlighted_option_chrome_matches_web() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-scrollable.highlight-first",
        "light",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_scrollable_page,
    );
}

#[test]
fn web_vs_fret_select_scrollable_highlighted_option_chrome_matches_web_dark() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-scrollable.highlight-first",
        "dark",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_scrollable_page,
    );
}

#[test]
fn web_vs_fret_combobox_demo_highlighted_option_chrome_matches_web() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "combobox-demo.highlight-first",
        "light",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_combobox_demo_page,
    );
}

#[test]
fn web_vs_fret_combobox_demo_highlighted_option_chrome_matches_web_dark() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "combobox-demo.highlight-first",
        "dark",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_combobox_demo_page,
    );
}

fn assert_listbox_focused_option_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
    a11y_label: &str,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();
    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx, &open)],
    );

    let (snap, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox && n.label.as_deref() == Some(a11y_label))
        .expect("trigger semantics (combobox) by a11y label");
    ui.set_focus(Some(trigger.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx, &open)],
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let option = snap
        .nodes
        .iter()
        .find(|n| n.flags.focused && n.role == SemanticsRole::ListBoxOption)
        .or_else(|| {
            // Some widgets use an aria-activedescendant model where the listbox retains focus and
            // the active option is indicated via `active_descendant`.
            let focused_listbox = snap
                .nodes
                .iter()
                .find(|n| n.flags.focused && n.role == SemanticsRole::ListBox)
                .or_else(|| {
                    snap.nodes
                        .iter()
                        .find(|n| n.role == SemanticsRole::ListBox && n.active_descendant.is_some())
                });

            let listbox = focused_listbox?;
            let active_id = listbox.active_descendant?;
            snap.nodes
                .iter()
                .find(|n| n.id == active_id && n.role == SemanticsRole::ListBoxOption)
        })
        .unwrap_or_else(|| {
            let focused_roles: Vec<SemanticsRole> =
                snap.nodes.iter().filter(|n| n.flags.focused).map(|n| n.role).collect();
            let focused_listbox_active = snap
                .nodes
                .iter()
                .find(|n| n.flags.focused && n.role == SemanticsRole::ListBox)
                .and_then(|n| n.active_descendant);
            let listbox_count = snap.nodes.iter().filter(|n| n.role == SemanticsRole::ListBox).count();
            let option_count = snap
                .nodes
                .iter()
                .filter(|n| n.role == SemanticsRole::ListBoxOption)
                .count();
            panic!(
                "expected focused listbox option semantics node (or listbox active_descendant)\n  listbox_count={listbox_count}\n  option_count={option_count}\n  focused_roles={focused_roles:?}\n  focused_listbox_active_descendant={focused_listbox_active:?}"
            )
        });

    let quad = find_best_solid_quad_within_matching_bg(&scene, option.bounds, expected.bg)
        .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option background quad"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        option.bounds,
        leftish_text_probe_point(option.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option text color"),
        text,
        expected.fg,
        0.03,
    );
}

#[test]
fn web_vs_fret_select_demo_focused_option_chrome_matches_web() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-demo.focus-first",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_demo_page,
        "Select",
    );
}

#[test]
fn web_vs_fret_select_demo_focused_option_chrome_matches_web_dark() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-demo.focus-first",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_demo_page,
        "Select",
    );
}

#[test]
fn web_vs_fret_select_scrollable_focused_option_chrome_matches_web() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-scrollable.focus-first",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_scrollable_page,
        "Select",
    );
}

#[test]
fn web_vs_fret_select_scrollable_focused_option_chrome_matches_web_dark() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-scrollable.focus-first",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_scrollable_page,
        "Select",
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
fn web_vs_fret_tooltip_surface_colors_match_web() {
    assert_hover_overlay_surface_colors_match_by_portal_slot_theme(
        "tooltip-demo",
        "tooltip-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Tooltip,
        "Hover",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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
fn web_vs_fret_tooltip_surface_colors_match_web_dark() {
    assert_hover_overlay_surface_colors_match_by_portal_slot_theme(
        "tooltip-demo",
        "tooltip-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Tooltip,
        "Hover",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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
fn web_vs_fret_hover_card_surface_colors_match_web() {
    assert_overlay_chrome_matches_by_portal_slot_theme(
        "hover-card-demo",
        "hover-card-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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
fn web_vs_fret_hover_card_surface_colors_match_web_dark() {
    assert_overlay_chrome_matches_by_portal_slot_theme(
        "hover-card-demo",
        "hover-card-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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
fn web_vs_fret_context_menu_demo_shadow_matches_web() {
    assert_context_menu_shadow_insets_match(
        "context-menu-demo",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
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
fn web_vs_fret_context_menu_demo_shadow_matches_web_dark() {
    assert_context_menu_shadow_insets_match(
        "context-menu-demo",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
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
fn web_vs_fret_context_menu_demo_small_viewport_shadow_matches_web() {
    assert_context_menu_shadow_insets_match(
        "context-menu-demo.vp1440x320",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
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
fn web_vs_fret_context_menu_demo_small_viewport_shadow_matches_web_dark() {
    assert_context_menu_shadow_insets_match(
        "context-menu-demo.vp1440x320",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
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
fn web_vs_fret_context_menu_demo_tiny_viewport_shadow_matches_web() {
    assert_context_menu_shadow_insets_match(
        "context-menu-demo.vp1440x240",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
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
fn web_vs_fret_context_menu_demo_tiny_viewport_shadow_matches_web_dark() {
    assert_context_menu_shadow_insets_match(
        "context-menu-demo.vp1440x240",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
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

fn assert_context_menu_demo_highlighted_item_background_matches_web(web_theme_name: &str) {
    let web = read_web_golden_open("context-menu-demo.highlight-first");
    let theme = web_theme_named(&web, web_theme_name);
    let expected_bg = web_find_highlighted_menu_item_background(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    let scheme = match web_theme_name {
        "dark" => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        _ => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    };
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);
    let build = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

        ContextMenu::new(open.clone()).into_element(
            cx,
            |cx| Button::new("Right click here").into_element(cx),
            |_cx| {
                vec![
                    ContextMenuEntry::Item(ContextMenuItem::new("Copy")),
                    ContextMenuEntry::Item(ContextMenuItem::new("Paste")),
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
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics (Right click here)");
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build(cx)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let item = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Copy"))
        .expect("context-menu hovered item semantics (Copy)");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: bounds_center(item.bounds),
            buttons: MouseButtons::default(),
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
        FrameId(2 + settle_frames),
        true,
        |cx| vec![build(cx)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let item = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Copy"))
        .expect("context-menu hovered item semantics (Copy)");
    let quad = find_best_solid_quad_within_matching_bg(&scene, item.bounds, expected_bg)
        .unwrap_or_else(|| panic!("painted quad for context-menu highlighted menuitem background"));
    assert_rgba_close(
        &format!("context-menu-demo {web_theme_name} highlighted menuitem background"),
        color_to_rgba(quad.background),
        expected_bg,
        0.03,
    );
}

#[test]
fn web_vs_fret_context_menu_demo_highlighted_item_background_matches_web() {
    assert_context_menu_demo_highlighted_item_background_matches_web("light");
}

#[test]
fn web_vs_fret_context_menu_demo_highlighted_item_background_matches_web_dark() {
    assert_context_menu_demo_highlighted_item_background_matches_web("dark");
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
fn web_vs_fret_menubar_demo_shadow_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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
fn web_vs_fret_menubar_demo_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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

fn assert_menubar_demo_highlighted_item_background_matches_web(web_theme_name: &str) {
    let web = read_web_golden_open("menubar-demo.highlight-first");
    let theme = web_theme_named(&web, web_theme_name);
    let expected_bg = web_find_highlighted_menu_item_background(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    let scheme = match web_theme_name {
        "dark" => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        _ => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    };
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar trigger semantics (File)");
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build_shadcn_menubar_demo(cx)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let item = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar hovered item semantics (New Tab)");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: bounds_center(item.bounds),
            buttons: MouseButtons::default(),
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
        FrameId(2 + settle_frames),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let item = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar hovered item semantics (New Tab)");
    let quad = find_best_solid_quad_within_matching_bg(&scene, item.bounds, expected_bg)
        .unwrap_or_else(|| panic!("painted quad for menubar highlighted menuitem background"));
    assert_rgba_close(
        &format!("menubar-demo {web_theme_name} highlighted menuitem background"),
        color_to_rgba(quad.background),
        expected_bg,
        0.03,
    );
}

#[test]
fn web_vs_fret_menubar_demo_highlighted_item_background_matches_web() {
    assert_menubar_demo_highlighted_item_background_matches_web("light");
}

#[test]
fn web_vs_fret_menubar_demo_highlighted_item_background_matches_web_dark() {
    assert_menubar_demo_highlighted_item_background_matches_web("dark");
}

fn assert_menubar_demo_focused_item_chrome_matches_web(web_theme_name: &str) {
    let web = read_web_golden_open("menubar-demo.focus-first");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    let scheme = match web_theme_name {
        "dark" => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        _ => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    };
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar trigger semantics (File)");
    ui.set_focus(Some(trigger.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build_shadcn_menubar_demo(cx)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let focused = snap
        .nodes
        .iter()
        .find(|n| n.flags.focused)
        .expect("focused semantics node");
    assert_eq!(
        focused.role,
        SemanticsRole::MenuItem,
        "expected focused node to be a menu item after ArrowDown open",
    );

    let quad = find_best_solid_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| panic!("menubar-demo {web_theme_name}: focused item background quad"));
    assert_rgba_close(
        &format!("menubar-demo {web_theme_name} focused item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| panic!("menubar-demo {web_theme_name}: focused item text color"));
    assert_rgba_close(
        &format!("menubar-demo {web_theme_name} focused item text color"),
        text,
        expected.fg,
        0.03,
    );
}

#[test]
fn web_vs_fret_menubar_demo_focused_item_chrome_matches_web() {
    assert_menubar_demo_focused_item_chrome_matches_web("light");
}

#[test]
fn web_vs_fret_menubar_demo_focused_item_chrome_matches_web_dark() {
    assert_menubar_demo_focused_item_chrome_matches_web("dark");
}

fn build_shadcn_menubar_demo(cx: &mut ElementContext<'_, App>) -> AnyElement {
    use fret_ui_shadcn::{
        Menubar, MenubarCheckboxItem, MenubarEntry, MenubarItem, MenubarMenu, MenubarRadioGroup,
        MenubarRadioItemSpec, MenubarShortcut,
    };

    #[derive(Default)]
    struct Models {
        view_bookmarks_bar: Option<Model<bool>>,
        view_full_urls: Option<Model<bool>>,
        profile_value: Option<Model<Option<Arc<str>>>>,
    }

    let existing = cx.with_state(Models::default, |st| {
        match (
            st.view_bookmarks_bar.as_ref(),
            st.view_full_urls.as_ref(),
            st.profile_value.as_ref(),
        ) {
            (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
            _ => None,
        }
    });

    let (view_bookmarks_bar, view_full_urls, profile_value) = if let Some(existing) = existing {
        existing
    } else {
        let view_bookmarks_bar = cx.app.models_mut().insert(false);
        let view_full_urls = cx.app.models_mut().insert(true);
        let profile_value = cx.app.models_mut().insert(Some(Arc::from("benoit")));

        cx.with_state(Models::default, |st| {
            st.view_bookmarks_bar = Some(view_bookmarks_bar.clone());
            st.view_full_urls = Some(view_full_urls.clone());
            st.profile_value = Some(profile_value.clone());
        });

        (view_bookmarks_bar, view_full_urls, profile_value)
    };

    Menubar::new(vec![
        MenubarMenu::new("File").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("New Tab")
                    .test_id("menubar.file.new_tab")
                    .trailing(MenubarShortcut::new("⌘T").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("New Window")
                    .trailing(MenubarShortcut::new("⌘N").into_element(cx)),
            ),
            MenubarEntry::Item(MenubarItem::new("New Incognito Window").disabled(true)),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(
                MenubarItem::new("Share")
                    .test_id("menubar.file.share")
                    .submenu(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ]),
            ),
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

#[test]
fn web_vs_fret_menubar_demo_view_shadow_matches_web() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.view",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "View",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_view_shadow_matches_web_dark() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.view",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "View",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_profiles_shadow_matches_web() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.profiles",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "Profiles",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_profiles_shadow_matches_web_dark() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.profiles",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "Profiles",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_small_viewport_shadow_matches_web() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.vp1440x320",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_small_viewport_shadow_matches_web_dark() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.vp1440x320",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_shadow_matches_web() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.vp1440x240",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_shadow_matches_web_dark() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.vp1440x240",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_root_shadow_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo");
    let theme = web_theme_named(&web, "light");
    let web_root =
        find_by_data_slot(&theme.root, "menubar").expect("web menubar root node (data-slot)");
    let expected = web_drop_shadow_insets(web_root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![
                Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                    MenubarEntry::Item(MenubarItem::new("New Tab")),
                    MenubarEntry::Item(MenubarItem::new("New Window")),
                    MenubarEntry::Separator,
                    MenubarEntry::Item(MenubarItem::new("Share")),
                ])])
                .into_element(cx),
            ]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let menubar = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuBar)
        .expect("fret menubar semantics node");
    let quad = find_best_chrome_quad(&scene, menubar.bounds).expect("menubar chrome quad");

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match("menubar-demo", "light", &expected, &candidates);
}

#[test]
fn web_vs_fret_menubar_root_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo");
    let theme = web_theme_named(&web, "dark");
    let web_root =
        find_by_data_slot(&theme.root, "menubar").expect("web menubar root node (data-slot)");
    let expected = web_drop_shadow_insets(web_root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![
                Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                    MenubarEntry::Item(MenubarItem::new("New Tab")),
                    MenubarEntry::Item(MenubarItem::new("New Window")),
                    MenubarEntry::Separator,
                    MenubarEntry::Item(MenubarItem::new("Share")),
                ])])
                .into_element(cx),
            ]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let menubar = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuBar)
        .expect("fret menubar semantics node");
    let quad = find_best_chrome_quad(&scene, menubar.bounds).expect("menubar chrome quad");

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match("menubar-demo", "dark", &expected, &candidates);
}

#[test]
fn web_vs_fret_menubar_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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
fn web_vs_fret_menubar_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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
fn web_vs_fret_menubar_demo_small_viewport_surface_colors_match_web() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.vp1440x320",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_small_viewport_surface_colors_match_web_dark() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.vp1440x320",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_surface_colors_match_web() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.vp1440x240",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_surface_colors_match_web_dark() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.vp1440x240",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_view_surface_colors_match_web() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.view",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "View",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_view_surface_colors_match_web_dark() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.view",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "View",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_profiles_surface_colors_match_web() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.profiles",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "Profiles",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_profiles_surface_colors_match_web_dark() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.profiles",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "Profiles",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_surface_colors_match_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.submenu",
        "menubar-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.submenu",
        "menubar-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_shadow_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "menubar-demo.submenu-kbd",
        "menubar-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "menubar-demo.submenu-kbd",
        "menubar-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_small_viewport_shadow_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "menubar-demo.submenu-kbd-vp1440x320",
        "menubar-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_small_viewport_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "menubar-demo.submenu-kbd-vp1440x320",
        "menubar-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_small_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "menubar-demo.submenu-kbd-vp1440x320",
        "menubar-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_small_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x320");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(320.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "menubar-demo.submenu-kbd-vp1440x320",
        "menubar-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_shadow_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "menubar-demo.submenu-kbd-vp1440x240",
        "menubar-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "menubar-demo.submenu-kbd-vp1440x240",
        "menubar-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "menubar-demo.submenu-kbd-vp1440x240",
        "menubar-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "menubar-demo.submenu-kbd-vp1440x240",
        "menubar-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
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

#[test]
fn web_vs_fret_navigation_menu_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_content_surface_colors_match(
        "navigation-menu-demo",
        "navigation-menu-content",
        "open",
        "home",
        "Home",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
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

#[test]
fn web_vs_fret_navigation_menu_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_content_surface_colors_match(
        "navigation-menu-demo",
        "navigation-menu-content",
        "open",
        "home",
        "Home",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
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

#[test]
fn web_vs_fret_navigation_menu_demo_shadow_matches_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_content_shadow_insets_match(
        "navigation-menu-demo",
        "navigation-menu-content",
        "open",
        "home",
        "Home",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
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

#[test]
fn web_vs_fret_navigation_menu_demo_shadow_matches_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_content_shadow_insets_match(
        "navigation-menu-demo",
        "navigation-menu-content",
        "open",
        "home",
        "Home",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
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

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_viewport_shadow_matches_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_shadow_insets_match(
        "navigation-menu-demo.home-mobile",
        "navigation-menu-viewport",
        "open",
        "Home",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
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

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_viewport_shadow_matches_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_shadow_insets_match(
        "navigation-menu-demo.home-mobile",
        "navigation-menu-viewport",
        "open",
        "Home",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
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

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_constrained_viewport_shadow_matches_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_shadow_insets_match(
        "navigation-menu-demo.home-mobile-vp375x320",
        "navigation-menu-viewport",
        "open",
        "Home",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
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

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_constrained_viewport_shadow_matches_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_shadow_insets_match(
        "navigation-menu-demo.home-mobile-vp375x320",
        "navigation-menu-viewport",
        "open",
        "Home",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
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

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_viewport_shadow_matches_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_shadow_insets_match(
        "navigation-menu-demo.components-mobile",
        "navigation-menu-viewport",
        "open",
        "Components",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let content = cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Px(Px(275.94));
                        layout.size.height = fret_ui::element::Length::Px(Px(474.33));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(vec![
                    NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    NavigationMenuItem::new("components", "Components", vec![content]),
                ])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_viewport_shadow_matches_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_shadow_insets_match(
        "navigation-menu-demo.components-mobile",
        "navigation-menu-viewport",
        "open",
        "Components",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let content = cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Px(Px(275.94));
                        layout.size.height = fret_ui::element::Length::Px(Px(474.33));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(vec![
                    NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    NavigationMenuItem::new("components", "Components", vec![content]),
                ])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_constrained_viewport_shadow_matches_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_shadow_insets_match(
        "navigation-menu-demo.components-mobile-vp375x320",
        "navigation-menu-viewport",
        "open",
        "Components",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let content = cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Px(Px(275.94));
                        layout.size.height = fret_ui::element::Length::Px(Px(474.33));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(vec![
                    NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    NavigationMenuItem::new("components", "Components", vec![content]),
                ])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_constrained_viewport_shadow_matches_web_dark()
{
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_shadow_insets_match(
        "navigation-menu-demo.components-mobile-vp375x320",
        "navigation-menu-viewport",
        "open",
        "Components",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let content = cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Px(Px(275.94));
                        layout.size.height = fret_ui::element::Length::Px(Px(474.33));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(vec![
                    NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    NavigationMenuItem::new("components", "Components", vec![content]),
                ])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_surface_colors_match(
        "navigation-menu-demo.components-mobile",
        "navigation-menu-viewport",
        "open",
        "Components",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let content = cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Px(Px(275.94));
                        layout.size.height = fret_ui::element::Length::Px(Px(474.33));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(vec![
                    NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    NavigationMenuItem::new("components", "Components", vec![content]),
                ])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_surface_colors_match(
        "navigation-menu-demo.components-mobile",
        "navigation-menu-viewport",
        "open",
        "Components",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let content = cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Px(Px(275.94));
                        layout.size.height = fret_ui::element::Length::Px(Px(474.33));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(vec![
                    NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    NavigationMenuItem::new("components", "Components", vec![content]),
                ])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_constrained_viewport_surface_colors_match_web()
 {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_surface_colors_match(
        "navigation-menu-demo.components-mobile-vp375x320",
        "navigation-menu-viewport",
        "open",
        "Components",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let content = cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Px(Px(275.94));
                        layout.size.height = fret_ui::element::Length::Px(Px(474.33));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(vec![
                    NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    NavigationMenuItem::new("components", "Components", vec![content]),
                ])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_constrained_viewport_surface_colors_match_web_dark()
 {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_surface_colors_match(
        "navigation-menu-demo.components-mobile-vp375x320",
        "navigation-menu-viewport",
        "open",
        "Components",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let content = cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Px(Px(275.94));
                        layout.size.height = fret_ui::element::Length::Px(Px(474.33));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(vec![
                    NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    NavigationMenuItem::new("components", "Components", vec![content]),
                ])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_indicator_shadow_matches_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_indicator_shadow_insets_match(
        "navigation-menu-demo-indicator",
        "navigation-menu-indicator",
        "visible",
        "Home",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(true)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Home content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_indicator_shadow_matches_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_indicator_shadow_insets_match(
        "navigation-menu-demo-indicator",
        "navigation-menu-indicator",
        "visible",
        "Home",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(true)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Home content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
