//! Focused Material 3 Tooltip layout, accessibility, and motion regression tests.

use fret_core::{
    AppWindowId, Color, Edges, NodeId, Paint, Point, PointerId, Px, Rect, Scene, SceneOp,
    SemanticsLive, SemanticsNode, SemanticsRole, Size, UiServices,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length};
use fret_ui::{ElementContext, UiHost, UiTree};
use fret_ui_kit::{ColorRef, OverlayController, OverlayStackEntryKind, WidgetStateProperty};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{
    Button, ButtonVariant, PlainTooltip, RichTooltip, TooltipProvider, TooltipStyle,
};

mod support;

use support::events::pointer_move;
use support::goldens::run_overlay_frame_with_scene_scaled;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn oversized_content_probe<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(Px(480.0));
    layout.size.height = Length::Px(Px(8.0));
    cx.container(
        ContainerProps {
            layout,
            ..Default::default()
        },
        |_cx| Vec::new(),
    )
}

fn semantics_node_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> SemanticsNode {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
                .cloned()
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
}

fn live_test_id_layout_bounds(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .find_map(|m| {
            ui.debug_node_bounds(m.node)
                .or_else(|| ui.debug_node_visual_bounds(m.node))
        })
        .unwrap_or_else(|| panic!("expected live layout bounds for test_id {id}"))
}

fn center_for_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Point {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"));
    center_for_node(ui, node)
}

fn center_for_node(ui: &UiTree<TestHost>, node: NodeId) -> Point {
    let bounds = ui
        .debug_node_visual_bounds(node)
        .or_else(|| ui.debug_node_bounds(node))
        .expect("expected node bounds");
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

fn scene_has_intermediate_overlay_motion(scene: &Scene) -> bool {
    let has_alpha = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushOpacity { opacity } if *opacity > 0.01 && *opacity < 0.99
        )
    });
    let has_scale = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if transform.b.abs() < 0.001
                    && transform.c.abs() < 0.001
                    && transform.a > 0.8
                    && transform.a < 1.0
                    && transform.d > 0.8
                    && transform.d < 1.0
        )
    });
    has_alpha && has_scale
}

fn color_close(actual: Color, expected: Color) -> bool {
    (actual.r - expected.r).abs() <= 0.001
        && (actual.g - expected.g).abs() <= 0.001
        && (actual.b - expected.b).abs() <= 0.001
        && (actual.a - expected.a).abs() <= 0.001
}

fn scene_has_solid_quad_color(scene: &Scene, expected: Color) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::Quad { background, .. }
                if matches!(background.paint, Paint::Solid(color) if color_close(color, expected))
        )
    })
}

fn scene_has_solid_text_color(scene: &Scene, expected: Color) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::Text { paint, .. }
                if matches!(paint.paint, Paint::Solid(color) if color_close(color, expected))
        )
    })
}

fn run_until_tooltip_visible(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    mut render: impl FnMut(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) -> Scene {
    for _ in 0..8 {
        let scene = run_overlay_frame_with_scene_scaled(
            ui,
            app,
            services,
            window,
            bounds,
            1.0,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let stack = OverlayController::stack_snapshot_for_window(ui, app, window);
        if stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
        {
            return scene;
        }
    }

    panic!("expected tooltip overlay to become visible");
}

fn assert_material_tooltip_semantics(
    ui: &UiTree<TestHost>,
    trigger_test_id: &str,
    tooltip_test_id: &str,
) {
    let trigger = semantics_node_by_test_id(ui, trigger_test_id);
    let tooltip = semantics_node_by_test_id(ui, tooltip_test_id);

    assert_eq!(tooltip.role, SemanticsRole::Tooltip);
    assert_eq!(tooltip.flags.live, Some(SemanticsLive::Assertive));
    assert!(
        trigger.described_by.contains(&tooltip.id),
        "expected trigger {trigger_test_id} to be described by tooltip {tooltip_test_id}"
    );
}

#[test]
fn plain_tooltip_matches_material_layout_a11y_and_motion() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(720.0), Px(420.0)),
    );
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                TooltipProvider::new()
                    .delay_duration_frames(0)
                    .skip_delay_duration_frames(0)
                    .with_elements(cx, |cx| {
                        let trigger = Button::new("Trigger")
                            .variant(ButtonVariant::Outlined)
                            .test_id("m3-tooltip-trigger")
                            .into_element(cx);
                        let probe = oversized_content_probe(cx);
                        let tooltip = PlainTooltip::new(trigger, "Plain tooltip")
                            .content_element(probe)
                            .open_delay_frames(Some(0))
                            .close_delay_frames(Some(0))
                            .test_id("m3-tooltip")
                            .into_element(cx);
                        vec![with_padding(cx, Px(48.0), tooltip)]
                    })
            })
        };

    run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let hover_at = center_for_test_id(&ui, "m3-tooltip-trigger");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), hover_at),
    );

    let first_open_scene = run_until_tooltip_visible(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        |ui, app, services| render(ui, app, services),
    );
    assert!(
        scene_has_intermediate_overlay_motion(&first_open_scene),
        "expected plain tooltip to fade and scale on the first visible open frame"
    );

    let chrome = live_test_id_layout_bounds(&ui, &app, window, "m3-tooltip.chrome");
    assert!(
        chrome.size.width.0 <= 200.5,
        "expected plain tooltip max width to be 200dp, got {}",
        chrome.size.width.0
    );
    assert!(
        chrome.size.width.0 >= 39.5 && chrome.size.height.0 >= 23.5,
        "expected plain tooltip chrome to respect 40x24dp minimum, got {:?}",
        chrome.size
    );
    assert_material_tooltip_semantics(&ui, "m3-tooltip-trigger", "m3-tooltip");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
    );
    let first_close_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    assert!(
        scene_has_intermediate_overlay_motion(&first_close_scene),
        "expected plain tooltip to fade and scale on the first close frame"
    );
}

