use fret_app::App;
use fret_core::{
    AppWindowId, Color, Corners, Event, KeyCode, Modifiers, Paint, Point, Px, Rect, Scene, SceneOp,
    SemanticsRole, Size as CoreSize,
};
use fret_icons::ids;
use fret_ui::tree::UiTree;
use fret_ui_kit::ChromeRefinement;
use fret_ui_kit::Space;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack as decl_stack;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use time::{Date, Month};

mod css_color;

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
    active: bool,
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

fn find_all<'a>(node: &'a WebNode, pred: &impl Fn(&'a WebNode) -> bool) -> Vec<&'a WebNode> {
    let mut out = Vec::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if pred(n) {
            out.push(n);
        }
        for child in &n.children {
            stack.push(child);
        }
    }
    out
}

fn contains_text(node: &WebNode, needle: &str) -> bool {
    if node.text.as_deref().is_some_and(|t| t.contains(needle)) {
        return true;
    }
    node.children.iter().any(|c| contains_text(c, needle))
}

fn has_descendant_attr(node: &WebNode, key: &str, value: &str) -> bool {
    if node.attrs.get(key).is_some_and(|v| v == value) {
        return true;
    }
    node.children
        .iter()
        .any(|c| has_descendant_attr(c, key, value))
}

fn parse_px(s: &str) -> Option<f32> {
    let s = s.trim();
    let v = s.strip_suffix("px").unwrap_or(s);
    v.parse::<f32>().ok()
}

fn parse_calendar_day_aria_label(label: &str) -> Option<(Date, bool)> {
    let selected = label.ends_with(", selected");
    let label = label.strip_suffix(", selected").unwrap_or(label);
    let label = label.strip_prefix("Today, ").unwrap_or(label);

    let (prefix, year) = label.rsplit_once(", ")?;
    if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let year: i32 = year.parse().ok()?;

    let (_weekday, month_and_day) = prefix.split_once(", ")?;
    let (month, day_with_suffix) = month_and_day.split_once(' ')?;
    let month_lower = month.to_lowercase();
    let month = match month_lower.as_str() {
        "january" => Month::January,
        "february" => Month::February,
        "march" => Month::March,
        "april" => Month::April,
        "may" => Month::May,
        "june" => Month::June,
        "july" => Month::July,
        "august" => Month::August,
        "september" => Month::September,
        "october" => Month::October,
        "november" => Month::November,
        "december" => Month::December,
        _ => return None,
    };

    let day_digits: String = day_with_suffix
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if day_digits.is_empty() {
        return None;
    }
    let day: u8 = day_digits.parse().ok()?;

    let date = Date::from_calendar_date(year, month, day).ok()?;
    Some((date, selected))
}

fn web_border_width_px(node: &WebNode) -> Option<f32> {
    node.computed_style
        .get("borderTopWidth")
        .map(String::as_str)
        .and_then(parse_px)
}

fn web_border_width_px_for(node: &WebNode, key: &str) -> Option<f32> {
    node.computed_style
        .get(key)
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
    border_color: Color,
    #[allow(dead_code)]
    background: Color,
    corners: [f32; 4],
}

fn find_best_quad(scene: &Scene, target: Rect) -> Option<PaintedQuad> {
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            background,
            border,
            corner_radii,
            border_paint,
            ..
        } = *op
        else {
            continue;
        };
        let fret_core::Paint::Solid(background) = background else {
            continue;
        };
        let fret_core::Paint::Solid(border_color) = border_paint else {
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
                background,
                border: [border.top.0, border.right.0, border.bottom.0, border.left.0],
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

fn assert_color_close(label: &str, actual: Color, expected_css: &str, tol: f32) {
    let expected = css_color::parse_css_color(expected_css)
        .unwrap_or_else(|| panic!("{label}: failed to parse css color: {expected_css}"));
    let actual = css_color::color_to_rgba(actual);
    assert_close(&format!("{label}.r"), actual.r, expected.r, tol);
    assert_close(&format!("{label}.g"), actual.g, expected.g, tol);
    assert_close(&format!("{label}.b"), actual.b, expected.b, tol);
    assert_close(&format!("{label}.a"), actual.a, expected.a, tol);
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

fn render_and_paint_with_focus_in_bounds(
    size: CoreSize,
    render: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
    focus: impl FnOnce(&fret_core::SemanticsSnapshot) -> fret_core::NodeId,
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
    let focus_node = focus(&snap);

    // Ensure focus-visible chrome is enabled (keyboard modality), then apply focus.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.set_focus(Some(focus_node));

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

fn web_box_shadow_focus_ring(node: &WebNode) -> Option<(String, f32)> {
    let box_shadow = node.computed_style.get("boxShadow").map(String::as_str)?;

    for layer in split_box_shadow_layers(box_shadow) {
        let Some((color, x, y, blur, spread)) = parse_box_shadow_layer(layer) else {
            continue;
        };
        if x.abs() <= 0.01 && y.abs() <= 0.01 && blur.abs() <= 0.01 && spread > 0.01 {
            return Some((color, spread));
        }
    }

    None
}

fn find_focus_ring_quad(scene: &Scene, target: Rect, spread: f32) -> Option<PaintedQuad> {
    let expected = Rect::new(
        Point::new(
            Px(target.origin.x.0 - spread),
            Px(target.origin.y.0 - spread),
        ),
        CoreSize::new(
            Px(target.size.width.0 + spread * 2.0),
            Px(target.size.height.0 + spread * 2.0),
        ),
    );

    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            background,
            border,
            corner_radii,
            border_paint,
            ..
        } = *op
        else {
            continue;
        };

        if background != Paint::TRANSPARENT {
            continue;
        }
        let fret_core::Paint::Solid(border_color) = border_paint else {
            continue;
        };
        let bw = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        if bw.iter().any(|v| (*v - spread).abs() > 0.15) {
            continue;
        }

        let score = (rect.origin.x.0 - expected.origin.x.0).abs()
            + (rect.origin.y.0 - expected.origin.y.0).abs()
            + (rect.size.width.0 - expected.size.width.0).abs()
            + (rect.size.height.0 - expected.size.height.0).abs();

        if score < best_score {
            best_score = score;
            best = Some(PaintedQuad {
                rect,
                background: Color::TRANSPARENT,
                border: bw,
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

#[test]
fn web_vs_fret_button_group_demo_button_chrome_matches() {
    let web = read_web_golden("button-group-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_top_group = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.attrs.get("role").is_some_and(|v| v == "group")
            && n.computed_style.get("gap").is_some_and(|v| v == "8px")
    })
    .expect("web top-level button group");
    let web_gap = web_top_group
        .computed_style
        .get("gap")
        .map(String::as_str)
        .and_then(parse_px)
        .expect("web button-group gap px");

    let web_go_back = find_first(&theme.root, &|n| {
        n.tag == "button" && n.attrs.get("aria-label").is_some_and(|v| v == "Go Back")
    })
    .expect("web go back button");
    let web_archive = find_first(&theme.root, &|n| {
        n.tag == "button" && contains_text(n, "Archive")
    })
    .expect("web archive button");
    let web_report = find_first(&theme.root, &|n| {
        n.tag == "button" && contains_text(n, "Report")
    })
    .expect("web report button");
    let web_snooze = find_first(&theme.root, &|n| {
        n.tag == "button" && contains_text(n, "Snooze")
    })
    .expect("web snooze button");
    let web_more_options = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "More Options")
    })
    .expect("web more options button");

    let expected_archive_border_l = web_border_width_px_for(web_archive, "borderLeftWidth")
        .expect("web Archive borderLeftWidth");
    let expected_archive_r_tl =
        web_corner_radius_effective_px_for(web_archive, "borderTopLeftRadius")
            .expect("web Archive borderTopLeftRadius");
    let expected_archive_r_tr =
        web_corner_radius_effective_px_for(web_archive, "borderTopRightRadius")
            .expect("web Archive borderTopRightRadius");

    let expected_report_border_l =
        web_border_width_px_for(web_report, "borderLeftWidth").expect("web Report borderLeftWidth");
    let expected_report_r_tl =
        web_corner_radius_effective_px_for(web_report, "borderTopLeftRadius")
            .expect("web Report borderTopLeftRadius");
    let expected_report_r_tr =
        web_corner_radius_effective_px_for(web_report, "borderTopRightRadius")
            .expect("web Report borderTopRightRadius");

    let expected_snooze_border_l =
        web_border_width_px_for(web_snooze, "borderLeftWidth").expect("web Snooze borderLeftWidth");

    let expected_more_border_l = web_border_width_px_for(web_more_options, "borderLeftWidth")
        .expect("web More Options borderLeftWidth");
    let expected_more_r_tl =
        web_corner_radius_effective_px_for(web_more_options, "borderTopLeftRadius")
            .expect("web More Options borderTopLeftRadius");
    let expected_more_r_tr =
        web_corner_radius_effective_px_for(web_more_options, "borderTopRightRadius")
            .expect("web More Options borderTopRightRadius");

    let (snap, scene) = render_and_paint(|cx| {
        let go_back = fret_ui_shadcn::Button::new("Go Back")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .size(fret_ui_shadcn::ButtonSize::Icon)
            .children(vec![decl_icon::icon(cx, ids::ui::CHEVRON_RIGHT)])
            .into();

        let archive = fret_ui_shadcn::Button::new("Archive")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .into();
        let report = fret_ui_shadcn::Button::new("Report")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .into();

        let snooze = fret_ui_shadcn::Button::new("Snooze")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .into();
        let more_options = fret_ui_shadcn::Button::new("More Options")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .size(fret_ui_shadcn::ButtonSize::Icon)
            .children(vec![decl_icon::icon(cx, ids::ui::MORE_HORIZONTAL)])
            .into();

        let top = fret_ui_shadcn::ButtonGroup::new(vec![
            fret_ui_shadcn::ButtonGroup::new(vec![go_back]).into(),
            fret_ui_shadcn::ButtonGroup::new(vec![archive, report]).into(),
            fret_ui_shadcn::ButtonGroup::new(vec![snooze, more_options]).into(),
        ]);

        vec![top.into_element(cx)]
    });

    let go_back = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Go Back"))
        .expect("fret Go Back button semantics node");
    let archive = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Archive"))
        .expect("fret Archive button semantics node");
    let report = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Report"))
        .expect("fret Report button semantics node");
    let snooze = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Snooze"))
        .expect("fret Snooze button semantics node");
    let more_options = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("More Options"))
        .expect("fret More Options button semantics node");

    let actual_gap =
        archive.bounds.origin.x.0 - (go_back.bounds.origin.x.0 + go_back.bounds.size.width.0);
    assert_close(
        "button-group-demo gap (go back → archive)",
        actual_gap,
        web_gap,
        1.0,
    );

    assert_close(
        "Go Back width",
        go_back.bounds.size.width.0,
        web_go_back.rect.w,
        1.0,
    );
    assert_close(
        "Go Back height",
        go_back.bounds.size.height.0,
        web_go_back.rect.h,
        1.0,
    );

    let quad_archive = find_best_quad(&scene, archive.bounds).expect("painted quad for Archive");
    assert_close(
        "Archive border-left width",
        quad_archive.border[3],
        expected_archive_border_l,
        0.2,
    );
    assert_close(
        "Archive top-left radius",
        quad_archive.corners[0],
        expected_archive_r_tl,
        1.0,
    );
    assert_close(
        "Archive top-right radius",
        quad_archive.corners[1],
        expected_archive_r_tr,
        1.0,
    );

    let quad_report = find_best_quad(&scene, report.bounds).expect("painted quad for Report");
    assert_close(
        "Report border-left width",
        quad_report.border[3],
        expected_report_border_l,
        0.2,
    );
    assert_close(
        "Report top-left radius",
        quad_report.corners[0],
        expected_report_r_tl,
        1.0,
    );
    assert_close(
        "Report top-right radius",
        quad_report.corners[1],
        expected_report_r_tr,
        1.0,
    );

    let quad_snooze = find_best_quad(&scene, snooze.bounds).expect("painted quad for Snooze");
    let quad_more =
        find_best_quad(&scene, more_options.bounds).expect("painted quad for More Options");
    assert_close(
        "More Options border-left width",
        quad_more.border[3],
        expected_more_border_l,
        0.2,
    );
    assert_close(
        "More Options top-left radius",
        quad_more.corners[0],
        expected_more_r_tl,
        1.0,
    );
    assert_close(
        "More Options top-right radius",
        quad_more.corners[1],
        expected_more_r_tr,
        1.0,
    );

    // The `button-group` recipe should merge borders inside the leaf group:
    // - the last button removes the left border (`border-l-0`),
    // - intermediate buttons remove both corner radii (`rounded-*-none`).
    assert_close(
        "Snooze border-left width",
        quad_snooze.border[3],
        expected_snooze_border_l,
        0.2,
    );
    assert_close(
        "More Options border-left width",
        quad_more.border[3],
        expected_more_border_l,
        0.2,
    );
}

#[test]
fn web_vs_fret_button_group_split_chrome_matches() {
    let web = read_web_golden("button-group-split");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_primary = find_first(&theme.root, &|n| {
        n.tag == "button" && contains_text(n, "Button")
    })
    .expect("web split primary button");
    let web_sep = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.attrs.get("role").is_some_and(|v| v == "none")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "vertical")
            && (n.rect.w - 1.0).abs() <= 0.1
    })
    .expect("web split separator");
    let web_icon = find_first(&theme.root, &|n| {
        n.tag == "button" && n.text.is_none() && (n.rect.w - 36.0).abs() <= 0.1
    })
    .expect("web split icon button");

    let expected_primary_border_top =
        web_border_width_px_for(web_primary, "borderTopWidth").expect("web primary borderTopWidth");
    let expected_primary_r_tl =
        web_corner_radius_effective_px_for(web_primary, "borderTopLeftRadius")
            .expect("web primary borderTopLeftRadius");
    let expected_primary_r_tr =
        web_corner_radius_effective_px_for(web_primary, "borderTopRightRadius")
            .expect("web primary borderTopRightRadius");

    let expected_icon_border_top =
        web_border_width_px_for(web_icon, "borderTopWidth").expect("web icon borderTopWidth");
    let expected_icon_r_tl = web_corner_radius_effective_px_for(web_icon, "borderTopLeftRadius")
        .expect("web icon borderTopLeftRadius");
    let expected_icon_r_tr = web_corner_radius_effective_px_for(web_icon, "borderTopRightRadius")
        .expect("web icon borderTopRightRadius");

    let (snap, scene) = render_and_paint(|cx| {
        let split = fret_ui_shadcn::ButtonGroup::new(vec![
            fret_ui_shadcn::Button::new("Button")
                .variant(fret_ui_shadcn::ButtonVariant::Secondary)
                .into(),
            fret_ui_shadcn::Separator::new()
                .orientation(fret_ui_shadcn::SeparatorOrientation::Vertical)
                .into(),
            fret_ui_shadcn::Button::new("Menu")
                .variant(fret_ui_shadcn::ButtonVariant::Secondary)
                .size(fret_ui_shadcn::ButtonSize::Icon)
                .children(vec![decl_icon::icon(cx, ids::ui::CHEVRON_DOWN)])
                .into(),
        ]);

        vec![split.into_element(cx)]
    });

    let primary = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Button"))
        .expect("fret split primary semantics node");
    let icon = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Menu"))
        .expect("fret split icon semantics node");

    assert_close(
        "split primary height",
        primary.bounds.size.height.0,
        web_primary.rect.h,
        1.0,
    );
    assert_close(
        "split icon width",
        icon.bounds.size.width.0,
        web_icon.rect.w,
        1.0,
    );
    assert_close(
        "split icon height",
        icon.bounds.size.height.0,
        web_icon.rect.h,
        1.0,
    );

    let quad_primary = find_best_quad(&scene, primary.bounds).expect("painted quad for primary");
    assert_close(
        "split primary border-top width",
        quad_primary.border[0],
        expected_primary_border_top,
        0.2,
    );
    assert_close(
        "split primary top-left radius",
        quad_primary.corners[0],
        expected_primary_r_tl,
        1.0,
    );
    assert_close(
        "split primary top-right radius",
        quad_primary.corners[1],
        expected_primary_r_tr,
        1.0,
    );

    let quad_icon = find_best_quad(&scene, icon.bounds).expect("painted quad for icon");
    assert_close(
        "split icon border-top width",
        quad_icon.border[0],
        expected_icon_border_top,
        0.2,
    );
    assert_close(
        "split icon top-left radius",
        quad_icon.corners[0],
        expected_icon_r_tl,
        1.0,
    );
    assert_close(
        "split icon top-right radius",
        quad_icon.corners[1],
        expected_icon_r_tr,
        1.0,
    );

    // Text shaping is intentionally stubbed in `FakeServices`, so we can't reliably assert the
    // intrinsic text-driven width of the primary button. Instead, gate:
    // - separator thickness,
    // - separator/chevron button placement (no gap),
    // - merged border/corner radii outcomes.
    let sep_w = web_sep.rect.w;
    let sep_h = web_sep.rect.h;
    let sep_target = Rect::new(
        Point::new(
            Px(primary.bounds.origin.x.0 + primary.bounds.size.width.0),
            primary.bounds.origin.y,
        ),
        CoreSize::new(Px(sep_w), Px(sep_h)),
    );
    let quad_sep = find_best_quad(&scene, sep_target).expect("painted quad for separator");
    assert_close(
        "split separator width",
        quad_sep.rect.size.width.0,
        sep_w,
        0.2,
    );
    assert_close(
        "split separator height",
        quad_sep.rect.size.height.0,
        sep_h,
        0.2,
    );

    let actual_icon_gap =
        icon.bounds.origin.x.0 - (quad_sep.rect.origin.x.0 + quad_sep.rect.size.width.0);
    assert_close("split separator → icon gap", actual_icon_gap, 0.0, 0.5);
}

