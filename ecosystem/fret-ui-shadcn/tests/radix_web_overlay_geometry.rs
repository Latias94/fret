use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, MouseButtons, NodeId, Point, PointerEvent,
    PointerType, Px, Rect, SemanticsRole, Size as CoreSize,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, LayoutStyle, Length, PositionStyle, PressableA11y,
    PressableProps, SemanticsProps,
};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::prelude::LayoutRefinement;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
struct TimelineGolden {
    version: u32,
    base: String,
    theme: String,
    item: String,
    primitive: String,
    scenario: String,
    steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize)]
struct Step {
    snapshot: Snapshot,
}

#[derive(Debug, Clone, Deserialize)]
struct Snapshot {
    dom: DomNode,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct DomRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct DomNode {
    tag: String,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    #[serde(default)]
    rect: Option<DomRect>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    children: Vec<DomNode>,
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn radix_web_path(file_stem: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("radix-web")
        .join("v4")
        .join("radix-vega")
        .join(format!("{file_stem}.json"))
}

fn read_timeline(file_stem: &str) -> TimelineGolden {
    let path = radix_web_path(file_stem);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing radix web golden: {}\nerror: {err}\n\nRe-generate it via (PowerShell):\n  pnpm -C repo-ref/ui/apps/v4 build\n  $env:NEXT_PUBLIC_APP_URL='http://localhost:4020'; pnpm -C repo-ref/ui/apps/v4 exec next start -p 4020\n  pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/radix-web/scripts/extract-behavior.mts --all --update --baseUrl=http://localhost:4020\n\nDocs:\n  goldens/radix-web/README.md",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "failed to parse radix web golden: {}\nerror: {err}",
            path.display()
        )
    })
}

fn find_first<'a>(node: &'a DomNode, pred: &impl Fn(&'a DomNode) -> bool) -> Option<&'a DomNode> {
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

fn require_rect(node: &DomNode, label: &str) -> DomRect {
    node.rect
        .unwrap_or_else(|| panic!("missing rect for {label} (tag={})", node.tag))
}

fn rect_right(r: DomRect) -> f32 {
    r.x + r.w
}

fn rect_bottom(r: DomRect) -> f32 {
    r.y + r.h
}

fn rect_center_x(r: DomRect) -> f32 {
    r.x + r.w * 0.5
}

fn rect_center_y(r: DomRect) -> f32 {
    r.y + r.h * 0.5
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

fn parse_side(value: &str) -> Side {
    match value {
        "top" => Side::Top,
        "right" => Side::Right,
        "bottom" => Side::Bottom,
        "left" => Side::Left,
        other => panic!("unsupported data-side: {other}"),
    }
}

fn parse_align(value: &str) -> Align {
    match value {
        "start" => Align::Start,
        "center" => Align::Center,
        "end" => Align::End,
        other => panic!("unsupported data-align: {other}"),
    }
}

fn rect_main_gap(side: Side, trigger: DomRect, content: DomRect) -> f32 {
    match side {
        Side::Bottom => content.y - rect_bottom(trigger),
        Side::Top => trigger.y - rect_bottom(content),
        Side::Right => content.x - rect_right(trigger),
        Side::Left => trigger.x - rect_right(content),
    }
}

fn rect_cross_delta(side: Side, align: Align, trigger: DomRect, content: DomRect) -> f32 {
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

fn assert_close(label: &str, actual: f32, expected: f32, tol: f32) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected≈{expected} (±{tol}) got={actual} (Δ={delta})"
    );
}

fn assert_rect_close(label: &str, actual: DomRect, expected: DomRect, tol: f32) {
    assert_close(&format!("{label}.x"), actual.x, expected.x, tol);
    assert_close(&format!("{label}.y"), actual.y, expected.y, tol);
    assert_close(&format!("{label}.w"), actual.w, expected.w, tol);
    assert_close(&format!("{label}.h"), actual.h, expected.h, tol);
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
        let mut advance_em: f32 = 0.0;
        let mut char_count: usize = 0;
        for ch in text.chars() {
            char_count += 1;
            advance_em += match ch {
                ' ' | '\t' => 0.25,
                'i' | 'l' | 'I' => 0.30,
                'm' | 'w' | 'M' | 'W' => 0.80,
                '0'..='9' => 0.52,
                'A'..='Z' => 0.60,
                _ => 0.46,
            };
        }

        let font_size = style.size.0;
        let mut width = Px(font_size * advance_em);
        if let Some(tracking_em) = style.letter_spacing_em {
            width.0 += font_size * tracking_em * (char_count.saturating_sub(1) as f32);
        }
        if let Some(max_width) = constraints.max_width {
            width.0 = width.0.min(max_width.0);
        }

        let line_height = style.line_height.unwrap_or(Px(font_size * 1.2));
        (
            fret_core::TextBlobId::default(),
            fret_core::TextMetrics {
                size: CoreSize::new(width, line_height),
                baseline: Px(line_height.0 * 0.8),
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
        "radix-web-overlay-geometry",
        render,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
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

fn dump_semantics(snap: &fret_core::SemanticsSnapshot) {
    eprintln!("-- semantics nodes: {}", snap.nodes.len());
    for n in &snap.nodes {
        if let Some(label) = n.label.as_deref() {
            eprintln!("- {:?} label={label}", n.role);
        } else {
            eprintln!("- {:?}", n.role);
        }
    }
}

fn fret_rect_to_dom(rect: Rect) -> DomRect {
    DomRect {
        x: rect.origin.x.0,
        y: rect.origin.y.0,
        w: rect.size.width.0,
        h: rect.size.height.0,
    }
}

fn find_best_overlay_panel_rect_in_subtree(
    ui: &UiTree<App>,
    root: NodeId,
    window_bounds: Rect,
) -> Option<Rect> {
    let mut stack = vec![root];
    let mut best: Option<(f32, Rect)> = None;

    while let Some(node) = stack.pop() {
        for child in ui.children(node).into_iter().rev() {
            stack.push(child);
        }

        let bounds = ui
            .debug_node_visual_bounds(node)
            .or_else(|| ui.debug_node_bounds(node))?;

        let is_fullscreen =
            bounds.origin == window_bounds.origin && bounds.size == window_bounds.size;
        if is_fullscreen {
            continue;
        }

        let w = bounds.size.width.0;
        let h = bounds.size.height.0;
        if w < 100.0 || h < 80.0 {
            continue;
        }

        if w > window_bounds.size.width.0 - 1.0 || h > window_bounds.size.height.0 - 1.0 {
            continue;
        }

        let area = w * h;
        if best.as_ref().is_none_or(|(best_area, _)| area > *best_area) {
            best = Some((area, bounds));
        }
    }

    best.map(|(_, rect)| rect)
}

fn find_overlay_panel_rect(ui: &UiTree<App>, window_bounds: Rect) -> Rect {
    let layers = ui.debug_layers_in_paint_order();

    for layer in layers.iter().rev() {
        if !layer.visible || !layer.hit_testable || !layer.blocks_underlay_input {
            continue;
        }

        if let Some(found) = find_best_overlay_panel_rect_in_subtree(ui, layer.root, window_bounds)
        {
            return found;
        }
    }

    for layer in layers.iter().rev() {
        if !layer.visible || !layer.hit_testable {
            continue;
        }

        if let Some(found) = find_best_overlay_panel_rect_in_subtree(ui, layer.root, window_bounds)
        {
            return found;
        }
    }

    panic!("failed to locate overlay panel rect in debug layers");
}

fn fixed_size_container(cx: &mut ElementContext<'_, App>, w: f32, h: f32) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(w));
                layout.size.height = Length::Px(Px(h));
                layout
            },
            ..Default::default()
        },
        |_cx| Vec::new(),
    )
}

fn spacer(cx: &mut ElementContext<'_, App>, w: Length, h: Length) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = w;
                layout.size.height = h;
                layout
            },
            ..Default::default()
        },
        |_cx| Vec::new(),
    )
}

