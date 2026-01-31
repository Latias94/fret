use fret_app::App;
use fret_core::{
    AppWindowId, Edges, NodeId, Point, Px, Rect, SemanticsRole, Size as CoreSize, TextOverflow,
    TextWrap,
};
use fret_ui::Theme;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length};
use fret_ui::tree::UiTree;
use fret_ui_kit::ColorRef;
use fret_ui_kit::ui;
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
    #[serde(rename = "computedStyle")]
    computed_style: BTreeMap<String, String>,
    rect: WebRect,
    #[serde(default)]
    text: Option<String>,
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

fn web_collect_tag<'a>(node: &'a WebNode, tag: &str, out: &mut Vec<&'a WebNode>) {
    if node.tag == tag {
        out.push(node);
    }
    for child in &node.children {
        web_collect_tag(child, tag, out);
    }
}

fn web_css_px(node: &WebNode, key: &str) -> Px {
    let v = node
        .computed_style
        .get(key)
        .unwrap_or_else(|| panic!("missing computedStyle.{key} for <{}>", node.tag));
    let s = v.trim();
    let s = s.strip_suffix("px").unwrap_or_else(|| {
        panic!(
            "expected computedStyle.{key} to be px, got {v:?} for <{}>",
            node.tag
        )
    });
    Px(s.parse::<f32>().unwrap_or_else(|_| {
        panic!(
            "invalid computedStyle.{key} px value {v:?} for <{}>",
            node.tag
        )
    }))
}

fn web_css_u16(node: &WebNode, key: &str) -> u16 {
    let v = node
        .computed_style
        .get(key)
        .unwrap_or_else(|| panic!("missing computedStyle.{key} for <{}>", node.tag));
    v.trim().parse::<u16>().unwrap_or_else(|_| {
        panic!(
            "invalid computedStyle.{key} u16 value {v:?} for <{}>",
            node.tag
        )
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

fn assert_rect_close_px(label: &str, actual: Rect, expected: WebRect, tol: f32) {
    assert_close_px(&format!("{label} x"), actual.origin.x, expected.x, tol);
    assert_close_px(&format!("{label} y"), actual.origin.y, expected.y, tol);
    assert_close_px(&format!("{label} w"), actual.size.width, expected.w, tol);
    assert_close_px(&format!("{label} h"), actual.size.height, expected.h, tol);
}

// Note: scene-level paint assertions for `typography-table` still live in
// `web_vs_fret_layout.rs` for now; once we split that file, we can move the paint-backed checks
// here and re-introduce shared helpers.

#[derive(Debug, Clone)]
struct RecordedTextPrepare {
    text: String,
    style: fret_core::TextStyle,
    constraints: fret_core::TextConstraints,
}

#[derive(Default)]
struct StyleAwareServices {
    prepared: Vec<RecordedTextPrepare>,
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

fn run_fret_root_with_ui_and_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<AnyElement>,
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
        "web-vs-fret-typography",
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

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    test_id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| panic!("missing semantics node with test_id={test_id:?}"))
}

fn assert_prepared_text_style<'a>(
    services: &'a StyleAwareServices,
    expected_text: &str,
    expected_size: Px,
    expected_line_height: Px,
    expected_weight: u16,
) -> &'a RecordedTextPrepare {
    let record = services
        .prepared
        .iter()
        .rev()
        .find(|r| r.text == expected_text)
        .unwrap_or_else(|| {
            let mut texts: Vec<_> = services.prepared.iter().map(|r| r.text.as_str()).collect();
            texts.sort();
            panic!(
                "missing prepared text style for {expected_text:?}; seen {} prepares: {texts:?}",
                services.prepared.len()
            )
        });

    assert_eq!(record.style.size, expected_size, "text size mismatch");
    assert_eq!(
        record.style.line_height,
        Some(expected_line_height),
        "line height mismatch"
    );
    assert_eq!(
        record.style.weight.0, expected_weight,
        "font weight mismatch"
    );
    record
}

