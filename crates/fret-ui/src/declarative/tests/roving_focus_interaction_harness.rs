use std::sync::Arc;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

use super::*;

const ROVING_FOCUS_INTERACTION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/roving_focus_interaction_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RovingFocusScenario {
    RovingFlex(RovingFlexScenario),
}

#[derive(Debug, Clone, Deserialize)]
struct RovingFlexScenario {
    wrap: bool,
    disabled: Vec<bool>,
    initial: RovingItem,
    key: RovingKey,
    #[serde(default)]
    wrap_items_in_pointer_regions: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RovingItem {
    A,
    B,
    C,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RovingKey {
    ArrowDown,
    ArrowUp,
}

#[derive(Default)]
struct RovingElementIds {
    a: Option<crate::elements::GlobalElementId>,
    b: Option<crate::elements::GlobalElementId>,
    c: Option<crate::elements::GlobalElementId>,
}

#[test]
fn mechanism_harness_roving_focus_interaction_matches_oracles() {
    let suite: MechanismSuite<RovingFocusScenario> =
        MechanismSuite::from_json_str(ROVING_FOCUS_INTERACTION)
            .expect("roving focus fixture suite");

    let mut observer: fn(
        &MechanismCase<RovingFocusScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<RovingFocusScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        RovingFocusScenario::RovingFlex(scenario) => observe_roving_flex(scenario),
    }
}

fn observe_roving_flex(
    scenario: &RovingFlexScenario,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(260.0), Px(160.0)),
    );
    let mut services = FakeTextService::default();
    let selection_model = app
        .models_mut()
        .insert(Option::<Arc<str>>::Some(scenario.initial.label()));
    let mut ids = RovingElementIds::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-roving-focus",
        |cx| build_roving_flex(cx, scenario, selection_model.clone(), &mut ids),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let initial = ids
        .get(scenario.initial)
        .ok_or_else(|| ScenarioObserveError::new("missing initial roving element"))?;
    let initial_node = crate::elements::node_for_element(&mut app, window, initial)
        .ok_or_else(|| ScenarioObserveError::new("missing initial roving node"))?;
    ui.set_focus(Some(initial_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: scenario.key.key_code(),
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing semantics snapshot"))?;
    let mut observed = ObservedTree::from_semantics_snapshot(&snapshot, bounds);
    observed.set_metric(
        "roving.selection.index",
        selection_index(app.models().get_cloned(&selection_model).flatten()),
    );
    Ok(observed)
}

fn build_roving_flex(
    cx: &mut ElementContext<'_, TestHost>,
    scenario: &RovingFlexScenario,
    selection_model: fret_runtime::Model<Option<Arc<str>>>,
    ids: &mut RovingElementIds,
) -> Vec<AnyElement> {
    let props = crate::element::RovingFlexProps {
        flex: crate::element::FlexProps {
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        },
        roving: crate::element::RovingFocusProps {
            enabled: true,
            wrap: scenario.wrap,
            disabled: Arc::from(scenario.disabled.clone().into_boxed_slice()),
        },
    };

    vec![cx.roving_flex(props, |cx| {
        install_roving_callbacks(cx, scenario.wrap, selection_model);
        vec![
            build_roving_item(
                cx,
                "a",
                "roving-a",
                &mut ids.a,
                scenario.wrap_items_in_pointer_regions,
            ),
            build_roving_item(
                cx,
                "b",
                "roving-b",
                &mut ids.b,
                scenario.wrap_items_in_pointer_regions,
            ),
            build_roving_item(
                cx,
                "c",
                "roving-c",
                &mut ids.c,
                scenario.wrap_items_in_pointer_regions,
            ),
        ]
    })]
}

fn install_roving_callbacks(
    cx: &mut ElementContext<'_, TestHost>,
    wrap: bool,
    selection_model: fret_runtime::Model<Option<Arc<str>>>,
) {
    let values: Arc<[Arc<str>]> = Arc::from([Arc::from("a"), Arc::from("b"), Arc::from("c")]);
    cx.roving_on_navigate(Arc::new(move |_host, _cx, it| {
        use crate::action::RovingNavigateResult;
        use fret_core::KeyCode;

        let Some(current) = it.current else {
            return RovingNavigateResult::NotHandled;
        };

        let forward = match it.key {
            KeyCode::ArrowDown => true,
            KeyCode::ArrowUp => false,
            _ => return RovingNavigateResult::NotHandled,
        };

        let is_disabled = |idx: usize| -> bool { it.disabled.get(idx).copied().unwrap_or(false) };
        let target = if wrap {
            (1..=it.len).find_map(|step| {
                let idx = if forward {
                    (current + step) % it.len
                } else {
                    (current + it.len - (step % it.len)) % it.len
                };
                (!is_disabled(idx)).then_some(idx)
            })
        } else if forward {
            ((current + 1)..it.len).find(|&idx| !is_disabled(idx))
        } else if current > 0 {
            (0..current).rev().find(|&idx| !is_disabled(idx))
        } else {
            None
        };

        RovingNavigateResult::Handled { target }
    }));

    cx.roving_on_active_change(Arc::new(move |host, _cx, idx| {
        let Some(value) = values.get(idx).cloned() else {
            return;
        };
        let _ = host
            .models_mut()
            .update(&selection_model, |selected| *selected = Some(value));
    }));
}

fn build_roving_item(
    cx: &mut ElementContext<'_, TestHost>,
    label: &'static str,
    test_id: &'static str,
    out: &mut Option<crate::elements::GlobalElementId>,
    wrap_in_pointer_region: bool,
) -> AnyElement {
    let mut make_pressable = |cx: &mut ElementContext<'_, TestHost>| {
        cx.pressable_with_id(
            crate::element::PressableProps {
                layout: {
                    let mut layout = crate::element::LayoutStyle::default();
                    layout.size.width = Length::Px(Px(80.0));
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
                vec![cx.text(label)]
            },
        )
    };

    if wrap_in_pointer_region {
        cx.pointer_region(crate::element::PointerRegionProps::default(), |cx| {
            vec![make_pressable(cx)]
        })
    } else {
        make_pressable(cx)
    }
}

fn selection_index(selected: Option<Arc<str>>) -> f32 {
    match selected.as_deref() {
        Some("a") => 0.0,
        Some("b") => 1.0,
        Some("c") => 2.0,
        _ => -1.0,
    }
}

impl RovingElementIds {
    fn get(&self, item: RovingItem) -> Option<crate::elements::GlobalElementId> {
        match item {
            RovingItem::A => self.a,
            RovingItem::B => self.b,
            RovingItem::C => self.c,
        }
    }
}

impl RovingItem {
    fn label(self) -> Arc<str> {
        match self {
            Self::A => Arc::from("a"),
            Self::B => Arc::from("b"),
            Self::C => Arc::from("c"),
        }
    }
}

impl RovingKey {
    fn key_code(self) -> fret_core::KeyCode {
        match self {
            Self::ArrowDown => fret_core::KeyCode::ArrowDown,
            Self::ArrowUp => fret_core::KeyCode::ArrowUp,
        }
    }
}
