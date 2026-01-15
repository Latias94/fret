use fret_app::App;
use fret_core::{
    AppWindowId, Edges, Event, FrameId, Modifiers, MouseButton, Point, PointerEvent, PointerType,
    Px, Rect, SemanticsRole, Size as CoreSize,
};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length};
use fret_ui::tree::UiTree;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::OverlayController;
use serde::Deserialize;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
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
    #[serde(default)]
    open: Option<WebOpenMeta>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebPoint {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebOpenMeta {
    #[allow(dead_code)]
    action: String,
    #[allow(dead_code)]
    selector: String,
    point: WebPoint,
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
    text: Option<String>,
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

fn web_find_by_data_slot_and_state<'a>(
    root: &'a WebNode,
    slot: &str,
    state: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
            && n.attrs
                .get("data-state")
                .is_some_and(|v| v.as_str() == state)
    })
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

fn point_rect(p: WebPoint) -> WebRect {
    WebRect {
        x: p.x,
        y: p.y,
        w: 0.0,
        h: 0.0,
    }
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
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        let (text, style) = match input {
            fret_core::TextInput::Plain { text, style } => (text.as_ref(), style.clone()),
            fret_core::TextInput::Attributed { text, base, .. } => (text.as_ref(), base.clone()),
            _ => (input.text(), fret_core::TextStyle::default()),
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
    web_portal_role: Option<&str>,
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
        "context-menu-demo" => find_first(&web.themes["light"].root, &|n| {
            n.text
                .as_deref()
                .is_some_and(|t| t.contains("Right click here"))
        })
        .or_else(|| {
            find_first(&web.themes["dark"].root, &|n| {
                n.text
                    .as_deref()
                    .is_some_and(|t| t.contains("Right click here"))
            })
        })
        .expect("web trigger (context menu)"),
        _ => find_first(&web.themes["light"].root, &|n| n.tag == "button")
            .or_else(|| find_first(&web.themes["dark"].root, &|n| n.tag == "button"))
            .expect("web trigger (button)"),
    };

    let web_portal_index = if let Some(web_portal_role) = web_portal_role {
        theme
            .portals
            .iter()
            .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
            .unwrap_or_else(|| panic!("missing web portal role={web_portal_role}"))
    } else {
        if theme.portals.is_empty() {
            panic!("missing web portals for {web_name}");
        }
        0
    };
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

fn assert_centered_overlay_placement_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_portal_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
        .unwrap_or_else(|| panic!("missing web portal role={web_portal_role} for {web_name}"));
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let expected_center_x = rect_center_x(web_portal.rect);
    let expected_center_y = rect_center_y(web_portal.rect);

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

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let center = (rect_center_x(r) - expected_center_x).abs()
                    + (rect_center_y(r) - expected_center_y).abs();
                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                center + 0.02 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?} for {web_name}"));

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    if debug {
        eprintln!(
            "{web_name} web portal={:?} expected_center=({}, {})",
            web_portal.rect, expected_center_x, expected_center_y
        );
        eprintln!("{web_name} selected fret portal={:?}", fret_portal);
    }

    assert_close(
        &format!("{web_name} center_x"),
        rect_center_x(fret_portal),
        expected_center_x,
        2.0,
    );
    assert_close(
        &format!("{web_name} center_y"),
        rect_center_y(fret_portal),
        expected_center_y,
        2.0,
    );
}

