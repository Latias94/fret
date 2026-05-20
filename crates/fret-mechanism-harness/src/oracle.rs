use fret_core::Rect;
use fret_diag_protocol::{
    UiBoundsMetricV1, UiComparisonV1, UiPredicateV1, UiSelectorV1, UiSemanticsActionV1,
    UiSemanticsCheckedStateV1, UiSemanticsLiveV1, UiSemanticsNumericFieldV1,
    UiSemanticsPressedStateV1, UiSemanticsScrollFieldV1,
};
use serde::{Deserialize, Serialize};

use crate::{
    BoundsSpace, ObservedNode, ObservedSemanticsAction, ObservedSemanticsCheckedState,
    ObservedSemanticsFlag, ObservedSemanticsLive, ObservedSemanticsPressedState,
    ObservedSemanticsRelation, ObservedTextRange, ObservedTextSelection, ObservedTree,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MechanismPredicate {
    UiPredicate {
        predicate: UiPredicateV1,
    },
    BoundsRect {
        target: UiSelectorV1,
        #[serde(default)]
        space: BoundsSpace,
        expected: Rect,
        #[serde(default)]
        eps_px: f32,
    },
    BoundsMetric {
        target: UiSelectorV1,
        #[serde(default)]
        space: BoundsSpace,
        metric: UiBoundsMetricV1,
        comparison: UiComparisonV1,
        value_px: f32,
        #[serde(default)]
        eps_px: f32,
    },
    BoundsMetricDelta {
        a: UiSelectorV1,
        b: UiSelectorV1,
        metric: UiBoundsMetricV1,
        comparison: UiComparisonV1,
        value_px: f32,
        #[serde(default)]
        a_space: BoundsSpace,
        #[serde(default)]
        b_space: BoundsSpace,
        #[serde(default)]
        eps_px: f32,
    },
    BoundsSpaceMetricDelta {
        target: UiSelectorV1,
        a_space: BoundsSpace,
        b_space: BoundsSpace,
        metric: UiBoundsMetricV1,
        comparison: UiComparisonV1,
        value_px: f32,
        #[serde(default)]
        eps_px: f32,
    },
    HitTestSample {
        sample_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<UiSelectorV1>,
    },
    HitTestSampleNot {
        sample_id: String,
        target: UiSelectorV1,
    },
    HitTestSampleBarrierRoot {
        sample_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<UiSelectorV1>,
    },
    HitTestSampleActiveLayerRootAt {
        sample_id: String,
        index: usize,
        target: UiSelectorV1,
    },
    OverlayExists {
        overlay_id: String,
    },
    OverlayBoundsRect {
        overlay_id: String,
        expected: Rect,
        #[serde(default)]
        eps_px: f32,
    },
    OverlayBoundsMetric {
        overlay_id: String,
        metric: UiBoundsMetricV1,
        comparison: UiComparisonV1,
        value_px: f32,
        #[serde(default)]
        eps_px: f32,
    },
    MechanismMetric {
        metric_id: String,
        comparison: UiComparisonV1,
        value: f32,
        #[serde(default)]
        eps: f32,
    },
    SemanticsRelationIncludes {
        source: UiSelectorV1,
        relation: ObservedSemanticsRelation,
        target: UiSelectorV1,
    },
    SemanticsFlagIs {
        target: UiSelectorV1,
        flag: ObservedSemanticsFlag,
        expected: bool,
    },
    SemanticsActionIs {
        target: UiSelectorV1,
        action: ObservedSemanticsAction,
        expected: bool,
    },
    SemanticsLiveIs {
        target: UiSelectorV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        live: Option<ObservedSemanticsLive>,
    },
    SemanticsLiveAtomicIs {
        target: UiSelectorV1,
        expected: bool,
    },
    SemanticsTextSelectionIs {
        target: UiSelectorV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected: Option<ObservedTextSelection>,
    },
    SemanticsTextCompositionRangeIs {
        target: UiSelectorV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected: Option<ObservedTextRange>,
    },
}

#[derive(Debug, Clone)]
pub struct PredicatePass;

#[derive(Debug, Clone)]
pub struct PredicateFailure {
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OracleEvalError {
    #[error("{0}")]
    Failure(String),
}

pub fn evaluate_predicate(
    tree: &ObservedTree,
    predicate: &MechanismPredicate,
) -> Result<PredicatePass, PredicateFailure> {
    match predicate {
        MechanismPredicate::UiPredicate { predicate } => eval_ui_predicate(tree, predicate),
        MechanismPredicate::BoundsRect {
            target,
            space,
            expected,
            eps_px,
        } => {
            let have = tree.bounds_for(target, *space).map_err(fail)?;
            if rect_approx_eq(have, *expected, eps_px.max(0.0)) {
                Ok(PredicatePass)
            } else {
                Err(failure(format!(
                    "bounds_rect mismatch target={target:?} space={space:?} expected={expected:?} actual={have:?} eps={}",
                    eps_px.max(0.0)
                )))
            }
        }
        MechanismPredicate::BoundsMetric {
            target,
            space,
            metric,
            comparison,
            value_px,
            eps_px,
        } => {
            let bounds = tree.bounds_for(target, *space).map_err(fail)?;
            let have = bounds_metric_value(bounds, *metric);
            pass_compare(have, *comparison, *value_px, eps_px.max(0.0), || {
                format!(
                    "bounds_metric mismatch target={target:?} space={space:?} metric={metric:?} comparison={comparison:?} expected={value_px} actual={have}"
                )
            })
        }
        MechanismPredicate::BoundsMetricDelta {
            a,
            b,
            metric,
            comparison,
            value_px,
            a_space,
            b_space,
            eps_px,
        } => {
            let a_bounds = tree.bounds_for(a, *a_space).map_err(fail)?;
            let b_bounds = tree.bounds_for(b, *b_space).map_err(fail)?;
            let have =
                bounds_metric_value(a_bounds, *metric) - bounds_metric_value(b_bounds, *metric);
            pass_compare(have, *comparison, *value_px, eps_px.max(0.0), || {
                format!(
                    "bounds_metric_delta mismatch a={a:?} b={b:?} metric={metric:?} comparison={comparison:?} expected={value_px} actual={have} a_bounds={a_bounds:?} b_bounds={b_bounds:?}"
                )
            })
        }
        MechanismPredicate::BoundsSpaceMetricDelta {
            target,
            a_space,
            b_space,
            metric,
            comparison,
            value_px,
            eps_px,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let a_bounds = node.bounds_in(*a_space);
            let b_bounds = node.bounds_in(*b_space);
            let have =
                bounds_metric_value(a_bounds, *metric) - bounds_metric_value(b_bounds, *metric);
            pass_compare(have, *comparison, *value_px, eps_px.max(0.0), || {
                format!(
                    "bounds_space_metric_delta mismatch target={target:?} metric={metric:?} {a_space:?}-{b_space:?} comparison={comparison:?} expected={value_px} actual={have} a_bounds={a_bounds:?} b_bounds={b_bounds:?}"
                )
            })
        }
        MechanismPredicate::HitTestSample { sample_id, target } => {
            let Some(sample) = tree.hit_sample(sample_id) else {
                return Err(failure(format!("missing hit-test sample {sample_id:?}")));
            };
            match target {
                None if sample.hit_node_id.is_none() && sample.hit_test_id.is_none() => {
                    Ok(PredicatePass)
                }
                None => Err(failure(format!(
                    "expected hit-test sample {sample_id:?} to miss, got node={:?} test_id={:?}",
                    sample.hit_node_id, sample.hit_test_id
                ))),
                Some(target) => {
                    let expected = tree.select_best(target).map_err(fail)?;
                    if hit_matches(expected, sample.hit_node_id, sample.hit_test_id.as_deref()) {
                        Ok(PredicatePass)
                    } else {
                        Err(failure(format!(
                            "hit-test sample {sample_id:?} mismatch expected={target:?} got node={:?} test_id={:?}",
                            sample.hit_node_id, sample.hit_test_id
                        )))
                    }
                }
            }
        }
        MechanismPredicate::HitTestSampleNot { sample_id, target } => {
            let Some(sample) = tree.hit_sample(sample_id) else {
                return Err(failure(format!("missing hit-test sample {sample_id:?}")));
            };
            let expected = tree.select_best(target).map_err(fail)?;
            if hit_matches(expected, sample.hit_node_id, sample.hit_test_id.as_deref()) {
                Err(failure(format!(
                    "hit-test sample {sample_id:?} unexpectedly matched target={target:?} node={:?} test_id={:?}",
                    sample.hit_node_id, sample.hit_test_id
                )))
            } else {
                Ok(PredicatePass)
            }
        }
        MechanismPredicate::HitTestSampleBarrierRoot { sample_id, target } => {
            let Some(sample) = tree.hit_sample(sample_id) else {
                return Err(failure(format!("missing hit-test sample {sample_id:?}")));
            };
            match target {
                None if sample.barrier_root_node_id.is_none() => Ok(PredicatePass),
                None => Err(failure(format!(
                    "expected hit-test sample {sample_id:?} to have no barrier root, got {:?}",
                    sample.barrier_root_node_id
                ))),
                Some(target) => {
                    let Some(root) = sample.barrier_root_node_id else {
                        return Err(failure(format!(
                            "expected hit-test sample {sample_id:?} barrier root to match {target:?}, got none"
                        )));
                    };
                    if selector_matches_node_id(tree, target, root) {
                        Ok(PredicatePass)
                    } else {
                        Err(failure(format!(
                            "hit-test sample {sample_id:?} barrier root mismatch expected={target:?} got={root:?}"
                        )))
                    }
                }
            }
        }
        MechanismPredicate::HitTestSampleActiveLayerRootAt {
            sample_id,
            index,
            target,
        } => {
            let Some(sample) = tree.hit_sample(sample_id) else {
                return Err(failure(format!("missing hit-test sample {sample_id:?}")));
            };
            let Some(root) = sample.active_layer_root_node_ids.get(*index).copied() else {
                return Err(failure(format!(
                    "hit-test sample {sample_id:?} missing active layer root at index={index} roots={:?}",
                    sample.active_layer_root_node_ids
                )));
            };
            if selector_matches_node_id(tree, target, root) {
                Ok(PredicatePass)
            } else {
                Err(failure(format!(
                    "hit-test sample {sample_id:?} active layer root mismatch index={index} expected={target:?} got={root:?} roots={:?}",
                    sample.active_layer_root_node_ids
                )))
            }
        }
        MechanismPredicate::OverlayExists { overlay_id } => {
            pass_bool(tree.overlay(overlay_id).is_some(), || {
                format!("expected overlay to exist: {overlay_id:?}")
            })
        }
        MechanismPredicate::OverlayBoundsRect {
            overlay_id,
            expected,
            eps_px,
        } => {
            let have = tree.overlay_bounds_for(overlay_id).map_err(fail)?;
            if rect_approx_eq(have, *expected, eps_px.max(0.0)) {
                Ok(PredicatePass)
            } else {
                Err(failure(format!(
                    "overlay_bounds_rect mismatch overlay_id={overlay_id:?} expected={expected:?} actual={have:?} eps={}",
                    eps_px.max(0.0)
                )))
            }
        }
        MechanismPredicate::OverlayBoundsMetric {
            overlay_id,
            metric,
            comparison,
            value_px,
            eps_px,
        } => {
            let bounds = tree.overlay_bounds_for(overlay_id).map_err(fail)?;
            let have = bounds_metric_value(bounds, *metric);
            pass_compare(have, *comparison, *value_px, eps_px.max(0.0), || {
                format!(
                    "overlay_bounds_metric mismatch overlay_id={overlay_id:?} metric={metric:?} comparison={comparison:?} expected={value_px} actual={have}"
                )
            })
        }
        MechanismPredicate::MechanismMetric {
            metric_id,
            comparison,
            value,
            eps,
        } => {
            let have = tree.metric_value(metric_id).map_err(fail)?;
            pass_compare(have, *comparison, *value, eps.max(0.0), || {
                format!(
                    "mechanism_metric mismatch metric_id={metric_id:?} comparison={comparison:?} expected={value} actual={have}"
                )
            })
        }
        MechanismPredicate::SemanticsRelationIncludes {
            source,
            relation,
            target,
        } => {
            let source_node = tree.select_best_unfiltered(source).map_err(fail)?;
            let target_node = tree.select_best_unfiltered(target).map_err(fail)?;
            let Some(target_id) = target_node.node_id else {
                return Err(failure(format!(
                    "semantics_relation_includes target has no node id target={target:?}"
                )));
            };
            let relation_ids = semantics_relation_ids(source_node, *relation);
            pass_bool(relation_ids.contains(&target_id), || {
                format!(
                    "semantics_relation_includes mismatch source={source:?} relation={relation:?} target={target:?} expected_node={target_id:?} actual={relation_ids:?}"
                )
            })
        }
        MechanismPredicate::SemanticsFlagIs {
            target,
            flag,
            expected,
        } => {
            let node = tree.select_best_unfiltered(target).map_err(fail)?;
            let Some(actual) = semantics_flag_value(node, *flag) else {
                return Err(failure(format!(
                    "semantics_flag_is target has no observed flag target={target:?} flag={flag:?}"
                )));
            };
            pass_bool(actual == *expected, || {
                format!(
                    "semantics_flag_is mismatch target={target:?} flag={flag:?} expected={expected} actual={actual}"
                )
            })
        }
        MechanismPredicate::SemanticsActionIs {
            target,
            action,
            expected,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let actual = node.actions.get(*action);
            pass_bool(actual == *expected, || {
                format!(
                    "semantics_action_is mismatch target={target:?} action={action:?} expected={expected} actual={actual}"
                )
            })
        }
        MechanismPredicate::SemanticsLiveIs { target, live } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.live == *live, || {
                format!(
                    "semantics_live_is mismatch target={target:?} expected={live:?} actual={:?}",
                    node.live
                )
            })
        }
        MechanismPredicate::SemanticsLiveAtomicIs { target, expected } => {
            let node = tree.select_best(target).map_err(fail)?;
            let Some(actual) = node.live_atomic else {
                return Err(failure(format!(
                    "semantics_live_atomic_is target has no observed live_atomic target={target:?}"
                )));
            };
            pass_bool(actual == *expected, || {
                format!(
                    "semantics_live_atomic_is mismatch target={target:?} expected={expected} actual={actual}"
                )
            })
        }
        MechanismPredicate::SemanticsTextSelectionIs { target, expected } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.text_selection == *expected, || {
                format!(
                    "semantics_text_selection_is mismatch target={target:?} expected={expected:?} actual={:?}",
                    node.text_selection
                )
            })
        }
        MechanismPredicate::SemanticsTextCompositionRangeIs { target, expected } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.text_composition == *expected, || {
                format!(
                    "semantics_text_composition_range_is mismatch target={target:?} expected={expected:?} actual={:?}",
                    node.text_composition
                )
            })
        }
    }
}

