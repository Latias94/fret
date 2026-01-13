use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, Point, Px, Rect, Scene, SceneOp, SemanticsRole, Size as CoreSize,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
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

fn parse_px(s: &str) -> Option<f32> {
    let s = s.trim();
    let v = s.strip_suffix("px").unwrap_or(s);
    v.parse::<f32>().ok()
}

fn web_border_width_px(node: &WebNode) -> Option<f32> {
    node.computed_style
        .get("borderTopWidth")
        .map(String::as_str)
        .and_then(parse_px)
}

fn web_corner_radius_effective_px(node: &WebNode) -> Option<f32> {
    let raw = node
        .computed_style
        .get("borderTopLeftRadius")
        .map(String::as_str)
        .and_then(parse_px)?;
    let max = node.rect.w.min(node.rect.h) * 0.5;
    Some(raw.min(max))
}

#[derive(Debug, Clone, Copy)]
struct PaintedQuad {
    #[allow(dead_code)]
    rect: Rect,
    border: [f32; 4],
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
    let mut best_containing: Option<PaintedQuad> = None;
    let mut best_area = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            ..
        } = *op
        else {
            continue;
        };

        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        if !has_border(&border) {
            continue;
        }

        if rect_contains(rect, target) {
            let area = rect_area(rect);
            if area < best_area {
                best_area = area;
                best_containing = Some(PaintedQuad {
                    rect,
                    border,
                    corners: [
                        corner_radii.top_left.0,
                        corner_radii.top_right.0,
                        corner_radii.bottom_right.0,
                        corner_radii.bottom_left.0,
                    ],
                });
            }
        }
    }

    if best_containing.is_some() {
        return best_containing;
    }

    // Fallback: if containment matching fails (e.g. semantics bounds already include the border),
    // use a best-effort score match.
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;
    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            ..
        } = *op
        else {
            continue;
        };

        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        if !has_border(&border) {
            continue;
        }

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        if score < best_score {
            best_score = score;
            best = Some(PaintedQuad {
                rect,
                border,
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

struct FakeServices;

impl fret_core::TextService for FakeServices {
    fn prepare(
        &mut self,
        _text: &str,
        _style: &fret_core::TextStyle,
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

fn assert_overlay_chrome_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = find_portal_by_role(theme, web_portal_role).expect("web portal root by role");
    let web_border = web_border_width_px(web_portal).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_portal).expect("web radius px");

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

    let overlay = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let quad =
        find_best_chrome_quad(&scene, overlay.bounds).expect("painted quad for overlay panel");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("{web_name} border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius,
            1.0,
        );
    }
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
