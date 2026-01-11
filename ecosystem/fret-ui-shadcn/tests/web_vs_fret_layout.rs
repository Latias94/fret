use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
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
    #[serde(rename = "className")]
    class_name: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    attrs: BTreeMap<String, String>,
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

fn contains_text(node: &WebNode, needle: &str) -> bool {
    if node.text.as_deref().is_some_and(|t| t.contains(needle)) {
        return true;
    }
    node.children.iter().any(|c| contains_text(c, needle))
}

fn web_find_by_tag_and_text<'a>(root: &'a WebNode, tag: &str, text: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| n.tag == tag && contains_text(n, text))
}

fn web_find_by_class_contains<'a>(root: &'a WebNode, needle: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.class_name.as_deref().is_some_and(|c| c.contains(needle))
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

#[derive(Default)]
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

fn run_fret_root(
    bounds: Rect,
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
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
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

#[test]
fn web_vs_fret_layout_button_default_height() {
    let web = read_web_golden("button-default");
    let theme = web_theme(&web);
    let web_button = web_find_by_tag_and_text(&theme.root, "button", "Button").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![fret_ui_shadcn::Button::new("Button").into_element(cx)]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Button"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button semantics node");

    assert_close_px(
        "button height",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_tabs_demo_tab_list_height() {
    let web = read_web_golden("tabs-demo");
    let theme = web_theme(&web);
    let web_tab_list = web_find_by_class_contains(
        &theme.root,
        "bg-muted text-muted-foreground inline-flex h-9 w-fit",
    )
    .expect("web tab list");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("account", "Account", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("password", "Password", vec![cx.text("Panel")]),
        ];

        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("account"))
                .items(items)
                .into_element(cx),
        ]
    });

    let tab_list = find_semantics(&snap, SemanticsRole::TabList, None).expect("fret tab list");
    assert_close_px(
        "tab list height",
        tab_list.bounds.size.height,
        web_tab_list.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_scroll_area_demo_root_size() {
    let web = read_web_golden("scroll-area-demo");
    let theme = web_theme(&web);
    let web_root = web_find_by_class_contains(&theme.root, "relative h-72 w-48 rounded-md border")
        .expect("web scroll area root");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let items: Vec<_> = (1..=50).map(|i| cx.text(format!("Item {i}"))).collect();

        let scroll_area = fret_ui_shadcn::ScrollArea::new(items)
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.w)))
                    .h_px(fret_ui_kit::MetricRef::Px(Px(web_root.rect.h))),
            )
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:scroll-area-demo:root")),
                ..Default::default()
            },
            move |_cx| vec![scroll_area],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:scroll-area-demo:root"),
    )
    .expect("fret scroll area root");

    assert_close_px(
        "scroll area root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "scroll area root height",
        root.bounds.size.height,
        web_root.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_select_scrollable_trigger_size() {
    let web = read_web_golden("select-scrollable");
    let theme = web_theme(&web);
    let web_trigger =
        web_find_by_class_contains(&theme.root, "w-[280px]").expect("web select trigger");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-select",
        |cx| {
            vec![
                fret_ui_shadcn::Select::new(value.clone(), open.clone())
                    .items([
                        fret_ui_shadcn::SelectItem::new("alpha", "Alpha"),
                        fret_ui_shadcn::SelectItem::new("beta", "Beta"),
                        fret_ui_shadcn::SelectItem::new("gamma", "Gamma"),
                    ])
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(fret_ui_kit::MetricRef::Px(Px(web_trigger.rect.w))),
                    )
                    .into_element(cx),
            ]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let combobox = find_semantics(&snap, SemanticsRole::ComboBox, None)
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret select trigger node");

    assert_close_px(
        "select trigger width",
        combobox.bounds.size.width,
        web_trigger.rect.w,
        1.0,
    );
    assert_close_px(
        "select trigger height",
        combobox.bounds.size.height,
        web_trigger.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_input_group_dropdown_height() {
    let web = read_web_golden("input-group-dropdown");
    let theme = web_theme(&web);
    let web_group = web_find_by_class_contains(&theme.root, "group/input-group border-input")
        .expect("web input group root");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let model: Model<String> = app.models_mut().insert(String::new());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout-input-group",
        |cx| {
            let container_layout = fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(384.0)));
            let container = cx.container(
                fret_ui::element::ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &fret_ui::Theme::global(&*cx.app),
                        container_layout,
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let group = fret_ui_shadcn::InputGroup::new(model.clone())
                        .a11y_label("Input group")
                        .into_element(cx);

                    vec![cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:input-group-dropdown:root")),
                            ..Default::default()
                        },
                        move |_cx| vec![group],
                    )]
                },
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:input-group-dropdown:root"),
    )
    .expect("fret input group root");

    assert_close_px(
        "input group height",
        group.bounds.size.height,
        web_group.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_card_with_form_width() {
    let web = read_web_golden("card-with-form");
    let theme = web_theme(&web);
    let web_card = web_find_by_class_contains(
        &theme.root,
        "bg-card text-card-foreground flex flex-col gap-6 rounded-xl border py-6 shadow-sm w-[350px]",
    )
    .expect("web card root");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let card = fret_ui_shadcn::Card::new(vec![
            fret_ui_shadcn::CardHeader::new(vec![
                fret_ui_shadcn::CardTitle::new("Title").into_element(cx),
                fret_ui_shadcn::CardDescription::new("Description").into_element(cx),
            ])
            .into_element(cx),
            fret_ui_shadcn::CardContent::new(vec![cx.text("Content")]).into_element(cx),
        ])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_px(fret_ui_kit::MetricRef::Px(Px(web_card.rect.w))),
        )
        .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:card-with-form:root")),
                ..Default::default()
            },
            move |_cx| vec![card],
        )]
    });

    let card = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:card-with-form:root"),
    )
    .expect("fret card root");

    assert_close_px("card width", card.bounds.size.width, web_card.rect.w, 1.0);
}
