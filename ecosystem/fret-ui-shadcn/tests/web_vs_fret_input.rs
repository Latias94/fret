use fret_app::App;
use fret_core::{AppWindowId, NodeId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
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
    id: Option<String>,
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

fn collect_tag<'a>(node: &'a WebNode, tag: &str, out: &mut Vec<&'a WebNode>) {
    if node.tag == tag {
        out.push(node);
    }
    for child in &node.children {
        collect_tag(child, tag, out);
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
        let style = match input {
            fret_core::TextInput::Plain { style, .. } => style,
            fret_core::TextInput::Attributed { base, .. } => base,
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

        let text_len = match input {
            fret_core::TextInput::Plain { text, .. } => text.chars().count(),
            fret_core::TextInput::Attributed { text, .. } => text.chars().count(),
            _ => 0,
        };

        let char_w = (style.size.0 * 0.55).max(1.0);
        let est_w = Px(char_w * text_len as f32);

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
                baseline: Px(h.0 * 0.8),
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

fn run_fret_root_with_ui_and_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (UiTree<App>, fret_core::SemanticsSnapshot, NodeId) {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        services,
        window,
        bounds,
        "web-vs-fret-input",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    (ui, snap, root)
}

fn find_semantics<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes.iter().find(|n| n.role == role)
}

fn assert_close_px(label: &str, actual: Px, expected: f32, tol: f32) {
    let delta = (actual.0 - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected≈{expected} (±{tol}) got={}",
        actual.0
    );
}

const INPUT_KEYS: &[&str] = &[
    "input-disabled",
    "input-file",
    "input-group-button",
    "input-group-button-group",
    "input-group-custom",
    "input-group-demo",
    "input-group-dropdown",
    "input-group-icon",
    "input-group-label",
    "input-group-spinner",
    "input-group-text",
    "input-group-textarea",
    "input-group-tooltip",
    "input-otp-controlled",
    "input-otp-demo",
    "input-otp-pattern",
    "input-otp-separator",
    "input-with-button",
    "input-with-label",
    "input-with-text",
];

#[test]
fn shadcn_input_goldens_are_targeted_gates() {
    for &key in INPUT_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        let mut inputs = Vec::new();
        collect_tag(&theme.root, "input", &mut inputs);

        let mut textareas = Vec::new();
        collect_tag(&theme.root, "textarea", &mut textareas);

        assert!(
            !inputs.is_empty() || !textareas.is_empty(),
            "expected at least one input-like control in {key}"
        );
    }
}

#[test]
fn web_vs_fret_input_disabled_height_matches() {
    let web = read_web_golden("input-disabled");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let input = fret_ui_shadcn::Input::new(model)
            .disabled(true)
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("input-disabled")),
                ..Default::default()
            },
            move |_cx| vec![input],
        )]
    });

    let input = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::TextField)
        .or_else(|| find_semantics(&snap, SemanticsRole::Panel))
        .expect("fret input node");
    assert_close_px(
        "input-disabled height",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_input_file_height_matches() {
    let web = read_web_golden("input-file");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let input = fret_ui_shadcn::Input::new(model).into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("input-file")),
                ..Default::default()
            },
            move |_cx| vec![input],
        )]
    });

    let input = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::TextField)
        .or_else(|| find_semantics(&snap, SemanticsRole::Panel))
        .expect("fret input node");
    assert_close_px(
        "input-file height",
        input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_input_otp_demo_height_matches() {
    let web = read_web_golden("input-otp-demo");
    let theme = web.themes.get("light").expect("missing light theme");
    let otp_row = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("items-center") && c.contains("gap-2"))
    })
    .expect("web otp row");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let otp = fret_ui_shadcn::InputOtp::new(model)
            .length(6)
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("input-otp-demo")),
                ..Default::default()
            },
            move |_cx| vec![otp],
        )]
    });

    let otp = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("input-otp-demo"))
        .expect("fret otp wrapper");
    assert_close_px(
        "input-otp-demo height",
        otp.bounds.size.height,
        otp_row.rect.h,
        1.0,
    );
}