#[test]
fn web_vs_fret_button_group_orientation_vertical_chrome_matches() {
    let web = read_web_golden("button-group-orientation");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_group = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "vertical")
    })
    .expect("web vertical button group");

    let web_top = find_first(web_group, &|n| {
        n.tag == "button" && (n.rect.y - 0.0).abs() <= 0.1
    })
    .expect("web top button");
    let web_bottom = find_first(web_group, &|n| {
        n.tag == "button" && (n.rect.y - 36.0).abs() <= 0.1
    })
    .expect("web bottom button");

    let expected_top_border_bottom =
        web_border_width_px_for(web_top, "borderBottomWidth").expect("web top borderBottomWidth");
    let expected_top_r_tl = web_corner_radius_effective_px_for(web_top, "borderTopLeftRadius")
        .expect("web top borderTopLeftRadius");
    let expected_top_r_bl = web_corner_radius_effective_px_for(web_top, "borderBottomLeftRadius")
        .expect("web top borderBottomLeftRadius");

    let expected_bottom_border_top =
        web_border_width_px_for(web_bottom, "borderTopWidth").expect("web bottom borderTopWidth");
    let expected_bottom_r_tl =
        web_corner_radius_effective_px_for(web_bottom, "borderTopLeftRadius")
            .expect("web bottom borderTopLeftRadius");
    let expected_bottom_r_bl =
        web_corner_radius_effective_px_for(web_bottom, "borderBottomLeftRadius")
            .expect("web bottom borderBottomLeftRadius");

    let (snap, scene) = render_and_paint(|cx| {
        let group = fret_ui_shadcn::ButtonGroup::new(vec![
            fret_ui_shadcn::Button::new("Play")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Icon)
                .children(vec![decl_icon::icon(cx, ids::ui::PLAY)])
                .into(),
            fret_ui_shadcn::Button::new("Stop")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Icon)
                .children(vec![decl_icon::icon(cx, ids::ui::CLOSE)])
                .into(),
        ])
        .a11y_label("Media controls")
        .orientation(fret_ui_shadcn::ButtonGroupOrientation::Vertical);

        vec![group.into_element(cx)]
    });

    let top = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Play"))
        .expect("fret top button semantics node");
    let bottom = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Stop"))
        .expect("fret bottom button semantics node");

    assert_close(
        "vertical top width",
        top.bounds.size.width.0,
        web_top.rect.w,
        1.0,
    );
    assert_close(
        "vertical top height",
        top.bounds.size.height.0,
        web_top.rect.h,
        1.0,
    );
    assert_close(
        "vertical bottom width",
        bottom.bounds.size.width.0,
        web_bottom.rect.w,
        1.0,
    );
    assert_close(
        "vertical bottom height",
        bottom.bounds.size.height.0,
        web_bottom.rect.h,
        1.0,
    );
    assert_close(
        "vertical button stacking y",
        bottom.bounds.origin.y.0 - top.bounds.origin.y.0,
        web_bottom.rect.y - web_top.rect.y,
        1.0,
    );

    let quad_top = find_best_quad(&scene, top.bounds).expect("painted quad for top button");
    assert_close(
        "vertical top border-bottom width",
        quad_top.border[2],
        expected_top_border_bottom,
        0.2,
    );
    assert_close(
        "vertical top top-left radius",
        quad_top.corners[0],
        expected_top_r_tl,
        1.0,
    );
    assert_close(
        "vertical top bottom-left radius",
        quad_top.corners[3],
        expected_top_r_bl,
        1.0,
    );

    let quad_bottom =
        find_best_quad(&scene, bottom.bounds).expect("painted quad for bottom button");
    assert_close(
        "vertical bottom border-top width",
        quad_bottom.border[0],
        expected_bottom_border_top,
        0.2,
    );
    assert_close(
        "vertical bottom top-left radius",
        quad_bottom.corners[0],
        expected_bottom_r_tl,
        1.0,
    );
    assert_close(
        "vertical bottom bottom-left radius",
        quad_bottom.corners[3],
        expected_bottom_r_bl,
        1.0,
    );
}

#[test]
fn web_vs_fret_button_group_nested_geometry_and_chrome_match() {
    let web = read_web_golden("button-group-nested");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_outer = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.attrs.get("role").is_some_and(|v| v == "group")
            && n.computed_style.get("gap").is_some_and(|v| v == "8px")
            && (n.rect.h - 32.0).abs() <= 0.1
    })
    .expect("web outer button group");

    let web_groups: Vec<&WebNode> = web_outer
        .children
        .iter()
        .filter(|c| c.tag == "div" && c.attrs.get("role").is_some_and(|v| v == "group"))
        .collect();
    assert!(
        web_groups.len() >= 2,
        "expected at least 2 nested groups, got {}",
        web_groups.len()
    );
    let web_group_a = web_groups[0];
    let web_group_b = web_groups[1];
    let expected_gap = web_group_b.rect.x - (web_group_a.rect.x + web_group_a.rect.w);

    let web_one = find_first(web_group_a, &|n| n.tag == "button" && contains_text(n, "1"))
        .expect("web button 1");
    let web_two = find_first(web_group_a, &|n| n.tag == "button" && contains_text(n, "2"))
        .expect("web button 2");
    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button" && n.attrs.get("aria-label").is_some_and(|v| v == "Previous")
    })
    .expect("web previous button");
    let web_next = find_first(&theme.root, &|n| {
        n.tag == "button" && n.attrs.get("aria-label").is_some_and(|v| v == "Next")
    })
    .expect("web next button");

    let expected_one_border_l =
        web_border_width_px_for(web_one, "borderLeftWidth").expect("web 1 borderLeftWidth");
    let expected_one_r_tl = web_corner_radius_effective_px_for(web_one, "borderTopLeftRadius")
        .expect("web 1 borderTopLeftRadius");
    let expected_one_r_tr = web_corner_radius_effective_px_for(web_one, "borderTopRightRadius")
        .expect("web 1 borderTopRightRadius");

    let expected_two_border_l =
        web_border_width_px_for(web_two, "borderLeftWidth").expect("web 2 borderLeftWidth");
    let expected_two_r_tl = web_corner_radius_effective_px_for(web_two, "borderTopLeftRadius")
        .expect("web 2 borderTopLeftRadius");
    let expected_two_r_tr = web_corner_radius_effective_px_for(web_two, "borderTopRightRadius")
        .expect("web 2 borderTopRightRadius");

    let expected_prev_border_l =
        web_border_width_px_for(web_prev, "borderLeftWidth").expect("web Previous borderLeftWidth");
    let expected_prev_r_tr = web_corner_radius_effective_px_for(web_prev, "borderTopRightRadius")
        .expect("web Previous borderTopRightRadius");
    let expected_next_border_l =
        web_border_width_px_for(web_next, "borderLeftWidth").expect("web Next borderLeftWidth");
    let expected_next_r_tl = web_corner_radius_effective_px_for(web_next, "borderTopLeftRadius")
        .expect("web Next borderTopLeftRadius");
    let expected_next_r_tr = web_corner_radius_effective_px_for(web_next, "borderTopRightRadius")
        .expect("web Next borderTopRightRadius");

    let (snap, scene) = render_and_paint(|cx| {
        let group_a = fret_ui_shadcn::ButtonGroup::new(vec![
            fret_ui_shadcn::Button::new("1")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .into(),
            fret_ui_shadcn::Button::new("2")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .into(),
            fret_ui_shadcn::Button::new("3")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .into(),
            fret_ui_shadcn::Button::new("4")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .into(),
            fret_ui_shadcn::Button::new("5")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .into(),
        ])
        .a11y_label("NestedGroupA")
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group_a.rect.w)));

        let group_b = fret_ui_shadcn::ButtonGroup::new(vec![
            fret_ui_shadcn::Button::new("Previous")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::IconSm)
                .children(vec![decl_icon::icon(cx, ids::ui::CHEVRON_RIGHT)])
                .into(),
            fret_ui_shadcn::Button::new("Next")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::IconSm)
                .children(vec![decl_icon::icon(cx, ids::ui::CHEVRON_RIGHT)])
                .into(),
        ])
        .a11y_label("NestedGroupB")
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(web_group_b.rect.w)));

        let outer = fret_ui_shadcn::ButtonGroup::new(vec![group_a.into(), group_b.into()])
            .a11y_label("OuterGroup");

        vec![outer.into_element(cx)]
    });

    let group_a = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Group && n.label.as_deref() == Some("NestedGroupA"))
        .expect("fret nested group A semantics node");
    let group_b = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Group && n.label.as_deref() == Some("NestedGroupB"))
        .expect("fret nested group B semantics node");

    let actual_gap =
        group_b.bounds.origin.x.0 - (group_a.bounds.origin.x.0 + group_a.bounds.size.width.0);
    assert_close("button-group-nested gap", actual_gap, expected_gap, 1.0);

    let one = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("1"))
        .expect("fret button 1 semantics node");
    let two = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("2"))
        .expect("fret button 2 semantics node");
    let prev = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Previous"))
        .expect("fret previous semantics node");
    let next = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Next"))
        .expect("fret next semantics node");

    let quad_one = find_best_quad(&scene, one.bounds).expect("painted quad for 1");
    assert_close(
        "button-group-nested 1 border-left",
        quad_one.border[3],
        expected_one_border_l,
        0.2,
    );
    assert_close(
        "button-group-nested 1 radius tl",
        quad_one.corners[0],
        expected_one_r_tl,
        1.0,
    );
    assert_close(
        "button-group-nested 1 radius tr",
        quad_one.corners[1],
        expected_one_r_tr,
        1.0,
    );

    let quad_two = find_best_quad(&scene, two.bounds).expect("painted quad for 2");
    assert_close(
        "button-group-nested 2 border-left",
        quad_two.border[3],
        expected_two_border_l,
        0.2,
    );
    assert_close(
        "button-group-nested 2 radius tl",
        quad_two.corners[0],
        expected_two_r_tl,
        1.0,
    );
    assert_close(
        "button-group-nested 2 radius tr",
        quad_two.corners[1],
        expected_two_r_tr,
        1.0,
    );

    let quad_prev = find_best_quad(&scene, prev.bounds).expect("painted quad for Previous");
    assert_close(
        "button-group-nested Previous border-left",
        quad_prev.border[3],
        expected_prev_border_l,
        0.2,
    );
    assert_close(
        "button-group-nested Previous radius tr",
        quad_prev.corners[1],
        expected_prev_r_tr,
        1.0,
    );

    let quad_next = find_best_quad(&scene, next.bounds).expect("painted quad for Next");
    assert_close(
        "button-group-nested Next border-left",
        quad_next.border[3],
        expected_next_border_l,
        0.2,
    );
    assert_close(
        "button-group-nested Next radius tl",
        quad_next.corners[0],
        expected_next_r_tl,
        1.0,
    );
    assert_close(
        "button-group-nested Next radius tr",
        quad_next.corners[1],
        expected_next_r_tr,
        1.0,
    );
}

