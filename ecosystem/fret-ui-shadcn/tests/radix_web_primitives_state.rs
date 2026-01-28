use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, KeyCode, Modifiers, MouseButton, Point, PointerEvent, PointerType,
    Px, Rect, SemanticsRole, Size as CoreSize, UiServices,
};
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::ElementContext;
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, LayoutStyle, Length, SemanticsProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
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
    action: Action,
    snapshot: Snapshot,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
#[serde(tag = "kind")]
enum Action {
    #[serde(rename = "load")]
    Load { url: String },
    #[serde(rename = "click")]
    Click { target: String },
    #[serde(rename = "press")]
    Press { key: String },
    #[serde(rename = "hover")]
    Hover { target: String },
}

#[derive(Debug, Clone, Deserialize)]
struct Snapshot {
    focus: FocusNode,
    dom: DomNode,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct FocusNode {
    tag: String,
    path: Vec<usize>,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct DomNode {
    tag: String,
    path: Vec<usize>,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
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
            "missing radix web golden: {}\nerror: {err}\n\nDocs:\n  goldens/radix-web/README.md",
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

fn require_attr<'a>(attrs: &'a BTreeMap<String, String>, key: &str) -> &'a str {
    attrs
        .get(key)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("missing focus attr: {key}"))
}

fn parse_bool_attr(attrs: &BTreeMap<String, String>, key: &str) -> bool {
    match require_attr(attrs, key) {
        "true" => true,
        "false" => false,
        other => panic!("invalid bool attr {key}={other:?}"),
    }
}

fn parse_target_index(target: &str) -> (&str, usize) {
    let (kind, rest) = target
        .split_once('[')
        .unwrap_or_else(|| panic!("expected indexed target like tab[1], got {target:?}"));
    let idx_str = rest
        .strip_suffix(']')
        .unwrap_or_else(|| panic!("expected closing ] in target {target:?}"));
    let idx = idx_str
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("invalid target index in {target:?}"));
    (kind, idx)
}

fn parse_key_sequence(key: &str) -> Vec<KeyCode> {
    key.split(',')
        .map(|k| match k.trim() {
            "ArrowLeft" => KeyCode::ArrowLeft,
            "ArrowRight" => KeyCode::ArrowRight,
            "ArrowUp" => KeyCode::ArrowUp,
            "ArrowDown" => KeyCode::ArrowDown,
            "Home" => KeyCode::Home,
            "End" => KeyCode::End,
            "Enter" => KeyCode::Enter,
            "NumpadEnter" => KeyCode::NumpadEnter,
            "Escape" => KeyCode::Escape,
            "Space" => KeyCode::Space,
            "Tab" => KeyCode::Tab,
            other => panic!("unsupported key in radix web action: {other:?}"),
        })
        .collect()
}

fn dispatch_key_down(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn UiServices,
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
    services: &mut dyn UiServices,
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

fn dispatch_web_press(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn UiServices,
    key: KeyCode,
) {
    dispatch_key_down(ui, app, services, key);
    dispatch_key_up(ui, app, services, key);
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

fn render_frame<I, F>(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn UiServices,
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
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "radix-state", render);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn click_center(ui: &mut UiTree<App>, app: &mut App, services: &mut dyn UiServices, center: Point) {
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

fn right_click_center(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn UiServices,
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

fn click_outside(ui: &mut UiTree<App>, app: &mut App, services: &mut dyn UiServices, bounds: Rect) {
    let click = Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 - 5.0),
        Px(bounds.origin.y.0 + bounds.size.height.0 - 5.0),
    );
    click_center(ui, app, services, click);
}

fn bounds_center(r: Rect) -> Point {
    Point::new(
        Px(r.origin.x.0 + r.size.width.0 * 0.5),
        Px(r.origin.y.0 + r.size.height.0 * 0.5),
    )
}

fn find_semantics<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
    label: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.role == role && n.label.as_deref() == Some(label))
        .unwrap_or_else(|| panic!("missing semantics node role={role:?} label={label:?}"))
}

fn find_semantics_by_role<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.role == role)
        .unwrap_or_else(|| panic!("missing semantics node role={role:?}"))
}

fn assert_focus_cleared(ui: &UiTree<App>, snap: &fret_core::SemanticsSnapshot, context: &str) {
    let focused: Vec<String> = snap
        .nodes
        .iter()
        .filter(|n| n.flags.focused)
        .map(|n| {
            format!(
                "{:?}:{:?}",
                n.role,
                n.label.as_deref().unwrap_or("<unlabeled>")
            )
        })
        .collect();
    assert!(
        focused.is_empty(),
        "{context}: expected semantics focus to be cleared, got focused={focused:?} ui_focus={:?}",
        ui.focus()
    );
    assert_eq!(
        ui.focus(),
        None,
        "{context}: expected UiTree focus to be cleared"
    );
}

fn has_semantics_role(snap: &fret_core::SemanticsSnapshot, role: SemanticsRole) -> bool {
    snap.nodes.iter().any(|n| n.role == role)
}

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(800.0), Px(600.0)),
    )
}

fn move_pointer(ui: &mut UiTree<App>, app: &mut App, services: &mut dyn UiServices, point: Point) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
}

fn deliver_all_timers_from_effects(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn UiServices,
) {
    let effects = app.flush_effects();
    let mut timer_tokens = Vec::new();
    for effect in effects {
        match effect {
            Effect::SetTimer { token, .. } => timer_tokens.push(token),
            other => app.push_effect(other),
        }
    }
    for token in timer_tokens {
        ui.dispatch_event(app, services, &Event::Timer { token });
    }
}

#[derive(Default)]
struct TimerQueue {
    pending: Vec<(TimerToken, Duration)>,
}

impl TimerQueue {
    fn ingest_effects(&mut self, app: &mut App) {
        let effects = app.flush_effects();
        for effect in effects {
            match effect {
                Effect::SetTimer { token, after, .. } => self.pending.push((token, after)),
                Effect::CancelTimer { token } => self.pending.retain(|(t, _)| *t != token),
                other => app.push_effect(other),
            }
        }
    }

    fn fire_after(
        &mut self,
        after: Duration,
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
    ) {
        let mut fire = Vec::new();
        self.pending.retain(|(token, a)| {
            if *a == after {
                fire.push(*token);
                false
            } else {
                true
            }
        });
        for token in fire {
            ui.dispatch_event(app, services, &Event::Timer { token });
        }
    }

    fn fire_all(&mut self, ui: &mut UiTree<App>, app: &mut App, services: &mut dyn UiServices) {
        let fire: Vec<TimerToken> = self.pending.drain(..).map(|(t, _)| t).collect();
        for token in fire {
            ui.dispatch_event(app, services, &Event::Timer { token });
        }
    }
}

fn has_dom_node_attr(node: &DomNode, key: &str, value: &str) -> bool {
    if node.attrs.get(key).is_some_and(|v| v.as_str() == value) {
        return true;
    }
    node.children
        .iter()
        .any(|child| has_dom_node_attr(child, key, value))
}

#[test]
fn radix_web_alert_dialog_open_cancel_matches_fret() {
    let golden = read_timeline("alert-dialog-example.alert-dialog.open-cancel.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "alert-dialog");
    assert_eq!(golden.scenario, "open-cancel");
    assert!(golden.steps.len() >= 3);

    let open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "alert-dialog-trigger"))
        .expect("open step");
    assert!(
        has_dom_node_attr(&open_step.snapshot.dom, "role", "alertdialog"),
        "web expected role=alertdialog to be present after open"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::AlertDialog::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open AlertDialog")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |cx| fret_ui_shadcn::AlertDialogContent::new(vec![cx.text("Content")]).into_element(cx),
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open AlertDialog");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        has_semantics_role(&snap, SemanticsRole::AlertDialog),
        "expected alert dialog semantics after open"
    );
    assert!(app.models().get_copied(&open).unwrap_or(false));

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    assert!(!app.models().get_copied(&open).unwrap_or(false));
}

#[test]
fn radix_web_dialog_open_close_matches_fret() {
    let golden = read_timeline("dialog-example.dialog.open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "dialog");
    assert_eq!(golden.scenario, "open-close");
    assert!(golden.steps.len() >= 3);

    let open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "dialog-trigger"))
        .expect("open step");
    assert!(
        has_dom_node_attr(&open_step.snapshot.dom, "role", "dialog"),
        "web expected role=dialog to be present after open"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::Dialog::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open Dialog")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |cx| fret_ui_shadcn::DialogContent::new(vec![cx.text("Hello")]).into_element(cx),
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open Dialog");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        has_semantics_role(&snap, SemanticsRole::Dialog),
        "expected dialog semantics after open"
    );
    assert!(app.models().get_copied(&open).unwrap_or(false));

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    assert!(!app.models().get_copied(&open).unwrap_or(false));
}

#[test]
fn radix_web_dropdown_menu_open_navigate_select_matches_fret() {
    let golden = read_timeline("dropdown-menu-example.dropdown-menu.open-navigate-select.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "dropdown-menu");
    assert_eq!(golden.scenario, "open-navigate-select");
    assert!(golden.steps.len() >= 3);

    let press_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "ArrowDown,Enter"))
        .expect("press step");
    assert!(
        !has_dom_node_attr(
            &press_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-content"
        ),
        "web expected menu to be closed after ArrowDown,Enter"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |_cx| {
                vec![fret_ui_shadcn::DropdownMenuEntry::Group(
                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("My Account"),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Profile"),
                        ),
                    ]),
                )]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    assert!(app.models().get_copied(&open).unwrap_or(false));
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let focused_role = snap.nodes.iter().find(|n| n.flags.focused).map(|n| n.role);
    assert_eq!(
        focused_role,
        Some(SemanticsRole::Menu),
        "expected focus to move to the menu content after opening"
    );

    let Action::Press { key } = &press_step.action else {
        unreachable!("press step must be a press action");
    };
    let keys = parse_key_sequence(key);
    for (idx, key) in keys.into_iter().enumerate() {
        dispatch_web_press(&mut ui, &mut app, &mut services, key);
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + idx as u64),
            true,
            |cx| vec![build(cx, &open)],
        );

        if idx == 0 {
            let snap = ui
                .semantics_snapshot()
                .cloned()
                .expect("semantics snapshot");
            let focused_role = snap.nodes.iter().find(|n| n.flags.focused).map(|n| n.role);
            assert_eq!(
                focused_role,
                Some(SemanticsRole::MenuItem),
                "expected ArrowDown to focus the first menu item"
            );
        }
    }

    assert!(
        !app.models().get_copied(&open).unwrap_or(false),
        "selecting a dropdown-menu item should close the menu"
    );
}

