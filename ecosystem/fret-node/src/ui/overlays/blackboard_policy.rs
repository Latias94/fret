use std::collections::BTreeMap;

use fret_core::{Point, Rect};

use crate::core::{
    CanvasPoint, CanvasSize, Graph, Node, NodeId, NodeKindKey, SYMBOL_REF_NODE_KIND, Symbol,
    SymbolId, is_symbol_ref_node, symbol_ref_node_data, symbol_ref_target_symbol_id,
};
use crate::io::NodeGraphViewState;
use crate::ops::{GraphOp, GraphOpBuilderExt as _, GraphTransaction};

use super::SymbolRenameOverlay;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlackboardAction {
    AddSymbol,
    InsertRef { symbol: SymbolId },
    Rename { symbol: SymbolId },
    Delete { symbol: SymbolId },
}

#[derive(Debug, Clone)]
pub(super) enum BlackboardActionPlan {
    Transaction(GraphTransaction),
    OpenSymbolRename(SymbolRenameOverlay),
}

pub(super) fn blackboard_action_a11y_label(action: BlackboardAction) -> &'static str {
    match action {
        BlackboardAction::AddSymbol => "Add symbol",
        BlackboardAction::InsertRef { .. } => "Insert symbol reference",
        BlackboardAction::Rename { .. } => "Rename symbol",
        BlackboardAction::Delete { .. } => "Delete symbol",
    }
}

pub(super) fn blackboard_action_button_label(action: BlackboardAction) -> &'static str {
    match action {
        BlackboardAction::AddSymbol => "+",
        BlackboardAction::InsertRef { .. } => "R",
        BlackboardAction::Rename { .. } => "E",
        BlackboardAction::Delete { .. } => "X",
    }
}

pub(super) fn blackboard_actions_in_order(
    symbols: &BTreeMap<SymbolId, Symbol>,
) -> Vec<BlackboardAction> {
    let mut out = Vec::with_capacity(1 + symbols.len() * 3);
    out.push(BlackboardAction::AddSymbol);
    for symbol in symbols.keys().copied() {
        out.push(BlackboardAction::InsertRef { symbol });
        out.push(BlackboardAction::Rename { symbol });
        out.push(BlackboardAction::Delete { symbol });
    }
    out
}

pub(super) fn next_blackboard_action(
    current: Option<BlackboardAction>,
    delta: i32,
    items: &[BlackboardAction],
) -> Option<BlackboardAction> {
    if items.is_empty() {
        return None;
    }

    let len = items.len() as i32;
    let idx0 = current
        .and_then(|action| items.iter().position(|candidate| *candidate == action))
        .unwrap_or(0) as i32;
    let mut next = idx0 + delta;
    next = ((next % len) + len) % len;
    Some(items[next as usize])
}

pub(super) fn plan_blackboard_action(
    graph: &Graph,
    view_state: &NodeGraphViewState,
    bounds: Rect,
    action: BlackboardAction,
    invoked_at_window: Point,
) -> Option<BlackboardActionPlan> {
    match action {
        BlackboardAction::AddSymbol => {
            let id = SymbolId::new();
            let symbol = Symbol {
                name: default_symbol_name(&graph.symbols),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            };
            Some(BlackboardActionPlan::Transaction(GraphTransaction {
                label: Some("Add Symbol".to_string()),
                ops: vec![GraphOp::AddSymbol { id, symbol }],
            }))
        }
        BlackboardAction::InsertRef { symbol } => {
            let center = viewport_center_canvas_point(view_state, bounds);
            let node = Node {
                kind: NodeKindKey::new(SYMBOL_REF_NODE_KIND),
                kind_version: 1,
                pos: center,
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 140.0,
                    height: 40.0,
                }),
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: symbol_ref_node_data(symbol),
            };
            let id = NodeId::new();
            Some(BlackboardActionPlan::Transaction(GraphTransaction {
                label: Some("Insert Symbol Ref".to_string()),
                ops: vec![GraphOp::AddNode { id, node }],
            }))
        }
        BlackboardAction::Rename { symbol } => Some(BlackboardActionPlan::OpenSymbolRename(
            SymbolRenameOverlay {
                symbol,
                invoked_at_window,
            },
        )),
        BlackboardAction::Delete { symbol } => {
            let symbol_value = graph.symbols.get(&symbol).cloned()?;

            let mut ref_nodes: Vec<NodeId> = graph
                .nodes
                .iter()
                .filter_map(|(node_id, node)| {
                    if !is_symbol_ref_node(node) {
                        return None;
                    }
                    let Ok(Some(target)) = symbol_ref_target_symbol_id(*node_id, node) else {
                        return None;
                    };
                    (target == symbol).then_some(*node_id)
                })
                .collect();
            ref_nodes.sort();

            let mut ops = Vec::new();
            for node_id in ref_nodes {
                if let Some(op) = graph.build_remove_node_op(node_id) {
                    ops.push(op);
                }
            }
            ops.push(GraphOp::RemoveSymbol {
                id: symbol,
                symbol: symbol_value,
            });

            Some(BlackboardActionPlan::Transaction(GraphTransaction {
                label: Some("Delete Symbol".to_string()),
                ops,
            }))
        }
    }
}