fn assert_viewport_anchored_overlay_placement_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_portal_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
        .unwrap_or_else(|| panic!("missing web portal role={web_portal_role} for {web_name}"));
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let expected_left = web_portal.rect.x;
    let expected_top = web_portal.rect.y;
    let expected_right = 1440.0 - rect_right(web_portal.rect);
    let expected_bottom = 900.0 - rect_bottom(web_portal.rect);

    let anchor_tol = 2.0;
    let anchor_left = expected_left.abs() <= anchor_tol;
    let anchor_top = expected_top.abs() <= anchor_tol;
    let anchor_right = expected_right.abs() <= anchor_tol;
    let anchor_bottom = expected_bottom.abs() <= anchor_tol;

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
    for frame_id in 2..=4 {
        let request_semantics = frame_id == 4;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_id as u64),
            request_semantics,
            |cx| {
                let content = build_frame(cx, &open);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let left = r.x;
                let top = r.y;
                let right = 1440.0 - rect_right(r);
                let bottom = 900.0 - rect_bottom(r);

                let mut score = 0.0;
                if anchor_left {
                    score += (left - expected_left).abs();
                }
                if anchor_top {
                    score += (top - expected_top).abs();
                }
                if anchor_right {
                    score += (right - expected_right).abs();
                }
                if anchor_bottom {
                    score += (bottom - expected_bottom).abs();
                }

                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                score + 0.02 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?} for {web_name}"));

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_left = fret_portal.x;
    let actual_top = fret_portal.y;
    let actual_right = 1440.0 - rect_right(fret_portal);
    let actual_bottom = 900.0 - rect_bottom(fret_portal);

    if debug {
        eprintln!(
            "{web_name} anchors: left={anchor_left} top={anchor_top} right={anchor_right} bottom={anchor_bottom}"
        );
        eprintln!(
            "{web_name} web portal={:?} expected_insets=(l={expected_left}, t={expected_top}, r={expected_right}, b={expected_bottom})",
            web_portal.rect
        );
        eprintln!(
            "{web_name} fret portal={:?} actual_insets=(l={actual_left}, t={actual_top}, r={actual_right}, b={actual_bottom})",
            fret_portal
        );
    }

    if anchor_left {
        assert_close(
            &format!("{web_name} inset_left"),
            actual_left,
            expected_left,
            2.0,
        );
    }
    if anchor_top {
        assert_close(
            &format!("{web_name} inset_top"),
            actual_top,
            expected_top,
            2.0,
        );
    }
    if anchor_right {
        assert_close(
            &format!("{web_name} inset_right"),
            actual_right,
            expected_right,
            2.0,
        );
    }
    if anchor_bottom {
        assert_close(
            &format!("{web_name} inset_bottom"),
            actual_bottom,
            expected_bottom,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_popover_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "popover-demo",
        Some("dialog"),
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
        Some("menu"),
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
        Some("listbox"),
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

fn assert_point_anchored_overlay_placement_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_portal_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
    open_fret_at: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        AppWindowId,
        WebPoint,
    ),
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let open_point = theme
        .open
        .as_ref()
        .map(|m| m.point)
        .unwrap_or_else(|| panic!("missing web open point for {web_name}"));

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

    let web_trigger = point_rect(open_point);
    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger, web_portal.rect);

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

    open_fret_at(&mut ui, &mut app, &mut services, window, open_point);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    let build_settle = build.clone();
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let content = build_settle(cx, &open);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

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

    let fret_trigger = point_rect(open_point);
    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

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
fn web_vs_fret_context_menu_demo_overlay_placement_matches() {
    assert_point_anchored_overlay_placement_matches(
        "context-menu-demo",
        "menu",
        SemanticsRole::Menu,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_bookmarks: Option<Model<bool>>,
                checked_full_urls: Option<Model<bool>>,
                radio_person: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_bookmarks.as_ref(),
                    st.checked_full_urls.as_ref(),
                    st.radio_person.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_bookmarks, checked_full_urls, radio_person) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_bookmarks = cx.app.models_mut().insert(true);
                    let checked_full_urls = cx.app.models_mut().insert(false);
                    let radio_person = cx.app.models_mut().insert(Some(Arc::from("pedro")));

                    cx.with_state(Models::default, |st| {
                        st.checked_bookmarks = Some(checked_bookmarks.clone());
                        st.checked_full_urls = Some(checked_full_urls.clone());
                        st.radio_person = Some(radio_person.clone());
                    });

                    (checked_bookmarks, checked_full_urls, radio_person)
                };

            fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
                cx,
                |cx| {
                    cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(300.0));
                                layout.size.height = Length::Px(Px(150.0));
                                layout
                            },
                            ..Default::default()
                        },
                        |cx| vec![cx.text("Right click here")],
                    )
                },
                |_cx| {
                    vec![
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Back").inset(true),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Forward")
                                .inset(true)
                                .disabled(true),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Reload").inset(true),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("More Tools").inset(true).submenu(
                                vec![
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Save Page..."),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new(
                                            "Create Shortcut...",
                                        ),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Name Window..."),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Separator,
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Developer Tools"),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Separator,
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Delete").variant(
                                            fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                        ),
                                    ),
                                ],
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Separator,
                        fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                            fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                checked_bookmarks,
                                "Show Bookmarks",
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                            fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                checked_full_urls,
                                "Show Full URLs",
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Separator,
                        fret_ui_shadcn::ContextMenuEntry::Label(
                            fret_ui_shadcn::ContextMenuLabel::new("People").inset(true),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::RadioGroup(
                            fret_ui_shadcn::ContextMenuRadioGroup::new(radio_person)
                                .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                    "pedro",
                                    "Pedro Duarte",
                                ))
                                .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                    "colm",
                                    "Colm Tuite",
                                )),
                        ),
                    ]
                },
            )
        },
        |ui, app, services, _window, point| {
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Down {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Up {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
        },
    );
}