#[test]
fn radix_web_context_menu_open_close_matches_fret() {
    let golden = read_timeline("context-menu-example.context-menu.context-open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "context-menu");
    assert_eq!(golden.scenario, "context-open-close");
    assert!(golden.steps.len() >= 3);

    let open_step = golden
        .steps
        .iter()
        .find(
            |s| matches!(&s.action, Action::Click { target } if target == "contextmenu:rightclick"),
        )
        .expect("open step");
    assert!(
        has_dom_node_attr(&open_step.snapshot.dom, "data-slot", "context-menu-content"),
        "web expected context menu content to be present after open"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
            |_cx| {
                vec![
                    fret_ui_shadcn::ContextMenuEntry::Item(fret_ui_shadcn::ContextMenuItem::new(
                        "Copy",
                    )),
                    fret_ui_shadcn::ContextMenuEntry::Item(fret_ui_shadcn::ContextMenuItem::new(
                        "Cut",
                    )),
                ]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Right click here");
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    assert!(app.models().get_copied(&open).unwrap_or(false));

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    assert!(!app.models().get_copied(&open).unwrap_or(false));

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert_focus_cleared(&ui, &snap, "context menu escape close");
}

#[test]
fn radix_web_menubar_open_navigate_close_matches_fret() {
    let golden = read_timeline("menubar-example.menubar.open-navigate-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "open-navigate-close");
    assert!(golden.steps.len() >= 3);

    let press_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "ArrowDown,Escape"))
        .expect("press step");
    assert!(
        !has_dom_node_attr(&press_step.snapshot.dom, "data-slot", "menubar-content"),
        "web expected menubar content to be closed after ArrowDown,Escape"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu};

        Menubar::new(vec![
            MenubarMenu::new("File").entries(vec![
                MenubarEntry::Group(MenubarGroup::new(vec![
                    MenubarEntry::Item(MenubarItem::new("Profile")),
                    MenubarEntry::Item(MenubarItem::new("Billing")),
                ])),
                MenubarEntry::Separator,
                MenubarEntry::Group(MenubarGroup::new(vec![MenubarEntry::Item(
                    MenubarItem::new("Settings"),
                )])),
            ]),
            MenubarMenu::new("Edit").entries(vec![MenubarEntry::Group(MenubarGroup::new(vec![
                MenubarEntry::Item(MenubarItem::new("Undo")),
                MenubarEntry::Item(MenubarItem::new("Redo")),
            ]))]),
        ])
        .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    let file_center = bounds_center(file.bounds);
    click_center(&mut ui, &mut app, &mut services, file_center);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    let Action::Press { key } = &press_step.action else {
        unreachable!("press step must be a press action");
    };
    let keys = parse_key_sequence(key);
    for (idx, key) in keys.into_iter().enumerate() {
        dispatch_web_press(&mut ui, &mut app, &mut services, key);
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + idx as u64),
            true,
            |cx| vec![build(cx)],
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    assert!(
        !file.flags.expanded,
        "expected menubar menu to be closed after Escape"
    );

    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("File"),
        "expected Escape close to restore focus to the menubar trigger"
    );
    assert!(
        ui.focus().is_some(),
        "expected UiTree focus to be set after menubar Escape close"
    );
}

#[test]
fn radix_web_menubar_hover_switch_trigger_matches_fret() {
    let golden = read_timeline("menubar-example.menubar.hover-switch-trigger.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "hover-switch-trigger");
    assert!(golden.steps.len() >= 3);

    let switch_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Hover { target } if target == "menubar-trigger:Edit"))
        .expect("switch step");

    let edit_open = find_first(&switch_step.snapshot.dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "menubar-trigger")
            && n.text.as_deref().is_some_and(|t| t.trim() == "Edit")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
            && n.attrs.get("aria-expanded").is_some_and(|v| v == "true")
    });
    assert!(
        edit_open.is_some(),
        "web expected Edit trigger to be open after hover switch"
    );

    let file_open = find_first(&switch_step.snapshot.dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "menubar-trigger")
            && n.text.as_deref().is_some_and(|t| t.trim() == "File")
            && n.attrs.get("data-state").is_some_and(|v| v == "open")
    });
    assert!(
        file_open.is_none(),
        "web expected File trigger to be closed after hover switch"
    );

    assert!(
        has_dom_node_attr(&switch_step.snapshot.dom, "data-slot", "menubar-content"),
        "web expected menubar content to remain present after hover switch"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let build = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu};

        Menubar::new(vec![
            MenubarMenu::new("File").entries(vec![
                MenubarEntry::Group(MenubarGroup::new(vec![
                    MenubarEntry::Item(MenubarItem::new("New Tab")),
                    MenubarEntry::Item(MenubarItem::new("New Window")),
                    MenubarEntry::Item(MenubarItem::new("New Incognito Window")),
                ])),
                MenubarEntry::Separator,
                MenubarEntry::Group(MenubarGroup::new(vec![MenubarEntry::Item(
                    MenubarItem::new("Print..."),
                )])),
            ]),
            MenubarMenu::new("Edit").entries(vec![MenubarEntry::Group(MenubarGroup::new(vec![
                MenubarEntry::Item(MenubarItem::new("Undo")),
                MenubarEntry::Item(MenubarItem::new("Redo")),
            ]))]),
        ])
        .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    let edit = find_semantics(&snap, SemanticsRole::MenuItem, "Edit");
    let file_center = bounds_center(file.bounds);
    let edit_center = bounds_center(edit.bounds);
    click_center(&mut ui, &mut app, &mut services, file_center);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    move_pointer(&mut ui, &mut app, &mut services, edit_center);
    deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        find_semantics(&snap, SemanticsRole::MenuItem, "Edit")
            .flags
            .expanded,
        "hovering a sibling trigger should switch which menubar menu is open"
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !find_semantics(&snap, SemanticsRole::MenuItem, "File")
            .flags
            .expanded,
        "expected File trigger to close after hover switch"
    );
}

#[test]
fn radix_web_menubar_outside_click_close_matches_fret() {
    let golden = read_timeline("menubar-example.menubar.outside-click-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "outside-click-close");
    assert!(golden.steps.len() >= 3);

    let close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "outside"))
        .expect("close step");
    assert!(
        !has_dom_node_attr(&close_step.snapshot.dom, "data-slot", "menubar-content"),
        "web expected menubar content to be absent after outside click"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let build = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu};

        Menubar::new(vec![
            MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Group(MenubarGroup::new(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ])),
                ])),
                MenubarEntry::Separator,
                MenubarEntry::Group(MenubarGroup::new(vec![MenubarEntry::Item(
                    MenubarItem::new("Print..."),
                )])),
            ]),
            MenubarMenu::new("Edit").entries(vec![MenubarEntry::Group(MenubarGroup::new(vec![
                MenubarEntry::Item(MenubarItem::new("Undo")),
                MenubarEntry::Item(MenubarItem::new("Redo")),
            ]))]),
        ])
        .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    click_center(&mut ui, &mut app, &mut services, bounds_center(file.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        find_semantics(&snap, SemanticsRole::MenuItem, "File")
            .flags
            .expanded,
        "expected File menu to be open before outside click"
    );

    click_outside(&mut ui, &mut app, &mut services, bounds);
    deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !find_semantics(&snap, SemanticsRole::MenuItem, "File")
            .flags
            .expanded,
        "expected File menu to be closed after outside click"
    );
    assert!(
        snap.nodes.iter().all(|n| !n.flags.focused),
        "expected focus to be cleared after closing the menubar via outside click"
    );
    assert_eq!(ui.focus(), None, "expected UiTree focus to be cleared");
}

#[test]
fn radix_web_menubar_submenu_outside_click_close_matches_fret() {
    let golden = read_timeline("menubar-example.menubar.submenu-outside-click-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "submenu-outside-click-close");
    assert!(golden.steps.len() >= 4);

    let close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "outside"))
        .expect("close step");
    assert!(
        !has_dom_node_attr(&close_step.snapshot.dom, "data-slot", "menubar-content"),
        "web expected menubar content to be absent after outside click"
    );
    assert!(
        !has_dom_node_attr(&close_step.snapshot.dom, "data-slot", "menubar-sub-content"),
        "web expected menubar submenu content to be absent after outside click"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let build = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu};

        Menubar::new(vec![
            MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Group(MenubarGroup::new(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ])),
                ])),
                MenubarEntry::Separator,
                MenubarEntry::Group(MenubarGroup::new(vec![MenubarEntry::Item(
                    MenubarItem::new("Print..."),
                )])),
            ]),
            MenubarMenu::new("Edit").entries(vec![MenubarEntry::Group(MenubarGroup::new(vec![
                MenubarEntry::Item(MenubarItem::new("Undo")),
                MenubarEntry::Item(MenubarItem::new("Redo")),
            ]))]),
        ])
        .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    click_center(&mut ui, &mut app, &mut services, bounds_center(file.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let share = find_semantics(&snap, SemanticsRole::MenuItem, "Share");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(share.bounds),
    );
    deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    snap.nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("submenu Email link item should be present after hover");

    click_outside(&mut ui, &mut app, &mut services, bounds);
    deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(4 + tick),
            request_semantics,
            |cx| vec![build(cx)],
        );
        deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !find_semantics(&snap, SemanticsRole::MenuItem, "File")
            .flags
            .expanded,
        "expected File menu to be closed after outside click"
    );
    assert!(
        snap.nodes
            .iter()
            .all(|n| n.label.as_deref().is_none_or(|l| l != "Email link")),
        "submenu content should be closed after outside click"
    );
    assert_focus_cleared(&ui, &snap, "menubar submenu outside click close");
}

#[test]
fn radix_web_navigation_menu_open_close_matches_fret() {
    let golden = read_timeline("navigation-menu-example.navigation-menu.open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "navigation-menu");
    assert_eq!(golden.scenario, "open-close");
    assert!(golden.steps.len() >= 3);

    let open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "navigation-menu-trigger"))
        .expect("open step");
    assert!(
        has_dom_node_attr(&open_step.snapshot.dom, "aria-expanded", "true"),
        "web expected aria-expanded=true after open"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, model: &Model<Option<Arc<str>>>| {
        fret_ui_shadcn::NavigationMenu::new(model.clone())
            .items(vec![
                fret_ui_shadcn::NavigationMenuItem::new("alpha", "Alpha", vec![cx.text("A")]),
                fret_ui_shadcn::NavigationMenuItem::new("beta", "Beta", vec![cx.text("B")]),
            ])
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx, &model)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let alpha = find_semantics(&snap, SemanticsRole::Button, "Alpha");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(alpha.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx, &model)],
    );

    assert_eq!(
        app.models()
            .read(&model, |v| v.clone())
            .unwrap_or_default()
            .as_deref(),
        Some("alpha")
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &model)],
    );

    assert_eq!(
        app.models()
            .read(&model, |v| v.clone())
            .unwrap_or_default()
            .as_deref(),
        None
    );
}

#[test]
fn radix_web_popover_open_close_matches_fret() {
    let golden = read_timeline("popover-example.popover.open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "popover");
    assert_eq!(golden.scenario, "open-close");
    assert!(golden.steps.len() >= 3);

    let open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "popover-trigger"))
        .expect("open step");
    assert!(
        has_dom_node_attr(&open_step.snapshot.dom, "data-slot", "popover-content"),
        "web expected popover content to be present after open"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::Popover::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open Popover")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |cx| fret_ui_shadcn::PopoverContent::new(vec![cx.text("Hello")]).into_element(cx),
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open Popover");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    assert!(app.models().get_copied(&open).unwrap_or(false));

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    assert!(!app.models().get_copied(&open).unwrap_or(false));
}

#[test]
fn radix_web_select_open_navigate_select_matches_fret() {
    let golden = read_timeline("select-example.select.open-navigate-select.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "select");
    assert_eq!(golden.scenario, "open-navigate-select");
    assert!(golden.steps.len() >= 3);

    let open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "select-trigger"))
        .expect("open step");
    assert!(
        has_dom_node_attr(&open_step.snapshot.dom, "role", "listbox"),
        "web expected role=listbox after open"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build =
        |cx: &mut ElementContext<'_, App>, value: &Model<Option<Arc<str>>>, open: &Model<bool>| {
            fret_ui_shadcn::Select::new(value.clone(), open.clone())
                .a11y_label("Select")
                .items([
                    fret_ui_shadcn::SelectItem::new("apple", "Apple"),
                    fret_ui_shadcn::SelectItem::new("banana", "Banana"),
                    fret_ui_shadcn::SelectItem::new("blueberry", "Blueberry"),
                    fret_ui_shadcn::SelectItem::new("grapes", "Grapes"),
                    fret_ui_shadcn::SelectItem::new("pineapple", "Pineapple"),
                ])
                .into_element(cx)
        };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx, &value, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let combobox = find_semantics_by_role(&snap, SemanticsRole::ComboBox);
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(combobox.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx, &value, &open)],
    );

    assert!(app.models().get_copied(&open).unwrap_or(false));

    let press_step = golden
        .steps
        .iter()
        .find(|s| matches!(s.action, Action::Press { .. }))
        .expect("press step");
    let Action::Press { key } = &press_step.action else {
        unreachable!("press step must be a press action");
    };
    let keys = parse_key_sequence(key);
    for (idx, key) in keys.into_iter().enumerate() {
        dispatch_web_press(&mut ui, &mut app, &mut services, key);
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + idx as u64),
            true,
            |cx| vec![build(cx, &value, &open)],
        );
    }

    assert!(
        !app.models().get_copied(&open).unwrap_or(false),
        "expected select listbox to close after Enter"
    );
}

