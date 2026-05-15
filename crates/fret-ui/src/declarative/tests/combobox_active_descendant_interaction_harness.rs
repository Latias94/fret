use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

use super::*;

const COMBOBOX_ACTIVE_DESCENDANT_INTERACTION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/combobox_active_descendant_interaction_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ComboboxActiveDescendantScenario {
    ComboboxTextFilterActiveDescendant(ComboboxTextFilterActiveDescendantScenario),
    RetainedVirtualListActiveDescendantActionState(
        RetainedVirtualListActiveDescendantActionStateScenario,
    ),
}

#[derive(Debug, Clone, Deserialize)]
struct ComboboxTextFilterActiveDescendantScenario {
    query: String,
    expected_item: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RetainedVirtualListActiveDescendantActionStateScenario {
    len: usize,
    viewport_width: f32,
    viewport_height: f32,
    row_height: f32,
    overscan: usize,
    keep_alive: usize,
    warmup_frames: usize,
    frames_per_step: usize,
    active_index: usize,
    away_offset_y: f32,
}

#[derive(Clone, Copy)]
struct ComboboxItemSpec {
    test_id: &'static str,
    label: &'static str,
}

const COMBOBOX_ITEMS: [ComboboxItemSpec; 3] = [
    ComboboxItemSpec {
        test_id: "combobox-active-item-alpha",
        label: "Alpha",
    },
    ComboboxItemSpec {
        test_id: "combobox-active-item-nuxt",
        label: "Nuxt",
    },
    ComboboxItemSpec {
        test_id: "combobox-active-item-zeta",
        label: "Zeta",
    },
];

#[test]
fn mechanism_harness_combobox_active_descendant_interaction_matches_oracles() {
    let suite: MechanismSuite<ComboboxActiveDescendantScenario> =
        MechanismSuite::from_json_str(COMBOBOX_ACTIVE_DESCENDANT_INTERACTION)
            .expect("combobox active-descendant fixture suite");

    let mut observer: fn(
        &MechanismCase<ComboboxActiveDescendantScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<ComboboxActiveDescendantScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        ComboboxActiveDescendantScenario::ComboboxTextFilterActiveDescendant(scenario) => {
            observe_text_filter_active_descendant(scenario)
        }
        ComboboxActiveDescendantScenario::RetainedVirtualListActiveDescendantActionState(
            scenario,
        ) => observe_retained_virtual_list_active_descendant_action_state(scenario),
    }
}

fn observe_text_filter_active_descendant(
    scenario: &ComboboxTextFilterActiveDescendantScenario,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );
    let mut services = FakeTextService::default();
    let query_model = app.models_mut().insert(String::new());

    let initial_query = app.models().get_cloned(&query_model).unwrap_or_default();
    let query_model_for_initial = query_model.clone();
    let root = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-combobox-active-descendant",
        move |cx| build_combobox_scene(cx, query_model_for_initial.clone(), initial_query.clone()),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui
        .children(root)
        .first()
        .copied()
        .ok_or_else(|| ScenarioObserveError::new("missing combobox input node"))?;
    ui.set_focus(Some(input_node));

    for ch in scenario.query.chars() {
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::TextInput(ch.to_string()),
        );
    }

    let query_after = app.models().get_cloned(&query_model).unwrap_or_default();
    let query_model_for_final = query_model.clone();
    let _ = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-combobox-active-descendant",
        move |cx| build_combobox_scene(cx, query_model_for_final.clone(), query_after.clone()),
    );
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing semantics snapshot"))?;
    let mut observed = ObservedTree::from_semantics_snapshot(&snapshot, bounds);
    let query = app.models().get_cloned(&query_model).unwrap_or_default();
    let matches = matching_items(&query);
    let active_index = active_index_for_query(&query);
    let active_item = active_index
        .and_then(|index| COMBOBOX_ITEMS.get(index))
        .ok_or_else(|| {
            ScenarioObserveError::new("combobox query did not resolve an active item")
        })?;
    if active_item.test_id != scenario.expected_item {
        return Err(ScenarioObserveError::new(format!(
            "combobox active item mismatch: expected {} but resolved {}",
            scenario.expected_item, active_item.test_id
        )));
    }

    observed.set_metric("combobox.query.len", query.chars().count() as f32);
    observed.set_metric("combobox.visible.count", matches.len() as f32);
    observed.set_metric(
        "combobox.active.index",
        active_index.map(|index| index as f32).unwrap_or(-1.0),
    );
    Ok(observed)
}