fn render_tooltip_fixture(
    cx: &mut ElementContext<'_, App>,
    web_trigger_rect: DomRect,
    web_content_rect: DomRect,
    side: Side,
    align: Align,
    trigger_id_out: std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>>,
) -> Vec<AnyElement> {
    let mut root_layout = LayoutStyle::default();
    root_layout.size.width = Length::Fill;
    root_layout.size.height = Length::Fill;
    root_layout.position = PositionStyle::Relative;

    vec![cx.container(
        ContainerProps {
            layout: root_layout,
            ..Default::default()
        },
        |cx| {
            fret_ui_shadcn::TooltipProvider::new()
                .delay_duration_frames(0)
                .skip_delay_duration_frames(0)
                .with_elements(cx, |cx| {
                    let trigger = cx.pressable(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(web_trigger_rect.w));
                                layout.size.height = Length::Px(Px(web_trigger_rect.h));
                                layout
                            },
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(Arc::from("Tooltip Trigger")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx, _st| {
                            vec![fixed_size_container(
                                cx,
                                web_trigger_rect.w,
                                web_trigger_rect.h,
                            )]
                        },
                    );
                    trigger_id_out.set(Some(trigger.id));

                    let content = fret_ui_shadcn::TooltipContent::new(Vec::new())
                        .refine_layout(
                            fret_ui_shadcn::LayoutRefinement::default()
                                .w_px(Px(web_content_rect.w))
                                .h_px(Px(web_content_rect.h)),
                        )
                        .into_element(cx);

                    let tooltip = fret_ui_shadcn::Tooltip::new(trigger, content)
                        .open_delay_frames(0)
                        .close_delay_frames(0)
                        .side(match side {
                            Side::Top => fret_ui_shadcn::TooltipSide::Top,
                            Side::Right => fret_ui_shadcn::TooltipSide::Right,
                            Side::Bottom => fret_ui_shadcn::TooltipSide::Bottom,
                            Side::Left => fret_ui_shadcn::TooltipSide::Left,
                        })
                        .align(match align {
                            Align::Start => fret_ui_shadcn::TooltipAlign::Start,
                            Align::Center => fret_ui_shadcn::TooltipAlign::Center,
                            Align::End => fret_ui_shadcn::TooltipAlign::End,
                        })
                        .into_element(cx);

                    let row = cx.flex(FlexProps::default(), move |cx| {
                        vec![
                            spacer(
                                cx,
                                Length::Px(Px(web_trigger_rect.x)),
                                Length::Px(Px(web_trigger_rect.h)),
                            ),
                            tooltip,
                        ]
                    });

                    vec![cx.flex(
                        FlexProps {
                            direction: fret_core::Axis::Vertical,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                spacer(cx, Length::Fill, Length::Px(Px(web_trigger_rect.y))),
                                row,
                            ]
                        },
                    )]
                })
        },
    )]
}

fn render_hover_card_fixture(
    cx: &mut ElementContext<'_, App>,
    web_trigger_rect: DomRect,
    web_content_rect: DomRect,
    side: Side,
    align: Align,
    trigger_id_out: std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>>,
) -> Vec<AnyElement> {
    let mut root_layout = LayoutStyle::default();
    root_layout.size.width = Length::Fill;
    root_layout.size.height = Length::Fill;
    root_layout.position = PositionStyle::Relative;

    vec![cx.container(
        ContainerProps {
            layout: root_layout,
            ..Default::default()
        },
        |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(web_trigger_rect.w));
                        layout.size.height = Length::Px(Px(web_trigger_rect.h));
                        layout
                    },
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: Some(Arc::from("HoverCard Trigger")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |cx, _st| {
                    vec![fixed_size_container(
                        cx,
                        web_trigger_rect.w,
                        web_trigger_rect.h,
                    )]
                },
            );
            trigger_id_out.set(Some(trigger.id));

            let content = cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("HoverCard Content")),
                    ..Default::default()
                },
                |cx| {
                    vec![fixed_size_container(
                        cx,
                        web_content_rect.w,
                        web_content_rect.h,
                    )]
                },
            );

            let hover_card = fret_ui_shadcn::HoverCard::new(trigger, content)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .side(match side {
                    Side::Top => fret_ui_shadcn::HoverCardSide::Top,
                    Side::Right => fret_ui_shadcn::HoverCardSide::Right,
                    Side::Bottom => fret_ui_shadcn::HoverCardSide::Bottom,
                    Side::Left => fret_ui_shadcn::HoverCardSide::Left,
                })
                .align(match align {
                    Align::Start => fret_ui_shadcn::HoverCardAlign::Start,
                    Align::Center => fret_ui_shadcn::HoverCardAlign::Center,
                    Align::End => fret_ui_shadcn::HoverCardAlign::End,
                })
                .into_element(cx);

            let row = cx.flex(FlexProps::default(), move |cx| {
                vec![
                    spacer(
                        cx,
                        Length::Px(Px(web_trigger_rect.x)),
                        Length::Px(Px(web_trigger_rect.h)),
                    ),
                    hover_card,
                ]
            });

            vec![cx.flex(
                FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        spacer(cx, Length::Fill, Length::Px(Px(web_trigger_rect.y))),
                        row,
                    ]
                },
            )]
        },
    )]
}

fn fixed_trigger(
    cx: &mut ElementContext<'_, App>,
    label: &str,
    left: f32,
    top: f32,
    w: f32,
    h: f32,
) -> AnyElement {
    cx.pressable(
        PressableProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(w));
                layout.size.height = Length::Px(Px(h));
                layout.inset.left = Some(Px(left));
                layout.inset.top = Some(Px(top));
                layout.position = PositionStyle::Absolute;
                layout
            },
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(Arc::from(label)),
                ..Default::default()
            },
            ..Default::default()
        },
        |cx, _st| vec![fixed_size_container(cx, w, h)],
    )
}

#[test]
fn radix_web_popover_open_geometry_matches_fret() {
    let golden = read_timeline("popover-example.popover.open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "popover-example");
    assert_eq!(golden.primitive, "popover");
    assert_eq!(golden.scenario, "open-close");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.tag == "button"
            && n.attrs.get("aria-haspopup").is_some_and(|v| v == "dialog")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
            && n.text.as_deref() == Some("Open Popover")
    })
    .expect("web trigger node");

    let web_content = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "popover-content")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web popover-content node");

    let side_str = web_content
        .attrs
        .get("data-side")
        .map(String::as_str)
        .unwrap_or("bottom");
    let align_str = web_content
        .attrs
        .get("data-align")
        .map(String::as_str)
        .unwrap_or("center");

    let side = parse_side(side_str);
    let align = parse_align(align_str);

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_content_rect = require_rect(web_content, "web content");
    let web_gap = rect_main_gap(side, web_trigger_rect, web_content_rect);
    let web_cross = rect_cross_delta(side, align, web_trigger_rect, web_content_rect);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    {
        let mut cfg = fret_ui_shadcn::shadcn_themes::shadcn_new_york_v4_config(
            fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
            fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        );
        // Radix's `radix-vega` select content uses `min-w-36` (Tailwind: 9rem).
        // Keep this test aligned to the upstream snapshots without coupling the recipe default.
        cfg.metrics
            .insert("component.select.min_width".to_string(), 144.0);
        fret_ui::Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));
    }
    assert_close(
        "select theme min width metric",
        fret_ui::Theme::global(&app)
            .metric_by_key("component.select.min_width")
            .unwrap_or(Px(-1.0))
            .0,
        144.0,
        0.01,
    );
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let trigger_id_out: std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));

    // Frame 1: closed (establish anchor bounds).
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let mut root_layout = LayoutStyle::default();
            root_layout.size.width = Length::Fill;
            root_layout.size.height = Length::Fill;
            root_layout.position = PositionStyle::Relative;

            vec![cx.container(
                ContainerProps {
                    layout: root_layout,
                    ..Default::default()
                },
                |cx| {
                    let popover = fret_ui_shadcn::Popover::new(open.clone())
                        .side(match parse_side(side_str) {
                            Side::Top => fret_ui_shadcn::PopoverSide::Top,
                            Side::Right => fret_ui_shadcn::PopoverSide::Right,
                            Side::Bottom => fret_ui_shadcn::PopoverSide::Bottom,
                            Side::Left => fret_ui_shadcn::PopoverSide::Left,
                        })
                        .align(match parse_align(align_str) {
                            Align::Start => fret_ui_shadcn::PopoverAlign::Start,
                            Align::Center => fret_ui_shadcn::PopoverAlign::Center,
                            Align::End => fret_ui_shadcn::PopoverAlign::End,
                        })
                        .into_element(
                            cx,
                            |cx| {
                                let trigger = fixed_trigger(
                                    cx,
                                    "Open Popover",
                                    web_trigger_rect.x,
                                    web_trigger_rect.y,
                                    web_trigger_rect.w,
                                    web_trigger_rect.h,
                                );
                                trigger_id_out.set(Some(trigger.id));
                                trigger
                            },
                            |cx| {
                                fret_ui_shadcn::PopoverContent::new(vec![fixed_size_container(
                                    cx,
                                    web_content_rect.w,
                                    web_content_rect.h,
                                )])
                                .a11y_label("Popover Content")
                                .into_element(cx)
                            },
                        );
                    vec![popover]
                },
            )]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    // Frame 2+: open, then settle motion (scale/opacity/translation) before measuring geometry.
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
                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;
                root_layout.position = PositionStyle::Relative;

                vec![cx.container(
                    ContainerProps {
                        layout: root_layout,
                        ..Default::default()
                    },
                    |cx| {
                        let popover = fret_ui_shadcn::Popover::new(open.clone())
                            .side(match parse_side(side_str) {
                                Side::Top => fret_ui_shadcn::PopoverSide::Top,
                                Side::Right => fret_ui_shadcn::PopoverSide::Right,
                                Side::Bottom => fret_ui_shadcn::PopoverSide::Bottom,
                                Side::Left => fret_ui_shadcn::PopoverSide::Left,
                            })
                            .align(match parse_align(align_str) {
                                Align::Start => fret_ui_shadcn::PopoverAlign::Start,
                                Align::Center => fret_ui_shadcn::PopoverAlign::Center,
                                Align::End => fret_ui_shadcn::PopoverAlign::End,
                            })
                            .into_element(
                                cx,
                                |cx| {
                                    let trigger = fixed_trigger(
                                        cx,
                                        "Open Popover",
                                        web_trigger_rect.x,
                                        web_trigger_rect.y,
                                        web_trigger_rect.w,
                                        web_trigger_rect.h,
                                    );
                                    trigger_id_out.set(Some(trigger.id));
                                    trigger
                                },
                                |cx| {
                                    fret_ui_shadcn::PopoverContent::new(vec![fixed_size_container(
                                        cx,
                                        web_content_rect.w,
                                        web_content_rect.h,
                                    )])
                                    .a11y_label("Popover Content")
                                    .into_element(cx)
                                },
                            );
                        vec![popover]
                    },
                )]
            },
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_trigger = find_semantics(&snap, SemanticsRole::Button, Some("Open Popover"))
        .unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret trigger semantics");
        });
    let fret_content = find_semantics(&snap, SemanticsRole::Panel, Some("Popover Content"))
        .unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret popover content semantics");
        });

    let fret_trigger_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_trigger.id)
            .expect("fret trigger visual bounds"),
    );
    let fret_content_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_content.id)
            .expect("fret popover content visual bounds"),
    );

    let fret_gap = rect_main_gap(side, fret_trigger_rect, fret_content_rect);
    let fret_cross = rect_cross_delta(side, align, fret_trigger_rect, fret_content_rect);

    if (fret_gap - web_gap).abs() > 2.0 {
        eprintln!("-- popover mismatch debug");
        eprintln!("side={side_str} align={align_str}");
        eprintln!("web trigger rect:  {:?}", web_trigger_rect);
        eprintln!("web content rect:  {:?}", web_content_rect);
        eprintln!("fret trigger rect: {:?}", fret_trigger_rect);
        eprintln!("fret content rect: {:?}", fret_content_rect);
        eprintln!("web gap={web_gap} cross={web_cross}");
        eprintln!("fret gap={fret_gap} cross={fret_cross}");
        if let Some(trigger) = trigger_id_out.get() {
            let last_bounds = fret_ui::elements::bounds_for_element(&mut app, window, trigger);
            let last_visual =
                fret_ui::elements::visual_bounds_for_element(&mut app, window, trigger);
            let node = fret_ui::elements::node_for_element(&mut app, window, trigger);
            eprintln!("fret trigger last bounds:  {last_bounds:?}");
            eprintln!("fret trigger last visual:  {last_visual:?}");
            eprintln!("fret trigger node:         {node:?}");
            if let Some(node) = node {
                eprintln!(
                    "fret trigger node bounds:  {:?}",
                    ui.debug_node_bounds(node)
                );
                eprintln!(
                    "fret trigger node visual:  {:?}",
                    ui.debug_node_visual_bounds(node)
                );
            }
        }
    }

    assert_close("popover main gap", fret_gap, web_gap, 2.0);
    assert_close("popover cross delta", fret_cross, web_cross, 2.0);
}