#[test]
fn web_vs_fret_tooltip_demo_overlay_placement_matches() {
    let web = read_web_golden_open("tooltip-demo");
    let theme = web_theme(&web);

    let web_trigger = find_first(&web.themes["light"].root, &|n| n.tag == "button")
        .or_else(|| find_first(&web.themes["dark"].root, &|n| n.tag == "button"))
        .expect("web trigger (button)");
    let trigger_w = web_trigger.rect.w;
    let trigger_h = web_trigger.rect.h;

    if theme.portals.is_empty() {
        panic!("missing web portals for tooltip-demo");
    }
    let web_portal_leaf = &theme.portals[0];
    let web_portal = theme.portal_wrappers.get(0).unwrap_or(web_portal_leaf);
    let content_w = web_portal.rect.w;
    let content_h = web_portal.rect.h;

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

    let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let trigger_id_out = trigger_id_out.clone();
            let content_id_out = content_id_out.clone();
            let trigger = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(trigger_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(trigger_h))),
                )
                .into_element(cx);
            trigger_id_out.set(Some(trigger.id));
            let content = fret_ui_shadcn::TooltipContent::new(vec![cx.text("Add to library")])
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(content_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(content_h))),
                )
                .into_element(cx);
            content_id_out.set(Some(content.id));
            let tooltip = fret_ui_shadcn::Tooltip::new(trigger, content).into_element(cx);
            vec![pad_root(cx, Px(0.0), tooltip)]
        },
    );

    let trigger_element = trigger_id_out.get().expect("tooltip trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("tooltip trigger node");
    ui.set_focus(Some(trigger_node));

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let trigger_id_out = trigger_id_out.clone();
                let content_id_out = content_id_out.clone();
                let trigger = fret_ui_shadcn::Button::new("Hover")
                    .variant(fret_ui_shadcn::ButtonVariant::Outline)
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(fret_ui_kit::MetricRef::Px(Px(trigger_w)))
                            .h_px(fret_ui_kit::MetricRef::Px(Px(trigger_h))),
                    )
                    .into_element(cx);
                trigger_id_out.set(Some(trigger.id));
                let content = fret_ui_shadcn::TooltipContent::new(vec![cx.text("Add to library")])
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(fret_ui_kit::MetricRef::Px(Px(content_w)))
                            .h_px(fret_ui_kit::MetricRef::Px(Px(content_h))),
                    )
                    .into_element(cx);
                content_id_out.set(Some(content.id));
                let tooltip = fret_ui_shadcn::Tooltip::new(trigger, content).into_element(cx);
                vec![pad_root(cx, Px(0.0), tooltip)]
            },
        );
    }

    let trigger_element = trigger_id_out.get().expect("tooltip trigger element id");
    let content_element = content_id_out.get().expect("tooltip content element id");

    let trigger_bounds = fret_ui::elements::bounds_for_element(&mut app, window, trigger_element)
        .expect("tooltip trigger bounds");
    let portal_bounds = fret_ui::elements::bounds_for_element(&mut app, window, content_element)
        .expect("tooltip content bounds");

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "tooltip-demo web trigger={:?} web portal={:?} fret trigger={:?} fret portal={:?}",
            web_trigger.rect, web_portal.rect, trigger_bounds, portal_bounds
        );
    }

    let fret_trigger = WebRect {
        x: trigger_bounds.origin.x.0,
        y: trigger_bounds.origin.y.0,
        w: trigger_bounds.size.width.0,
        h: trigger_bounds.size.height.0,
    };
    let fret_portal = WebRect {
        x: portal_bounds.origin.x.0,
        y: portal_bounds.origin.y.0,
        w: portal_bounds.size.width.0,
        h: portal_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close("tooltip-demo main_gap", actual_gap, expected_gap, 1.0);
    assert_close(
        "tooltip-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
}

#[test]
fn web_vs_fret_hover_card_demo_overlay_placement_matches() {
    let web = read_web_golden_open("hover-card-demo");
    let theme = web_theme(&web);

    let web_trigger = find_first(&web.themes["light"].root, &|n| n.tag == "button")
        .or_else(|| find_first(&web.themes["dark"].root, &|n| n.tag == "button"))
        .expect("web trigger (button)");
    let trigger_w = web_trigger.rect.w;
    let trigger_h = web_trigger.rect.h;

    if theme.portals.is_empty() {
        panic!("missing web portals for hover-card-demo");
    }
    let web_portal_leaf = &theme.portals[0];
    let web_portal = theme.portal_wrappers.get(0).unwrap_or(web_portal_leaf);
    let content_w = web_portal.rect.w;
    let content_h = web_portal.rect.h;

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

    let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let trigger_id_out = trigger_id_out.clone();
            let content_id_out = content_id_out.clone();
            let trigger = fret_ui_shadcn::Button::new("@nextjs")
                .variant(fret_ui_shadcn::ButtonVariant::Link)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(trigger_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(trigger_h))),
                )
                .into_element(cx);
            trigger_id_out.set(Some(trigger.id));

            let content = fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")])
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(fret_ui_kit::MetricRef::Px(Px(content_w)))
                        .h_px(fret_ui_kit::MetricRef::Px(Px(content_h))),
                )
                .into_element(cx);
            content_id_out.set(Some(content.id));

            let hover_card = fret_ui_shadcn::HoverCard::new(trigger, content)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx);

            vec![pad_root(cx, Px(0.0), hover_card)]
        },
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::KeyA,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let trigger_element = trigger_id_out.get().expect("hover card trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("hover card trigger node");
    ui.set_focus(Some(trigger_node));

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let trigger_id_out = trigger_id_out.clone();
                let content_id_out = content_id_out.clone();
                let trigger = fret_ui_shadcn::Button::new("@nextjs")
                    .variant(fret_ui_shadcn::ButtonVariant::Link)
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(fret_ui_kit::MetricRef::Px(Px(trigger_w)))
                            .h_px(fret_ui_kit::MetricRef::Px(Px(trigger_h))),
                    )
                    .into_element(cx);
                trigger_id_out.set(Some(trigger.id));

                let content = fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")])
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(fret_ui_kit::MetricRef::Px(Px(content_w)))
                            .h_px(fret_ui_kit::MetricRef::Px(Px(content_h))),
                    )
                    .into_element(cx);
                content_id_out.set(Some(content.id));

                let hover_card = fret_ui_shadcn::HoverCard::new(trigger, content)
                    .open_delay_frames(0)
                    .close_delay_frames(0)
                    .into_element(cx);

                vec![pad_root(cx, Px(0.0), hover_card)]
            },
        );
    }

    let trigger_element = trigger_id_out.get().expect("hover card trigger element id");
    let content_element = content_id_out.get().expect("hover card content element id");

    let trigger_bounds = fret_ui::elements::bounds_for_element(&mut app, window, trigger_element)
        .expect("hover card trigger bounds");
    let portal_bounds = fret_ui::elements::bounds_for_element(&mut app, window, content_element)
        .expect("hover card content bounds");

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "hover-card-demo web trigger={:?} web portal={:?} fret trigger={:?} fret portal={:?}",
            web_trigger.rect, web_portal.rect, trigger_bounds, portal_bounds
        );
    }

    let fret_trigger = WebRect {
        x: trigger_bounds.origin.x.0,
        y: trigger_bounds.origin.y.0,
        w: trigger_bounds.size.width.0,
        h: trigger_bounds.size.height.0,
    };
    let fret_portal = WebRect {
        x: portal_bounds.origin.x.0,
        y: portal_bounds.origin.y.0,
        w: portal_bounds.size.width.0,
        h: portal_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close("hover-card-demo main_gap", actual_gap, expected_gap, 1.0);
    assert_close(
        "hover-card-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_overlay_placement_matches() {
    let web = read_web_golden_open("navigation-menu-demo");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .unwrap_or_else(|| {
                find_first(&theme.root, &|n| {
                    n.attrs
                        .get("data-slot")
                        .is_some_and(|v| v.as_str() == "navigation-menu-trigger")
                })
                .expect("web trigger slot=navigation-menu-trigger")
            });
    let web_content =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-content", "open")
            .expect("web content slot=navigation-menu-content state=open");

    let web_side = infer_side(web_trigger.rect, web_content.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_content.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_content.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_content.rect);

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

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(vec![fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(vec![fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![cx.text("Content")],
                    )])
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, "home",
            )
        },
    )
    .expect("fret nav menu content id");
    let content_bounds = fret_ui::elements::bounds_for_element(&mut app, window, content_id)
        .expect("fret nav menu content bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_content = WebRect {
        x: content_bounds.origin.x.0,
        y: content_bounds.origin.y.0,
        w: content_bounds.size.width.0,
        h: content_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_content);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_content);

    assert_close(
        "navigation-menu-demo main_gap",
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        "navigation-menu-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
}

