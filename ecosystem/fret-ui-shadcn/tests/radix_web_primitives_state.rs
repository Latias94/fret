use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, KeyCode, Modifiers, MouseButton, Point, PointerEvent, PointerType,
    Px, Rect, SemanticsRole, Size as CoreSize, UiServices,
};
use fret_runtime::{Effect, Model};
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
            other => panic!("unsupported key in radix web action: {other:?}"),
        })
        .collect()
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

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id: FrameId,
    request_semantics: bool,
    render: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) {
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
            position: center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
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

fn has_dom_node_attr(node: &DomNode, key: &str, value: &str) -> bool {
    if node.attrs.get(key).is_some_and(|v| v.as_str() == value) {
        return true;
    }
    node.children
        .iter()
        .any(|child| has_dom_node_attr(child, key, value))
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
    let email = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email"))
        .expect("submenu Email item should be present after hover");

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
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
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
                        .h_px(fret_ui_shadcn::prelude::MetricRef::Px(Px(200.0))),
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