#[test]
fn radix_web_dropdown_menu_open_geometry_matches_fret() {
    let golden = read_timeline("dropdown-menu-example.dropdown-menu.open-navigate-select.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "dropdown-menu-example");
    assert_eq!(golden.primitive, "dropdown-menu");
    assert_eq!(golden.scenario, "open-navigate-select");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.tag == "button"
            && n.attrs.get("aria-haspopup").is_some_and(|v| v == "menu")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
            && n.text.as_deref() == Some("Open")
    })
    .expect("web trigger node");

    let web_menu = find_first(dom, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "menu")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web menu node");

    let side_str = web_menu
        .attrs
        .get("data-side")
        .map(String::as_str)
        .unwrap_or("bottom");
    let align_str = web_menu
        .attrs
        .get("data-align")
        .map(String::as_str)
        .unwrap_or("start");

    let side = parse_side(side_str);
    let align = parse_align(align_str);

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_menu_rect = require_rect(web_menu, "web menu");
    let web_gap = rect_main_gap(side, web_trigger_rect, web_menu_rect);
    let web_cross = rect_cross_delta(side, align, web_trigger_rect, web_menu_rect);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    // Frame 1: closed (establish anchor bounds).
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let trigger = fixed_trigger(cx, "Open", 200.0, 200.0, 80.0, 36.0);
            let menu = fret_ui_shadcn::DropdownMenu::new(open.clone())
                .side(match parse_side(side_str) {
                    Side::Top => fret_ui_shadcn::DropdownMenuSide::Top,
                    Side::Right => fret_ui_shadcn::DropdownMenuSide::Right,
                    Side::Bottom => fret_ui_shadcn::DropdownMenuSide::Bottom,
                    Side::Left => fret_ui_shadcn::DropdownMenuSide::Left,
                })
                .align(match parse_align(align_str) {
                    Align::Start => fret_ui_shadcn::DropdownMenuAlign::Start,
                    Align::Center => fret_ui_shadcn::DropdownMenuAlign::Center,
                    Align::End => fret_ui_shadcn::DropdownMenuAlign::End,
                })
                .into_element(
                    cx,
                    |_cx| trigger,
                    |_cx| {
                        vec![
                            fret_ui_shadcn::DropdownMenuEntry::Group(
                                fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                    fret_ui_shadcn::DropdownMenuEntry::Item(
                                        fret_ui_shadcn::DropdownMenuItem::new("Profile"),
                                    ),
                                    fret_ui_shadcn::DropdownMenuEntry::Item(
                                        fret_ui_shadcn::DropdownMenuItem::new("Billing"),
                                    ),
                                    fret_ui_shadcn::DropdownMenuEntry::Item(
                                        fret_ui_shadcn::DropdownMenuItem::new("Settings"),
                                    ),
                                ]),
                            ),
                            fret_ui_shadcn::DropdownMenuEntry::Separator,
                            fret_ui_shadcn::DropdownMenuEntry::Group(
                                fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                    fret_ui_shadcn::DropdownMenuEntry::Item(
                                        fret_ui_shadcn::DropdownMenuItem::new("GitHub"),
                                    ),
                                    fret_ui_shadcn::DropdownMenuEntry::Item(
                                        fret_ui_shadcn::DropdownMenuItem::new("Support"),
                                    ),
                                    fret_ui_shadcn::DropdownMenuEntry::Item(
                                        fret_ui_shadcn::DropdownMenuItem::new("API"),
                                    ),
                                ]),
                            ),
                        ]
                    },
                );
            vec![menu]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    // Frame 2+: open, then settle motion (scale/opacity/translation) before measuring geometry.
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
                let trigger = fixed_trigger(cx, "Open", 200.0, 200.0, 80.0, 36.0);
                let menu = fret_ui_shadcn::DropdownMenu::new(open.clone())
                    .side(match parse_side(side_str) {
                        Side::Top => fret_ui_shadcn::DropdownMenuSide::Top,
                        Side::Right => fret_ui_shadcn::DropdownMenuSide::Right,
                        Side::Bottom => fret_ui_shadcn::DropdownMenuSide::Bottom,
                        Side::Left => fret_ui_shadcn::DropdownMenuSide::Left,
                    })
                    .align(match parse_align(align_str) {
                        Align::Start => fret_ui_shadcn::DropdownMenuAlign::Start,
                        Align::Center => fret_ui_shadcn::DropdownMenuAlign::Center,
                        Align::End => fret_ui_shadcn::DropdownMenuAlign::End,
                    })
                    .into_element(
                        cx,
                        |_cx| trigger,
                        |_cx| {
                            vec![
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Profile"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Billing"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Settings"),
                                        ),
                                    ]),
                                ),
                                fret_ui_shadcn::DropdownMenuEntry::Separator,
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("GitHub"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Support"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("API"),
                                        ),
                                    ]),
                                ),
                            ]
                        },
                    );
                vec![menu]
            },
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_trigger =
        find_semantics(&snap, SemanticsRole::Button, Some("Open")).unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret trigger semantics");
        });
    let menu_id = fret_trigger.controls.first().copied().unwrap_or_else(|| {
        dump_semantics(&snap);
        panic!("expected dropdown trigger.controls to include menu root");
    });
    let menu_root = snap
        .nodes
        .iter()
        .find(|n| n.id == menu_id)
        .unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret menu semantics node by trigger.controls");
        });
    let menu_panel = snap
        .nodes
        .iter()
        .find(|n| {
            n.parent == Some(menu_root.id)
                && n.role == SemanticsRole::Generic
                && n.bounds.size.width.0 < bounds.size.width.0
                && n.bounds.size.height.0 < bounds.size.height.0
        })
        .unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("expected menu panel node under menu_root");
        });

    let fret_trigger_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_trigger.id)
            .expect("fret trigger visual bounds"),
    );
    let fret_menu_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(menu_panel.id)
            .expect("fret menu panel visual bounds"),
    );

    let fret_gap = rect_main_gap(side, fret_trigger_rect, fret_menu_rect);
    let fret_cross = rect_cross_delta(side, align, fret_trigger_rect, fret_menu_rect);

    assert_close("dropdown-menu main gap", fret_gap, web_gap, 2.0);
    assert_close("dropdown-menu cross delta", fret_cross, web_cross, 2.0);
}

