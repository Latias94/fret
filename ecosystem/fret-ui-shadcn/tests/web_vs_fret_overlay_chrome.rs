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
    rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    computed_style: BTreeMap<String, String>,
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
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
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
                    .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
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
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
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
                                .w_px(Px(320.0))
                                .h_px(Px(245.33334)),
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
                                .w_px(Px(320.0))
                                .h_px(Px(245.33334)),
                        )
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    assert_overlay_surface_colors_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
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
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    assert_overlay_surface_colors_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
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
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_shadow_matches_web() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    assert_overlay_shadow_insets_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
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
        },
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_shadow_matches_web_dark() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    assert_overlay_shadow_insets_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
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
fn web_vs_fret_select_scrollable_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "select-scrollable",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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
fn web_vs_fret_select_scrollable_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "select-scrollable",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
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
