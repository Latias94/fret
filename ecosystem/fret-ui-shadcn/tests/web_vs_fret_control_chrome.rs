use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Scene, SceneOp, SemanticsRole, Size as CoreSize};
use fret_ui::tree::UiTree;
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
    root: WebNode,
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
    attrs: BTreeMap<String, String>,
    #[serde(default)]
    text: Option<String>,
    rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    computed_style: BTreeMap<String, String>,
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

fn web_corner_radius_effective_px_for(node: &WebNode, key: &str) -> Option<f32> {
    let raw = node
        .computed_style
        .get(key)
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

fn find_best_quad(scene: &Scene, target: Rect) -> Option<PaintedQuad> {
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

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        if score < best_score {
            best_score = score;
            best = Some(PaintedQuad {
                rect,
                border: [border.top.0, border.right.0, border.bottom.0, border.left.0],
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

fn render_and_paint(
    render: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (fret_core::SemanticsSnapshot, Scene) {
    render_and_paint_in_bounds(CoreSize::new(Px(1024.0), Px(768.0)), render)
}

fn render_and_paint_in_bounds(
    size: CoreSize,
    render: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-control-chrome",
        render,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    (snap, scene)
}

fn assert_close(label: &str, actual: f32, expected: f32, tol: f32) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected≈{expected} (±{tol}) got={actual} (Δ={delta})"
    );
}

#[test]
fn web_vs_fret_input_demo_control_chrome_matches() {
    let web = read_web_golden("input-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_input = find_first(&theme.root, &|n| {
        n.tag == "input" && (n.rect.h - 36.0).abs() <= 0.1
    })
    .expect("web input node");

    let web_border = web_border_width_px(web_input).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_input).expect("web radius px");

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::Input::new(model)
                .a11y_label("Input")
                .into_element(cx),
        ]
    });

    let input = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::TextField && n.label.as_deref() == Some("Input"))
        .or_else(|| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::TextField)
        })
        .expect("fret input semantics node");

    let quad = find_best_quad(&scene, input.bounds).expect("painted quad for input");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("input border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("input radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_badge_demo_chrome_matches() {
    let web = read_web_golden("badge-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_badge = find_first(&theme.root, &|n| {
        n.tag == "span" && n.text.as_deref() == Some("Badge")
    })
    .expect("web badge node");

    let web_border = web_border_width_px(web_badge).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_badge).expect("web radius px");
    let web_w = web_badge.rect.w;
    let web_h = web_badge.rect.h;

    let (_snap, scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Badge::new("Badge")
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(web_h))),
                )
                .into_element(cx),
        ]
    });

    let target = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(web_w), Px(web_h)),
    );
    let quad = find_best_quad(&scene, target).expect("painted quad for badge");

    assert_close("badge width", quad.rect.size.width.0, web_w, 1.0);
    assert_close("badge height", quad.rect.size.height.0, web_h, 1.0);
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("badge border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("badge radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

fn assert_badge_variant_chrome_matches(
    web_name: &str,
    label: &'static str,
    variant: fret_ui_shadcn::BadgeVariant,
) {
    let web = read_web_golden(web_name);
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_badge = find_first(&theme.root, &|n| {
        n.tag == "span" && n.text.as_deref() == Some(label)
    })
    .expect("web badge node");

    let web_border = web_border_width_px(web_badge).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_badge).expect("web radius px");
    let web_w = web_badge.rect.w;
    let web_h = web_badge.rect.h;

    let (_snap, scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Badge::new(label)
                .variant(variant)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(web_h))),
                )
                .into_element(cx),
        ]
    });

    let target = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(web_w), Px(web_h)),
    );
    let quad = find_best_quad(&scene, target).expect("painted quad for badge");

    assert_close("badge width", quad.rect.size.width.0, web_w, 1.0);
    assert_close("badge height", quad.rect.size.height.0, web_h, 1.0);
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("badge border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("badge radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_badge_secondary_chrome_matches() {
    assert_badge_variant_chrome_matches(
        "badge-secondary",
        "Secondary",
        fret_ui_shadcn::BadgeVariant::Secondary,
    );
}

#[test]
fn web_vs_fret_badge_destructive_chrome_matches() {
    assert_badge_variant_chrome_matches(
        "badge-destructive",
        "Destructive",
        fret_ui_shadcn::BadgeVariant::Destructive,
    );
}

#[test]
fn web_vs_fret_badge_outline_chrome_matches() {
    assert_badge_variant_chrome_matches(
        "badge-outline",
        "Outline",
        fret_ui_shadcn::BadgeVariant::Outline,
    );
}

#[test]
fn web_vs_fret_kbd_demo_key_chrome_matches() {
    let web = read_web_golden("kbd-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_kbd = find_first(&theme.root, &|n| {
        n.tag == "kbd" && n.text.as_deref() == Some("B")
    })
    .expect("web kbd node (B)");

    let web_border = web_border_width_px(web_kbd).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_kbd).expect("web radius px");
    let web_w = web_kbd.rect.w;
    let web_h = web_kbd.rect.h;

    let (_snap, scene) =
        render_and_paint(|cx| vec![fret_ui_shadcn::Kbd::new("B").into_element(cx)]);

    let target = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(web_w), Px(web_h)),
    );
    let quad = find_best_quad(&scene, target).expect("painted quad for kbd");

    assert_close("kbd width", quad.rect.size.width.0, web_w, 1.0);
    assert_close("kbd height", quad.rect.size.height.0, web_h, 1.0);
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("kbd border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("kbd radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_separator_demo_geometry_matches() {
    let web = read_web_golden("separator-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_sep_h = find_first(&theme.root, &|n| {
        n.tag == "div" && (n.rect.h - 1.0).abs() <= 0.1
    })
    .expect("web horizontal separator node");
    let web_sep_v = find_first(&theme.root, &|n| {
        n.tag == "div" && (n.rect.w - 1.0).abs() <= 0.1
    })
    .expect("web vertical separator node");

    // Horizontal separator: fill width at y=0.
    let (_snap, scene) =
        render_and_paint_in_bounds(CoreSize::new(Px(web_sep_h.rect.w), Px(80.0)), |cx| {
            vec![
                fret_ui_shadcn::Separator::new()
                    .orientation(fret_ui_shadcn::SeparatorOrientation::Horizontal)
                    .into_element(cx),
            ]
        });
    let target = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(web_sep_h.rect.w), Px(web_sep_h.rect.h)),
    );
    let quad = find_best_quad(&scene, target).expect("painted quad for horizontal separator");
    assert_close(
        "separator horizontal width",
        quad.rect.size.width.0,
        web_sep_h.rect.w,
        1.0,
    );
    assert_close(
        "separator horizontal height",
        quad.rect.size.height.0,
        web_sep_h.rect.h,
        0.6,
    );

    // Vertical separator: fill height at y=0.
    let (_snap, scene) =
        render_and_paint_in_bounds(CoreSize::new(Px(80.0), Px(web_sep_v.rect.h)), |cx| {
            vec![
                fret_ui_shadcn::Separator::new()
                    .orientation(fret_ui_shadcn::SeparatorOrientation::Vertical)
                    .into_element(cx),
            ]
        });
    let target = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(web_sep_v.rect.w), Px(web_sep_v.rect.h)),
    );
    let quad = find_best_quad(&scene, target).expect("painted quad for vertical separator");
    assert_close(
        "separator vertical width",
        quad.rect.size.width.0,
        web_sep_v.rect.w,
        0.6,
    );
    assert_close(
        "separator vertical height",
        quad.rect.size.height.0,
        web_sep_v.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_toggle_group_demo_chrome_matches() {
    let debug = std::env::var("FRET_DEBUG_TOGGLE_GROUP_CHROME").is_ok();
    let web = read_web_golden("toggle-group-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_group = find_first(&theme.root, &|n| {
        if n.tag != "div" {
            return false;
        }
        if n.attrs.get("role").map(String::as_str) != Some("group") {
            return false;
        }
        if n.children.len() != 3 {
            return false;
        }
        n.children.iter().all(|c| {
            c.tag == "button"
                && c.attrs
                    .get("aria-label")
                    .map(String::as_str)
                    .is_some_and(|v| v.starts_with("Toggle "))
        })
    })
    .expect("web toggle group node");

    let web_items = web_group
        .children
        .iter()
        .filter(|n| n.tag == "button")
        .collect::<Vec<_>>();
    assert_eq!(web_items.len(), 3, "expected 3 toggle group items");

    let (_snap, scene) = render_and_paint(|cx| {
        use fret_icons::ids::ui as icon_ids;

        let i1 = fret_ui_shadcn::ToggleGroupItem::new(
            "bold",
            vec![fret_ui_shadcn::icon::icon(cx, icon_ids::CHECK.clone())],
        )
        .a11y_label("Toggle bold");
        let i2 = fret_ui_shadcn::ToggleGroupItem::new(
            "italic",
            vec![fret_ui_shadcn::icon::icon(cx, icon_ids::CHEVRON_UP.clone())],
        )
        .a11y_label("Toggle italic");
        let i3 = fret_ui_shadcn::ToggleGroupItem::new(
            "strike",
            vec![fret_ui_shadcn::icon::icon(cx, icon_ids::CLOSE.clone())],
        )
        .a11y_label("Toggle strikethrough");

        vec![
            fret_ui_shadcn::ToggleGroup::single_uncontrolled::<&str>(None)
                .variant(fret_ui_shadcn::ToggleVariant::Outline)
                .items([i1, i2, i3])
                .into_element(cx),
        ]
    });

    for (idx, web_item) in web_items.into_iter().enumerate() {
        let web_w = web_item.rect.w;
        let web_h = web_item.rect.h;
        let target = Rect::new(
            Point::new(Px(web_item.rect.x), Px(web_item.rect.y)),
            CoreSize::new(Px(web_w), Px(web_h)),
        );
        let quad = find_best_quad(&scene, target).expect("painted quad for toggle group item");

        if debug {
            eprintln!(
                "toggle-group item[{idx}] web_rect=({},{} {}x{}) quad_rect=({},{} {}x{}) border={:?} corners={:?}",
                web_item.rect.x,
                web_item.rect.y,
                web_item.rect.w,
                web_item.rect.h,
                quad.rect.origin.x.0,
                quad.rect.origin.y.0,
                quad.rect.size.width.0,
                quad.rect.size.height.0,
                quad.border,
                quad.corners,
            );
        }

        assert_close(
            &format!("toggle-group item[{idx}] width"),
            quad.rect.size.width.0,
            web_w,
            1.0,
        );
        assert_close(
            &format!("toggle-group item[{idx}] height"),
            quad.rect.size.height.0,
            web_h,
            1.0,
        );

        let web_border_top = web_item
            .computed_style
            .get("borderTopWidth")
            .map(String::as_str)
            .and_then(parse_px)
            .expect("borderTopWidth px");
        let web_border_right = web_item
            .computed_style
            .get("borderRightWidth")
            .map(String::as_str)
            .and_then(parse_px)
            .expect("borderRightWidth px");
        let web_border_bottom = web_item
            .computed_style
            .get("borderBottomWidth")
            .map(String::as_str)
            .and_then(parse_px)
            .expect("borderBottomWidth px");
        let web_border_left = web_item
            .computed_style
            .get("borderLeftWidth")
            .map(String::as_str)
            .and_then(parse_px)
            .expect("borderLeftWidth px");
        let expected_border = [
            web_border_top,
            web_border_right,
            web_border_bottom,
            web_border_left,
        ];

        for (edge_idx, (actual, expected)) in quad.border.iter().zip(expected_border).enumerate() {
            assert_close(
                &format!("toggle-group item[{idx}] border[{edge_idx}]"),
                *actual,
                expected,
                0.6,
            );
        }

        let expected_corners = [
            web_corner_radius_effective_px_for(web_item, "borderTopLeftRadius")
                .expect("borderTopLeftRadius px"),
            web_corner_radius_effective_px_for(web_item, "borderTopRightRadius")
                .expect("borderTopRightRadius px"),
            web_corner_radius_effective_px_for(web_item, "borderBottomRightRadius")
                .expect("borderBottomRightRadius px"),
            web_corner_radius_effective_px_for(web_item, "borderBottomLeftRadius")
                .expect("borderBottomLeftRadius px"),
        ];
        for (corner_idx, (actual, expected)) in
            quad.corners.iter().zip(expected_corners).enumerate()
        {
            assert_close(
                &format!("toggle-group item[{idx}] radius[{corner_idx}]"),
                *actual,
                expected,
                1.0,
            );
        }
    }
}