#[test]
fn web_vs_fret_typography_h1_geometry_light() {
    let web = read_web_golden("typography-h1");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h1 = find_first(&theme.root, &|n| n.tag == "h1").expect("web h1");

    let text = web_h1.text.clone().unwrap_or_default();
    let size = web_css_px(web_h1, "fontSize");
    let line_height = web_css_px(web_h1, "lineHeight");
    let weight = web_css_u16(web_h1, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h1")),
                ..Default::default()
            },
            move |_cx| vec![heading],
        )]
    });

    let h1 = find_by_test_id(&snap, "typography-h1");
    assert_rect_close_px("typography-h1", h1.bounds, web_h1.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_typography_h2_geometry_light() {
    let web = read_web_golden("typography-h2");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h2 = find_first(&theme.root, &|n| n.tag == "h2").expect("web h2");

    let text = web_h2.text.clone().unwrap_or_default();
    let size = web_css_px(web_h2, "fontSize");
    let line_height = web_css_px(web_h2, "lineHeight");
    let weight = web_css_u16(web_h2, "fontWeight");
    let padding_bottom = web_css_px(web_h2, "paddingBottom");
    let border_bottom = web_css_px(web_h2, "borderBottomWidth");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let border_color = theme.color_required("border");

        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        let container = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                padding: Edges {
                    bottom: padding_bottom,
                    ..Edges::all(Px(0.0))
                },
                border: Edges {
                    bottom: border_bottom,
                    ..Edges::all(Px(0.0))
                },
                border_color: Some(border_color),
                ..Default::default()
            },
            move |_cx| vec![heading],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h2")),
                ..Default::default()
            },
            move |_cx| vec![container],
        )]
    });

    let h2 = find_by_test_id(&snap, "typography-h2");
    assert_rect_close_px("typography-h2", h2.bounds, web_h2.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_typography_h3_geometry_light() {
    let web = read_web_golden("typography-h3");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h3 = find_first(&theme.root, &|n| n.tag == "h3").expect("web h3");

    let text = web_h3.text.clone().unwrap_or_default();
    let size = web_css_px(web_h3, "fontSize");
    let line_height = web_css_px(web_h3, "lineHeight");
    let weight = web_css_u16(web_h3, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h3")),
                ..Default::default()
            },
            move |_cx| vec![heading],
        )]
    });

    let h3 = find_by_test_id(&snap, "typography-h3");
    assert_rect_close_px("typography-h3", h3.bounds, web_h3.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_typography_h4_geometry_light() {
    let web = read_web_golden("typography-h4");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h4 = find_first(&theme.root, &|n| n.tag == "h4").expect("web h4");

    let text = web_h4.text.clone().unwrap_or_default();
    let size = web_css_px(web_h4, "fontSize");
    let line_height = web_css_px(web_h4, "lineHeight");
    let weight = web_css_u16(web_h4, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h4")),
                ..Default::default()
            },
            move |_cx| vec![heading],
        )]
    });

    let h4 = find_by_test_id(&snap, "typography-h4");
    assert_rect_close_px("typography-h4", h4.bounds, web_h4.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_typography_p_geometry_light() {
    let web = read_web_golden("typography-p");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");

    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let p = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-p")),
                ..Default::default()
            },
            move |_cx| vec![p],
        )]
    });

    let p = find_by_test_id(&snap, "typography-p");
    assert_rect_close_px("typography-p", p.bounds, web_p.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_typography_lead_geometry_light() {
    let web = read_web_golden("typography-lead");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");

    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let p = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .text_color(ColorRef::Token {
                key: "muted-foreground",
                fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
            })
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-lead")),
                ..Default::default()
            },
            move |_cx| vec![p],
        )]
    });

    let p = find_by_test_id(&snap, "typography-lead");
    assert_rect_close_px("typography-lead", p.bounds, web_p.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_typography_muted_geometry_light() {
    let web = read_web_golden("typography-muted");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");

    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let p = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .text_color(ColorRef::Token {
                key: "muted-foreground",
                fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
            })
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-muted")),
                ..Default::default()
            },
            move |_cx| vec![p],
        )]
    });

    let p = find_by_test_id(&snap, "typography-muted");
    assert_rect_close_px("typography-muted", p.bounds, web_p.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_typography_large_geometry_light() {
    let web = read_web_golden("typography-large");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_div =
        find_first(&theme.root, &|n| n.tag == "div" && n.text.is_some()).expect("web div");

    let text = web_div.text.clone().unwrap_or_default();
    let size = web_css_px(web_div, "fontSize");
    let line_height = web_css_px(web_div, "lineHeight");
    let weight = web_css_u16(web_div, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let div = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-large")),
                ..Default::default()
            },
            move |_cx| vec![div],
        )]
    });

    let div = find_by_test_id(&snap, "typography-large");
    assert_rect_close_px("typography-large", div.bounds, web_div.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

#[test]
fn web_vs_fret_typography_small_text_style_light() {
    let web = read_web_golden("typography-small");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_small = find_first(&theme.root, &|n| n.tag == "small").expect("web small");

    let text = web_small.text.clone().unwrap_or_default();
    let size = web_css_px(web_small, "fontSize");
    let line_height = web_css_px(web_small, "lineHeight");
    let weight = web_css_u16(web_small, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, _snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let small = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .nowrap()
            .into_element(cx);

        vec![small]
    });

    let record = assert_prepared_text_style(&services, &text, size, line_height, weight);
    assert_eq!(record.constraints.wrap, TextWrap::None);
}