#[test]
fn web_vs_fret_button_group_separator_geometry_and_chrome_match() {
    let web = read_web_golden("button-group-separator");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_copy = find_first(&theme.root, &|n| {
        n.tag == "button" && contains_text(n, "Copy")
    })
    .expect("web Copy button");
    let web_paste = find_first(&theme.root, &|n| {
        n.tag == "button" && contains_text(n, "Paste")
    })
    .expect("web Paste button");
    let web_sep = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.attrs.get("role").is_some_and(|v| v == "none")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "vertical")
            && (n.rect.w - 1.0).abs() <= 0.1
    })
    .expect("web separator node");

    let expected_copy_border_top =
        web_border_width_px_for(web_copy, "borderTopWidth").expect("web Copy borderTopWidth");
    let expected_copy_r_tl = web_corner_radius_effective_px_for(web_copy, "borderTopLeftRadius")
        .expect("web Copy borderTopLeftRadius");
    let expected_copy_r_tr = web_corner_radius_effective_px_for(web_copy, "borderTopRightRadius")
        .expect("web Copy borderTopRightRadius");

    let expected_paste_border_top =
        web_border_width_px_for(web_paste, "borderTopWidth").expect("web Paste borderTopWidth");
    let expected_paste_r_tl = web_corner_radius_effective_px_for(web_paste, "borderTopLeftRadius")
        .expect("web Paste borderTopLeftRadius");
    let expected_paste_r_tr = web_corner_radius_effective_px_for(web_paste, "borderTopRightRadius")
        .expect("web Paste borderTopRightRadius");

    let (snap, scene) = render_and_paint(|cx| {
        let group = fret_ui_shadcn::ButtonGroup::new(vec![
            fret_ui_shadcn::Button::new("Copy")
                .variant(fret_ui_shadcn::ButtonVariant::Secondary)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .into(),
            fret_ui_shadcn::Separator::new()
                .orientation(fret_ui_shadcn::SeparatorOrientation::Vertical)
                .into(),
            fret_ui_shadcn::Button::new("Paste")
                .variant(fret_ui_shadcn::ButtonVariant::Secondary)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .into(),
        ]);
        vec![group.into_element(cx)]
    });

    let copy = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Copy"))
        .expect("fret Copy button semantics node");
    let paste = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Paste"))
        .expect("fret Paste button semantics node");

    let quad_copy = find_best_quad(&scene, copy.bounds).expect("painted quad for Copy");
    assert_close(
        "button-group-separator Copy border-top",
        quad_copy.border[0],
        expected_copy_border_top,
        0.2,
    );
    assert_close(
        "button-group-separator Copy radius tl",
        quad_copy.corners[0],
        expected_copy_r_tl,
        1.0,
    );
    assert_close(
        "button-group-separator Copy radius tr",
        quad_copy.corners[1],
        expected_copy_r_tr,
        1.0,
    );

    let quad_paste = find_best_quad(&scene, paste.bounds).expect("painted quad for Paste");
    assert_close(
        "button-group-separator Paste border-top",
        quad_paste.border[0],
        expected_paste_border_top,
        0.2,
    );
    assert_close(
        "button-group-separator Paste radius tl",
        quad_paste.corners[0],
        expected_paste_r_tl,
        1.0,
    );
    assert_close(
        "button-group-separator Paste radius tr",
        quad_paste.corners[1],
        expected_paste_r_tr,
        1.0,
    );

    let sep_w = web_sep.rect.w;
    let sep_h = web_sep.rect.h;
    let sep_target = Rect::new(
        Point::new(
            Px(copy.bounds.origin.x.0 + copy.bounds.size.width.0),
            copy.bounds.origin.y,
        ),
        CoreSize::new(Px(sep_w), Px(sep_h)),
    );
    let quad_sep = find_best_quad(&scene, sep_target).expect("painted quad for separator");
    assert_close(
        "button-group-separator sep width",
        quad_sep.rect.size.width.0,
        sep_w,
        0.2,
    );
    assert_close(
        "button-group-separator sep height",
        quad_sep.rect.size.height.0,
        sep_h,
        0.2,
    );
    let actual_paste_gap =
        paste.bounds.origin.x.0 - (quad_sep.rect.origin.x.0 + quad_sep.rect.size.width.0);
    assert_close(
        "button-group-separator sep → paste gap",
        actual_paste_gap,
        0.0,
        0.5,
    );
}

