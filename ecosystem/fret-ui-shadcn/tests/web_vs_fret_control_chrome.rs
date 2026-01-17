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
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(200.0)),
    );

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
