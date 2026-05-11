use std::sync::Arc;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

use super::*;

const FOCUS_SCOPE_INTERACTION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/focus_scope_interaction_v1.json"
));
const NESTED_FOCUS_SCOPE_INTERACTION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/focus_scope_nested_interaction_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FocusScopeScenario {
    FocusScope(FocusScopeCase),
    NestedFocusScope(NestedFocusScopeCase),
}

#[derive(Debug, Clone, Deserialize)]
struct FocusScopeCase {
    trap_focus: bool,
    initial: FocusTarget,
    steps: Vec<FocusScopeStep>,
}

#[derive(Debug, Clone, Deserialize)]
struct NestedFocusScopeCase {
    outer_trap_focus: bool,
    inner_trap_focus: bool,
    initial: NestedFocusTarget,
    steps: Vec<NestedFocusScopeStep>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FocusScopeStep {
    Command { command: FocusCommand },
    PointerClick { target: FocusTarget },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum NestedFocusScopeStep {
    Command { command: FocusCommand },
    PointerClick { target: NestedFocusTarget },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FocusTarget {
    Before,
    InsideA,
    InsideB,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NestedFocusTarget {
    OuterBefore,
    InnerA,
    InnerB,
    OuterSibling,
    OuterAfter,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FocusCommand {
    Next,
    Previous,
}

#[derive(Default)]
struct FocusScopeElementIds {
    before: Option<crate::elements::GlobalElementId>,
    inside_a: Option<crate::elements::GlobalElementId>,
    inside_b: Option<crate::elements::GlobalElementId>,
    after: Option<crate::elements::GlobalElementId>,
}

#[derive(Default)]
struct NestedFocusScopeElementIds {
    outer_before: Option<crate::elements::GlobalElementId>,
    outer_scope: Option<crate::elements::GlobalElementId>,
    inner_scope: Option<crate::elements::GlobalElementId>,
    inner_a: Option<crate::elements::GlobalElementId>,
    inner_b: Option<crate::elements::GlobalElementId>,
    outer_sibling: Option<crate::elements::GlobalElementId>,
    outer_after: Option<crate::elements::GlobalElementId>,
}

#[test]
fn mechanism_harness_focus_scope_interaction_matches_oracles() {
    let suite: MechanismSuite<FocusScopeScenario> =
        MechanismSuite::from_json_str(FOCUS_SCOPE_INTERACTION).expect("focus scope fixture suite");

    let mut observer: fn(
        &MechanismCase<FocusScopeScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

#[test]
fn mechanism_harness_nested_focus_scope_interaction_matches_oracles() {
    let suite: MechanismSuite<FocusScopeScenario> =
        MechanismSuite::from_json_str(NESTED_FOCUS_SCOPE_INTERACTION)
            .expect("nested focus scope fixture suite");

    let mut observer: fn(
        &MechanismCase<FocusScopeScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<FocusScopeScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        FocusScopeScenario::FocusScope(scenario) => observe_focus_scope(scenario),
        FocusScopeScenario::NestedFocusScope(scenario) => observe_nested_focus_scope(scenario),
    }
}

fn observe_focus_scope(scenario: &FocusScopeCase) -> Result<ObservedTree, ScenarioObserveError> {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(260.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();
    let after_activated = app.models_mut().insert(false);
    let mut ids = FocusScopeElementIds::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-focus-scope",
        |cx| build_focus_scope(cx, scenario.trap_focus, after_activated.clone(), &mut ids),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let initial = ids
        .node_for(&mut app, window, scenario.initial)
        .ok_or_else(|| ScenarioObserveError::new("missing initial focus target"))?;
    ui.set_focus(Some(initial));

    for step in &scenario.steps {
        match step {
            FocusScopeStep::Command { command } => {
                let handled = ui.dispatch_command(&mut app, &mut services, &command.command_id());
                if !handled {
                    return Err(ScenarioObserveError::new(format!(
                        "focus command was not handled: {command:?}"
                    )));
                }
            }
            FocusScopeStep::PointerClick { target } => {
                let node = ids
                    .node_for(&mut app, window, *target)
                    .ok_or_else(|| ScenarioObserveError::new("missing pointer click target"))?;
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
        "focus.in_scope",
        if ids.focus_is_inside(&mut app, window, ui.focus()) {
            1.0
        } else {
            0.0
        },
    );
    observed.set_metric(
        "scope.after.activated",
        if app.models().get_copied(&after_activated) == Some(true) {
            1.0
        } else {
            0.0
        },
    );
    Ok(observed)
}

fn observe_nested_focus_scope(
    scenario: &NestedFocusScopeCase,
) -> Result<ObservedTree, ScenarioObserveError> {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(140.0)),
    );
    let mut services = FakeTextService::default();
    let outer_after_activated = app.models_mut().insert(false);
    let mut ids = NestedFocusScopeElementIds::default();

    let _root = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-nested-focus-scope",
        |cx| {
            build_nested_focus_scope(
                cx,
                scenario.outer_trap_focus,
                scenario.inner_trap_focus,
                outer_after_activated.clone(),
                &mut ids,
            )
        },
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let initial = ids
        .node_for(&mut app, window, scenario.initial)
        .ok_or_else(|| ScenarioObserveError::new("missing initial nested focus target"))?;
    ui.set_focus(Some(initial));

    for step in &scenario.steps {
        match step {
            NestedFocusScopeStep::Command { command } => {
                let handled = ui.dispatch_command(&mut app, &mut services, &command.command_id());
                if !handled {
                    return Err(ScenarioObserveError::new(format!(
                        "focus command was not handled: {command:?}"
                    )));
                }
            }
            NestedFocusScopeStep::PointerClick { target } => {
                let node = ids.node_for(&mut app, window, *target).ok_or_else(|| {
                    ScenarioObserveError::new("missing nested pointer click target")
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
        "nested.focus.in_inner_scope",
        if ids.focus_is_within_inner_scope(&mut app, window, ui.focus()) {
            1.0
        } else {
            0.0
        },
    );
    observed.set_metric(
        "nested.outer.after.activated",
        if app.models().get_copied(&outer_after_activated) == Some(true) {
            1.0
        } else {
            0.0
        },
    );
    Ok(observed)
}

fn build_nested_focus_scope(
    cx: &mut ElementContext<'_, TestHost>,
    outer_trap_focus: bool,
    inner_trap_focus: bool,
    outer_after_activated: fret_runtime::Model<bool>,
    ids: &mut NestedFocusScopeElementIds,
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
                build_focus_button(
                    cx,
                    "Outer Before",
                    "nested-outer-before",
                    &mut ids.outer_before,
                    None,
                ),
                cx.focus_scope_with_id(
                    crate::element::FocusScopeProps {
                        trap_focus: outer_trap_focus,
                        ..Default::default()
                    },
                    |cx, outer_scope_id| {
                        ids.outer_scope = Some(outer_scope_id);
                        vec![
                            cx.focus_scope_with_id(
                                crate::element::FocusScopeProps {
                                    trap_focus: inner_trap_focus,
                                    ..Default::default()
                                },
                                |cx, inner_scope_id| {
                                    ids.inner_scope = Some(inner_scope_id);
                                    vec![
                                        build_focus_button(
                                            cx,
                                            "Inner A",
                                            "nested-inner-a",
                                            &mut ids.inner_a,
                                            None,
                                        ),
                                        build_focus_button(
                                            cx,
                                            "Inner B",
                                            "nested-inner-b",
                                            &mut ids.inner_b,
                                            None,
                                        ),
                                    ]
                                },
                            ),
                            build_focus_button(
                                cx,
                                "Outer Sibling",
                                "nested-outer-sibling",
                                &mut ids.outer_sibling,
                                None,
                            ),
                        ]
                    },
                ),
                build_focus_button(
                    cx,
                    "Outer After",
                    "nested-outer-after",
                    &mut ids.outer_after,
                    Some(outer_after_activated),
                ),
            ]
        },
    )]
}

fn build_focus_scope(
    cx: &mut ElementContext<'_, TestHost>,
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
                build_focus_button(cx, "Before", "scope-before", &mut ids.before, None),
                cx.focus_scope_with_id(
                    crate::element::FocusScopeProps {
                        trap_focus,
                        ..Default::default()
                    },
                    |cx, _id| {
                        vec![
                            build_focus_button(
                                cx,
                                "Inside A",
                                "scope-inside-a",
                                &mut ids.inside_a,
                                None,
                            ),
                            build_focus_button(
                                cx,
                                "Inside B",
                                "scope-inside-b",
                                &mut ids.inside_b,
                                None,
                            ),
                        ]
                    },
                ),
                build_focus_button(
                    cx,
                    "After",
                    "scope-after",
                    &mut ids.after,
                    Some(after_activated),
                ),
            ]
        },
    )]
}

fn build_focus_button(
    cx: &mut ElementContext<'_, TestHost>,
    label: &'static str,
    test_id: &'static str,
    out: &mut Option<crate::elements::GlobalElementId>,
    on_activate_model: Option<fret_runtime::Model<bool>>,
) -> AnyElement {
    cx.pressable_with_id(
        crate::element::PressableProps {
            layout: {
                let mut layout = crate::element::LayoutStyle::default();
                layout.size.width = Length::Px(Px(40.0));
                layout.size.height = Length::Px(Px(24.0));
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
        |cx, _state, id| {
            *out = Some(id);
            if let Some(model) = on_activate_model {
                cx.pressable_on_activate(Arc::new(move |host, _cx, _reason| {
                    let _ = host.models_mut().update(&model, |activated| {
                        *activated = true;
                    });
                }));
            }
            vec![cx.text(label)]
        },
    )
}

fn dispatch_pointer_click(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
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
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    Ok(())
}

impl FocusScopeElementIds {
    fn node_for(
        &self,
        app: &mut TestHost,
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
        app: &mut TestHost,
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

impl NestedFocusScopeElementIds {
    fn node_for(
        &self,
        app: &mut TestHost,
        window: AppWindowId,
        target: NestedFocusTarget,
    ) -> Option<NodeId> {
        let element = match target {
            NestedFocusTarget::OuterBefore => self.outer_before,
            NestedFocusTarget::InnerA => self.inner_a,
            NestedFocusTarget::InnerB => self.inner_b,
            NestedFocusTarget::OuterSibling => self.outer_sibling,
            NestedFocusTarget::OuterAfter => self.outer_after,
        }?;
        crate::elements::node_for_element(app, window, element)
    }

    fn focus_is_within_inner_scope(
        &self,
        app: &mut TestHost,
        window: AppWindowId,
        focus: Option<NodeId>,
    ) -> bool {
        let Some(focus) = focus else {
            return false;
        };
        [NestedFocusTarget::InnerA, NestedFocusTarget::InnerB]
            .into_iter()
            .filter_map(|target| self.node_for(app, window, target))
            .any(|node| node == focus)
    }
}

impl FocusCommand {
    fn command_id(self) -> CommandId {
        match self {
            Self::Next => CommandId::from("focus.next"),
            Self::Previous => CommandId::from("focus.previous"),
        }
    }
}