#[test]
fn web_vs_fret_button_group_size_geometry_and_chrome_match() {
    let web = read_web_golden("button-group-size");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let mut web_groups: Vec<&WebNode> = Vec::new();
    fn collect_groups<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if node.tag == "div" && node.attrs.get("role").is_some_and(|v| v == "group") {
            out.push(node);
        }
        for child in &node.children {
            collect_groups(child, out);
        }
    }
    collect_groups(&theme.root, &mut web_groups);
    web_groups.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert_eq!(web_groups.len(), 3, "expected 3 button groups in golden");

    let web_sm = web_groups[0];
    let web_md = web_groups[1];
    let web_lg = web_groups[2];

    fn collect_buttons<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if node.tag == "button" {
            out.push(node);
        }
        for child in &node.children {
            collect_buttons(child, out);
        }
    }

    let mut web_sm_buttons = Vec::new();
    collect_buttons(web_sm, &mut web_sm_buttons);
    assert_eq!(
        web_sm_buttons.len(),
        4,
        "expected 4 buttons in small button group golden"
    );
    let web_sm_first = web_sm_buttons[0];
    let web_sm_mid = web_sm_buttons[1];
    let web_sm_icon = web_sm_buttons[3];

    let mut web_md_buttons = Vec::new();
    collect_buttons(web_md, &mut web_md_buttons);
    assert_eq!(
        web_md_buttons.len(),
        4,
        "expected 4 buttons in default button group golden"
    );
    let web_md_first = web_md_buttons[0];
    let web_md_mid = web_md_buttons[1];
    let web_md_icon = web_md_buttons[3];

    let mut web_lg_buttons = Vec::new();
    collect_buttons(web_lg, &mut web_lg_buttons);
    assert_eq!(
        web_lg_buttons.len(),
        4,
        "expected 4 buttons in large button group golden"
    );
    let web_lg_first = web_lg_buttons[0];
    let web_lg_mid = web_lg_buttons[1];
    let web_lg_icon = web_lg_buttons[3];

    let expected_gap_sm_to_md = web_md.rect.y - (web_sm.rect.y + web_sm.rect.h);
    let expected_gap_md_to_lg = web_lg.rect.y - (web_md.rect.y + web_md.rect.h);

    let expected_sm_first_r_tl =
        web_corner_radius_effective_px_for(web_sm_first, "borderTopLeftRadius")
            .expect("web small first borderTopLeftRadius");
    let expected_sm_first_r_tr =
        web_corner_radius_effective_px_for(web_sm_first, "borderTopRightRadius")
            .expect("web small first borderTopRightRadius");
    let expected_sm_icon_r_tr =
        web_corner_radius_effective_px_for(web_sm_icon, "borderTopRightRadius")
            .expect("web small icon borderTopRightRadius");

    let expected_md_first_r_tl =
        web_corner_radius_effective_px_for(web_md_first, "borderTopLeftRadius")
            .expect("web default first borderTopLeftRadius");
    let expected_md_first_r_tr =
        web_corner_radius_effective_px_for(web_md_first, "borderTopRightRadius")
            .expect("web default first borderTopRightRadius");
    let expected_md_icon_r_tr =
        web_corner_radius_effective_px_for(web_md_icon, "borderTopRightRadius")
            .expect("web default icon borderTopRightRadius");

    let expected_lg_first_r_tl =
        web_corner_radius_effective_px_for(web_lg_first, "borderTopLeftRadius")
            .expect("web large first borderTopLeftRadius");
    let expected_lg_first_r_tr =
        web_corner_radius_effective_px_for(web_lg_first, "borderTopRightRadius")
            .expect("web large first borderTopRightRadius");
    let expected_lg_icon_r_tr =
        web_corner_radius_effective_px_for(web_lg_icon, "borderTopRightRadius")
            .expect("web large icon borderTopRightRadius");

    let expected_sm_mid_border_l =
        web_border_width_px_for(web_sm_mid, "borderLeftWidth").expect("web small mid borderLeft");
    let expected_sm_icon_border_l =
        web_border_width_px_for(web_sm_icon, "borderLeftWidth").expect("web small icon borderLeft");

    let expected_md_mid_border_l =
        web_border_width_px_for(web_md_mid, "borderLeftWidth").expect("web default mid borderLeft");
    let expected_md_icon_border_l = web_border_width_px_for(web_md_icon, "borderLeftWidth")
        .expect("web default icon borderLeft");

    let expected_lg_mid_border_l =
        web_border_width_px_for(web_lg_mid, "borderLeftWidth").expect("web large mid borderLeft");
    let expected_lg_icon_border_l =
        web_border_width_px_for(web_lg_icon, "borderLeftWidth").expect("web large icon borderLeft");

    let (snap, scene) = render_and_paint(|cx| {
        let group_sm = fret_ui_shadcn::ButtonGroup::new(vec![
            fret_ui_shadcn::Button::new("Small")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .test_id("button-group-size.sm.first")
                .into(),
            fret_ui_shadcn::Button::new("Button")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .test_id("button-group-size.sm.mid")
                .into(),
            fret_ui_shadcn::Button::new("Group")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .test_id("button-group-size.sm.last_text")
                .into(),
            fret_ui_shadcn::Button::new("")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::IconSm)
                .children(vec![decl_icon::icon(cx, ids::ui::MORE_HORIZONTAL)])
                .test_id("button-group-size.sm.icon")
                .into(),
        ])
        .a11y_label("ButtonGroupSizeSm");

        let group_md = fret_ui_shadcn::ButtonGroup::new(vec![
            fret_ui_shadcn::Button::new("Default")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Default)
                .test_id("button-group-size.md.first")
                .into(),
            fret_ui_shadcn::Button::new("Button")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Default)
                .test_id("button-group-size.md.mid")
                .into(),
            fret_ui_shadcn::Button::new("Group")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Default)
                .test_id("button-group-size.md.last_text")
                .into(),
            fret_ui_shadcn::Button::new("")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Icon)
                .children(vec![decl_icon::icon(cx, ids::ui::MORE_HORIZONTAL)])
                .test_id("button-group-size.md.icon")
                .into(),
        ])
        .a11y_label("ButtonGroupSizeMd");

        let group_lg = fret_ui_shadcn::ButtonGroup::new(vec![
            fret_ui_shadcn::Button::new("Large")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Lg)
                .test_id("button-group-size.lg.first")
                .into(),
            fret_ui_shadcn::Button::new("Button")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Lg)
                .test_id("button-group-size.lg.mid")
                .into(),
            fret_ui_shadcn::Button::new("Group")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Lg)
                .test_id("button-group-size.lg.last_text")
                .into(),
            fret_ui_shadcn::Button::new("")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::IconLg)
                .children(vec![decl_icon::icon(cx, ids::ui::MORE_HORIZONTAL)])
                .test_id("button-group-size.lg.icon")
                .into(),
        ])
        .a11y_label("ButtonGroupSizeLg");

        vec![decl_stack::vstack(
            cx,
            decl_stack::VStackProps::default()
                .gap(Space::N8)
                .items_start(),
            move |cx| {
                vec![
                    group_sm.into_element(cx),
                    group_md.into_element(cx),
                    group_lg.into_element(cx),
                ]
            },
        )]
    });

    let group_sm = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Group && n.label.as_deref() == Some("ButtonGroupSizeSm"))
        .expect("missing semantics for small button group");
    let group_md = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Group && n.label.as_deref() == Some("ButtonGroupSizeMd"))
        .expect("missing semantics for default button group");
    let group_lg = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Group && n.label.as_deref() == Some("ButtonGroupSizeLg"))
        .expect("missing semantics for large button group");

    assert_close(
        "button-group-size small group y",
        group_sm.bounds.origin.y.0,
        web_sm.rect.y,
        1.0,
    );
    assert_close(
        "button-group-size small group height",
        group_sm.bounds.size.height.0,
        web_sm.rect.h,
        1.0,
    );
    assert_close(
        "button-group-size default group y",
        group_md.bounds.origin.y.0,
        web_md.rect.y,
        1.0,
    );
    assert_close(
        "button-group-size default group height",
        group_md.bounds.size.height.0,
        web_md.rect.h,
        1.0,
    );
    assert_close(
        "button-group-size large group y",
        group_lg.bounds.origin.y.0,
        web_lg.rect.y,
        1.0,
    );
    assert_close(
        "button-group-size large group height",
        group_lg.bounds.size.height.0,
        web_lg.rect.h,
        1.0,
    );

    let actual_gap_sm_to_md =
        group_md.bounds.origin.y.0 - (group_sm.bounds.origin.y.0 + group_sm.bounds.size.height.0);
    assert_close(
        "button-group-size gap (small → default)",
        actual_gap_sm_to_md,
        expected_gap_sm_to_md,
        1.0,
    );
    let actual_gap_md_to_lg =
        group_lg.bounds.origin.y.0 - (group_md.bounds.origin.y.0 + group_md.bounds.size.height.0);
    assert_close(
        "button-group-size gap (default → large)",
        actual_gap_md_to_lg,
        expected_gap_md_to_lg,
        1.0,
    );

    let by_id = |id: &str| {
        snap.nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some(id))
            .unwrap_or_else(|| panic!("missing semantics node {id}"))
            .bounds
    };

    let sm_first_bounds = by_id("button-group-size.sm.first");
    let sm_mid_bounds = by_id("button-group-size.sm.mid");
    let sm_icon_bounds = by_id("button-group-size.sm.icon");

    let md_first_bounds = by_id("button-group-size.md.first");
    let md_mid_bounds = by_id("button-group-size.md.mid");
    let md_icon_bounds = by_id("button-group-size.md.icon");

    let lg_first_bounds = by_id("button-group-size.lg.first");
    let lg_mid_bounds = by_id("button-group-size.lg.mid");
    let lg_icon_bounds = by_id("button-group-size.lg.icon");

    assert_close(
        "button-group-size small icon width",
        sm_icon_bounds.size.width.0,
        web_sm_icon.rect.w,
        1.0,
    );
    assert_close(
        "button-group-size small icon height",
        sm_icon_bounds.size.height.0,
        web_sm_icon.rect.h,
        1.0,
    );
    assert_close(
        "button-group-size default icon width",
        md_icon_bounds.size.width.0,
        web_md_icon.rect.w,
        1.0,
    );
    assert_close(
        "button-group-size default icon height",
        md_icon_bounds.size.height.0,
        web_md_icon.rect.h,
        1.0,
    );
    assert_close(
        "button-group-size large icon width",
        lg_icon_bounds.size.width.0,
        web_lg_icon.rect.w,
        1.0,
    );
    assert_close(
        "button-group-size large icon height",
        lg_icon_bounds.size.height.0,
        web_lg_icon.rect.h,
        1.0,
    );

    let quad_sm_first = find_best_quad(&scene, sm_first_bounds).expect("painted quad for sm first");
    let quad_sm_mid = find_best_quad(&scene, sm_mid_bounds).expect("painted quad for sm mid");
    let quad_sm_icon = find_best_quad(&scene, sm_icon_bounds).expect("painted quad for sm icon");

    assert_close(
        "button-group-size small first radius tl",
        quad_sm_first.corners[0],
        expected_sm_first_r_tl,
        1.0,
    );
    assert_close(
        "button-group-size small first radius tr",
        quad_sm_first.corners[1],
        expected_sm_first_r_tr,
        1.0,
    );
    assert_close(
        "button-group-size small icon radius tr",
        quad_sm_icon.corners[1],
        expected_sm_icon_r_tr,
        1.0,
    );
    assert_close(
        "button-group-size small mid border-left",
        quad_sm_mid.border[3],
        expected_sm_mid_border_l,
        0.2,
    );
    assert_close(
        "button-group-size small icon border-left",
        quad_sm_icon.border[3],
        expected_sm_icon_border_l,
        0.2,
    );

    let quad_md_first = find_best_quad(&scene, md_first_bounds).expect("painted quad for md first");
    let quad_md_mid = find_best_quad(&scene, md_mid_bounds).expect("painted quad for md mid");
    let quad_md_icon = find_best_quad(&scene, md_icon_bounds).expect("painted quad for md icon");

    assert_close(
        "button-group-size default first radius tl",
        quad_md_first.corners[0],
        expected_md_first_r_tl,
        1.0,
    );
    assert_close(
        "button-group-size default first radius tr",
        quad_md_first.corners[1],
        expected_md_first_r_tr,
        1.0,
    );
    assert_close(
        "button-group-size default icon radius tr",
        quad_md_icon.corners[1],
        expected_md_icon_r_tr,
        1.0,
    );
    assert_close(
        "button-group-size default mid border-left",
        quad_md_mid.border[3],
        expected_md_mid_border_l,
        0.2,
    );
    assert_close(
        "button-group-size default icon border-left",
        quad_md_icon.border[3],
        expected_md_icon_border_l,
        0.2,
    );

    let quad_lg_first = find_best_quad(&scene, lg_first_bounds).expect("painted quad for lg first");
    let quad_lg_mid = find_best_quad(&scene, lg_mid_bounds).expect("painted quad for lg mid");
    let quad_lg_icon = find_best_quad(&scene, lg_icon_bounds).expect("painted quad for lg icon");

    assert_close(
        "button-group-size large first radius tl",
        quad_lg_first.corners[0],
        expected_lg_first_r_tl,
        1.0,
    );
    assert_close(
        "button-group-size large first radius tr",
        quad_lg_first.corners[1],
        expected_lg_first_r_tr,
        1.0,
    );
    assert_close(
        "button-group-size large icon radius tr",
        quad_lg_icon.corners[1],
        expected_lg_icon_r_tr,
        1.0,
    );
    assert_close(
        "button-group-size large mid border-left",
        quad_lg_mid.border[3],
        expected_lg_mid_border_l,
        0.2,
    );
    assert_close(
        "button-group-size large icon border-left",
        quad_lg_icon.border[3],
        expected_lg_icon_border_l,
        0.2,
    );

    assert_close(
        "button-group-size small first radius tr (merged)",
        quad_sm_first.corners[1],
        expected_sm_first_r_tr,
        1.0,
    );
    assert_close(
        "button-group-size default first radius tr (merged)",
        quad_md_first.corners[1],
        expected_md_first_r_tr,
        1.0,
    );
    assert_close(
        "button-group-size large first radius tr (merged)",
        quad_lg_first.corners[1],
        expected_lg_first_r_tr,
        1.0,
    );
}

#[test]
fn web_vs_fret_button_group_dropdown_geometry_and_chrome_match() {
    let web = read_web_golden("button-group-dropdown");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_group = find_first(&theme.root, &|n| {
        n.tag == "div" && n.attrs.get("role").is_some_and(|v| v == "group")
    })
    .expect("web button-group node");

    let web_follow = find_first(web_group, &|n| {
        n.tag == "button" && contains_text(n, "Follow")
    })
    .expect("web Follow button");
    let web_trigger = find_first(web_group, &|n| {
        n.tag == "button" && n.attrs.get("aria-expanded").is_some()
    })
    .expect("web dropdown trigger button");

    let expected_group_h = web_group.rect.h;
    let expected_trigger_w = web_trigger.rect.w;
    let expected_trigger_border_l = web_border_width_px_for(web_trigger, "borderLeftWidth")
        .expect("web trigger borderLeftWidth");

    let expected_follow_r_tl =
        web_corner_radius_effective_px_for(web_follow, "borderTopLeftRadius")
            .expect("web Follow borderTopLeftRadius");
    let expected_follow_r_tr =
        web_corner_radius_effective_px_for(web_follow, "borderTopRightRadius")
            .expect("web Follow borderTopRightRadius");
    let expected_trigger_r_tr =
        web_corner_radius_effective_px_for(web_trigger, "borderTopRightRadius")
            .expect("web trigger borderTopRightRadius");

    let (snap, scene) = render_and_paint(|cx| {
        let follow = fret_ui_shadcn::Button::new("Follow")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .into();

        // Upstream `button-group-dropdown` uses asymmetric padding (`pl-2 pr-3`) for the trigger.
        // We express that via `ChromeRefinement` without changing global button sizing rules.
        let trigger = fret_ui_shadcn::Button::new("")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .refine_style(ChromeRefinement::default().pl(Space::N2))
            .children(vec![decl_icon::icon(cx, ids::ui::CHEVRON_DOWN)])
            .test_id("button-group-dropdown.trigger")
            .into();

        let group = fret_ui_shadcn::ButtonGroup::new(vec![follow, trigger])
            .a11y_label("ButtonGroupDropdown");
        vec![group.into_element(cx)]
    });

    let group = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Group && n.label.as_deref() == Some("ButtonGroupDropdown")
        })
        .expect("missing semantics for dropdown button group");

    assert_close(
        "button-group-dropdown group height",
        group.bounds.size.height.0,
        expected_group_h,
        1.0,
    );

    let trigger_bounds = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("button-group-dropdown.trigger"))
        .expect("missing semantics for dropdown trigger")
        .bounds;

    assert_close(
        "button-group-dropdown trigger width",
        trigger_bounds.size.width.0,
        expected_trigger_w,
        1.0,
    );

    let follow_node = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Follow"))
        .expect("missing semantics for Follow button");
    let quad_follow = find_best_quad(&scene, follow_node.bounds).expect("painted quad for Follow");
    assert_close(
        "button-group-dropdown Follow radius tl",
        quad_follow.corners[0],
        expected_follow_r_tl,
        1.0,
    );
    assert_close(
        "button-group-dropdown Follow radius tr",
        quad_follow.corners[1],
        expected_follow_r_tr,
        1.0,
    );

    let quad_trigger =
        find_best_quad(&scene, trigger_bounds).expect("painted quad for dropdown trigger");
    assert_close(
        "button-group-dropdown trigger border-left",
        quad_trigger.border[3],
        expected_trigger_border_l,
        0.2,
    );
    assert_close(
        "button-group-dropdown trigger radius tr",
        quad_trigger.corners[1],
        expected_trigger_r_tr,
        1.0,
    );
}