#[test]
fn radix_web_tooltip_hover_show_hide_matches_fret() {
    let golden = read_timeline("tooltip-example.tooltip.hover-show-hide.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "tooltip");
    assert_eq!(golden.scenario, "hover-show-hide");
    assert!(golden.steps.len() >= 3);

    let open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Hover { target } if target == "tooltip-trigger"))
        .expect("open step");
    assert!(
        has_dom_node_attr(&open_step.snapshot.dom, "role", "tooltip"),
        "web expected role=tooltip after open"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>| {
        let trigger = fret_ui_shadcn::Button::new("Hover").into_element(cx);
        let content = fret_ui_shadcn::TooltipContent::new(vec![cx.text("Tip")]).into_element(cx);
        fret_ui_shadcn::Tooltip::new(trigger, content)
            .open_delay_frames(0)
            .close_delay_frames(0)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Hover");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        has_semantics_role(&snap, SemanticsRole::Tooltip),
        "expected tooltip semantics after hover"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 1;
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
            |cx| vec![build(cx)],
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !has_semantics_role(&snap, SemanticsRole::Tooltip),
        "expected tooltip to be dismissed after Escape"
    );
}

#[test]
fn radix_web_hover_card_hover_matches_fret() {
    let golden = read_timeline("hover-card-example.hover-card.hover.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "hover-card");
    assert_eq!(golden.scenario, "hover");
    assert!(golden.steps.len() >= 3);

    let open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Hover { target } if target == "hover-card-trigger"))
        .expect("open step");
    assert!(
        has_dom_node_attr(&open_step.snapshot.dom, "data-slot", "hover-card-content"),
        "web expected hover-card content to be present after hover"
    );

    let close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Hover { target } if target == "mouse-away"))
        .expect("close step");
    assert!(
        has_dom_node_attr(&close_step.snapshot.dom, "data-slot", "hover-card-content")
            && has_dom_node_attr(&close_step.snapshot.dom, "data-state", "closed"),
        "web expected hover-card content to remain mounted with data-state=closed after hover out"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let open: Model<bool> = app.models_mut().insert(false);
    let build = |cx: &mut ElementContext<'_, App>| {
        let trigger = fret_ui_shadcn::Button::new("@nextjs").into_element(cx);
        let content = cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("HoverCardContent")),
                ..Default::default()
            },
            |cx| {
                vec![fret_ui_shadcn::HoverCardContent::new(vec![cx.text("card")]).into_element(cx)]
            },
        );

        fret_ui_shadcn::HoverCard::new(trigger, content)
            .open(Some(open.clone()))
            .open_delay_frames(0)
            .close_delay_frames(0)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "@nextjs");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let content = find_semantics(&snap, SemanticsRole::Panel, "HoverCardContent");
    assert_eq!(content.label.as_deref(), Some("HoverCardContent"));

    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(
            Px(bounds.size.width.0 - 1.0),
            Px(bounds.size.height.0 - 1.0),
        ),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let content_present = snap
        .nodes
        .iter()
        .any(|n| n.role == SemanticsRole::Panel && n.label.as_deref() == Some("HoverCardContent"));
    assert!(
        content_present,
        "expected hover-card content to remain mounted during close (data-state=closed in web golden)"
    );
    let open_now = app.models_mut().read(&open, |v| *v).unwrap_or(false);
    assert!(
        !open_now,
        "expected hover-card open state to be false after hover out"
    );
}

#[test]
fn radix_web_checkbox_toggle_state_matches_fret() {
    let golden = read_timeline("checkbox-example.checkbox.toggle.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "checkbox");
    assert_eq!(golden.scenario, "toggle");
    assert!(golden.steps.len() >= 2);

    let step = &golden.steps[1];
    let Action::Click { target } = &step.action else {
        panic!("expected click action");
    };
    assert_eq!(target, "checkbox");
    let expected_checked = parse_bool_attr(&step.snapshot.focus.attrs, "aria-checked");

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let checked: Model<bool> = app.models_mut().insert(!expected_checked);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
                fret_ui_shadcn::Checkbox::new(checked.clone())
                    .a11y_label("Checkbox")
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let cb = find_semantics(&snap, SemanticsRole::Checkbox, "Checkbox");
    click_center(&mut ui, &mut app, &mut services, bounds_center(cb.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            vec![
                fret_ui_shadcn::Checkbox::new(checked.clone())
                    .a11y_label("Checkbox")
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let cb = find_semantics(&snap, SemanticsRole::Checkbox, "Checkbox");
    assert_eq!(cb.flags.checked, Some(expected_checked));
}

#[test]
fn radix_web_dropdown_menu_submenu_hover_select_matches_fret() {
    let golden = read_timeline("dropdown-menu-example.dropdown-menu.submenu-hover-select.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "dropdown-menu");
    assert_eq!(golden.scenario, "submenu-hover-select");
    assert!(golden.steps.len() >= 5);

    let open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "dropdown-menu:with-submenu"))
        .expect("open step");
    assert!(
        has_dom_node_attr(
            &open_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-content"
        ),
        "web expected dropdown menu content to be present after open"
    );
    assert!(
        !has_dom_node_attr(
            &open_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-sub-content"
        ),
        "web expected submenu to be closed immediately after open"
    );

    let sub_open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Hover { target } if target.contains("dropdown-menu-sub-trigger")))
        .expect("submenu open step");
    assert!(
        has_dom_node_attr(
            &sub_open_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-sub-content"
        ),
        "web expected dropdown menu submenu content to be present after hover"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |_cx| {
                vec![fret_ui_shadcn::DropdownMenuEntry::Group(
                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Team"),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Invite users").submenu(vec![
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Email"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Message"),
                                        ),
                                    ]),
                                ),
                                fret_ui_shadcn::DropdownMenuEntry::Separator,
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("More..."),
                                        ),
                                    ]),
                                ),
                            ]),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("New Team"),
                        ),
                    ]),
                )]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !snap
            .nodes
            .iter()
            .any(|n| { n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email") }),
        "submenu item should not be present before hovering the submenu trigger"
    );

    let group_ancestor_id = |node: &fret_core::SemanticsNode| {
        let mut cur = node.parent;
        while let Some(id) = cur {
            let parent = snap.nodes.iter().find(|n| n.id == id)?;
            if parent.role == SemanticsRole::Group {
                return Some(id);
            }
            cur = parent.parent;
        }
        None
    };

    let team = find_semantics(&snap, SemanticsRole::MenuItem, "Team");
    let invite = find_semantics(&snap, SemanticsRole::MenuItem, "Invite users");
    assert_eq!(
        group_ancestor_id(&team),
        group_ancestor_id(&invite),
        "dropdown menu group should contain its menu items"
    );

    let invite = find_semantics(&snap, SemanticsRole::MenuItem, "Invite users");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(invite.bounds),
    );
    deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");

    let group_ancestor_id = |node: &fret_core::SemanticsNode| {
        let mut cur = node.parent;
        while let Some(id) = cur {
            let parent = snap.nodes.iter().find(|n| n.id == id)?;
            if parent.role == SemanticsRole::Group {
                return Some(id);
            }
            cur = parent.parent;
        }
        None
    };

    let email = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email"))
        .expect("submenu Email item should be present after hover");
    let message = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Message"))
        .expect("submenu Message item should be present after hover");
    assert_eq!(
        group_ancestor_id(email),
        group_ancestor_id(message),
        "submenu group should contain its menu items"
    );

    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(email.bounds),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx, &open)],
    );

    let open_now = app.models_mut().read(&open, |v| *v).unwrap_or(false);
    assert!(
        !open_now,
        "selecting a submenu item should close the dropdown menu"
    );
}

#[test]
fn radix_web_context_menu_submenu_hover_select_matches_fret() {
    let golden = read_timeline("context-menu-example.context-menu.submenu-hover-select.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "context-menu");
    assert_eq!(golden.scenario, "submenu-hover-select");
    assert!(golden.steps.len() >= 4);

    let open_step = golden
        .steps
        .iter()
        .find(|s| {
            matches!(&s.action, Action::Click { target } if target == "context-menu:with-submenu")
        })
        .expect("open step");
    assert!(
        has_dom_node_attr(&open_step.snapshot.dom, "data-slot", "context-menu-content"),
        "web expected context menu content to be present after open"
    );

    let sub_open_step = golden
        .steps
        .iter()
        .find(|s| {
            matches!(&s.action, Action::Hover { target } if target.contains("context-menu-sub-trigger"))
        })
        .expect("submenu open step");
    assert!(
        has_dom_node_attr(
            &sub_open_step.snapshot.dom,
            "data-slot",
            "context-menu-sub-content"
        ),
        "web expected context menu submenu content to be present after hover"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
            |_cx| {
                use fret_ui_shadcn::context_menu::ContextMenuItemVariant;
                use fret_ui_shadcn::{ContextMenuEntry, ContextMenuGroup, ContextMenuItem};

                vec![
                    ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                        ContextMenuEntry::Item(ContextMenuItem::new("Copy")),
                        ContextMenuEntry::Item(ContextMenuItem::new("Cut")),
                    ])),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").submenu(vec![
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete")
                                        .variant(ContextMenuItemVariant::Destructive),
                                ),
                            ])),
                    ])),
                ]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Right click here");
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");

    let group_ancestor_id = |node: &fret_core::SemanticsNode| {
        let mut cur = node.parent;
        while let Some(id) = cur {
            let parent = snap.nodes.iter().find(|n| n.id == id)?;
            if parent.role == SemanticsRole::Group {
                return Some(id);
            }
            cur = parent.parent;
        }
        None
    };

    let copy = find_semantics(&snap, SemanticsRole::MenuItem, "Copy");
    let cut = find_semantics(&snap, SemanticsRole::MenuItem, "Cut");
    assert_eq!(
        group_ancestor_id(copy),
        group_ancestor_id(cut),
        "context menu group should parent its menu items"
    );

    let more_tools = find_semantics(&snap, SemanticsRole::MenuItem, "More Tools");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(more_tools.bounds),
    );
    deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");

    let group_ancestor_id = |node: &fret_core::SemanticsNode| {
        let mut cur = node.parent;
        while let Some(id) = cur {
            let parent = snap.nodes.iter().find(|n| n.id == id)?;
            if parent.role == SemanticsRole::Group {
                return Some(id);
            }
            cur = parent.parent;
        }
        None
    };

    let save_page = find_semantics(&snap, SemanticsRole::MenuItem, "Save Page...");
    let create_shortcut = find_semantics(&snap, SemanticsRole::MenuItem, "Create Shortcut...");
    assert_eq!(
        group_ancestor_id(save_page),
        group_ancestor_id(create_shortcut),
        "submenu group should parent its menu items"
    );

    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(save_page.bounds),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx, &open)],
    );

    let open_now = app.models().get_copied(&open).unwrap_or(false);
    assert!(
        !open_now,
        "selecting a submenu item should close the context menu"
    );
}

