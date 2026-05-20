use fret_core::{NodeId, Rect};

use crate::ui::style::NodeGraphStyle;

use super::rename_host_layout::{RenameHostLayoutPlan, plan_rename_host_layout};
use super::rename_policy::{RenameOverlaySession, RenameOverlaySessionKey};

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RenameHostLifecyclePlan {
    Hidden {
        focus_restore: Option<NodeId>,
    },
    CancelActiveSession {
        focus_restore: Option<NodeId>,
    },
    Active {
        rect: Rect,
        session_key: RenameOverlaySessionKey,
        just_opened: bool,
        seed_text: Option<String>,
        focus_request: Option<NodeId>,
    },
}

pub(super) fn plan_rename_host_lifecycle(
    style: &NodeGraphStyle,
    bounds: Rect,
    session: Option<&RenameOverlaySession>,
    child: Option<NodeId>,
    focus: Option<NodeId>,
    restore_focus: Option<NodeId>,
    last_opened_session: Option<RenameOverlaySessionKey>,
    seed_text_for_session: impl FnOnce(&RenameOverlaySession) -> String,
) -> RenameHostLifecyclePlan {
    match plan_rename_host_layout(style, bounds, session, child, focus, last_opened_session) {
        RenameHostLayoutPlan::Hidden => RenameHostLifecyclePlan::Hidden {
            focus_restore: focus_restore_for_hidden_child(child, focus, restore_focus),
        },
        RenameHostLayoutPlan::CancelActiveSession => RenameHostLifecyclePlan::CancelActiveSession {
            focus_restore: focus_restore_for_hidden_child(child, focus, restore_focus),
        },
        RenameHostLayoutPlan::Active {
            rect,
            session_key,
            just_opened,
        } => {
            let session = session.expect("active rename session for active rename lifecycle plan");
            RenameHostLifecyclePlan::Active {
                rect,
                session_key,
                just_opened,
                seed_text: just_opened.then(|| seed_text_for_session(session)),
                focus_request: just_opened.then_some(child).flatten(),
            }
        }
    }
}

fn focus_restore_for_hidden_child(
    child: Option<NodeId>,
    focus: Option<NodeId>,
    restore_focus: Option<NodeId>,
) -> Option<NodeId> {
    if child.is_some() && child == focus {
        restore_focus
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{NodeId, Point, Px, Rect, Size};
    use slotmap::KeyData;

    use crate::core::{
        CanvasPoint, CanvasRect, CanvasSize, Graph, GraphId, Group, GroupId, Symbol, SymbolId,
    };
    use crate::ui::overlays::group_rename::{GroupRenameOverlay, SymbolRenameOverlay};
    use crate::ui::overlays::rename_lifecycle::{
        RenameHostLifecyclePlan, plan_rename_host_lifecycle,
    };
    use crate::ui::overlays::rename_policy::{RenameOverlaySession, rename_session_seed_text};
    use crate::ui::style::NodeGraphStyle;

    fn node(raw: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(raw))
    }

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        )
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
    fn rename_lifecycle_seeds_and_focuses_new_group_session() {
        let (graph, group, _) = graph_with_group_and_symbol();
        let session = RenameOverlaySession::Group(GroupRenameOverlay {
            group,
            invoked_at_window: Point::new(Px(100.0), Px(120.0)),
        });
        let child = node(1);
        let canvas = node(2);

        let plan = plan_rename_host_lifecycle(
            &NodeGraphStyle::default(),
            bounds(),
            Some(&session),
            Some(child),
            None,
            Some(canvas),
            None,
            |session| rename_session_seed_text(&graph, session),
        );

        match plan {
            RenameHostLifecyclePlan::Active {
                session_key,
                just_opened,
                seed_text,
                focus_request,
                ..
            } => {
                assert_eq!(session_key, session.key());
                assert!(just_opened);
                assert_eq!(seed_text.as_deref(), Some("Group A"));
                assert_eq!(focus_request, Some(child));
            }
            other => panic!("unexpected lifecycle plan: {other:?}"),
        }
    }

    #[test]
    fn rename_lifecycle_does_not_reseed_or_refocus_existing_session() {
        let (graph, group, _) = graph_with_group_and_symbol();
        let session = RenameOverlaySession::Group(GroupRenameOverlay {
            group,
            invoked_at_window: Point::new(Px(100.0), Px(120.0)),
        });
        let child = node(1);

        let plan = plan_rename_host_lifecycle(
            &NodeGraphStyle::default(),
            bounds(),
            Some(&session),
            Some(child),
            Some(child),
            Some(node(2)),
            Some(session.key()),
            |session| rename_session_seed_text(&graph, session),
        );

        match plan {
            RenameHostLifecyclePlan::Active {
                just_opened,
                seed_text,
                focus_request,
                ..
            } => {
                assert!(!just_opened);
                assert_eq!(seed_text, None);
                assert_eq!(focus_request, None);
            }
            other => panic!("unexpected lifecycle plan: {other:?}"),
        }
    }

    #[test]
    fn rename_lifecycle_cancels_focus_loss_without_stealing_new_focus() {
        let (graph, group, _) = graph_with_group_and_symbol();
        let session = RenameOverlaySession::Group(GroupRenameOverlay {
            group,
            invoked_at_window: Point::new(Px(100.0), Px(120.0)),
        });

        let plan = plan_rename_host_lifecycle(
            &NodeGraphStyle::default(),
            bounds(),
            Some(&session),
            Some(node(1)),
            Some(node(3)),
            Some(node(2)),
            Some(session.key()),
            |session| rename_session_seed_text(&graph, session),
        );

        assert_eq!(
            plan,
            RenameHostLifecyclePlan::CancelActiveSession {
                focus_restore: None
            }
        );
    }

    #[test]
    fn rename_lifecycle_restores_focus_when_hidden_child_still_owns_focus() {
        let child = node(1);
        let canvas = node(2);
        let plan = plan_rename_host_lifecycle(
            &NodeGraphStyle::default(),
            bounds(),
            None,
            Some(child),
            Some(child),
            Some(canvas),
            None,
            |_| String::new(),
        );

        assert_eq!(
            plan,
            RenameHostLifecyclePlan::Hidden {
                focus_restore: Some(canvas)
            }
        );
    }

    #[test]
    fn rename_lifecycle_seeds_symbol_sessions_from_graph() {
        let (graph, _, symbol) = graph_with_group_and_symbol();
        let session = RenameOverlaySession::Symbol(SymbolRenameOverlay {
            symbol,
            invoked_at_window: Point::new(Px(100.0), Px(120.0)),
        });

        let plan = plan_rename_host_lifecycle(
            &NodeGraphStyle::default(),
            bounds(),
            Some(&session),
            Some(node(1)),
            None,
            Some(node(2)),
            None,
            |session| rename_session_seed_text(&graph, session),
        );

        match plan {
            RenameHostLifecyclePlan::Active { seed_text, .. } => {
                assert_eq!(seed_text.as_deref(), Some("Symbol A"));
            }
            other => panic!("unexpected lifecycle plan: {other:?}"),
        }
    }
}
