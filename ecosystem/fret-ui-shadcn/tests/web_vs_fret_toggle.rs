use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_kit::Space;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::text as decl_text;
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

fn web_theme<'a>(web: &'a WebGolden) -> &'a WebGoldenTheme {
    web.themes.get("light").expect("missing light theme")
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

#[derive(Default)]
struct StyleAwareServices;

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

fn run_fret_root_with_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
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
        "web-vs-fret-toggle",
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

fn assert_close_px(label: &str, actual: Px, expected: f32, tol: f32) {
    let delta = (actual.0 - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected≈{expected} (±{tol}) got={}",
        actual.0
    );
}

fn web_find_button_by_aria_label<'a>(root: &'a WebNode, aria_label: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.tag == "button" && n.attrs.get("aria-label").is_some_and(|v| v == aria_label)
    })
}

fn web_find_toggle_group_container<'a>(root: &'a WebNode) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.tag == "div" && n.attrs.get("role").is_some_and(|v| v == "group")
    })
}

const TOGGLE_KEYS: &[&str] = &[
    "toggle-disabled",
    "toggle-group-disabled",
    "toggle-group-lg",
    "toggle-group-outline",
    "toggle-group-single",
    "toggle-group-sm",
    "toggle-group-spacing",
    "toggle-lg",
    "toggle-outline",
    "toggle-sm",
    "toggle-with-text",
];

#[test]
fn shadcn_toggle_goldens_are_targeted_gates() {
    for &key in TOGGLE_KEYS {
        let web = read_web_golden(key);
        let theme = web_theme(&web);

        let has_button_with_aria_label = find_first(&theme.root, &|n| {
            n.tag == "button" && n.attrs.get("aria-label").is_some()
        })
        .is_some();

        assert!(
            has_button_with_aria_label,
            "expected at least one <button aria-label=...> in {key}"
        );
    }
}

fn assert_toggle_variant_geometry_matches(
    web_name: &str,
    aria_label: &str,
    size: fret_ui_shadcn::ToggleSize,
    variant: fret_ui_shadcn::ToggleVariant,
    disabled: bool,
    with_text: bool,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_toggle = web_find_button_by_aria_label(&theme.root, aria_label).expect("web toggle");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let model: Model<bool> = cx.app.models_mut().insert(false);
        let mut toggle = fret_ui_shadcn::Toggle::new(model)
            .size(size)
            .variant(variant)
            .disabled(disabled)
            .a11y_label(aria_label)
            .children(vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)]);

        if with_text {
            toggle = toggle.label("Italic");
        }

        vec![toggle.into_element(cx)]
    });

    let toggle = find_semantics(&snap, SemanticsRole::Button, Some(aria_label))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret toggle semantics node");

    assert_close_px(
        &format!("{web_name} height"),
        toggle.bounds.size.height,
        web_toggle.rect.h,
        1.0,
    );

    if !with_text {
        assert_close_px(
            &format!("{web_name} width"),
            toggle.bounds.size.width,
            web_toggle.rect.w,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_toggle_sm_geometry_matches() {
    assert_toggle_variant_geometry_matches(
        "toggle-sm",
        "Toggle italic",
        fret_ui_shadcn::ToggleSize::Sm,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        false,
    );
}

#[test]
fn web_vs_fret_toggle_lg_geometry_matches() {
    assert_toggle_variant_geometry_matches(
        "toggle-lg",
        "Toggle italic",
        fret_ui_shadcn::ToggleSize::Lg,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        false,
    );
}

#[test]
fn web_vs_fret_toggle_outline_geometry_matches() {
    assert_toggle_variant_geometry_matches(
        "toggle-outline",
        "Toggle italic",
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Outline,
        false,
        false,
    );
}

#[test]
fn web_vs_fret_toggle_disabled_geometry_matches() {
    assert_toggle_variant_geometry_matches(
        "toggle-disabled",
        "Toggle italic",
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Default,
        true,
        false,
    );
}

#[test]
fn web_vs_fret_toggle_with_text_height_matches() {
    assert_toggle_variant_geometry_matches(
        "toggle-with-text",
        "Toggle italic",
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        true,
    );
}

#[derive(Debug, Clone, Copy)]
struct ToggleGroupItemSpec {
    value: &'static str,
    a11y_label: &'static str,
    text: Option<&'static str>,
}

fn assert_toggle_group_variant_heights_match(
    web_name: &str,
    group_kind: fret_ui_shadcn::ToggleGroupKind,
    size: fret_ui_shadcn::ToggleSize,
    variant: fret_ui_shadcn::ToggleVariant,
    disabled: bool,
    spacing: Space,
    items: &'static [ToggleGroupItemSpec],
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);
    let web_group =
        web_find_toggle_group_container(&theme.root).expect("web toggle-group container");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let group_label: Arc<str> = Arc::from(format!("Golden:{web_name}:group"));

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let model_single: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
        let model_multi: Model<Vec<Arc<str>>> = cx.app.models_mut().insert(Vec::new());

        let base = match group_kind {
            fret_ui_shadcn::ToggleGroupKind::Single => {
                fret_ui_shadcn::ToggleGroup::single(model_single)
            }
            fret_ui_shadcn::ToggleGroupKind::Multiple => {
                fret_ui_shadcn::ToggleGroup::multiple(model_multi)
            }
        };

        let group = items.iter().fold(
            base.size(size)
                .variant(variant)
                .disabled(disabled)
                .spacing(spacing),
            |group, spec| {
                let mut children: Vec<fret_ui::element::AnyElement> =
                    vec![decl_icon::icon(cx, fret_icons::ids::ui::SEARCH)];
                if let Some(text) = spec.text {
                    children.push(decl_text::text_sm(cx, text));
                }
                group.item(
                    fret_ui_shadcn::ToggleGroupItem::new(spec.value, children)
                        .a11y_label(spec.a11y_label),
                )
            },
        );

        let group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(group_label.clone()),
                ..Default::default()
            },
            move |cx| vec![group.into_element(cx)],
        );

        vec![group]
    });

    let group = find_semantics(&snap, SemanticsRole::Panel, Some(group_label.as_ref()))
        .expect("fret toggle-group semantics node");
    assert_close_px(
        &format!("{web_name} group height"),
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );

    let item_role = match group_kind {
        fret_ui_shadcn::ToggleGroupKind::Single => SemanticsRole::RadioButton,
        fret_ui_shadcn::ToggleGroupKind::Multiple => SemanticsRole::Button,
    };

    for spec in items {
        let web_item =
            web_find_button_by_aria_label(&theme.root, spec.a11y_label).expect("web toggle item");
        let item = find_semantics(&snap, item_role, Some(spec.a11y_label))
            .expect("fret toggle item semantics node");
        assert_close_px(
            &format!("{web_name} item height ({})", spec.a11y_label),
            item.bounds.size.height,
            web_item.rect.h,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_toggle_group_sm_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "bold",
            a11y_label: "Toggle bold",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "italic",
            a11y_label: "Toggle italic",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "strikethrough",
            a11y_label: "Toggle strikethrough",
            text: None,
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-sm",
        fret_ui_shadcn::ToggleGroupKind::Single,
        fret_ui_shadcn::ToggleSize::Sm,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        Space::N0,
        ITEMS,
    );
}

