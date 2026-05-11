use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::*;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

const POINTER_OCCLUSION_ROUTING: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tree/tests/fixtures/pointer_occlusion_routing_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PointerOcclusionRoutingScenario {
    PointerOcclusionRouting {
        #[serde(default)]
        underlay_capture_on_down: bool,
        overlay_kind: OverlayKindFixture,
        #[serde(default)]
        overlay_blocks_underlay_input: bool,
        #[serde(default = "default_overlay_hit_testable")]
        overlay_hit_testable: bool,
        pointer_occlusion: PointerOcclusionFixture,
        #[serde(default)]
        wants_pointer_down_outside: bool,
        #[serde(default)]
        wants_pointer_move: bool,
        #[serde(default)]
        hit_test_before: Option<PointFixture>,
        #[serde(rename = "events")]
        steps: Vec<PointerEventFixture>,
    },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OverlayKindFixture {
    CornerCapture,
    ObserverDown,
    ObserverMove,
    Transparent,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PointerOcclusionFixture {
    BlockMouse,
    BlockMouseExceptScroll,
    None,
}

impl From<PointerOcclusionFixture> for PointerOcclusion {
    fn from(value: PointerOcclusionFixture) -> Self {
        match value {
            PointerOcclusionFixture::BlockMouse => PointerOcclusion::BlockMouse,
            PointerOcclusionFixture::BlockMouseExceptScroll => {
                PointerOcclusion::BlockMouseExceptScroll
            }
            PointerOcclusionFixture::None => PointerOcclusion::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct PointFixture {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PointerEventFixture {
    Down {
        x: f32,
        y: f32,
        #[serde(default)]
        pointer_id: u64,
        #[serde(default)]
        pointer_type: PointerTypeFixture,
    },
    Move {
        x: f32,
        y: f32,
        #[serde(default)]
        pointer_id: u64,
        #[serde(default)]
        pointer_type: PointerTypeFixture,
        #[serde(default)]
        buttons_left: bool,
    },
    Wheel {
        x: f32,
        y: f32,
        #[serde(default)]
        pointer_id: u64,
        #[serde(default)]
        pointer_type: PointerTypeFixture,
    },
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PointerTypeFixture {
    #[default]
    Mouse,
    Pen,
    Touch,
    Unknown,
}

impl From<PointerTypeFixture> for fret_core::PointerType {
    fn from(value: PointerTypeFixture) -> Self {
        match value {
            PointerTypeFixture::Mouse => fret_core::PointerType::Mouse,
            PointerTypeFixture::Pen => fret_core::PointerType::Pen,
            PointerTypeFixture::Touch => fret_core::PointerType::Touch,
            PointerTypeFixture::Unknown => fret_core::PointerType::Unknown,
        }
    }
}

#[derive(Default, Clone)]
struct EventCounts {
    downs: Arc<AtomicUsize>,
    moves: Arc<AtomicUsize>,
    wheels: Arc<AtomicUsize>,
    cancels: Arc<AtomicUsize>,
}

impl EventCounts {
    fn down(&self) {
        self.downs.fetch_add(1, Ordering::SeqCst);
    }

    fn move_(&self) {
        self.moves.fetch_add(1, Ordering::SeqCst);
    }

    fn wheel(&self) {
        self.wheels.fetch_add(1, Ordering::SeqCst);
    }

    fn cancel(&self) {
        self.cancels.fetch_add(1, Ordering::SeqCst);
    }

    fn load_downs(&self) -> f32 {
        self.downs.load(Ordering::SeqCst) as f32
    }

    fn load_moves(&self) -> f32 {
        self.moves.load(Ordering::SeqCst) as f32
    }

    fn load_wheels(&self) -> f32 {
        self.wheels.load(Ordering::SeqCst) as f32
    }

    fn load_cancels(&self) -> f32 {
        self.cancels.load(Ordering::SeqCst) as f32
    }
}

#[derive(Clone, Copy)]
enum HitRegion {
    Corner { width: f32, height: f32 },
    Full,
    Transparent,
}

struct RoutingProbeWidget {
    events: EventCounts,
    observers: EventCounts,
    observer_filter: ObserverFilter,
    hit_region: HitRegion,
    capture_on_down: bool,
    stop_down_propagation: bool,
}

#[derive(Clone, Copy)]
enum ObserverFilter {
    All,
    Down,
    Move,
    None,
}

impl<H: UiHost> Widget<H> for RoutingProbeWidget {
    fn hit_test(&self, bounds: Rect, position: Point) -> bool {
        match self.hit_region {
            HitRegion::Corner { width, height } => {
                position.x.0 >= bounds.origin.x.0
                    && position.y.0 >= bounds.origin.y.0
                    && position.x.0 <= bounds.origin.x.0 + width
                    && position.y.0 <= bounds.origin.y.0 + height
            }
            HitRegion::Full => bounds.contains(position),
            HitRegion::Transparent => false,
        }
    }

    fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
            return;
        }
        if observer_matches(self.observer_filter, event) {
            count_event(&self.observers, event);
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Bubble {
            return;
        }
        count_event(&self.events, event);
        if matches!(event, Event::Pointer(PointerEvent::Down { .. })) && self.capture_on_down {
            cx.capture_pointer(cx.node);
            if self.stop_down_propagation {
                cx.stop_propagation();
            }
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn mechanism_harness_pointer_occlusion_routing_matches_oracles() {
    let suite: MechanismSuite<PointerOcclusionRoutingScenario> =
        MechanismSuite::from_json_str(POINTER_OCCLUSION_ROUTING)
            .expect("pointer occlusion routing fixture suite");

    let mut observer: fn(
        &MechanismCase<PointerOcclusionRoutingScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<PointerOcclusionRoutingScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        PointerOcclusionRoutingScenario::PointerOcclusionRouting {
            underlay_capture_on_down,
            overlay_kind,
            overlay_blocks_underlay_input,
            overlay_hit_testable,
            pointer_occlusion,
            wants_pointer_down_outside,
            wants_pointer_move,
            hit_test_before,
            steps,
        } => observe_pointer_occlusion_routing_case(
            *underlay_capture_on_down,
            *overlay_kind,
            *overlay_blocks_underlay_input,
            *overlay_hit_testable,
            *pointer_occlusion,
            *wants_pointer_down_outside,
            *wants_pointer_move,
            *hit_test_before,
            steps,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn observe_pointer_occlusion_routing_case(
    underlay_capture_on_down: bool,
    overlay_kind: OverlayKindFixture,
    overlay_blocks_underlay_input: bool,
    overlay_hit_testable: bool,
    pointer_occlusion: PointerOcclusionFixture,
    wants_pointer_down_outside: bool,
    wants_pointer_move: bool,
    hit_test_before: Option<PointFixture>,
    steps: &[PointerEventFixture],
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let underlay_events = EventCounts::default();
    let underlay_observers = EventCounts::default();
    let underlay = ui.create_node(RoutingProbeWidget {
        events: underlay_events.clone(),
        observers: underlay_observers.clone(),
        hit_region: HitRegion::Full,
        observer_filter: ObserverFilter::None,
        capture_on_down: underlay_capture_on_down,
        stop_down_propagation: false,
    });
    ui.set_root(underlay);

    let overlay_events = EventCounts::default();
    let overlay_observers = EventCounts::default();
    let overlay = ui.create_node(overlay_widget(
        overlay_kind,
        overlay_events.clone(),
        overlay_observers.clone(),
    ));
    let overlay_layer = ui.push_overlay_root_with_options(
        overlay,
        crate::OverlayRootOptions {
            blocks_underlay_input: overlay_blocks_underlay_input,
            hit_testable: overlay_hit_testable,
        },
    );
    ui.set_layer_pointer_occlusion(overlay_layer, pointer_occlusion.into());
    ui.set_layer_wants_pointer_down_outside_events(overlay_layer, wants_pointer_down_outside);
    ui.set_layer_wants_pointer_move_events(overlay_layer, wants_pointer_move);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut observed = ObservedTree::new(bounds);
    if let Some(point) = hit_test_before {
        let hit = ui.debug_hit_test(Point::new(Px(point.x), Px(point.y)));
        observed.set_metric(
            "hit.before.hit_underlay",
            bool_metric(hit.hit == Some(underlay)),
        );
        observed.set_metric(
            "hit.before.hit_overlay",
            bool_metric(hit.hit == Some(overlay)),
        );
        observed.set_metric("hit.before.hit_miss", bool_metric(hit.hit.is_none()));
        observed.set_metric(
            "hit.before.barrier_present",
            bool_metric(hit.barrier_root.is_some()),
        );
    }

    for step in steps {
        let event = event_from_fixture(step);
        ui.dispatch_event(&mut app, &mut services, &event);
    }

    append_count_metrics(&mut observed, "underlay", &underlay_events);
    append_count_metrics(&mut observed, "underlay_observer", &underlay_observers);
    append_count_metrics(&mut observed, "overlay", &overlay_events);
    append_count_metrics(&mut observed, "observer", &overlay_observers);
    append_capture_metrics(&ui, &mut observed);
    append_arbitration_metrics(&ui, &mut observed);

    Ok(observed)
}

fn overlay_widget(
    kind: OverlayKindFixture,
    events: EventCounts,
    observers: EventCounts,
) -> RoutingProbeWidget {
    match kind {
        OverlayKindFixture::CornerCapture => RoutingProbeWidget {
            events,
            observers,
            observer_filter: ObserverFilter::Down,
            hit_region: HitRegion::Corner {
                width: 20.0,
                height: 20.0,
            },
            capture_on_down: true,
            stop_down_propagation: true,
        },
        OverlayKindFixture::ObserverDown
        | OverlayKindFixture::ObserverMove
        | OverlayKindFixture::Transparent => {
            let observer_filter = match kind {
                OverlayKindFixture::ObserverDown => ObserverFilter::Down,
                OverlayKindFixture::ObserverMove => ObserverFilter::Move,
                OverlayKindFixture::Transparent => ObserverFilter::None,
                OverlayKindFixture::CornerCapture => ObserverFilter::All,
            };
            RoutingProbeWidget {
                events,
                observers,
                observer_filter,
                hit_region: HitRegion::Transparent,
                capture_on_down: false,
                stop_down_propagation: false,
            }
        }
    }
}

fn event_from_fixture(fixture: &PointerEventFixture) -> Event {
    match fixture {
        PointerEventFixture::Down {
            x,
            y,
            pointer_id,
            pointer_type,
        } => Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(*x), Px(*y)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(*pointer_id),
            pointer_type: (*pointer_type).into(),
        }),
        PointerEventFixture::Move {
            x,
            y,
            pointer_id,
            pointer_type,
            buttons_left,
        } => Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(*x), Px(*y)),
            buttons: fret_core::MouseButtons {
                left: *buttons_left,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(*pointer_id),
            pointer_type: (*pointer_type).into(),
        }),
        PointerEventFixture::Wheel {
            x,
            y,
            pointer_id,
            pointer_type,
        } => Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(*x), Px(*y)),
            delta: Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(*pointer_id),
            pointer_type: (*pointer_type).into(),
        }),
    }
}

fn count_event(counts: &EventCounts, event: &Event) {
    match event {
        Event::Pointer(PointerEvent::Down { .. }) => counts.down(),
        Event::Pointer(PointerEvent::Move { .. }) => counts.move_(),
        Event::Pointer(PointerEvent::Wheel { .. }) => counts.wheel(),
        Event::PointerCancel(_) => counts.cancel(),
        _ => {}
    }
}

fn observer_matches(filter: ObserverFilter, event: &Event) -> bool {
    match filter {
        ObserverFilter::All => {
            matches!(
                event,
                Event::Pointer(PointerEvent::Down { .. } | PointerEvent::Move { .. })
            )
        }
        ObserverFilter::Down => matches!(event, Event::Pointer(PointerEvent::Down { .. })),
        ObserverFilter::Move => matches!(event, Event::Pointer(PointerEvent::Move { .. })),
        ObserverFilter::None => false,
    }
}

fn append_count_metrics(observed: &mut ObservedTree, prefix: &str, counts: &EventCounts) {
    observed.set_metric(format!("{prefix}.downs"), counts.load_downs());
    observed.set_metric(format!("{prefix}.moves"), counts.load_moves());
    observed.set_metric(format!("{prefix}.wheels"), counts.load_wheels());
    observed.set_metric(format!("{prefix}.cancels"), counts.load_cancels());
}

fn append_capture_metrics(ui: &UiTree<crate::test_host::TestHost>, observed: &mut ObservedTree) {
    let pointer0 = ui.captured_for(fret_core::PointerId(0));
    let pointer1 = ui.captured_for(fret_core::PointerId(1));
    observed.set_metric("capture.pointer0_active", bool_metric(pointer0.is_some()));
    observed.set_metric("capture.pointer1_active", bool_metric(pointer1.is_some()));
    observed.set_metric(
        "capture.any_active",
        bool_metric(ui.any_captured_node().is_some()),
    );
}

fn append_arbitration_metrics(
    ui: &UiTree<crate::test_host::TestHost>,
    observed: &mut ObservedTree,
) {
    let arbitration = ui.input_arbitration_snapshot();
    observed.set_metric(
        "arbitration.modal_barrier_present",
        bool_metric(arbitration.modal_barrier_root.is_some()),
    );
    observed.set_metric(
        "arbitration.pointer_occlusion_kind",
        match arbitration.pointer_occlusion {
            PointerOcclusion::None => 0.0,
            PointerOcclusion::BlockMouse => 1.0,
            PointerOcclusion::BlockMouseExceptScroll => 2.0,
        },
    );
    observed.set_metric(
        "arbitration.pointer_capture_active",
        bool_metric(arbitration.pointer_capture_active),
    );
    observed.set_metric(
        "arbitration.pointer_capture_multiple_layers",
        bool_metric(arbitration.pointer_capture_multiple_layers),
    );
}

fn bool_metric(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}

fn default_overlay_hit_testable() -> bool {
    true
}
