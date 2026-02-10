#![cfg(feature = "web-goldens")]
// Heavy, web-golden-backed conformance. Enable via:
//   cargo nextest run -p fret-ui-shadcn --features web-goldens

use fret_app::App;
use fret_core::{
    AppWindowId, Color, Event, FrameId, KeyCode, Modifiers, MouseButton, MouseButtons, Paint,
    Point, PointerEvent, PointerType, Px, Rect, Scene, SceneOp, SemanticsRole, Size as CoreSize,
    Transform2D,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::elements::{GlobalElementId, bounds_for_element, with_element_cx};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use serde::Deserialize;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

mod css_color;
use css_color::{color_to_rgba, parse_css_color};

#[path = "web_vs_fret_overlay_chrome/web.rs"]
mod web;
use web::*;

#[path = "web_vs_fret_overlay_chrome/support.rs"]
mod support;
use support::*;

#[path = "web_vs_fret_overlay_chrome/alert_dialog.rs"]
mod alert_dialog;
#[path = "web_vs_fret_overlay_chrome/button_group.rs"]
mod button_group;
#[path = "web_vs_fret_overlay_chrome/calendar.rs"]
mod calendar;
#[path = "web_vs_fret_overlay_chrome/combobox.rs"]
mod combobox;
#[path = "web_vs_fret_overlay_chrome/command_dialog.rs"]
mod command_dialog;
#[path = "web_vs_fret_overlay_chrome/context_menu.rs"]
mod context_menu;
#[path = "web_vs_fret_overlay_chrome/date_picker.rs"]
mod date_picker;
#[path = "web_vs_fret_overlay_chrome/dialog.rs"]
mod dialog;
#[path = "web_vs_fret_overlay_chrome/drawer.rs"]
mod drawer;
#[path = "web_vs_fret_overlay_chrome/dropdown_menu.rs"]
mod dropdown_menu;
#[path = "web_vs_fret_overlay_chrome/hover_card.rs"]
mod hover_card;
#[path = "web_vs_fret_overlay_chrome/menubar.rs"]
mod menubar;
#[path = "web_vs_fret_overlay_chrome/navigation_menu.rs"]
mod navigation_menu;
#[path = "web_vs_fret_overlay_chrome/popover.rs"]
mod popover;
#[path = "web_vs_fret_overlay_chrome/select.rs"]
mod select;
#[path = "web_vs_fret_overlay_chrome/sheet.rs"]
mod sheet;
#[path = "web_vs_fret_overlay_chrome/tooltip.rs"]
mod tooltip;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

fn setup_app_with_shadcn_theme(app: &mut App) {
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}

fn setup_app_with_shadcn_theme_scheme(
    app: &mut App,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        scheme,
    );
}

fn render_frame<I, F>(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id: FrameId,
    request_semantics: bool,
    render: F,
) where
    F: FnOnce(&mut ElementContext<'_, App>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    app.set_frame_id(frame_id);
    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "web-vs-fret-overlay-chrome",
        render,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn paint_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    bounds: Rect,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
    (snap, scene)
}

fn bounds_for_viewport(viewport: WebViewport) -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(viewport.w), Px(viewport.h)),
    )
}

fn assert_close(label: &str, actual: f32, expected: f32, tol: f32) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected＞{expected} (㊣{tol}) got={actual} (忖={delta})"
    );
}

fn assert_rgba_close(label: &str, actual: css_color::Rgba, expected: css_color::Rgba, tol: f32) {
    assert_close(&format!("{label}.r"), actual.r, expected.r, tol);
    assert_close(&format!("{label}.g"), actual.g, expected.g, tol);
    assert_close(&format!("{label}.b"), actual.b, expected.b, tol);
    assert_close(&format!("{label}.a"), actual.a, expected.a, tol);
}

fn right_click_center(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    center: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: center,
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
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

fn left_click_center(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    center: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

fn dispatch_key_down(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    key: KeyCode,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::KeyDown {
            key,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
}

fn dispatch_key_up(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    key: KeyCode,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::KeyUp {
            key,
            modifiers: Modifiers::default(),
        },
    );
}

fn dispatch_key_press(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    key: KeyCode,
) {
    dispatch_key_down(ui, app, services, key);
    dispatch_key_up(ui, app, services, key);
}

fn leftish_text_probe_point(bounds: Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + 40.0),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

fn fret_find_active_listbox_option<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
) -> Option<&'a fret_core::SemanticsNode> {
    if let Some(focused) = snap
        .nodes
        .iter()
        .find(|n| n.flags.focused && n.role == SemanticsRole::ListBoxOption)
    {
        return Some(focused);
    }

    for owner in snap.nodes.iter().filter(|n| n.active_descendant.is_some()) {
        let active_id = owner.active_descendant?;
        let target = snap.nodes.iter().find(|n| n.id == active_id)?;
        if target.role == SemanticsRole::ListBoxOption {
            return Some(target);
        }
    }

    None
}

fn assert_overlay_chrome_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = find_portal_by_role(theme, web_portal_role).expect("web portal root by role");
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
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
        |cx| vec![build_frame1(cx, &open)],
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
        true,
        |cx| vec![build_frame2(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = largest_semantics_node(&snap, fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let mut quad =
        find_best_chrome_quad(&scene, overlay.bounds).expect("painted quad for overlay panel");
    if has_border(&web_border) && !has_border(&quad.border) {
        quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
            .unwrap_or_else(|| panic!("painted border quad for overlay panel ({web_name})"));
    }
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn assert_click_overlay_chrome_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_role: SemanticsRole,
    fret_trigger_role: SemanticsRole,
    fret_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = find_portal_by_role(theme, web_portal_role).expect("web portal root by role");
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_trigger_role && n.label.as_deref() == Some(fret_trigger_label))
        .unwrap_or_else(|| {
            panic!(
                "missing trigger semantics node: {fret_trigger_role:?} label={fret_trigger_label:?} for {web_name}"
            )
        });
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let build_frame2 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_frame2(cx)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = largest_semantics_node(&snap, fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let quad =
        find_best_chrome_quad(&scene, overlay.bounds).expect("painted quad for overlay panel");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn assert_click_overlay_surface_colors_match_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_trigger_role: SemanticsRole,
    fret_trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_trigger_role && n.label.as_deref() == Some(fret_trigger_label))
        .unwrap_or_else(|| {
            panic!(
                "missing trigger semantics node: {fret_trigger_role:?} label={fret_trigger_label:?} for {web_name}"
            )
        });
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_context_menu_chrome_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_role: SemanticsRole,
    trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = find_portal_by_role(theme, web_portal_role).expect("web portal root by role");
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
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
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let build_frame2 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_frame2(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let quad =
        find_best_chrome_quad(&scene, overlay.bounds).expect("painted quad for overlay panel");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn assert_navigation_menu_content_chrome_matches(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    open_value: &str,
    trigger_label: &str,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_content = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let web_border = web_border_widths_px(web_content).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_content).expect("web radius px");

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &model, &root_id_out)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &model, &root_id_out)],
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("missing fret navigation-menu content id for {open_value}"));

    let target = bounds_for_element(&mut app, window, content_id).unwrap_or_else(|| {
        panic!("missing fret bounds for navigation-menu content id {content_id:?}")
    });

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad =
        find_best_chrome_quad(&scene, target).expect("painted quad for navigation-menu content");

    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn assert_navigation_menu_content_surface_colors_match(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    open_value: &str,
    trigger_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_content = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let web_background = web_content
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_content).expect("web border widths px");
    let web_border_color = web_content
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &model, &root_id_out)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &model, &root_id_out)],
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-surface-colors",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("missing fret navigation-menu content id for {open_value}"));

    let target = bounds_for_element(&mut app, window, content_id).unwrap_or_else(|| {
        panic!("missing fret bounds for navigation-menu content id {content_id:?}")
    });

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad =
        find_best_chrome_quad(&scene, target).expect("painted quad for navigation-menu content");

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_navigation_menu_content_shadow_insets_match(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    open_value: &str,
    trigger_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_content = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let expected = web_drop_shadow_insets(web_content);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &model, &root_id_out)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    let _trigger_bounds = trigger.bounds;
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &model, &root_id_out)],
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-shadow-insets",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("missing fret navigation-menu content id for {open_value}"));

    let target = bounds_for_element(&mut app, window, content_id).unwrap_or_else(|| {
        panic!("missing fret bounds for navigation-menu content id {content_id:?}")
    });

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad =
        find_best_chrome_quad(&scene, target).expect("painted quad for navigation-menu content");

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    maybe_dump_shadow_candidates(
        &format!("{web_name} {web_theme_name} navigation-menu-content"),
        &expected,
        &candidates,
    );
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn assert_navigation_menu_viewport_shadow_insets_match(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    trigger_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_viewport = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let expected = web_drop_shadow_insets(web_viewport);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &model, &root_id_out)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    let _trigger_bounds = trigger.bounds;
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &model, &root_id_out)],
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let panel_id = with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-panel-id",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("missing fret navigation-menu viewport panel id");

    let target = bounds_for_element(&mut app, window, panel_id).unwrap_or_else(|| {
        panic!("missing fret bounds for navigation-menu viewport panel id {panel_id:?}")
    });

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad(&scene, target)
        .expect("painted quad for navigation-menu viewport panel");

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    maybe_dump_shadow_candidates(
        &format!("{web_name} {web_theme_name} navigation-menu-viewport"),
        &expected,
        &candidates,
    );
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn assert_navigation_menu_viewport_surface_colors_match(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    trigger_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_viewport = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let web_background = web_viewport
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_viewport).expect("web border widths px");
    let web_border_color = web_viewport
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &model, &root_id_out)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &model, &root_id_out)],
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let panel_id = with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-surface-colors",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("missing fret navigation-menu viewport panel id");

    let target = bounds_for_element(&mut app, window, panel_id).unwrap_or_else(|| {
        panic!("missing fret bounds for navigation-menu viewport panel id {panel_id:?}")
    });

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad(&scene, target)
        .expect("painted quad for navigation-menu viewport panel");

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} viewport_border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_navigation_menu_indicator_shadow_insets_match(
    web_name: &str,
    web_slot: &str,
    web_state: &str,
    trigger_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &Model<Option<Arc<str>>>,
        &Rc<Cell<Option<GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_indicator = find_by_data_slot_and_state(&theme.root, web_slot, web_state)
        .unwrap_or_else(|| panic!("missing web node data-slot={web_slot} data-state={web_state}"));
    let web_diamond = find_first(web_indicator, &|n| {
        let box_shadow = n
            .computed_style
            .get("boxShadow")
            .map(String::as_str)
            .unwrap_or("");
        !box_shadow.is_empty() && box_shadow != "none"
    })
    .expect("missing web indicator diamond node (expected non-empty boxShadow)");

    let expected = web_drop_shadow_insets(web_diamond);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &model, &root_id_out)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &model, &root_id_out)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let root_id = root_id_out.get().expect("navigation menu root id");
    let diamond_id = with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-indicator-diamond-id",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_indicator_diamond_id(
                cx, root_id,
            )
        },
    )
    .expect("missing fret navigation-menu indicator diamond id");
    let diamond_bounds = bounds_for_element(&mut app, window, diamond_id).unwrap_or_else(|| {
        panic!("missing fret bounds for navigation-menu indicator diamond id {diamond_id:?}")
    });
    let panel_rect = diamond_bounds;

    let candidates = fret_drop_shadow_insets_candidates(&scene, panel_rect);
    maybe_dump_shadow_candidates(
        &format!("{web_name} {web_theme_name} navigation-menu-indicator"),
        &expected,
        &candidates,
    );
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn find_best_chrome_quad_by_size(
    scene: &Scene,
    expected_w: f32,
    expected_h: f32,
    expected_border: [f32; 4],
) -> Option<PaintedQuad> {
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_paint,
            ..
        } = *op
        else {
            continue;
        };

        let background = paint_solid_color(background);
        let border_color = paint_solid_color(border_paint);
        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        let has_background = background.a > 0.01;
        if !has_background && !has_border(&border) {
            continue;
        }
        // Ignore drop-shadow layers when looking for "surface chrome".
        if !has_border(&border) && background.a < 0.5 {
            continue;
        }
        if has_border(&expected_border) && !has_border(&border) {
            continue;
        }

        let w = rect.size.width.0;
        let h = rect.size.height.0;
        let score = (w - expected_w).abs() + (h - expected_h).abs();
        if score < best_score {
            best_score = score;
            best = Some(PaintedQuad {
                rect,
                background,
                border,
                border_color,
                corners: [
                    corner_radii.top_left.0,
                    corner_radii.top_right.0,
                    corner_radii.bottom_right.0,
                    corner_radii.bottom_left.0,
                ],
            });
        }
    }

    best
}