#[test]
fn web_vs_fret_button_demo_control_chrome_matches() {
    let web = read_web_golden("button-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_button = find_first(&theme.root, &|n| {
        n.tag == "button" && !n.attrs.contains_key("aria-label")
    })
    .expect("web button node");

    let web_border = web_border_width_px(web_button).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_button).expect("web radius px");
    let web_w = web_button.rect.w;
    let web_h = web_button.rect.h;

    let (snap, scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Button::new("Button")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(web_h))),
                )
                .into_element(cx),
        ]
    });

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Button"))
        .or_else(|| snap.nodes.iter().find(|n| n.role == SemanticsRole::Button))
        .expect("fret button semantics node");

    let quad = find_best_quad(&scene, button.bounds).expect("painted quad for button");

    assert_close("button width", quad.rect.size.width.0, web_w, 1.0);
    assert_close("button height", quad.rect.size.height.0, web_h, 1.0);
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("button border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("button radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_button_icon_control_chrome_matches() {
    let web = read_web_golden("button-icon");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_button = find_first(&theme.root, &|n| n.tag == "button").expect("web button node");

    let web_border = web_border_width_px(web_button).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_button).expect("web radius px");
    let web_w = web_button.rect.w;
    let web_h = web_button.rect.h;

    let (snap, scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Button::new("Icon")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Icon)
                .into_element(cx),
        ]
    });

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button)
        .expect("fret button semantics node");

    let quad = find_best_quad(&scene, button.bounds).expect("painted quad for button");

    assert_close("button-icon width", quad.rect.size.width.0, web_w, 1.0);
    assert_close("button-icon height", quad.rect.size.height.0, web_h, 1.0);
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("button-icon border[{idx}]"),
            *edge,
            web_border,
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("button-icon radius[{idx}]"),
            *corner,
            web_radius,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_button_loading_control_chrome_matches() {
    let web = read_web_golden("button-loading");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_button = find_first(&theme.root, &|n| n.tag == "button").expect("web button node");

    let web_border = web_border_width_px(web_button).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_button).expect("web radius px");
    let web_w = web_button.rect.w;
    let web_h = web_button.rect.h;

    let (snap, scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Button::new("Submit")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .disabled(true)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(web_h))),
                )
                .into_element(cx),
        ]
    });

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button)
        .expect("fret button semantics node");

    let quad = find_best_quad(&scene, button.bounds).expect("painted quad for button");

    assert_close("button-loading width", quad.rect.size.width.0, web_w, 1.0);
    assert_close("button-loading height", quad.rect.size.height.0, web_h, 1.0);
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("button-loading border[{idx}]"),
            *edge,
            web_border,
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("button-loading radius[{idx}]"),
            *corner,
            web_radius,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_button_rounded_control_chrome_matches() {
    let web = read_web_golden("button-rounded");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_button = find_first(&theme.root, &|n| n.tag == "button").expect("web button node");

    let web_border = web_border_width_px(web_button).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_button).expect("web radius px");
    let web_w = web_button.rect.w;
    let web_h = web_button.rect.h;

    let (snap, scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Button::new("Up")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Icon)
                .refine_style(
                    fret_ui_kit::ChromeRefinement::default().rounded(fret_ui_kit::Radius::Full),
                )
                .into_element(cx),
        ]
    });

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button)
        .expect("fret button semantics node");

    let quad = find_best_quad(&scene, button.bounds).expect("painted quad for button");

    assert_close("button-rounded width", quad.rect.size.width.0, web_w, 1.0);
    assert_close("button-rounded height", quad.rect.size.height.0, web_h, 1.0);
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("button-rounded border[{idx}]"),
            *edge,
            web_border,
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("button-rounded radius[{idx}]"),
            *corner,
            web_radius,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_button_with_icon_control_chrome_matches() {
    let web = read_web_golden("button-with-icon");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_button = find_first(&theme.root, &|n| n.tag == "button").expect("web button node");

    let web_border = web_border_width_px(web_button).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_button).expect("web radius px");
    let web_w = web_button.rect.w;
    let web_h = web_button.rect.h;

    let (snap, scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Button::new("New Branch")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(web_h))),
                )
                .into_element(cx),
        ]
    });

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button)
        .expect("fret button semantics node");

    let quad = find_best_quad(&scene, button.bounds).expect("painted quad for button");

    assert_close("button-with-icon width", quad.rect.size.width.0, web_w, 1.0);
    assert_close(
        "button-with-icon height",
        quad.rect.size.height.0,
        web_h,
        1.0,
    );
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("button-with-icon border[{idx}]"),
            *edge,
            web_border,
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("button-with-icon radius[{idx}]"),
            *corner,
            web_radius,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_textarea_demo_control_chrome_matches() {
    let web = read_web_golden("textarea-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_textarea = find_first(&theme.root, &|n| {
        n.tag == "textarea" && (n.rect.h - 64.0).abs() <= 0.1
    })
    .expect("web textarea node");

    let web_border = web_border_width_px(web_textarea).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_textarea).expect("web radius px");

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::Textarea::new(model)
                .a11y_label("Textarea")
                .into_element(cx),
        ]
    });

    let textarea = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::TextField && n.label.as_deref() == Some("Textarea"))
        .or_else(|| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::TextField)
        })
        .expect("fret textarea semantics node");

    let quad = find_best_quad(&scene, textarea.bounds).expect("painted quad for textarea");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("textarea border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("textarea radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_select_scrollable_trigger_chrome_matches() {
    let web = read_web_golden("select-scrollable");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_trigger = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|v| v == "combobox")
            && n.attrs.get("aria-expanded").is_some_and(|v| v == "false")
            && (n.rect.h - 36.0).abs() <= 0.1
    })
    .expect("web select trigger node");

    let web_border = web_border_width_px(web_trigger).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_trigger).expect("web radius px");

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
        let open: fret_runtime::Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Select::new(model, open)
                .a11y_label("Select")
                .item(fret_ui_shadcn::SelectItem::new("one", "One"))
                .item(fret_ui_shadcn::SelectItem::new("two", "Two"))
                .into_element(cx),
        ]
    });

    let select = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox && n.label.as_deref() == Some("Select"))
        .or_else(|| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::ComboBox)
        })
        .expect("fret select semantics node");

    let quad = find_best_quad(&scene, select.bounds).expect("painted quad for select trigger");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("select border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("select radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_switch_demo_track_chrome_matches() {
    let web = read_web_golden("switch-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_switch = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.id.as_deref() == Some("airplane-mode")
            && n.attrs.get("role").is_some_and(|v| v == "switch")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
    })
    .expect("web switch track node");

    let web_border = web_border_width_px(web_switch).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_switch).expect("web radius px");

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Switch::new(model)
                .a11y_label("Switch")
                .into_element(cx),
        ]
    });

    let switch = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Switch && n.label.as_deref() == Some("Switch"))
        .or_else(|| snap.nodes.iter().find(|n| n.role == SemanticsRole::Switch))
        .expect("fret switch semantics node");

    let quad = find_best_quad(&scene, switch.bounds).expect("painted quad for switch");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("switch border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("switch radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_checkbox_demo_control_chrome_matches() {
    let web = read_web_golden("checkbox-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_checkbox = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.id.as_deref() == Some("terms")
            && n.attrs.get("role").is_some_and(|v| v == "checkbox")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
    })
    .expect("web checkbox control node");

    let web_border = web_border_width_px(web_checkbox).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_checkbox).expect("web radius px");

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Checkbox::new(model)
                .a11y_label("Checkbox")
                .into_element(cx),
        ]
    });

    let checkbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Checkbox && n.label.as_deref() == Some("Checkbox"))
        .or_else(|| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Checkbox)
        })
        .expect("fret checkbox semantics node");

    let quad = find_best_quad(&scene, checkbox.bounds).expect("painted quad for checkbox");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("checkbox border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("checkbox radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_slider_demo_thumb_chrome_matches() {
    let web = read_web_golden("slider-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_thumb = find_first(&theme.root, &|n| {
        n.tag == "span"
            && n.attrs.get("role").is_some_and(|v| v == "slider")
            && (n.rect.w - 16.0).abs() <= 0.1
            && (n.rect.h - 16.0).abs() <= 0.1
    })
    .expect("web slider thumb node");

    let web_border = web_border_width_px(web_thumb).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_thumb).expect("web radius px");

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<Vec<f32>> = cx.app.models_mut().insert(vec![50.0]);
        vec![
            fret_ui_shadcn::Slider::new(model)
                .range(0.0, 100.0)
                .a11y_label("Slider")
                .into_element(cx),
        ]
    });

    let slider = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Slider && n.label.as_deref() == Some("Slider"))
        .or_else(|| snap.nodes.iter().find(|n| n.role == SemanticsRole::Slider))
        .expect("fret slider semantics node");

    let quad = find_best_quad(&scene, slider.bounds).expect("painted quad for slider thumb");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("slider border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("slider radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_radio_group_demo_control_chrome_matches() {
    let debug = std::env::var("FRET_DEBUG_RADIO_CHROME").is_ok();
    let web = read_web_golden("radio-group-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_radio = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|v| v == "radio")
            && n.id.as_deref() == Some("r1")
            && (n.rect.w - 16.0).abs() <= 0.1
            && (n.rect.h - 16.0).abs() <= 0.1
    })
    .expect("web radio control node");

    let web_border = web_border_width_px(web_radio).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_radio).expect("web radius px");

    let (snap, scene) = render_and_paint(|cx| {
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

    let radio_row = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::RadioButton && n.label.as_deref() == Some("Default"))
        .or_else(|| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::RadioButton)
        })
        .expect("fret radio semantics node");

    let target = Rect::new(radio_row.bounds.origin, CoreSize::new(Px(16.0), Px(16.0)));
    let quad = find_best_quad(&scene, target).expect("painted quad for radio control");
    if debug {
        eprintln!(
            "radio target origin=({},{}), quad_rect=({},{} {}x{}), border={:?}, corners={:?}",
            target.origin.x.0,
            target.origin.y.0,
            quad.rect.origin.x.0,
            quad.rect.origin.y.0,
            quad.rect.size.width.0,
            quad.rect.size.height.0,
            quad.border,
            quad.corners,
        );

        let mut candidates: Vec<PaintedQuad> = Vec::new();
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
            let score = (rect.origin.x.0 - target.origin.x.0).abs()
                + (rect.origin.y.0 - target.origin.y.0).abs()
                + (rect.size.width.0 - target.size.width.0).abs()
                + (rect.size.height.0 - target.size.height.0).abs();
            if score <= 8.0 {
                candidates.push(PaintedQuad {
                    rect,
                    border: [border.top.0, border.right.0, border.bottom.0, border.left.0],
                    corners: [
                        corner_radii.top_left.0,
                        corner_radii.top_right.0,
                        corner_radii.bottom_right.0,
                        corner_radii.bottom_left.0,
                    ],
                });
            }
        }
        candidates.sort_by(|a, b| {
            let score_a = (a.rect.origin.x.0 - target.origin.x.0).abs()
                + (a.rect.origin.y.0 - target.origin.y.0).abs()
                + (a.rect.size.width.0 - target.size.width.0).abs()
                + (a.rect.size.height.0 - target.size.height.0).abs();
            let score_b = (b.rect.origin.x.0 - target.origin.x.0).abs()
                + (b.rect.origin.y.0 - target.origin.y.0).abs()
                + (b.rect.size.width.0 - target.size.width.0).abs()
                + (b.rect.size.height.0 - target.size.height.0).abs();
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (idx, cand) in candidates.into_iter().take(6).enumerate() {
            eprintln!(
                "radio cand[{idx}] rect=({},{} {}x{}) border={:?} corners={:?}",
                cand.rect.origin.x.0,
                cand.rect.origin.y.0,
                cand.rect.size.width.0,
                cand.rect.size.height.0,
                cand.border,
                cand.corners,
            );
        }
    }

    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("radio border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("radio radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_progress_demo_control_chrome_matches() {
    let web = read_web_golden("progress-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_track = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "progressbar")
    })
    .expect("web progressbar node");

    let web_border = web_border_width_px(web_track).expect("web border width px");
    let web_radius = web_corner_radius_effective_px(web_track).expect("web radius px");
    let web_w = web_track.rect.w;
    let web_h = web_track.rect.h;

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<f32> = cx.app.models_mut().insert(42.0);
        vec![
            fret_ui_shadcn::Progress::new(model)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_w))),
                )
                .into_element(cx),
        ]
    });
    drop(snap);

    let target = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(web_w), Px(web_h)),
    );
    let quad = find_best_quad(&scene, target).expect("painted quad for progress track");

    assert_close("progress width", quad.rect.size.width.0, web_w, 1.0);
    assert_close("progress height", quad.rect.size.height.0, web_h, 1.0);
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("progress border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("progress radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_toggle_demo_control_chrome_matches() {
    let web = read_web_golden("toggle-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_toggle = find_first(&theme.root, &|n| n.tag == "button").expect("web toggle node");
    let web_border = web_border_width_px(web_toggle).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_toggle).expect("web radius px");
    let web_w = web_toggle.rect.w;
    let web_h = web_toggle.rect.h;

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Toggle::new(model)
                // Web `toggle-demo` is `size="sm" variant="outline"` (shadcn v4 registry example).
                .variant(fret_ui_shadcn::ToggleVariant::Outline)
                .size(fret_ui_shadcn::ToggleSize::Sm)
                .a11y_label("Toggle bookmark")
                .label("Bookmark")
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(web_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(web_h))),
                )
                .into_element(cx),
        ]
    });

    let toggle = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Toggle bookmark"))
        .or_else(|| snap.nodes.iter().find(|n| n.role == SemanticsRole::Button))
        .expect("fret toggle semantics node");

    let quad = find_best_quad(&scene, toggle.bounds).expect("painted quad for toggle");

    assert_close("toggle width", quad.rect.size.width.0, web_w, 1.0);
    assert_close("toggle height", quad.rect.size.height.0, web_h, 1.0);
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("toggle border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("toggle radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_alert_demo_chrome_matches() {
    let web = read_web_golden("alert-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_alert = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "alert")
    })
    .expect("web alert node");
    let web_border = web_border_width_px(web_alert).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_alert).expect("web radius px");
    let web_w = web_alert.rect.w;

    let (snap, scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Alert::new(vec![
                fret_ui_shadcn::AlertTitle::new("Heads up!").into_element(cx),
                fret_ui_shadcn::AlertDescription::new("You can add components to your app.")
                    .into_element(cx),
            ])
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(fret_ui_kit::MetricRef::Px(Px(web_w))),
            )
            .into_element(cx),
        ]
    });

    let alert = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Alert)
        .expect("fret alert semantics node");
    let quad = find_best_quad(&scene, alert.bounds).expect("painted quad for alert");

    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("alert border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("alert radius[{idx}]"), *corner, web_radius, 1.0);
    }
}

#[test]
fn web_vs_fret_alert_destructive_chrome_matches() {
    let web = read_web_golden("alert-destructive");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_alert = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "alert")
    })
    .expect("web alert node");
    let web_border = web_border_width_px(web_alert).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_alert).expect("web radius px");
    let web_w = web_alert.rect.w;

    let (snap, scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Alert::new(vec![
                fret_ui_shadcn::AlertTitle::new("Heads up!").into_element(cx),
                fret_ui_shadcn::AlertDescription::new("You can add components to your app.")
                    .into_element(cx),
            ])
            .variant(fret_ui_shadcn::AlertVariant::Destructive)
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(fret_ui_kit::MetricRef::Px(Px(web_w))),
            )
            .into_element(cx),
        ]
    });

    let alert = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Alert)
        .expect("fret alert semantics node");
    let quad = find_best_quad(&scene, alert.bounds).expect("painted quad for alert");

    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("alert border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("alert radius[{idx}]"), *corner, web_radius, 1.0);
    }
}
