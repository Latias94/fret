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
use input_events::dispatch_key_press;

const RECIPE_SEMANTICS_CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/recipe_semantics_cases_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RecipeSemanticsScenario {
    ComboboxOpen {
        #[serde(default)]
        selected: Option<String>,
        #[serde(default)]
        navigation: Vec<NavigationStep>,
    },
    SelectOpen {
        #[serde(default)]
        selected: Option<String>,
        #[serde(default)]
        navigation: Vec<NavigationStep>,
    },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NavigationStep {
    ArrowDown,
}

#[test]
fn mechanism_harness_recipe_semantics_cases_match_oracles() {
    let suite: MechanismSuite<RecipeSemanticsScenario> =
        MechanismSuite::from_json_str(RECIPE_SEMANTICS_CASES)
            .expect("recipe semantics fixture suite");

    let mut observer: fn(
        &MechanismCase<RecipeSemanticsScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<RecipeSemanticsScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        RecipeSemanticsScenario::ComboboxOpen {
            selected,
            navigation,
        } => observe_combobox_open(selected.as_deref(), navigation),
        RecipeSemanticsScenario::SelectOpen {
            selected,
            navigation,
        } => observe_select_open(selected.as_deref(), navigation),
    }
}

fn observe_combobox_open(
    selected: Option<&str>,
    navigation: &[NavigationStep],
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(selected.map(Arc::from));
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_combobox_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        value.clone(),
        open.clone(),
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    render_combobox_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
    );
    render_combobox_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
    );

    if !navigation.is_empty() {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .ok_or_else(|| ScenarioObserveError::new("missing combobox snapshot before nav"))?;
        let input = find_by_test_id(&snap, "semantics-combobox-input");
        ui.set_focus(Some(input.id));
        render_combobox_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            value.clone(),
            open.clone(),
        );
        for step in navigation {
            dispatch_key_press(&mut ui, &mut app, &mut services, step.key_code());
        }
        render_combobox_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            value.clone(),
            open.clone(),
        );
        render_combobox_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            value,
            open,
        );
    }

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing final combobox semantics snapshot"))?;
    Ok(ObservedTree::from_semantics_snapshot(&snapshot, bounds))
}

fn observe_select_open(
    selected: Option<&str>,
    navigation: &[NavigationStep],
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(selected.map(Arc::from));
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_select_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        value.clone(),
        open.clone(),
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    render_select_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
    );
    render_select_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
    );

    if !navigation.is_empty() {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .ok_or_else(|| ScenarioObserveError::new("missing select snapshot before nav"))?;
        let trigger = find_by_test_id(&snap, "semantics-select-trigger");
        ui.set_focus(Some(trigger.id));
        for step in navigation {
            dispatch_key_press(&mut ui, &mut app, &mut services, step.key_code());
        }
        render_select_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            value.clone(),
            open.clone(),
        );
        render_select_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            value,
            open,
        );
    }

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing final select semantics snapshot"))?;
    Ok(ObservedTree::from_semantics_snapshot(&snapshot, bounds))
}

fn render_combobox_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-recipe-semantics-combobox",
        request_semantics,
        move |cx| {
            vec![
                shadcn::Combobox::new(value, open)
                    .a11y_label("Framework")
                    .test_id_prefix("semantics-combobox")
                    .items(framework_items())
                    .into_element(cx),
            ]
        },
    );
}

fn render_select_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-recipe-semantics-select",
        request_semantics,
        move |cx| {
            vec![
                shadcn::Select::new(value, open)
                    .value(shadcn::SelectValue::new().placeholder("Select"))
                    .a11y_label("Framework")
                    .trigger_test_id("semantics-select-trigger")
                    .test_id_prefix("semantics-select")
                    .items(select_items())
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

fn framework_items() -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("alpha", "Alpha"),
        shadcn::ComboboxItem::new("beta", "Beta"),
        shadcn::ComboboxItem::new("gamma", "Gamma"),
    ]
}

fn select_items() -> Vec<shadcn::SelectItem> {
    vec![
        shadcn::SelectItem::new("alpha", "Alpha").test_id("semantics-select-item-alpha"),
        shadcn::SelectItem::new("beta", "Beta").test_id("semantics-select-item-beta"),
        shadcn::SelectItem::new("gamma", "Gamma").test_id("semantics-select-item-gamma"),
    ]
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

impl NavigationStep {
    fn key_code(self) -> KeyCode {
        match self {
            Self::ArrowDown => KeyCode::ArrowDown,
        }
    }
}