#[test]
fn web_vs_fret_typography_blockquote_geometry_light() {
    let web = read_web_golden("typography-blockquote");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_bq = find_first(&theme.root, &|n| n.tag == "blockquote").expect("web blockquote");

    let text = web_bq.text.clone().unwrap_or_default();
    let size = web_css_px(web_bq, "fontSize");
    let line_height = web_css_px(web_bq, "lineHeight");
    let border_left = web_css_px(web_bq, "borderLeftWidth");
    let padding_left = web_css_px(web_bq, "paddingLeft");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let border_color = theme.color_required("border");

        let quote = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(400))
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
            .into_element(cx);

        let container = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                padding: Edges {
                    left: padding_left,
                    ..Edges::all(Px(0.0))
                },
                border: Edges {
                    left: border_left,
                    ..Edges::all(Px(0.0))
                },
                border_color: Some(border_color),
                ..Default::default()
            },
            move |_cx| vec![quote],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-blockquote")),
                ..Default::default()
            },
            move |_cx| vec![container],
        )]
    });

    let bq = find_by_test_id(&snap, "typography-blockquote");
    assert_rect_close_px("typography-blockquote", bq.bounds, web_bq.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, 400);
}

#[test]
fn web_vs_fret_typography_list_geometry_light() {
    let web = read_web_golden("typography-list");
    let theme = web.themes.get("light").expect("missing light theme");

    let mut web_lis = Vec::new();
    web_collect_tag(&theme.root, "li", &mut web_lis);
    web_lis.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_lis.len(), 3, "expected 3 web li nodes");

    let li_texts: Vec<_> = web_lis
        .iter()
        .map(|li| li.text.clone().unwrap_or_default())
        .collect();
    let li_texts_ui = li_texts.clone();

    let li_size = web_css_px(web_lis[0], "fontSize");
    let li_line_height = web_css_px(web_lis[0], "lineHeight");
    let li_weight = web_css_u16(web_lis[0], "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, _snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        li_texts_ui
            .clone()
            .into_iter()
            .map(|text| {
                ui::text(cx, text)
                    .text_size_px(li_size)
                    .line_height_px(li_line_height)
                    .font_weight(fret_core::FontWeight(li_weight))
                    .into_element(cx)
            })
            .collect::<Vec<_>>()
    });

    for text in &li_texts {
        assert_prepared_text_style(&services, text, li_size, li_line_height, li_weight);
    }
}

#[test]
fn web_vs_fret_typography_inline_code_padding_and_style_light() {
    let web = read_web_golden("typography-inline-code");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_code = find_first(&theme.root, &|n| n.tag == "code").expect("web code");

    let text = web_code.text.clone().unwrap_or_default();
    let size = web_css_px(web_code, "fontSize");
    let line_height = web_css_px(web_code, "lineHeight");
    let weight = web_css_u16(web_code, "fontWeight");
    let pt = web_css_px(web_code, "paddingTop");
    let pb = web_css_px(web_code, "paddingBottom");
    let pl = web_css_px(web_code, "paddingLeft");
    let pr = web_css_px(web_code, "paddingRight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let bg = theme.color_required("muted");

        let code_text_el = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .nowrap()
            .into_element(cx);

        let code_text = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-inline-code-text")),
                ..Default::default()
            },
            move |_cx| vec![code_text_el],
        );

        let code = cx.container(
            ContainerProps {
                padding: Edges {
                    top: pt,
                    right: pr,
                    bottom: pb,
                    left: pl,
                },
                background: Some(bg),
                corner_radii: fret_core::Corners::all(Px(4.0)),
                ..Default::default()
            },
            move |_cx| vec![code_text],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-inline-code")),
                ..Default::default()
            },
            move |_cx| vec![code],
        )]
    });

    assert_prepared_text_style(&services, &text, size, line_height, weight);

    let code = find_by_test_id(&snap, "typography-inline-code");
    let code_text = find_by_test_id(&snap, "typography-inline-code-text");

    assert_close_px(
        "inline-code padding left",
        Px(code_text.bounds.origin.x.0 - code.bounds.origin.x.0),
        pl.0,
        0.25,
    );
    assert_close_px(
        "inline-code padding top",
        Px(code_text.bounds.origin.y.0 - code.bounds.origin.y.0),
        pt.0,
        0.25,
    );

    let code_right = code.bounds.origin.x.0 + code.bounds.size.width.0;
    let text_right = code_text.bounds.origin.x.0 + code_text.bounds.size.width.0;
    assert_close_px(
        "inline-code padding right",
        Px(code_right - text_right),
        pr.0,
        0.25,
    );

    let code_bottom = code.bounds.origin.y.0 + code.bounds.size.height.0;
    let text_bottom = code_text.bounds.origin.y.0 + code_text.bounds.size.height.0;
    assert_close_px(
        "inline-code padding bottom",
        Px(code_bottom - text_bottom),
        pb.0,
        0.25,
    );
}

