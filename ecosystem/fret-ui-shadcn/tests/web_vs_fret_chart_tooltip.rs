use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_ui::element::{AnyElement, LayoutStyle, Length};
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
    golden.themes.get("light").expect("missing light theme")
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
        .is_some_and(|class| class.split_whitespace().any(|t| t == token))
}

fn contains_text(node: &WebNode, needle: &str) -> bool {
    if node.text.as_deref().is_some_and(|t| t.contains(needle)) {
        return true;
    }
    node.children.iter().any(|c| contains_text(c, needle))
}

#[derive(Default)]
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
        "web-vs-fret-chart-tooltip",
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

fn assert_close_px(label: &str, a: Px, b: f32, tol: f32) {
    let delta = (a.0 - b).abs();
    assert!(
        delta <= tol,
        "{label}: expected ~{b:.3}px, got {:.3}px (Δ={delta:.3}px, tol={tol:.3}px)",
        a.0
    );
}

fn assert_rect_close_px(label: &str, actual: Rect, expected: WebRect, tol: f32) {
    assert_close_px(&format!("{label} x"), actual.origin.x, expected.x, tol);
    assert_close_px(&format!("{label} y"), actual.origin.y, expected.y, tol);
    assert_close_px(&format!("{label} w"), actual.size.width, expected.w, tol);
    assert_close_px(&format!("{label} h"), actual.size.height, expected.h, tol);
}

fn assert_chart_tooltip_rect_matches_web(
    web_name: &str,
    indicator: fret_ui_shadcn::ChartTooltipIndicator,
    hide_indicator: bool,
    hide_label: bool,
    kind: fret_ui_shadcn::ChartTooltipContentKind,
    fixed_width_border_box: Option<Px>,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_tooltip = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "border-border/50")
            && class_has_token(n, "bg-background")
            && class_has_token(n, "shadow-xl")
            && class_has_token(n, "min-w-[8rem]")
    })
    .expect("web chart tooltip node");

    let advanced_layout = matches!(
        kind,
        fret_ui_shadcn::ChartTooltipContentKind::AdvancedKcalTotal
    ) && web_name == "chart-tooltip-advanced";

    let (web_item_0, web_item_1, web_total_row) = if advanced_layout {
        let item_row = |name: &str| {
            find_first(web_tooltip, &|n| {
                n.tag == "div"
                    && class_has_token(n, "flex")
                    && class_has_token(n, "w-full")
                    && class_has_token(n, "items-center")
                    && class_has_token(n, "gap-2")
                    && contains_text(n, name)
            })
            .unwrap_or_else(|| panic!("web chart tooltip item row for {name}"))
        };

        let total_row = find_first(web_tooltip, &|n| {
            n.tag == "div"
                && class_has_token(n, "flex")
                && class_has_token(n, "basis-full")
                && class_has_token(n, "border-t")
                && class_has_token(n, "mt-1.5")
                && class_has_token(n, "pt-1.5")
                && contains_text(n, "Total")
        })
        .expect("web chart tooltip total row");

        (
            Some(item_row("Running")),
            Some(item_row("Swimming")),
            Some(total_row),
        )
    } else {
        (None, None, None)
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = Arc::<str>::from(format!("Golden:{web_name}:tooltip"));

    let snap = run_fret_root(bounds, |cx| {
        let mut tooltip = fret_ui_shadcn::ChartTooltipContent::new()
            .label("Tue")
            .indicator(indicator)
            .hide_indicator(hide_indicator)
            .hide_label(hide_label)
            .kind(kind)
            .items([
                fret_ui_shadcn::ChartTooltipItem::new("Running", "380"),
                fret_ui_shadcn::ChartTooltipItem::new("Swimming", "420"),
            ]);
        if advanced_layout {
            tooltip = tooltip.test_id_prefix(label.clone());
        }
        if let Some(width) = fixed_width_border_box {
            tooltip = tooltip.fixed_width_border_box(width);
        }

        let tooltip = tooltip.into_element(cx);
        let tooltip = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(web_tooltip.rect.x));
                    layout.inset.top = Some(Px(web_tooltip.rect.y));
                    layout.size.width = Length::Px(Px(web_tooltip.rect.w));
                    layout
                },
                role: SemanticsRole::Panel,
                label: Some(label.clone()),
                ..Default::default()
            },
            move |_cx| vec![tooltip],
        );
        vec![tooltip]
    });

    let tooltip = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
        .unwrap_or_else(|| panic!("missing fret chart tooltip semantics for {web_name}"));
    assert_rect_close_px(web_name, tooltip.bounds, web_tooltip.rect, 1.0);

    if advanced_layout {
        let item_0_label = format!("{label}:item-0");
        let item_1_label = format!("{label}:item-1");
        let total_row_label = format!("{label}:total-row");

        let item_0 = find_semantics(&snap, SemanticsRole::Panel, Some(&item_0_label))
            .expect("missing fret chart tooltip item-0 semantics");
        let item_1 = find_semantics(&snap, SemanticsRole::Panel, Some(&item_1_label))
            .expect("missing fret chart tooltip item-1 semantics");
        let total_row = find_semantics(&snap, SemanticsRole::Panel, Some(&total_row_label))
            .expect("missing fret chart tooltip total-row semantics");

        assert_rect_close_px(
            "chart-tooltip-advanced item-0",
            item_0.bounds,
            web_item_0.expect("web item-0").rect,
            1.0,
        );
        assert_rect_close_px(
            "chart-tooltip-advanced item-1",
            item_1.bounds,
            web_item_1.expect("web item-1").rect,
            1.0,
        );
        assert_rect_close_px(
            "chart-tooltip-advanced total-row",
            total_row.bounds,
            web_total_row.expect("web total-row").rect,
            1.0,
        );
    }
}