fn assert_overlay_chrome_matches_by_portal_slot(
    web_name: &str,
    web_portal_slot: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = theme
        .portals
        .iter()
        .find(|n| {
            n.attrs
                .get("data-slot")
                .is_some_and(|v| v == web_portal_slot)
        })
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");
    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
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
        |cx| vec![build_frame1(cx, &open)],
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
        true,
        |cx| vec![build_frame2(cx, &open)],
    );

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .expect("painted quad for overlay panel");

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn assert_overlay_chrome_matches_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");
    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(web_w.max(640.0)), Px(web_h.max(480.0))),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
        |cx| vec![build_frame1(cx, &open)],
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .expect("painted quad for overlay panel");

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }

    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn assert_overlay_surface_colors_match(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));

    let bounds = web
        .themes
        .get(web_theme_name)
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(640.0), Px(480.0)),
            )
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
        |cx| vec![build_frame1(cx, &open)],
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for tick in 0..settle_frames.max(1) {
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames.max(1),
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = largest_semantics_node(&snap, fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));
    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn find_by_data_slot<'a>(node: &'a WebNode, slot: &str) -> Option<&'a WebNode> {
    find_first(node, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
    })
}

fn web_find_highlighted_menu_item_background(theme: &WebGoldenTheme) -> css_color::Rgba {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        let is_menuitem = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "menuitem");
        let is_item_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str().ends_with("-item"));
        if is_menuitem && is_item_slot {
            if let Some(bg) = node
                .computed_style
                .get("backgroundColor")
                .map(String::as_str)
                .and_then(parse_css_color)
                && bg.a > 0.01
            {
                out.push(node);
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    for portal in &theme.portals {
        collect(portal, &mut candidates);
    }
    let highlighted = candidates
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .expect("web highlighted menuitem (data-slot ends_with '-item')");
    highlighted
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web highlighted menuitem backgroundColor")
}

#[derive(Debug, Clone, Copy)]
struct WebHighlightedNodeChrome {
    bg: css_color::Rgba,
    fg: css_color::Rgba,
}

fn web_find_active_element<'a>(theme: &'a WebGoldenTheme) -> &'a WebNode {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(
        node: &'a WebNode,
        active_descendants: &mut Vec<&'a WebNode>,
        actives: &mut Vec<&'a WebNode>,
    ) {
        if node.active_descendant {
            active_descendants.push(node);
        }
        if node.active {
            actives.push(node);
        }
        for child in &node.children {
            collect(child, active_descendants, actives);
        }
    }

    let mut active_descendants: Vec<&WebNode> = Vec::new();
    let mut actives: Vec<&WebNode> = Vec::new();
    collect(&theme.root, &mut active_descendants, &mut actives);
    for portal in &theme.portals {
        collect(portal, &mut active_descendants, &mut actives);
    }

    if let Some(best) = active_descendants
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
    {
        return best;
    }

    actives
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .expect("web activeElement")
}

fn web_find_active_element_chrome(theme: &WebGoldenTheme) -> WebHighlightedNodeChrome {
    let active = web_find_active_element(theme);

    let bg = active
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web active element backgroundColor");
    let fg = active
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web active element color");

    WebHighlightedNodeChrome { bg, fg }
}

fn web_find_menu_item_chrome_by_slot_variant_and_text(
    theme: &WebGoldenTheme,
    item_slot: &str,
    variant: &str,
    text: &str,
) -> WebHighlightedNodeChrome {
    fn collect<'a>(
        node: &'a WebNode,
        item_slot: &str,
        variant: &str,
        text: &str,
        out: &mut Vec<&'a WebNode>,
    ) {
        let is_menuitem = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "menuitem");
        let is_item_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == item_slot);
        let is_variant = node
            .attrs
            .get("data-variant")
            .is_some_and(|v| v.as_str() == variant);
        let has_text = node.text.as_deref() == Some(text);
        if is_menuitem && is_item_slot && is_variant && has_text {
            out.push(node);
        }
        for child in &node.children {
            collect(child, item_slot, variant, text, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    collect(&theme.root, item_slot, variant, text, &mut candidates);
    for portal in &theme.portals {
        collect(portal, item_slot, variant, text, &mut candidates);
    }
    for wrapper in &theme.portal_wrappers {
        collect(wrapper, item_slot, variant, text, &mut candidates);
    }

    let node = candidates.first().copied().unwrap_or_else(|| {
        panic!("web menu item not found: slot={item_slot} variant={variant:?} text={text:?}")
    });

    let bg = node
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web menu item backgroundColor");
    let fg = node
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web menu item color");

    WebHighlightedNodeChrome { bg, fg }
}

fn web_find_highlighted_listbox_option_chrome(
    theme: &WebGoldenTheme,
    item_slot: &str,
) -> WebHighlightedNodeChrome {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(node: &'a WebNode, item_slot: &str, out: &mut Vec<&'a WebNode>) {
        let is_option = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "option");
        let is_item_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == item_slot);
        if is_option && is_item_slot {
            if let Some(bg) = node
                .computed_style
                .get("backgroundColor")
                .map(String::as_str)
                .and_then(parse_css_color)
                && bg.a > 0.01
            {
                out.push(node);
            }
        }
        for child in &node.children {
            collect(child, item_slot, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    for portal in &theme.portals {
        collect(portal, item_slot, &mut candidates);
    }
    let highlighted = candidates
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .unwrap_or_else(|| panic!("web highlighted option (data-slot={item_slot})"));

    let bg = highlighted
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web highlighted option backgroundColor");
    let fg = highlighted
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web highlighted option color");

    WebHighlightedNodeChrome { bg, fg }
}

fn assert_overlay_panel_size_matches_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = web
        .themes
        .get(web_theme_name)
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(640.0), Px(480.0)),
            )
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
        |cx| vec![build_frame1(cx, &open)],
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for tick in 0..settle_frames.max(1) {
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames.max(1),
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let overlay = largest_semantics_node(&snap, fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .or_else(|| find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border))
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if std::env::var("FRET_DEBUG_WEB_VS_FRET_OVERLAY_CHROME")
        .ok()
        .is_some_and(|v| v == "1")
    {
        eprintln!(
            "overlay chrome debug: web_name={web_name} role={:?} overlay_bounds=({}, {}, {}, {}) quad=({}, {}, {}, {}) web=({}, {})",
            fret_role,
            overlay.bounds.origin.x.0,
            overlay.bounds.origin.y.0,
            overlay.bounds.size.width.0,
            overlay.bounds.size.height.0,
            quad.rect.origin.x.0,
            quad.rect.origin.y.0,
            quad.rect.size.width.0,
            quad.rect.size.height.0,
            web_w,
            web_h
        );
    }

    assert_close(
        &format!("{web_name} {web_theme_name} panel.w"),
        quad.rect.size.width.0,
        web_w,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} panel.h"),
        quad.rect.size.height.0,
        web_h,
        1.0,
    );
}

fn assert_overlay_panel_size_matches_by_portal_slot_theme_with_tol(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    settle_frames: u64,
    tol: f32,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = web
        .themes
        .get(web_theme_name)
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(640.0), Px(480.0)),
            )
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
        |cx| vec![build_frame1(cx, &open)],
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for tick in 0..settle_frames.max(1) {
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames.max(1),
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let overlay = largest_semantics_node(&snap, fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .or_else(|| find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border))
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if std::env::var("FRET_DEBUG_WEB_VS_FRET_OVERLAY_CHROME")
        .ok()
        .is_some_and(|v| v == "1")
    {
        eprintln!(
            "overlay chrome debug: web_name={web_name} role={:?} overlay_bounds=({}, {}, {}, {}) quad=({}, {}, {}, {}) web=({}, {})",
            fret_role,
            overlay.bounds.origin.x.0,
            overlay.bounds.origin.y.0,
            overlay.bounds.size.width.0,
            overlay.bounds.size.height.0,
            quad.rect.origin.x.0,
            quad.rect.origin.y.0,
            quad.rect.size.width.0,
            quad.rect.size.height.0,
            web_w,
            web_h
        );
    }

    assert_close(
        &format!("{web_name} {web_theme_name} panel.w"),
        quad.rect.size.width.0,
        web_w,
        tol,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} panel.h"),
        quad.rect.size.height.0,
        web_h,
        tol,
    );
}

