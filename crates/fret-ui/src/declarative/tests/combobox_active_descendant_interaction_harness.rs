use std::cell::Cell;
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
}

#[derive(Debug, Clone, Deserialize)]
struct ComboboxTextFilterActiveDescendantScenario {
    query: String,
    expected_item: String,
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
