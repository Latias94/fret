use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::*;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

const TIMER_DISPATCH: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tree/tests/fixtures/timer_dispatch_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum TimerDispatchScenario {
    TimerDispatch {
        #[serde(default = "default_overlay_visible")]
        overlay_visible: bool,
        #[serde(default = "default_overlay_hit_testable")]
        overlay_hit_testable: bool,
        #[serde(default)]
        remove_overlay_before_dispatch: bool,
        steps: Vec<TimerStep>,
    },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TimerTargetFixture {
    Base,
    Overlay,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum TimerStep {
    CaptureMetrics {
        label: String,
    },
    DispatchTimer {
        token: u64,
        label: String,
    },
    RecordTimerTarget {
        target: TimerTargetFixture,
        token: u64,
    },
}

#[derive(Default, Clone)]
struct TimerHits {
    hits: Arc<AtomicUsize>,
}

impl TimerHits {
    fn hit(&self) {
        self.hits.fetch_add(1, Ordering::SeqCst);
    }

    fn load(&self) -> f32 {
        self.hits.load(Ordering::SeqCst) as f32
    }
}

struct TimerProbe {
    hits: TimerHits,
}

impl<H: UiHost> Widget<H> for TimerProbe {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if matches!(event, Event::Timer { .. }) {
            self.hits.hit();
            cx.stop_propagation();
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct TimerHarnessNodes {
    base_element: crate::elements::GlobalElementId,
    overlay_element: crate::elements::GlobalElementId,
    overlay_layer: Option<UiLayerId>,
    base_hits: TimerHits,
    overlay_hits: TimerHits,
}

#[test]
fn mechanism_harness_timer_dispatch_matches_oracles() {
    let suite: MechanismSuite<TimerDispatchScenario> =
        MechanismSuite::from_json_str(TIMER_DISPATCH).expect("timer dispatch fixture suite");

    let mut observer: fn(
        &MechanismCase<TimerDispatchScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<TimerDispatchScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        TimerDispatchScenario::TimerDispatch {
            overlay_visible,
            overlay_hit_testable,
            remove_overlay_before_dispatch,
            steps,
        } => observe_timer_dispatch_case(
            *overlay_visible,
            *overlay_hit_testable,
            *remove_overlay_before_dispatch,
            steps,
        ),
    }
}

fn observe_timer_dispatch_case(
    overlay_visible: bool,
    overlay_hit_testable: bool,
    remove_overlay_before_dispatch: bool,
    steps: &[TimerStep],
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    let harness = build_timer_harness(
        &mut ui,
        &mut services,
        overlay_visible,
        overlay_hit_testable,
        remove_overlay_before_dispatch,
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut observed = ObservedTree::new(bounds);
    for step in steps {
        apply_timer_step(
            &mut app,
            &mut ui,
            &mut services,
            window,
            &harness,
            step,
            &mut observed,
        );
    }

    append_timer_metrics(&harness, &mut observed, None);
    Ok(observed)
}

fn build_timer_harness(
    ui: &mut UiTree<crate::test_host::TestHost>,
    services: &mut dyn UiServices,
    overlay_visible: bool,
    overlay_hit_testable: bool,
    remove_overlay_before_dispatch: bool,
) -> TimerHarnessNodes {
    let base_hits = TimerHits::default();
    let overlay_hits = TimerHits::default();
    let base_element = crate::elements::GlobalElementId(6201);
    let overlay_element = crate::elements::GlobalElementId(6202);

    let base_root = ui.create_node_for_element(
        base_element,
        TimerProbe {
            hits: base_hits.clone(),
        },
    );
    ui.set_root(base_root);

    let overlay_root = ui.create_node_for_element(
        overlay_element,
        TimerProbe {
            hits: overlay_hits.clone(),
        },
    );
    let overlay_layer = ui.push_overlay_root_with_options(
        overlay_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: false,
            hit_testable: overlay_hit_testable,
        },
    );
    ui.set_layer_visible(overlay_layer, overlay_visible);
    let overlay_layer = if remove_overlay_before_dispatch {
        let _ = ui.remove_layer(services, overlay_layer);
        None
    } else {
        Some(overlay_layer)
    };

    TimerHarnessNodes {
        base_element,
        overlay_element,
        overlay_layer,
        base_hits,
        overlay_hits,
    }
}

fn apply_timer_step(
    app: &mut crate::test_host::TestHost,
    ui: &mut UiTree<crate::test_host::TestHost>,
    services: &mut dyn UiServices,
    window: AppWindowId,
    harness: &TimerHarnessNodes,
    step: &TimerStep,
    observed: &mut ObservedTree,
) {
    match step {
        TimerStep::CaptureMetrics { label } => {
            append_timer_metrics(harness, observed, Some(label));
        }
        TimerStep::DispatchTimer { token, label } => {
            ui.dispatch_event(
                app,
                services,
                &Event::Timer {
                    token: fret_core::TimerToken(*token),
                },
            );
            append_timer_metrics(harness, observed, Some(label));
        }
        TimerStep::RecordTimerTarget { target, token } => {
            let element = match target {
                TimerTargetFixture::Base => harness.base_element,
                TimerTargetFixture::Overlay => harness.overlay_element,
            };
            crate::elements::record_timer_target(
                app,
                window,
                fret_core::TimerToken(*token),
                element,
            );
        }
    }
}

fn append_timer_metrics(
    harness: &TimerHarnessNodes,
    observed: &mut ObservedTree,
    prefix: Option<&str>,
) {
    set_metric(
        observed,
        prefix,
        "base.timer_hits",
        harness.base_hits.load(),
    );
    set_metric(
        observed,
        prefix,
        "overlay.timer_hits",
        harness.overlay_hits.load(),
    );
    set_metric(
        observed,
        prefix,
        "overlay.layer_present",
        bool_metric(harness.overlay_layer.is_some()),
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

fn default_overlay_visible() -> bool {
    true
}

fn default_overlay_hit_testable() -> bool {
    true
}