#[test]
fn web_vs_fret_toggle_group_lg_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "bold",
            a11y_label: "Toggle bold",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "italic",
            a11y_label: "Toggle italic",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "strikethrough",
            a11y_label: "Toggle strikethrough",
            text: None,
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-lg",
        fret_ui_shadcn::ToggleGroupKind::Multiple,
        fret_ui_shadcn::ToggleSize::Lg,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        Space::N0,
        ITEMS,
    );
}

#[test]
fn web_vs_fret_toggle_group_outline_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "bold",
            a11y_label: "Toggle bold",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "italic",
            a11y_label: "Toggle italic",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "strikethrough",
            a11y_label: "Toggle strikethrough",
            text: None,
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-outline",
        fret_ui_shadcn::ToggleGroupKind::Multiple,
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Outline,
        false,
        Space::N0,
        ITEMS,
    );
}

#[test]
fn web_vs_fret_toggle_group_disabled_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "bold",
            a11y_label: "Toggle bold",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "italic",
            a11y_label: "Toggle italic",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "strikethrough",
            a11y_label: "Toggle strikethrough",
            text: None,
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-disabled",
        fret_ui_shadcn::ToggleGroupKind::Multiple,
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Default,
        true,
        Space::N0,
        ITEMS,
    );
}

#[test]
fn web_vs_fret_toggle_group_single_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "bold",
            a11y_label: "Toggle bold",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "italic",
            a11y_label: "Toggle italic",
            text: None,
        },
        ToggleGroupItemSpec {
            value: "strikethrough",
            a11y_label: "Toggle strikethrough",
            text: None,
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-single",
        fret_ui_shadcn::ToggleGroupKind::Single,
        fret_ui_shadcn::ToggleSize::Default,
        fret_ui_shadcn::ToggleVariant::Default,
        false,
        Space::N0,
        ITEMS,
    );
}

#[test]
fn web_vs_fret_toggle_group_spacing_heights_match() {
    const ITEMS: &[ToggleGroupItemSpec] = &[
        ToggleGroupItemSpec {
            value: "star",
            a11y_label: "Toggle star",
            text: Some("Star"),
        },
        ToggleGroupItemSpec {
            value: "heart",
            a11y_label: "Toggle heart",
            text: Some("Heart"),
        },
        ToggleGroupItemSpec {
            value: "bookmark",
            a11y_label: "Toggle bookmark",
            text: Some("Bookmark"),
        },
    ];
    assert_toggle_group_variant_heights_match(
        "toggle-group-spacing",
        fret_ui_shadcn::ToggleGroupKind::Multiple,
        fret_ui_shadcn::ToggleSize::Sm,
        fret_ui_shadcn::ToggleVariant::Outline,
        false,
        Space::N2,
        ITEMS,
    );
}
