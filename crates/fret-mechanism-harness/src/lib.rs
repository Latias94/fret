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
    BoundsSpace, ObservedHitTestSample, ObservedMechanismMetric, ObservedNode, ObservedOverlay,
    ObservedRoot, ObservedSemanticsAction, ObservedSemanticsActions, ObservedSemanticsCheckedState,
    ObservedSemanticsFlag, ObservedSemanticsLive, ObservedSemanticsNumeric,
    ObservedSemanticsPressedState, ObservedSemanticsRelation, ObservedSemanticsScroll,
    ObservedTextRange, ObservedTextSelection, ObservedTree, QueryError, role_label,
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
    use fret_diag_protocol::{
        UiBoundsMetricV1, UiComparisonV1, UiPredicateV1, UiSelectorV1, UiSemanticsNumericFieldV1,
        UiSemanticsScrollFieldV1,
    };
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

    #[test]
    fn hit_test_sample_layer_routing_oracles_match_roots() {
        let mut tree = ObservedTree::new(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        ));
        let mut base = ObservedNode::new(
            "base-root",
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(200.0), Px(120.0)),
            ),
        );
        base.node_id = Some(1);
        let mut modal = ObservedNode::new(
            "modal-root",
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(200.0), Px(120.0)),
            ),
        );
        modal.node_id = Some(2);
        tree.barrier_root_node_id = Some(2);
        tree.push_node(base);
        tree.push_node(modal);
        tree.push_hit_test_sample(ObservedHitTestSample {
            id: "modal-center".to_string(),
            point: Point::new(Px(10.0), Px(10.0)),
            hit_node_id: None,
            hit_test_id: None,
            barrier_root_node_id: Some(2),
            active_layer_root_node_ids: vec![2],
        });

        evaluate_predicate(
            &tree,
            &MechanismPredicate::HitTestSample {
                sample_id: "modal-center".to_string(),
                target: None,
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::HitTestSampleBarrierRoot {
                sample_id: "modal-center".to_string(),
                target: Some(UiSelectorV1::TestId {
                    id: "modal-root".to_string(),
                    root_z_index: None,
                }),
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::HitTestSampleActiveLayerRootAt {
                sample_id: "modal-center".to_string(),
                index: 0,
                target: UiSelectorV1::TestId {
                    id: "modal-root".to_string(),
                    root_z_index: None,
                },
            },
        )
        .unwrap();
    }

    #[test]
    fn mechanism_metrics_can_assert_non_geometry_facts() {
        let mut tree = ObservedTree::new(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ));
        tree.set_metric("node.root.subtree_layout_dirty_count", 1.0);
        tree.set_metric("node.child.exists", 0.0);

        evaluate_predicate(
            &tree,
            &MechanismPredicate::MechanismMetric {
                metric_id: "node.root.subtree_layout_dirty_count".to_string(),
                comparison: UiComparisonV1::Eq,
                value: 1.0,
                eps: 0.0,
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::MechanismMetric {
                metric_id: "node.child.exists".to_string(),
                comparison: UiComparisonV1::Eq,
                value: 0.0,
                eps: 0.0,
            },
        )
        .unwrap();
    }

    #[test]
    fn semantics_relation_and_flag_oracles_match_observed_nodes() {
        let mut tree = ObservedTree::new(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ));

        let mut combo = ObservedNode::new(
            "combo",
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(80.0), Px(24.0))),
        );
        combo.node_id = Some(1);
        combo.active_descendant_node_id = Some(2);
        combo.controls_node_ids = vec![3];
        combo.disabled = Some(true);

        let mut option = ObservedNode::new(
            "option",
            Rect::new(Point::new(Px(0.0), Px(24.0)), Size::new(Px(80.0), Px(24.0))),
        );
        option.node_id = Some(2);

        let mut listbox = ObservedNode::new(
            "listbox",
            Rect::new(Point::new(Px(0.0), Px(48.0)), Size::new(Px(80.0), Px(48.0))),
        );
        listbox.node_id = Some(3);

        tree.push_node(combo);
        tree.push_node(option);
        tree.push_node(listbox);

        evaluate_predicate(
            &tree,
            &MechanismPredicate::SemanticsRelationIncludes {
                source: UiSelectorV1::TestId {
                    id: "combo".to_string(),
                    root_z_index: None,
                },
                relation: ObservedSemanticsRelation::ActiveDescendant,
                target: UiSelectorV1::TestId {
                    id: "option".to_string(),
                    root_z_index: None,
                },
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::SemanticsRelationIncludes {
                source: UiSelectorV1::TestId {
                    id: "combo".to_string(),
                    root_z_index: None,
                },
                relation: ObservedSemanticsRelation::Controls,
                target: UiSelectorV1::TestId {
                    id: "listbox".to_string(),
                    root_z_index: None,
                },
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::SemanticsFlagIs {
                target: UiSelectorV1::TestId {
                    id: "combo".to_string(),
                    root_z_index: None,
                },
                flag: ObservedSemanticsFlag::Disabled,
                expected: true,
            },
        )
        .unwrap();
    }

    #[test]
    fn semantics_relation_oracles_resolve_targets_across_barrier_roots() {
        let mut tree = ObservedTree::new(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ));

        let mut trigger = ObservedNode::new(
            "trigger",
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(80.0), Px(24.0))),
        );
        trigger.node_id = Some(1);
        trigger.controls_node_ids = vec![2];

        let mut listbox = ObservedNode::new(
            "listbox",
            Rect::new(Point::new(Px(0.0), Px(24.0)), Size::new(Px(80.0), Px(48.0))),
        );
        listbox.node_id = Some(2);
        listbox.labelled_by_node_ids = vec![1];

        tree.barrier_root_node_id = Some(2);
        tree.push_node(trigger);
        tree.push_node(listbox);

        evaluate_predicate(
            &tree,
            &MechanismPredicate::SemanticsRelationIncludes {
                source: UiSelectorV1::TestId {
                    id: "listbox".to_string(),
                    root_z_index: None,
                },
                relation: ObservedSemanticsRelation::LabelledBy,
                target: UiSelectorV1::TestId {
                    id: "trigger".to_string(),
                    root_z_index: None,
                },
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::SemanticsRelationIncludes {
                source: UiSelectorV1::TestId {
                    id: "trigger".to_string(),
                    root_z_index: None,
                },
                relation: ObservedSemanticsRelation::Controls,
                target: UiSelectorV1::TestId {
                    id: "listbox".to_string(),
                    root_z_index: None,
                },
            },
        )
        .unwrap();
    }

    #[test]
    fn semantics_value_state_actions_and_structured_metadata_are_queryable() {
        let mut tree = ObservedTree::new(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        ));

        let mut input = ObservedNode::new(
            "editor",
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(28.0))),
        );
        input.node_id = Some(1);
        input.role = Some("text_field".to_string());
        input.label = Some("Editor".to_string());
        input.value = Some("hello".to_string());
        input.text_selection = Some(ObservedTextSelection {
            anchor: 2,
            focus: 2,
        });
        input.text_composition = Some(ObservedTextRange { start: 1, end: 3 });
        input.actions = ObservedSemanticsActions {
            focus: true,
            set_text_selection: true,
            ..Default::default()
        };

        let mut option = ObservedNode::new(
            "option",
            Rect::new(
                Point::new(Px(0.0), Px(30.0)),
                Size::new(Px(100.0), Px(24.0)),
            ),
        );
        option.node_id = Some(2);
        option.selected = Some(true);
        option.checked = Some(true);
        option.pos_in_set = Some(2);
        option.set_size = Some(5);
        option.actions.invoke = true;

        let mut status = ObservedNode::new(
            "status",
            Rect::new(
                Point::new(Px(0.0), Px(60.0)),
                Size::new(Px(100.0), Px(24.0)),
            ),
        );
        status.node_id = Some(3);
        status.live = Some(ObservedSemanticsLive::Polite);
        status.live_atomic = Some(true);
        status.numeric = Some(ObservedSemanticsNumeric {
            value: Some(50.0),
            min: Some(0.0),
            max: Some(100.0),
            step: Some(1.0),
            jump: Some(10.0),
        });
        status.scroll = Some(ObservedSemanticsScroll {
            y: Some(40.0),
            y_min: Some(0.0),
            y_max: Some(120.0),
            ..Default::default()
        });

        tree.push_node(input);
        tree.push_node(option);
        tree.push_node(status);

        let editor = UiSelectorV1::TestId {
            id: "editor".to_string(),
            root_z_index: None,
        };
        let option_selector = UiSelectorV1::TestId {
            id: "option".to_string(),
            root_z_index: None,
        };
        let status_selector = UiSelectorV1::TestId {
            id: "status".to_string(),
            root_z_index: None,
        };

        evaluate_predicate(
            &tree,
            &MechanismPredicate::UiPredicate {
                predicate: UiPredicateV1::ValueEquals {
                    target: editor.clone(),
                    text: "hello".to_string(),
                },
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::UiPredicate {
                predicate: UiPredicateV1::TextCompositionIs {
                    target: editor.clone(),
                    composing: true,
                },
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::SemanticsTextSelectionIs {
                target: editor.clone(),
                expected: Some(ObservedTextSelection {
                    anchor: 2,
                    focus: 2,
                }),
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::SemanticsActionIs {
                target: editor,
                action: ObservedSemanticsAction::SetTextSelection,
                expected: true,
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::UiPredicate {
                predicate: UiPredicateV1::SelectedIs {
                    target: option_selector.clone(),
                    selected: true,
                },
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::UiPredicate {
                predicate: UiPredicateV1::CheckedIs {
                    target: option_selector.clone(),
                    checked: true,
                },
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::UiPredicate {
                predicate: UiPredicateV1::PosInSetIs {
                    target: option_selector.clone(),
                    pos_in_set: 2,
                },
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::SemanticsActionIs {
                target: option_selector,
                action: ObservedSemanticsAction::Invoke,
                expected: true,
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::SemanticsLiveIs {
                target: status_selector.clone(),
                live: Some(ObservedSemanticsLive::Polite),
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::UiPredicate {
                predicate: UiPredicateV1::SemanticsNumericApproxEq {
                    target: status_selector.clone(),
                    field: UiSemanticsNumericFieldV1::Value,
                    value: 50.0,
                    eps: 0.01,
                },
            },
        )
        .unwrap();
        evaluate_predicate(
            &tree,
            &MechanismPredicate::UiPredicate {
                predicate: UiPredicateV1::SemanticsScrollApproxEq {
                    target: status_selector,
                    field: UiSemanticsScrollFieldV1::YMax,
                    value: 120.0,
                    eps: 0.01,
                },
            },
        )
        .unwrap();
    }

    #[test]
    fn active_item_none_rejects_roving_focus_descendants() {
        let mut tree = ObservedTree::new(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ));

        let mut listbox = ObservedNode::new(
            "listbox",
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(48.0))),
        );
        listbox.node_id = Some(1);

        let mut option = ObservedNode::new(
            "option",
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(24.0))),
        );
        option.node_id = Some(2);
        option.parent_node_id = Some(1);

        tree.push_node(listbox);
        tree.push_node(option);

        let predicate = MechanismPredicate::UiPredicate {
            predicate: UiPredicateV1::ActiveItemIsNone {
                container: UiSelectorV1::TestId {
                    id: "listbox".to_string(),
                    root_z_index: None,
                },
            },
        };

        tree.focus_node_id = Some(1);
        evaluate_predicate(&tree, &predicate).unwrap();

        tree.focus_node_id = Some(2);
        assert!(
            evaluate_predicate(&tree, &predicate).is_err(),
            "roving focus on a descendant should count as an active item"
        );
    }

    #[test]
    fn focus_oracle_can_match_restored_focus_outside_pointer_barrier() {
        let mut tree = ObservedTree::new(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        ));
        let mut trigger = ObservedNode::new(
            "trigger",
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(80.0), Px(24.0))),
        );
        trigger.node_id = Some(1);
        let mut modal = ObservedNode::new(
            "modal",
            Rect::new(
                Point::new(Px(0.0), Px(30.0)),
                Size::new(Px(120.0), Px(80.0)),
            ),
        );
        modal.node_id = Some(2);
        tree.focus_node_id = Some(1);
        tree.barrier_root_node_id = Some(2);
        tree.push_node(trigger);
        tree.push_node(modal);

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
    }
}