#[test]
fn radix_web_select_item_aligned_geometry_matches_fret() {
    let golden = read_timeline("select-example.select.open-navigate-select.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "select-example");
    assert_eq!(golden.primitive, "select");
    assert_eq!(golden.scenario, "open-navigate-select");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|v| v == "combobox")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
            && n.text.as_deref() == Some("Select a fruit")
    })
    .expect("web trigger node");

    let web_listbox = find_first(dom, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "listbox")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
            && n.attrs
                .get("data-slot")
                .is_some_and(|v| v == "select-content")
    })
    .expect("web listbox node");

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_listbox_rect = require_rect(web_listbox, "web listbox");
    let web_top_delta = web_listbox_rect.y - web_trigger_rect.y;
    let web_left_delta = web_listbox_rect.x - web_trigger_rect.x;
    let web_width_delta = web_listbox_rect.w - web_trigger_rect.w;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    {
        let mut cfg = fret_ui_shadcn::shadcn_themes::shadcn_new_york_v4_config(
            fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
            fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        );
        // Radix's `radix-vega` select content uses `min-w-36` (Tailwind: 9rem).
        // Keep this geometry assertion aligned to upstream snapshots without changing the
        // shadcn recipe defaults.
        cfg.metrics
            .insert("component.select.min_width".to_string(), 144.0);
        fret_ui::Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));
    }
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let items = vec![
        fret_ui_shadcn::SelectItem::new("apple", "Apple"),
        fret_ui_shadcn::SelectItem::new("banana", "Banana"),
        fret_ui_shadcn::SelectItem::new("blueberry", "Blueberry"),
        fret_ui_shadcn::SelectItem::new("grapes", "Grapes").disabled(true),
        fret_ui_shadcn::SelectItem::new("pineapple", "Pineapple"),
    ];

    let trigger_origin = Point::new(Px(web_trigger_rect.x), Px(web_trigger_rect.y));

    // Frame 1: closed (establish trigger bounds).
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let mut root_layout = LayoutStyle::default();
            root_layout.size.width = Length::Fill;
            root_layout.size.height = Length::Fill;
            root_layout.position = PositionStyle::Relative;

            let mut trigger_layout = LayoutStyle::default();
            trigger_layout.position = PositionStyle::Absolute;
            trigger_layout.inset.left = Some(trigger_origin.x);
            trigger_layout.inset.top = Some(trigger_origin.y);

            vec![cx.container(
                ContainerProps {
                    layout: root_layout,
                    ..Default::default()
                },
                |cx| {
                    vec![cx.container(
                        ContainerProps {
                            layout: trigger_layout,
                            ..Default::default()
                        },
                        |cx| {
                            vec![
                                fret_ui_shadcn::Select::new(value.clone(), open.clone())
                                    .placeholder("Select a fruit")
                                    .a11y_label("Select a fruit")
                                    .position(fret_ui_shadcn::select::SelectPosition::ItemAligned)
                                    .items(items.clone())
                                    .into_element(cx),
                            ]
                        },
                    )]
                },
            )]
        },
    );

    // Open via a real trigger click so the component can initialize its internal item-aligned state.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(web_trigger_rect.x + 10.0), Px(web_trigger_rect.y + 10.0)),
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
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(web_trigger_rect.x + 10.0), Px(web_trigger_rect.y + 10.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Frame 2+: open, then settle (motion + item-aligned bookkeeping) before measuring geometry.
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 4;
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
                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;
                root_layout.position = PositionStyle::Relative;

                let mut trigger_layout = LayoutStyle::default();
                trigger_layout.position = PositionStyle::Absolute;
                trigger_layout.inset.left = Some(trigger_origin.x);
                trigger_layout.inset.top = Some(trigger_origin.y);

                vec![cx.container(
                    ContainerProps {
                        layout: root_layout,
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.container(
                            ContainerProps {
                                layout: trigger_layout,
                                ..Default::default()
                            },
                            |cx| {
                                vec![
                                    fret_ui_shadcn::Select::new(value.clone(), open.clone())
                                        .placeholder("Select a fruit")
                                        .a11y_label("Select a fruit")
                                        .position(
                                            fret_ui_shadcn::select::SelectPosition::ItemAligned,
                                        )
                                        .items(items.clone())
                                        .into_element(cx),
                                ]
                            },
                        )]
                    },
                )]
            },
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_trigger = find_semantics(&snap, SemanticsRole::ComboBox, Some("Select a fruit"))
        .unwrap_or_else(|| {
            find_semantics(&snap, SemanticsRole::ComboBox, None).expect("fret combobox semantics")
        });
    let fret_listbox = find_semantics(&snap, SemanticsRole::ListBox, None).unwrap_or_else(|| {
        dump_semantics(&snap);
        panic!("fret listbox semantics");
    });

    let fret_trigger_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_trigger.id)
            .expect("fret trigger visual bounds"),
    );
    let fret_listbox_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_listbox.id)
            .expect("fret listbox visual bounds"),
    );

    let fret_top_delta = fret_listbox_rect.y - fret_trigger_rect.y;
    let fret_left_delta = fret_listbox_rect.x - fret_trigger_rect.x;
    let fret_width_delta = fret_listbox_rect.w - fret_trigger_rect.w;

    if (fret_top_delta - web_top_delta).abs() > 2.5
        || (fret_left_delta - web_left_delta).abs() > 2.5
        || (fret_width_delta - web_width_delta).abs() > 3.0
    {
        eprintln!("-- select mismatch debug");
        eprintln!("web trigger rect:   {:?}", web_trigger_rect);
        eprintln!("web listbox rect:   {:?}", web_listbox_rect);
        eprintln!("fret trigger rect:  {:?}", fret_trigger_rect);
        eprintln!("fret listbox rect:  {:?}", fret_listbox_rect);
        eprintln!("web top={web_top_delta} left={web_left_delta} w={web_width_delta}");
        eprintln!("fret top={fret_top_delta} left={fret_left_delta} w={fret_width_delta}");

        let listboxes: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::ListBox)
            .collect();
        eprintln!("listbox nodes: {}", listboxes.len());
        for n in listboxes {
            let vb = ui.debug_node_visual_bounds(n.id);
            eprintln!(
                "- id={:?} parent={:?} bounds={:?} visual={:?} label={:?}",
                n.id, n.parent, n.bounds, vb, n.label
            );
        }

        let candidates: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| {
                n.bounds.size.width.0 < bounds.size.width.0
                    && n.bounds.size.height.0 < bounds.size.height.0
            })
            .take(20)
            .collect();
        eprintln!("non-window nodes (first 20): {}", candidates.len());
        for n in candidates {
            let vb = ui.debug_node_visual_bounds(n.id);
            eprintln!(
                "- role={:?} id={:?} bounds={:?} visual={:?} label={:?}",
                n.role, n.id, n.bounds, vb, n.label
            );
        }
    }

    assert_close(
        "select item-aligned top delta",
        fret_top_delta,
        web_top_delta,
        2.5,
    );
    assert_close(
        "select item-aligned left delta",
        fret_left_delta,
        web_left_delta,
        2.5,
    );
    assert_close(
        "select item-aligned width delta",
        fret_width_delta,
        web_width_delta,
        3.0,
    );
}