fn semantics_relation_ids(node: &ObservedNode, relation: ObservedSemanticsRelation) -> Vec<u64> {
    match relation {
        ObservedSemanticsRelation::ActiveDescendant => {
            node.active_descendant_node_id.into_iter().collect()
        }
        ObservedSemanticsRelation::LabelledBy => node.labelled_by_node_ids.clone(),
        ObservedSemanticsRelation::DescribedBy => node.described_by_node_ids.clone(),
        ObservedSemanticsRelation::Controls => node.controls_node_ids.clone(),
    }
}

fn semantics_flag_value(node: &ObservedNode, flag: ObservedSemanticsFlag) -> Option<bool> {
    match flag {
        ObservedSemanticsFlag::Disabled => node.disabled,
        ObservedSemanticsFlag::Hidden => node.hidden,
        ObservedSemanticsFlag::Selected => node.selected,
        ObservedSemanticsFlag::Expanded => node.expanded,
        ObservedSemanticsFlag::Checked => node.checked,
        ObservedSemanticsFlag::LiveAtomic => node.live_atomic,
    }
}

fn eval_ui_predicate(
    tree: &ObservedTree,
    predicate: &UiPredicateV1,
) -> Result<PredicatePass, PredicateFailure> {
    match predicate {
        UiPredicateV1::Exists { target } => pass_bool(!tree.select(target).is_empty(), || {
            format!("expected selector to exist: {target:?}")
        }),
        UiPredicateV1::NotExists { target } => pass_bool(tree.select(target).is_empty(), || {
            format!("expected selector to be absent: {target:?}")
        }),
        UiPredicateV1::ExistsUnder { scope, target } => {
            pass_bool(!tree.select_under(scope, target).is_empty(), || {
                format!("expected selector {target:?} under scope {scope:?}")
            })
        }
        UiPredicateV1::NotExistsUnder { scope, target } => {
            pass_bool(tree.select_under(scope, target).is_empty(), || {
                format!("expected selector {target:?} to be absent under scope {scope:?}")
            })
        }
        UiPredicateV1::FocusIs { target } => {
            let expected = tree.select_best_unfiltered(target).map_err(fail)?;
            pass_bool(expected.node_id == tree.focus_node_id, || {
                format!(
                    "expected focus target {target:?}, actual={:?}",
                    tree.focus_node_id
                )
            })
        }
        UiPredicateV1::FocusedDescendantIs { scope, target } => {
            let Ok(scope) = tree.select_best(scope) else {
                return Err(failure("focused_descendant scope was not found"));
            };
            let Ok(target) = tree.select_best(target) else {
                return Err(failure("focused_descendant target was not found"));
            };
            let focus = tree.focus_node_id;
            let matches = target.node_id == focus
                && scope
                    .node_id
                    .zip(focus)
                    .map(|(scope, focus)| {
                        focus == scope
                            || tree
                                .select(&UiSelectorV1::NodeId {
                                    node: focus,
                                    root_z_index: None,
                                })
                                .first()
                                .is_some_and(|_| {
                                    tree.select_under(
                                        &UiSelectorV1::NodeId {
                                            node: scope,
                                            root_z_index: None,
                                        },
                                        &UiSelectorV1::NodeId {
                                            node: focus,
                                            root_z_index: None,
                                        },
                                    )
                                    .len()
                                        == 1
                                })
                    })
                    .unwrap_or(false);
            pass_bool(matches, || "focused_descendant mismatch".to_string())
        }
        UiPredicateV1::RoleIs { target, role } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.role.as_deref() == Some(role.as_str()), || {
                format!(
                    "role mismatch target={target:?} expected={role:?} actual={:?}",
                    node.role
                )
            })
        }
        UiPredicateV1::LabelContains { target, text } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(
                node.label
                    .as_deref()
                    .is_some_and(|label| label.contains(text)),
                || {
                    format!(
                        "label mismatch target={target:?} expected contains={text:?} actual={:?}",
                        node.label
                    )
                },
            )
        }
        UiPredicateV1::LabelLenIs { target, len_bytes } => {
            let node = tree.select_best(target).map_err(fail)?;
            let have = node.label.as_deref().map(|label| label.len() as u32);
            pass_bool(have == Some(*len_bytes), || {
                format!(
                    "label_len_is mismatch target={target:?} expected={len_bytes} actual={have:?}"
                )
            })
        }
        UiPredicateV1::LabelLenGe {
            target,
            min_len_bytes,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let have = node.label.as_deref().map(|label| label.len() as u32);
            pass_bool(have.is_some_and(|len| len >= *min_len_bytes), || {
                format!(
                    "label_len_ge mismatch target={target:?} expected_min={min_len_bytes} actual={have:?}"
                )
            })
        }
        UiPredicateV1::ValueContains { target, text } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(
                node.value
                    .as_deref()
                    .is_some_and(|value| value.contains(text)),
                || {
                    format!(
                        "value_contains mismatch target={target:?} expected contains={text:?} actual={:?}",
                        node.value
                    )
                },
            )
        }
        UiPredicateV1::ValueEquals { target, text } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.value.as_deref() == Some(text.as_str()), || {
                format!(
                    "value_equals mismatch target={target:?} expected={text:?} actual={:?}",
                    node.value
                )
            })
        }
        UiPredicateV1::ValueLenIs { target, len_bytes } => {
            let node = tree.select_best(target).map_err(fail)?;
            let have = node.value.as_deref().map(|value| value.len() as u32);
            pass_bool(have == Some(*len_bytes), || {
                format!(
                    "value_len_is mismatch target={target:?} expected={len_bytes} actual={have:?}"
                )
            })
        }
        UiPredicateV1::ValueLenGe {
            target,
            min_len_bytes,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let have = node.value.as_deref().map(|value| value.len() as u32);
            pass_bool(have.is_some_and(|len| len >= *min_len_bytes), || {
                format!(
                    "value_len_ge mismatch target={target:?} expected_min={min_len_bytes} actual={have:?}"
                )
            })
        }
        UiPredicateV1::PosInSetIs { target, pos_in_set } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.pos_in_set == Some(*pos_in_set), || {
                format!(
                    "pos_in_set_is mismatch target={target:?} expected={pos_in_set} actual={:?}",
                    node.pos_in_set
                )
            })
        }
        UiPredicateV1::SetSizeIs { target, set_size } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.set_size == Some(*set_size), || {
                format!(
                    "set_size_is mismatch target={target:?} expected={set_size} actual={:?}",
                    node.set_size
                )
            })
        }
        UiPredicateV1::LevelIs { target, level } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.level == Some(*level), || {
                format!(
                    "level_is mismatch target={target:?} expected={level} actual={:?}",
                    node.level
                )
            })
        }
        UiPredicateV1::CheckedIs { target, checked } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.checked == Some(*checked), || {
                format!(
                    "checked_is mismatch target={target:?} expected={checked} actual={:?}",
                    node.checked
                )
            })
        }
        UiPredicateV1::CheckedStateIs { target, state } => {
            let node = tree.select_best(target).map_err(fail)?;
            let expected = state.map(observed_checked_state_from_protocol);
            pass_bool(node.checked_state == expected, || {
                format!(
                    "checked_state_is mismatch target={target:?} expected={expected:?} actual={:?}",
                    node.checked_state
                )
            })
        }
        UiPredicateV1::PressedStateIs { target, state } => {
            let node = tree.select_best(target).map_err(fail)?;
            let expected = state.map(observed_pressed_state_from_protocol);
            pass_bool(node.pressed_state == expected, || {
                format!(
                    "pressed_state_is mismatch target={target:?} expected={expected:?} actual={:?}",
                    node.pressed_state
                )
            })
        }
        UiPredicateV1::ExpandedIs { target, expanded } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.expanded == Some(*expanded), || {
                format!(
                    "expanded_is mismatch target={target:?} expected={expanded} actual={:?}",
                    node.expanded
                )
            })
        }
        UiPredicateV1::SemanticsLiveIs { target, live } => {
            let node = tree.select_best(target).map_err(fail)?;
            let expected = live.map(observed_live_from_protocol);
            pass_bool(node.live == expected, || {
                format!(
                    "semantics_live_is mismatch target={target:?} expected={expected:?} actual={:?}",
                    node.live
                )
            })
        }
        UiPredicateV1::SemanticsLiveAtomicIs {
            target,
            live_atomic,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.live_atomic == Some(*live_atomic), || {
                format!(
                    "semantics_live_atomic_is mismatch target={target:?} expected={live_atomic} actual={:?}",
                    node.live_atomic
                )
            })
        }
        UiPredicateV1::SelectedIs { target, selected } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.selected == Some(*selected), || {
                format!(
                    "selected_is mismatch target={target:?} expected={selected} actual={:?}",
                    node.selected
                )
            })
        }
        UiPredicateV1::DisabledIs { target, disabled } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.disabled == Some(*disabled), || {
                format!(
                    "disabled_is mismatch target={target:?} expected={disabled} actual={:?}",
                    node.disabled
                )
            })
        }
        UiPredicateV1::SemanticsActionIs {
            target,
            action,
            enabled,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let actual = node.actions.get(observed_action_from_protocol(*action));
            pass_bool(actual == *enabled, || {
                format!(
                    "semantics_action_is mismatch target={target:?} action={action:?} expected={enabled} actual={actual}"
                )
            })
        }
        UiPredicateV1::CapturedIs { target, captured } => {
            let node = tree.select_best_unfiltered(target).map_err(fail)?;
            let actual = node.node_id == tree.captured_node_id;
            pass_bool(actual == *captured, || {
                format!(
                    "captured_is mismatch target={target:?} expected={captured} actual={actual} captured={:?} target_node={:?}",
                    tree.captured_node_id, node.node_id
                )
            })
        }
        UiPredicateV1::SemanticsNumericApproxEq {
            target,
            field,
            value,
            eps,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let have = semantics_numeric_value(node, *field);
            pass_bool(
                have.is_some_and(|have| (have - *value).abs() <= eps.max(0.0)),
                || {
                    format!(
                        "semantics_numeric_approx_eq mismatch target={target:?} field={field:?} expected={value} actual={have:?} eps={}",
                        eps.max(0.0)
                    )
                },
            )
        }
        UiPredicateV1::SemanticsScrollIsFinite { target, field } => {
            let node = tree.select_best(target).map_err(fail)?;
            let have = semantics_scroll_value(node, *field);
            pass_bool(have.is_some_and(f64::is_finite), || {
                format!(
                    "semantics_scroll_is_finite mismatch target={target:?} field={field:?} actual={have:?}"
                )
            })
        }
        UiPredicateV1::SemanticsScrollApproxEq {
            target,
            field,
            value,
            eps,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let have = semantics_scroll_value(node, *field);
            pass_bool(
                have.is_some_and(|have| (have - *value).abs() <= eps.max(0.0)),
                || {
                    format!(
                        "semantics_scroll_approx_eq mismatch target={target:?} field={field:?} expected={value} actual={have:?} eps={}",
                        eps.max(0.0)
                    )
                },
            )
        }
        UiPredicateV1::SemanticsScrollNotApproxEq {
            target,
            field,
            value,
            eps,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let have = semantics_scroll_value(node, *field);
            pass_bool(
                have.is_some_and(|have| (have - *value).abs() > eps.max(0.0)),
                || {
                    format!(
                        "semantics_scroll_not_approx_eq mismatch target={target:?} field={field:?} not_expected={value} actual={have:?} eps={}",
                        eps.max(0.0)
                    )
                },
            )
        }
        UiPredicateV1::TextCompositionIs { target, composing } => {
            let node = tree.select_best(target).map_err(fail)?;
            let have = node.text_composition.is_some();
            pass_bool(have == *composing, || {
                format!(
                    "text_composition_is mismatch target={target:?} expected={composing} actual={have} range={:?}",
                    node.text_composition
                )
            })
        }
        UiPredicateV1::CheckedIsNone { target } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(node.checked.is_none(), || {
                format!(
                    "checked_is_none mismatch target={target:?} checked={:?}",
                    node.checked
                )
            })
        }
        UiPredicateV1::ActiveItemIs { container, item } => {
            let container = tree.select_best(container).map_err(fail)?;
            let item = tree.select_best_unfiltered(item).map_err(fail)?;
            let item_id = item.node_id;
            let matches_active_descendant = item_id == container.active_descendant_node_id;
            let matches_focus = item_id == tree.focus_node_id;
            pass_bool(matches_active_descendant || matches_focus, || {
                format!(
                    "active_item_is mismatch container={container:?} item={item:?} active_descendant={:?} focus={:?}",
                    container.active_descendant_node_id, tree.focus_node_id
                )
            })
        }
        UiPredicateV1::ActiveItemIsNone { container } => {
            let container = tree.select_best(container).map_err(fail)?;
            let focus_inside_container_item = container
                .node_id
                .zip(tree.focus_node_id)
                .is_some_and(|(container, focus)| {
                    focus != container && tree.node_is_descendant_of_or_self(focus, container)
                });
            pass_bool(
                container.active_descendant_node_id.is_none() && !focus_inside_container_item,
                || {
                    format!(
                        "active_item_is_none mismatch container={container:?} active_descendant={:?} focus={:?}",
                        container.active_descendant_node_id, tree.focus_node_id
                    )
                },
            )
        }
        UiPredicateV1::VisibleInWindow { target } => {
            let node = tree.select_best(target).map_err(fail)?;
            pass_bool(rects_intersect(node.bounds, tree.window_bounds), || {
                format!("expected target to be visible in window: {target:?}")
            })
        }
        UiPredicateV1::BoundsWithinWindow {
            target,
            padding_px,
            padding_insets_px,
            eps_px,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let pad = padding_px.max(0.0);
            let insets = padding_insets_px
                .unwrap_or_else(|| fret_diag_protocol::UiPaddingInsetsV1::uniform(0.0));
            let window = tree.window_bounds;
            let left = window.origin.x.0 + pad + insets.left_px.max(0.0);
            let top = window.origin.y.0 + pad + insets.top_px.max(0.0);
            let right = window.origin.x.0 + window.size.width.0 - pad - insets.right_px.max(0.0);
            let bottom = window.origin.y.0 + window.size.height.0 - pad - insets.bottom_px.max(0.0);
            pass_bool(
                rect_within(node.bounds, left, top, right, bottom, eps_px.max(0.0)),
                || {
                    format!(
                        "expected target bounds to be within window: {target:?} bounds={:?}",
                        node.bounds
                    )
                },
            )
        }
        UiPredicateV1::BoundsMinSize {
            target,
            min_w_px,
            min_h_px,
            eps_px,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let eps = eps_px.max(0.0);
            pass_bool(
                node.bounds.size.width.0 + eps >= min_w_px.max(0.0)
                    && node.bounds.size.height.0 + eps >= min_h_px.max(0.0),
                || {
                    format!(
                        "bounds_min_size mismatch target={target:?} bounds={:?}",
                        node.bounds
                    )
                },
            )
        }
        UiPredicateV1::BoundsMaxSize {
            target,
            max_w_px,
            max_h_px,
            eps_px,
        } => {
            let node = tree.select_best(target).map_err(fail)?;
            let eps = eps_px.max(0.0);
            pass_bool(
                node.bounds.size.width.0 <= max_w_px.max(0.0) + eps
                    && node.bounds.size.height.0 <= max_h_px.max(0.0) + eps,
                || {
                    format!(
                        "bounds_max_size mismatch target={target:?} bounds={:?}",
                        node.bounds
                    )
                },
            )
        }
        UiPredicateV1::BoundsApproxEqual { a, b, eps_px } => {
            let a_bounds = tree.bounds_for(a, BoundsSpace::Layout).map_err(fail)?;
            let b_bounds = tree.bounds_for(b, BoundsSpace::Layout).map_err(fail)?;
            pass_bool(rect_approx_eq(a_bounds, b_bounds, eps_px.max(0.0)), || {
                format!(
                    "bounds_approx_equal mismatch a={a:?} b={b:?} a_bounds={a_bounds:?} b_bounds={b_bounds:?}"
                )
            })
        }
        UiPredicateV1::BoundsCenterApproxEqual { a, b, eps_px } => {
            let a_bounds = tree.bounds_for(a, BoundsSpace::Layout).map_err(fail)?;
            let b_bounds = tree.bounds_for(b, BoundsSpace::Layout).map_err(fail)?;
            let eps = eps_px.max(0.0);
            pass_bool(
                (bounds_metric_value(a_bounds, UiBoundsMetricV1::CenterX)
                    - bounds_metric_value(b_bounds, UiBoundsMetricV1::CenterX))
                .abs()
                    <= eps
                    && (bounds_metric_value(a_bounds, UiBoundsMetricV1::CenterY)
                        - bounds_metric_value(b_bounds, UiBoundsMetricV1::CenterY))
                    .abs()
                        <= eps,
                || {
                    format!(
                        "bounds_center_approx_equal mismatch a={a:?} b={b:?} a_bounds={a_bounds:?} b_bounds={b_bounds:?}"
                    )
                },
            )
        }
        UiPredicateV1::BoundsMetricDelta {
            a,
            b,
            metric,
            comparison,
            value_px,
            eps_px,
        } => {
            let a_bounds = tree.bounds_for(a, BoundsSpace::Layout).map_err(fail)?;
            let b_bounds = tree.bounds_for(b, BoundsSpace::Layout).map_err(fail)?;
            let have =
                bounds_metric_value(a_bounds, *metric) - bounds_metric_value(b_bounds, *metric);
            pass_compare(have, *comparison, *value_px, eps_px.max(0.0), || {
                format!(
                    "bounds_metric_delta mismatch a={a:?} b={b:?} metric={metric:?} expected={value_px} actual={have}"
                )
            })
        }
        UiPredicateV1::BoundsNonOverlapping { a, b, eps_px } => {
            let a_bounds = tree.bounds_for(a, BoundsSpace::Layout).map_err(fail)?;
            let b_bounds = tree.bounds_for(b, BoundsSpace::Layout).map_err(fail)?;
            pass_bool(!rects_overlap(a_bounds, b_bounds, eps_px.max(0.0)), || {
                format!(
                    "bounds_non_overlapping mismatch a={a:?} b={b:?} a_bounds={a_bounds:?} b_bounds={b_bounds:?}"
                )
            })
        }
        UiPredicateV1::BoundsOverlapping { a, b, eps_px } => {
            let a_bounds = tree.bounds_for(a, BoundsSpace::Layout).map_err(fail)?;
            let b_bounds = tree.bounds_for(b, BoundsSpace::Layout).map_err(fail)?;
            pass_bool(rects_overlap(a_bounds, b_bounds, eps_px.max(0.0)), || {
                format!(
                    "bounds_overlapping mismatch a={a:?} b={b:?} a_bounds={a_bounds:?} b_bounds={b_bounds:?}"
                )
            })
        }
        UiPredicateV1::BoundsOverlappingX { a, b, eps_px } => {
            let a_bounds = tree.bounds_for(a, BoundsSpace::Layout).map_err(fail)?;
            let b_bounds = tree.bounds_for(b, BoundsSpace::Layout).map_err(fail)?;
            pass_bool(
                axis_overlap(
                    a_bounds.origin.x.0,
                    a_bounds.size.width.0,
                    b_bounds.origin.x.0,
                    b_bounds.size.width.0,
                    eps_px.max(0.0),
                ),
                || {
                    format!(
                        "bounds_overlapping_x mismatch a={a:?} b={b:?} a_bounds={a_bounds:?} b_bounds={b_bounds:?}"
                    )
                },
            )
        }
        UiPredicateV1::BoundsOverlappingY { a, b, eps_px } => {
            let a_bounds = tree.bounds_for(a, BoundsSpace::Layout).map_err(fail)?;
            let b_bounds = tree.bounds_for(b, BoundsSpace::Layout).map_err(fail)?;
            pass_bool(
                axis_overlap(
                    a_bounds.origin.y.0,
                    a_bounds.size.height.0,
                    b_bounds.origin.y.0,
                    b_bounds.size.height.0,
                    eps_px.max(0.0),
                ),
                || {
                    format!(
                        "bounds_overlapping_y mismatch a={a:?} b={b:?} a_bounds={a_bounds:?} b_bounds={b_bounds:?}"
                    )
                },
            )
        }
        _ => Err(failure(format!(
            "UiPredicateV1 variant is not supported by mechanism harness oracle: {predicate:?}"
        ))),
    }
}

