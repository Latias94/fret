use fret_core::{NodeId, Rect};

use crate::ui::style::NodeGraphStyle;

use super::rename_policy::{
    RenameOverlaySession, RenameOverlaySessionKey, rename_overlay_rect_at,
    rename_overlay_should_cancel_on_focus_loss,
};

#[derive(Debug, Clone)]
pub(super) enum RenameHostLayoutPlan {
    Hidden,
    CancelActiveSession,
    Active {
        rect: Rect,
        session_key: RenameOverlaySessionKey,
        just_opened: bool,
    },
}

pub(super) fn plan_rename_host_layout(
    style: &NodeGraphStyle,
    bounds: Rect,
    session: Option<&RenameOverlaySession>,
    child: Option<NodeId>,
    focus: Option<NodeId>,
    last_opened_session: Option<RenameOverlaySessionKey>,
) -> RenameHostLayoutPlan {
    let Some(session) = session else {
        return RenameHostLayoutPlan::Hidden;
    };

    let session_key = session.key();
    let just_opened = last_opened_session != Some(session_key);
    if rename_overlay_should_cancel_on_focus_loss(child, focus, just_opened) {
        return RenameHostLayoutPlan::CancelActiveSession;
    }

    RenameHostLayoutPlan::Active {
        rect: rename_overlay_rect_at(style, session.invoked_at_window(), bounds),
        session_key,
        just_opened,
    }
}

#[cfg(test)]
mod tests {
    use super::{RenameHostLayoutPlan, plan_rename_host_layout};
    use crate::core::{GroupId, SymbolId};
    use crate::ui::style::NodeGraphStyle;
    use crate::ui::{GroupRenameOverlay, SymbolRenameOverlay};
    use fret_core::{NodeId, Point, Px, Rect, Size};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        )
    }

    #[test]
    fn rename_host_layout_is_hidden_without_active_session() {
        let plan = plan_rename_host_layout(
            &NodeGraphStyle::default(),
            bounds(),
            None,
            Some(NodeId::default()),
            None,
            None,
        );
        assert!(matches!(plan, RenameHostLayoutPlan::Hidden));
    }

    #[test]
    fn rename_host_layout_cancels_focus_loss_after_open_frame() {
        let session =
            super::super::rename_policy::RenameOverlaySession::Group(GroupRenameOverlay {
                group: GroupId::new(),
                invoked_at_window: Point::new(Px(100.0), Px(120.0)),
            });
        let child = Some(NodeId::default());
        let plan = plan_rename_host_layout(
            &NodeGraphStyle::default(),
            bounds(),
            Some(&session),
            child,
            None,
            Some(session.key()),
        );
        assert!(matches!(plan, RenameHostLayoutPlan::CancelActiveSession));
    }

    #[test]
    fn rename_host_layout_marks_new_session_and_clamps_rect() {
        let session =
            super::super::rename_policy::RenameOverlaySession::Symbol(SymbolRenameOverlay {
                symbol: SymbolId::new(),
                invoked_at_window: Point::new(Px(780.0), Px(590.0)),
            });
        let plan = plan_rename_host_layout(
            &NodeGraphStyle::default(),
            bounds(),
            Some(&session),
            None,
            None,
            None,
        );

        match plan {
            RenameHostLayoutPlan::Active {
                rect,
                session_key,
                just_opened,
            } => {
                assert!(just_opened);
                assert_eq!(session_key, session.key());
                assert!(rect.origin.x.0 + rect.size.width.0 <= 800.0);
                assert!(rect.origin.y.0 + rect.size.height.0 <= 600.0);
            }
            other => panic!("unexpected plan: {other:?}"),
        }
    }
}