#[test]
fn web_vs_fret_typography_table_targeted_gate_light_contract() {
    let web = read_web_golden("typography-table");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_table = find_first(&theme.root, &|n| n.tag == "table").expect("web table");

    let mut web_trs = Vec::new();
    web_collect_tag(web_table, "tr", &mut web_trs);
    assert!(!web_trs.is_empty(), "expected at least one web tr node");
}

#[test]
fn web_vs_fret_typography_demo_targeted_gate_light_contract() {
    let web = read_web_golden("typography-demo");
    let theme = web.themes.get("light").expect("missing light theme");

    let _ = find_first(&theme.root, &|n| n.tag == "h1").expect("web h1");
    let _ = find_first(&theme.root, &|n| n.tag == "h2").expect("web h2");
    let _ = find_first(&theme.root, &|n| n.tag == "blockquote").expect("web blockquote");
    let _ = find_first(&theme.root, &|n| n.tag == "ul").expect("web ul");
}

#[test]
fn web_vs_fret_typography_demo_vp375_wraps_paragraph_light_contract() {
    let web = read_web_golden("typography-demo.vp375x900");
    let theme = web.themes.get("light").expect("missing light theme");

    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");
    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    // Sanity-check the golden: at this viewport width, the prose paragraph should wrap to multiple lines.
    assert!(
        web_p.rect.h > line_height.0 * 2.0,
        "expected web paragraph to wrap at vp375; h={} line_height={}",
        web_p.rect.h,
        line_height.0
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let text_for_render = text.clone();
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-demo-vp375-p")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(cx, text_for_render)
                        .w_full()
                        .text_size_px(size)
                        .line_height_px(line_height)
                        .font_weight(fret_core::FontWeight(weight))
                        .into_element(cx),
                ]
            },
        )]
    });

    let p = find_by_test_id(&snap, "typography-demo-vp375-p");
    assert!(
        p.bounds.size.height.0 > line_height.0 * 2.0,
        "expected fret paragraph to wrap at vp375; h={} line_height={}",
        p.bounds.size.height.0,
        line_height.0
    );

    let record = assert_prepared_text_style(&services, &text, size, line_height, weight);
    assert_eq!(record.constraints.wrap, TextWrap::Word);
    let max_w = record
        .constraints
        .max_width
        .expect("expected fret text max_width constraint for wrapped prose");
    assert!(
        max_w.0.is_finite() && max_w.0 > 0.0,
        "invalid max_width constraint: {max_w:?}"
    );
}

#[test]
fn web_vs_fret_typography_demo_vp768_wraps_paragraph_light_contract() {
    let web = read_web_golden("typography-demo.vp768x900");
    let theme = web.themes.get("light").expect("missing light theme");

    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");
    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    // Sanity-check the golden: this page still contains long prose that wraps even at tablet widths.
    assert!(
        web_p.rect.h > line_height.0 * 2.0,
        "expected web paragraph to wrap at vp768; h={} line_height={}",
        web_p.rect.h,
        line_height.0
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let text_for_render = text.clone();
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-demo-vp768-p")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(cx, text_for_render)
                        .w_full()
                        .text_size_px(size)
                        .line_height_px(line_height)
                        .font_weight(fret_core::FontWeight(weight))
                        .into_element(cx),
                ]
            },
        )]
    });

    let p = find_by_test_id(&snap, "typography-demo-vp768-p");
    assert!(
        p.bounds.size.height.0 > line_height.0 * 2.0,
        "expected fret paragraph to wrap at vp768; h={} line_height={}",
        p.bounds.size.height.0,
        line_height.0
    );

    let record = assert_prepared_text_style(&services, &text, size, line_height, weight);
    assert_eq!(record.constraints.wrap, TextWrap::Word);
    let max_w = record
        .constraints
        .max_width
        .expect("expected fret text max_width constraint for wrapped prose");
    assert!(
        max_w.0.is_finite() && max_w.0 > 0.0,
        "invalid max_width constraint: {max_w:?}"
    );
}