fn assert_chart_legend_rect_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_legend = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "flex")
            && class_has_token(n, "items-center")
            && class_has_token(n, "justify-center")
            && class_has_token(n, "gap-4")
            && (class_has_token(n, "pt-3") || class_has_token(n, "pb-3"))
    })
    .expect("web chart legend node");

    let vertical_align = if class_has_token(web_legend, "pb-3") {
        fret_ui_shadcn::ChartLegendVerticalAlign::Top
    } else {
        fret_ui_shadcn::ChartLegendVerticalAlign::Bottom
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = Arc::<str>::from(format!("Golden:{web_name}:legend"));

    let snap = run_fret_root(bounds, |cx| {
        let legend = fret_ui_shadcn::ChartLegendContent::new()
            .vertical_align(vertical_align)
            .items([
                fret_ui_shadcn::ChartLegendItem::new("Desktop"),
                fret_ui_shadcn::ChartLegendItem::new("Mobile"),
            ])
            .into_element(cx);

        let legend = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(web_legend.rect.x));
                    layout.inset.top = Some(Px(web_legend.rect.y));
                    layout.size.width = Length::Px(Px(web_legend.rect.w));
                    layout
                },
                role: SemanticsRole::Panel,
                label: Some(label.clone()),
                ..Default::default()
            },
            move |_cx| vec![legend],
        );

        vec![legend]
    });

    let legend = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
        .unwrap_or_else(|| panic!("missing fret chart legend semantics for {web_name}"));

    assert_rect_close_px(web_name, legend.bounds, web_legend.rect, 1.0);
}

