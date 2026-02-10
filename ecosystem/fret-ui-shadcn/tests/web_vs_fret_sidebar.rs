use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size as CoreSize, TextWrap};
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_shadcn::sidebar::SidebarMenuButtonSize;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
    #[serde(default)]
    attrs: BTreeMap<String, String>,
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
    golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden")
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
        .is_some_and(|c| c.split_whitespace().any(|t| t == token))
}

fn assert_close_px(label: &str, actual: Px, expected: f32, tol: f32) {
    let delta = (actual.0 - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected≈{expected} (±{tol}) got={}",
        actual.0
    );
}

#[derive(Default)]
struct FakeServices;

impl fret_core::TextService for FakeServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        let (text, style) = match input {
            fret_core::TextInput::Plain { text, style } => (text.as_ref(), style),
            fret_core::TextInput::Attributed { text, base, .. } => (text.as_ref(), base),
            _ => {
                debug_assert!(false, "unsupported TextInput variant");
                return (
                    fret_core::TextBlobId::default(),
                    fret_core::TextMetrics {
                        size: CoreSize::new(Px(0.0), Px(0.0)),
                        baseline: Px(0.0),
                    },
                );
            }
        };

        let line_height = style
            .line_height
            .unwrap_or(Px((style.size.0 * 1.4).max(0.0)));

        let char_w = (style.size.0 * 0.55).max(1.0);
        let est_w = Px(char_w * text.chars().count() as f32);

        let max_w = constraints.max_width.unwrap_or(est_w);
        let (lines, w) = match constraints.wrap {
            TextWrap::Word if max_w.0.is_finite() && max_w.0 > 0.0 => {
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
                baseline: Px(h.0 * 0.8),
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
        "web-vs-fret-sidebar",
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

fn web_find_sidebar_menu_button_by_height<'a>(
    root: &'a WebNode,
    height_token: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        (n.tag == "button" || n.tag == "a")
            && class_has_token(n, "peer/menu-button")
            && class_has_token(n, height_token)
    })
}

fn assert_sidebar_menu_button_heights_match_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_default = web_find_sidebar_menu_button_by_height(&theme.root, "h-8")
        .unwrap_or_else(|| panic!("missing web sidebar menu button (h-8) in {web_name}"));
    let web_lg = web_find_sidebar_menu_button_by_height(&theme.root, "h-12");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap_default = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::SidebarMenuButton::new("Sidebar Menu Button")
                .size(SidebarMenuButtonSize::Default)
                .into_element(cx),
        ]
    });

    let fret_default = find_semantics(
        &snap_default,
        SemanticsRole::Button,
        Some("Sidebar Menu Button"),
    )
    .or_else(|| find_semantics(&snap_default, SemanticsRole::Button, None))
    .expect("fret sidebar menu button (default) semantics node");

    assert_close_px(
        &format!("{web_name} menu button height (h-8)"),
        fret_default.bounds.size.height,
        web_default.rect.h,
        1.0,
    );

    if let Some(web_lg) = web_lg {
        let collapsed = (web_lg.rect.h - 32.0).abs() <= 1.0;
        let snap_lg = run_fret_root(bounds, |cx| {
            vec![
                fret_ui_shadcn::SidebarMenuButton::new("Sidebar Menu Button")
                    .size(SidebarMenuButtonSize::Lg)
                    .collapsed(collapsed)
                    .into_element(cx),
            ]
        });

        let fret_lg = find_semantics(&snap_lg, SemanticsRole::Button, Some("Sidebar Menu Button"))
            .or_else(|| find_semantics(&snap_lg, SemanticsRole::Button, None))
            .expect("fret sidebar menu button (lg) semantics node");

        assert_close_px(
            &format!("{web_name} menu button height (h-12)"),
            fret_lg.bounds.size.height,
            web_lg.rect.h,
            1.0,
        );
    }
}

const SIDEBAR_KEYS: &[&str] = &[
    "sidebar-01",
    "sidebar-02",
    "sidebar-03",
    "sidebar-04",
    "sidebar-05",
    "sidebar-06",
    "sidebar-07",
    "sidebar-08",
    "sidebar-09",
    "sidebar-10",
    "sidebar-11",
    "sidebar-12",
    "sidebar-14",
    "sidebar-15",
    "sidebar-16",
];

#[test]
fn shadcn_sidebar_goldens_are_targeted_gates() {
    for &key in SIDEBAR_KEYS {
        let web = read_web_golden(key);
        let theme = web_theme(&web);

        let found = find_first(&theme.root, &|n| {
            (n.tag == "button" || n.tag == "a") && class_has_token(n, "peer/menu-button")
        });
        assert!(
            found.is_some(),
            "expected at least one web sidebar menu button in {key}"
        );
    }
}

#[test]
fn web_vs_fret_sidebar_01_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-01");
}

#[test]
fn web_vs_fret_sidebar_02_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-02");
}

#[test]
fn web_vs_fret_sidebar_03_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-03");
}

#[test]
fn web_vs_fret_sidebar_04_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-04");
}

#[test]
fn web_vs_fret_sidebar_05_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-05");
}

#[test]
fn web_vs_fret_sidebar_06_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-06");
}

#[test]
fn web_vs_fret_sidebar_07_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-07");
}

#[test]
fn web_vs_fret_sidebar_08_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-08");
}

#[test]
fn web_vs_fret_sidebar_09_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-09");
}

#[test]
fn web_vs_fret_sidebar_10_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-10");
}

#[test]
fn web_vs_fret_sidebar_11_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-11");
}

#[test]
fn web_vs_fret_sidebar_12_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-12");
}

#[test]
fn web_vs_fret_sidebar_14_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-14");
}

#[test]
fn web_vs_fret_sidebar_15_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-15");
}

#[test]
fn web_vs_fret_sidebar_16_menu_button_heights_match_web() {
    assert_sidebar_menu_button_heights_match_web("sidebar-16");
}
