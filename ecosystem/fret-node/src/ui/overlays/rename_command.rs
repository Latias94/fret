use fret_core::KeyCode;
use fret_runtime::CommandId;

use crate::Graph;
use crate::ops::GraphTransaction;

use super::group_rename::NodeGraphOverlayState;
use super::rename_policy::{
    RenameOverlaySessionKey, active_rename_session, build_rename_commit_transaction,
    clear_rename_sessions,
};

const CMD_RENAME_SUBMIT_PREFIX: &str = "fret_node.rename.submit:";
const CMD_RENAME_CANCEL_PREFIX: &str = "fret_node.rename.cancel:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenameTextCommand {
    Submit { session: RenameOverlaySessionKey },
    Cancel { session: RenameOverlaySessionKey },
}

#[derive(Debug, Clone)]
pub(super) enum RenameCommandOutcome {
    NotHandled,
    Handled,
    Commit(GraphTransaction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenameHostKeyDecision {
    Close,
    CommitAndClose,
    Ignore,
}

pub(super) fn decide_rename_host_key(key: KeyCode) -> RenameHostKeyDecision {
    match key {
        KeyCode::Escape => RenameHostKeyDecision::Close,
        KeyCode::Enter | KeyCode::NumpadEnter => RenameHostKeyDecision::CommitAndClose,
        _ => RenameHostKeyDecision::Ignore,
    }
}

pub(super) fn rename_submit_command(session: RenameOverlaySessionKey) -> CommandId {
    rename_command(CMD_RENAME_SUBMIT_PREFIX, session)
}

pub(super) fn rename_cancel_command(session: RenameOverlaySessionKey) -> CommandId {
    rename_command(CMD_RENAME_CANCEL_PREFIX, session)
}

pub(super) fn parse_rename_text_command(command: &CommandId) -> Option<RenameTextCommand> {
    let id = command.as_str();
    if let Some(rest) = id.strip_prefix(CMD_RENAME_SUBMIT_PREFIX) {
        return Some(RenameTextCommand::Submit {
            session: parse_rename_session_key(rest)?,
        });
    }
    if let Some(rest) = id.strip_prefix(CMD_RENAME_CANCEL_PREFIX) {
        return Some(RenameTextCommand::Cancel {
            session: parse_rename_session_key(rest)?,
        });
    }
    None
}

pub(super) fn apply_rename_text_command(
    graph: &Graph,
    state: &mut NodeGraphOverlayState,
    rename_text: &str,
    command: RenameTextCommand,
) -> RenameCommandOutcome {
    match command {
        RenameTextCommand::Cancel { session } => close_matching_session(state, session),
        RenameTextCommand::Submit { session } => {
            let Some(active) = active_rename_session(state) else {
                return RenameCommandOutcome::NotHandled;
            };
            if active.key() != session {
                return RenameCommandOutcome::NotHandled;
            }

            let tx = build_rename_commit_transaction(graph, &active, rename_text);
            clear_rename_sessions(state);
            tx.map_or(RenameCommandOutcome::Handled, RenameCommandOutcome::Commit)
        }
    }
}

pub(super) fn apply_rename_host_key_decision(
    graph: &Graph,
    state: &mut NodeGraphOverlayState,
    rename_text: &str,
    decision: RenameHostKeyDecision,
) -> RenameCommandOutcome {
    match decision {
        RenameHostKeyDecision::Ignore => RenameCommandOutcome::NotHandled,
        RenameHostKeyDecision::Close => {
            if active_rename_session(state).is_none() {
                return RenameCommandOutcome::NotHandled;
            }
            clear_rename_sessions(state);
            RenameCommandOutcome::Handled
        }
        RenameHostKeyDecision::CommitAndClose => {
            let Some(active) = active_rename_session(state) else {
                return RenameCommandOutcome::NotHandled;
            };
            let tx = build_rename_commit_transaction(graph, &active, rename_text);
            clear_rename_sessions(state);
            tx.map_or(RenameCommandOutcome::Handled, RenameCommandOutcome::Commit)
        }
    }
}

fn close_matching_session(
    state: &mut NodeGraphOverlayState,
    session: RenameOverlaySessionKey,
) -> RenameCommandOutcome {
    let Some(active) = active_rename_session(state) else {
        return RenameCommandOutcome::NotHandled;
    };
    if active.key() != session {
        return RenameCommandOutcome::NotHandled;
    }
    clear_rename_sessions(state);
    RenameCommandOutcome::Handled
}

fn rename_command(prefix: &str, session: RenameOverlaySessionKey) -> CommandId {
    let entity = match session {
        RenameOverlaySessionKey::Group(group) => format!("group:{}", group.0),
        RenameOverlaySessionKey::Symbol(symbol) => format!("symbol:{}", symbol.0),
    };
    CommandId::new(format!("{prefix}{entity}"))
}

fn parse_rename_session_key(value: &str) -> Option<RenameOverlaySessionKey> {
    let (kind, id) = value.split_once(':')?;
    let id = uuid::Uuid::parse_str(id).ok()?;
    match kind {
        "group" => Some(RenameOverlaySessionKey::Group(crate::core::GroupId(id))),
        "symbol" => Some(RenameOverlaySessionKey::Symbol(crate::core::SymbolId(id))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{KeyCode, Point, Px};
    use fret_runtime::CommandId;

    use crate::Graph;
    use crate::core::{
        CanvasPoint, CanvasRect, CanvasSize, GraphId, Group, GroupId, Symbol, SymbolId,
    };
    use crate::ops::GraphOp;
    use crate::ui::overlays::group_rename::{
        GroupRenameOverlay, NodeGraphOverlayState, SymbolRenameOverlay,
    };
    use crate::ui::overlays::rename_policy::RenameOverlaySessionKey;

    use super::{
        RenameCommandOutcome, RenameHostKeyDecision, RenameTextCommand,
        apply_rename_host_key_decision, apply_rename_text_command, decide_rename_host_key,
        parse_rename_text_command, rename_cancel_command, rename_submit_command,
    };

    fn rename_point() -> Point {
        Point::new(Px(10.0), Px(20.0))
    }

    fn graph_with_group_and_symbol() -> (Graph, GroupId, SymbolId) {
        let group = GroupId::from_u128(0x11111111111111111111111111111111);
        let symbol = SymbolId::from_u128(0x22222222222222222222222222222222);
        let mut graph = Graph::new(GraphId::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa));
        graph.groups.insert(
            group,
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
            symbol,
            Symbol {
                name: "Symbol A".to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );
        (graph, group, symbol)
    }

    #[test]
    fn rename_text_command_protocol_roundtrips_and_rejects_malformed_commands() {
        let (_, group, symbol) = graph_with_group_and_symbol();

        assert_eq!(
            parse_rename_text_command(&rename_submit_command(RenameOverlaySessionKey::Group(
                group
            ))),
            Some(RenameTextCommand::Submit {
                session: RenameOverlaySessionKey::Group(group)
            })
        );
        assert_eq!(
            parse_rename_text_command(&rename_cancel_command(RenameOverlaySessionKey::Symbol(
                symbol
            ))),
            Some(RenameTextCommand::Cancel {
                session: RenameOverlaySessionKey::Symbol(symbol)
            })
        );

        for command in [
            CommandId::from("fret_node.rename.submit:not-a-uuid"),
            CommandId::from("fret_node.rename.cancel:not-a-uuid"),
            CommandId::from("fret_node.rename.submit:node:22222222-2222-2222-2222-222222222222"),
            CommandId::from("fret_node.rename.unknown:group:11111111-1111-1111-1111-111111111111"),
        ] {
            assert_eq!(parse_rename_text_command(&command), None);
        }
    }

    #[test]
    fn rename_text_command_applies_submit_and_cancel_without_retained_host() {
        let (graph, group, symbol) = graph_with_group_and_symbol();
        let mut state = NodeGraphOverlayState {
            group_rename: Some(GroupRenameOverlay {
                group,
                invoked_at_window: rename_point(),
            }),
            symbol_rename: None,
        };

        let stale = apply_rename_text_command(
            &graph,
            &mut state,
            "Group B",
            RenameTextCommand::Submit {
                session: RenameOverlaySessionKey::Symbol(symbol),
            },
        );
        assert!(matches!(stale, RenameCommandOutcome::NotHandled));
        assert!(state.group_rename.is_some());

        let commit = apply_rename_text_command(
            &graph,
            &mut state,
            "Group B",
            RenameTextCommand::Submit {
                session: RenameOverlaySessionKey::Group(group),
            },
        );
        let RenameCommandOutcome::Commit(tx) = commit else {
            panic!("rename submit should commit the active group rename");
        };
        assert_eq!(tx.label.as_deref(), Some("Rename Group"));
        assert!(matches!(
            tx.ops.as_slice(),
            [GraphOp::SetGroupTitle { id, to, .. }] if id == &group && to == "Group B"
        ));
        assert!(state.group_rename.is_none());
        assert!(state.symbol_rename.is_none());

        state.symbol_rename = Some(SymbolRenameOverlay {
            symbol,
            invoked_at_window: rename_point(),
        });
        let cancel = apply_rename_text_command(
            &graph,
            &mut state,
            "Ignored",
            RenameTextCommand::Cancel {
                session: RenameOverlaySessionKey::Symbol(symbol),
            },
        );
        assert!(matches!(cancel, RenameCommandOutcome::Handled));
        assert!(state.group_rename.is_none());
        assert!(state.symbol_rename.is_none());
    }

    #[test]
    fn rename_host_key_decision_applies_submit_cancel_and_ignore_without_retained_host() {
        let (graph, group, _) = graph_with_group_and_symbol();
        let mut state = NodeGraphOverlayState {
            group_rename: Some(GroupRenameOverlay {
                group,
                invoked_at_window: rename_point(),
            }),
            symbol_rename: None,
        };

        assert_eq!(
            decide_rename_host_key(KeyCode::Tab),
            RenameHostKeyDecision::Ignore
        );
        let ignore = apply_rename_host_key_decision(
            &graph,
            &mut state,
            "Group B",
            decide_rename_host_key(KeyCode::Tab),
        );
        assert!(matches!(ignore, RenameCommandOutcome::NotHandled));
        assert!(state.group_rename.is_some());

        let commit = apply_rename_host_key_decision(
            &graph,
            &mut state,
            "Group B",
            decide_rename_host_key(KeyCode::Enter),
        );
        assert!(matches!(commit, RenameCommandOutcome::Commit(_)));
        assert!(state.group_rename.is_none());
        assert!(state.symbol_rename.is_none());
    }
}