fn assert_overlay_panel_size_matches_by_portal_slot_theme_size_only(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
        |cx| vec![build_frame1(cx, &open)],
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for tick in 0..settle_frames.max(1) {
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames.max(1),
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    assert_close(
        &format!("{web_name} {web_theme_name} panel.w"),
        quad.rect.size.width.0,
        web_w,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} panel.h"),
        quad.rect.size.height.0,
        web_h,
        1.0,
    );
}

fn assert_overlay_shadow_insets_match(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let expected = web_drop_shadow_insets(web_portal);

    let bounds = web
        .themes
        .get(web_theme_name)
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(640.0), Px(480.0)),
            )
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
        |cx| vec![build_frame1(cx, &open)],
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for tick in 0..settle_frames.max(1) {
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames.max(1),
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = largest_semantics_node(&snap, fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));
    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_trigger_role: SemanticsRole,
    fret_trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let expected = web_drop_shadow_insets(web_portal);
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_trigger_role && n.label.as_deref() == Some(fret_trigger_label))
        .unwrap_or_else(|| {
            panic!(
                "missing trigger semantics node: {fret_trigger_role:?} label={fret_trigger_label:?} for {web_name}"
            )
        });
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn assert_click_overlay_panel_size_matches_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_trigger_role: SemanticsRole,
    fret_trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_trigger_role && n.label.as_deref() == Some(fret_trigger_label))
        .unwrap_or_else(|| {
            panic!(
                "missing trigger semantics node: {fret_trigger_role:?} label={fret_trigger_label:?} for {web_name}"
            )
        });
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    assert_close(
        &format!("{web_name} {web_theme_name} panel.w"),
        quad.rect.size.width.0,
        web_w,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} panel.h"),
        quad.rect.size.height.0,
        web_h,
        1.0,
    );
}

fn assert_context_menu_shadow_insets_match(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let expected = web_drop_shadow_insets(web_portal);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
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
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let build_frame2 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_frame2(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let overlay = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));
    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn assert_context_menu_panel_size_matches_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let _overlay = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    assert_close(
        &format!("{web_name} {web_theme_name} panel.w"),
        quad.rect.size.width.0,
        web_w,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} panel.h"),
        quad.rect.size.height.0,
        web_h,
        1.0,
    );
}

fn largest_semantics_node<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes.iter().filter(|n| n.role == role).max_by(|a, b| {
        let a_area = a.bounds.size.width.0 * a.bounds.size.height.0;
        let b_area = b.bounds.size.width.0 * b.bounds.size.height.0;
        a_area
            .partial_cmp(&b_area)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn fret_find_topmost_menu_item_in_menu<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    menu_bounds: Rect,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::MenuItem)
        .filter(|n| rect_intersection_area(n.bounds, menu_bounds) > 0.01)
        .min_by(|a, b| {
            let ay = a.bounds.origin.y.0;
            let by = b.bounds.origin.y.0;
            let ax = a.bounds.origin.x.0;
            let bx = b.bounds.origin.x.0;
            ay.total_cmp(&by).then_with(|| ax.total_cmp(&bx))
        })
}

fn hover_open_at(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    position: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
}

fn assert_hover_overlay_chrome_matches(
    web_name: &str,
    web_portal_slot: &str,
    fret_role: SemanticsRole,
    fret_trigger_label: &str,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = theme
        .portals
        .iter()
        .find(|n| {
            n.attrs
                .get("data-slot")
                .is_some_and(|v| v == web_portal_slot)
        })
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let trigger_id_out: std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &trigger_id_out)],
    );

    let frame1_snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger_semantics = frame1_snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(fret_trigger_label))
        .unwrap_or_else(|| {
            panic!(
                "missing trigger semantics node: Button label={fret_trigger_label:?} for {web_name}"
            )
        });
    let trigger_center = Point::new(
        Px(trigger_semantics.bounds.origin.x.0 + trigger_semantics.bounds.size.width.0 * 0.5),
        Px(trigger_semantics.bounds.origin.y.0 + trigger_semantics.bounds.size.height.0 * 0.5),
    );

    let trigger_element = trigger_id_out.get().expect("trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("trigger node");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::KeyA,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.set_focus(Some(trigger_node));
    hover_open_at(&mut ui, &mut app, &mut services, trigger_center);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &trigger_id_out)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let overlay = largest_semantics_node(&snap, fret_role).unwrap_or_else(|| {
        let mut roles: Vec<String> = snap.nodes.iter().map(|n| format!("{:?}", n.role)).collect();
        roles.sort();
        roles.dedup();
        panic!("missing fret semantics node: {fret_role:?}; roles={roles:?}");
    });

    let quad = find_best_chrome_quad(&scene, overlay.bounds).expect("painted quad for overlay");

    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

fn assert_hover_overlay_surface_colors_match_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    fret_trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let trigger_id_out: std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &trigger_id_out)],
    );

    let frame1_snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger_semantics = frame1_snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(fret_trigger_label))
        .unwrap_or_else(|| {
            panic!(
                "missing trigger semantics node: Button label={fret_trigger_label:?} for {web_name}"
            )
        });
    let trigger_center = Point::new(
        Px(trigger_semantics.bounds.origin.x.0 + trigger_semantics.bounds.size.width.0 * 0.5),
        Px(trigger_semantics.bounds.origin.y.0 + trigger_semantics.bounds.size.height.0 * 0.5),
    );

    let trigger_element = trigger_id_out.get().expect("trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("trigger node");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::KeyA,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.set_focus(Some(trigger_node));
    hover_open_at(&mut ui, &mut app, &mut services, trigger_center);

    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &trigger_id_out)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let overlay = largest_semantics_node(&snap, fret_role).unwrap_or_else(|| {
        let mut roles: Vec<String> = snap.nodes.iter().map(|n| format!("{:?}", n.role)).collect();
        roles.sort();
        roles.dedup();
        panic!("missing fret semantics node: {fret_role:?}; roles={roles:?}");
    });

    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .or_else(|| find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border))
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_hover_overlay_panel_size_matches_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    fret_trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    assert_hover_overlay_panel_size_matches_by_portal_slot_theme_ex(
        web_name,
        web_portal_slot,
        web_theme_name,
        scheme,
        fret_role,
        fret_trigger_label,
        settle_frames,
        true,
        build,
    );
}

fn assert_hover_overlay_panel_height_matches_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    fret_trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    assert_hover_overlay_panel_size_matches_by_portal_slot_theme_ex(
        web_name,
        web_portal_slot,
        web_theme_name,
        scheme,
        fret_role,
        fret_trigger_label,
        settle_frames,
        false,
        build,
    );
}

fn assert_hover_overlay_panel_size_matches_by_portal_slot_theme_ex(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    fret_trigger_label: &str,
    settle_frames: u64,
    check_width: bool,
    build: impl Fn(
        &mut ElementContext<'_, App>,
        &std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> AnyElement
    + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let trigger_id_out: std::rc::Rc<std::cell::Cell<Option<fret_ui::elements::GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &trigger_id_out)],
    );

    let frame1_snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger_semantics = frame1_snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(fret_trigger_label))
        .unwrap_or_else(|| {
            panic!(
                "missing trigger semantics node: Button label={fret_trigger_label:?} for {web_name}"
            )
        });
    let trigger_center = Point::new(
        Px(trigger_semantics.bounds.origin.x.0 + trigger_semantics.bounds.size.width.0 * 0.5),
        Px(trigger_semantics.bounds.origin.y.0 + trigger_semantics.bounds.size.height.0 * 0.5),
    );

    let trigger_element = trigger_id_out.get().expect("trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("trigger node");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::KeyA,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.set_focus(Some(trigger_node));
    hover_open_at(&mut ui, &mut app, &mut services, trigger_center);

    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &trigger_id_out)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let overlay = largest_semantics_node(&snap, fret_role).unwrap_or_else(|| {
        let mut roles: Vec<String> = snap.nodes.iter().map(|n| format!("{:?}", n.role)).collect();
        roles.sort();
        roles.dedup();
        panic!("missing fret semantics node: {fret_role:?}; roles={roles:?}");
    });

    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .or_else(|| find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border))
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if check_width {
        assert_close(
            &format!("{web_name} {web_theme_name} panel.w"),
            quad.rect.size.width.0,
            web_w,
            1.0,
        );
    }
    assert_close(
        &format!("{web_name} {web_theme_name} panel.h"),
        quad.rect.size.height.0,
        web_h,
        1.0,
    );
}