#[test]
fn rich_tooltip_matches_material_layout_and_a11y() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(760.0), Px(460.0)),
    );
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                TooltipProvider::new()
                    .delay_duration_frames(0)
                    .skip_delay_duration_frames(0)
                    .with_elements(cx, |cx| {
                        let trigger = Button::new("Rich trigger")
                            .variant(ButtonVariant::Outlined)
                            .test_id("m3-rich-tooltip-trigger")
                            .into_element(cx);
                        let probe = oversized_content_probe(cx);
                        let tooltip = RichTooltip::new(trigger, "Rich tooltip")
                            .content_element(probe)
                            .open_delay_frames(Some(0))
                            .close_delay_frames(Some(0))
                            .test_id("m3-rich-tooltip")
                            .into_element(cx);
                        vec![with_padding(cx, Px(48.0), tooltip)]
                    })
            })
        };

    run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let hover_at = center_for_test_id(&ui, "m3-rich-tooltip-trigger");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), hover_at),
    );

    let first_open_scene = run_until_tooltip_visible(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        |ui, app, services| render(ui, app, services),
    );
    assert!(
        scene_has_intermediate_overlay_motion(&first_open_scene),
        "expected rich tooltip to fade and scale on the first visible open frame"
    );

    let chrome = live_test_id_layout_bounds(&ui, &app, window, "m3-rich-tooltip.chrome");
    assert!(
        chrome.size.width.0 >= 319.5 && chrome.size.width.0 <= 320.5,
        "expected rich tooltip max width to be 320dp, got {}",
        chrome.size.width.0
    );
    assert!(
        chrome.size.height.0 >= 23.5,
        "expected rich tooltip chrome to respect 24dp minimum height, got {:?}",
        chrome.size
    );
    assert_material_tooltip_semantics(&ui, "m3-rich-tooltip-trigger", "m3-rich-tooltip");
}

#[test]
fn plain_tooltip_style_overrides_paint_and_layout_contract() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let background = Color {
        r: 0.09,
        g: 0.14,
        b: 0.19,
        a: 1.0,
    };
    let text = Color {
        r: 0.92,
        g: 0.82,
        b: 0.38,
        a: 1.0,
    };
    let style = TooltipStyle::default()
        .plain_container_background(WidgetStateProperty::new(Some(ColorRef::Color(background))))
        .plain_supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(text))))
        .plain_container_padding(WidgetStateProperty::new(Some(Edges {
            left: Px(18.0),
            right: Px(18.0),
            top: Px(10.0),
            bottom: Px(10.0),
        })))
        .plain_container_corner_radius(WidgetStateProperty::new(Some(Px(10.0))))
        .plain_container_max_width(WidgetStateProperty::new(Some(Px(128.0))))
        .container_min_height(WidgetStateProperty::new(Some(Px(48.0))));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(720.0), Px(420.0)),
    );
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let style = style.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                TooltipProvider::new()
                    .delay_duration_frames(0)
                    .skip_delay_duration_frames(0)
                    .with_elements(cx, |cx| {
                        let trigger = Button::new("Styled plain")
                            .variant(ButtonVariant::Outlined)
                            .test_id("m3-plain-style-trigger")
                            .into_element(cx);
                        let tooltip = PlainTooltip::new(
                            trigger,
                            "Styled plain tooltip copy wraps inside a custom max width.",
                        )
                        .style(style)
                        .open_delay_frames(Some(0))
                        .close_delay_frames(Some(0))
                        .test_id("m3-plain-style")
                        .into_element(cx);
                        vec![with_padding(cx, Px(64.0), tooltip)]
                    })
            })
        };

    run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let hover_at = center_for_test_id(&ui, "m3-plain-style-trigger");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), hover_at),
    );

    let scene = run_until_tooltip_visible(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        |ui, app, services| render(ui, app, services),
    );

    assert!(
        scene_has_solid_quad_color(&scene, background),
        "expected TooltipStyle plain_container_background to paint tooltip chrome"
    );
    assert!(
        scene_has_solid_text_color(&scene, text),
        "expected TooltipStyle plain_supporting_text_color to paint tooltip text"
    );

    let chrome = live_test_id_layout_bounds(&ui, &app, window, "m3-plain-style.chrome");
    assert!(
        chrome.size.width.0 <= 128.5,
        "plain_container_max_width override should affect tooltip layout; bounds={chrome:?}"
    );
    assert!(
        chrome.size.height.0 >= 47.5,
        "container_min_height override should affect tooltip layout; bounds={chrome:?}"
    );
}

