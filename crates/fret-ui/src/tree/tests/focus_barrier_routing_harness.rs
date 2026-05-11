use std::collections::HashMap;

use super::*;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

const FOCUS_BARRIER_ROUTING: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tree/tests/fixtures/focus_barrier_routing_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FocusBarrierRoutingScenario {
    FocusBarrierRouting {
        #[serde(default)]
        overlay_blocks_underlay_input: bool,
        #[serde(default = "default_overlay_hit_testable")]
        overlay_hit_testable: bool,
        steps: Vec<FocusStep>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum FocusStep {
    CaptureMetrics {
        label: String,
    },
    DispatchCommand {
        command: FocusCommand,
        label: String,
    },
    DispatchTimer {
        token: u64,
        label: String,
    },
    SetFocus {
        target: FocusTarget,
    },
    SetFocusBarrier {
        enabled: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FocusTarget {
    OverlayA,
    OverlayB,
    Underlay,
}

#[derive(Debug, Clone, Copy, Deserialize)]
enum FocusCommand {
    #[serde(rename = "focus.next")]
    Next,
    #[serde(rename = "focus.previous")]
    Previous,
}

impl FocusCommand {
    fn id(self) -> CommandId {
        match self {
            FocusCommand::Next => CommandId::from("focus.next"),
            FocusCommand::Previous => CommandId::from("focus.previous"),
        }
    }
}

impl FocusTarget {
    fn metric_name(self) -> &'static str {
        match self {
            FocusTarget::OverlayA => "overlay_a",
            FocusTarget::OverlayB => "overlay_b",
            FocusTarget::Underlay => "underlay",
        }
    }
}

#[derive(Default)]
struct HarnessFocusable;

impl<H: UiHost> Widget<H> for HarnessFocusable {
    fn is_focusable(&self) -> bool {
        true
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct FocusHarnessNodes {
    overlay_root: NodeId,
    overlay_layer: UiLayerId,
    nodes_by_target: HashMap<FocusTarget, NodeId>,
}

#[test]
fn mechanism_harness_focus_barrier_routing_matches_oracles() {
    let suite: MechanismSuite<FocusBarrierRoutingScenario> =
        MechanismSuite::from_json_str(FOCUS_BARRIER_ROUTING)
            .expect("focus barrier routing fixture suite");

    let mut observer: fn(
        &MechanismCase<FocusBarrierRoutingScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<FocusBarrierRoutingScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        FocusBarrierRoutingScenario::FocusBarrierRouting {
            overlay_blocks_underlay_input,
            overlay_hit_testable,
            steps,
        } => observe_focus_barrier_routing_case(
            *overlay_blocks_underlay_input,
            *overlay_hit_testable,
            steps,
        ),
    }
}

fn observe_focus_barrier_routing_case(
    overlay_blocks_underlay_input: bool,
    overlay_hit_testable: bool,
    steps: &[FocusStep],
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let harness = build_focus_harness(&mut ui, overlay_blocks_underlay_input, overlay_hit_testable);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut observed = ObservedTree::new(bounds);
    for step in steps {
        apply_focus_step(
            &mut app,
            &mut ui,
            &mut services,
            &harness,
            step,
            &mut observed,
        )?;
    }

    append_focus_metrics(&ui, &harness, &mut observed, None);
    Ok(observed)
}

fn build_focus_harness(
    ui: &mut UiTree<crate::test_host::TestHost>,
    overlay_blocks_underlay_input: bool,
    overlay_hit_testable: bool,
) -> FocusHarnessNodes {
    let base_root = ui.create_node(TestStack);
    let underlay = ui.create_node(HarnessFocusable);
    ui.add_child(base_root, underlay);
    ui.set_root(base_root);

    let overlay_root = ui.create_node(TestStack);
    let overlay_a = ui.create_node(HarnessFocusable);
    let overlay_b = ui.create_node(HarnessFocusable);
    ui.add_child(overlay_root, overlay_a);
    ui.add_child(overlay_root, overlay_b);
    let overlay_layer = ui.push_overlay_root_with_options(
        overlay_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: overlay_blocks_underlay_input,
            hit_testable: overlay_hit_testable,
        },
    );

    let nodes_by_target = HashMap::from([
        (FocusTarget::Underlay, underlay),
        (FocusTarget::OverlayA, overlay_a),
        (FocusTarget::OverlayB, overlay_b),
    ]);

    FocusHarnessNodes {
        overlay_root,
        overlay_layer,
        nodes_by_target,
    }
}

fn apply_focus_step(
    app: &mut crate::test_host::TestHost,
    ui: &mut UiTree<crate::test_host::TestHost>,
    services: &mut dyn UiServices,
    harness: &FocusHarnessNodes,
    step: &FocusStep,
    observed: &mut ObservedTree,
) -> Result<(), ScenarioObserveError> {
    match step {
        FocusStep::CaptureMetrics { label } => {
            append_focus_metrics(ui, harness, observed, Some(label));
        }
        FocusStep::DispatchCommand { command, label } => {
            let handled = ui.dispatch_command(app, services, &command.id());
            set_metric(
                observed,
                None,
                format!("command.{label}.handled"),
                bool_metric(handled),
            );
            append_focus_metrics(ui, harness, observed, Some(label));
        }
        FocusStep::DispatchTimer { token, label } => {
            ui.dispatch_event(
                app,
                services,
                &Event::Timer {
                    token: fret_core::TimerToken(*token),
                },
            );
            append_focus_metrics(ui, harness, observed, Some(label));
        }
        FocusStep::SetFocus { target } => {
            let node = *harness
                .nodes_by_target
                .get(target)
                .ok_or_else(|| ScenarioObserveError::new("unknown focus target"))?;
            ui.set_focus(Some(node));
        }
        FocusStep::SetFocusBarrier { enabled } => {
            ui.set_layer_blocks_underlay_focus(harness.overlay_layer, *enabled);
        }
    }
    Ok(())
}

fn append_focus_metrics(
    ui: &UiTree<crate::test_host::TestHost>,
    harness: &FocusHarnessNodes,
    observed: &mut ObservedTree,
    prefix: Option<&str>,
) {
    let focus = ui.focus();
    set_metric(observed, prefix, "focus.none", bool_metric(focus.is_none()));
    for (target, node) in &harness.nodes_by_target {
        set_metric(
            observed,
            prefix,
            format!("focus.{}", target.metric_name()),
            bool_metric(focus == Some(*node)),
        );
    }

    let arbitration = ui.input_arbitration_snapshot();
    set_metric(
        observed,
        prefix,
        "arbitration.focus_barrier_present",
        bool_metric(arbitration.focus_barrier_root.is_some()),
    );
    set_metric(
        observed,
        prefix,
        "arbitration.focus_barrier_is_overlay_root",
        bool_metric(arbitration.focus_barrier_root == Some(harness.overlay_root)),
    );
    set_metric(
        observed,
        prefix,
        "arbitration.modal_barrier_present",
        bool_metric(arbitration.modal_barrier_root.is_some()),
    );
    set_metric(
        observed,
        prefix,
        "arbitration.modal_barrier_is_overlay_root",
        bool_metric(arbitration.modal_barrier_root == Some(harness.overlay_root)),
    );
}

fn set_metric(
    observed: &mut ObservedTree,
    prefix: Option<&str>,
    id: impl Into<String>,
    value: f32,
) {
    observed.set_metric(metric_id(prefix, id.into()), value);
}

fn metric_id(prefix: Option<&str>, id: String) -> String {
    if let Some(prefix) = prefix {
        format!("capture.{prefix}.{id}")
    } else {
        id
    }
}

fn bool_metric(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}

fn default_overlay_hit_testable() -> bool {
    true
}