fn assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    hover_open_at(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(submenu_trigger.bounds),
    );

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    ui.set_focus(Some(submenu_trigger.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

fn assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let expected = web_drop_shadow_insets(web_portal);
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    ui.set_focus(Some(submenu_trigger.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

fn assert_menu_subcontent_panel_size_matches_by_portal_slot_theme_keyboard_submenu(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    ui.set_focus(Some(submenu_trigger.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    assert_close(
        &format!("{web_name} {web_theme_name} panel.w"),
        quad.rect.size.width.0,
        web_w,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} panel.h"),
        quad.rect.size.height.0,
        web_h,
        1.0,
    );
}

fn web_find_open_menu_subtrigger_chrome(
    theme: &WebGoldenTheme,
    subtrigger_slot: &str,
    subtrigger_text: &str,
) -> WebHighlightedNodeChrome {
    fn collect<'a>(
        node: &'a WebNode,
        subtrigger_slot: &str,
        subtrigger_text: &str,
        out: &mut Vec<&'a WebNode>,
    ) {
        let is_menuitem = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "menuitem");
        let is_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == subtrigger_slot);
        let is_open = node
            .attrs
            .get("data-state")
            .is_some_and(|v| v.as_str() == "open");
        let has_text = node.text.as_deref() == Some(subtrigger_text);
        if is_menuitem && is_slot && is_open && has_text {
            out.push(node);
        }
        for child in &node.children {
            collect(child, subtrigger_slot, subtrigger_text, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    collect(
        &theme.root,
        subtrigger_slot,
        subtrigger_text,
        &mut candidates,
    );
    for portal in &theme.portals {
        collect(portal, subtrigger_slot, subtrigger_text, &mut candidates);
    }
    for wrapper in &theme.portal_wrappers {
        collect(wrapper, subtrigger_slot, subtrigger_text, &mut candidates);
    }

    let node = candidates.first().copied().unwrap_or_else(|| {
        panic!("web menu subtrigger not found: {subtrigger_slot} text={subtrigger_text:?}")
    });

    let bg = node
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web subtrigger backgroundColor");
    let fg = node
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web subtrigger color");

    WebHighlightedNodeChrome { bg, fg }
}

fn assert_menu_subtrigger_open_chrome_matches_web_by_portal_slot_theme_keyboard_submenu(
    web_name: &str,
    web_subtrigger_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected =
        web_find_open_menu_subtrigger_chrome(theme, web_subtrigger_slot, submenu_trigger_label);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    ui.set_focus(Some(submenu_trigger.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });

    let quad = find_best_solid_quad_within_matching_bg(&scene, trigger.bounds, expected.bg)
        .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: subtrigger open background quad"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} subtrigger open background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        trigger.bounds,
        leftish_text_probe_point(trigger.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: subtrigger open text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} subtrigger open text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_command_dialog_focused_item_chrome_matches_web(web_theme_name: &str) {
    assert_command_dialog_focused_item_chrome_matches_web_named(
        "command-dialog.focus-first",
        web_theme_name,
    );
}

fn assert_command_dialog_focused_item_chrome_matches_web_named(
    web_name: &str,
    web_theme_name: &str,
) {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_listbox_option_chrome(theme, "command-item");

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    let scheme = match web_theme_name {
        "dark" => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        _ => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    };
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            vec![
                CommandDialog::new(open.clone(), query, items)
                    .into_element(cx, |cx| Button::new("Open").into_element(cx)),
            ]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| {
                #[derive(Default)]
                struct Models {
                    query: Option<Model<String>>,
                }

                let existing = cx.with_state(Models::default, |st| st.query.clone());
                let query = if let Some(existing) = existing {
                    existing
                } else {
                    let model = cx.app.models_mut().insert(String::new());
                    cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                    model
                };

                let items = vec![
                    CommandItem::new("Calendar"),
                    CommandItem::new("Search Emoji"),
                    CommandItem::new("Calculator"),
                ];

                vec![
                    CommandDialog::new(open.clone(), query, items)
                        .into_element(cx, |cx| Button::new("Open").into_element(cx)),
                ]
            },
        );
    }

    let (snap, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    if let Some(text_field) = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::TextField)
        .max_by(|a, b| rect_area(a.bounds).total_cmp(&rect_area(b.bounds)))
    {
        ui.set_focus(Some(text_field.id));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + settle_frames),
            true,
            |cx| {
                #[derive(Default)]
                struct Models {
                    query: Option<Model<String>>,
                }

                let existing = cx.with_state(Models::default, |st| st.query.clone());
                let query = if let Some(existing) = existing {
                    existing
                } else {
                    let model = cx.app.models_mut().insert(String::new());
                    cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                    model
                };

                let items = vec![
                    CommandItem::new("Calendar"),
                    CommandItem::new("Search Emoji"),
                    CommandItem::new("Calculator"),
                ];

                vec![
                    CommandDialog::new(open.clone(), query, items)
                        .into_element(cx, |cx| Button::new("Open").into_element(cx)),
                ]
            },
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3 + settle_frames),
        true,
        |cx| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            vec![
                CommandDialog::new(open.clone(), query, items)
                    .into_element(cx, |cx| Button::new("Open").into_element(cx)),
            ]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let option = fret_find_active_listbox_option(&snap).unwrap_or_else(|| {
        let focused_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.flags.focused)
            .map(|n| n.role)
            .collect();
        let active_owner_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.active_descendant.is_some())
            .map(|n| n.role)
            .collect();
        panic!(
            "{web_name} {web_theme_name}: expected active listbox option\n  focused_roles={focused_roles:?}\n  active_descendant_owner_roles={active_owner_roles:?}"
        )
    });

    let quad = find_best_solid_quad_within_matching_bg(&scene, option.bounds, expected.bg)
        .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option background quad"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        option.bounds,
        leftish_text_probe_point(option.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_dropdown_menu_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    DropdownMenu::new(open.clone())
        // new-york-v4 dropdown-menu-demo: `DropdownMenuContent className="w-56"`.
        .min_width(Px(224.0))
        .into_element(
            cx,
            |cx| {
                Button::new("Open")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| {
                vec![
                    DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Profile")
                            .trailing(DropdownMenuShortcut::new("⇧⌘P").into_element(cx)),
                    ),
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Billing")
                            .trailing(DropdownMenuShortcut::new("⌘B").into_element(cx)),
                    ),
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Settings")
                            .trailing(DropdownMenuShortcut::new("⌘S").into_element(cx)),
                    ),
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Keyboard shortcuts")
                            .trailing(DropdownMenuShortcut::new("⌘K").into_element(cx)),
                    ),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                    DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(vec![
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                    ])),
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("New Team")
                            .trailing(DropdownMenuShortcut::new("⌘+T").into_element(cx)),
                    ),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                    DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                    DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Log out")
                            .trailing(DropdownMenuShortcut::new("⇧⌘Q").into_element(cx)),
                    ),
                ]
            },
        )
}

fn render_dropdown_menu_demo_settled(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id_base: u64,
    open: &Model<bool>,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            FrameId(frame_id_base + tick),
            tick + 1 == settle_frames,
            |cx| vec![build_shadcn_dropdown_menu_demo(cx, open)],
        );
    }
    paint_frame(ui, app, services, bounds)
}

fn assert_dropdown_menu_highlighted_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_menu_item_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let (snap2, _) = render_dropdown_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
    );

    let menu = largest_semantics_node(&snap2, SemanticsRole::Menu)
        .expect("dropdown-menu menu semantics node (largest)");
    let item = fret_find_topmost_menu_item_in_menu(&snap2, menu.bounds)
        .expect("dropdown-menu first menu item semantics node");

    hover_open_at(&mut ui, &mut app, &mut services, bounds_center(item.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let menu = largest_semantics_node(&snap, SemanticsRole::Menu)
        .expect("dropdown-menu menu semantics node (largest)");
    let item = fret_find_topmost_menu_item_in_menu(&snap, menu.bounds)
        .expect("dropdown-menu first menu item semantics node after hover");

    let quad = find_best_solid_quad_within_matching_bg(&scene, item.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text =
        find_best_text_color_near(&scene, item.bounds, leftish_text_probe_point(item.bounds))
            .unwrap_or_else(|| {
                panic!("{web_name} {web_theme_name}: highlighted menu item text color")
            });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_dropdown_menu_focused_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let (snap2, _) = render_dropdown_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
    );

    let menu = largest_semantics_node(&snap2, SemanticsRole::Menu)
        .expect("dropdown-menu menu semantics node (largest)");
    let item = fret_find_topmost_menu_item_in_menu(&snap2, menu.bounds)
        .expect("dropdown-menu first menu item semantics node");

    ui.set_focus(Some(item.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let menu = largest_semantics_node(&snap, SemanticsRole::Menu)
        .expect("dropdown-menu menu semantics node (largest)");
    let fallback = fret_find_topmost_menu_item_in_menu(&snap, menu.bounds)
        .expect("dropdown-menu first menu item semantics node after focus");
    let focused = fret_find_active_menu_item(&snap).unwrap_or(fallback);
    let focused = focused;
    if focused.role != SemanticsRole::MenuItem {
        let focused_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.flags.focused)
            .map(|n| n.role)
            .collect();
        panic!(
            "{web_name} {web_theme_name}: expected focused menu item semantics node\n  focused_roles={focused_roles:?}"
        );
    }

    let quad = find_best_solid_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: focused menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused menu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_button_group_demo_dropdown_menu(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
    label_value: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui::UiHost;
    use fret_ui::element::{ContainerProps, LayoutStyle, Length};
    use fret_ui_shadcn::{
        Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
        DropdownMenuGroup, DropdownMenuItem, DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
    };

    fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(16.0));
                    layout.size.height = Length::Px(Px(16.0));
                    layout
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }

    DropdownMenu::new(open.clone())
        .align(DropdownMenuAlign::End)
        // new-york-v4 button-group-demo: `DropdownMenuContent className="w-52"`.
        .min_width(Px(208.0))
        .into_element(
            cx,
            |cx| {
                Button::new("More Options")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Icon)
                    .children([icon_stub(cx)])
                    .into_element(cx)
            },
            |cx| {
                vec![
                    DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Mark as Read").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Archive").leading(icon_stub(cx)),
                        ),
                    ])),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Snooze").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Add to Calendar").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Add to List").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Label As...")
                                .leading(icon_stub(cx))
                                .submenu(vec![DropdownMenuEntry::RadioGroup(
                                    DropdownMenuRadioGroup::new(label_value)
                                        .item(DropdownMenuRadioItemSpec::new(
                                            "personal", "Personal",
                                        ))
                                        .item(DropdownMenuRadioItemSpec::new("work", "Work"))
                                        .item(DropdownMenuRadioItemSpec::new("other", "Other")),
                                )]),
                        ),
                    ])),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Trash")
                                .leading(icon_stub(cx))
                                .variant(
                                fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                            ),
                        ),
                    ])),
                ]
            },
        )
}

