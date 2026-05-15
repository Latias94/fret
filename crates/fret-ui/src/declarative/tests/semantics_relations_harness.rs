use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedSemanticsRelation, ObservedTree,
    ScenarioObserveError,
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
    SemanticsWrapperHierarchyMetadata,
    HiddenSubtreePolicy,
    RelationTargetsDetachReattach,
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
    if matches!(
        &case.scenario,
        SemanticsRelationScenario::RelationTargetsDetachReattach
    ) {
        return observe_relation_targets_detach_reattach();
    }

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
                          checked: Option<bool>,
                          enabled: bool| {
                cx.pressable(
                    crate::element::PressableProps {
                        layout: {
                            let mut layout = crate::element::LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(24.0));
                            layout
                        },
                        enabled,
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
                        option(cx, "option-two", "Two", 2, true, Some(true), true),
                        option(cx, "option-three", "Three", 3, false, None, true),
                        option(cx, "option-disabled", "Disabled", 4, false, None, false),
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
        SemanticsRelationScenario::SemanticsWrapperHierarchyMetadata => {
            let root = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::TreeItem,
                    label: Some(Arc::from("Root")),
                    test_id: Some(Arc::from("tree-root")),
                    level: Some(1),
                    expanded: Some(true),
                    ..Default::default()
                },
                |cx, _id| {
                    vec![cx.semantics_with_id(
                        crate::element::SemanticsProps {
                            role: fret_core::SemanticsRole::TreeItem,
                            label: Some(Arc::from("Child")),
                            test_id: Some(Arc::from("tree-child")),
                            level: Some(2),
                            expanded: Some(false),
                            ..Default::default()
                        },
                        |_cx, _id| Vec::new(),
                    )]
                },
            );

            vec![root]
        }
        SemanticsRelationScenario::HiddenSubtreePolicy => {
            let visible = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Button,
                    label: Some(Arc::from("Visible")),
                    test_id: Some(Arc::from("visible-button")),
                    ..Default::default()
                },
                |_cx, _id| Vec::new(),
            );

            let hidden = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Group,
                    label: Some(Arc::from("Hidden Group")),
                    test_id: Some(Arc::from("hidden-group")),
                    hidden: true,
                    ..Default::default()
                },
                |cx, _id| {
                    vec![cx.semantics_with_id(
                        crate::element::SemanticsProps {
                            role: fret_core::SemanticsRole::Button,
                            label: Some(Arc::from("Hidden Child")),
                            test_id: Some(Arc::from("hidden-child-button")),
                            ..Default::default()
                        },
                        |_cx, _id| Vec::new(),
                    )]
                },
            );

            vec![visible, hidden]
        }
        SemanticsRelationScenario::RelationTargetsDetachReattach => {
            unreachable!("relation target mutation uses a multi-frame observer")
        }
    }
}

fn observe_relation_targets_detach_reattach() -> Result<ObservedTree, ScenarioObserveError> {
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
    let label_element: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let description_element: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let controlled_element: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    render_relation_targets_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        label_element.clone(),
        description_element.clone(),
        controlled_element.clone(),
    );
    let initial_snapshot = capture_semantics_snapshot(&mut ui, &mut app, &mut services, bounds)?;

    render_relation_targets_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        label_element.clone(),
        description_element.clone(),
        controlled_element.clone(),
    );
    let detached_snapshot = capture_semantics_snapshot(&mut ui, &mut app, &mut services, bounds)?;

    render_relation_targets_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        label_element,
        description_element,
        controlled_element,
    );
    let final_snapshot = capture_semantics_snapshot(&mut ui, &mut app, &mut services, bounds)?;

    let mut observed = ObservedTree::from_semantics_snapshot(&final_snapshot, bounds);
    observed.set_metric(
        "relation_targets.initial.labelled_by",
        bool_metric(snapshot_relation_includes(
            &initial_snapshot,
            "relation-source",
            ObservedSemanticsRelation::LabelledBy,
            "relation-label",
        )),
    );
    observed.set_metric(
        "relation_targets.initial.described_by",
        bool_metric(snapshot_relation_includes(
            &initial_snapshot,
            "relation-source",
            ObservedSemanticsRelation::DescribedBy,
            "relation-description",
        )),
    );
    observed.set_metric(
        "relation_targets.initial.controls",
        bool_metric(snapshot_relation_includes(
            &initial_snapshot,
            "relation-source",
            ObservedSemanticsRelation::Controls,
            "relation-controlled",
        )),
    );
    observed.set_metric(
        "relation_targets.detached.label_absent",
        bool_metric(snapshot_node_by_test_id(&detached_snapshot, "relation-label").is_none()),
    );
    observed.set_metric(
        "relation_targets.detached.description_absent",
        bool_metric(snapshot_node_by_test_id(&detached_snapshot, "relation-description").is_none()),
    );
    observed.set_metric(
        "relation_targets.detached.controlled_absent",
        bool_metric(snapshot_node_by_test_id(&detached_snapshot, "relation-controlled").is_none()),
    );
    observed.set_metric(
        "relation_targets.detached.labelled_by_empty",
        bool_metric(snapshot_relation_empty(
            &detached_snapshot,
            "relation-source",
            ObservedSemanticsRelation::LabelledBy,
        )),
    );
    observed.set_metric(
        "relation_targets.detached.described_by_empty",
        bool_metric(snapshot_relation_empty(
            &detached_snapshot,
            "relation-source",
            ObservedSemanticsRelation::DescribedBy,
        )),
    );
    observed.set_metric(
        "relation_targets.detached.controls_empty",
        bool_metric(snapshot_relation_empty(
            &detached_snapshot,
            "relation-source",
            ObservedSemanticsRelation::Controls,
        )),
    );
    observed.set_metric(
        "relation_targets.final.labelled_by",
        bool_metric(snapshot_relation_includes(
            &final_snapshot,
            "relation-source",
            ObservedSemanticsRelation::LabelledBy,
            "relation-label",
        )),
    );
    observed.set_metric(
        "relation_targets.final.described_by",
        bool_metric(snapshot_relation_includes(
            &final_snapshot,
            "relation-source",
            ObservedSemanticsRelation::DescribedBy,
            "relation-description",
        )),
    );
    observed.set_metric(
        "relation_targets.final.controls",
        bool_metric(snapshot_relation_includes(
            &final_snapshot,
            "relation-source",
            ObservedSemanticsRelation::Controls,
            "relation-controlled",
        )),
    );
    Ok(observed)
}

