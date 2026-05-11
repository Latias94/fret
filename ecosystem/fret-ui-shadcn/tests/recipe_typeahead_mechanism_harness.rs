use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, FrameId, KeyCode, Point, Px, Rect, Size as CoreSize};
use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;
use serde::Deserialize;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::{click_at, dispatch_key_press};

#[path = "support/timers.rs"]
mod timers;
use timers::TimerQueue;

const RECIPE_TYPEAHEAD_CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/recipe_typeahead_cases_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RecipeTypeaheadScenario {
    SelectTriggerTypeahead { key: TypeaheadKey },
    DropdownMenuOpenTypeahead { key: TypeaheadKey },
    MenubarOpenTypeahead { key: TypeaheadKey },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TypeaheadKey {
    KeyB,
    KeyO,
    KeyX,
}

#[test]
fn mechanism_harness_recipe_typeahead_cases_match_oracles() {
    let suite: MechanismSuite<RecipeTypeaheadScenario> =
        MechanismSuite::from_json_str(RECIPE_TYPEAHEAD_CASES)
            .expect("recipe typeahead fixture suite");

    let mut observer: fn(
        &MechanismCase<RecipeTypeaheadScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<RecipeTypeaheadScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match case.scenario {
        RecipeTypeaheadScenario::SelectTriggerTypeahead { key } => {
            observe_select_trigger_typeahead(key)
        }
        RecipeTypeaheadScenario::DropdownMenuOpenTypeahead { key } => {
            observe_dropdown_menu_open_typeahead(key)
        }
        RecipeTypeaheadScenario::MenubarOpenTypeahead { key } => {
            observe_menubar_open_typeahead(key)
        }
    }
}

fn observe_select_trigger_typeahead(
    key: TypeaheadKey,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);
    let change_calls: Model<u32> = app.models_mut().insert(0);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_select_typeahead_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
        change_calls.clone(),
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing select typeahead snapshot"))?;
    let trigger = find_by_test_id(&snap, "select-trigger");
    ui.set_focus(Some(trigger.id));

    dispatch_key_press(&mut ui, &mut app, &mut services, key.key_code());

    render_select_typeahead_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
        change_calls.clone(),
    );

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing final select typeahead snapshot"))?;
    let mut observed = ObservedTree::from_semantics_snapshot(&snapshot, bounds);
    observed.set_metric(
        "recipe.open",
        if app.models().get_copied(&open) == Some(true) {
            1.0
        } else {
            0.0
        },
    );
    observed.set_metric(
        "select.selected.index",
        selected_index(app.models().get_cloned(&value).flatten()),
    );
    observed.set_metric(
        "select.value_change.calls",
        app.models().get_copied(&change_calls).unwrap_or(0) as f32,
    );
    Ok(observed)
}

fn observe_dropdown_menu_open_typeahead(
    key: TypeaheadKey,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_dropdown_menu_typeahead_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "menu-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    render_dropdown_menu_typeahead_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

    render_dropdown_menu_typeahead_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        open.clone(),
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, key.key_code());

    render_dropdown_menu_typeahead_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing final dropdown typeahead snapshot"))?;
    let mut observed = ObservedTree::from_semantics_snapshot(&snapshot, bounds);
    observed.set_metric(
        "recipe.open",
        if app.models().get_copied(&open) == Some(true) {
            1.0
        } else {
            0.0
        },
    );
    Ok(observed)
}

fn observe_menubar_open_typeahead(key: TypeaheadKey) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_menubar_typeahead_frame(&mut ui, &mut app, &mut services, window, bounds, true);
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing initial menubar typeahead snapshot"))?;
    let file_trigger = find_by_test_id(&snap, "menubar-trigger");
    ui.set_focus(Some(file_trigger.id));

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

    render_menubar_typeahead_frame(&mut ui, &mut app, &mut services, window, bounds, false);
    dispatch_key_press(&mut ui, &mut app, &mut services, key.key_code());

    render_menubar_typeahead_frame(&mut ui, &mut app, &mut services, window, bounds, true);

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing final menubar typeahead snapshot"))?;
    Ok(ObservedTree::from_semantics_snapshot(&snapshot, bounds))
}