#[test]
fn radix_web_context_menu_submenu_unsafe_leave_matches_fret() {
    let golden = read_timeline("context-menu-example.context-menu.submenu-unsafe-leave.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "context-menu");
    assert_eq!(golden.scenario, "submenu-unsafe-leave");
    assert!(golden.steps.len() >= 4);

    let close_step = golden
        .steps
        .iter()
        .find(
            |s| matches!(&s.action, Action::Hover { target } if target == "context-menu-item:Copy"),
        )
        .expect("close step");
    assert!(
        !has_dom_node_attr(
            &close_step.snapshot.dom,
            "data-slot",
            "context-menu-sub-content"
        ),
        "web expected context submenu content to be absent after leaving"
    );
    assert!(
        has_dom_node_attr(
            &close_step.snapshot.dom,
            "data-slot",
            "context-menu-content"
        ),
        "web expected context menu content to remain present after leaving"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
            |_cx| {
                use fret_ui_shadcn::context_menu::ContextMenuItemVariant;
                use fret_ui_shadcn::{ContextMenuEntry, ContextMenuGroup, ContextMenuItem};

                vec![
                    ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                        ContextMenuEntry::Item(ContextMenuItem::new("Copy")),
                        ContextMenuEntry::Item(ContextMenuItem::new("Cut")),
                    ])),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").submenu(vec![
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(
                            vec![ContextMenuEntry::Item(
                            ContextMenuItem::new("Delete")
                                .variant(ContextMenuItemVariant::Destructive),
                        )],
                        )),
                    ])),
                ]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Right click here");
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let more_tools = find_semantics(&snap, SemanticsRole::MenuItem, "More Tools");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(more_tools.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(100), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    snap.nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page..."))
        .expect("submenu should be open after hover delay");

    let copy = find_semantics(&snap, SemanticsRole::MenuItem, "Copy");
    let copy_center = bounds_center(copy.bounds);
    move_pointer(&mut ui, &mut app, &mut services, copy_center);
    timers.ingest_effects(&mut app);
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(Px(copy_center.x.0 - 10.0), copy_center.y),
    );
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(120), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .all(|n| n.label.as_deref().is_none_or(|l| l != "Save Page...")),
        "context submenu content should close after leaving"
    );

    let open_now = app.models().get_copied(&open).unwrap_or(false);
    assert!(
        open_now,
        "context menu should remain open after leaving the submenu"
    );
}

#[test]
fn radix_web_menubar_submenu_hover_select_matches_fret() {
    let golden = read_timeline("menubar-example.menubar.submenu-hover-select.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "submenu-hover-select");
    assert!(golden.steps.len() >= 4);

    let open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target.contains("menubar:with-submenu")))
        .expect("open step");
    assert!(
        has_dom_node_attr(&open_step.snapshot.dom, "data-slot", "menubar-content"),
        "web expected menubar content to be present after open"
    );

    let sub_open_step = golden
        .steps
        .iter()
        .find(|s| {
            matches!(&s.action, Action::Hover { target } if target.contains("menubar-sub-trigger"))
        })
        .expect("submenu open step");
    assert!(
        has_dom_node_attr(
            &sub_open_step.snapshot.dom,
            "data-slot",
            "menubar-sub-content"
        ),
        "web expected menubar submenu content to be present after hover"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let build = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu};

        Menubar::new(vec![
            MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Group(MenubarGroup::new(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ])),
                ])),
                MenubarEntry::Separator,
                MenubarEntry::Group(MenubarGroup::new(vec![MenubarEntry::Item(
                    MenubarItem::new("Print..."),
                )])),
            ]),
            MenubarMenu::new("Edit").entries(vec![
                MenubarEntry::Group(MenubarGroup::new(vec![
                    MenubarEntry::Item(MenubarItem::new("Undo")),
                    MenubarEntry::Item(MenubarItem::new("Redo")),
                ])),
                MenubarEntry::Separator,
                MenubarEntry::Submenu(MenubarItem::new("Find").submenu(vec![MenubarEntry::Group(
                    MenubarGroup::new(vec![
                        MenubarEntry::Item(MenubarItem::new("Find...")),
                        MenubarEntry::Item(MenubarItem::new("Find Next")),
                        MenubarEntry::Item(MenubarItem::new("Find Previous")),
                    ]),
                )])),
                MenubarEntry::Separator,
                MenubarEntry::Group(MenubarGroup::new(vec![
                    MenubarEntry::Item(MenubarItem::new("Cut")),
                    MenubarEntry::Item(MenubarItem::new("Copy")),
                    MenubarEntry::Item(MenubarItem::new("Paste")),
                ])),
            ]),
        ])
        .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    let file_center = bounds_center(file.bounds);
    click_center(&mut ui, &mut app, &mut services, file_center);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let share = find_semantics(&snap, SemanticsRole::MenuItem, "Share");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(share.bounds),
    );
    deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let email = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("submenu Email link item should be present after hover");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(email.bounds),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx)],
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(5),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    assert!(
        !file.flags.expanded,
        "selecting a submenu item should close the menubar menu"
    );
}

#[test]
fn radix_web_menubar_submenu_unsafe_leave_matches_fret() {
    let golden = read_timeline("menubar-example.menubar.submenu-unsafe-leave.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "submenu-unsafe-leave");
    assert!(golden.steps.len() >= 4);

    let close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Hover { target } if target == "menubar-trigger:Edit"))
        .expect("close step");
    assert!(
        !has_dom_node_attr(&close_step.snapshot.dom, "data-slot", "menubar-sub-content"),
        "web expected menubar submenu content to be absent after leaving"
    );
    assert!(
        has_dom_node_attr(&close_step.snapshot.dom, "data-slot", "menubar-content"),
        "web expected menubar content to remain present after leaving"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let build = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu};

        Menubar::new(vec![
            MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Group(MenubarGroup::new(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ])),
                ])),
                MenubarEntry::Separator,
                MenubarEntry::Group(MenubarGroup::new(vec![MenubarEntry::Item(
                    MenubarItem::new("Print..."),
                )])),
            ]),
            MenubarMenu::new("Edit").entries(vec![
                MenubarEntry::Group(MenubarGroup::new(vec![
                    MenubarEntry::Item(MenubarItem::new("Undo")),
                    MenubarEntry::Item(MenubarItem::new("Redo")),
                ])),
                MenubarEntry::Separator,
                MenubarEntry::Submenu(MenubarItem::new("Find").submenu(vec![MenubarEntry::Group(
                    MenubarGroup::new(vec![
                        MenubarEntry::Item(MenubarItem::new("Find...")),
                        MenubarEntry::Item(MenubarItem::new("Find Next")),
                        MenubarEntry::Item(MenubarItem::new("Find Previous")),
                    ]),
                )])),
                MenubarEntry::Separator,
                MenubarEntry::Group(MenubarGroup::new(vec![
                    MenubarEntry::Item(MenubarItem::new("Cut")),
                    MenubarEntry::Item(MenubarItem::new("Copy")),
                    MenubarEntry::Item(MenubarItem::new("Paste")),
                ])),
            ]),
        ])
        .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    let edit = find_semantics(&snap, SemanticsRole::MenuItem, "Edit");
    let file_center = bounds_center(file.bounds);
    let edit_center = bounds_center(edit.bounds);
    click_center(&mut ui, &mut app, &mut services, file_center);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let share = find_semantics(&snap, SemanticsRole::MenuItem, "Share");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(share.bounds),
    );
    deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    snap.nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("submenu Email link item should be present after hover");

    move_pointer(&mut ui, &mut app, &mut services, edit_center);
    deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .all(|n| n.label.as_deref().is_none_or(|l| l != "Email link")),
        "menubar submenu content should close after leaving"
    );

    assert!(
        find_semantics(&snap, SemanticsRole::MenuItem, "File")
            .flags
            .expanded,
        "hovering a sibling trigger should close the submenu but keep the menubar menu open"
    );
}

#[test]
fn radix_web_dropdown_menu_submenu_grace_corridor_matches_fret() {
    let golden = read_timeline("dropdown-menu-example.dropdown-menu.submenu-grace-corridor.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "dropdown-menu");
    assert_eq!(golden.scenario, "submenu-grace-corridor");
    assert!(golden.steps.len() >= 6);

    let inside_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Hover { target } if target == "submenu-grace:inside"))
        .expect("inside step");
    assert!(
        has_dom_node_attr(
            &inside_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-sub-content"
        ),
        "web expected submenu content to remain present after grace corridor move"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |_cx| {
                vec![fret_ui_shadcn::DropdownMenuEntry::Group(
                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Team"),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Invite users").submenu(vec![
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Email"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Message"),
                                        ),
                                    ]),
                                ),
                                fret_ui_shadcn::DropdownMenuEntry::Separator,
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("More..."),
                                        ),
                                    ]),
                                ),
                            ]),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("New Team"),
                        ),
                    ]),
                )]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let invite = find_semantics(&snap, SemanticsRole::MenuItem, "Invite users");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(invite.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(100), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let email = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email"))
        .expect("submenu should be open");

    let invite_center = bounds_center(invite.bounds);
    let email_center = bounds_center(email.bounds);
    let mid = Point::new(
        Px((invite_center.x.0 + email_center.x.0) * 0.5),
        Px((invite_center.y.0 + email_center.y.0) * 0.5),
    );
    move_pointer(&mut ui, &mut app, &mut services, mid);
    timers.ingest_effects(&mut app);

    move_pointer(&mut ui, &mut app, &mut services, email_center);
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(120), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(300), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let email = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email"))
        .expect("submenu content should remain open after grace corridor move");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(email.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(5),
        true,
        |cx| vec![build(cx, &open)],
    );

    let open_now = app.models().get_copied(&open).unwrap_or(false);
    assert!(
        !open_now,
        "selecting a submenu item should close the dropdown menu"
    );
}

#[test]
fn radix_web_dropdown_menu_submenu_unsafe_leave_matches_fret() {
    let golden = read_timeline("dropdown-menu-example.dropdown-menu.submenu-unsafe-leave.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "dropdown-menu");
    assert_eq!(golden.scenario, "submenu-unsafe-leave");
    assert!(golden.steps.len() >= 4);

    let close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Hover { target } if target == "dropdown-menu-item:Team"))
        .expect("close step");
    assert!(
        !has_dom_node_attr(
            &close_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-sub-content"
        ),
        "web expected submenu content to be absent after hovering an unsafe sibling"
    );
    assert!(
        has_dom_node_attr(
            &close_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-content"
        ),
        "web expected root menu content to remain present after hovering an unsafe sibling"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |_cx| {
                vec![fret_ui_shadcn::DropdownMenuEntry::Group(
                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Team"),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Invite users").submenu(vec![
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Email"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Message"),
                                        ),
                                    ]),
                                ),
                                fret_ui_shadcn::DropdownMenuEntry::Separator,
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("More..."),
                                        ),
                                    ]),
                                ),
                            ]),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("New Team"),
                        ),
                    ]),
                )]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let invite = find_semantics(&snap, SemanticsRole::MenuItem, "Invite users");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(invite.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(100), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    snap.nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email"))
        .expect("submenu should be open after hover delay");
    let team = find_semantics(&snap, SemanticsRole::MenuItem, "Team");

    move_pointer(&mut ui, &mut app, &mut services, bounds_center(team.bounds));
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(120), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .all(|n| n.label.as_deref().is_none_or(|l| l != "Email")),
        "submenu content should close after leaving to an unsafe sibling"
    );

    let open_now = app.models().get_copied(&open).unwrap_or(false);
    assert!(
        open_now,
        "root dropdown menu should remain open after leaving submenu"
    );
}

#[test]
fn radix_web_dropdown_menu_submenu_keyboard_open_close_matches_fret() {
    let golden =
        read_timeline("dropdown-menu-example.dropdown-menu.submenu-keyboard-open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "dropdown-menu");
    assert_eq!(golden.scenario, "submenu-keyboard-open-close");
    assert!(golden.steps.len() >= 4);

    let sub_open_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "ArrowDown,ArrowRight"))
        .expect("submenu open step");
    assert!(
        has_dom_node_attr(
            &sub_open_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-sub-content"
        ),
        "web expected submenu content to be present after ArrowRight"
    );

    let sub_close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "ArrowLeft"))
        .expect("submenu close step");
    assert!(
        !has_dom_node_attr(
            &sub_close_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-sub-content"
        ),
        "web expected submenu content to be absent after ArrowLeft"
    );
    assert_eq!(
        sub_close_step.snapshot.focus.text.as_deref(),
        Some("Invite users"),
        "web expected focus to return to the submenu trigger after ArrowLeft"
    );
    assert_eq!(
        sub_close_step
            .snapshot
            .focus
            .attrs
            .get("data-slot")
            .map(String::as_str),
        Some("dropdown-menu-sub-trigger"),
        "web expected focused element to be the dropdown-menu submenu trigger after ArrowLeft"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |_cx| {
                vec![fret_ui_shadcn::DropdownMenuEntry::Group(
                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Team"),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Invite users").submenu(vec![
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Email"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Message"),
                                        ),
                                    ]),
                                ),
                                fret_ui_shadcn::DropdownMenuEntry::Separator,
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("More..."),
                                        ),
                                    ]),
                                ),
                            ]),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("New Team"),
                        ),
                    ]),
                )]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open");
    ui.set_focus(Some(trigger.id));
    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Team"),
        "expected first menu item to be focused after opening via ArrowDown"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Invite users"),
        "expected submenu trigger to be focused after ArrowDown"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx, &open)],
    );

    // Allow the submenu to mount and nominate a focus target, then deliver the focus timer.
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(0), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(5),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email")),
        "submenu should be open after ArrowRight"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Email"),
        "expected first submenu item to be focused after ArrowRight"
    );
    assert_eq!(
        ui.focus(),
        Some(focused.id),
        "expected UiTree focus to match focused semantics node"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowLeft);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(6),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !snap
            .nodes
            .iter()
            .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email")),
        "submenu should be closed after ArrowLeft"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Invite users"),
        "expected focus to return to the submenu trigger after ArrowLeft"
    );
}