#[test]
fn web_vs_fret_button_group_popover_geometry_and_chrome_match() {
    let web = read_web_golden("button-group-popover");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_group = find_first(&theme.root, &|n| {
        n.tag == "div" && n.attrs.get("role").is_some_and(|v| v == "group")
    })
    .expect("web button-group node");

    let web_lead = find_first(web_group, &|n| {
        n.tag == "button" && n.attrs.get("aria-label").is_none()
    })
    .expect("web lead button");
    let web_trigger = find_first(web_group, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Open Popover")
    })
    .expect("web popover trigger button");

    let expected_lead_border_l =
        web_border_width_px_for(web_lead, "borderLeftWidth").expect("web lead borderLeftWidth");
    let expected_lead_r_tl = web_corner_radius_effective_px_for(web_lead, "borderTopLeftRadius")
        .expect("web lead borderTopLeftRadius");
    let expected_lead_r_tr = web_corner_radius_effective_px_for(web_lead, "borderTopRightRadius")
        .expect("web lead borderTopRightRadius");

    let expected_trigger_border_l = web_border_width_px_for(web_trigger, "borderLeftWidth")
        .expect("web trigger borderLeftWidth");
    let expected_trigger_r_tl =
        web_corner_radius_effective_px_for(web_trigger, "borderTopLeftRadius")
            .expect("web trigger borderTopLeftRadius");
    let expected_trigger_r_tr =
        web_corner_radius_effective_px_for(web_trigger, "borderTopRightRadius")
            .expect("web trigger borderTopRightRadius");

    let (snap, scene) = render_and_paint(|cx| {
        let lead = fret_ui_shadcn::Button::new("")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .children(vec![decl_icon::icon(cx, ids::ui::CHEVRON_RIGHT)])
            .test_id("button-group-popover.lead")
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(Px(web_lead.rect.w))
                    .h_px(Px(web_lead.rect.h)),
            )
            .into();

        let trigger = fret_ui_shadcn::Button::new("Open Popover")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .size(fret_ui_shadcn::ButtonSize::Icon)
            .children(vec![decl_icon::icon(cx, ids::ui::CHEVRON_DOWN)])
            .test_id("button-group-popover.trigger")
            .into();

        let group =
            fret_ui_shadcn::ButtonGroup::new(vec![lead, trigger]).a11y_label("ButtonGroupPopover");
        vec![group.into_element(cx)]
    });

    let group = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Group && n.label.as_deref() == Some("ButtonGroupPopover")
        })
        .expect("missing semantics for popover button group");

    assert_close(
        "button-group-popover group height",
        group.bounds.size.height.0,
        web_group.rect.h,
        1.0,
    );

    let lead_bounds = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("button-group-popover.lead"))
        .expect("missing lead semantics node")
        .bounds;
    let trigger_bounds = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("button-group-popover.trigger"))
        .expect("missing trigger semantics node")
        .bounds;

    let actual_gap =
        trigger_bounds.origin.x.0 - (lead_bounds.origin.x.0 + lead_bounds.size.width.0);
    assert_close(
        "button-group-popover lead → trigger gap",
        actual_gap,
        0.0,
        0.5,
    );

    let quad_lead = find_best_quad(&scene, lead_bounds).expect("painted quad for lead");
    assert_close(
        "button-group-popover lead border-left",
        quad_lead.border[3],
        expected_lead_border_l,
        0.2,
    );
    assert_close(
        "button-group-popover lead radius tl",
        quad_lead.corners[0],
        expected_lead_r_tl,
        1.0,
    );
    assert_close(
        "button-group-popover lead radius tr",
        quad_lead.corners[1],
        expected_lead_r_tr,
        1.0,
    );

    let quad_trigger = find_best_quad(&scene, trigger_bounds).expect("painted quad for trigger");
    assert_close(
        "button-group-popover trigger border-left",
        quad_trigger.border[3],
        expected_trigger_border_l,
        0.2,
    );
    assert_close(
        "button-group-popover trigger radius tl",
        quad_trigger.corners[0],
        expected_trigger_r_tl,
        1.0,
    );
    assert_close(
        "button-group-popover trigger radius tr",
        quad_trigger.corners[1],
        expected_trigger_r_tr,
        1.0,
    );
}

#[test]
fn web_vs_fret_button_group_input_geometry_and_chrome_match() {
    let web = read_web_golden("button-group-input");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_group = find_first(&theme.root, &|n| {
        n.tag == "div" && n.attrs.get("role").is_some_and(|v| v == "group")
    })
    .expect("web button-group node");

    let web_input = find_first(web_group, &|n| n.tag == "input").expect("web input node");
    let web_button = find_first(web_group, &|n| {
        n.tag == "button" && n.attrs.get("aria-label").is_some_and(|v| v == "Search")
    })
    .expect("web search button");

    let expected_input_border_l =
        web_border_width_px_for(web_input, "borderLeftWidth").expect("web input borderLeftWidth");
    let expected_input_r_tr = web_corner_radius_effective_px_for(web_input, "borderTopRightRadius")
        .expect("web input borderTopRightRadius");

    let expected_button_border_l =
        web_border_width_px_for(web_button, "borderLeftWidth").expect("web button borderLeftWidth");
    let expected_button_r_tl =
        web_corner_radius_effective_px_for(web_button, "borderTopLeftRadius")
            .expect("web button borderTopLeftRadius");
    let expected_button_r_tr =
        web_corner_radius_effective_px_for(web_button, "borderTopRightRadius")
            .expect("web button borderTopRightRadius");

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());

        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("ButtonGroupInputInput")
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(web_input.rect.w)))
            .corner_radii_override(Corners {
                top_left: Px(8.0),
                bottom_left: Px(8.0),
                top_right: Px(0.0),
                bottom_right: Px(0.0),
            })
            .into_element(cx);

        let button = fret_ui_shadcn::Button::new("Search")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .border_left_width_override(Px(0.0))
            .corner_radii_override(Corners {
                top_left: Px(0.0),
                bottom_left: Px(0.0),
                top_right: Px(8.0),
                bottom_right: Px(8.0),
            })
            .children(vec![decl_icon::icon(cx, ids::ui::SEARCH)])
            .test_id("button-group-input.search")
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(Px(web_button.rect.w))
                    .h_px(Px(web_button.rect.h)),
            )
            .into_element(cx);

        let group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                label: Some(Arc::from("ButtonGroupInput")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.flex(
                    fret_ui::element::FlexProps {
                        layout: fret_ui::element::LayoutStyle::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(0.0),
                        padding: fret_core::Edges::all(Px(0.0)),
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Stretch,
                        wrap: false,
                    },
                    move |_cx| vec![input, button],
                )]
            },
        );

        vec![group]
    });

    let input = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::TextField
                && n.label.as_deref() == Some("ButtonGroupInputInput")
        })
        .expect("missing semantics for input");
    let button = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("button-group-input.search"))
        .expect("missing semantics for button");

    let actual_gap =
        button.bounds.origin.x.0 - (input.bounds.origin.x.0 + input.bounds.size.width.0);
    assert_close("button-group-input gap", actual_gap, 0.0, 0.5);

    let quad_input = find_best_quad(&scene, input.bounds).expect("painted quad for input");
    assert_close(
        "button-group-input input border-left",
        quad_input.border[3],
        expected_input_border_l,
        0.2,
    );
    assert_close(
        "button-group-input input radius tr",
        quad_input.corners[1],
        expected_input_r_tr,
        1.0,
    );

    let quad_button = find_best_quad(&scene, button.bounds).expect("painted quad for button");
    assert_close(
        "button-group-input button border-left",
        quad_button.border[3],
        expected_button_border_l,
        0.2,
    );
    assert_close(
        "button-group-input button radius tl",
        quad_button.corners[0],
        expected_button_r_tl,
        1.0,
    );
    assert_close(
        "button-group-input button radius tr",
        quad_button.corners[1],
        expected_button_r_tr,
        1.0,
    );
}

#[test]
fn web_vs_fret_input_demo_focus_ring_matches() {
    let web = read_web_golden("input-demo.focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input node");
    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_input).expect("web input focus ring");
    let expected_border_color = web_input
        .computed_style
        .get("borderTopColor")
        .map(String::as_str)
        .expect("web input borderTopColor");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());
            vec![
                fret_ui_shadcn::Input::new(model)
                    .a11y_label("InputDemoInput")
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(web_input.rect.w))
                            .h_px(Px(web_input.rect.h)),
                    )
                    .into_element(cx),
            ]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::TextField
                        && n.label.as_deref() == Some("InputDemoInput")
                })
                .map(|n| n.id)
                .expect("missing fret input semantics node")
        },
    );

    let input = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::TextField && n.label.as_deref() == Some("InputDemoInput")
        })
        .expect("missing semantics for input");

    let ring_quad =
        find_focus_ring_quad(&scene, input.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "input-demo focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );

    let quad_input = find_best_quad(&scene, input.bounds).expect("painted quad for input");
    assert_color_close(
        "input-demo focus border color",
        quad_input.border_color,
        expected_border_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_input_demo_aria_invalid_focus_ring_matches() {
    let web = read_web_golden("input-demo.invalid-focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input node");
    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_input).expect("web input invalid focus ring");
    let expected_border_color = web_input
        .computed_style
        .get("borderTopColor")
        .map(String::as_str)
        .expect("web input borderTopColor");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());
            vec![
                fret_ui_shadcn::Input::new(model)
                    .a11y_label("InputDemoInputInvalid")
                    .aria_invalid(true)
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(web_input.rect.w))
                            .h_px(Px(web_input.rect.h)),
                    )
                    .into_element(cx),
            ]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::TextField
                        && n.label.as_deref() == Some("InputDemoInputInvalid")
                })
                .map(|n| n.id)
                .expect("missing fret input semantics node")
        },
    );

    let input = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::TextField
                && n.label.as_deref() == Some("InputDemoInputInvalid")
        })
        .expect("missing semantics for input");

    let ring_quad =
        find_focus_ring_quad(&scene, input.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "input-demo invalid focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );

    let quad_input = find_best_quad(&scene, input.bounds).expect("painted quad for input");
    assert_color_close(
        "input-demo invalid focus border color",
        quad_input.border_color,
        expected_border_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_input_group_demo_focus_ring_matches() {
    let web = read_web_golden("input-group-demo.focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_group = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.attrs.get("data-slot").is_some_and(|v| v == "input-group")
            && n.computed_style
                .get("boxShadow")
                .is_some_and(|v| v.contains(" 3px"))
    })
    .expect("web focused input-group node");
    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_group).expect("web input-group focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());

            let leading =
                vec![cx.opacity(0.5, move |cx| vec![decl_icon::icon(cx, ids::ui::SEARCH)])];
            let trailing = vec![cx.text_props(fret_ui::element::TextProps {
                layout: fret_ui::element::LayoutStyle::default(),
                text: Arc::from("12 results"),
                style: None,
                color: None,
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
            })];

            let group = fret_ui_shadcn::InputGroup::new(model)
                .a11y_label("InputGroupControl")
                .leading(leading)
                .trailing(trailing)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(web_group.rect.w))
                        .h_px(Px(web_group.rect.h)),
                )
                .into_element(cx);

            vec![cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Group,
                    label: Some(Arc::from("InputGroupRoot")),
                    ..Default::default()
                },
                move |_cx| vec![group],
            )]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::TextField
                        && n.label.as_deref() == Some("InputGroupControl")
                })
                .map(|n| n.id)
                .expect("missing fret input-group control semantics node")
        },
    );

    let root = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Group && n.label.as_deref() == Some("InputGroupRoot"))
        .expect("missing semantics for group root");

    let ring_quad =
        find_focus_ring_quad(&scene, root.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "input-group-demo focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_input_group_demo_aria_invalid_focus_ring_matches() {
    let web = read_web_golden("input-group-demo.invalid-focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_group = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.attrs.get("data-slot").is_some_and(|v| v == "input-group")
            && n.computed_style
                .get("boxShadow")
                .is_some_and(|v| v.contains(" 3px"))
    })
    .expect("web focused invalid input-group node");
    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_group).expect("web input-group invalid focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());

            let leading =
                vec![cx.opacity(0.5, move |cx| vec![decl_icon::icon(cx, ids::ui::SEARCH)])];
            let trailing = vec![cx.text_props(fret_ui::element::TextProps {
                layout: fret_ui::element::LayoutStyle::default(),
                text: Arc::from("12 results"),
                style: None,
                color: None,
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
            })];

            let group = fret_ui_shadcn::InputGroup::new(model)
                .a11y_label("InputGroupControlInvalid")
                .aria_invalid(true)
                .leading(leading)
                .trailing(trailing)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(web_group.rect.w))
                        .h_px(Px(web_group.rect.h)),
                )
                .into_element(cx);

            vec![cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Group,
                    label: Some(Arc::from("InputGroupRootInvalid")),
                    ..Default::default()
                },
                move |_cx| vec![group],
            )]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::TextField
                        && n.label.as_deref() == Some("InputGroupControlInvalid")
                })
                .map(|n| n.id)
                .expect("missing fret input-group control semantics node")
        },
    );

    let root = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Group && n.label.as_deref() == Some("InputGroupRootInvalid")
        })
        .expect("missing semantics for group root");

    let ring_quad =
        find_focus_ring_quad(&scene, root.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "input-group-demo invalid focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_button_group_select_geometry_and_chrome_match() {
    let web = read_web_golden("button-group-select");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_group = find_first(&theme.root, &|n| {
        n.tag == "div" && n.attrs.get("role").is_some_and(|v| v == "group")
    })
    .expect("web button-group node");

    let web_combobox = find_first(web_group, &|n| {
        n.tag == "button" && n.attrs.get("role").is_some_and(|v| v == "combobox")
    })
    .expect("web combobox button");
    let web_input = find_first(web_group, &|n| n.tag == "input").expect("web input node");
    let web_send = find_first(web_group, &|n| {
        n.tag == "button" && n.attrs.get("aria-label").is_some_and(|v| v == "Send")
    })
    .expect("web Send button");

    let expected_combobox_r_tr =
        web_corner_radius_effective_px_for(web_combobox, "borderTopRightRadius")
            .expect("web combobox borderTopRightRadius");
    let expected_input_border_l =
        web_border_width_px_for(web_input, "borderLeftWidth").expect("web input borderLeftWidth");
    let expected_input_r_tl = web_corner_radius_effective_px_for(web_input, "borderTopLeftRadius")
        .expect("web input borderTopLeftRadius");
    let expected_input_r_tr = web_corner_radius_effective_px_for(web_input, "borderTopRightRadius")
        .expect("web input borderTopRightRadius");

    let expected_gap = web_send.rect.x - (web_input.rect.x + web_input.rect.w);

    let (snap, scene) = render_and_paint(|cx| {
        let select_value: fret_runtime::Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
        let select_open: fret_runtime::Model<bool> = cx.app.models_mut().insert(false);
        let input_model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());

        let radius = Px(8.0);

        let select = fret_ui_shadcn::Select::new(select_value, select_open)
            .placeholder(Arc::from(""))
            .a11y_label("ButtonGroupSelectCombobox")
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(Px(web_combobox.rect.w))
                    .h_px(Px(web_combobox.rect.h)),
            )
            .corner_radii_override(Corners {
                top_left: radius,
                bottom_left: radius,
                top_right: Px(0.0),
                bottom_right: Px(0.0),
            })
            .into_element(cx);

        let input = fret_ui_shadcn::Input::new(input_model)
            .a11y_label("ButtonGroupSelectInput")
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(Px(web_input.rect.w))
                    .h_px(Px(web_input.rect.h)),
            )
            .border_left_width_override(Px(0.0))
            .corner_radii_override(Corners {
                top_left: Px(0.0),
                bottom_left: Px(0.0),
                top_right: radius,
                bottom_right: radius,
            })
            .into_element(cx);

        let left = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                label: Some(Arc::from("ButtonGroupSelectLeft")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.flex(
                    fret_ui::element::FlexProps {
                        layout: fret_ui::element::LayoutStyle::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(0.0),
                        padding: fret_core::Edges::all(Px(0.0)),
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Stretch,
                        wrap: false,
                    },
                    move |_cx| vec![select, input],
                )]
            },
        );

        let send = fret_ui_shadcn::Button::new("Send")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .size(fret_ui_shadcn::ButtonSize::Icon)
            .children(vec![decl_icon::icon(cx, ids::ui::CHEVRON_RIGHT)])
            .test_id("button-group-select.send")
            .into_element(cx);

        let group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                label: Some(Arc::from("ButtonGroupSelect")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.flex(
                    fret_ui::element::FlexProps {
                        layout: fret_ui::element::LayoutStyle::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(8.0),
                        padding: fret_core::Edges::all(Px(0.0)),
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Stretch,
                        wrap: false,
                    },
                    move |_cx| vec![left, send],
                )]
            },
        );

        vec![group]
    });

    let left_group = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Group && n.label.as_deref() == Some("ButtonGroupSelectLeft")
        })
        .expect("missing semantics for left group");
    let send = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("button-group-select.send"))
        .expect("missing semantics for Send button");

    let actual_gap =
        send.bounds.origin.x.0 - (left_group.bounds.origin.x.0 + left_group.bounds.size.width.0);
    assert_close(
        "button-group-select gap (left → send)",
        actual_gap,
        expected_gap,
        1.0,
    );

    let combobox = snap
        .nodes
        .iter()
        .find(|n| n.label.as_deref() == Some("ButtonGroupSelectCombobox"))
        .expect("missing semantics for combobox");
    let input = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::TextField
                && n.label.as_deref() == Some("ButtonGroupSelectInput")
        })
        .expect("missing semantics for input");

    assert_close(
        "button-group-select combobox width",
        combobox.bounds.size.width.0,
        web_combobox.rect.w,
        1.0,
    );
    assert_close(
        "button-group-select input width",
        input.bounds.size.width.0,
        web_input.rect.w,
        1.0,
    );

    let quad_combobox = find_best_quad(&scene, combobox.bounds).expect("painted quad for combobox");
    assert_close(
        "button-group-select combobox radius tr",
        quad_combobox.corners[1],
        expected_combobox_r_tr,
        1.0,
    );

    let quad_input = find_best_quad(&scene, input.bounds).expect("painted quad for input");
    assert_close(
        "button-group-select input border-left",
        quad_input.border[3],
        expected_input_border_l,
        0.2,
    );
    assert_close(
        "button-group-select input radius tl",
        quad_input.corners[0],
        expected_input_r_tl,
        1.0,
    );
    assert_close(
        "button-group-select input radius tr",
        quad_input.corners[1],
        expected_input_r_tr,
        1.0,
    );
}