fn assert_button_group_demo_dropdown_menu_destructive_item_idle_fg_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("button-group-demo");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_menu_item_chrome_by_slot_variant_and_text(
        theme,
        "dropdown-menu-item",
        "destructive",
        "Trash",
    );
    assert!(
        expected.bg.a < 0.02,
        "button-group-demo {web_theme_name}: expected destructive item bg to be transparent, got={expected:?}"
    );

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);
    let label_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("personal")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_button_group_demo_dropdown_menu(
                cx,
                &open,
                label_value.clone(),
            )]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| {
                vec![build_shadcn_button_group_demo_dropdown_menu(
                    cx,
                    &open,
                    label_value.clone(),
                )]
            },
        );
    }

    let (snap, _scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trash = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Trash"))
        .expect("button-group-demo: destructive Trash menu item semantics node");
    assert!(
        !trash.flags.focused,
        "button-group-demo {web_theme_name}: expected Trash to be idle (not focused)"
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| {
            vec![build_shadcn_button_group_demo_dropdown_menu(
                cx,
                &open,
                label_value.clone(),
            )]
        },
    );

    let (_, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let text =
        find_best_text_color_near(&scene, trash.bounds, leftish_text_probe_point(trash.bounds))
            .unwrap_or_else(|| {
                panic!("button-group-demo {web_theme_name}: destructive idle menu item text color")
            });
    assert_rgba_close(
        &format!("button-group-demo {web_theme_name} destructive idle menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_button_group_demo_dropdown_menu_destructive_focused_item_chrome_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("button-group-demo.destructive-focus");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);
    let label_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("personal")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_button_group_demo_dropdown_menu(
                cx,
                &open,
                label_value.clone(),
            )]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| {
                vec![build_shadcn_button_group_demo_dropdown_menu(
                    cx,
                    &open,
                    label_value.clone(),
                )]
            },
        );
    }

    let (snap2, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trash = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Trash"))
        .expect("button-group-demo: destructive Trash menu item semantics node");

    ui.set_focus(Some(trash.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| {
            vec![build_shadcn_button_group_demo_dropdown_menu(
                cx,
                &open,
                label_value.clone(),
            )]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let focused_fallback = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Trash"))
        .expect("button-group-demo: Trash menu item semantics node after focus");
    let focused = fret_find_active_menu_item(&snap).unwrap_or(focused_fallback);
    assert!(
        focused.role == SemanticsRole::MenuItem && focused.label.as_deref() == Some("Trash"),
        "button-group-demo.destructive-focus {web_theme_name}: expected focused menu item to be Trash, got role={:?} label={:?}",
        focused.role,
        focused.label
    );

    let quad = find_best_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!(
                "button-group-demo.destructive-focus {web_theme_name}: destructive focused menu item background quad"
            )
        });
    assert_rgba_close(
        &format!(
            "button-group-demo.destructive-focus {web_theme_name} destructive focused menu item background"
        ),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "button-group-demo.destructive-focus {web_theme_name}: destructive focused menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "button-group-demo.destructive-focus {web_theme_name} destructive focused menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_context_menu_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
    checked_bookmarks: Model<bool>,
    checked_full_urls: Model<bool>,
    radio_person: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonVariant, ContextMenu, ContextMenuCheckboxItem, ContextMenuEntry,
        ContextMenuItem, ContextMenuLabel, ContextMenuRadioGroup, ContextMenuRadioItemSpec,
        ContextMenuShortcut,
    };

    ContextMenu::new(open.clone())
        // new-york-v4 context-menu-demo: `ContextMenuContent className="w-52"`.
        .min_width(Px(208.0))
        // new-york-v4 context-menu-demo: `ContextMenuSubContent className="w-44"`.
        .submenu_min_width(Px(176.0))
        .into_element(
            cx,
            |cx| {
                Button::new("Right click here")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| {
                vec![
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Back")
                            .inset(true)
                            .trailing(ContextMenuShortcut::new("⌘[").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Forward")
                            .inset(true)
                            .disabled(true)
                            .trailing(ContextMenuShortcut::new("⌘]").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Reload")
                            .inset(true)
                            .trailing(ContextMenuShortcut::new("⌘R").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").inset(true).submenu(
                        vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Delete").variant(
                                fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                            )),
                        ],
                    )),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                        checked_bookmarks,
                        "Show Bookmarks",
                    )),
                    ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                        checked_full_urls,
                        "Show Full URLs",
                    )),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::Label(ContextMenuLabel::new("People").inset(true)),
                    ContextMenuEntry::RadioGroup(
                        ContextMenuRadioGroup::new(radio_person)
                            .item(ContextMenuRadioItemSpec::new("pedro", "Pedro Duarte"))
                            .item(ContextMenuRadioItemSpec::new("colm", "Colm Tuite")),
                    ),
                ]
            },
        )
}

fn build_shadcn_context_menu_demo_stateful(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
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

    let (checked_bookmarks, checked_full_urls, radio_person) = if let Some(existing) = existing {
        existing
    } else {
        let checked_bookmarks = cx.app.models_mut().insert(false);
        let checked_full_urls = cx.app.models_mut().insert(true);
        let radio_person = cx.app.models_mut().insert(Some(Arc::from("benoit")));

        cx.with_state(Models::default, |st| {
            st.checked_bookmarks = Some(checked_bookmarks.clone());
            st.checked_full_urls = Some(checked_full_urls.clone());
            st.radio_person = Some(radio_person.clone());
        });

        (checked_bookmarks, checked_full_urls, radio_person)
    };

    build_shadcn_context_menu_demo(cx, open, checked_bookmarks, checked_full_urls, radio_person)
}

fn render_context_menu_demo_settled(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id_base: u64,
    open: &Model<bool>,
    checked_bookmarks: Model<bool>,
    checked_full_urls: Model<bool>,
    radio_person: Model<Option<Arc<str>>>,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            FrameId(frame_id_base + tick),
            tick + 1 == settle_frames,
            |cx| {
                vec![build_shadcn_context_menu_demo(
                    cx,
                    open,
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                )]
            },
        );
    }
    paint_frame(ui, app, services, bounds)
}