#[test]
fn radix_web_tooltip_hover_geometry_matches_fret() {
    let golden = read_timeline("tooltip-example.tooltip.hover-show-hide.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "tooltip-example");
    assert_eq!(golden.primitive, "tooltip");
    assert_eq!(golden.scenario, "hover-show-hide");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "tooltip-trigger")
            && n.attrs
                .get("data-state")
                .is_some_and(|v| v == "delayed-open")
    })
    .expect("web tooltip trigger node");

    let web_content = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "tooltip-content")
            && n.attrs
                .get("data-state")
                .is_some_and(|v| v == "delayed-open")
    })
    .expect("web tooltip content node");

    let side_str = web_content
        .attrs
        .get("data-side")
        .map(String::as_str)
        .unwrap_or("top");
    let align_str = web_content
        .attrs
        .get("data-align")
        .map(String::as_str)
        .unwrap_or("center");

    let side = parse_side(side_str);
    let align = parse_align(align_str);

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_content_rect = require_rect(web_content, "web content");
    let web_gap = rect_main_gap(side, web_trigger_rect, web_content_rect);
    let web_cross = rect_cross_delta(side, align, web_trigger_rect, web_content_rect);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    {
        let mut cfg = fret_ui_shadcn::shadcn_themes::shadcn_new_york_v4_config(
            fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
            fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        );
        cfg.metrics.insert(
            "component.menubar.min_width".to_string(),
            web_content_rect.w,
        );
        fret_ui::Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));
    }
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let trigger_id_out: std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));

    // Frame 1: establish trigger bounds.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            render_tooltip_fixture(
                cx,
                web_trigger_rect,
                web_content_rect,
                side,
                align,
                trigger_id_out.clone(),
            )
        },
    );

    // Focus trigger to open tooltip.
    let trigger_element = trigger_id_out.get().expect("tooltip trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("tooltip trigger node");
    ui.set_focus(Some(trigger_node));

    // Frame 2+: open, then settle motion (scale/opacity/translation) before measuring geometry.
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
                render_tooltip_fixture(
                    cx,
                    web_trigger_rect,
                    web_content_rect,
                    side,
                    align,
                    trigger_id_out.clone(),
                )
            },
        );
    }

    assert_eq!(
        trigger_id_out.get(),
        Some(trigger_element),
        "expected tooltip trigger element ID to remain stable across frames"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_trigger = find_semantics(&snap, SemanticsRole::Button, Some("Tooltip Trigger"))
        .unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret tooltip trigger semantics");
        });
    let fret_tooltip = find_semantics(&snap, SemanticsRole::Tooltip, None).unwrap_or_else(|| {
        dump_semantics(&snap);
        panic!("fret tooltip semantics");
    });

    let fret_trigger_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_trigger.id)
            .expect("fret trigger visual bounds"),
    );
    let fret_content_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_tooltip.id)
            .expect("fret tooltip visual bounds"),
    );

    let fret_gap = rect_main_gap(side, fret_trigger_rect, fret_content_rect);
    let fret_cross = rect_cross_delta(side, align, fret_trigger_rect, fret_content_rect);

    assert_close("tooltip main gap", fret_gap, web_gap, 2.0);
    assert_close("tooltip cross delta", fret_cross, web_cross, 2.0);
}

#[test]
fn radix_web_hover_card_hover_geometry_matches_fret() {
    let golden = read_timeline("hover-card-example.hover-card.hover.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "hover-card-example");
    assert_eq!(golden.primitive, "hover-card");
    assert_eq!(golden.scenario, "hover");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.tag == "button"
            && n.attrs.get("data-slot").is_some_and(|v| v == "button")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
            && n.text.as_deref() == Some("top")
    })
    .expect("web hover-card trigger node");

    let web_content = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "hover-card-content")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web hover-card content node");

    let side_str = web_content
        .attrs
        .get("data-side")
        .map(String::as_str)
        .unwrap_or("bottom");
    let align_str = web_content
        .attrs
        .get("data-align")
        .map(String::as_str)
        .unwrap_or("center");

    let side = parse_side(side_str);
    let align = parse_align(align_str);

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_content_rect = require_rect(web_content, "web content");
    let web_gap = rect_main_gap(side, web_trigger_rect, web_content_rect);
    let web_cross = rect_cross_delta(side, align, web_trigger_rect, web_content_rect);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

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
    let trigger_id_out: std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));

    // Frame 1: establish trigger bounds.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            render_hover_card_fixture(
                cx,
                web_trigger_rect,
                web_content_rect,
                side,
                align,
                trigger_id_out.clone(),
            )
        },
    );

    // Hover trigger to open hover card.
    let trigger_element = trigger_id_out.get().expect("hover-card trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("hover-card trigger node");
    let trigger_bounds = ui
        .debug_node_bounds(trigger_node)
        .expect("hover-card trigger bounds");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(
                Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
                Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
            ),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    // Frame 2+: open, then settle motion (scale/opacity/translation) before measuring geometry.
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
                render_hover_card_fixture(
                    cx,
                    web_trigger_rect,
                    web_content_rect,
                    side,
                    align,
                    trigger_id_out.clone(),
                )
            },
        );
    }

    assert_eq!(
        trigger_id_out.get(),
        Some(trigger_element),
        "expected hover-card trigger element ID to remain stable across frames"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_trigger = find_semantics(&snap, SemanticsRole::Button, Some("HoverCard Trigger"))
        .unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret hover-card trigger semantics");
        });
    let fret_content = find_semantics(&snap, SemanticsRole::Panel, Some("HoverCard Content"))
        .unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret hover-card content semantics");
        });

    let fret_trigger_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_trigger.id)
            .expect("fret trigger visual bounds"),
    );
    let fret_content_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_content.id)
            .expect("fret hover-card content visual bounds"),
    );

    let fret_gap = rect_main_gap(side, fret_trigger_rect, fret_content_rect);
    let fret_cross = rect_cross_delta(side, align, fret_trigger_rect, fret_content_rect);

    assert_close("hover-card main gap", fret_gap, web_gap, 2.0);
    assert_close("hover-card cross delta", fret_cross, web_cross, 2.0);
}

#[test]
fn radix_web_context_menu_open_geometry_matches_fret() {
    let golden = read_timeline("context-menu-example.context-menu.context-open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "context-menu-example");
    assert_eq!(golden.primitive, "context-menu");
    assert_eq!(golden.scenario, "context-open-close");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "context-menu-trigger")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web context-menu trigger node");

    let web_content = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "context-menu-content")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web context-menu content node");

    let side_str = web_content
        .attrs
        .get("data-side")
        .map(String::as_str)
        .unwrap_or("right");
    let align_str = web_content
        .attrs
        .get("data-align")
        .map(String::as_str)
        .unwrap_or("start");

    let side = parse_side(side_str);
    let align = parse_align(align_str);

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_content_rect = require_rect(web_content, "web content");

    // `extract-behavior.mts`: `rightClickUntilRoleAppears` clicks `box.x + 5, box.y + 5`.
    let web_anchor_rect = DomRect {
        x: web_trigger_rect.x + 5.0,
        y: web_trigger_rect.y + 5.0,
        w: 0.0,
        h: 0.0,
    };
    let web_gap = rect_main_gap(side, web_anchor_rect, web_content_rect);
    let web_cross = rect_cross_delta(side, align, web_anchor_rect, web_content_rect);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    // Frame 1: build the tree and establish stable trigger bounds.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let mut root_layout = LayoutStyle::default();
            root_layout.size.width = Length::Fill;
            root_layout.size.height = Length::Fill;
            root_layout.position = PositionStyle::Relative;

            let open = open.clone();
            let trigger_left = web_trigger_rect.x;
            let trigger_top = web_trigger_rect.y;
            let trigger_w = web_trigger_rect.w;
            let trigger_h = web_trigger_rect.h;

            vec![cx.flex(
                FlexProps {
                    layout: root_layout,
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        spacer(cx, Length::Fill, Length::Px(Px(trigger_top))),
                        cx.flex(
                            FlexProps {
                                direction: fret_core::Axis::Horizontal,
                                ..Default::default()
                            },
                            move |cx| {
                                let open = open.clone();
                                let trigger = cx.pressable(
                                    PressableProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Px(Px(trigger_w));
                                            layout.size.height = Length::Px(Px(trigger_h));
                                            layout
                                        },
                                        a11y: PressableA11y {
                                            role: Some(SemanticsRole::Button),
                                            label: Some(Arc::from("ContextMenu Trigger")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |cx, _st| vec![fixed_size_container(cx, trigger_w, trigger_h)],
                                );

                                let menu = fret_ui_shadcn::ContextMenu::new(open)
                                    .min_width(Px(web_content_rect.w))
                                    .into_element(
                                        cx,
                                        |_cx| trigger,
                                        |_cx| {
                                            vec![fret_ui_shadcn::ContextMenuEntry::Group(
                                                fret_ui_shadcn::ContextMenuGroup::new(vec![
                                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                                        fret_ui_shadcn::ContextMenuItem::new(
                                                            "Back",
                                                        ),
                                                    ),
                                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                                        fret_ui_shadcn::ContextMenuItem::new(
                                                            "Forward",
                                                        ),
                                                    ),
                                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                                        fret_ui_shadcn::ContextMenuItem::new(
                                                            "Reload",
                                                        ),
                                                    ),
                                                ]),
                                            )]
                                        },
                                    );

                                vec![
                                    spacer(
                                        cx,
                                        Length::Px(Px(trigger_left)),
                                        Length::Px(Px(trigger_h)),
                                    ),
                                    menu,
                                ]
                            },
                        ),
                    ]
                },
            )]
        },
    );

    let click_position = Point::new(Px(web_anchor_rect.x), Px(web_anchor_rect.y));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: click_position,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: click_position,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: click_position,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    // Frame 2+: open, then settle motion (scale/opacity/translation) before measuring geometry.
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
                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;
                root_layout.position = PositionStyle::Relative;

                let open = open.clone();
                let trigger_left = web_trigger_rect.x;
                let trigger_top = web_trigger_rect.y;
                let trigger_w = web_trigger_rect.w;
                let trigger_h = web_trigger_rect.h;

                vec![cx.flex(
                    FlexProps {
                        layout: root_layout,
                        direction: fret_core::Axis::Vertical,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            spacer(cx, Length::Fill, Length::Px(Px(trigger_top))),
                            cx.flex(
                                FlexProps {
                                    direction: fret_core::Axis::Horizontal,
                                    ..Default::default()
                                },
                                move |cx| {
                                    let open = open.clone();
                                    let trigger = cx.pressable(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(trigger_w));
                                                layout.size.height = Length::Px(Px(trigger_h));
                                                layout
                                            },
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: Some(Arc::from("ContextMenu Trigger")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st| {
                                            vec![fixed_size_container(cx, trigger_w, trigger_h)]
                                        },
                                    );

                                    let menu = fret_ui_shadcn::ContextMenu::new(open)
                                        .min_width(Px(web_content_rect.w))
                                        .into_element(
                                            cx,
                                            |_cx| trigger,
                                            |_cx| {
                                                vec![fret_ui_shadcn::ContextMenuEntry::Group(
                                                    fret_ui_shadcn::ContextMenuGroup::new(vec![
                                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                                            fret_ui_shadcn::ContextMenuItem::new(
                                                                "Back",
                                                            ),
                                                        ),
                                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                                            fret_ui_shadcn::ContextMenuItem::new(
                                                                "Forward",
                                                            ),
                                                        ),
                                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                                            fret_ui_shadcn::ContextMenuItem::new(
                                                                "Reload",
                                                            ),
                                                        ),
                                                    ]),
                                                )]
                                            },
                                        );

                                    vec![
                                        spacer(
                                            cx,
                                            Length::Px(Px(trigger_left)),
                                            Length::Px(Px(trigger_h)),
                                        ),
                                        menu,
                                    ]
                                },
                            ),
                        ]
                    },
                )]
            },
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let back_item = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Back"))
        .unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret Back menu item semantics");
        });

    let mut candidate_ids = Vec::new();
    let mut current = Some(back_item.id);
    while let Some(id) = current {
        candidate_ids.push(id);
        current = snap
            .nodes
            .iter()
            .find(|n| n.id == id)
            .and_then(|n| n.parent);
    }

    let mut best: Option<&fret_core::SemanticsNode> = None;
    let mut best_score = f32::INFINITY;
    for id in candidate_ids {
        let Some(node) = snap.nodes.iter().find(|n| n.id == id) else {
            continue;
        };
        if node.bounds.size.width.0 >= bounds.size.width.0
            || node.bounds.size.height.0 >= bounds.size.height.0
        {
            continue;
        }
        let vb = ui.debug_node_visual_bounds(node.id).unwrap_or(node.bounds);
        let score = (vb.size.width.0 - web_content_rect.w).abs()
            + (vb.size.height.0 - web_content_rect.h).abs();
        if score < best_score {
            best = Some(node);
            best_score = score;
        }
    }

    let menu_panel = best.unwrap_or_else(|| {
        dump_semantics(&snap);
        panic!("fret context-menu panel ancestor");
    });

    let fret_menu_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(menu_panel.id)
            .expect("fret context-menu visual bounds"),
    );
    let fret_anchor_rect = DomRect {
        x: web_anchor_rect.x,
        y: web_anchor_rect.y,
        w: 0.0,
        h: 0.0,
    };

    let fret_gap = rect_main_gap(side, fret_anchor_rect, fret_menu_rect);
    let fret_cross = rect_cross_delta(side, align, fret_anchor_rect, fret_menu_rect);

    assert_close("context-menu main gap", fret_gap, web_gap, 2.0);
    assert_close("context-menu cross delta", fret_cross, web_cross, 2.0);
}