fn observed_live_from_protocol(live: UiSemanticsLiveV1) -> ObservedSemanticsLive {
    match live {
        UiSemanticsLiveV1::Off => ObservedSemanticsLive::Off,
        UiSemanticsLiveV1::Polite => ObservedSemanticsLive::Polite,
        UiSemanticsLiveV1::Assertive => ObservedSemanticsLive::Assertive,
    }
}

fn observed_checked_state_from_protocol(
    state: UiSemanticsCheckedStateV1,
) -> ObservedSemanticsCheckedState {
    match state {
        UiSemanticsCheckedStateV1::False => ObservedSemanticsCheckedState::False,
        UiSemanticsCheckedStateV1::True => ObservedSemanticsCheckedState::True,
        UiSemanticsCheckedStateV1::Mixed => ObservedSemanticsCheckedState::Mixed,
    }
}

fn observed_pressed_state_from_protocol(
    state: UiSemanticsPressedStateV1,
) -> ObservedSemanticsPressedState {
    match state {
        UiSemanticsPressedStateV1::False => ObservedSemanticsPressedState::False,
        UiSemanticsPressedStateV1::True => ObservedSemanticsPressedState::True,
        UiSemanticsPressedStateV1::Mixed => ObservedSemanticsPressedState::Mixed,
    }
}

