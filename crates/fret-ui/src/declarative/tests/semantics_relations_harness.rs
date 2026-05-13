use std::cell::Cell;
use std::sync::Arc;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

use super::*;

const SEMANTICS_RELATIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/semantics_relations_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SemanticsRelationScenario {
    TextInputControlsElement,
    TextInputActiveDescendantElement,
    AttachSemanticsRelationsAndFlags,
    SemanticsWrapperRelationsAndFlags,
    TextInputRegionValueAndEditingMetadata,
    PressableCollectionMetadata,
    SemanticsWrapperLiveAndStructuredMetadata,
}

#[test]
fn mechanism_harness_semantics_relations_match_oracles() {
    let suite: MechanismSuite<SemanticsRelationScenario> =
        MechanismSuite::from_json_str(SEMANTICS_RELATIONS)
            .expect("semantics relation fixture suite");

    let mut observer: fn(
        &MechanismCase<SemanticsRelationScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<SemanticsRelationScenario>,
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
    let text_model = app.models_mut().insert("hello".to_string());
    let focus_element: Cell<Option<crate::elements::GlobalElementId>> = Cell::new(None);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-semantics-relations",
        |cx| build_scenario(cx, &case.scenario, text_model, &focus_element),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    if let Some(element) = focus_element.get() {
        let node = crate::elements::node_for_element(&mut app, window, element)
            .ok_or_else(|| ScenarioObserveError::new("focus target element did not resolve"))?;
        ui.set_focus(Some(node));
    }

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.semantics_snapshot()
        .map(|snapshot| ObservedTree::from_semantics_snapshot(snapshot, bounds))
        .ok_or_else(|| ScenarioObserveError::new("missing semantics snapshot"))
}

fn build_scenario(
    cx: &mut ElementContext<'_, TestHost>,
    scenario: &SemanticsRelationScenario,
    text_model: fret_runtime::Model<String>,
    focus_element: &Cell<Option<crate::elements::GlobalElementId>>,
) -> Vec<AnyElement> {
    match scenario {
        SemanticsRelationScenario::TextInputControlsElement => {
            let listbox_id_out: Cell<Option<crate::elements::GlobalElementId>> = Cell::new(None);
            let listbox = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::ListBox,
                    test_id: Some(Arc::from("listbox")),
                    ..Default::default()
                },
                |_cx, id| {
                    listbox_id_out.set(Some(id));
                    Vec::new()
                },
            );

            let mut props = TextInputProps::new(text_model);
            props.layout.size.width = Length::Px(Px(120.0));
            props.layout.size.height = Length::Px(Px(28.0));
            props.test_id = Some(Arc::from("combo"));
            props.a11y_role = Some(fret_core::SemanticsRole::ComboBox);
            props.controls_element = listbox_id_out.get().map(|id| id.0);

            vec![cx.text_input(props), listbox]
        }
        SemanticsRelationScenario::TextInputActiveDescendantElement => {
            let option_id_out: Cell<Option<crate::elements::GlobalElementId>> = Cell::new(None);
            let option = cx.pressable_with_id(
                crate::element::PressableProps {
                    layout: {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = Length::Px(Px(120.0));
                        layout.size.height = Length::Px(Px(24.0));
                        layout
                    },
                    focusable: false,
                    a11y: crate::element::PressableA11y {
                        role: Some(fret_core::SemanticsRole::ListBoxOption),
                        test_id: Some(Arc::from("option")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |cx, _state, id| {
                    option_id_out.set(Some(id));
                    vec![cx.text("Option")]
                },
            );

            let mut props = TextInputProps::new(text_model);
            props.layout.size.width = Length::Px(Px(120.0));
            props.layout.size.height = Length::Px(Px(28.0));
            props.test_id = Some(Arc::from("combo"));
            props.a11y_role = Some(fret_core::SemanticsRole::ComboBox);
            props.active_descendant_element = option_id_out.get().map(|id| id.0);

            vec![cx.text_input(props), option]
        }
        SemanticsRelationScenario::AttachSemanticsRelationsAndFlags => {
            let label = cx.text("Decorated Label").test_id("decorated-label");
            let description = cx
                .text("Decorated Description")
                .test_id("decorated-description");
            let listbox = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::ListBox,
                    test_id: Some(Arc::from("decorated-listbox")),
                    ..Default::default()
                },
                |_cx, _id| Vec::new(),
            );
            let option = cx.text("Decorated Option").attach_semantics(
                crate::element::SemanticsDecoration::default()
                    .test_id("decorated-option")
                    .role(fret_core::SemanticsRole::ListBoxOption),
            );
            let target = cx.text("Decorated Target").attach_semantics(
                crate::element::SemanticsDecoration::default()
                    .test_id("decorated-target")
                    .role(fret_core::SemanticsRole::ComboBox)
                    .active_descendant_element(option.id.0)
                    .labelled_by_element(label.id.0)
                    .described_by_element(description.id.0)
                    .controls_element(listbox.id.0)
                    .disabled(true)
                    .hidden(true),
            );

            vec![label, description, listbox, option, target]
        }
        SemanticsRelationScenario::SemanticsWrapperRelationsAndFlags => {
            let label = cx.text("Wrapper Label").test_id("wrapper-label");
            let description = cx
                .text("Wrapper Description")
                .test_id("wrapper-description");
            let controlled = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Panel,
                    test_id: Some(Arc::from("wrapper-controlled")),
                    ..Default::default()
                },
                |_cx, _id| Vec::new(),
            );
            let target = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Checkbox,
                    test_id: Some(Arc::from("wrapper-target")),
                    labelled_by_element: Some(label.id.0),
                    described_by_element: Some(description.id.0),
                    controls_element: Some(controlled.id.0),
                    disabled: true,
                    hidden: true,
                    ..Default::default()
                },
                |_cx, _id| Vec::new(),
            );

            vec![label, description, controlled, target]
        }
        SemanticsRelationScenario::TextInputRegionValueAndEditingMetadata => {
            let mut props = crate::element::TextInputRegionProps::default();
            props.layout.size.width = Length::Px(Px(180.0));
            props.layout.size.height = Length::Px(Px(48.0));
            props.a11y_label = Some(Arc::from("Editor"));
            props.a11y_value = Some(Arc::from("hello"));
            props.a11y_text_selection = Some((2, 2));
            props.a11y_text_composition = Some((1, 3));

            let region = cx.text_input_region(props, |_cx| Vec::<AnyElement>::new());
            focus_element.set(Some(region.id));
            vec![region.attach_semantics(
                crate::element::SemanticsDecoration::default().test_id("editor-region"),
            )]
        }
        SemanticsRelationScenario::PressableCollectionMetadata => {
            let option = |cx: &mut ElementContext<'_, TestHost>,
                          test_id: &'static str,
                          label: &'static str,
                          pos_in_set: u32,
                          selected: bool,
                          checked: Option<bool>| {
                cx.pressable(
                    crate::element::PressableProps {
                        layout: {
                            let mut layout = crate::element::LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(24.0));
                            layout
                        },
                        a11y: crate::element::PressableA11y {
                            role: Some(fret_core::SemanticsRole::ListBoxOption),
                            label: Some(Arc::from(label)),
                            test_id: Some(Arc::from(test_id)),
                            selected,
                            checked,
                            pos_in_set: Some(pos_in_set),
                            set_size: Some(5),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _state| vec![cx.text(label)],
                )
            };

            vec![cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::ListBox,
                    test_id: Some(Arc::from("collection-listbox")),
                    ..Default::default()
                },
                |cx, _id| {
                    vec![
                        option(cx, "option-two", "Two", 2, true, Some(true)),
                        option(cx, "option-three", "Three", 3, false, None),
                    ]
                },
            )]
        }
        SemanticsRelationScenario::SemanticsWrapperLiveAndStructuredMetadata => {
            let live = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Status,
                    label: Some(Arc::from("Sync status")),
                    test_id: Some(Arc::from("live-status")),
                    value: Some(Arc::from("Saved")),
                    live: Some(fret_core::SemanticsLive::Polite),
                    live_atomic: true,
                    ..Default::default()
                },
                |_cx, _id| Vec::new(),
            );

            let range = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Slider,
                    label: Some(Arc::from("Volume")),
                    test_id: Some(Arc::from("volume-slider")),
                    value: Some(Arc::from("50")),
                    numeric_value: Some(50.0),
                    min_numeric_value: Some(0.0),
                    max_numeric_value: Some(100.0),
                    numeric_value_step: Some(1.0),
                    numeric_value_jump: Some(10.0),
                    value_editable: Some(true),
                    ..Default::default()
                },
                |_cx, _id| Vec::new(),
            );

            let viewport = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Viewport,
                    label: Some(Arc::from("Results")),
                    test_id: Some(Arc::from("results-viewport")),
                    scroll_y: Some(40.0),
                    scroll_y_min: Some(0.0),
                    scroll_y_max: Some(120.0),
                    ..Default::default()
                },
                |_cx, _id| Vec::new(),
            );

            vec![live, range, viewport]
        }
    }
}