#[test]
fn radix_web_context_menu_submenu_keyboard_open_close_matches_fret() {
    let golden =
        read_timeline("context-menu-example.context-menu.submenu-keyboard-open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "context-menu");
    assert_eq!(golden.scenario, "submenu-keyboard-open-close");
    assert!(golden.steps.len() >= 4);

    let sub_open_step = golden
        .steps
        .iter()
        .find(|s| {
            matches!(&s.action, Action::Press { key } if key.split(',').last() == Some("ArrowRight"))
        })
        .expect("submenu open step");
    assert!(
        has_dom_node_attr(
            &sub_open_step.snapshot.dom,
            "data-slot",
            "context-menu-sub-content"
        ),
        "web expected submenu content to be present after ArrowRight"
    );

    let sub_close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "ArrowLeft"))
        .expect("submenu close step");
    assert!(
        !has_dom_node_attr(
            &sub_close_step.snapshot.dom,
            "data-slot",
            "context-menu-sub-content"
        ),
        "web expected submenu content to be absent after ArrowLeft"
    );
    assert_eq!(
        sub_close_step.snapshot.focus.text.as_deref(),
        Some("More Tools"),
        "web expected focus to return to the submenu trigger after ArrowLeft"
    );
    assert_eq!(
        sub_close_step
            .snapshot
            .focus
            .attrs
            .get("data-slot")
            .map(String::as_str),
        Some("context-menu-sub-trigger"),
        "web expected focused element to be the context-menu submenu trigger after ArrowLeft"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
            |_cx| {
                use fret_ui_shadcn::context_menu::ContextMenuItemVariant;
                use fret_ui_shadcn::{ContextMenuEntry, ContextMenuGroup, ContextMenuItem};

                vec![
                    ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                        ContextMenuEntry::Item(ContextMenuItem::new("Copy")),
                        ContextMenuEntry::Item(ContextMenuItem::new("Cut")),
                    ])),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").submenu(vec![
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete")
                                        .variant(ContextMenuItemVariant::Destructive),
                                ),
                            ])),
                    ])),
                ]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Right click here");
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    let mut frame = 3;
    for _ in 0..20 {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("semantics snapshot");
        let focused_label = snap
            .nodes
            .iter()
            .find(|n| n.flags.focused)
            .and_then(|n| n.label.as_deref());
        if focused_label == Some("More Tools") {
            break;
        }

        dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            true,
            |cx| vec![build(cx, &open)],
        );
        frame += 1;
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("More Tools"),
        "expected submenu trigger to be focused before ArrowRight"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx, &open)],
    );
    frame += 1;

    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(0), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx, &open)],
    );
    frame += 1;

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page...")
        }),
        "submenu should be open after ArrowRight"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Save Page..."),
        "expected first submenu item to be focused after ArrowRight"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowLeft);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page...")
        }),
        "submenu should be closed after ArrowLeft"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("More Tools"),
        "expected focus to return to the submenu trigger after ArrowLeft"
    );
}

#[test]
fn radix_web_menubar_submenu_keyboard_open_close_matches_fret() {
    let golden = read_timeline("menubar-example.menubar.submenu-keyboard-open-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "submenu-keyboard-open-close");
    assert!(golden.steps.len() >= 4);

    let sub_open_step = golden
        .steps
        .iter()
        .find(|s| {
            matches!(&s.action, Action::Press { key } if key.split(',').last() == Some("ArrowRight"))
        })
        .expect("submenu open step");
    assert!(
        has_dom_node_attr(
            &sub_open_step.snapshot.dom,
            "data-slot",
            "menubar-sub-content"
        ),
        "web expected submenu content to be present after ArrowRight"
    );

    let sub_close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "ArrowLeft"))
        .expect("submenu close step");
    assert!(
        !has_dom_node_attr(
            &sub_close_step.snapshot.dom,
            "data-slot",
            "menubar-sub-content"
        ),
        "web expected submenu content to be absent after ArrowLeft"
    );
    assert_eq!(
        sub_close_step.snapshot.focus.text.as_deref(),
        Some("Share"),
        "web expected focus to return to the submenu trigger after ArrowLeft"
    );
    assert_eq!(
        sub_close_step
            .snapshot
            .focus
            .attrs
            .get("data-slot")
            .map(String::as_str),
        Some("menubar-sub-trigger"),
        "web expected focused element to be the menubar submenu trigger after ArrowLeft"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu};

        Menubar::new(vec![
            MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Group(MenubarGroup::new(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ])),
                ])),
                MenubarEntry::Separator,
                MenubarEntry::Group(MenubarGroup::new(vec![MenubarEntry::Item(
                    MenubarItem::new("Print..."),
                )])),
            ]),
            MenubarMenu::new("Edit").entries(vec![MenubarEntry::Group(MenubarGroup::new(vec![
                MenubarEntry::Item(MenubarItem::new("Undo")),
                MenubarEntry::Item(MenubarItem::new("Redo")),
            ]))]),
        ])
        .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    click_center(&mut ui, &mut app, &mut services, bounds_center(file.bounds));
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    let mut frame = 3;
    for _ in 0..20 {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("semantics snapshot");
        let focused_label = snap
            .nodes
            .iter()
            .find(|n| n.flags.focused)
            .and_then(|n| n.label.as_deref());
        if focused_label == Some("Share") {
            break;
        }

        dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            true,
            |cx| vec![build(cx)],
        );
        frame += 1;
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Share"),
        "expected submenu trigger to be focused before ArrowRight"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx)],
    );
    frame += 1;

    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(0), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link")
        }),
        "submenu should be open after ArrowRight"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Email link"),
        "expected first submenu item to be focused after ArrowRight"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowLeft);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame + 1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link")
        }),
        "submenu should be closed after ArrowLeft"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Share"),
        "expected focus to return to the submenu trigger after ArrowLeft"
    );
}

#[test]
fn radix_web_menubar_submenu_arrowleft_escape_close_matches_fret() {
    let golden = read_timeline("menubar-example.menubar.submenu-arrowleft-escape-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "submenu-arrowleft-escape-close");
    assert!(golden.steps.len() >= 5);

    let sub_close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "ArrowLeft"))
        .expect("submenu close step");
    assert!(
        !has_dom_node_attr(
            &sub_close_step.snapshot.dom,
            "data-slot",
            "menubar-sub-content"
        ),
        "web expected submenu content to be absent after ArrowLeft"
    );
    assert_eq!(
        sub_close_step.snapshot.focus.text.as_deref(),
        Some("Share"),
        "web expected focus to return to the submenu trigger after ArrowLeft"
    );
    assert_eq!(
        sub_close_step
            .snapshot
            .focus
            .attrs
            .get("data-slot")
            .map(String::as_str),
        Some("menubar-sub-trigger"),
        "web expected focused element to be the menubar submenu trigger after ArrowLeft"
    );

    let close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "Escape"))
        .expect("close step");
    assert!(
        !has_dom_node_attr(&close_step.snapshot.dom, "data-slot", "menubar-content"),
        "web expected menubar content to be absent after Escape"
    );
    assert_eq!(
        close_step.snapshot.focus.text.as_deref(),
        Some("File"),
        "web expected focus to return to the File trigger after Escape"
    );
    assert_eq!(
        close_step
            .snapshot
            .focus
            .attrs
            .get("data-slot")
            .map(String::as_str),
        Some("menubar-trigger"),
        "web expected focused element to be the menubar trigger after Escape"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu};

        Menubar::new(vec![
            MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Group(MenubarGroup::new(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ])),
                ])),
                MenubarEntry::Separator,
                MenubarEntry::Group(MenubarGroup::new(vec![MenubarEntry::Item(
                    MenubarItem::new("Print..."),
                )])),
            ]),
            MenubarMenu::new("Edit").entries(vec![MenubarEntry::Group(MenubarGroup::new(vec![
                MenubarEntry::Item(MenubarItem::new("Undo")),
                MenubarEntry::Item(MenubarItem::new("Redo")),
            ]))]),
        ])
        .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    click_center(&mut ui, &mut app, &mut services, bounds_center(file.bounds));
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    let mut frame = 3;
    for _ in 0..20 {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("semantics snapshot");
        let focused_label = snap
            .nodes
            .iter()
            .find(|n| n.flags.focused)
            .and_then(|n| n.label.as_deref());
        if focused_label == Some("Share") {
            break;
        }

        dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            true,
            |cx| vec![build(cx)],
        );
        frame += 1;
    }

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx)],
    );
    frame += 1;

    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(0), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx)],
    );
    frame += 1;

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link")
        }),
        "submenu should be open after ArrowRight"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowLeft);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx)],
    );
    frame += 1;

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link")
        }),
        "submenu should be closed after ArrowLeft"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Share"),
        "expected focus to return to the submenu trigger after ArrowLeft"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame + tick),
            request_semantics,
            |cx| vec![build(cx)],
        );
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !find_semantics(&snap, SemanticsRole::MenuItem, "File")
            .flags
            .expanded,
        "expected menubar menu to be closed after Escape"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("File"),
        "expected focus to return to the trigger after Escape"
    );
}

#[test]
fn radix_web_context_menu_submenu_grace_corridor_matches_fret() {
    let golden = read_timeline("context-menu-example.context-menu.submenu-grace-corridor.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "context-menu");
    assert_eq!(golden.scenario, "submenu-grace-corridor");
    assert!(golden.steps.len() >= 6);

    let inside_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Hover { target } if target == "submenu-grace:inside"))
        .expect("inside step");
    assert!(
        has_dom_node_attr(
            &inside_step.snapshot.dom,
            "data-slot",
            "context-menu-sub-content"
        ),
        "web expected submenu content to remain present after grace corridor move"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
            |_cx| {
                use fret_ui_shadcn::context_menu::ContextMenuItemVariant;
                use fret_ui_shadcn::{ContextMenuEntry, ContextMenuGroup, ContextMenuItem};

                vec![
                    ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                        ContextMenuEntry::Item(ContextMenuItem::new("Copy")),
                        ContextMenuEntry::Item(ContextMenuItem::new("Cut")),
                    ])),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").submenu(vec![
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete")
                                        .variant(ContextMenuItemVariant::Destructive),
                                ),
                            ])),
                    ])),
                ]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Right click here");
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let more_tools = find_semantics(&snap, SemanticsRole::MenuItem, "More Tools");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(more_tools.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(100), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let save_page = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page..."))
        .expect("submenu should be open");

    let more_tools_center = bounds_center(more_tools.bounds);
    let save_page_center = bounds_center(save_page.bounds);
    let mid = Point::new(
        Px((more_tools_center.x.0 + save_page_center.x.0) * 0.5),
        Px((more_tools_center.y.0 + save_page_center.y.0) * 0.5),
    );
    move_pointer(&mut ui, &mut app, &mut services, mid);
    timers.ingest_effects(&mut app);

    move_pointer(&mut ui, &mut app, &mut services, save_page_center);
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(120), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(300), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let save_page = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page..."))
        .expect("submenu content should remain open after grace corridor move");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(save_page.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(5),
        true,
        |cx| vec![build(cx, &open)],
    );

    let open_now = app.models().get_copied(&open).unwrap_or(false);
    assert!(
        !open_now,
        "selecting a submenu item should close the context menu"
    );
}