fn observed_action_from_protocol(action: UiSemanticsActionV1) -> ObservedSemanticsAction {
    match action {
        UiSemanticsActionV1::Focus => ObservedSemanticsAction::Focus,
        UiSemanticsActionV1::Invoke => ObservedSemanticsAction::Invoke,
        UiSemanticsActionV1::SetValue => ObservedSemanticsAction::SetValue,
        UiSemanticsActionV1::Decrement => ObservedSemanticsAction::Decrement,
        UiSemanticsActionV1::Increment => ObservedSemanticsAction::Increment,
        UiSemanticsActionV1::ScrollBy => ObservedSemanticsAction::ScrollBy,
        UiSemanticsActionV1::SetTextSelection => ObservedSemanticsAction::SetTextSelection,
    }
}

fn semantics_numeric_value(node: &ObservedNode, field: UiSemanticsNumericFieldV1) -> Option<f64> {
    let numeric = node.numeric?;
    match field {
        UiSemanticsNumericFieldV1::Value => numeric.value,
        UiSemanticsNumericFieldV1::Min => numeric.min,
        UiSemanticsNumericFieldV1::Max => numeric.max,
        UiSemanticsNumericFieldV1::Step => numeric.step,
        UiSemanticsNumericFieldV1::Jump => numeric.jump,
    }
}