fn render_select_typeahead_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    change_calls: Model<u32>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-recipe-typeahead-select",
        request_semantics,
        move |cx| {
            let items: Vec<shadcn::SelectItem> = ["apple", "banana", "blueberry"]
                .into_iter()
                .map(|value| shadcn::SelectItem::new(value, value))
                .collect();
            vec![
                shadcn::Select::new(value, open)
                    .value(shadcn::SelectValue::new().placeholder("Select"))
                    .a11y_label("Select")
                    .trigger_test_id("select-trigger")
                    .on_value_change(move |host, action_cx, _chosen| {
                        let _ = host.models_mut().update(&change_calls, |count| *count += 1);
                        host.request_redraw(action_cx.window);
                    })
                    .items(items)
                    .into_element(cx),
            ]
        },
    );
}

fn render_dropdown_menu_typeahead_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-recipe-typeahead-dropdown-menu",
        request_semantics,
        move |cx| {
            vec![shadcn::DropdownMenu::from_open(open).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open")
                        .test_id("menu-trigger")
                        .into_element(cx)
                },
                |_cx| {
                    vec![
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Profile")
                                .test_id("dropdown-menu-item-profile")
                                .value("profile"),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Billing")
                                .test_id("dropdown-menu-item-billing")
                                .value("billing"),
                        ),
                    ]
                },
            )]
        },
    );
}

fn render_menubar_typeahead_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-recipe-typeahead-menubar",
        request_semantics,
        move |cx| {
            vec![
                shadcn::Menubar::new(vec![
                    shadcn::MenubarMenu::new("File")
                        .test_id("menubar-trigger")
                        .entries(vec![
                            shadcn::MenubarEntry::Item(
                                shadcn::MenubarItem::new("New").test_id("menubar-item-new"),
                            ),
                            shadcn::MenubarEntry::Item(
                                shadcn::MenubarItem::new("Open").test_id("menubar-item-open"),
                            ),
                            shadcn::MenubarEntry::Item(
                                shadcn::MenubarItem::new("Exit").test_id("menubar-item-exit"),
                            ),
                        ]),
                    shadcn::MenubarMenu::new("Edit")
                        .test_id("menubar-edit-trigger")
                        .entries(vec![shadcn::MenubarEntry::Item(
                            shadcn::MenubarItem::new("Undo").test_id("menubar-item-undo"),
                        )]),
                ])
                .into_element(cx),
            ]
        },
    );
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    name: &str,
    request_semantics: bool,
    root: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);
    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, name, root);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn click_trigger(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    test_id: &str,
) -> Result<(), ScenarioObserveError> {
    let point = point_for_test_id(ui, test_id)?;
    click_at(ui, app, services, point);
    Ok(())
}

fn flush_timers(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    timers: &mut TimerQueue,
) {
    timers.ingest_effects(app);
    timers.fire_all(ui, app, services);
}

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("missing semantics node with test_id={id:?}"))
}

fn point_for_test_id(ui: &UiTree<App>, test_id: &str) -> Result<Point, ScenarioObserveError> {
    let snap = ui.semantics_snapshot().cloned().ok_or_else(|| {
        ScenarioObserveError::new("missing semantics snapshot before point lookup")
    })?;
    let node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .ok_or_else(|| ScenarioObserveError::new(format!("missing test id {test_id:?}")))?;
    Ok(Point::new(
        Px(node.bounds.origin.x.0 + 5.0),
        Px(node.bounds.origin.y.0 + 5.0),
    ))
}

fn selected_index(selected: Option<Arc<str>>) -> f32 {
    match selected.as_deref() {
        Some("apple") => 0.0,
        Some("banana") => 1.0,
        Some("blueberry") => 2.0,
        _ => -1.0,
    }
}

fn default_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    )
}

fn themed_app() -> App {
    let mut app = App::new();
    shadcn::themes::apply_shadcn_new_york(
        &mut app,
        shadcn::themes::ShadcnBaseColor::Neutral,
        shadcn::themes::ShadcnColorScheme::Light,
    );
    app
}

impl TypeaheadKey {
    fn key_code(self) -> KeyCode {
        match self {
            Self::KeyB => KeyCode::KeyB,
            Self::KeyO => KeyCode::KeyO,
            Self::KeyX => KeyCode::KeyX,
        }
    }
}