#[test]
fn radix_web_menubar_submenu_grace_corridor_matches_fret() {
    let golden = read_timeline("menubar-example.menubar.submenu-grace-corridor.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "menubar");
    assert_eq!(golden.scenario, "submenu-grace-corridor");
    assert!(golden.steps.len() >= 6);

    let inside_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Hover { target } if target == "submenu-grace:inside"))
        .expect("inside step");
    assert!(
        has_dom_node_attr(
            &inside_step.snapshot.dom,
            "data-slot",
            "menubar-sub-content"
        ),
        "web expected submenu content to remain present after grace corridor move"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu};

        Menubar::new(vec![MenubarMenu::new("File").entries(vec![
            MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![MenubarEntry::Group(
                MenubarGroup::new(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ]),
            )])),
            MenubarEntry::Separator,
            MenubarEntry::Group(MenubarGroup::new(vec![MenubarEntry::Item(
                MenubarItem::new("Print..."),
            )])),
        ])])
        .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    click_center(&mut ui, &mut app, &mut services, bounds_center(file.bounds));
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let share = find_semantics(&snap, SemanticsRole::MenuItem, "Share");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(share.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(100), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let email = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("submenu should be open");

    let share_center = bounds_center(share.bounds);
    let email_center = bounds_center(email.bounds);
    let mid = Point::new(
        Px((share_center.x.0 + email_center.x.0) * 0.5),
        Px((share_center.y.0 + email_center.y.0) * 0.5),
    );
    move_pointer(&mut ui, &mut app, &mut services, mid);
    timers.ingest_effects(&mut app);

    move_pointer(&mut ui, &mut app, &mut services, email_center);
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(120), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(300), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let email = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("submenu content should remain open after grace corridor move");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(email.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(5),
        true,
        |cx| vec![build(cx)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let file = find_semantics(&snap, SemanticsRole::MenuItem, "File");
    assert!(
        !file.flags.expanded,
        "selecting a submenu item should close the menubar menu"
    );
}

#[test]
fn radix_web_switch_toggle_state_matches_fret() {
    let golden = read_timeline("switch-example.switch.toggle.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "switch");
    assert_eq!(golden.scenario, "toggle");
    assert!(golden.steps.len() >= 2);

    let step = &golden.steps[1];
    let Action::Click { target } = &step.action else {
        panic!("expected click action");
    };
    assert_eq!(target, "switch");
    let expected_checked = parse_bool_attr(&step.snapshot.focus.attrs, "aria-checked");

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let checked: Model<bool> = app.models_mut().insert(!expected_checked);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
                fret_ui_shadcn::Switch::new(checked.clone())
                    .a11y_label("Switch")
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let sw = find_semantics(&snap, SemanticsRole::Switch, "Switch");
    click_center(&mut ui, &mut app, &mut services, bounds_center(sw.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            vec![
                fret_ui_shadcn::Switch::new(checked.clone())
                    .a11y_label("Switch")
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let sw = find_semantics(&snap, SemanticsRole::Switch, "Switch");
    assert_eq!(sw.flags.checked, Some(expected_checked));
}

#[test]
fn radix_web_tabs_click_second_tab_state_matches_fret() {
    let golden = read_timeline("tabs-example.tabs.click-second-tab.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "tabs");
    assert_eq!(golden.scenario, "click-second-tab");
    assert!(golden.steps.len() >= 2);

    let step = &golden.steps[1];
    let Action::Click { target } = &step.action else {
        panic!("expected click action");
    };
    let (kind, idx) = parse_target_index(target);
    assert_eq!(kind, "tab");
    assert_eq!(idx, 1);
    let expected_selected = parse_bool_attr(&step.snapshot.focus.attrs, "aria-selected");
    assert!(expected_selected, "web expected tab[1] to be selected");

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let selected: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("tab0")));
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let labels = ["Tab 0", "Tab 1"];
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
                fret_ui_shadcn::Tabs::new(selected.clone())
                    .item(fret_ui_shadcn::TabsItem::new("tab0", labels[0], Vec::new()))
                    .item(fret_ui_shadcn::TabsItem::new("tab1", labels[1], Vec::new()))
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let tab1 = find_semantics(&snap, SemanticsRole::Tab, labels[1]);
    click_center(&mut ui, &mut app, &mut services, bounds_center(tab1.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            vec![
                fret_ui_shadcn::Tabs::new(selected.clone())
                    .item(fret_ui_shadcn::TabsItem::new("tab0", labels[0], Vec::new()))
                    .item(fret_ui_shadcn::TabsItem::new("tab1", labels[1], Vec::new()))
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let tab0 = find_semantics(&snap, SemanticsRole::Tab, labels[0]);
    let tab1 = find_semantics(&snap, SemanticsRole::Tab, labels[1]);
    assert_eq!(tab0.flags.selected, false);
    assert_eq!(tab1.flags.selected, true);
}

#[test]
fn radix_web_radio_group_click_second_radio_state_matches_fret() {
    let golden = read_timeline("radio-group-example.radio-group.select-second.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "radio-group");
    assert_eq!(golden.scenario, "select-second");
    assert!(golden.steps.len() >= 2);

    let step = &golden.steps[1];
    let Action::Click { target } = &step.action else {
        panic!("expected click action");
    };
    let (kind, idx) = parse_target_index(target);
    assert_eq!(kind, "radio");
    assert_eq!(idx, 1);
    let expected_checked = parse_bool_attr(&step.snapshot.focus.attrs, "aria-checked");
    assert!(expected_checked, "web expected radio[1] to be checked");

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let selected: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("r0")));
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let labels = ["Radio 0", "Radio 1"];
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
                fret_ui_shadcn::RadioGroup::new(selected.clone())
                    .a11y_label("Options")
                    .item(fret_ui_shadcn::RadioGroupItem::new("r0", labels[0]))
                    .item(fret_ui_shadcn::RadioGroupItem::new("r1", labels[1]))
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let radio1 = find_semantics(&snap, SemanticsRole::RadioButton, labels[1]);
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(radio1.bounds),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            vec![
                fret_ui_shadcn::RadioGroup::new(selected.clone())
                    .a11y_label("Options")
                    .item(fret_ui_shadcn::RadioGroupItem::new("r0", labels[0]))
                    .item(fret_ui_shadcn::RadioGroupItem::new("r1", labels[1]))
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let radio0 = find_semantics(&snap, SemanticsRole::RadioButton, labels[0]);
    let radio1 = find_semantics(&snap, SemanticsRole::RadioButton, labels[1]);
    assert_eq!(radio0.flags.checked, Some(false));
    assert_eq!(radio1.flags.checked, Some(true));
}

#[test]
fn radix_web_toggle_click_state_matches_fret() {
    let golden = read_timeline("toggle-example.toggle.toggle.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "toggle");
    assert_eq!(golden.scenario, "toggle");
    assert!(golden.steps.len() >= 2);

    let step = &golden.steps[1];
    let Action::Click { target } = &step.action else {
        panic!("expected click action");
    };
    assert_eq!(target, "toggle");
    let expected_pressed = parse_bool_attr(&step.snapshot.focus.attrs, "aria-pressed");

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let pressed: Model<bool> = app.models_mut().insert(!expected_pressed);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
                fret_ui_shadcn::Toggle::new(pressed.clone())
                    .a11y_label("Toggle")
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let toggle = find_semantics(&snap, SemanticsRole::Button, "Toggle");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(toggle.bounds),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            vec![
                fret_ui_shadcn::Toggle::new(pressed.clone())
                    .a11y_label("Toggle")
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let toggle = find_semantics(&snap, SemanticsRole::Button, "Toggle");
    assert_eq!(toggle.flags.selected, expected_pressed);
}

#[test]
fn radix_web_toggle_group_click_second_item_state_matches_fret() {
    let golden = read_timeline("toggle-group-example.toggle-group.select-second.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "toggle-group");
    assert_eq!(golden.scenario, "select-second");
    assert!(golden.steps.len() >= 2);

    let step = &golden.steps[1];
    let Action::Click { target } = &step.action else {
        panic!("expected click action");
    };
    let (kind, idx) = parse_target_index(target);
    assert_eq!(kind, "toggle-group");
    assert_eq!(idx, 1);
    let expected_pressed = parse_bool_attr(&step.snapshot.focus.attrs, "aria-pressed");
    assert!(expected_pressed, "web expected toggle-group[1] pressed");

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let selected: Model<Vec<Arc<str>>> = app.models_mut().insert(Vec::new());
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let labels = ["Bold", "Italic"];
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
                fret_ui_shadcn::ToggleGroup::multiple(selected.clone())
                    .item(
                        fret_ui_shadcn::ToggleGroupItem::new("bold", vec![cx.text(labels[0])])
                            .a11y_label(labels[0]),
                    )
                    .item(
                        fret_ui_shadcn::ToggleGroupItem::new("italic", vec![cx.text(labels[1])])
                            .a11y_label(labels[1]),
                    )
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let italic = find_semantics(&snap, SemanticsRole::Button, labels[1]);
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(italic.bounds),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            vec![
                fret_ui_shadcn::ToggleGroup::multiple(selected.clone())
                    .item(
                        fret_ui_shadcn::ToggleGroupItem::new("bold", vec![cx.text(labels[0])])
                            .a11y_label(labels[0]),
                    )
                    .item(
                        fret_ui_shadcn::ToggleGroupItem::new("italic", vec![cx.text(labels[1])])
                            .a11y_label(labels[1]),
                    )
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let bold = find_semantics(&snap, SemanticsRole::Button, labels[0]);
    let italic = find_semantics(&snap, SemanticsRole::Button, labels[1]);
    assert_eq!(bold.flags.selected, false);
    assert_eq!(italic.flags.selected, true);
}

#[test]
fn radix_web_slider_arrow_right_state_matches_fret() {
    let golden = read_timeline("slider-example.slider.arrow-right.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "slider");
    assert_eq!(golden.scenario, "arrow-right");
    assert!(golden.steps.len() >= 2);

    let step = &golden.steps[1];
    let Action::Press { key } = &step.action else {
        panic!("expected press action");
    };
    let keys = parse_key_sequence(key);

    let expected_value = require_attr(&step.snapshot.focus.attrs, "aria-valuenow").to_string();

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let values: Model<Vec<f32>> = app.models_mut().insert(vec![50.0]);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let slider = fret_ui_shadcn::Slider::new(values.clone())
                .a11y_label("Slider")
                .into_element(cx);
            vec![slider]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let slider = find_semantics(&snap, SemanticsRole::Slider, "Slider");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(slider.bounds),
    );

    for key in keys {
        dispatch_web_press(&mut ui, &mut app, &mut services, key);
    }

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            vec![
                fret_ui_shadcn::Slider::new(values.clone())
                    .a11y_label("Slider")
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let slider = find_semantics(&snap, SemanticsRole::Slider, "Slider");
    assert_eq!(slider.value.as_deref(), Some(expected_value.as_str()));
}

fn fixed_height_spacer(cx: &mut ElementContext<'_, App>, height: Px) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Px(height);
                layout
            },
            ..Default::default()
        },
        |_cx| Vec::new(),
    )
}

#[test]
fn radix_web_accordion_toggle_first_state_matches_fret() {
    let golden = read_timeline("accordion-example.accordion.toggle-first.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "accordion");
    assert_eq!(golden.scenario, "toggle-first");
    assert!(golden.steps.len() >= 2);

    let step = &golden.steps[1];
    let Action::Click { target } = &step.action else {
        panic!("expected click action");
    };
    assert_eq!(target, "accordion-trigger");

    let trigger = find_first(&step.snapshot.dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "accordion-trigger")
            && n.attrs.contains_key("aria-expanded")
    })
    .expect("web accordion trigger");
    let expected_expanded = parse_bool_attr(&trigger.attrs, "aria-expanded");

    let label = "Accordion Trigger".to_string();

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let open: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let trigger = fret_ui_shadcn::AccordionTrigger::new(vec![cx.text(label.clone())])
                .a11y_label(label.clone());
            let content = fret_ui_shadcn::AccordionContent::new(vec![cx.text("Content")]);
            vec![
                fret_ui_shadcn::Accordion::single(open.clone())
                    .item(fret_ui_shadcn::AccordionItem::new(
                        "item-0", trigger, content,
                    ))
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, &label);
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            let trigger = fret_ui_shadcn::AccordionTrigger::new(vec![cx.text(label.clone())])
                .a11y_label(label.clone());
            let content = fret_ui_shadcn::AccordionContent::new(vec![cx.text("Content")]);
            vec![
                fret_ui_shadcn::Accordion::single(open.clone())
                    .item(fret_ui_shadcn::AccordionItem::new(
                        "item-0", trigger, content,
                    ))
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, &label);
    assert_eq!(trigger.flags.expanded, expected_expanded);
}

#[test]
fn radix_web_collapsible_toggle_state_matches_fret() {
    let golden = read_timeline("collapsible-example.collapsible.toggle.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "collapsible");
    assert_eq!(golden.scenario, "toggle");
    assert!(golden.steps.len() >= 2);

    let step = &golden.steps[1];
    let Action::Click { target } = &step.action else {
        panic!("expected click action");
    };
    assert_eq!(target, "collapsible-trigger");

    let trigger = find_first(&step.snapshot.dom, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "collapsible-trigger")
            && n.attrs.contains_key("aria-expanded")
    })
    .expect("web collapsible trigger");
    let expected_expanded = parse_bool_attr(&trigger.attrs, "aria-expanded");

    let window = AppWindowId::default();
    let bounds = window_bounds();
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

    let label: Arc<str> = Arc::from("Collapsible");

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
                fret_ui_shadcn::Collapsible::new(open.clone()).into_element_with_open_model(
                    cx,
                    |cx, open, is_open| {
                        fret_ui_shadcn::CollapsibleTrigger::new(
                            open.clone(),
                            vec![cx.text("Trigger")],
                        )
                        .a11y_label(label.clone())
                        .into_element(cx, is_open)
                    },
                    |cx| {
                        fret_ui_shadcn::CollapsibleContent::new(vec![cx.text("Content")])
                            .into_element(cx)
                    },
                ),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let button = find_semantics(&snap, SemanticsRole::Button, &label);
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(button.bounds),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            vec![
                fret_ui_shadcn::Collapsible::new(open.clone()).into_element_with_open_model(
                    cx,
                    |cx, open, is_open| {
                        fret_ui_shadcn::CollapsibleTrigger::new(
                            open.clone(),
                            vec![cx.text("Trigger")],
                        )
                        .a11y_label(label.clone())
                        .into_element(cx, is_open)
                    },
                    |cx| {
                        fret_ui_shadcn::CollapsibleContent::new(vec![cx.text("Content")])
                            .into_element(cx)
                    },
                ),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let button = find_semantics(&snap, SemanticsRole::Button, &label);
    assert_eq!(button.flags.expanded, expected_expanded);
}

#[test]
fn radix_web_scroll_area_scroll_top_delta_matches_fret() {
    let golden = read_timeline("scroll-area-example.scroll-area.scroll.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "scroll-area");
    assert_eq!(golden.scenario, "scroll");
    assert!(golden.steps.len() >= 2);

    let step1 = &golden.steps[1];
    let Action::Press { key } = &step1.action else {
        panic!("expected press action");
    };
    let expected_scroll_top = key
        .strip_prefix("scrollTop=")
        .unwrap_or_else(|| panic!("expected scrollTop action, got {key:?}"))
        .parse::<f32>()
        .unwrap_or_else(|_| panic!("expected numeric scrollTop action, got {key:?}"));

    // Scrolling down increases scrollTop, making content appear to move up in the viewport.
    let web_delta_y = -expected_scroll_top;

    let window = AppWindowId::default();
    let bounds = window_bounds();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let handle = ScrollHandle::default();
    let marker_label: Arc<str> = Arc::from("scroll marker");

    fn scroll_fixture(
        cx: &mut ElementContext<'_, App>,
        marker_label: Arc<str>,
        handle: ScrollHandle,
    ) -> Vec<AnyElement> {
        let marker = cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Generic,
                label: Some(marker_label),
                ..Default::default()
            },
            move |cx| vec![fixed_height_spacer(cx, Px(1.0))],
        );

        let mut children: Vec<AnyElement> = Vec::new();
        children.push(fixed_height_spacer(cx, Px(200.0)));
        children.push(marker);
        for _ in 0..60 {
            children.push(fixed_height_spacer(cx, Px(20.0)));
        }

        let content = cx.flex(
            FlexProps {
                direction: fret_core::Axis::Vertical,
                ..Default::default()
            },
            move |_cx| children,
        );

        vec![
            fret_ui_shadcn::ScrollArea::new(vec![content])
                .scroll_handle(handle)
                .refine_layout(
                    fret_ui_shadcn::prelude::LayoutRefinement::default()
                        .w_full()
                        .h_px(Px(200.0)),
                )
                .into_element(cx),
        ]
    }

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| scroll_fixture(cx, marker_label.clone(), handle.clone()),
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let marker0 = find_semantics(&snap, SemanticsRole::Generic, &marker_label);
    let marker0_y = marker0.bounds.origin.y.0;

    handle.set_offset(Point::new(Px(0.0), Px(expected_scroll_top)));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| scroll_fixture(cx, marker_label.clone(), handle.clone()),
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let marker1 = find_semantics(&snap, SemanticsRole::Generic, &marker_label);
    let marker1_y = marker1.bounds.origin.y.0;

    let fret_delta_y = marker1_y - marker0_y;
    assert!(
        (fret_delta_y - web_delta_y).abs() <= 2.0,
        "scroll-area delta mismatch: web_delta_y={web_delta_y} fret_delta_y={fret_delta_y}"
    );
}

#[test]
fn radix_web_dropdown_menu_submenu_arrowleft_escape_close_matches_fret() {
    let golden =
        read_timeline("dropdown-menu-example.dropdown-menu.submenu-arrowleft-escape-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "dropdown-menu");
    assert_eq!(golden.scenario, "submenu-arrowleft-escape-close");
    assert!(golden.steps.len() >= 5);

    let sub_open_step = golden
        .steps
        .iter()
        .find(|s| {
            matches!(&s.action, Action::Press { key } if key.split(',').last() == Some("ArrowRight"))
        })
        .expect("submenu open step");
    assert!(
        has_dom_node_attr(
            &sub_open_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-sub-content"
        ),
        "web expected submenu content to be present after ArrowRight"
    );

    let sub_close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "ArrowLeft"))
        .expect("submenu close step");
    assert!(
        !has_dom_node_attr(
            &sub_close_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-sub-content"
        ),
        "web expected submenu content to be absent after ArrowLeft"
    );
    assert_eq!(
        sub_close_step.snapshot.focus.text.as_deref(),
        Some("Invite users"),
        "web expected focus to return to the submenu trigger after ArrowLeft"
    );
    assert_eq!(
        sub_close_step
            .snapshot
            .focus
            .attrs
            .get("data-slot")
            .map(String::as_str),
        Some("dropdown-menu-sub-trigger"),
        "web expected focused element to be the dropdown-menu submenu trigger after ArrowLeft"
    );

    let close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "Escape"))
        .expect("close step");
    assert!(
        !has_dom_node_attr(
            &close_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-content"
        ),
        "web expected dropdown menu content to be absent after Escape"
    );
    assert_eq!(
        close_step.snapshot.focus.text.as_deref(),
        Some("Open"),
        "web expected focus to return to the trigger after Escape"
    );
    assert_eq!(
        close_step
            .snapshot
            .focus
            .attrs
            .get("data-slot")
            .map(String::as_str),
        Some("dropdown-menu-trigger"),
        "web expected focused element to be the dropdown-menu trigger after Escape"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |_cx| {
                vec![fret_ui_shadcn::DropdownMenuEntry::Group(
                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Team"),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Invite users").submenu(vec![
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Email"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Message"),
                                        ),
                                    ]),
                                ),
                                fret_ui_shadcn::DropdownMenuEntry::Separator,
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("More..."),
                                        ),
                                    ]),
                                ),
                            ]),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("New Team"),
                        ),
                    ]),
                )]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open");
    ui.set_focus(Some(trigger.id));

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let focused_role = snap.nodes.iter().find(|n| n.flags.focused).map(|n| n.role);
    assert_eq!(
        focused_role,
        Some(SemanticsRole::MenuItem),
        "expected ArrowDown to focus the first menu item"
    );

    let mut frame = 3;
    for _ in 0..20 {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("semantics snapshot");
        let focused_label = snap
            .nodes
            .iter()
            .find(|n| n.flags.focused)
            .and_then(|n| n.label.as_deref());
        if focused_label == Some("Invite users") {
            break;
        }

        dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            true,
            |cx| vec![build(cx, &open)],
        );
        frame += 1;
    }

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx, &open)],
    );
    frame += 1;

    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(0), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx, &open)],
    );
    frame += 1;

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email")),
        "submenu should be open after ArrowRight"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowLeft);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx, &open)],
    );
    frame += 1;

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !snap
            .nodes
            .iter()
            .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email")),
        "submenu should be closed after ArrowLeft"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Invite users"),
        "expected focus to return to the submenu trigger after ArrowLeft"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame + tick),
            request_semantics,
            |cx| vec![build(cx, &open)],
        );
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);
    }

    assert!(
        !app.models().get_copied(&open).unwrap_or(false),
        "expected dropdown menu to be closed after Escape"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes.iter().all(|n| {
            n.role != SemanticsRole::MenuItem || n.label.as_deref().is_none_or(|l| l != "Team")
        }),
        "expected dropdown menu content to be absent after Escape"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Open"),
        "expected focus to return to the trigger after Escape"
    );
    assert_eq!(
        ui.focus(),
        Some(focused.id),
        "expected UiTree focus to match focused semantics node"
    );
}