fn build_combobox_scene(
    cx: &mut ElementContext<'_, TestHost>,
    query_model: fret_runtime::Model<String>,
    query: String,
) -> Vec<AnyElement> {
    let active_index = active_index_for_query(&query);

    let listbox_id_out: Cell<Option<crate::elements::GlobalElementId>> = Cell::new(None);
    let option_ids: [Cell<Option<crate::elements::GlobalElementId>>; COMBOBOX_ITEMS.len()] =
        std::array::from_fn(|_| Cell::new(None));

    let listbox = cx.semantics_with_id(
        crate::element::SemanticsProps {
            role: fret_core::SemanticsRole::ListBox,
            test_id: Some(Arc::from("combobox-active-listbox")),
            ..Default::default()
        },
        |cx, id| {
            listbox_id_out.set(Some(id));

            COMBOBOX_ITEMS
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let option_test_id = Arc::from(item.test_id);
                    let option_id_cell = &option_ids[index];
                    cx.semantics_with_id(
                        crate::element::SemanticsProps {
                            role: fret_core::SemanticsRole::ListBoxOption,
                            test_id: Some(option_test_id),
                            ..Default::default()
                        },
                        move |cx, option_id| {
                            option_id_cell.set(Some(option_id));
                            vec![cx.text(item.label)]
                        },
                    )
                })
                .collect::<Vec<_>>()
        },
    );

    let active_descendant_element = active_index
        .and_then(|index| option_ids[index].get())
        .map(|id| id.0);

    let mut input_props = TextInputProps::new(query_model);
    input_props.layout.size.width = Length::Px(Px(180.0));
    input_props.layout.size.height = Length::Px(Px(28.0));
    input_props.test_id = Some(Arc::from("combobox-active-input"));
    input_props.a11y_role = Some(fret_core::SemanticsRole::ComboBox);
    input_props.controls_element = listbox_id_out.get().map(|id| id.0);
    input_props.active_descendant_element = active_descendant_element;

    vec![cx.text_input(input_props), listbox]
}

fn matching_items(query: &str) -> Vec<&'static ComboboxItemSpec> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return COMBOBOX_ITEMS.iter().collect();
    }

    COMBOBOX_ITEMS
        .iter()
        .filter(|item| item.label.to_ascii_lowercase().starts_with(&query))
        .collect()
}

fn active_index_for_query(query: &str) -> Option<usize> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return COMBOBOX_ITEMS
            .iter()
            .enumerate()
            .find_map(|(index, item)| (!item.label.is_empty()).then_some(index));
    }

    COMBOBOX_ITEMS
        .iter()
        .enumerate()
        .find(|(_, item)| item.label.to_ascii_lowercase().starts_with(&query))
        .map(|(index, _)| index)
}

fn observe_retained_virtual_list_active_descendant_action_state(
    scenario: &RetainedVirtualListActiveDescendantActionStateScenario,
) -> Result<ObservedTree, ScenarioObserveError> {
    if scenario.active_index >= scenario.len {
        return Err(ScenarioObserveError::new(format!(
            "active_index {} is outside len {}",
            scenario.active_index, scenario.len
        )));
    }

    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(
            Px(scenario.viewport_width),
            Px(scenario.viewport_height + 48.0),
        ),
    );
    let mut services = FakeTextService::default();
    let input_model = app.models_mut().insert(String::new());
    let active_element: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let input_element: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    for _ in 0..scenario.warmup_frames {
        render_retained_active_descendant_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            scenario,
            &scroll_handle,
            input_model.clone(),
            active_element.clone(),
            input_element.clone(),
            false,
        );
    }

    let input_node = input_element
        .get()
        .and_then(|element| crate::elements::node_for_element(&mut app, window, element))
        .ok_or_else(|| ScenarioObserveError::new("missing retained active-descendant input"))?;
    ui.set_focus(Some(input_node));

    let initial_snapshot = capture_semantics_snapshot(&mut ui, &mut app, &mut services, bounds)?;
    let initial_item_id = retained_active_row_test_id(scenario.active_index);
    let initial_active_matches =
        snapshot_active_item_matches(&initial_snapshot, "retained-active-input", &initial_item_id);

    scroll_handle.set_offset(Point::new(Px(0.0), Px(scenario.away_offset_y)));
    for _ in 0..scenario.frames_per_step {
        render_retained_active_descendant_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            scenario,
            &scroll_handle,
            input_model.clone(),
            active_element.clone(),
            input_element.clone(),
            false,
        );
    }

    let away_snapshot = capture_semantics_snapshot(&mut ui, &mut app, &mut services, bounds)?;
    let away_active_none = snapshot_active_item_none(&away_snapshot, "retained-active-input");
    let away_row_exists = snapshot_node_by_test_id(&away_snapshot, &initial_item_id).is_some();

    scroll_handle.set_offset(Point::new(Px(0.0), Px(0.0)));
    for _ in 0..scenario.frames_per_step {
        render_retained_active_descendant_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            scenario,
            &scroll_handle,
            input_model.clone(),
            active_element.clone(),
            input_element.clone(),
            true,
        );
    }

    let final_snapshot = capture_semantics_snapshot(&mut ui, &mut app, &mut services, bounds)?;
    let final_active_matches =
        snapshot_active_item_matches(&final_snapshot, "retained-active-input", &initial_item_id);
    let final_row = snapshot_node_by_test_id(&final_snapshot, &initial_item_id)
        .ok_or_else(|| ScenarioObserveError::new("missing reattached active row"))?;

    let mut observed = ObservedTree::from_semantics_snapshot(&final_snapshot, bounds);
    observed.set_metric(
        "retained_active.initial.active_item_matches",
        bool_metric(initial_active_matches),
    );
    observed.set_metric(
        "retained_active.away.active_item_none",
        bool_metric(away_active_none),
    );
    observed.set_metric(
        "retained_active.away.row_exists",
        bool_metric(away_row_exists),
    );
    observed.set_metric(
        "retained_active.final.active_item_matches",
        bool_metric(final_active_matches),
    );
    observed.set_metric(
        "retained_active.final.disabled",
        bool_metric(final_row.flags.disabled),
    );
    observed.set_metric(
        "retained_active.final.invoke",
        bool_metric(final_row.actions.invoke),
    );
    Ok(observed)
}

