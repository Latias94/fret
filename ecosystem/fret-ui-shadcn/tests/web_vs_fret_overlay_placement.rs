use fret_app::App;
use fret_core::{AppWindowId, Edges, FrameId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length};
use fret_ui::tree::UiTree;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::OverlayController;
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
    #[allow(dead_code)]
    root: WebNode,
    #[serde(default)]
    portals: Vec<WebNode>,
    #[serde(rename = "portalWrappers", default)]
    portal_wrappers: Vec<WebNode>,
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

#[derive(Debug, Clone, Copy)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy)]
enum Align {
    Start,
    Center,
    End,
}

fn pad_root<H: UiHost>(cx: &mut ElementContext<'_, H>, pad: Px, child: AnyElement) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                layout
            },
            padding: Edges::all(pad),
            ..Default::default()
        },
        move |_cx| vec![child],
    )
}

fn first_container_px_size(element: &AnyElement) -> Option<(f32, f32)> {
    fn visit(node: &AnyElement) -> Option<(f32, f32)> {
        if let fret_ui::element::ElementKind::Container(props) = &node.kind {
            if let (Length::Px(w), Length::Px(h)) =
                (props.layout.size.width, props.layout.size.height)
            {
                return Some((w.0, h.0));
            }
        }
        for child in &node.children {
            if let Some(found) = visit(child) {
                return Some(found);
            }
        }
        None
    }
    visit(element)
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn web_golden_open_path(name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(format!("{name}.open.json"))
}

fn read_web_golden_open(name: &str) -> WebGolden {
    let path = web_golden_open_path(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web open golden: {}\nerror: {err}\n\nGenerate it via:\n  pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts {name} --modes=open --update --baseUrl=http://localhost:4020\n\nDocs:\n  docs/shadcn-web-goldens.md",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "failed to parse web open golden: {}\nerror: {err}",
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

fn find_attr_in_subtree<'a>(node: &'a WebNode, key: &str) -> Option<&'a str> {
    node.attrs.get(key).map(String::as_str).or_else(|| {
        for child in &node.children {
            if let Some(found) = find_attr_in_subtree(child, key) {
                return Some(found);
            }
        }
        None
    })
}

fn parse_side(value: &str) -> Option<Side> {
    Some(match value {
        "top" => Side::Top,
        "right" => Side::Right,
        "bottom" => Side::Bottom,
        "left" => Side::Left,
        _ => return None,
    })
}

fn parse_align(value: &str) -> Option<Align> {
    Some(match value {
        "start" => Align::Start,
        "center" => Align::Center,
        "end" => Align::End,
        _ => return None,
    })
}

fn rect_right(r: WebRect) -> f32 {
    r.x + r.w
}

fn rect_bottom(r: WebRect) -> f32 {
    r.y + r.h
}

fn rect_center_x(r: WebRect) -> f32 {
    r.x + r.w * 0.5
}

fn rect_center_y(r: WebRect) -> f32 {
    r.y + r.h * 0.5
}

fn rect_main_gap(side: Side, trigger: WebRect, content: WebRect) -> f32 {
    match side {
        Side::Bottom => content.y - rect_bottom(trigger),
        Side::Top => trigger.y - rect_bottom(content),
        Side::Right => content.x - rect_right(trigger),
        Side::Left => trigger.x - rect_right(content),
    }
}

fn rect_cross_delta(side: Side, align: Align, trigger: WebRect, content: WebRect) -> f32 {
    match side {
        Side::Top | Side::Bottom => match align {
            Align::Start => content.x - trigger.x,
            Align::Center => rect_center_x(content) - rect_center_x(trigger),
            Align::End => rect_right(content) - rect_right(trigger),
        },
        Side::Left | Side::Right => match align {
            Align::Start => content.y - trigger.y,
            Align::Center => rect_center_y(content) - rect_center_y(trigger),
            Align::End => rect_bottom(content) - rect_bottom(trigger),
        },
    }
}

fn infer_side(trigger: WebRect, content: WebRect) -> Side {
    let candidates = [
        (Side::Bottom, rect_main_gap(Side::Bottom, trigger, content)),
        (Side::Top, rect_main_gap(Side::Top, trigger, content)),
        (Side::Right, rect_main_gap(Side::Right, trigger, content)),
        (Side::Left, rect_main_gap(Side::Left, trigger, content)),
    ];
    candidates
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(side, _)| side)
        .unwrap_or(Side::Bottom)
}

fn infer_align(side: Side, trigger: WebRect, content: WebRect) -> Align {
    let candidates = [
        (
            Align::Start,
            rect_cross_delta(side, Align::Start, trigger, content).abs(),
        ),
        (
            Align::Center,
            rect_cross_delta(side, Align::Center, trigger, content).abs(),
        ),
        (
            Align::End,
            rect_cross_delta(side, Align::End, trigger, content).abs(),
        ),
    ];
    candidates
        .into_iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(align, _)| align)
        .unwrap_or(Align::Start)
}

fn assert_close(label: &str, actual: f32, expected: f32, tol: f32) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected＞{expected} (㊣{tol}) got={actual} (忖={delta})"
    );
}

#[derive(Default)]
struct StyleAwareServices;

