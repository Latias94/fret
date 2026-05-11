use std::sync::Arc;

use crate::action::UiActionHost;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, NodeId, Point, PointerType, Px, Rect, Size,
};
use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

use super::*;
use crate::element::{AnyElement, CrossAlign, MainAlign};
use crate::elements::ElementContext;

const FOCUS_SCOPE_STALE_PARENT_INTERACTION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tree/tests/fixtures/focus_scope_stale_parent_interaction_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FocusScopeStaleParentScenario {
    FocusScopeStaleParent(FocusScopeStaleParentCase),
}

#[derive(Debug, Clone, Deserialize)]
struct FocusScopeStaleParentCase {
    trap_focus: bool,
    initial: FocusTarget,
    stale_parent_pointers: bool,
    steps: Vec<FocusScopeStep>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FocusScopeStep {
    PointerClick { target: FocusTarget },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FocusTarget {
    Before,
    InsideA,
    InsideB,
    After,
}

#[derive(Default)]
struct FocusScopeElementIds {
    before: Option<crate::elements::GlobalElementId>,
    scope: Option<crate::elements::GlobalElementId>,
    inside_a: Option<crate::elements::GlobalElementId>,
    inside_b: Option<crate::elements::GlobalElementId>,
    after: Option<crate::elements::GlobalElementId>,
}

#[test]
fn mechanism_harness_focus_scope_stale_parent_interaction_matches_oracles() {
    let suite: MechanismSuite<FocusScopeStaleParentScenario> =
        MechanismSuite::from_json_str(FOCUS_SCOPE_STALE_PARENT_INTERACTION)
            .expect("stale-parent focus scope fixture suite");

    let mut observer: fn(
        &MechanismCase<FocusScopeStaleParentScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<FocusScopeStaleParentScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        FocusScopeStaleParentScenario::FocusScopeStaleParent(scenario) => {
            observe_stale_parent_focus_scope(scenario)
        }
    }
}

fn observe_stale_parent_focus_scope(
    scenario: &FocusScopeStaleParentCase,
) -> Result<ObservedTree, ScenarioObserveError> {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );
    let mut services = FakeUiServices;
    let after_clicked = app.models_mut().insert(false);
    let mut ids = FocusScopeElementIds::default();

    let root = crate::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-stale-parent-focus-scope",
        |cx| build_focus_scope(cx, scenario.trap_focus, after_clicked.clone(), &mut ids),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let initial = ids
        .node_for(&mut app, window, scenario.initial)
        .ok_or_else(|| ScenarioObserveError::new("missing initial focus target"))?;
    ui.set_focus(Some(initial));

    if scenario.stale_parent_pointers {
        let inside_a_node = ids
            .node_for(&mut app, window, FocusTarget::InsideA)
            .ok_or_else(|| ScenarioObserveError::new("missing stale-parent inside a node"))?;
        let inside_b_node = ids
            .node_for(&mut app, window, FocusTarget::InsideB)
            .ok_or_else(|| ScenarioObserveError::new("missing stale-parent inside b node"))?;
        ui.nodes.get_mut(inside_a_node).unwrap().parent = None;
        ui.nodes.get_mut(inside_b_node).unwrap().parent = None;
    }

    let inside_a_node = ids
        .node_for(&mut app, window, FocusTarget::InsideA)
        .ok_or_else(|| ScenarioObserveError::new("missing stale-parent inside a node"))?;
    let (active_roots, barrier_root) = ui.active_input_layers();
    let dispatch_snapshot = ui.build_dispatch_snapshot_for_layer_roots(
        FrameId(1),
        active_roots.as_slice(),
        barrier_root,
    );
    let child_reachable = dispatch_snapshot.pre.get(inside_a_node).is_some();

    for step in &scenario.steps {
        match step {
            FocusScopeStep::PointerClick { target } => {
                let node = ids.node_for(&mut app, window, *target).ok_or_else(|| {
                    ScenarioObserveError::new("missing stale-parent pointer click target")
                })?;
                dispatch_pointer_click(&mut ui, &mut app, &mut services, node)?;
            }
        }
    }

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing semantics snapshot"))?;
    let mut observed = ObservedTree::from_semantics_snapshot(&snapshot, bounds);
    observed.set_metric(
        "stale_parent.child_reachable",
        if child_reachable { 1.0 } else { 0.0 },
    );
    observed.set_metric(
        "focus.in_scope",
        if ids.focus_is_inside(&mut app, window, ui.focus()) {
            1.0
        } else {
            0.0
        },
    );
    observed.set_metric(
        "scope.after.activated",
        if app.models().get_copied(&after_clicked) == Some(true) {
            1.0
        } else {
            0.0
        },
    );
    Ok(observed)
}

fn build_focus_scope(
    cx: &mut ElementContext<'_, crate::test_host::TestHost>,
    trap_focus: bool,
    after_activated: fret_runtime::Model<bool>,
    ids: &mut FocusScopeElementIds,
) -> Vec<AnyElement> {
    vec![cx.flex(
        crate::element::FlexProps {
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0).into(),
            padding: fret_core::Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Start,
            ..Default::default()
        },
        |cx| {
            vec![
                build_focus_button(cx, "Before", "stale-before", &mut ids.before, None),
                cx.focus_scope_with_id(
                    crate::element::FocusScopeProps {
                        trap_focus,
                        ..Default::default()
                    },
                    |cx, scope_id| {
                        ids.scope = Some(scope_id);
                        vec![
                            build_focus_button(
                                cx,
                                "Inside A",
                                "stale-inside-a",
                                &mut ids.inside_a,
                                None,
                            ),
                            build_focus_button(
                                cx,
                                "Inside B",
                                "stale-inside-b",
                                &mut ids.inside_b,
                                None,
                            ),
                        ]
                    },
                ),
                build_focus_button(
                    cx,
                    "After",
                    "stale-after",
                    &mut ids.after,
                    Some(after_activated),
                ),
            ]
        },
    )]
}