#[test]
fn web_vs_fret_button_group_input_group_geometry_matches() {
    let web = read_web_golden("button-group-input-group");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_outer = find_first(&theme.root, &|n| {
        n.tag == "div" && n.attrs.get("role").is_some_and(|v| v == "group")
    })
    .expect("web outer group");

    let web_left_button = find_first(web_outer, &|n| {
        n.tag == "button" && (n.rect.w - 36.0).abs() <= 0.1
    })
    .expect("web left icon button");
    let web_wrapper = find_first(web_outer, &|n| {
        n.tag == "div"
            && n.attrs.get("role").is_some_and(|v| v == "group")
            && (n.rect.x - 44.0).abs() <= 0.2
            && n.computed_style
                .get("borderTopWidth")
                .is_some_and(|v| v == "1px")
    })
    .expect("web wrapper group");
    let web_input = find_first(web_wrapper, &|n| n.tag == "input").expect("web input node");

    let expected_gap = web_wrapper.rect.x - (web_left_button.rect.x + web_left_button.rect.w);
    let expected_wrapper_border_top =
        web_border_width_px_for(web_wrapper, "borderTopWidth").expect("web wrapper borderTopWidth");
    let expected_wrapper_r_tl =
        web_corner_radius_effective_px_for(web_wrapper, "borderTopLeftRadius")
            .expect("web wrapper borderTopLeftRadius");

    let (snap, scene) = render_and_paint(|cx| {
        let theme = fret_ui::Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");
        let bg = theme.color_required("background");

        let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());

        let left = fret_ui_shadcn::Button::new("")
            .variant(fret_ui_shadcn::ButtonVariant::Outline)
            .size(fret_ui_shadcn::ButtonSize::Icon)
            .children(vec![decl_icon::icon(cx, ids::ui::CHEVRON_RIGHT)])
            .corner_radii_override(Corners::all(Px(9999.0)))
            .test_id("button-group-input-group.left")
            .into_element(cx);

        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("ButtonGroupInputGroupInput")
            .refine_style(
                ChromeRefinement::default()
                    .border_width(Px(0.0))
                    .radius(Px(0.0))
                    .pr(Space::N2),
            )
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(Px(web_input.rect.w))
                    .h_px(Px(web_input.rect.h)),
            )
            .into_element(cx);

        let tiny = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Button,
                test_id: Some(Arc::from("button-group-input-group.tiny")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = fret_ui::element::Length::Px(Px(24.0));
                            layout.size.height = fret_ui::element::Length::Px(Px(24.0));
                            layout
                        },
                        background: Some(bg),
                        border: fret_core::Edges::all(Px(0.0)),
                        border_color: None,
                        corner_radii: Corners::all(Px(9999.0)),
                        ..Default::default()
                    },
                    |cx| vec![decl_icon::icon(cx, ids::ui::MORE_HORIZONTAL)],
                )]
            },
        );

        let right = cx.container(
            fret_ui::element::ContainerProps {
                layout: {
                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.size.width = fret_ui::element::Length::Px(Px(36.0));
                    layout.size.height = fret_ui::element::Length::Px(Px(36.0));
                    layout.margin.right = fret_ui::element::MarginEdge::Px(Px(-7.2));
                    layout
                },
                padding: fret_core::Edges {
                    top: Px(0.0),
                    right: Px(12.0),
                    bottom: Px(0.0),
                    left: Px(0.0),
                },
                ..Default::default()
            },
            move |cx| {
                vec![cx.flex(
                    fret_ui::element::FlexProps {
                        layout: fret_ui::element::LayoutStyle::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(0.0),
                        padding: fret_core::Edges::all(Px(0.0)),
                        justify: fret_ui::element::MainAlign::Center,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| vec![tiny],
                )]
            },
        );

        let wrapper = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                label: Some(Arc::from("ButtonGroupInputGroupPill")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width =
                                fret_ui::element::Length::Px(Px(web_wrapper.rect.w));
                            layout.size.height =
                                fret_ui::element::Length::Px(Px(web_wrapper.rect.h));
                            layout
                        },
                        padding: fret_core::Edges::all(Px(0.0)),
                        background: Some(bg),
                        border: fret_core::Edges::all(Px(1.0)),
                        border_color: Some(border),
                        corner_radii: Corners::all(Px(9999.0)),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.flex(
                            fret_ui::element::FlexProps {
                                layout: fret_ui::element::LayoutStyle::default(),
                                direction: fret_core::Axis::Horizontal,
                                gap: Px(0.0),
                                padding: fret_core::Edges::all(Px(0.0)),
                                justify: fret_ui::element::MainAlign::Start,
                                align: fret_ui::element::CrossAlign::Stretch,
                                wrap: false,
                            },
                            move |_cx| vec![input, right],
                        )]
                    },
                )]
            },
        );

        let group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                label: Some(Arc::from("ButtonGroupInputGroup")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.flex(
                    fret_ui::element::FlexProps {
                        layout: fret_ui::element::LayoutStyle::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(8.0),
                        padding: fret_core::Edges::all(Px(0.0)),
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Stretch,
                        wrap: false,
                    },
                    move |_cx| vec![left, wrapper],
                )]
            },
        );

        vec![group]
    });

    let group = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Group && n.label.as_deref() == Some("ButtonGroupInputGroup")
        })
        .expect("missing semantics for outer group");
    assert_close(
        "button-group-input-group group width",
        group.bounds.size.width.0,
        web_outer.rect.w,
        1.0,
    );
    assert_close(
        "button-group-input-group group height",
        group.bounds.size.height.0,
        web_outer.rect.h,
        1.0,
    );

    let left = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("button-group-input-group.left"))
        .expect("missing left button semantics");
    let wrapper = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Group
                && n.label.as_deref() == Some("ButtonGroupInputGroupPill")
        })
        .expect("missing wrapper semantics");

    let actual_gap =
        wrapper.bounds.origin.x.0 - (left.bounds.origin.x.0 + left.bounds.size.width.0);
    assert_close(
        "button-group-input-group gap",
        actual_gap,
        expected_gap,
        1.0,
    );

    let quad_wrapper = find_best_quad(&scene, wrapper.bounds).expect("painted quad for wrapper");
    assert_close(
        "button-group-input-group wrapper border-top",
        quad_wrapper.border[0],
        expected_wrapper_border_top,
        0.2,
    );
    assert_close(
        "button-group-input-group wrapper radius tl",
        quad_wrapper.corners[0],
        expected_wrapper_r_tl,
        1.0,
    );

    let tiny = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("button-group-input-group.tiny"))
        .expect("missing tiny control semantics");
    assert_close(
        "button-group-input-group tiny width",
        tiny.bounds.size.width.0,
        24.0,
        0.5,
    );
    assert_close(
        "button-group-input-group tiny height",
        tiny.bounds.size.height.0,
        24.0,
        0.5,
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
fn web_vs_fret_input_demo_aria_invalid_border_color_matches() {
    let web = read_web_golden("input-demo.invalid");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_input = find_first(&theme.root, &|n| {
        n.tag == "input" && n.attrs.get("aria-invalid").is_some_and(|v| v == "true")
    })
    .or_else(|| {
        find_first(&theme.root, &|n| {
            n.tag == "input" && (n.rect.h - 36.0).abs() <= 0.1
        })
    })
    .expect("web input node");

    let web_border_color = web_input
        .computed_style
        .get("borderTopColor")
        .map(String::as_str)
        .expect("web borderTopColor");

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::Input::new(model)
                .a11y_label("Input")
                .aria_invalid(true)
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
    assert_color_close(
        "input aria-invalid border color",
        quad.border_color,
        web_border_color,
        0.03,
    );
}

#[test]
fn web_vs_fret_input_group_demo_aria_invalid_border_color_matches() {
    let web = read_web_golden("input-group-demo.invalid");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_group = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.attrs.get("data-slot").is_some_and(|v| v == "input-group")
            && has_descendant_attr(n, "aria-invalid", "true")
    })
    .expect("web input-group node");

    let web_border_color = web_group
        .computed_style
        .get("borderTopColor")
        .map(String::as_str)
        .expect("web borderTopColor");

    let web_w = web_group.rect.w;
    let web_h = web_group.rect.h;

    let (_snap, scene) = render_and_paint_in_bounds(CoreSize::new(Px(web_w), Px(web_h)), |cx| {
        let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::InputGroup::new(model)
                .aria_invalid(true)
                .into_element(cx),
        ]
    });

    let target = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(web_w), Px(web_h)),
    );
    let quad = find_best_quad(&scene, target).expect("painted quad for input-group");
    assert_color_close(
        "input-group aria-invalid border color",
        quad.border_color,
        web_border_color,
        0.03,
    );
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
                        .w_px(Px(web_w))
                        .h_px(Px(web_h)),
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
                        .w_px(Px(web_w))
                        .h_px(Px(web_h)),
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
fn web_vs_fret_card_demo_chrome_matches() {
    let web = read_web_golden("card-demo");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_card = find_first(&theme.root, &|n| {
        if n.tag != "div" {
            return false;
        }
        if !contains_text(n, "Login to your account") {
            return false;
        }
        let border = web_border_width_px(n);
        let radius = web_corner_radius_effective_px(n);
        border.is_some_and(|v| (v - 1.0).abs() <= 0.1) && radius.is_some_and(|v| v >= 8.0)
    })
    .expect("web card container");

    let web_border = web_border_width_px(web_card).expect("web borderTopWidth px");
    let web_radius = web_corner_radius_effective_px(web_card).expect("web radius px");
    let web_w = web_card.rect.w;
    let web_h = web_card.rect.h;

    let (_snap, scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Card::new(Vec::new())
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(web_w))
                        .h_px(Px(web_h)),
                )
                .into_element(cx),
        ]
    });

    let target = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(web_w), Px(web_h)),
    );
    let quad = find_best_quad(&scene, target).expect("painted quad for card");

    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(&format!("card border[{idx}]"), *edge, web_border, 0.6);
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(&format!("card radius[{idx}]"), *corner, web_radius, 1.0);
    }
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
                        .w_px(Px(web_w))
                        .h_px(Px(web_h)),
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
fn web_vs_fret_button_demo_focus_ring_matches() {
    let web = read_web_golden("button-demo.focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_button = find_first(&theme.root, &|n| {
        n.tag == "button" && n.text.as_deref() == Some("Button")
    })
    .expect("web button node");

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_button).expect("web button focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            vec![
                fret_ui_shadcn::Button::new("Button")
                    .variant(fret_ui_shadcn::ButtonVariant::Outline)
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(web_button.rect.w))
                            .h_px(Px(web_button.rect.h)),
                    )
                    .into_element(cx),
            ]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Button"))
                .map(|n| n.id)
                .expect("missing fret button semantics node")
        },
    );

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Button"))
        .expect("missing semantics for button");

    let ring_quad =
        find_focus_ring_quad(&scene, button.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "button-demo focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
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
                        .w_px(Px(web_w))
                        .h_px(Px(web_h)),
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
                        .w_px(Px(web_w))
                        .h_px(Px(web_h)),
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
fn web_vs_fret_button_size_demo_heights_match() {
    let web = read_web_golden("button-size");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let mut web_buttons: Vec<&WebNode> = Vec::new();
    fn collect_buttons<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        if node.tag == "button" {
            out.push(node);
        }
        for child in &node.children {
            collect_buttons(child, out);
        }
    }
    collect_buttons(&theme.root, &mut web_buttons);
    assert_eq!(
        web_buttons.len(),
        6,
        "expected 6 buttons in button-size golden"
    );

    let expected_h: Vec<f32> = web_buttons.iter().map(|b| b.rect.h).collect();
    let expected_w: Vec<f32> = web_buttons.iter().map(|b| b.rect.w).collect();

    let (snap, _scene) = render_and_paint(|cx| {
        vec![
            fret_ui_shadcn::Button::new("Small")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .test_id("button-size.small")
                .into_element(cx),
            fret_ui_shadcn::Button::new("")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::IconSm)
                .test_id("button-size.icon-sm")
                .into_element(cx),
            fret_ui_shadcn::Button::new("Default")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Default)
                .test_id("button-size.default")
                .into_element(cx),
            fret_ui_shadcn::Button::new("")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Icon)
                .test_id("button-size.icon")
                .into_element(cx),
            fret_ui_shadcn::Button::new("Large")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Lg)
                .test_id("button-size.large")
                .into_element(cx),
            fret_ui_shadcn::Button::new("")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::IconLg)
                .test_id("button-size.icon-lg")
                .into_element(cx),
        ]
    });

    let actual_h = |test_id: &str| {
        snap.nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some(test_id))
            .unwrap_or_else(|| panic!("missing semantics node {test_id}"))
            .bounds
            .size
            .height
            .0
    };

    assert_close(
        "button-size small height",
        actual_h("button-size.small"),
        expected_h[0],
        1.0,
    );
    assert_close(
        "button-size icon-sm height",
        actual_h("button-size.icon-sm"),
        expected_h[1],
        1.0,
    );
    assert_close(
        "button-size default height",
        actual_h("button-size.default"),
        expected_h[2],
        1.0,
    );
    assert_close(
        "button-size icon height",
        actual_h("button-size.icon"),
        expected_h[3],
        1.0,
    );
    assert_close(
        "button-size large height",
        actual_h("button-size.large"),
        expected_h[4],
        1.0,
    );
    assert_close(
        "button-size icon-lg height",
        actual_h("button-size.icon-lg"),
        expected_h[5],
        1.0,
    );

    let actual_w = |test_id: &str| {
        snap.nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some(test_id))
            .unwrap_or_else(|| panic!("missing semantics node {test_id}"))
            .bounds
            .size
            .width
            .0
    };

    assert_close(
        "button-size icon-sm width",
        actual_w("button-size.icon-sm"),
        expected_w[1],
        1.0,
    );
    assert_close(
        "button-size icon width",
        actual_w("button-size.icon"),
        expected_w[3],
        1.0,
    );
    assert_close(
        "button-size icon-lg width",
        actual_w("button-size.icon-lg"),
        expected_w[5],
        1.0,
    );
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
fn web_vs_fret_textarea_demo_aria_invalid_border_color_matches() {
    let web = read_web_golden("textarea-demo.invalid");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_textarea = find_first(&theme.root, &|n| {
        n.tag == "textarea" && n.attrs.get("aria-invalid").is_some_and(|v| v == "true")
    })
    .expect("web textarea node");

    let web_border_color = web_textarea
        .computed_style
        .get("borderTopColor")
        .map(String::as_str)
        .expect("web borderTopColor");

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());
        vec![
            fret_ui_shadcn::Textarea::new(model)
                .a11y_label("TextareaInvalid")
                .aria_invalid(true)
                .into_element(cx),
        ]
    });

    let textarea = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::TextField && n.label.as_deref() == Some("TextareaInvalid")
        })
        .or_else(|| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::TextField)
        })
        .expect("fret textarea semantics node");

    let quad = find_best_quad(&scene, textarea.bounds).expect("painted quad for textarea");
    assert_color_close(
        "textarea aria-invalid border color",
        quad.border_color,
        web_border_color,
        0.03,
    );
}