fn assert_context_menu_highlighted_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_menu_item_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(false);
    let checked_full_urls: Model<bool> = app.models_mut().insert(true);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics node");

    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let (snap2, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );

    let back = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Back"))
        .expect("context-menu Back item semantics node");

    hover_open_at(&mut ui, &mut app, &mut services, bounds_center(back.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let back = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Back"))
        .expect("context-menu Back item semantics node after hover");

    let quad = find_best_solid_quad_within_matching_bg(&scene, back.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text =
        find_best_text_color_near(&scene, back.bounds, leftish_text_probe_point(back.bounds))
            .unwrap_or_else(|| {
                panic!("{web_name} {web_theme_name}: highlighted menu item text color")
            });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_context_menu_focused_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(false);
    let checked_full_urls: Model<bool> = app.models_mut().insert(true);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics node");

    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let (snap2, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );

    let back = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Back"))
        .expect("context-menu Back item semantics node");

    ui.set_focus(Some(back.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let focused = fret_find_active_menu_item(&snap).unwrap_or_else(|| {
        let focused_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.flags.focused)
            .map(|n| n.role)
            .collect();
        panic!(
            "{web_name} {web_theme_name}: expected focused menu item semantics node\n  focused_roles={focused_roles:?}"
        )
    });

    let quad = find_best_solid_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: focused menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused menu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_context_menu_submenu_highlighted_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_menu_item_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(375.0), Px(240.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(false);
    let checked_full_urls: Model<bool> = app.models_mut().insert(true);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics node");

    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let (snap2, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );

    let more_tools = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
        .expect("context-menu More Tools item semantics node");
    ui.set_focus(Some(more_tools.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let (snap3, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        200,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );
    let save_page = snap3
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page..."))
        .expect("context-menu submenu item (Save Page...) semantics node");

    ui.set_focus(Some(save_page.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(350),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let save_page = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page..."))
        .expect("context-menu submenu item (Save Page...) semantics node after focus");

    let quad = find_best_solid_quad_within_matching_bg(&scene, save_page.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted submenu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted submenu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        save_page.bounds,
        leftish_text_probe_point(save_page.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: highlighted submenu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted submenu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_context_menu_submenu_destructive_focused_item_chrome_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("context-menu-demo.submenu-kbd-delete-focus");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(false);
    let checked_full_urls: Model<bool> = app.models_mut().insert(true);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics node");

    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let (snap2, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );

    let more_tools = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
        .expect("context-menu More Tools item semantics node");
    ui.set_focus(Some(more_tools.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    // Settle the submenu open motion.
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(500 + tick),
            tick + 1 == settle_frames,
            |cx| {
                vec![build_shadcn_context_menu_demo(
                    cx,
                    &open,
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                )]
            },
        );
    }

    let snap3 = ui
        .semantics_snapshot()
        .expect("semantics snapshot after submenu open")
        .clone();
    let delete = snap3
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Delete"))
        .expect("context-menu submenu destructive Delete item semantics node");

    ui.set_focus(Some(delete.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(600),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let focused = fret_find_active_menu_item(&snap).unwrap_or_else(|| {
        panic!(
            "context-menu-demo.submenu-kbd-delete-focus {web_theme_name}: expected focused menu item semantics node"
        )
    });
    assert_eq!(
        focused.label.as_deref(),
        Some("Delete"),
        "context-menu-demo.submenu-kbd-delete-focus {web_theme_name}: focused menu item label"
    );

    let quad = find_best_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!(
                "context-menu-demo.submenu-kbd-delete-focus {web_theme_name}: destructive focused menu item background quad"
            )
        });
    assert_rgba_close(
        &format!(
            "context-menu-demo.submenu-kbd-delete-focus {web_theme_name} destructive focused menu item background"
        ),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "context-menu-demo.submenu-kbd-delete-focus {web_theme_name}: destructive focused menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "context-menu-demo.submenu-kbd-delete-focus {web_theme_name} destructive focused menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_context_menu_submenu_destructive_item_idle_fg_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("context-menu-demo.submenu-kbd");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_menu_item_chrome_by_slot_variant_and_text(
        theme,
        "context-menu-item",
        "destructive",
        "Delete",
    );
    assert!(
        expected.bg.a <= 0.01,
        "context-menu-demo.submenu-kbd {web_theme_name}: expected destructive item bg to be transparent, got={expected:?}"
    );

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(false);
    let checked_full_urls: Model<bool> = app.models_mut().insert(true);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics node");

    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let (snap2, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );

    let more_tools = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
        .expect("context-menu More Tools item semantics node");
    ui.set_focus(Some(more_tools.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    // Settle the submenu open motion.
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(500 + tick),
            tick + 1 == settle_frames,
            |cx| {
                vec![build_shadcn_context_menu_demo(
                    cx,
                    &open,
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                )]
            },
        );
    }

    let snap3 = ui
        .semantics_snapshot()
        .expect("semantics snapshot after submenu open")
        .clone();
    let delete = snap3
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Delete"))
        .expect("context-menu submenu destructive Delete item semantics node");
    assert!(
        !delete.flags.focused,
        "context-menu-demo.submenu-kbd {web_theme_name}: expected Delete to be idle (not focused)"
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(600),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (_, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let text = find_best_text_color_near(
        &scene,
        delete.bounds,
        leftish_text_probe_point(delete.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "context-menu-demo.submenu-kbd {web_theme_name}: destructive idle menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "context-menu-demo.submenu-kbd {web_theme_name} destructive idle menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_popover_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_core::Px;
    use fret_ui::Theme;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
    use fret_ui_shadcn::{Button, ButtonVariant, Popover, PopoverContent};

    Popover::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open popover")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let sm_px = theme.metric_required("font.size");
            let sm_line_height = theme.metric_required("font.line_height");
            let muted_fg = theme.color_required("muted.foreground");

            // popover-demo uses `h4.leading-none.font-medium` (line height = 16px).
            let title = ui::text(cx, "Dimensions")
                .text_size_px(sm_px)
                .line_height_px(Px(16.0))
                .font_medium()
                .nowrap()
                .into_element(cx);
            // popover-demo uses `p.text-sm.text-muted-foreground` (line height = 20px).
            let description = ui::text(cx, "Set the dimensions for the layer.")
                .text_size_px(sm_px)
                .line_height_px(sm_line_height)
                .text_color(ColorRef::Color(muted_fg))
                .into_element(cx);
            let header = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2),
                move |_cx| vec![title, description],
            );

            fn labeled_input_row<H: fret_ui::UiHost>(
                cx: &mut ElementContext<'_, H>,
                label: &str,
                value: &str,
            ) -> AnyElement {
                use fret_core::Px;
                use fret_ui_kit::declarative::stack;
                use fret_ui_kit::{LayoutRefinement, Space};
                use fret_ui_shadcn::{Input, Label};

                let label_el = Label::new(label).into_element(cx);
                let model = cx.app.models_mut().insert(value.to_string());
                let input_el = Input::new(model)
                    .a11y_label(label)
                    .refine_layout(LayoutRefinement::default().h_px(Px(32.0)).flex_grow(1.0))
                    .into_element(cx);

                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N4).items_center(),
                    move |_cx| vec![label_el, input_el],
                )
            }

            let rows = vec![
                labeled_input_row(cx, "Width", "100%"),
                labeled_input_row(cx, "Max. width", "300px"),
                labeled_input_row(cx, "Height", "25px"),
                labeled_input_row(cx, "Max. height", "none"),
            ];
            let fields = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2),
                move |_cx| rows,
            );

            PopoverContent::new([header, fields])
                .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                .into_element(cx)
        },
    )
}

fn build_shadcn_calendar_22_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::Month;

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(192.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Date of birth").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                let calendar = fret_ui_shadcn::Calendar::new(month, selected)
                    .week_start(time::Weekday::Sunday)
                    .disable_outside_days(false)
                    .into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn build_shadcn_calendar_23_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::Month;

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(224.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Select your stay").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                let calendar = fret_ui_shadcn::Calendar::new(month, selected)
                    .week_start(time::Weekday::Sunday)
                    .disable_outside_days(false)
                    .into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn hover_first_listbox_option(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
) {
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let listbox = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBox)
        .max_by(|a, b| {
            rect_area(ui.debug_node_bounds(a.id).unwrap_or(a.bounds))
                .total_cmp(&rect_area(ui.debug_node_bounds(b.id).unwrap_or(b.bounds)))
        })
        .expect("listbox");
    let listbox_bounds = ui.debug_node_bounds(listbox.id).unwrap_or(listbox.bounds);
    let mut option_candidates: Vec<(Rect, &fret_core::SemanticsNode)> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .map(|n| (ui.debug_node_bounds(n.id).unwrap_or(n.bounds), n))
        .collect();
    option_candidates.sort_by(|(a, _), (b, _)| {
        a.origin
            .y
            .0
            .total_cmp(&b.origin.y.0)
            .then_with(|| a.origin.x.0.total_cmp(&b.origin.x.0))
    });

    let option = option_candidates
        .iter()
        .find(|(bounds, _)| rect_contains(listbox_bounds, *bounds))
        .map(|(_, n)| *n)
        .unwrap_or_else(|| {
            let samples: Vec<Rect> = option_candidates.iter().take(8).map(|(b, _)| *b).collect();
            panic!(
                "listbox option\n  listbox_bounds={listbox_bounds:?}\n  first_option_bounds={samples:?}"
            )
        });
    let option_bounds = ui.debug_node_bounds(option.id).unwrap_or(option.bounds);

    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: bounds_center(option_bounds),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
}

fn build_shadcn_select_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let entries: Vec<SelectEntry> = vec![
        SelectGroup::new(vec![
            SelectLabel::new("Fruits").into(),
            SelectItem::new("apple", "Apple").into(),
            SelectItem::new("banana", "Banana").into(),
            SelectItem::new("blueberry", "Blueberry").into(),
            SelectItem::new("grapes", "Grapes").into(),
            SelectItem::new("pineapple", "Pineapple").into(),
        ])
        .into(),
    ];

    fret_ui_shadcn::Select::new(value, open.clone())
        .a11y_label("Select")
        .placeholder("Select a fruit")
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default().w_px(fret_ui_kit::MetricRef::Px(Px(180.0))),
        )
        .entries(entries)
        .into_element(cx)
}

fn build_shadcn_select_scrollable_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
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
            SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
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
            fret_ui_kit::LayoutRefinement::default().w_px(fret_ui_kit::MetricRef::Px(Px(280.0))),
        )
        .entries(entries)
        .into_element(cx)
}

fn build_shadcn_select_scrollable_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    build_shadcn_select_scrollable_page(cx, open)
}

fn build_shadcn_combobox_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Combobox, ComboboxItem};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let items = vec![
        ComboboxItem::new("apple", "Apple"),
        ComboboxItem::new("banana", "Banana"),
        ComboboxItem::new("blueberry", "Blueberry"),
        ComboboxItem::new("grapes", "Grapes"),
        ComboboxItem::new("pineapple", "Pineapple"),
    ];

    Combobox::new(value, open.clone())
        .a11y_label("Select a fruit")
        .width(Px(200.0))
        .items(items)
        .into_element(cx)
}

fn assert_listbox_highlighted_option_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    web_option_slot: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_listbox_option_chrome(theme, web_option_slot);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();
    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx, &open)],
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build(cx, &open)],
        );
    }

    hover_first_listbox_option(&mut ui, &mut app, &mut services);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2 + settle_frames),
        true,
        |cx| vec![build(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let listbox = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBox)
        .max_by(|a, b| {
            rect_area(ui.debug_node_bounds(a.id).unwrap_or(a.bounds))
                .total_cmp(&rect_area(ui.debug_node_bounds(b.id).unwrap_or(b.bounds)))
        })
        .expect("listbox");
    let listbox_bounds = ui.debug_node_bounds(listbox.id).unwrap_or(listbox.bounds);
    let option = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| {
            rect_contains(
                listbox_bounds,
                ui.debug_node_bounds(n.id).unwrap_or(n.bounds),
            )
        })
        .min_by(|a, b| {
            let a_bounds = ui.debug_node_bounds(a.id).unwrap_or(a.bounds);
            let b_bounds = ui.debug_node_bounds(b.id).unwrap_or(b.bounds);
            a_bounds
                .origin
                .y
                .0
                .total_cmp(&b_bounds.origin.y.0)
                .then_with(|| a_bounds.origin.x.0.total_cmp(&b_bounds.origin.x.0))
        })
        .expect("listbox option");
    let option_bounds = ui.debug_node_bounds(option.id).unwrap_or(option.bounds);

    let quad = find_best_solid_quad_within_matching_bg(&scene, option_bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted option background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted option background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(&scene, listbox_bounds, bounds_center(option_bounds))
        .unwrap_or_else(|| {
            let mut total_text = 0usize;
            let mut samples_raw: Vec<(f32, f32)> = Vec::new();
            let mut samples_tx: Vec<(f32, f32)> = Vec::new();
            scene_walk(&scene, |st, op| {
                let SceneOp::Text { origin, .. } = *op else {
                    return;
                };
                total_text += 1;
                if samples_raw.len() < 16 {
                    samples_raw.push((origin.x.0, origin.y.0));
                }
                if samples_tx.len() < 16 {
                    let p = st.transform.apply_point(origin);
                    samples_tx.push((p.x.0, p.y.0));
                }
            });
            panic!(
                "{web_name} {web_theme_name}: highlighted option text color (no text ops near)\n  total_text_ops={total_text}\n  sample_origins_raw={samples_raw:?}\n  sample_origins_tx={samples_tx:?}\n  listbox_bounds={listbox_bounds:?}\n  option_bounds={option_bounds:?}",
            )
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted option text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_command_dialog_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    #[derive(Default)]
    struct Models {
        query: Option<Model<String>>,
    }

    let existing = cx.with_state(Models::default, |st| st.query.clone());
    let query = if let Some(existing) = existing {
        existing
    } else {
        let model = cx.app.models_mut().insert(String::new());
        cx.with_state(Models::default, |st| st.query = Some(model.clone()));
        model
    };

    let items = vec![
        CommandItem::new("Calendar"),
        CommandItem::new("Search Emoji"),
        CommandItem::new("Calculator"),
    ];

    CommandDialog::new(open.clone(), query, items)
        .into_element(cx, |cx| Button::new("Open").into_element(cx))
}

fn assert_listbox_focused_option_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    web_option_slot: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
    a11y_label: &str,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_listbox_option_chrome(theme, web_option_slot);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();
    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx, &open)],
    );

    let (snap, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox && n.label.as_deref() == Some(a11y_label))
        .expect("trigger semantics (combobox) by a11y label");
    ui.set_focus(Some(trigger.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx, &open)],
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build(cx, &open)],
        );
    }

    let (mut snap, mut scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    if fret_find_active_listbox_option(&snap).is_none() {
        // If the trigger key path did not produce an active item (some pages open via click and
        // move focus into an inner text field), force the open state and drive ArrowDown on the
        // first text field inside the overlay.
        let _ = app.models_mut().update(&open, |v| *v = true);
        let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
        for tick in 0..settle_frames {
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                FrameId(3 + settle_frames + tick),
                tick + 1 == settle_frames,
                |cx| vec![build(cx, &open)],
            );
        }

        let (snap2, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
        if let Some(text_field) = snap2
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::TextField)
            .max_by(|a, b| rect_area(a.bounds).total_cmp(&rect_area(b.bounds)))
        {
            ui.set_focus(Some(text_field.id));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                FrameId(3 + settle_frames + settle_frames),
                true,
                |cx| vec![build(cx, &open)],
            );
            dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
        }

        (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    }

    let option = fret_find_active_listbox_option(&snap).unwrap_or_else(|| {
        let focused_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.flags.focused)
            .map(|n| n.role)
            .collect();
        let listbox_count = snap.nodes.iter().filter(|n| n.role == SemanticsRole::ListBox).count();
        let option_count = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::ListBoxOption)
            .count();
        let active_owner_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.active_descendant.is_some())
            .map(|n| n.role)
            .collect();
        panic!(
            "expected focused listbox option semantics node (or any active_descendant -> option)\n  listbox_count={listbox_count}\n  option_count={option_count}\n  focused_roles={focused_roles:?}\n  active_descendant_owner_roles={active_owner_roles:?}"
        )
    });

    let quad = find_best_solid_quad_within_matching_bg(&scene, option.bounds, expected.bg)
        .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option background quad"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        option.bounds,
        leftish_text_probe_point(option.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_hover_card_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_core::Px;
    use fret_ui::Theme;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
    use fret_ui_shadcn::{
        Avatar, AvatarFallback, AvatarImage, Button, ButtonVariant, HoverCard, HoverCardContent,
    };

    let theme = Theme::global(&*cx.app).clone();
    let sm_px = theme.metric_required("font.size");
    let sm_line_height = theme.metric_required("font.line_height");
    let xs_px = theme
        .metric_by_key("component.tooltip.text_px")
        .unwrap_or(Px((sm_px.0 - 2.0).max(10.0)));
    let xs_line_height = theme
        .metric_by_key("component.tooltip.line_height")
        .unwrap_or(Px((sm_line_height.0 - 4.0).max(12.0)));
    let muted_fg = theme.color_required("muted.foreground");

    let trigger_el = Button::new("@nextjs")
        .variant(ButtonVariant::Link)
        .into_element(cx);

    let avatar = Avatar::new([
        AvatarImage::maybe(None).into_element(cx),
        AvatarFallback::new("VC").into_element(cx),
    ])
    .into_element(cx);

    let heading = ui::text(cx, "@nextjs")
        .text_size_px(sm_px)
        .line_height_px(sm_line_height)
        .font_semibold()
        .into_element(cx);
    let body = ui::text(
        cx,
        "The React Framework – created and maintained by @vercel.",
    )
    .text_size_px(sm_px)
    .line_height_px(sm_line_height)
    .into_element(cx);
    let joined = ui::text(cx, "Joined December 2021")
        .text_size_px(xs_px)
        .line_height_px(xs_line_height)
        .text_color(ColorRef::Color(muted_fg))
        .into_element(cx);

    let text_block = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .layout(LayoutRefinement::default().w_px(Px(238.0))),
        move |_cx| vec![heading, body, joined],
    );

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .layout(LayoutRefinement::default().w_full()),
        move |_cx| vec![avatar, text_block],
    );

    let content_el = HoverCardContent::new([row])
        .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx);

    HoverCard::new(trigger_el, content_el)
        .open(Some(open.clone()))
        .into_element(cx)
}

