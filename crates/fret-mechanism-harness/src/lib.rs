//! Shared harness primitives for Fret runtime mechanism conformance.
//!
//! This crate intentionally stays below component policy. It models controllable cases, observable
//! runtime facts, and reusable oracles so `fret-ui`, ecosystem recipes, and diagnostics scripts can
//! assert the same mechanism outcomes without sharing UI construction code.

mod fixture;
mod observe;
mod oracle;
mod runner;

pub use fixture::{
    CaseEvidence, MechanismCase, MechanismDomain, MechanismOracle, MechanismSuite,
    MechanismSuiteLoadError,
};
pub use observe::{
    BoundsSpace, ObservedHitTestSample, ObservedNode, ObservedOverlay, ObservedRoot, ObservedTree,
    QueryError, role_label,
};
pub use oracle::{
    MechanismPredicate, OracleEvalError, PredicateFailure, PredicatePass, evaluate_predicate,
};
pub use runner::{
    CaseReport, MechanismHarness, MechanismReport, ScenarioObserveError, ScenarioObserver,
};

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px, Rect, Size};
    use fret_diag_protocol::{UiBoundsMetricV1, UiComparisonV1, UiPredicateV1, UiSelectorV1};
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Clone, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum Scenario {
        Static,
    }

    #[test]
    fn fixture_runner_reports_case_addressable_failures() {
        let raw = r#"
        {
          "schema_version": 1,
          "suite_id": "mechanism-harness.self-test",
          "owner_layer": "crates/fret-mechanism-harness",
          "domains": ["layout"],
          "cases": [
            {
              "id": "static-pass",
              "scenario": { "kind": "static" },
              "oracle": {
                "predicates": [
                  {
                    "kind": "bounds_metric",
                    "target": { "kind": "test_id", "id": "box" },
                    "metric": "width",
                    "comparison": "eq",
                    "value_px": 20.0,
                    "eps_px": 0.01
                  }
                ]
              }
            }
          ]
        }"#;
        let suite: MechanismSuite<Scenario> = MechanismSuite::from_json_str(raw).unwrap();
        let mut observer = |_case: &MechanismCase<Scenario>| {
            let mut tree = ObservedTree::new(Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(100.0), Px(100.0)),
            ));
            tree.push_node(ObservedNode::new(
                "box",
                Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(10.0))),
            ));
            Ok(tree)
        };

        let report = MechanismHarness::new().run_suite(&suite, &mut observer);
        assert!(report.passed(), "{}", report.failure_summary());
    }

    #[test]
    fn bounds_metric_delta_can_express_axis_alignment() {
        let mut tree = ObservedTree::new(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ));
        tree.push_node(ObservedNode::new(
            "a",
            Rect::new(Point::new(Px(0.0), Px(10.0)), Size::new(Px(20.0), Px(10.0))),
        ));
        tree.push_node(ObservedNode::new(
            "b",
            Rect::new(Point::new(Px(30.0), Px(5.0)), Size::new(Px(20.0), Px(20.0))),
        ));

        let pred = MechanismPredicate::BoundsMetricDelta {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
                root_z_index: None,
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
                root_z_index: None,
            },
            metric: UiBoundsMetricV1::CenterY,
            comparison: UiComparisonV1::Eq,
            value_px: 0.0,
            a_space: BoundsSpace::Layout,
            b_space: BoundsSpace::Layout,
            eps_px: 0.01,
        };

        evaluate_predicate(&tree, &pred).unwrap();
    }

    #[test]
    fn overlay_and_focus_observations_are_queryable_oracles() {
        let mut tree = ObservedTree::new(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        ));
        let mut trigger = ObservedNode::new(
            "trigger",
            Rect::new(Point::new(Px(8.0), Px(8.0)), Size::new(Px(80.0), Px(24.0))),
        );
        trigger.node_id = Some(7);
        tree.focus_node_id = Some(7);
        tree.push_node(trigger);
        tree.overlays.push(ObservedOverlay {
            id: "menu".to_string(),
            anchor: Some(UiSelectorV1::TestId {
                id: "trigger".to_string(),
                root_z_index: None,
            }),
            panel: None,
            bounds: Some(Rect::new(
                Point::new(Px(8.0), Px(36.0)),
                Size::new(Px(120.0), Px(64.0)),
            )),
        });

        assert_eq!(
            tree.focus_node().and_then(|node| node.test_id.as_deref()),
            Some("trigger")
        );
        evaluate_predicate(
            &tree,
            &MechanismPredicate::UiPredicate {
                predicate: UiPredicateV1::FocusIs {
                    target: UiSelectorV1::TestId {
                        id: "trigger".to_string(),
                        root_z_index: None,
                    },
                },
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::OverlayBoundsMetric {
                overlay_id: "menu".to_string(),
                metric: UiBoundsMetricV1::Width,
                comparison: UiComparisonV1::Eq,
                value_px: 120.0,
                eps_px: 0.01,
            },
        )
        .unwrap();
    }
}