#[test]
fn web_vs_fret_textarea_demo_focus_ring_matches() {
    let web = read_web_golden("textarea-demo.focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_textarea =
        find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea node");
    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_textarea).expect("web textarea focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());
            vec![
                fret_ui_shadcn::Textarea::new(model)
                    .a11y_label("TextareaFocus")
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(web_textarea.rect.w))
                            .h_px(Px(web_textarea.rect.h)),
                    )
                    .into_element(cx),
            ]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::TextField
                        && n.label.as_deref() == Some("TextareaFocus")
                })
                .map(|n| n.id)
                .expect("missing fret textarea semantics node")
        },
    );

    let textarea = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::TextField && n.label.as_deref() == Some("TextareaFocus"))
        .expect("missing semantics for textarea");

    let ring_quad = find_focus_ring_quad(&scene, textarea.bounds, expected_ring_spread)
        .expect("focus ring quad");
    assert_color_close(
        "textarea-demo focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_textarea_demo_aria_invalid_focus_ring_matches() {
    let web = read_web_golden("textarea-demo.invalid-focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_textarea =
        find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea node");
    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_textarea).expect("web textarea invalid focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let model: fret_runtime::Model<String> = cx.app.models_mut().insert(String::new());
            vec![
                fret_ui_shadcn::Textarea::new(model)
                    .a11y_label("TextareaInvalidFocus")
                    .aria_invalid(true)
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(web_textarea.rect.w))
                            .h_px(Px(web_textarea.rect.h)),
                    )
                    .into_element(cx),
            ]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::TextField
                        && n.label.as_deref() == Some("TextareaInvalidFocus")
                })
                .map(|n| n.id)
                .expect("missing fret textarea semantics node")
        },
    );

    let textarea = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::TextField && n.label.as_deref() == Some("TextareaInvalidFocus")
        })
        .expect("missing semantics for textarea");

    let ring_quad = find_focus_ring_quad(&scene, textarea.bounds, expected_ring_spread)
        .expect("focus ring quad");
    assert_color_close(
        "textarea-demo invalid focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
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
fn web_vs_fret_select_demo_aria_invalid_border_color_matches() {
    let web = read_web_golden("select-demo.invalid");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_trigger = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|v| v == "combobox")
            && n.attrs.get("aria-invalid").is_some_and(|v| v == "true")
    })
    .expect("web select trigger node");

    let web_border_color = web_trigger
        .computed_style
        .get("borderTopColor")
        .map(String::as_str)
        .expect("web borderTopColor");

    let (snap, scene) = render_and_paint(|cx| {
        let model: fret_runtime::Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
        let open: fret_runtime::Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Select::new(model, open)
                .a11y_label("SelectInvalid")
                .aria_invalid(true)
                .item(fret_ui_shadcn::SelectItem::new("one", "One"))
                .item(fret_ui_shadcn::SelectItem::new("two", "Two"))
                .into_element(cx),
        ]
    });

    let select = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox && n.label.as_deref() == Some("SelectInvalid"))
        .or_else(|| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::ComboBox)
        })
        .expect("fret select semantics node");

    let quad = find_best_quad(&scene, select.bounds).expect("painted quad for select trigger");
    assert_color_close(
        "select aria-invalid border color",
        quad.border_color,
        web_border_color,
        0.03,
    );
}