fn build_shadcn_menubar_demo(cx: &mut ElementContext<'_, App>) -> AnyElement {
    use fret_ui_shadcn::{
        Menubar, MenubarCheckboxItem, MenubarEntry, MenubarItem, MenubarMenu, MenubarRadioGroup,
        MenubarRadioItemSpec, MenubarShortcut,
    };

    #[derive(Default)]
    struct Models {
        view_bookmarks_bar: Option<Model<bool>>,
        view_full_urls: Option<Model<bool>>,
        profile_value: Option<Model<Option<Arc<str>>>>,
    }

    let existing = cx.with_state(Models::default, |st| {
        match (
            st.view_bookmarks_bar.as_ref(),
            st.view_full_urls.as_ref(),
            st.profile_value.as_ref(),
        ) {
            (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
            _ => None,
        }
    });

    let (view_bookmarks_bar, view_full_urls, profile_value) = if let Some(existing) = existing {
        existing
    } else {
        let view_bookmarks_bar = cx.app.models_mut().insert(false);
        let view_full_urls = cx.app.models_mut().insert(true);
        let profile_value = cx.app.models_mut().insert(Some(Arc::from("benoit")));

        cx.with_state(Models::default, |st| {
            st.view_bookmarks_bar = Some(view_bookmarks_bar.clone());
            st.view_full_urls = Some(view_full_urls.clone());
            st.profile_value = Some(profile_value.clone());
        });

        (view_bookmarks_bar, view_full_urls, profile_value)
    };

    Menubar::new(vec![
        MenubarMenu::new("File").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("New Tab")
                    .test_id("menubar.file.new_tab")
                    .trailing(MenubarShortcut::new("⌘T").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("New Window")
                    .trailing(MenubarShortcut::new("⌘N").into_element(cx)),
            ),
            MenubarEntry::Item(MenubarItem::new("New Incognito Window").disabled(true)),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(
                MenubarItem::new("Share")
                    .test_id("menubar.file.share")
                    .submenu(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ]),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Print...").trailing(MenubarShortcut::new("⌘P").into_element(cx)),
            ),
        ]),
        MenubarMenu::new("Edit").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("Undo").trailing(MenubarShortcut::new("⌘Z").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Redo").trailing(MenubarShortcut::new("⇧⌘Z").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(MenubarItem::new("Find").submenu(vec![
                MenubarEntry::Item(MenubarItem::new("Search the web")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Find...")),
                MenubarEntry::Item(MenubarItem::new("Find Next")),
                MenubarEntry::Item(MenubarItem::new("Find Previous")),
            ])),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Cut")),
            MenubarEntry::Item(MenubarItem::new("Copy")),
            MenubarEntry::Item(MenubarItem::new("Paste")),
        ]),
        MenubarMenu::new("View").entries(vec![
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_bookmarks_bar,
                "Always Show Bookmarks Bar",
            )),
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_full_urls,
                "Always Show Full URLs",
            )),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Reload")
                    .inset(true)
                    .trailing(MenubarShortcut::new("⌘R").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Force Reload")
                    .disabled(true)
                    .inset(true)
                    .trailing(MenubarShortcut::new("⇧⌘R").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Toggle Fullscreen").inset(true)),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Hide Sidebar").inset(true)),
        ]),
        MenubarMenu::new("Profiles").entries(vec![
            MenubarEntry::RadioGroup(
                MenubarRadioGroup::new(profile_value)
                    .item(MenubarRadioItemSpec::new("andy", "Andy"))
                    .item(MenubarRadioItemSpec::new("benoit", "Benoit"))
                    .item(MenubarRadioItemSpec::new("Luis", "Luis")),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Edit...").inset(true)),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Add Profile...").inset(true)),
        ]),
    ])
    .into_element(cx)
}

fn web_find_highlighted_menu_item_chrome(theme: &WebGoldenTheme) -> WebHighlightedNodeChrome {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        let is_menuitem = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "menuitem");
        let is_item_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str().ends_with("-item"));
        if is_menuitem && is_item_slot {
            if let Some(bg) = node
                .computed_style
                .get("backgroundColor")
                .map(String::as_str)
                .and_then(parse_css_color)
                && bg.a > 0.01
            {
                out.push(node);
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    for portal in &theme.portals {
        collect(portal, &mut candidates);
    }

    let highlighted = candidates
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .expect("web highlighted menuitem (data-slot ends_with '-item')");

    let bg = highlighted
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web highlighted menuitem backgroundColor");
    let fg = highlighted
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web highlighted menuitem color");

    WebHighlightedNodeChrome { bg, fg }
}

fn fret_find_active_menu_item<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
) -> Option<&'a fret_core::SemanticsNode> {
    if let Some(focused) = snap
        .nodes
        .iter()
        .find(|n| n.flags.focused && n.role == SemanticsRole::MenuItem)
    {
        return Some(focused);
    }

    for owner in snap.nodes.iter().filter(|n| n.active_descendant.is_some()) {
        let active_id = owner.active_descendant?;
        let target = snap.nodes.iter().find(|n| n.id == active_id)?;
        if target.role == SemanticsRole::MenuItem {
            return Some(target);
        }
    }

    None
}

fn render_shadcn_menubar_demo_settled(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id_base: u64,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            FrameId(frame_id_base + tick),
            tick + 1 == settle_frames,
            |cx| vec![build_shadcn_menubar_demo(cx)],
        );
    }
    paint_frame(ui, app, services, bounds)
}

