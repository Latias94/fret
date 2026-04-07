use fret_core::{Point, Px, Rect, Size};

use crate::Graph;
use crate::core::{GroupId, SymbolId};
use crate::ops::{GraphOp, GraphTransaction};
use crate::ui::screen_space_placement::clamp_rect_to_bounds;
use crate::ui::style::NodeGraphStyle;

use super::group_rename::{GroupRenameOverlay, NodeGraphOverlayState, SymbolRenameOverlay};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenameOverlaySessionKey {
    Group(GroupId),
    Symbol(SymbolId),
}

#[derive(Debug, Clone)]
pub(super) enum RenameOverlaySession {
    Group(GroupRenameOverlay),
    Symbol(SymbolRenameOverlay),
}

impl RenameOverlaySession {
    pub(super) fn key(&self) -> RenameOverlaySessionKey {
        match self {
            Self::Group(rename) => RenameOverlaySessionKey::Group(rename.group),
            Self::Symbol(rename) => RenameOverlaySessionKey::Symbol(rename.symbol),
        }
    }

    pub(super) fn invoked_at_window(&self) -> Point {
        match self {
            Self::Group(rename) => rename.invoked_at_window,
            Self::Symbol(rename) => rename.invoked_at_window,
        }
    }
}

pub(super) fn active_rename_session(state: &NodeGraphOverlayState) -> Option<RenameOverlaySession> {
    if let Some(rename) = state.group_rename.clone() {
        return Some(RenameOverlaySession::Group(rename));
    }
    state
        .symbol_rename
        .clone()
        .map(RenameOverlaySession::Symbol)
}

pub(super) fn clear_rename_sessions(state: &mut NodeGraphOverlayState) {
    state.group_rename = None;
    state.symbol_rename = None;
}

pub(super) fn rename_overlay_rect_at(
    style: &NodeGraphStyle,
    desired_origin: Point,
    bounds: Rect,
) -> Rect {
    clamp_rect_to_bounds(
        Rect::new(desired_origin, rename_overlay_size_at(style)),
        bounds,
    )
}

pub(super) fn rename_overlay_should_cancel_on_focus_loss(
    child: Option<fret_core::NodeId>,
    focus: Option<fret_core::NodeId>,
    just_opened: bool,
) -> bool {
    !just_opened && child.is_some_and(|child| focus != Some(child))
}

pub(super) fn rename_session_seed_text(graph: &Graph, session: &RenameOverlaySession) -> String {
    match session {
        RenameOverlaySession::Group(rename) => graph
            .groups
            .get(&rename.group)
            .map(|group| group.title.clone())
            .unwrap_or_default(),
        RenameOverlaySession::Symbol(rename) => graph
            .symbols
            .get(&rename.symbol)
            .map(|symbol| symbol.name.clone())
            .unwrap_or_default(),
    }
}

pub(super) fn build_rename_commit_transaction(
    graph: &Graph,
    session: &RenameOverlaySession,
    to: &str,
) -> Option<GraphTransaction> {
    match session {
        RenameOverlaySession::Group(rename) => {
            let group = graph.groups.get(&rename.group)?;
            if group.title == to {
                return None;
            }
            Some(GraphTransaction {
                label: Some("Rename Group".to_string()),
                ops: vec![GraphOp::SetGroupTitle {
                    id: rename.group,
                    from: group.title.clone(),
                    to: to.to_string(),
                }],
            })
        }
        RenameOverlaySession::Symbol(rename) => {
            let symbol = graph.symbols.get(&rename.symbol)?;
            if symbol.name == to {
                return None;
            }
            Some(GraphTransaction {
                label: Some("Rename Symbol".to_string()),
                ops: vec![GraphOp::SetSymbolName {
                    id: rename.symbol,
                    from: symbol.name.clone(),
                    to: to.to_string(),
                }],
            })
        }
    }
}

fn rename_overlay_size_at(style: &NodeGraphStyle) -> Size {
    let w = style.paint.context_menu_width.max(40.0);
    let h = (style.paint.context_menu_item_height.max(20.0)
        + 2.0 * style.paint.context_menu_padding)
        .max(24.0);
    Size::new(Px(w), Px(h))
}