#[allow(clippy::too_many_arguments)]
fn render_relation_targets_frame(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    targets_present: bool,
    label_element: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
    description_element: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
    controlled_element: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
) {
    render_root_for_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-harness-semantics-relation-targets",
        |cx| {
            let mut children = Vec::new();
            if targets_present {
                let label = cx.text("Relation Label").test_id("relation-label");
                label_element.set(Some(label.id));
                children.push(label);

                let description = cx
                    .text("Relation Description")
                    .test_id("relation-description");
                description_element.set(Some(description.id));
                children.push(description);

                let controlled = cx.semantics_with_id(
                    crate::element::SemanticsProps {
                        role: fret_core::SemanticsRole::Panel,
                        label: Some(Arc::from("Relation Controlled")),
                        test_id: Some(Arc::from("relation-controlled")),
                        ..Default::default()
                    },
                    |_cx, _id| Vec::new(),
                );
                controlled_element.set(Some(controlled.id));
                children.push(controlled);
            }

            let source = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::ComboBox,
                    label: Some(Arc::from("Relation Source")),
                    test_id: Some(Arc::from("relation-source")),
                    labelled_by_element: label_element.get().map(|id| id.0),
                    described_by_element: description_element.get().map(|id| id.0),
                    controls_element: controlled_element.get().map(|id| id.0),
                    ..Default::default()
                },
                |_cx, _id| Vec::new(),
            );
            children.push(source);
            children
        },
    );
    ui.layout_all(app, services, bounds, 1.0);
    app.advance_frame();
}

fn capture_semantics_snapshot(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
) -> Result<fret_core::SemanticsSnapshot, ScenarioObserveError> {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    ui.semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing semantics snapshot"))
}

fn snapshot_node_by_test_id<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    test_id: &str,
) -> Option<&'a fret_core::SemanticsNode> {
    snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
}

fn snapshot_relation_includes(
    snapshot: &fret_core::SemanticsSnapshot,
    source_test_id: &str,
    relation: ObservedSemanticsRelation,
    target_test_id: &str,
) -> bool {
    let Some(source) = snapshot_node_by_test_id(snapshot, source_test_id) else {
        return false;
    };
    let Some(target) = snapshot_node_by_test_id(snapshot, target_test_id) else {
        return false;
    };
    match relation {
        ObservedSemanticsRelation::ActiveDescendant => source.active_descendant == Some(target.id),
        ObservedSemanticsRelation::LabelledBy => source.labelled_by.contains(&target.id),
        ObservedSemanticsRelation::DescribedBy => source.described_by.contains(&target.id),
        ObservedSemanticsRelation::Controls => source.controls.contains(&target.id),
    }
}

fn snapshot_relation_empty(
    snapshot: &fret_core::SemanticsSnapshot,
    source_test_id: &str,
    relation: ObservedSemanticsRelation,
) -> bool {
    snapshot_node_by_test_id(snapshot, source_test_id).is_some_and(|source| match relation {
        ObservedSemanticsRelation::ActiveDescendant => source.active_descendant.is_none(),
        ObservedSemanticsRelation::LabelledBy => source.labelled_by.is_empty(),
        ObservedSemanticsRelation::DescribedBy => source.described_by.is_empty(),
        ObservedSemanticsRelation::Controls => source.controls.is_empty(),
    })
}

fn bool_metric(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}