fn default_symbol_name(symbols: &BTreeMap<SymbolId, Symbol>) -> String {
    if !symbols.values().any(|symbol| symbol.name == "Symbol") {
        return "Symbol".to_string();
    }

    for i in 2..=9999 {
        let candidate = format!("Symbol {i}");
        if !symbols.values().any(|symbol| symbol.name == candidate) {
            return candidate;
        }
    }

    "Symbol".to_string()
}

fn viewport_center_canvas_point(view_state: &NodeGraphViewState, bounds: Rect) -> CanvasPoint {
    let zoom = if view_state.zoom.is_finite() && view_state.zoom > 0.0 {
        view_state.zoom
    } else {
        1.0
    };

    CanvasPoint {
        x: 0.5 * bounds.size.width.0 / zoom - view_state.pan.x,
        y: 0.5 * bounds.size.height.0 / zoom - view_state.pan.y,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        BlackboardAction, BlackboardActionPlan, blackboard_action_a11y_label,
        blackboard_action_button_label, blackboard_actions_in_order, next_blackboard_action,
        plan_blackboard_action,
    };
    use crate::core::{Graph, GraphId, Symbol, SymbolId};
    use crate::io::NodeGraphViewState;
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn blackboard_action_roster_and_navigation_order_stay_stable() {
        let symbol_a = SymbolId::new();
        let symbol_b = SymbolId::new();
        let symbols = BTreeMap::from([
            (
                symbol_a,
                Symbol {
                    name: "A".to_string(),
                    ty: None,
                    default_value: None,
                    meta: serde_json::Value::Null,
                },
            ),
            (
                symbol_b,
                Symbol {
                    name: "B".to_string(),
                    ty: None,
                    default_value: None,
                    meta: serde_json::Value::Null,
                },
            ),
        ]);

        let items = blackboard_actions_in_order(&symbols);
        let sorted_symbols: Vec<_> = symbols.keys().copied().collect();
        assert_eq!(items[0], BlackboardAction::AddSymbol);
        assert_eq!(
            items[1],
            BlackboardAction::InsertRef {
                symbol: sorted_symbols[0]
            }
        );
        assert_eq!(
            items[2],
            BlackboardAction::Rename {
                symbol: sorted_symbols[0]
            }
        );
        assert_eq!(
            items[3],
            BlackboardAction::Delete {
                symbol: sorted_symbols[0]
            }
        );
        assert_eq!(
            next_blackboard_action(Some(BlackboardAction::AddSymbol), -1, &items),
            Some(BlackboardAction::Delete {
                symbol: sorted_symbols[1]
            })
        );
    }

    #[test]
    fn blackboard_action_planning_keeps_labels_and_default_names_consistent() {
        let symbol = SymbolId::new();
        let graph = Graph::new(GraphId::new());
        let view_state = NodeGraphViewState::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let add = plan_blackboard_action(
            &graph,
            &view_state,
            bounds,
            BlackboardAction::AddSymbol,
            Point::new(Px(10.0), Px(20.0)),
        )
        .expect("add plan");
        let rename = plan_blackboard_action(
            &graph,
            &view_state,
            bounds,
            BlackboardAction::Rename { symbol },
            Point::new(Px(30.0), Px(40.0)),
        )
        .expect("rename plan");

        match add {
            BlackboardActionPlan::Transaction(tx) => {
                assert_eq!(tx.label.as_deref(), Some("Add Symbol"));
            }
            other => panic!("unexpected add plan: {other:?}"),
        }

        match rename {
            BlackboardActionPlan::OpenSymbolRename(overlay) => {
                assert_eq!(overlay.symbol, symbol);
            }
            other => panic!("unexpected rename plan: {other:?}"),
        }

        assert_eq!(
            blackboard_action_a11y_label(BlackboardAction::Delete { symbol }),
            "Delete symbol"
        );
        assert_eq!(
            blackboard_action_button_label(BlackboardAction::InsertRef { symbol }),
            "R"
        );
    }
}
