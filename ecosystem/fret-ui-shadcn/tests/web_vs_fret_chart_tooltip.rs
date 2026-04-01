#![cfg(feature = "chart")]
use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_ui::element::{AnyElement, LayoutStyle, Length};
use fret_ui::tree::UiTree;
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

#[path = "support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

#[path = "support/assert.rs"]
mod test_assert;
use test_assert::assert_close_px;

#[path = "support/web_tree.rs"]
mod web_tree;
use web_tree::contains_text;

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

    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
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

fn assert_rect_close_px(label: &str, actual: Rect, expected: WebRect, tol: f32) {
    assert_close_px(&format!("{label} x"), actual.origin.x, expected.x, tol);
    assert_close_px(&format!("{label} y"), actual.origin.y, expected.y, tol);
    assert_close_px(&format!("{label} w"), actual.size.width, expected.w, tol);
    assert_close_px(&format!("{label} h"), actual.size.height, expected.h, tol);
}

fn assert_chart_tooltip_rect_matches_web(
    web_name: &str,
    indicator: shadcn::ChartTooltipIndicator,
    hide_indicator: bool,
    hide_label: bool,
    kind: shadcn::ChartTooltipContentKind,
    fixed_width_border_box: Option<Px>,
    configure: impl FnOnce(shadcn::ChartTooltipContent) -> shadcn::ChartTooltipContent,
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

    let advanced_layout = matches!(kind, shadcn::ChartTooltipContentKind::AdvancedKcalTotal)
        && web_name == "chart-tooltip-advanced";

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
        let mut tooltip = configure(
            shadcn::ChartTooltipContent::new()
                .label("Tue")
                .indicator(indicator)
                .hide_indicator(hide_indicator)
                .hide_label(hide_label)
                .kind(kind)
                .items([
                    shadcn::ChartTooltipItem::new("Running", "380"),
                    shadcn::ChartTooltipItem::new("Swimming", "420"),
                ]),
        );
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
                    layout.inset.left = Some(Px(web_tooltip.rect.x)).into();
                    layout.inset.top = Some(Px(web_tooltip.rect.y)).into();
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
        shadcn::ChartLegendVerticalAlign::Top
    } else {
        shadcn::ChartLegendVerticalAlign::Bottom
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let label = Arc::<str>::from(format!("Golden:{web_name}:legend"));

    let snap = run_fret_root(bounds, |cx| {
        let legend = shadcn::ChartLegendContent::new()
            .vertical_align(vertical_align)
            .items([
                shadcn::ChartLegendItem::new("Desktop"),
                shadcn::ChartLegendItem::new("Mobile"),
            ])
            .into_element(cx);

        let legend = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(web_legend.rect.x)).into();
                    layout.inset.top = Some(Px(web_legend.rect.y)).into();
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
        let legend = shadcn::ChartLegendContent::new()
            .gap(fret_ui_kit::Space::N2)
            .wrap(true)
            .item_width_px(Px(72.5))
            .item_justify_center(true)
            .items([
                shadcn::ChartLegendItem::new("Chrome"),
                shadcn::ChartLegendItem::new("Safari"),
                shadcn::ChartLegendItem::new("Firefox"),
                shadcn::ChartLegendItem::new("Edge"),
                shadcn::ChartLegendItem::new("Other"),
            ])
            .into_element(cx);

        let legend = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(Px(web_legend.rect.x)).into();
                    layout.inset.top = Some(Px(web_legend.rect.y)).into();
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
        shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
        shadcn::ChartTooltipContentKind::Default,
        None,
        |tooltip| tooltip,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_indicator_line_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-indicator-line",
        shadcn::ChartTooltipIndicator::Line,
        false,
        false,
        shadcn::ChartTooltipContentKind::Default,
        None,
        |tooltip| tooltip,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_indicator_none_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-indicator-none",
        shadcn::ChartTooltipIndicator::Dot,
        true,
        false,
        shadcn::ChartTooltipContentKind::Default,
        None,
        |tooltip| tooltip,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_label_none_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-label-none",
        shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
        shadcn::ChartTooltipContentKind::Default,
        None,
        |tooltip| tooltip,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_icons_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-icons",
        shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
        shadcn::ChartTooltipContentKind::Default,
        None,
        |tooltip| tooltip,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_label_custom_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-label-custom",
        shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
        shadcn::ChartTooltipContentKind::Default,
        None,
        |tooltip| tooltip,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_label_formatter_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-label-formatter",
        shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
        shadcn::ChartTooltipContentKind::Default,
        None,
        |tooltip| {
            tooltip.label("2024-07-16").label_formatter(|context| {
                let label = context.label.as_deref().unwrap_or_default();
                match label {
                    "2024-07-16" => Arc::<str>::from("July 16, 2024"),
                    _ => Arc::<str>::from(label),
                }
            })
        },
    );
}

#[test]
fn web_vs_fret_chart_tooltip_formatter_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-formatter",
        shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
        shadcn::ChartTooltipContentKind::Default,
        None,
        |tooltip| {
            tooltip.formatter(|context| {
                shadcn::ChartTooltipFormattedItem::from_item(&context.item)
                    .value_suffix("kcal")
                    .row_min_width(Px(130.0))
            })
        },
    );
}

#[test]
fn web_vs_fret_chart_tooltip_advanced_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-advanced",
        shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
        shadcn::ChartTooltipContentKind::AdvancedKcalTotal,
        Some(Px(180.0)),
        |tooltip| tooltip,
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
        shadcn::ChartTooltipIndicator::Dot,
        false,
        false,
        shadcn::ChartTooltipContentKind::Default,
        None,
        |tooltip| tooltip,
    );
}

#[test]
fn web_vs_fret_chart_tooltip_advanced_small_viewport_geometry_matches_web() {
    assert_chart_tooltip_rect_matches_web(
        "chart-tooltip-advanced.vp375x320",
        shadcn::ChartTooltipIndicator::Dot,
        false,
        true,
        shadcn::ChartTooltipContentKind::AdvancedKcalTotal,
        Some(Px(180.0)),
        |tooltip| tooltip,
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