fn build_focus_button(
    cx: &mut ElementContext<'_, crate::test_host::TestHost>,
    label: &'static str,
    test_id: &'static str,
    out: &mut Option<crate::elements::GlobalElementId>,
    on_activate_model: Option<fret_runtime::Model<bool>>,
) -> AnyElement {
    cx.pressable_with_id(
        crate::element::PressableProps {
            layout: {
                let mut layout = crate::element::LayoutStyle::default();
                layout.size.width = crate::element::Length::Px(Px(40.0));
                layout.size.height = crate::element::Length::Px(Px(24.0));
                layout
            },
            a11y: crate::element::PressableA11y {
                role: Some(fret_core::SemanticsRole::Button),
                label: Some(Arc::from(label)),
                test_id: Some(Arc::from(test_id)),
                ..Default::default()
            },
            ..Default::default()
        },
        |cx: &mut ElementContext<'_, crate::test_host::TestHost>, _state, id| {
            *out = Some(id);
            if let Some(model) = on_activate_model {
                cx.pressable_on_activate(Arc::new(
                    move |host: &mut dyn UiActionHost, _cx, _reason| {
                        let _ = host.models_mut().update(&model, |v| *v = true);
                    },
                ));
            }
            vec![cx.text(label)]
        },
    )
}

fn dispatch_pointer_click(
    ui: &mut UiTree<crate::test_host::TestHost>,
    app: &mut crate::test_host::TestHost,
    services: &mut FakeUiServices,
    target: NodeId,
) -> Result<(), ScenarioObserveError> {
    let bounds = ui
        .debug_node_bounds(target)
        .ok_or_else(|| ScenarioObserveError::new("missing pointer target bounds"))?;
    let position = Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 / 2.0),
        Px(bounds.origin.y.0 + bounds.size.height.0 / 2.0),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    Ok(())
}

impl FocusScopeElementIds {
    fn node_for(
        &self,
        app: &mut crate::test_host::TestHost,
        window: AppWindowId,
        target: FocusTarget,
    ) -> Option<NodeId> {
        let element = match target {
            FocusTarget::Before => self.before,
            FocusTarget::InsideA => self.inside_a,
            FocusTarget::InsideB => self.inside_b,
            FocusTarget::After => self.after,
        }?;
        crate::elements::node_for_element(app, window, element)
    }

    fn focus_is_inside(
        &self,
        app: &mut crate::test_host::TestHost,
        window: AppWindowId,
        focus: Option<NodeId>,
    ) -> bool {
        let Some(focus) = focus else {
            return false;
        };
        [FocusTarget::InsideA, FocusTarget::InsideB]
            .into_iter()
            .filter_map(|target| self.node_for(app, window, target))
            .any(|node| node == focus)
    }
}