fn assert_menubar_focused_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    ui.set_focus(Some(file_trigger.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

    let (snap, scene) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 201);

    let new_tab = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .unwrap_or_else(|| {
            let focused_labels: Vec<&str> = snap
                .nodes
                .iter()
                .filter(|n| n.flags.focused)
                .filter_map(|n| n.label.as_deref())
                .collect();
            let menu_item_labels: Vec<&str> = snap
                .nodes
                .iter()
                .filter(|n| n.role == SemanticsRole::MenuItem)
                .filter_map(|n| n.label.as_deref())
                .collect();
            panic!(
                "{web_name} {web_theme_name}: expected menubar menu item 'New Tab'\n  focused_labels={focused_labels:?}\n  menu_item_labels={menu_item_labels:?}",
            )
        });

    let quad = find_best_solid_quad_within_matching_bg(&scene, new_tab.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: focused menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        new_tab.bounds,
        leftish_text_probe_point(new_tab.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused menu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_menubar_file_menu_destructive(
    cx: &mut ElementContext<'_, App>,
    new_tab_destructive: bool,
    new_window_destructive: bool,
) -> AnyElement {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu, MenubarShortcut};

    let mut new_tab =
        MenubarItem::new("New Tab").trailing(MenubarShortcut::new("⌘T").into_element(cx));
    if new_tab_destructive {
        new_tab = new_tab.variant(fret_ui_shadcn::menubar::MenubarItemVariant::Destructive);
    }

    let mut new_window =
        MenubarItem::new("New Window").trailing(MenubarShortcut::new("⌘N").into_element(cx));
    if new_window_destructive {
        new_window = new_window.variant(fret_ui_shadcn::menubar::MenubarItemVariant::Destructive);
    }

    Menubar::new(vec![MenubarMenu::new("File").entries(vec![
        MenubarEntry::Item(new_tab),
        MenubarEntry::Item(new_window),
        MenubarEntry::Item(MenubarItem::new("New Incognito Window").disabled(true)),
        MenubarEntry::Separator,
        MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
            MenubarEntry::Item(MenubarItem::new("Email link")),
            MenubarEntry::Item(MenubarItem::new("Messages")),
            MenubarEntry::Item(MenubarItem::new("Notes")),
        ])),
        MenubarEntry::Separator,
        MenubarEntry::Item(
            MenubarItem::new("Print...").trailing(MenubarShortcut::new("⌘P").into_element(cx)),
        ),
    ])])
    .into_element(cx)
}

fn render_shadcn_menubar_file_menu_settled(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id_base: u64,
    new_tab_destructive: bool,
    new_window_destructive: bool,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            FrameId(frame_id_base + tick),
            tick + 1 == settle_frames,
            |cx| {
                vec![build_shadcn_menubar_file_menu_destructive(
                    cx,
                    new_tab_destructive,
                    new_window_destructive,
                )]
            },
        );
    }
    paint_frame(ui, app, services, bounds)
}

fn assert_menubar_file_menu_destructive_item_idle_fg_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("menubar-demo.destructive-idle");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_menu_item_chrome_by_slot_variant_and_text(
        theme,
        "menubar-item",
        "destructive",
        "New Window ⌘N",
    );
    assert!(
        expected.bg.a < 0.02,
        "menubar-demo.destructive-idle {web_theme_name}: expected destructive item bg to be transparent, got={expected:?}"
    );

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_file_menu_destructive(cx, false, true)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap2, _) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        false,
        true,
    );

    let new_tab = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node");
    ui.set_focus(Some(new_tab.id));

    let (snap, scene) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        200,
        false,
        true,
    );

    let new_window = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Window"))
        .expect("menubar New Window item semantics node");
    assert!(
        !new_window.flags.focused,
        "menubar-demo.destructive-idle {web_theme_name}: expected New Window to be idle (not focused)"
    );

    let text = find_best_text_color_near(
        &scene,
        new_window.bounds,
        leftish_text_probe_point(new_window.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "menubar-demo.destructive-idle {web_theme_name}: destructive idle menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "menubar-demo.destructive-idle {web_theme_name} destructive idle menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_menubar_file_menu_destructive_focused_item_chrome_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("menubar-demo.destructive-focus-first");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_file_menu_destructive(cx, true, false)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap2, _) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        true,
        false,
    );

    let new_tab = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node");
    ui.set_focus(Some(new_tab.id));

    let (snap, scene) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        200,
        true,
        false,
    );

    let focused_fallback = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node after focus");
    let focused = fret_find_active_menu_item(&snap).unwrap_or(focused_fallback);
    assert!(
        focused.role == SemanticsRole::MenuItem && focused.label.as_deref() == Some("New Tab"),
        "menubar-demo.destructive-focus-first {web_theme_name}: expected focused menu item to be New Tab, got role={:?} label={:?}",
        focused.role,
        focused.label
    );

    let quad = find_best_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!(
                "menubar-demo.destructive-focus-first {web_theme_name}: destructive focused menu item background quad"
            )
        });
    assert_rgba_close(
        &format!(
            "menubar-demo.destructive-focus-first {web_theme_name} destructive focused menu item background"
        ),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "menubar-demo.destructive-focus-first {web_theme_name}: destructive focused menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "menubar-demo.destructive-focus-first {web_theme_name} destructive focused menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_menubar_highlighted_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_menu_item_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap, _) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 2);

    let new_tab = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node");

    hover_open_at(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(new_tab.bounds),
    );

    let (snap, scene) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 200);

    let new_tab = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node after hover");

    let quad = find_best_solid_quad_within_matching_bg(&scene, new_tab.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        new_tab.bounds,
        leftish_text_probe_point(new_tab.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: highlighted menu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_menubar_submenu_highlighted_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_menu_item_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(375.0), Px(240.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap2, _) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 2);

    let share = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
        .expect("menubar submenu trigger (Share) semantics node");
    ui.set_focus(Some(share.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let (snap3, _) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 200);
    let email_link = snap3
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("menubar submenu item (Email link) semantics node");
    ui.set_focus(Some(email_link.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(350),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let email_link = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("menubar submenu item (Email link) semantics node after focus");

    let quad = find_best_solid_quad_within_matching_bg(&scene, email_link.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted submenu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted submenu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        email_link.bounds,
        leftish_text_probe_point(email_link.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: highlighted submenu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted submenu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_navigation_menu_trigger_surface_colors_match(
    web_name: &str,
    open_label: &str,
    open_value: &str,
    closed_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_open_trigger = find_by_data_slot_and_state_and_text(
        &theme.root,
        "navigation-menu-trigger",
        "open",
        open_label,
    )
    .unwrap_or_else(|| {
        panic!(
            "missing web open trigger: slot=navigation-menu-trigger state=open text={open_label:?}"
        )
    });
    let web_closed_trigger = find_by_data_slot_and_state_and_text(
        &theme.root,
        "navigation-menu-trigger",
        "closed",
        closed_label,
    )
    .unwrap_or_else(|| {
        panic!(
            "missing web closed trigger: slot=navigation-menu-trigger state=closed text={closed_label:?}"
        )
    });

    let web_open_bg = web_open_trigger
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_open_text = web_open_trigger
        .computed_style
        .get("color")
        .and_then(|v| parse_css_color(v));

    let web_closed_bg = web_closed_trigger
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_closed_text = web_closed_trigger
        .computed_style
        .get("color")
        .and_then(|v| parse_css_color(v));

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![
                NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(vec![
                        NavigationMenuItem::new("home", "Home", vec![cx.text("Home content")]),
                        NavigationMenuItem::new(
                            "components",
                            "Components",
                            vec![cx.text("Components content")],
                        ),
                        NavigationMenuItem::new("list", "List", vec![cx.text("List content")]),
                    ])
                    .into_element(cx),
            ]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let open_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(open_label))
        .unwrap_or_else(|| panic!("missing fret trigger semantics node: Button {open_label:?}"));
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(open_trigger.bounds),
    );

    let _ = app
        .models_mut()
        .update(&model, |v| *v = Some(Arc::from(open_value)));

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
                vec![
                    NavigationMenu::new(model.clone())
                        .viewport(false)
                        .indicator(false)
                        .items(vec![
                            NavigationMenuItem::new("home", "Home", vec![cx.text("Home content")]),
                            NavigationMenuItem::new(
                                "components",
                                "Components",
                                vec![cx.text("Components content")],
                            ),
                            NavigationMenuItem::new("list", "List", vec![cx.text("List content")]),
                        ])
                        .into_element(cx),
                ]
            },
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let open_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(open_label))
        .unwrap_or_else(|| {
            panic!("missing fret trigger semantics node after open: {open_label:?}")
        });
    let closed_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(closed_label))
        .unwrap_or_else(|| {
            panic!("missing fret trigger semantics node after open: {closed_label:?}")
        });

    let open_quad = find_best_chrome_quad(&scene, open_trigger.bounds)
        .expect("painted quad for navigation-menu trigger chrome (open)");
    let closed_quad = find_best_chrome_quad(&scene, closed_trigger.bounds)
        .expect("painted quad for navigation-menu trigger chrome (closed)");

    if let Some(web_open_bg) = web_open_bg
        && web_open_bg.a > 0.01
    {
        let fret_bg = color_to_rgba(open_quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.r"),
            fret_bg.r,
            web_open_bg.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.g"),
            fret_bg.g,
            web_open_bg.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.b"),
            fret_bg.b,
            web_open_bg.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.a"),
            fret_bg.a,
            web_open_bg.a,
            0.02,
        );
    }

    if let Some(web_closed_bg) = web_closed_bg
        && web_closed_bg.a > 0.01
    {
        let fret_bg = color_to_rgba(closed_quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.r"),
            fret_bg.r,
            web_closed_bg.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.g"),
            fret_bg.g,
            web_closed_bg.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.b"),
            fret_bg.b,
            web_closed_bg.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.a"),
            fret_bg.a,
            web_closed_bg.a,
            0.02,
        );
    }

    if let Some(web_open_text) = web_open_text
        && web_open_text.a > 0.01
    {
        let text = find_best_text_color_near(
            &scene,
            open_trigger.bounds,
            bounds_center(open_trigger.bounds),
        )
        .expect("open trigger text color");
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.r"),
            text.r,
            web_open_text.r,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.g"),
            text.g,
            web_open_text.g,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.b"),
            text.b,
            web_open_text.b,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.a"),
            text.a,
            web_open_text.a,
            0.05,
        );
    }

    if let Some(web_closed_text) = web_closed_text
        && web_closed_text.a > 0.01
    {
        let text = find_best_text_color_near(
            &scene,
            closed_trigger.bounds,
            bounds_center(closed_trigger.bounds),
        )
        .expect("closed trigger text color");
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.r"),
            text.r,
            web_closed_text.r,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.g"),
            text.g,
            web_closed_text.g,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.b"),
            text.b,
            web_closed_text.b,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.a"),
            text.a,
            web_closed_text.a,
            0.05,
        );
    }
}