#[allow(clippy::too_many_arguments)]
fn render_retained_active_descendant_frame(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    scenario: &RetainedVirtualListActiveDescendantActionStateScenario,
    scroll_handle: &crate::scroll::VirtualListScrollHandle,
    input_model: fret_runtime::Model<String>,
    active_element: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
    input_element: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
    active_disabled: bool,
) {
    render_root_for_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-harness-retained-active-descendant-action-state",
        |cx| {
            let active_index = scenario.active_index;
            let len = scenario.len;
            let row_height = scenario.row_height;
            let active_element_for_row = active_element.clone();
            let row: crate::windowed_surface_host::RetainedVirtualListRowFn<TestHost> =
                Arc::new(move |cx, index| {
                    let is_active = index == active_index;
                    let disabled = active_disabled && is_active;
                    let mut layout = crate::element::LayoutStyle::default();
                    layout.size.width = crate::element::Length::Fill;
                    layout.size.height = crate::element::Length::Px(Px(row_height));

                    let row_id = retained_active_row_test_id(index);
                    let label = format!("Row {index}");
                    let row = cx.pressable_with_id(
                        crate::element::PressableProps {
                            layout,
                            enabled: !disabled,
                            focusable: false,
                            a11y: crate::element::PressableA11y {
                                role: Some(fret_core::SemanticsRole::ListBoxOption),
                                label: Some(Arc::<str>::from(label.clone())),
                                test_id: Some(Arc::<str>::from(row_id)),
                                selected: is_active,
                                pos_in_set: Some(index.saturating_add(1) as u32),
                                set_size: Some(len as u32),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx, _st, _id| vec![cx.text(label.clone())],
                    );
                    if is_active {
                        active_element_for_row.set(Some(row.id));
                    }
                    row
                });

            let mut list_layout = crate::element::LayoutStyle::default();
            list_layout.size.width = crate::element::Length::Fill;
            list_layout.size.height = crate::element::Length::Px(Px(scenario.viewport_height));
            list_layout.overflow = crate::element::Overflow::Clip;

            let mut options = crate::element::VirtualListOptions::known(
                Px(scenario.row_height),
                scenario.overscan,
                {
                    let row_height = scenario.row_height;
                    move |_index| Px(row_height)
                },
            )
            .keep_alive(scenario.keep_alive);
            options.items_revision = u64::from(active_disabled);

            let key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn =
                Arc::new(|index| index as crate::ItemKey);

            let listbox_id_out: Cell<Option<crate::elements::GlobalElementId>> = Cell::new(None);
            let listbox = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::ListBox,
                    test_id: Some(Arc::from("retained-active-listbox")),
                    ..Default::default()
                },
                |cx, id| {
                    listbox_id_out.set(Some(id));
                    vec![cx.virtual_list_keyed_retained_with_layout(
                        list_layout,
                        scenario.len,
                        options,
                        scroll_handle,
                        key_at,
                        row,
                    )]
                },
            );

            let mut input_props = TextInputProps::new(input_model);
            input_props.layout.size.width = Length::Fill;
            input_props.layout.size.height = Length::Px(Px(28.0));
            input_props.test_id = Some(Arc::from("retained-active-input"));
            input_props.a11y_role = Some(fret_core::SemanticsRole::ComboBox);
            input_props.a11y_label = Some(Arc::from("Retained active descendant search"));
            input_props.controls_element = listbox_id_out.get().map(|id| id.0);
            input_props.active_descendant_element = active_element.get().map(|id| id.0);
            let input = cx.text_input(input_props);
            input_element.set(Some(input.id));

            vec![input, listbox]
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

fn retained_active_row_test_id(index: usize) -> String {
    format!("retained-active-row-{index}")
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

fn snapshot_active_item_matches(
    snapshot: &fret_core::SemanticsSnapshot,
    container_test_id: &str,
    item_test_id: &str,
) -> bool {
    let Some(container) = snapshot_node_by_test_id(snapshot, container_test_id) else {
        return false;
    };
    let Some(item) = snapshot_node_by_test_id(snapshot, item_test_id) else {
        return false;
    };
    container.active_descendant == Some(item.id)
}

fn snapshot_active_item_none(
    snapshot: &fret_core::SemanticsSnapshot,
    container_test_id: &str,
) -> bool {
    snapshot_node_by_test_id(snapshot, container_test_id)
        .is_some_and(|container| container.active_descendant.is_none())
}

fn bool_metric(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}