#[test]
fn web_vs_fret_select_demo_focus_ring_matches() {
    let web = read_web_golden("select-demo.focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_trigger = find_first(&theme.root, &|n| {
        n.tag == "button" && n.attrs.get("role").is_some_and(|v| v == "combobox")
    })
    .expect("web select trigger node");

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_trigger).expect("web select focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let model: fret_runtime::Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            let open: fret_runtime::Model<bool> = cx.app.models_mut().insert(false);
            vec![
                fret_ui_shadcn::Select::new(model, open)
                    .a11y_label("SelectFocus")
                    .item(fret_ui_shadcn::SelectItem::new("one", "One"))
                    .item(fret_ui_shadcn::SelectItem::new("two", "Two"))
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(web_trigger.rect.w))
                            .h_px(Px(web_trigger.rect.h)),
                    )
                    .into_element(cx),
            ]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::ComboBox && n.label.as_deref() == Some("SelectFocus")
                })
                .map(|n| n.id)
                .expect("missing fret select semantics node")
        },
    );

    let select = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox && n.label.as_deref() == Some("SelectFocus"))
        .expect("missing semantics for select");

    let ring_quad =
        find_focus_ring_quad(&scene, select.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "select-demo focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_calendar_14_focus_ring_matches_web() {
    let web = read_web_golden("calendar-14.focus-kbd-selected");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_focused_button = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.contains_key("aria-label")
            && web_box_shadow_focus_ring(n).is_some()
    })
    .expect("web focused calendar day button");

    let web_label = web_focused_button
        .attrs
        .get("aria-label")
        .expect("web focused day aria-label");
    let (selected_date, _selected) =
        parse_calendar_day_aria_label(web_label).expect("parse web focused day aria-label");

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_focused_button).expect("web focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            use fret_ui_headless::calendar::CalendarMonth;

            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let border = theme.color_required("border");

            let month_model: fret_runtime::Model<CalendarMonth> = cx.app.models_mut().insert(
                CalendarMonth::new(selected_date.year(), selected_date.month()),
            );
            let selected: fret_runtime::Model<Option<Date>> =
                cx.app.models_mut().insert(Some(selected_date));

            let calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                .cell_size(Px(web_focused_button.rect.w))
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(fret_ui_kit::Radius::Lg)
                        .border_1()
                        .border_color(fret_ui_kit::ColorRef::Color(border))
                        .shadow_sm(),
                )
                .into_element(cx);

            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        layout
                    },
                    padding: fret_core::Edges::all(Px(64.0)),
                    ..Default::default()
                },
                move |_cx| vec![calendar],
            )]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
                .map(|n| n.id)
                .expect("missing fret focused day semantics node")
        },
    );

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
        .expect("missing semantics for focused day button");

    let ring_quad =
        find_focus_ring_quad(&scene, button.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "calendar-14 focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_calendar_14_vp375x320_focus_ring_matches_web() {
    let web = read_web_golden("calendar-14.focus-kbd-selected-vp375x320");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_focused_button = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.contains_key("aria-label")
            && web_box_shadow_focus_ring(n).is_some()
    })
    .expect("web focused calendar day button");

    let web_label = web_focused_button
        .attrs
        .get("aria-label")
        .expect("web focused day aria-label");
    let (selected_date, _selected) =
        parse_calendar_day_aria_label(web_label).expect("parse web focused day aria-label");

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_focused_button).expect("web focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            use fret_ui_headless::calendar::CalendarMonth;

            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let border = theme.color_required("border");

            let month_model: fret_runtime::Model<CalendarMonth> = cx.app.models_mut().insert(
                CalendarMonth::new(selected_date.year(), selected_date.month()),
            );
            let selected: fret_runtime::Model<Option<Date>> =
                cx.app.models_mut().insert(Some(selected_date));

            let calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                .cell_size(Px(web_focused_button.rect.w))
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(fret_ui_kit::Radius::Lg)
                        .border_1()
                        .border_color(fret_ui_kit::ColorRef::Color(border))
                        .shadow_sm(),
                )
                .into_element(cx);

            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        layout
                    },
                    padding: fret_core::Edges::all(Px(64.0)),
                    ..Default::default()
                },
                move |_cx| vec![calendar],
            )]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
                .map(|n| n.id)
                .expect("missing fret focused day semantics node")
        },
    );

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
        .expect("missing semantics for focused day button");

    let ring_quad =
        find_focus_ring_quad(&scene, button.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "calendar-14.vp375x320 focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_calendar_03_focus_ring_matches_web() {
    let web = read_web_golden("calendar-03.focus-kbd-selected");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_focused_button = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.contains_key("aria-label")
            && web_box_shadow_focus_ring(n).is_some()
    })
    .expect("web focused calendar day button");

    let web_label = web_focused_button
        .attrs
        .get("aria-label")
        .expect("web focused day aria-label");
    let (focused_date, _selected) =
        parse_calendar_day_aria_label(web_label).expect("parse web focused day aria-label");

    let selected_dates: Vec<Date> = find_all(&theme.root, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .into_iter()
    .filter_map(|n| n.attrs.get("aria-label"))
    .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
    .map(|(d, _)| d)
    .collect();

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_focused_button).expect("web focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            use fret_ui_headless::calendar::CalendarMonth;

            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let border = theme.color_required("border");

            let month_model: fret_runtime::Model<CalendarMonth> = cx.app.models_mut().insert(
                CalendarMonth::new(focused_date.year(), focused_date.month()),
            );

            let selected: fret_runtime::Model<Vec<Date>> =
                cx.app.models_mut().insert(selected_dates.clone());

            let calendar = fret_ui_shadcn::CalendarMultiple::new(month_model, selected)
                .required(true)
                .max(5)
                .cell_size(Px(web_focused_button.rect.w))
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(fret_ui_kit::Radius::Lg)
                        .border_1()
                        .border_color(fret_ui_kit::ColorRef::Color(border))
                        .shadow_sm(),
                )
                .into_element(cx);

            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        layout
                    },
                    padding: fret_core::Edges::all(Px(64.0)),
                    ..Default::default()
                },
                move |_cx| vec![calendar],
            )]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
                .map(|n| n.id)
                .expect("missing fret focused day semantics node")
        },
    );

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
        .expect("missing semantics for focused day button");

    let ring_quad =
        find_focus_ring_quad(&scene, button.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "calendar-03 focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_calendar_03_vp375x320_focus_ring_matches_web() {
    let web = read_web_golden("calendar-03.focus-kbd-selected-vp375x320");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_focused_button = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.contains_key("aria-label")
            && web_box_shadow_focus_ring(n).is_some()
    })
    .expect("web focused calendar day button");

    let web_label = web_focused_button
        .attrs
        .get("aria-label")
        .expect("web focused day aria-label");
    let (focused_date, _selected) =
        parse_calendar_day_aria_label(web_label).expect("parse web focused day aria-label");

    let selected_dates: Vec<Date> = find_all(&theme.root, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .into_iter()
    .filter_map(|n| n.attrs.get("aria-label"))
    .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
    .map(|(d, _)| d)
    .collect();

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_focused_button).expect("web focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            use fret_ui_headless::calendar::CalendarMonth;

            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let border = theme.color_required("border");

            let month_model: fret_runtime::Model<CalendarMonth> = cx.app.models_mut().insert(
                CalendarMonth::new(focused_date.year(), focused_date.month()),
            );

            let selected: fret_runtime::Model<Vec<Date>> =
                cx.app.models_mut().insert(selected_dates.clone());

            let calendar = fret_ui_shadcn::CalendarMultiple::new(month_model, selected)
                .required(true)
                .max(5)
                .cell_size(Px(web_focused_button.rect.w))
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(fret_ui_kit::Radius::Lg)
                        .border_1()
                        .border_color(fret_ui_kit::ColorRef::Color(border))
                        .shadow_sm(),
                )
                .into_element(cx);

            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        layout
                    },
                    padding: fret_core::Edges::all(Px(64.0)),
                    ..Default::default()
                },
                move |_cx| vec![calendar],
            )]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
                .map(|n| n.id)
                .expect("missing fret focused day semantics node")
        },
    );

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
        .expect("missing semantics for focused day button");

    let ring_quad =
        find_focus_ring_quad(&scene, button.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "calendar-03.vp375x320 focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_calendar_04_focus_ring_matches_web() {
    let web = read_web_golden("calendar-04.focus-kbd-range-start");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_focused_button = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.contains_key("aria-label")
            && web_box_shadow_focus_ring(n).is_some()
    })
    .expect("web focused calendar day button");

    let web_label = web_focused_button
        .attrs
        .get("aria-label")
        .expect("web focused day aria-label");
    let (focused_date, _selected) =
        parse_calendar_day_aria_label(web_label).expect("parse web focused day aria-label");

    let selected_dates: Vec<Date> = find_all(&theme.root, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .into_iter()
    .filter_map(|n| n.attrs.get("aria-label"))
    .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
    .map(|(d, _)| d)
    .collect();
    assert!(
        selected_dates.len() >= 2,
        "expected at least 2 selected dates for range mode"
    );
    let (range_min, range_max) = selected_dates
        .iter()
        .copied()
        .fold((selected_dates[0], selected_dates[0]), |(min, max), d| {
            (min.min(d), max.max(d))
        });

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_focused_button).expect("web focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let border = theme.color_required("border");

            let month_model: fret_runtime::Model<CalendarMonth> = cx.app.models_mut().insert(
                CalendarMonth::new(focused_date.year(), focused_date.month()),
            );

            let selected: fret_runtime::Model<DateRangeSelection> =
                cx.app.models_mut().insert(DateRangeSelection {
                    from: Some(range_min),
                    to: Some(range_max),
                });

            let calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
                .cell_size(Px(web_focused_button.rect.w))
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(fret_ui_kit::Radius::Lg)
                        .border_1()
                        .border_color(fret_ui_kit::ColorRef::Color(border))
                        .shadow_sm(),
                )
                .into_element(cx);

            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        layout
                    },
                    padding: fret_core::Edges::all(Px(64.0)),
                    ..Default::default()
                },
                move |_cx| vec![calendar],
            )]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
                .map(|n| n.id)
                .expect("missing fret focused day semantics node")
        },
    );

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
        .expect("missing semantics for focused day button");

    let ring_quad =
        find_focus_ring_quad(&scene, button.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "calendar-04 focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_calendar_04_vp375x320_focus_ring_matches_web() {
    let web = read_web_golden("calendar-04.focus-kbd-range-start-vp375x320");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_focused_button = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.contains_key("aria-label")
            && web_box_shadow_focus_ring(n).is_some()
    })
    .expect("web focused calendar day button");

    let web_label = web_focused_button
        .attrs
        .get("aria-label")
        .expect("web focused day aria-label");
    let (focused_date, _selected) =
        parse_calendar_day_aria_label(web_label).expect("parse web focused day aria-label");

    let selected_dates: Vec<Date> = find_all(&theme.root, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .into_iter()
    .filter_map(|n| n.attrs.get("aria-label"))
    .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
    .map(|(d, _)| d)
    .collect();
    assert!(
        selected_dates.len() >= 2,
        "expected at least 2 selected dates for range mode"
    );
    let (range_min, range_max) = selected_dates
        .iter()
        .copied()
        .fold((selected_dates[0], selected_dates[0]), |(min, max), d| {
            (min.min(d), max.max(d))
        });

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_focused_button).expect("web focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let border = theme.color_required("border");

            let month_model: fret_runtime::Model<CalendarMonth> = cx.app.models_mut().insert(
                CalendarMonth::new(focused_date.year(), focused_date.month()),
            );

            let selected: fret_runtime::Model<DateRangeSelection> =
                cx.app.models_mut().insert(DateRangeSelection {
                    from: Some(range_min),
                    to: Some(range_max),
                });

            let calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
                .cell_size(Px(web_focused_button.rect.w))
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(fret_ui_kit::Radius::Lg)
                        .border_1()
                        .border_color(fret_ui_kit::ColorRef::Color(border))
                        .shadow_sm(),
                )
                .into_element(cx);

            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        layout
                    },
                    padding: fret_core::Edges::all(Px(64.0)),
                    ..Default::default()
                },
                move |_cx| vec![calendar],
            )]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
                .map(|n| n.id)
                .expect("missing fret focused day semantics node")
        },
    );

    let button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(web_label))
        .expect("missing semantics for focused day button");

    let ring_quad =
        find_focus_ring_quad(&scene, button.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "calendar-04.vp375x320 focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
}

#[test]
fn web_vs_fret_select_demo_aria_invalid_focus_ring_matches() {
    let web = read_web_golden("select-demo.invalid-focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_trigger = find_first(&theme.root, &|n| {
        n.tag == "button" && n.attrs.get("role").is_some_and(|v| v == "combobox")
    })
    .expect("web select trigger node");

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_trigger).expect("web select invalid focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let model: fret_runtime::Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            let open: fret_runtime::Model<bool> = cx.app.models_mut().insert(false);
            vec![
                fret_ui_shadcn::Select::new(model, open)
                    .a11y_label("SelectInvalidFocus")
                    .aria_invalid(true)
                    .item(fret_ui_shadcn::SelectItem::new("one", "One"))
                    .item(fret_ui_shadcn::SelectItem::new("two", "Two"))
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(web_trigger.rect.w))
                            .h_px(Px(web_trigger.rect.h)),
                    )
                    .into_element(cx),
            ]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::ComboBox
                        && n.label.as_deref() == Some("SelectInvalidFocus")
                })
                .map(|n| n.id)
                .expect("missing fret select semantics node")
        },
    );

    let select = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::ComboBox && n.label.as_deref() == Some("SelectInvalidFocus")
        })
        .expect("missing semantics for select");

    let ring_quad =
        find_focus_ring_quad(&scene, select.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "select-demo invalid focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
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
fn web_vs_fret_switch_demo_focus_ring_matches() {
    let web = read_web_golden("switch-demo.focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_switch = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.id.as_deref() == Some("airplane-mode")
            && n.attrs.get("role").is_some_and(|v| v == "switch")
    })
    .expect("web switch track node");

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_switch).expect("web switch focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let model: fret_runtime::Model<bool> = cx.app.models_mut().insert(false);
            vec![
                fret_ui_shadcn::Switch::new(model)
                    .a11y_label("SwitchFocus")
                    .into_element(cx),
            ]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Switch && n.label.as_deref() == Some("SwitchFocus")
                })
                .map(|n| n.id)
                .expect("missing fret switch semantics node")
        },
    );

    let switch = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Switch && n.label.as_deref() == Some("SwitchFocus"))
        .expect("missing semantics for switch");

    let ring_quad =
        find_focus_ring_quad(&scene, switch.bounds, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "switch-demo focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
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
fn web_vs_fret_checkbox_demo_focus_ring_matches() {
    let web = read_web_golden("checkbox-demo.focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_checkbox = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.id.as_deref() == Some("terms")
            && n.attrs.get("role").is_some_and(|v| v == "checkbox")
    })
    .expect("web checkbox node");

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_checkbox).expect("web checkbox focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let model: fret_runtime::Model<bool> = cx.app.models_mut().insert(false);
            vec![
                fret_ui_shadcn::Checkbox::new(model)
                    .a11y_label("CheckboxFocus")
                    .into_element(cx),
            ]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Checkbox && n.label.as_deref() == Some("CheckboxFocus")
                })
                .map(|n| n.id)
                .expect("missing fret checkbox semantics node")
        },
    );

    let checkbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Checkbox && n.label.as_deref() == Some("CheckboxFocus"))
        .expect("missing semantics for checkbox");

    let ring_quad = find_focus_ring_quad(&scene, checkbox.bounds, expected_ring_spread)
        .expect("focus ring quad");
    assert_color_close(
        "checkbox-demo focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
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
                background,
                border,
                border_paint,
                corner_radii,
                ..
            } = *op
            else {
                continue;
            };
            let fret_core::Paint::Solid(background) = background else {
                continue;
            };
            let fret_core::Paint::Solid(border_color) = border_paint else {
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
                    border_color,
                    background,
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
fn web_vs_fret_radio_group_demo_focus_ring_matches() {
    let web = read_web_golden("radio-group-demo.focus");
    let theme = web
        .themes
        .get("light")
        .or_else(|| web.themes.get("dark"))
        .expect("missing theme in web golden");

    let web_radio = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|v| v == "radio")
            && n.id.as_deref() == Some("r2")
    })
    .expect("web radio control node");

    let (expected_ring_color, expected_ring_spread) =
        web_box_shadow_focus_ring(web_radio).expect("web radio focus ring");

    let (snap, scene) = render_and_paint_with_focus_in_bounds(
        CoreSize::new(Px(1024.0), Px(768.0)),
        |cx| {
            let items = vec![
                fret_ui_shadcn::RadioGroupItem::new("default", "Default"),
                fret_ui_shadcn::RadioGroupItem::new("comfortable", "Comfortable"),
                fret_ui_shadcn::RadioGroupItem::new("compact", "Compact"),
            ];

            let group = items.into_iter().fold(
                fret_ui_shadcn::RadioGroup::uncontrolled(Some("comfortable")).a11y_label("Options"),
                |group, item| group.item(item),
            );

            vec![group.into_element(cx)]
        },
        |snap| {
            snap.nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::RadioButton
                        && n.label.as_deref() == Some("Comfortable")
                })
                .map(|n| n.id)
                .expect("missing fret radio semantics node")
        },
    );

    let radio_row = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::RadioButton && n.label.as_deref() == Some("Comfortable"))
        .expect("missing semantics for radio");

    let target = Rect::new(radio_row.bounds.origin, CoreSize::new(Px(16.0), Px(16.0)));
    let ring_quad =
        find_focus_ring_quad(&scene, target, expected_ring_spread).expect("focus ring quad");
    assert_color_close(
        "radio-group-demo focus ring color",
        ring_quad.border_color,
        &expected_ring_color,
        0.06,
    );
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
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(web_w)))
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
                        .w_px(Px(web_w))
                        .h_px(Px(web_h)),
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
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(web_w)))
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
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(web_w)))
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
