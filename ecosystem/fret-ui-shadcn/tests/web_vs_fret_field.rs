use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::Theme;
use fret_ui::element::{AnyElement, ContainerProps};
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
    #[serde(rename = "className")]
    class_name: Option<String>,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    #[serde(default)]
    text: Option<String>,
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

fn contains_text(node: &WebNode, needle: &str) -> bool {
    node.text.as_deref().is_some_and(|t| t.contains(needle))
        || node.children.iter().any(|c| contains_text(c, needle))
}

fn web_find_by_tag_and_text<'a>(root: &'a WebNode, tag: &str, text: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| n.tag == tag && contains_text(n, text))
}

fn web_find_by_class_tokens<'a>(root: &'a WebNode, tokens: &[&str]) -> Option<&'a WebNode> {
    find_first(root, &|n| tokens.iter().all(|t| class_has_token(n, t)))
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
struct StyleAwareServices {
    prepared: Vec<RecordedTextPrepare>,
}

#[derive(Debug, Clone)]
struct RecordedTextPrepare {
    text: String,
    style: fret_core::TextStyle,
    constraints: fret_core::TextConstraints,
}

impl fret_core::TextService for StyleAwareServices {
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
        self.prepared.push(RecordedTextPrepare {
            text: text.to_string(),
            style: style.clone(),
            constraints,
        });

        let line_height = style
            .line_height
            .unwrap_or(Px((style.size.0 * 1.4).max(0.0)));