#[test]
fn radix_web_navigation_menu_open_geometry_matches_fret() {
    let golden = read_timeline("navigation-menu-example.navigation-menu.open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "navigation-menu-example");
    assert_eq!(golden.primitive, "navigation-menu");
    assert_eq!(golden.scenario, "open-close");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.tag == "button"
            && n.attrs
                .get("data-slot")
                .is_some_and(|v| v == "navigation-menu-trigger")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
            && n.text.as_deref() == Some("Getting started")
    })
    .expect("web trigger node");

    let web_viewport = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "navigation-menu-viewport")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web viewport node");

    let side = Side::Bottom;
    let align = Align::Start;

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_viewport_rect = require_rect(web_viewport, "web viewport");
    let web_gap = rect_main_gap(side, web_trigger_rect, web_viewport_rect);
    let web_cross = rect_cross_delta(side, align, web_trigger_rect, web_viewport_rect);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None::<Arc<str>>);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    // Frame 1: closed (establish anchor bounds for the trigger element).
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let mut root_layout = LayoutStyle::default();
            root_layout.size.width = Length::Fill;
            root_layout.size.height = Length::Fill;
            root_layout.position = PositionStyle::Relative;

            vec![cx.container(
                ContainerProps {
                    layout: root_layout,
                    ..Default::default()
                },
                |cx| {
                    let items = vec![fret_ui_shadcn::NavigationMenuItem::new(
                        "getting_started",
                        "Getting started",
                        vec![fixed_size_container(
                            cx,
                            web_viewport_rect.w.max(200.0),
                            web_viewport_rect.h.max(120.0),
                        )],
                    )];
                    vec![
                        fret_ui_shadcn::NavigationMenu::new(model.clone())
                            .items(items)
                            .into_element(cx),
                    ]
                },
            )]
        },
    );

    // Frame 2+: open, then settle motion (scale/opacity/translation) before measuring geometry.
    app.models_mut()
        .update(&model, |selected| {
            *selected = Some(Arc::from("getting_started"));
        })
        .expect("update navigation-menu model");

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_200 + 2;
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
                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;
                root_layout.position = PositionStyle::Relative;

                vec![cx.container(
                    ContainerProps {
                        layout: root_layout,
                        ..Default::default()
                    },
                    |cx| {
                        let items = vec![fret_ui_shadcn::NavigationMenuItem::new(
                            "getting_started",
                            "Getting started",
                            vec![fixed_size_container(
                                cx,
                                web_viewport_rect.w.max(200.0),
                                web_viewport_rect.h.max(120.0),
                            )],
                        )];
                        vec![
                            fret_ui_shadcn::NavigationMenu::new(model.clone())
                                .items(items)
                                .into_element(cx),
                        ]
                    },
                )]
            },
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_trigger = find_semantics(&snap, SemanticsRole::Button, Some("Getting started"))
        .unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret navigation-menu trigger semantics");
        });

    let overlay_stack =
        fret_ui_kit::OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    let nav_root = overlay_stack
        .topmost_popover
        .expect("expected navigation-menu overlay root id");
    let anchor_element = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "radix-web-overlay-geometry",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx,
                nav_root,
                "getting_started",
            )
        },
    )
    .expect("expected navigation-menu trigger element id");
    let anchor_rect = fret_rect_to_dom(
        fret_ui::elements::bounds_for_element(&mut app, window, anchor_element)
            .expect("expected trigger element bounds"),
    );

    let viewport_panel_element = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "radix-web-overlay-geometry",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(
                cx, nav_root,
            )
        },
    )
    .expect("expected navigation-menu viewport panel element id");

    let fret_trigger_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_trigger.id)
            .expect("fret trigger visual bounds"),
    );
    let fret_viewport_rect = fret_rect_to_dom(
        fret_ui::elements::bounds_for_element(&mut app, window, viewport_panel_element)
            .or_else(|| {
                fret_ui::elements::visual_bounds_for_element(
                    &mut app,
                    window,
                    viewport_panel_element,
                )
            })
            .expect("expected viewport panel bounds"),
    );

    let fret_gap = rect_main_gap(side, fret_trigger_rect, fret_viewport_rect);
    let fret_cross = rect_cross_delta(side, align, fret_trigger_rect, fret_viewport_rect);

    let theme = fret_ui::Theme::global(&app);
    let side_offset = theme
        .metric_by_key("component.navigation_menu.viewport.side_offset")
        .map(|v| v.0)
        .unwrap_or(-1.0);
    assert_close(
        &format!(
            "navigation-menu main gap (side_offset={side_offset}, trigger_h={}, trigger_bottom={}, viewport_y={}, anchor_bottom={})",
            fret_trigger_rect.h,
            fret_trigger_rect.y + fret_trigger_rect.h,
            fret_viewport_rect.y,
            anchor_rect.y + anchor_rect.h
        ),
        fret_gap,
        web_gap,
        2.0,
    );
    assert_close("navigation-menu cross delta", fret_cross, web_cross, 2.0);
}