#[test]
fn rich_tooltip_style_overrides_paint_parts_and_layout_contract() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let background = Color {
        r: 0.18,
        g: 0.13,
        b: 0.22,
        a: 1.0,
    };
    let title = Color {
        r: 0.96,
        g: 0.78,
        b: 0.46,
        a: 1.0,
    };
    let supporting = Color {
        r: 0.82,
        g: 0.90,
        b: 0.98,
        a: 1.0,
    };
    let style = TooltipStyle::default()
        .rich_container_background(WidgetStateProperty::new(Some(ColorRef::Color(background))))
        .rich_title_color(WidgetStateProperty::new(Some(ColorRef::Color(title))))
        .rich_supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(supporting))))
        .rich_container_padding(WidgetStateProperty::new(Some(Edges {
            left: Px(20.0),
            right: Px(20.0),
            top: Px(14.0),
            bottom: Px(18.0),
        })))
        .rich_container_corner_radius(WidgetStateProperty::new(Some(Px(18.0))))
        .rich_container_max_width(WidgetStateProperty::new(Some(Px(184.0))))
        .rich_text_gap(WidgetStateProperty::new(Some(Px(12.0))))
        .container_min_height(WidgetStateProperty::new(Some(Px(84.0))));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(760.0), Px(460.0)),
    );
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let style = style.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                TooltipProvider::new()
                    .delay_duration_frames(0)
                    .skip_delay_duration_frames(0)
                    .with_elements(cx, |cx| {
                        let trigger = Button::new("Styled rich")
                            .variant(ButtonVariant::Outlined)
                            .test_id("m3-rich-style-trigger")
                            .into_element(cx);
                        let tooltip = RichTooltip::new(
                            trigger,
                            "Rich tooltip supporting text wraps inside the styled container.",
                        )
                        .title("Rich style")
                        .style(style)
                        .open_delay_frames(Some(0))
                        .close_delay_frames(Some(0))
                        .test_id("m3-rich-style")
                        .into_element(cx);
                        vec![with_padding(cx, Px(64.0), tooltip)]
                    })
            })
        };

    run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let hover_at = center_for_test_id(&ui, "m3-rich-style-trigger");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), hover_at),
    );

    let scene = run_until_tooltip_visible(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        |ui, app, services| render(ui, app, services),
    );

    assert!(
        scene_has_solid_quad_color(&scene, background),
        "expected TooltipStyle rich_container_background to paint tooltip chrome"
    );
    assert!(
        scene_has_solid_text_color(&scene, title),
        "expected TooltipStyle rich_title_color to paint the title"
    );
    assert!(
        scene_has_solid_text_color(&scene, supporting),
        "expected TooltipStyle rich_supporting_text_color to paint supporting text"
    );

    let chrome = live_test_id_layout_bounds(&ui, &app, window, "m3-rich-style.chrome");
    assert!(
        chrome.size.width.0 <= 184.5,
        "rich_container_max_width override should affect tooltip layout; bounds={chrome:?}"
    );
    assert!(
        chrome.size.height.0 >= 83.5,
        "container_min_height override should affect rich tooltip layout; bounds={chrome:?}"
    );

    let title_bounds = live_test_id_layout_bounds(&ui, &app, window, "m3-rich-style.title");
    let supporting_bounds =
        live_test_id_layout_bounds(&ui, &app, window, "m3-rich-style.supporting-text");
    assert!(
        supporting_bounds.origin.y.0 > title_bounds.origin.y.0,
        "rich tooltip title and supporting-text parts should keep vertical order"
    );
}