impl fret_core::TextService for StyleAwareServices {
    fn prepare(
        &mut self,
        text: &str,
        style: &fret_core::TextStyle,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
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

fn setup_app_with_shadcn_theme(app: &mut App) {
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id: FrameId,
    request_semantics: bool,
    render: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) {
    app.set_frame_id(frame_id);
    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "web-vs-fret-overlay-placement",
        render,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn assert_overlay_placement_matches(
    web_name: &str,
    web_portal_role: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
    fret_trigger_role: SemanticsRole,
    fret_trigger_label: Option<&str>,
    fret_portal_role: SemanticsRole,
) {
    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger = match web_name {
        "select-scrollable" => find_first(&web.themes["light"].root, &|n| {
            n.attrs.get("role").is_some_and(|v| v == "combobox")
        })
        .or_else(|| {
            find_first(&web.themes["dark"].root, &|n| {
                n.attrs.get("role").is_some_and(|v| v == "combobox")
            })
        })
        .expect("web trigger (combobox)"),
        _ => find_first(&web.themes["light"].root, &|n| n.tag == "button")
            .or_else(|| find_first(&web.themes["dark"].root, &|n| n.tag == "button"))
            .expect("web trigger (button)"),
    };

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
        .unwrap_or_else(|| panic!("missing web portal role={web_portal_role}"));
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1440.0), Px(900.0)),
    );

    let open: Model<bool> = app.models_mut().insert(false);
    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let content = build_frame1(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let build_frame2 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        false,
        |cx| {
            let content = build_frame2(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    let build_frame3 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        false,
        |cx| {
            let content = build_frame3(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    // Some Radix-ish overlays (notably Select item-aligned) need a fully-mounted frame before
    // placement can converge (we rely on last-frame bounds for item alignment inputs).
    let build_frame4 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| {
            let content = build_frame4(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let trigger = snap
        .nodes
        .iter()
        .find(|n| {
            if n.role != fret_trigger_role {
                return false;
            }
            if let Some(label) = fret_trigger_label {
                return n.label.as_deref() == Some(label);
            }
            true
        })
        .unwrap_or_else(|| panic!("missing fret trigger role={fret_trigger_role:?}"));

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let ah = a.bounds.size.height.0;
            let bw = b.bounds.size.width.0;
            let bh = b.bounds.size.height.0;

            let score_a = (aw - expected_portal_w).abs() + (ah - expected_portal_h).abs();
            let score_b = (bw - expected_portal_w).abs() + (bh - expected_portal_h).abs();
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?}"));

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    if debug {
        let candidates: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == fret_portal_role)
            .collect();
        eprintln!(
            "{web_name} fret portal candidates role={fret_portal_role:?}: {}",
            candidates.len()
        );
        for (idx, n) in candidates.iter().enumerate().take(6) {
            eprintln!(
                "  [{idx}] bounds={:?} label={:?} flags={:?}",
                n.bounds, n.label, n.flags
            );
        }
        eprintln!(
            "{web_name} web side={web_side:?} align={web_align:?}\n  web trigger={:?}\n  web portal={:?}\n  fret trigger={:?}\n  fret portal={:?}",
            web_trigger.rect, web_portal.rect, fret_trigger, fret_portal
        );
    }

    assert_close(
        &format!("{web_name} main_gap"),
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        &format!("{web_name} cross_delta"),
        actual_cross,
        expected_cross,
        1.5,
    );
}

#[test]
fn web_vs_fret_popover_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "popover-demo",
        "dialog",
        |cx, open| {
            fret_ui_shadcn::Popover::new(open.clone()).into_element(
                cx,
                |cx| {
                    fret_ui_shadcn::Button::new("Open popover")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    let content = fret_ui_shadcn::PopoverContent::new(Vec::new())
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(fret_ui_kit::MetricRef::Px(Px(320.0)))
                                .h_px(fret_ui_kit::MetricRef::Px(Px(245.33334))),
                        )
                        .into_element(cx);
                    if std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
                        .ok()
                        .is_some_and(|v| v == "1")
                    {
                        eprintln!(
                            "popover-demo content container px size={:?}",
                            first_container_px_size(&content)
                        );
                    }
                    content
                },
            )
        },
        SemanticsRole::Button,
        Some("Open popover"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "dropdown-menu-demo",
        "menu",
        |cx, open| {
            fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
                cx,
                |cx| fret_ui_shadcn::Button::new("Open").into_element(cx),
                |_cx| {
                    vec![fret_ui_shadcn::DropdownMenuEntry::Item(
                        fret_ui_shadcn::DropdownMenuItem::new("Alpha"),
                    )]
                },
            )
        },
        SemanticsRole::Button,
        Some("Open"),
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_select_scrollable_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-scrollable",
        "listbox",
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("North America").into(),
                    SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Europe & Africa").into(),
                    SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    SelectItem::new("cet", "Central European Time (CET)").into(),
                    SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Asia").into(),
                    SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    SelectItem::new("ist", "India Standard Time (IST)").into(),
                    SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)")
                        .into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Australia & Pacific").into(),
                    SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                    SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                    SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                    SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("South America").into(),
                    SelectItem::new("art", "Argentina Time (ART)").into(),
                    SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a timezone")
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(280.0))),
                )
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}