        let char_w = (style.size.0 * 0.55).max(1.0);
        let est_w = Px(char_w * text.chars().count() as f32);

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

impl fret_core::MaterialService for StyleAwareServices {
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

fn run_fret_root_with_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
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

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        services,
        window,
        bounds,
        "web-vs-fret-field",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, services, bounds, 1.0);

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

const FIELD_KEYS: &[&str] = &[
    "field-checkbox",
    "field-choice-card",
    "field-demo",
    "field-fieldset",
    "field-group",
    "field-input",
    "field-radio",
    "field-responsive",
    "field-select",
    "field-slider",
    "field-switch",
    "field-textarea",
];

#[test]
fn shadcn_field_goldens_are_targeted_gates() {
    for &key in FIELD_KEYS {
        let web = read_web_golden(key);
        let theme = web_theme(&web);

        let found = find_first(&theme.root, &|n| {
            n.tag == "input"
                || n.tag == "textarea"
                || n.tag == "select"
                || n.tag == "button"
                || n.tag == "fieldset"
                || n.attrs.get("role").is_some()
        });
        assert!(
            found.is_some(),
            "expected at least one field/control element in {key}"
        );
    }
}

#[test]
fn web_vs_fret_field_input_geometry() {
    let web = read_web_golden("field-input");
    let theme = web_theme(&web);

    let web_username_label =
        web_find_by_tag_and_text(&theme.root, "label", "Username").expect("web username label");
    let web_password_label =
        web_find_by_tag_and_text(&theme.root, "label", "Password").expect("web password label");

    let web_username_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");
    let web_inputs: Vec<&WebNode> = {
        let mut out = Vec::new();
        fn walk<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
            if n.tag == "input" {
                out.push(n);
            }
            for c in &n.children {
                walk(c, out);
            }
        }
        walk(&theme.root, &mut out);
        out.sort_by(|a, b| {
            a.rect
                .y
                .partial_cmp(&b.rect.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    };
    let web_password_input = web_inputs.get(1).copied().unwrap_or(web_username_input);
    let web_username_desc = web_find_by_tag_and_text(
        &theme.root,
        "p",
        "Choose a unique username for your account.",
    )
    .expect("web username desc");
    let web_password_desc = web_find_by_tag_and_text(&theme.root, "p", "Must be at least 8")
        .expect("web password desc");

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root container");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let username: Model<String> = cx.app.models_mut().insert(String::new());
        let password: Model<String> = cx.app.models_mut().insert(String::new());

        let username_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:username:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Username").into_element(cx)],
        );
        let username_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-input:username:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(username)
                        .a11y_label("Username")
                        .placeholder("Max Leiter")
                        .into_element(cx),
                ]
            },
        );
        let username_desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:username:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new(
                        "Choose a unique username for your account.",
                    )
                    .into_element(cx),
                ]
            },
        );

        let username_field =
            fret_ui_shadcn::Field::new(vec![username_label, username_input, username_desc])
                .into_element(cx);

        let password_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:password:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Password").into_element(cx)],
        );
        let password_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-input:password:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(password)
                        .a11y_label("Password")
                        .placeholder("????????")
                        .into_element(cx),
                ]
            },
        );
        let password_desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:password:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new("Must be at least 8 characters long.")
                        .into_element(cx),
                ]
            },
        );

        let password_field =
            fret_ui_shadcn::Field::new(vec![password_label, password_desc, password_input])
                .into_element(cx);

        let group =
            fret_ui_shadcn::FieldGroup::new(vec![username_field, password_field]).into_element(cx);
        let set = fret_ui_shadcn::FieldSet::new(vec![group]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_px(Px(web_root.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:field-input:root"))
        .expect("fret root");

    let username_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:username:label"),
    )
    .expect("fret username label");
    let username_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-input:username:input"),
    )
    .expect("fret username input");
    let username_desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:username:desc"),
    )
    .expect("fret username desc");

    let password_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:password:label"),
    )
    .expect("fret password label");
    let password_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-input:password:input"),
    )
    .expect("fret password input");
    let password_desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:password:desc"),
    )
    .expect("fret password desc");

    assert_close_px(
        "field-input root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-input username label y",
        username_label.bounds.origin.y,
        web_username_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input username input y",
        username_input.bounds.origin.y,
        web_username_input.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input username desc y",
        username_desc.bounds.origin.y,
        web_username_desc.rect.y,
        1.0,
    );

    let username_label_to_input_gap = username_input.bounds.origin.y.0
        - (username_label.bounds.origin.y.0 + username_label.bounds.size.height.0);
    assert!(
        (username_label_to_input_gap - 12.0).abs() <= 1.0,
        "field-input username label->input gap: expected ~12 got={username_label_to_input_gap}"
    );

    let username_input_to_desc_gap = username_desc.bounds.origin.y.0
        - (username_input.bounds.origin.y.0 + username_input.bounds.size.height.0);
    assert!(
        (username_input_to_desc_gap - 12.0).abs() <= 1.0,
        "field-input username input->desc gap: expected ~12 got={username_input_to_desc_gap}"
    );

    assert_close_px(
        "field-input password label y",
        password_label.bounds.origin.y,
        web_password_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input password desc y",
        password_desc.bounds.origin.y,
        web_password_desc.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input password input y",
        password_input.bounds.origin.y,
        web_password_input.rect.y,
        1.0,
    );

    let password_label_to_desc_gap = password_desc.bounds.origin.y.0
        - (password_label.bounds.origin.y.0 + password_label.bounds.size.height.0);
    assert!(
        (password_label_to_desc_gap - 8.0).abs() <= 1.0,
        "field-input password label->desc gap: expected ~8 got={password_label_to_desc_gap}"
    );

    let password_desc_to_input_gap = password_input.bounds.origin.y.0
        - (password_desc.bounds.origin.y.0 + password_desc.bounds.size.height.0);
    assert!(
        (password_desc_to_input_gap - 12.0).abs() <= 1.0,
        "field-input password desc->input gap: expected ~12 got={password_desc_to_input_gap}"
    );

    let field_to_field_gap = password_label.bounds.origin.y.0
        - (username_desc.bounds.origin.y.0 + username_desc.bounds.size.height.0);
    assert!(
        (field_to_field_gap - 28.0).abs() <= 1.0,
        "field-input field->field gap: expected ~28 got={field_to_field_gap}"
    );
}
