use super::*;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use fret_ui_shadcn::facade as shadcn;
use serde::Deserialize;
use slotmap::Key as _;

const RECIPE_CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/mechanism_layout_recipe_cases_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RecipeScenario {
    ButtonGroupTextAddonsCenterWithInputControl,
}

#[test]
fn mechanism_harness_recipe_layout_cases_match_oracles() {
    let suite: MechanismSuite<RecipeScenario> =
        MechanismSuite::from_json_str(RECIPE_CASES).expect("recipe mechanism fixture suite");

    let mut observer: fn(
        &MechanismCase<RecipeScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<RecipeScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(360.0), Px(80.0)),
    );
    let (ui, snapshot, _root) = run_fret_root_with_ui(bounds, |cx| match case.scenario {
        RecipeScenario::ButtonGroupTextAddonsCenterWithInputControl => {
            let model: Model<String> = cx.app.models_mut().insert(String::new());
            let control_id = "mechanism-button-group-url";

            vec![
                shadcn::ButtonGroup::new([
                    shadcn::ButtonGroupText::new("https://")
                        .test_id("mechanism-button-group-text-prefix")
                        .into(),
                    shadcn::Input::new(model)
                        .control_id(control_id)
                        .a11y_label("URL")
                        .placeholder("my-app")
                        .test_id("mechanism-button-group-text-control")
                        .refine_layout(
                            LayoutRefinement::default()
                                .w_px(MetricRef::Px(Px(220.0)))
                                .min_w_0(),
                        )
                        .into(),
                    shadcn::ButtonGroupText::new(".com")
                        .test_id("mechanism-button-group-text-suffix")
                        .into(),
                ])
                .into_element(cx)
                .test_id("mechanism-button-group-text"),
            ]
        }
    });

    let mut observed = ObservedTree::from_semantics_snapshot(&snapshot, bounds);
    for node in &snapshot.nodes {
        if let Some(layout) = ui.debug_node_bounds(node.id) {
            observed.set_layout_bounds_for_node_id(node.id.data().as_ffi(), layout);
        }
    }

    Ok(observed)
}