#[test]
fn radix_web_menubar_open_geometry_matches_fret() {
    // NOTE: This test intentionally uses the simplest possible scene graph (menubar as the root
    // element) to keep the menubar's internal state stable across frames/events.

    let golden = read_timeline("menubar-example.menubar.open-navigate-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "menubar-example");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "open-navigate-close");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "menubar-trigger")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web menubar trigger node");

    let web_content = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "menubar-content")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web menubar-content node");

    let side_str = web_content
        .attrs
        .get("data-side")
        .map(String::as_str)
        .unwrap_or("bottom");
    let align_str = web_content
        .attrs
        .get("data-align")
        .map(String::as_str)
        .unwrap_or("start");

    let side = parse_side(side_str);
    let align = parse_align(align_str);

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_content_rect = require_rect(web_content, "web content");
    let web_gap = rect_main_gap(side, web_trigger_rect, web_content_rect);
    let web_cross = rect_cross_delta(side, align, web_trigger_rect, web_content_rect);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

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

    fn render_scene(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
        vec![
            fret_ui_shadcn::Menubar::new(vec![
                fret_ui_shadcn::MenubarMenu::new("File").entries(vec![
                    fret_ui_shadcn::MenubarEntry::Group(fret_ui_shadcn::MenubarGroup::new(vec![
                        fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new(
                            "New Tab ⌘T",
                        )),
                        fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new(
                            "New Window ⌘N",
                        )),
                        fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new(
                            "New Incognito Window",
                        )),
                    ])),
                    fret_ui_shadcn::MenubarEntry::Separator,
                    fret_ui_shadcn::MenubarEntry::Group(fret_ui_shadcn::MenubarGroup::new(vec![
                        fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new(
                            "Print... ⌘P",
                        )),
                    ])),
                ]),
                fret_ui_shadcn::MenubarMenu::new("Edit").entries(vec![
                    fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new("Undo")),
                    fret_ui_shadcn::MenubarEntry::Item(fret_ui_shadcn::MenubarItem::new("Redo")),
                ]),
            ])
            .into_element(cx),
        ]
    }

    // Frame 1: closed (establish trigger bounds).
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        render_scene,
    );

    let snap0 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger0 =
        find_semantics(&snap0, SemanticsRole::MenuItem, Some("File")).unwrap_or_else(|| {
            dump_semantics(&snap0);
            panic!("fret menubar trigger semantics");
        });
    let trigger_bounds0 = ui
        .debug_node_visual_bounds(trigger0.id)
        .expect("fret menubar trigger visual bounds");
    let click = Point::new(
        Px(trigger_bounds0.origin.x.0 + trigger_bounds0.size.width.0 * 0.5),
        Px(trigger_bounds0.origin.y.0 + trigger_bounds0.size.height.0 * 0.5),
    );

    ui.set_focus(Some(trigger0.id));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: click,
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
            pointer_id: fret_core::PointerId(0),
            position: click,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    // Frame 2+: open, then settle motion before measuring geometry.
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
            render_scene,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_trigger =
        find_semantics(&snap, SemanticsRole::MenuItem, Some("File")).unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret menubar trigger semantics");
        });
    assert!(
        fret_trigger.flags.expanded,
        "expected menubar trigger to be expanded after click"
    );

    let new_tab_item = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab ⌘T"))
        .unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret New Tab menu item semantics");
        });

    let mut candidate_ids = Vec::new();
    let mut current = Some(new_tab_item.id);
    while let Some(id) = current {
        candidate_ids.push(id);
        current = snap
            .nodes
            .iter()
            .find(|n| n.id == id)
            .and_then(|n| n.parent);
    }

    let mut menu_panel_id = None;
    for (idx, id) in candidate_ids.iter().enumerate() {
        let Some(node) = snap.nodes.iter().find(|n| n.id == *id) else {
            continue;
        };
        if node.role == SemanticsRole::Menu && idx > 0 {
            menu_panel_id = Some(candidate_ids[idx - 1]);
            break;
        }
    }

    let menu_panel = if let Some(id) = menu_panel_id {
        snap.nodes.iter().find(|n| n.id == id).unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret menubar panel node");
        })
    } else {
        let mut best: Option<&fret_core::SemanticsNode> = None;
        let mut best_score = f32::INFINITY;
        for id in candidate_ids {
            let Some(node) = snap.nodes.iter().find(|n| n.id == id) else {
                continue;
            };
            if node.bounds.size.width.0 >= bounds.size.width.0
                || node.bounds.size.height.0 >= bounds.size.height.0
            {
                continue;
            }
            let vb = ui.debug_node_visual_bounds(node.id).unwrap_or(node.bounds);
            let score = (vb.size.width.0 - web_content_rect.w).abs()
                + (vb.size.height.0 - web_content_rect.h).abs();
            if score < best_score {
                best = Some(node);
                best_score = score;
            }
        }

        best.unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret menubar panel ancestor");
        })
    };

    let fret_trigger_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_trigger.id)
            .expect("fret menubar trigger visual bounds"),
    );
    let fret_menu_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(menu_panel.id)
            .expect("fret menubar menu visual bounds"),
    );

    let fret_gap = rect_main_gap(side, fret_trigger_rect, fret_menu_rect);
    let fret_cross = rect_cross_delta(side, align, fret_trigger_rect, fret_menu_rect);

    assert_close("menubar main gap", fret_gap, web_gap, 2.0);
    assert_close("menubar cross delta", fret_cross, web_cross, 2.0);

    /*
    let golden = read_timeline("menubar-example.menubar.open-navigate-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "menubar-example");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "open-navigate-close");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "menubar-trigger")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web menubar trigger node");

    let web_content = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "menubar-content")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web menubar-content node");

    let side_str = web_content
        .attrs
        .get("data-side")
        .map(String::as_str)
        .unwrap_or("bottom");
    let align_str = web_content
        .attrs
        .get("data-align")
        .map(String::as_str)
        .unwrap_or("start");

    let side = parse_side(side_str);
    let align = parse_align(align_str);

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_content_rect = require_rect(web_content, "web content");
    let web_gap = rect_main_gap(side, web_trigger_rect, web_content_rect);
    let web_cross = rect_cross_delta(side, align, web_trigger_rect, web_content_rect);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

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

    let origin = Point::new(Px(200.0), Px(100.0));

    // Frame 1: closed (establish trigger bounds).
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let mut root_layout = LayoutStyle::default();
            root_layout.size.width = Length::Fill;
            root_layout.size.height = Length::Fill;
            root_layout.position = PositionStyle::Relative;

            let mut origin_layout = LayoutStyle::default();
            origin_layout.position = PositionStyle::Absolute;
            origin_layout.inset.left = Some(origin.x);
            origin_layout.inset.top = Some(origin.y);

            vec![cx.keyed("menubar-scene", |cx| {
                cx.container(
                    ContainerProps {
                        layout: root_layout,
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.container(
                            ContainerProps {
                                layout: origin_layout,
                                ..Default::default()
                            },
                            |cx| {
                                vec![cx.keyed("menubar-root", |cx| {
                                    fret_ui_shadcn::Menubar::new(vec![
                                        fret_ui_shadcn::MenubarMenu::new("File").entries(vec![
                                            fret_ui_shadcn::MenubarEntry::Item(
                                                fret_ui_shadcn::MenubarItem::new("New Tab"),
                                            ),
                                            fret_ui_shadcn::MenubarEntry::Item(
                                                fret_ui_shadcn::MenubarItem::new("New Window"),
                                            ),
                                            fret_ui_shadcn::MenubarEntry::Separator,
                                            fret_ui_shadcn::MenubarEntry::Item(
                                                fret_ui_shadcn::MenubarItem::new("Share"),
                                            ),
                                        ]),
                                        fret_ui_shadcn::MenubarMenu::new("Edit").entries(vec![
                                            fret_ui_shadcn::MenubarEntry::Item(
                                                fret_ui_shadcn::MenubarItem::new("Undo"),
                                            ),
                                            fret_ui_shadcn::MenubarEntry::Item(
                                                fret_ui_shadcn::MenubarItem::new("Redo"),
                                            ),
                                        ]),
                                    ])
                                    .into_element(cx)
                                })]
                            },
                        )]
                    },
                )
            })]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_trigger =
        find_semantics(&snap, SemanticsRole::MenuItem, Some("File")).unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret menubar trigger semantics");
        });
    ui.set_focus(Some(fret_trigger.id));

    // Render once with focus applied so focused-only key wiring is installed.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        false,
        |cx| {
            let mut root_layout = LayoutStyle::default();
            root_layout.size.width = Length::Fill;
            root_layout.size.height = Length::Fill;
            root_layout.position = PositionStyle::Relative;

            let mut origin_layout = LayoutStyle::default();
            origin_layout.position = PositionStyle::Absolute;
            origin_layout.inset.left = Some(origin.x);
            origin_layout.inset.top = Some(origin.y);

            vec![cx.keyed("menubar-scene", |cx| {
                cx.container(
                    ContainerProps {
                        layout: root_layout,
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.container(
                            ContainerProps {
                                layout: origin_layout,
                                ..Default::default()
                            },
                            |cx| {
                                vec![cx.keyed("menubar-root", |cx| {
                                    fret_ui_shadcn::Menubar::new(vec![
                                        fret_ui_shadcn::MenubarMenu::new("File").entries(vec![
                                            fret_ui_shadcn::MenubarEntry::Item(
                                                fret_ui_shadcn::MenubarItem::new("New Tab"),
                                            ),
                                            fret_ui_shadcn::MenubarEntry::Item(
                                                fret_ui_shadcn::MenubarItem::new("New Window"),
                                            ),
                                            fret_ui_shadcn::MenubarEntry::Separator,
                                            fret_ui_shadcn::MenubarEntry::Item(
                                                fret_ui_shadcn::MenubarItem::new("Share"),
                                            ),
                                        ]),
                                        fret_ui_shadcn::MenubarMenu::new("Edit").entries(vec![
                                            fret_ui_shadcn::MenubarEntry::Item(
                                                fret_ui_shadcn::MenubarItem::new("Undo"),
                                            ),
                                            fret_ui_shadcn::MenubarEntry::Item(
                                                fret_ui_shadcn::MenubarItem::new("Redo"),
                                            ),
                                        ]),
                                    ])
                                    .into_element(cx)
                                })]
                            },
                        )]
                    },
                )
            })]
        },
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    // Frame 2+: open, then settle motion before measuring geometry.
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            request_semantics,
            |cx| {
                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;
                root_layout.position = PositionStyle::Relative;

                let mut origin_layout = LayoutStyle::default();
                origin_layout.position = PositionStyle::Absolute;
                origin_layout.inset.left = Some(origin.x);
                origin_layout.inset.top = Some(origin.y);

                vec![cx.keyed("menubar-scene", |cx| {
                    cx.container(
                        ContainerProps {
                            layout: root_layout,
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.container(
                                ContainerProps {
                                    layout: origin_layout,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![cx.keyed("menubar-root", |cx| {
                                        fret_ui_shadcn::Menubar::new(vec![
                                            fret_ui_shadcn::MenubarMenu::new("File").entries(vec![
                                                fret_ui_shadcn::MenubarEntry::Item(
                                                    fret_ui_shadcn::MenubarItem::new("New Tab"),
                                                ),
                                                fret_ui_shadcn::MenubarEntry::Item(
                                                    fret_ui_shadcn::MenubarItem::new("New Window"),
                                                ),
                                                fret_ui_shadcn::MenubarEntry::Separator,
                                                fret_ui_shadcn::MenubarEntry::Item(
                                                    fret_ui_shadcn::MenubarItem::new("Share"),
                                                ),
                                            ]),
                                            fret_ui_shadcn::MenubarMenu::new("Edit").entries(vec![
                                                fret_ui_shadcn::MenubarEntry::Item(
                                                    fret_ui_shadcn::MenubarItem::new("Undo"),
                                                ),
                                                fret_ui_shadcn::MenubarEntry::Item(
                                                    fret_ui_shadcn::MenubarItem::new("Redo"),
                                                ),
                                            ]),
                                        ])
                                        .into_element(cx)
                                    })]
                                },
                            )]
                        },
                    )
                })]
            },
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_trigger =
        find_semantics(&snap, SemanticsRole::MenuItem, Some("File")).unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret menubar trigger semantics");
        });
    assert!(
        fret_trigger.flags.expanded,
        "expected menubar trigger to be expanded after ArrowDown"
    );
    let fret_menu = find_semantics(&snap, SemanticsRole::Menu, None).unwrap_or_else(|| {
        dump_semantics(&snap);
        panic!("fret menubar menu semantics");
    });

    let fret_trigger_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_trigger.id)
            .expect("fret menubar trigger visual bounds"),
    );
    let fret_menu_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_menu.id)
            .expect("fret menubar menu visual bounds"),
    );

    let fret_gap = rect_main_gap(side, fret_trigger_rect, fret_menu_rect);
    let fret_cross = rect_cross_delta(side, align, fret_trigger_rect, fret_menu_rect);

    assert_close("menubar main gap", fret_gap, web_gap, 2.0);
    assert_close("menubar cross delta", fret_cross, web_cross, 2.0);
    */
}