fn semantics_scroll_value(node: &ObservedNode, field: UiSemanticsScrollFieldV1) -> Option<f64> {
    let scroll = node.scroll?;
    match field {
        UiSemanticsScrollFieldV1::X => scroll.x,
        UiSemanticsScrollFieldV1::XMin => scroll.x_min,
        UiSemanticsScrollFieldV1::XMax => scroll.x_max,
        UiSemanticsScrollFieldV1::Y => scroll.y,
        UiSemanticsScrollFieldV1::YMin => scroll.y_min,
        UiSemanticsScrollFieldV1::YMax => scroll.y_max,
    }
}

fn hit_matches(node: &ObservedNode, hit_node_id: Option<u64>, hit_test_id: Option<&str>) -> bool {
    node.node_id.is_some_and(|id| Some(id) == hit_node_id)
        || node
            .test_id
            .as_deref()
            .is_some_and(|id| Some(id) == hit_test_id)
}

fn selector_matches_node_id(tree: &ObservedTree, selector: &UiSelectorV1, node_id: u64) -> bool {
    tree.select(selector)
        .into_iter()
        .any(|node| node.node_id == Some(node_id))
}

pub fn bounds_metric_value(bounds: Rect, metric: UiBoundsMetricV1) -> f32 {
    let left = bounds.origin.x.0;
    let top = bounds.origin.y.0;
    let width = bounds.size.width.0.max(0.0);
    let height = bounds.size.height.0.max(0.0);
    match metric {
        UiBoundsMetricV1::Left => left,
        UiBoundsMetricV1::Top => top,
        UiBoundsMetricV1::Right => left + width,
        UiBoundsMetricV1::Bottom => top + height,
        UiBoundsMetricV1::Width => width,
        UiBoundsMetricV1::Height => height,
        UiBoundsMetricV1::CenterX => left + width * 0.5,
        UiBoundsMetricV1::CenterY => top + height * 0.5,
    }
}