#[test]
fn radix_web_context_menu_submenu_arrowleft_escape_close_matches_fret() {
    let golden =
        read_timeline("context-menu-example.context-menu.submenu-arrowleft-escape-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "context-menu");
    assert_eq!(golden.scenario, "submenu-arrowleft-escape-close");
    assert!(golden.steps.len() >= 5);

    let sub_open_step = golden
        .steps
        .iter()
        .find(|s| {
            matches!(&s.action, Action::Press { key } if key.split(',').last() == Some("ArrowRight"))
        })
        .expect("submenu open step");
    assert!(
        has_dom_node_attr(
            &sub_open_step.snapshot.dom,
            "data-slot",
            "context-menu-sub-content"
        ),
        "web expected submenu content to be present after ArrowRight"
    );

    let sub_close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "ArrowLeft"))
        .expect("submenu close step");
    assert!(
        !has_dom_node_attr(
            &sub_close_step.snapshot.dom,
            "data-slot",
            "context-menu-sub-content"
        ),
        "web expected submenu content to be absent after ArrowLeft"
    );
    assert_eq!(
        sub_close_step.snapshot.focus.text.as_deref(),
        Some("More Tools"),
        "web expected focus to return to the submenu trigger after ArrowLeft"
    );
    assert_eq!(
        sub_close_step
            .snapshot
            .focus
            .attrs
            .get("data-slot")
            .map(String::as_str),
        Some("context-menu-sub-trigger"),
        "web expected focused element to be the context-menu submenu trigger after ArrowLeft"
    );

    let close_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Press { key } if key == "Escape"))
        .expect("close step");
    assert!(
        !has_dom_node_attr(
            &close_step.snapshot.dom,
            "data-slot",
            "context-menu-content"
        ),
        "web expected context menu content to be absent after Escape"
    );
    assert_eq!(
        close_step.snapshot.focus.tag, "body",
        "web expected focus to be restored to <body/> after Escape"
    );
    assert!(
        close_step.snapshot.focus.attrs.is_empty(),
        "web expected focus attrs to be empty after Escape"
    );
    assert_eq!(
        close_step.snapshot.focus.text.as_deref(),
        None,
        "web expected focused element text to be absent after Escape"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
            |_cx| {
                use fret_ui_shadcn::context_menu::ContextMenuItemVariant;
                use fret_ui_shadcn::{ContextMenuEntry, ContextMenuGroup, ContextMenuItem};

                vec![
                    ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                        ContextMenuEntry::Item(ContextMenuItem::new("Copy")),
                        ContextMenuEntry::Item(ContextMenuItem::new("Cut")),
                    ])),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").submenu(vec![
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(
                                ContextMenuItem::new("Delete")
                                    .variant(ContextMenuItemVariant::Destructive),
                            ),
                        ])),
                    ])),
                ]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Right click here");
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);

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

    assert!(app.models().get_copied(&open).unwrap_or(false));

    let mut frame = 3;
    for _ in 0..20 {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("semantics snapshot");
        let focused_label = snap
            .nodes
            .iter()
            .find(|n| n.flags.focused)
            .and_then(|n| n.label.as_deref());
        if focused_label == Some("More Tools") {
            break;
        }

        dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            true,
            |cx| vec![build(cx, &open)],
        );
        frame += 1;
    }

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx, &open)],
    );
    frame += 1;

    timers.ingest_effects(&mut app);
    timers.fire_after(Duration::from_millis(0), &mut ui, &mut app, &mut services);
    timers.ingest_effects(&mut app);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx, &open)],
    );
    frame += 1;

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page...")
        }),
        "submenu should be open after ArrowRight"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::ArrowLeft);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| vec![build(cx, &open)],
    );
    frame += 1;

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        !snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page...")
        }),
        "submenu should be closed after ArrowLeft"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("More Tools"),
        "expected focus to return to the submenu trigger after ArrowLeft"
    );

    dispatch_web_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame + tick),
            request_semantics,
            |cx| vec![build(cx, &open)],
        );
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);
    }

    assert!(
        !app.models().get_copied(&open).unwrap_or(false),
        "expected context menu to be closed after Escape"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert_focus_cleared(&ui, &snap, "context menu escape close");
}