fn assert_chart_pie_legend_rect_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_legend = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "recharts-legend-wrapper")
    })
    .and_then(|wrapper| {
        find_first(wrapper, &|n| {
            n.tag == "div"
                && class_has_token(n, "flex")
                && class_has_token(n, "items-center")
                && class_has_token(n, "justify-center")
                && class_has_token(n, "pt-3")
                && class_has_token(n, "-translate-y-2")
                && class_has_token(n, "flex-wrap")
                && class_has_token(n, "gap-2")
        })
    })
    .expect("web chart pie legend node");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = Arc::<str>::from(format!("Golden:{web_name}:pie-legend"));

    let snap = run_fret_root(bounds, |cx| {
        let legend = fret_ui_shadcn::ChartLegendContent::new()
            .gap(fret_ui_kit::Space::N2)
            .wrap(true)
            .item_width_px(Px(72.5))
            .item_justify_center(true)
            .items([
                fret_ui_shadcn::ChartLegendItem::new("Chrome"),
                fret_ui_shadcn::ChartLegendItem::new("Safari"),
                fret_ui_shadcn::ChartLegendItem::new("Firefox"),
                fret_ui_shadcn::ChartLegendItem::new("Edge"),
                fret_ui_shadcn::ChartLegendItem::new("Other"),
            ])
            .into_element(cx);

        let legend = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(web_legend.rect.x));
                    layout.inset.top = Some(Px(web_legend.rect.y));
                    layout.size.width = Length::Px(Px(web_legend.rect.w));
                    layout
                },
                role: SemanticsRole::Panel,
                label: Some(label.clone()),
                ..Default::default()
            },
            move |_cx| vec![legend],
        );

        vec![legend]
    });

    let legend = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
        .unwrap_or_else(|| panic!("missing fret chart pie legend semantics for {web_name}"));

    assert_rect_close_px(web_name, legend.bounds, web_legend.rect, 1.0);
}

#[test]
fn web_vs_fret_chart_tooltip_default_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-default",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
        fret_ui_shadcn::ChartTooltipContentKind::Default,
        None,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_indicator_line_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-indicator-line",
        fret_ui_shadcn::ChartTooltipIndicator::Line,
        false,
        false,
        fret_ui_shadcn::ChartTooltipContentKind::Default,
        None,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_indicator_none_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-indicator-none",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        true,
        false,
        fret_ui_shadcn::ChartTooltipContentKind::Default,
        None,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_label_none_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-label-none",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
        fret_ui_shadcn::ChartTooltipContentKind::Default,
        None,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_icons_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-icons",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
        fret_ui_shadcn::ChartTooltipContentKind::Default,
        None,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_label_custom_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-label-custom",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
        fret_ui_shadcn::ChartTooltipContentKind::Default,
        None,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_label_formatter_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-label-formatter",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
        fret_ui_shadcn::ChartTooltipContentKind::Default,
        None,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_formatter_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-formatter",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
        fret_ui_shadcn::ChartTooltipContentKind::FormatterKcal,
        None,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_advanced_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-advanced",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
        fret_ui_shadcn::ChartTooltipContentKind::AdvancedKcalTotal,
        Some(Px(180.0)),
    );
}

#[test]
fn web_vs_fret_chart_area_legend_geometry_matches_web() {
    assert_chart_legend_rect_matches_web("chart-area-legend");
}

#[test]
fn web_vs_fret_chart_bar_demo_legend_geometry_matches_web() {
    assert_chart_legend_rect_matches_web("chart-bar-demo-legend");
}

#[test]
fn web_vs_fret_chart_radar_legend_geometry_matches_web() {
    assert_chart_legend_rect_matches_web("chart-radar-legend");
}

#[test]
fn web_vs_fret_chart_pie_legend_geometry_matches_web() {
    assert_chart_pie_legend_rect_matches_web("chart-pie-legend");
}

#[test]
fn web_vs_fret_chart_tooltip_default_small_viewport_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-default.vp375x320",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
        fret_ui_shadcn::ChartTooltipContentKind::Default,
        None,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_advanced_small_viewport_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-advanced.vp375x320",
        fret_ui_shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
        fret_ui_shadcn::ChartTooltipContentKind::AdvancedKcalTotal,
        Some(Px(180.0)),
    );
}

#[test]
fn web_vs_fret_chart_area_legend_small_viewport_geometry_matches_web() {
    assert_chart_legend_rect_matches_web("chart-area-legend.vp375x320");
}

#[test]
fn web_vs_fret_chart_bar_demo_legend_small_viewport_geometry_matches_web() {
    assert_chart_legend_rect_matches_web("chart-bar-demo-legend.vp375x320");
}

#[test]
fn web_vs_fret_chart_radar_legend_small_viewport_geometry_matches_web() {
    assert_chart_legend_rect_matches_web("chart-radar-legend.vp375x320");
}

#[test]
fn web_vs_fret_chart_pie_legend_small_viewport_geometry_matches_web() {
    assert_chart_pie_legend_rect_matches_web("chart-pie-legend.vp375x320");
}