#[test]
fn radix_web_dialog_open_geometry_matches_fret() {
    let golden = read_timeline("dialog-example.dialog.open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "dialog-example");
    assert_eq!(golden.primitive, "dialog");
    assert_eq!(golden.scenario, "open-close");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "dialog-trigger")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web dialog trigger node");

    let web_content = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "dialog-content")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web dialog-content node");

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_content_rect = require_rect(web_content, "web content");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let open: Model<bool> = app.models_mut().insert(true);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_200 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(1 + tick),
            request_semantics,
            |cx| {
                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;
                root_layout.position = PositionStyle::Relative;

                vec![cx.container(
                    ContainerProps {
                        layout: root_layout,
                        ..Default::default()
                    },
                    |cx| {
                        vec![fret_ui_shadcn::Dialog::new(open.clone()).into_element(
                            cx,
                            |cx| {
                                fixed_trigger(
                                    cx,
                                    "Open Dialog",
                                    web_trigger_rect.x,
                                    web_trigger_rect.y,
                                    web_trigger_rect.w,
                                    web_trigger_rect.h,
                                )
                            },
                            |cx| {
                                fret_ui_shadcn::DialogContent::new(Vec::new())
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .w_px(Px(web_content_rect.w))
                                            .h_px(Px(web_content_rect.h))
                                            .max_w(Px(web_content_rect.w)),
                                    )
                                    .into_element(cx)
                            },
                        )]
                    },
                )]
            },
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_dialog = find_semantics(&snap, SemanticsRole::Dialog, None).unwrap_or_else(|| {
        dump_semantics(&snap);
        panic!("fret dialog semantics");
    });
    let fret_dialog_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_dialog.id)
            .expect("fret dialog visual bounds"),
    );

    assert_rect_close("dialog rect", fret_dialog_rect, web_content_rect, 2.0);
}

#[test]
fn radix_web_alert_dialog_open_geometry_matches_fret() {
    let golden = read_timeline("alert-dialog-example.alert-dialog.open-cancel.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.theme, "light");
    assert_eq!(golden.item, "alert-dialog-example");
    assert_eq!(golden.primitive, "alert-dialog");
    assert_eq!(golden.scenario, "open-cancel");
    assert!(golden.steps.len() >= 2);

    let dom = &golden.steps[1].snapshot.dom;
    let web_trigger = find_first(dom, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v == "button")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
            && n.attrs.get("aria-haspopup").is_some_and(|v| v == "dialog")
    })
    .expect("web alert-dialog trigger node");

    let web_content = find_first(dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "alert-dialog-content")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    })
    .expect("web alert-dialog-content node");

    let web_trigger_rect = require_rect(web_trigger, "web trigger");
    let web_content_rect = require_rect(web_content, "web content");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(800.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let open: Model<bool> = app.models_mut().insert(true);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_200 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(1 + tick),
            request_semantics,
            |cx| {
                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;
                root_layout.position = PositionStyle::Relative;

                vec![cx.container(
                    ContainerProps {
                        layout: root_layout,
                        ..Default::default()
                    },
                    |cx| {
                        vec![fret_ui_shadcn::AlertDialog::new(open.clone()).into_element(
                            cx,
                            |cx| {
                                fixed_trigger(
                                    cx,
                                    "Open AlertDialog",
                                    web_trigger_rect.x,
                                    web_trigger_rect.y,
                                    web_trigger_rect.w,
                                    web_trigger_rect.h,
                                )
                            },
                            |cx| {
                                fret_ui_shadcn::AlertDialogContent::new(Vec::new())
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .w_px(Px(web_content_rect.w))
                                            .h_px(Px(web_content_rect.h))
                                            .max_w(Px(web_content_rect.w)),
                                    )
                                    .into_element(cx)
                            },
                        )]
                    },
                )]
            },
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let fret_dialog =
        find_semantics(&snap, SemanticsRole::AlertDialog, None).unwrap_or_else(|| {
            dump_semantics(&snap);
            panic!("fret alert-dialog semantics");
        });
    let fret_dialog_rect = fret_rect_to_dom(
        ui.debug_node_visual_bounds(fret_dialog.id)
            .expect("fret alert-dialog visual bounds"),
    );

    if (fret_dialog_rect.x - web_content_rect.x).abs() > 2.0
        || (fret_dialog_rect.y - web_content_rect.y).abs() > 2.0
        || (fret_dialog_rect.w - web_content_rect.w).abs() > 2.0
        || (fret_dialog_rect.h - web_content_rect.h).abs() > 2.0
    {
        eprintln!("-- alert-dialog mismatch debug");
        eprintln!("web content rect:  {:?}", web_content_rect);
        eprintln!("fret dialog rect:  {:?}", fret_dialog_rect);
    }

    assert_rect_close("alert-dialog rect", fret_dialog_rect, web_content_rect, 2.0);
}