#[cfg(test)]
mod tests {
    use super::{
        RenameOverlaySession, RenameOverlaySessionKey, active_rename_session,
        build_rename_commit_transaction, clear_rename_sessions, rename_session_seed_text,
    };
    use crate::core::{GraphId, Group, GroupId, Symbol, SymbolId};
    use crate::ui::{GroupRenameOverlay, NodeGraphOverlayState, SymbolRenameOverlay};
    use crate::{Graph, core::CanvasPoint, core::CanvasRect, core::CanvasSize};
    use fret_core::{Point, Px};

    #[test]
    fn active_session_prefers_group_overlay_when_both_are_present() {
        let group = GroupId::new();
        let symbol = SymbolId::new();
        let state = NodeGraphOverlayState {
            group_rename: Some(GroupRenameOverlay {
                group,
                invoked_at_window: Point::new(Px(10.0), Px(20.0)),
            }),
            symbol_rename: Some(SymbolRenameOverlay {
                symbol,
                invoked_at_window: Point::new(Px(30.0), Px(40.0)),
            }),
        };

        let session = active_rename_session(&state).expect("rename session");
        assert_eq!(session.key(), RenameOverlaySessionKey::Group(group));
    }

    #[test]
    fn clear_sessions_drops_group_and_symbol_rename_state() {
        let mut state = NodeGraphOverlayState {
            group_rename: Some(GroupRenameOverlay {
                group: GroupId::new(),
                invoked_at_window: Point::new(Px(10.0), Px(20.0)),
            }),
            symbol_rename: Some(SymbolRenameOverlay {
                symbol: SymbolId::new(),
                invoked_at_window: Point::new(Px(30.0), Px(40.0)),
            }),
        };

        clear_rename_sessions(&mut state);
        assert!(state.group_rename.is_none());
        assert!(state.symbol_rename.is_none());
    }

    #[test]
    fn build_commit_transaction_uses_active_entity_specific_payload() {
        let group_id = GroupId::new();
        let symbol_id = SymbolId::new();
        let mut graph = Graph::new(GraphId::new());
        graph.groups.insert(
            group_id,
            Group {
                title: "Group A".to_string(),
                rect: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 100.0,
                        height: 40.0,
                    },
                },
                color: None,
            },
        );
        graph.symbols.insert(
            symbol_id,
            Symbol {
                name: "Symbol A".to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );

        let group_session = RenameOverlaySession::Group(GroupRenameOverlay {
            group: group_id,
            invoked_at_window: Point::new(Px(10.0), Px(20.0)),
        });
        let symbol_session = RenameOverlaySession::Symbol(SymbolRenameOverlay {
            symbol: symbol_id,
            invoked_at_window: Point::new(Px(30.0), Px(40.0)),
        });

        assert_eq!(rename_session_seed_text(&graph, &group_session), "Group A");
        assert_eq!(
            rename_session_seed_text(&graph, &symbol_session),
            "Symbol A"
        );

        let group_tx =
            build_rename_commit_transaction(&graph, &group_session, "Group B").expect("group tx");
        assert_eq!(group_tx.label.as_deref(), Some("Rename Group"));

        let symbol_tx = build_rename_commit_transaction(&graph, &symbol_session, "Symbol B")
            .expect("symbol tx");
        assert_eq!(symbol_tx.label.as_deref(), Some("Rename Symbol"));
    }

    #[test]
    fn build_commit_transaction_skips_unchanged_or_missing_targets() {
        let group_id = GroupId::new();
        let mut graph = Graph::new(GraphId::new());
        graph.groups.insert(
            group_id,
            Group {
                title: "Group A".to_string(),
                rect: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 100.0,
                        height: 40.0,
                    },
                },
                color: None,
            },
        );

        let unchanged = RenameOverlaySession::Group(GroupRenameOverlay {
            group: group_id,
            invoked_at_window: Point::new(Px(10.0), Px(20.0)),
        });
        let missing = RenameOverlaySession::Symbol(SymbolRenameOverlay {
            symbol: SymbolId::new(),
            invoked_at_window: Point::new(Px(30.0), Px(40.0)),
        });

        assert!(build_rename_commit_transaction(&graph, &unchanged, "Group A").is_none());
        assert!(build_rename_commit_transaction(&graph, &missing, "Anything").is_none());
    }
}