fn pass_compare(
    have: f32,
    comparison: UiComparisonV1,
    want: f32,
    eps: f32,
    message: impl FnOnce() -> String,
) -> Result<PredicatePass, PredicateFailure> {
    let passed = match comparison {
        UiComparisonV1::Eq => (have - want).abs() <= eps,
        UiComparisonV1::Ge => have + eps >= want,
        UiComparisonV1::Le => have <= want + eps,
    };
    pass_bool(passed, message)
}

fn pass_bool(
    passed: bool,
    message: impl FnOnce() -> String,
) -> Result<PredicatePass, PredicateFailure> {
    if passed {
        Ok(PredicatePass)
    } else {
        Err(failure(message()))
    }
}

fn fail(err: impl std::fmt::Display) -> PredicateFailure {
    failure(err.to_string())
}

fn failure(message: impl Into<String>) -> PredicateFailure {
    PredicateFailure {
        message: message.into(),
    }
}

fn rect_approx_eq(a: Rect, b: Rect, eps: f32) -> bool {
    (a.origin.x.0 - b.origin.x.0).abs() <= eps
        && (a.origin.y.0 - b.origin.y.0).abs() <= eps
        && (a.size.width.0 - b.size.width.0).abs() <= eps
        && (a.size.height.0 - b.size.height.0).abs() <= eps
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
    axis_overlap(
        a.origin.x.0,
        a.size.width.0,
        b.origin.x.0,
        b.size.width.0,
        0.0,
    ) && axis_overlap(
        a.origin.y.0,
        a.size.height.0,
        b.origin.y.0,
        b.size.height.0,
        0.0,
    )
}

fn rects_overlap(a: Rect, b: Rect, eps: f32) -> bool {
    axis_overlap(
        a.origin.x.0,
        a.size.width.0,
        b.origin.x.0,
        b.size.width.0,
        eps,
    ) && axis_overlap(
        a.origin.y.0,
        a.size.height.0,
        b.origin.y.0,
        b.size.height.0,
        eps,
    )
}

fn axis_overlap(a0: f32, aw: f32, b0: f32, bw: f32, eps: f32) -> bool {
    let a1 = a0 + aw.max(0.0);
    let b1 = b0 + bw.max(0.0);
    (a1.min(b1) - a0.max(b0)).max(0.0) > eps
}

fn rect_within(bounds: Rect, left: f32, top: f32, right: f32, bottom: f32, eps: f32) -> bool {
    let node_left = bounds.origin.x.0;
    let node_top = bounds.origin.y.0;
    let node_right = bounds.origin.x.0 + bounds.size.width.0;
    let node_bottom = bounds.origin.y.0 + bounds.size.height.0;
    node_left >= left - eps
        && node_top >= top - eps
        && node_right <= right + eps
        && node_bottom <= bottom + eps
}