#[test]
fn web_vs_fret_menubar_demo_overlay_placement_matches() {
    let web = read_web_golden_open("menubar-demo");
    let theme = web_theme(&web);

    let web_trigger = web_find_by_data_slot_and_state(&theme.root, "menubar-trigger", "open")
        .unwrap_or_else(|| {
            find_first(&theme.root, &|n| {
                n.attrs
                    .get("data-slot")
                    .is_some_and(|v| v.as_str() == "menubar-trigger")
            })
            .expect("web trigger slot=menubar-trigger")
        });

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == "menu"))
        .expect("web portal role=menu");
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

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = fret_ui_shadcn::Menubar::new(vec![
                fret_ui_shadcn::MenubarMenu::new("File").entries(vec![
                    fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new("New Tab")),
                    fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new(
                        "New Window",
                    )),
                    fret_ui_shadcn::MenubarEntry::Separator,
                    fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new("Share")),
                ]),
            ])
            .into_element(cx);
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = fret_ui_shadcn::Menubar::new(vec![
                    fret_ui_shadcn::MenubarMenu::new("File").entries(vec![
                        fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new(
                            "New Tab",
                        )),
                        fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new(
                            "New Window",
                        )),
                        fret_ui_shadcn::MenubarEntry::Separator,
                        fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new(
                            "Share",
                        )),
                    ]),
                ])
                .into_element(cx);
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let gap = rect_main_gap(web_side, fret_trigger, r);
                let cross = rect_cross_delta(web_side, web_align, fret_trigger, r);
                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                (gap - expected_gap).abs() + (cross - expected_cross).abs() + 0.05 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("fret menubar portal semantics (Menu)");

    if debug {
        let candidates: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Menu)
            .collect();
        eprintln!("menubar-demo fret Menu candidates: {}", candidates.len());
        for (idx, n) in candidates.iter().enumerate().take(8) {
            eprintln!("  [{idx}] bounds={:?} label={:?}", n.bounds, n.label);
        }
        eprintln!(
            "menubar-demo web trigger={:?} web portal={:?}\n  fret trigger={:?}\n  selected portal={:?}",
            web_trigger.rect, web_portal.rect, fret_trigger, portal.bounds
        );
        eprintln!(
            "menubar-demo fret trigger flags={:?} root_count={} node_count={}",
            trigger.flags,
            snap.roots.len(),
            snap.nodes.len()
        );
    }

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close("menubar-demo main_gap", actual_gap, expected_gap, 1.0);
    assert_close(
        "menubar-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
}

#[test]
fn web_vs_fret_dialog_demo_overlay_center_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "dialog-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| DialogContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_alert_dialog_demo_overlay_center_matches() {
    use fret_ui_shadcn::{AlertDialog, AlertDialogContent, Button, ButtonVariant};

    assert_centered_overlay_placement_matches(
        "alert-dialog-demo",
        "alertdialog",
        SemanticsRole::AlertDialog,
        |cx, open| {
            AlertDialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Show Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    AlertDialogContent::new(vec![cx.text("Are you absolutely sure?")])
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_demo_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_top_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Top).into_element(
                cx,
                |cx| {
                    Button::new("top")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_right_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.right",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Right)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("right")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_bottom_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.bottom",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Bottom)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("bottom")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_left_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.left",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Left).into_element(
                cx,
                |cx| {
                    Button::new("left")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