#[test]
fn radix_web_dropdown_menu_outside_click_close_matches_fret() {
    let golden = read_timeline("dropdown-menu-example.dropdown-menu.outside-click-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "dropdown-menu");
    assert_eq!(golden.scenario, "outside-click-close");
    assert!(golden.steps.len() >= 3);

    let outside_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "outside"))
        .expect("outside click step");
    assert!(
        !has_dom_node_attr(
            &outside_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-content"
        ),
        "web expected dropdown menu content to be absent after outside click"
    );
    assert_eq!(
        outside_step.snapshot.focus.text.as_deref(),
        Some("Open"),
        "web expected focus to return to the trigger after outside click"
    );
    assert_eq!(
        outside_step
            .snapshot
            .focus
            .attrs
            .get("data-slot")
            .map(String::as_str),
        Some("dropdown-menu-trigger"),
        "web expected focused element to be the dropdown-menu trigger after outside click"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |_cx| {
                vec![fret_ui_shadcn::DropdownMenuEntry::Group(
                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Team"),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Invite users").submenu(vec![
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Email"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Message"),
                                        ),
                                    ]),
                                ),
                            ]),
                        ),
                    ]),
                )]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    assert!(app.models().get_copied(&open).unwrap_or(false));

    click_outside(&mut ui, &mut app, &mut services, bounds);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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
            |cx| vec![build(cx, &open)],
        );
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);
    }

    assert!(
        !app.models().get_copied(&open).unwrap_or(false),
        "expected dropdown menu to be closed after outside click"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .all(|n| n.label.as_deref().is_none_or(|l| l != "Team")),
        "expected dropdown menu content to be absent after outside click"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Open"),
        "expected focus to return to the trigger after outside click"
    );
    assert_eq!(
        ui.focus(),
        Some(focused.id),
        "expected UiTree focus to match focused semantics node"
    );
}

#[test]
fn radix_web_dropdown_menu_submenu_outside_click_close_matches_fret() {
    let golden =
        read_timeline("dropdown-menu-example.dropdown-menu.submenu-outside-click-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "dropdown-menu");
    assert_eq!(golden.scenario, "submenu-outside-click-close");
    assert!(golden.steps.len() >= 4);

    let outside_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "outside"))
        .expect("outside click step");
    assert!(
        !has_dom_node_attr(
            &outside_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-content"
        ),
        "web expected dropdown menu content to be absent after outside click"
    );
    assert!(
        !has_dom_node_attr(
            &outside_step.snapshot.dom,
            "data-slot",
            "dropdown-menu-sub-content"
        ),
        "web expected dropdown menu submenu content to be absent after outside click"
    );
    assert_eq!(
        outside_step.snapshot.focus.text.as_deref(),
        Some("Open"),
        "web expected focus to return to the trigger after outside click"
    );
    assert_eq!(
        outside_step
            .snapshot
            .focus
            .attrs
            .get("data-slot")
            .map(String::as_str),
        Some("dropdown-menu-trigger"),
        "web expected focused element to be the dropdown-menu trigger after outside click"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
            cx,
            |cx| {
                fret_ui_shadcn::Button::new("Open")
                    .toggle_model(open.clone())
                    .into_element(cx)
            },
            |_cx| {
                vec![fret_ui_shadcn::DropdownMenuEntry::Group(
                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Team"),
                        ),
                        fret_ui_shadcn::DropdownMenuEntry::Item(
                            fret_ui_shadcn::DropdownMenuItem::new("Invite users").submenu(vec![
                                fret_ui_shadcn::DropdownMenuEntry::Group(
                                    fret_ui_shadcn::DropdownMenuGroup::new(vec![
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Email"),
                                        ),
                                        fret_ui_shadcn::DropdownMenuEntry::Item(
                                            fret_ui_shadcn::DropdownMenuItem::new("Message"),
                                        ),
                                    ]),
                                ),
                            ]),
                        ),
                    ]),
                )]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Open");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let invite = find_semantics(&snap, SemanticsRole::MenuItem, "Invite users");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(invite.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    snap.nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email"))
        .expect("submenu should be open after hover");

    click_outside(&mut ui, &mut app, &mut services, bounds);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(4 + tick),
            request_semantics,
            |cx| vec![build(cx, &open)],
        );
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);
    }

    assert!(
        !app.models().get_copied(&open).unwrap_or(false),
        "expected dropdown menu to be closed after outside click"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .all(|n| n.label.as_deref().is_none_or(|l| l != "Email")),
        "expected dropdown menu submenu content to be absent after outside click"
    );
    let focused = snap.nodes.iter().find(|n| n.flags.focused).expect("focus");
    assert_eq!(
        focused.label.as_deref(),
        Some("Open"),
        "expected focus to return to the trigger after outside click"
    );
}

#[test]
fn radix_web_context_menu_outside_click_close_matches_fret() {
    let golden = read_timeline("context-menu-example.context-menu.outside-click-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "context-menu");
    assert_eq!(golden.scenario, "outside-click-close");
    assert!(golden.steps.len() >= 3);

    let outside_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "outside"))
        .expect("outside click step");
    assert!(
        !has_dom_node_attr(
            &outside_step.snapshot.dom,
            "data-slot",
            "context-menu-content"
        ),
        "web expected context menu content to be absent after outside click"
    );
    assert_eq!(
        outside_step.snapshot.focus.tag, "body",
        "web expected focus to be restored to <body/> after outside click"
    );
    assert!(
        outside_step.snapshot.focus.attrs.is_empty(),
        "web expected focus attrs to be empty after outside click"
    );
    assert_eq!(
        outside_step.snapshot.focus.text.as_deref(),
        None,
        "web expected focused element text to be absent after outside click"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
            |_cx| {
                use fret_ui_shadcn::context_menu::ContextMenuItemVariant;
                use fret_ui_shadcn::{ContextMenuEntry, ContextMenuGroup, ContextMenuItem};

                vec![
                    ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                        ContextMenuEntry::Item(ContextMenuItem::new("Copy")),
                        ContextMenuEntry::Item(ContextMenuItem::new("Cut")),
                    ])),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").submenu(vec![
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(
                                ContextMenuItem::new("Delete")
                                    .variant(ContextMenuItemVariant::Destructive),
                            ),
                        ])),
                    ])),
                ]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Right click here");
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    assert!(app.models().get_copied(&open).unwrap_or(false));

    click_outside(&mut ui, &mut app, &mut services, bounds);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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
            |cx| vec![build(cx, &open)],
        );
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);
    }

    assert!(
        !app.models().get_copied(&open).unwrap_or(false),
        "expected context menu to be closed after outside click"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert_focus_cleared(&ui, &snap, "context menu outside click close");
}

#[test]
fn radix_web_context_menu_submenu_outside_click_close_matches_fret() {
    let golden =
        read_timeline("context-menu-example.context-menu.submenu-outside-click-close.light");
    assert!(golden.version >= 1);
    assert_eq!(golden.base, "radix");
    assert_eq!(golden.primitive, "context-menu");
    assert_eq!(golden.scenario, "submenu-outside-click-close");
    assert!(golden.steps.len() >= 4);

    let outside_step = golden
        .steps
        .iter()
        .find(|s| matches!(&s.action, Action::Click { target } if target == "outside"))
        .expect("outside click step");
    assert!(
        !has_dom_node_attr(
            &outside_step.snapshot.dom,
            "data-slot",
            "context-menu-content"
        ),
        "web expected context menu content to be absent after outside click"
    );
    assert!(
        !has_dom_node_attr(
            &outside_step.snapshot.dom,
            "data-slot",
            "context-menu-sub-content"
        ),
        "web expected context menu submenu content to be absent after outside click"
    );
    assert_eq!(
        outside_step.snapshot.focus.tag, "body",
        "web expected focus to be restored to <body/> after outside click"
    );
    assert!(
        outside_step.snapshot.focus.attrs.is_empty(),
        "web expected focus attrs to be empty after outside click"
    );
    assert_eq!(
        outside_step.snapshot.focus.text.as_deref(),
        None,
        "web expected focused element text to be absent after outside click"
    );

    let window = AppWindowId::default();
    let bounds = window_bounds();
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
    let mut timers = TimerQueue::default();

    let build = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
            |_cx| {
                use fret_ui_shadcn::context_menu::ContextMenuItemVariant;
                use fret_ui_shadcn::{ContextMenuEntry, ContextMenuGroup, ContextMenuItem};

                vec![
                    ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                        ContextMenuEntry::Item(ContextMenuItem::new("Copy")),
                        ContextMenuEntry::Item(ContextMenuItem::new("Cut")),
                    ])),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").submenu(vec![
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                        ])),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Group(ContextMenuGroup::new(vec![
                            ContextMenuEntry::Item(
                                ContextMenuItem::new("Delete")
                                    .variant(ContextMenuItemVariant::Destructive),
                            ),
                        ])),
                    ])),
                ]
            },
        )
    };

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let trigger = find_semantics(&snap, SemanticsRole::Button, "Right click here");
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

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

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    let more_tools = find_semantics(&snap, SemanticsRole::MenuItem, "More Tools");
    move_pointer(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(more_tools.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        true,
        |cx| vec![build(cx, &open)],
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    snap.nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page..."))
        .expect("submenu should be open after hover");

    click_outside(&mut ui, &mut app, &mut services, bounds);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(4 + tick),
            request_semantics,
            |cx| vec![build(cx, &open)],
        );
        timers.ingest_effects(&mut app);
        timers.fire_all(&mut ui, &mut app, &mut services);
    }

    assert!(
        !app.models().get_copied(&open).unwrap_or(false),
        "expected context menu to be closed after outside click"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .all(|n| n.label.as_deref().is_none_or(|l| l != "Save Page...")),
        "expected context menu submenu content to be absent after outside click"
    );
    assert_focus_cleared(&ui, &snap, "context menu submenu outside click close");
}
