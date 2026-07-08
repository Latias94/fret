#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::dock::declarative) enum DeclarativePointerMoveOwner {
    ViewportCapture,
    BlockedByViewportCapture,
    DividerDrag,
    FloatingDrag,
    PendingPanelDrag,
    PendingTabsGroupDrag,
    Hover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::dock::declarative) enum DeclarativePointerUpOwner {
    ViewportCapture,
    FloatingClose,
    FloatingDrag,
    DividerDrag,
    PendingPanelDrag,
    PendingTabsGroupDrag,
    TabClose,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::dock::declarative) enum DeclarativePointerCancelOwner {
    ViewportCapture,
    ActiveDockingOrFloatingSession,
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::dock::declarative) struct DeclarativeInteractionSnapshot {
    pub(in crate::dock::declarative) viewport_capture_for_pointer: bool,
    pub(in crate::dock::declarative) viewport_capture_in_window: bool,
    pub(in crate::dock::declarative) divider_drag_for_pointer: bool,
    pub(in crate::dock::declarative) floating_close_for_pointer: bool,
    pub(in crate::dock::declarative) floating_drag_for_pointer: bool,
    pub(in crate::dock::declarative) pending_panel_drag_for_pointer: bool,
    pub(in crate::dock::declarative) pending_tabs_drag_for_pointer: bool,
    pub(in crate::dock::declarative) tab_close_for_pointer: bool,
}

pub(in crate::dock::declarative) fn arbitrate_pointer_move(
    snapshot: DeclarativeInteractionSnapshot,
) -> DeclarativePointerMoveOwner {
    if snapshot.viewport_capture_for_pointer {
        return DeclarativePointerMoveOwner::ViewportCapture;
    }
    if snapshot.viewport_capture_in_window {
        return DeclarativePointerMoveOwner::BlockedByViewportCapture;
    }
    if snapshot.divider_drag_for_pointer {
        return DeclarativePointerMoveOwner::DividerDrag;
    }
    if snapshot.floating_drag_for_pointer {
        return DeclarativePointerMoveOwner::FloatingDrag;
    }
    if snapshot.pending_panel_drag_for_pointer {
        return DeclarativePointerMoveOwner::PendingPanelDrag;
    }
    if snapshot.pending_tabs_drag_for_pointer {
        return DeclarativePointerMoveOwner::PendingTabsGroupDrag;
    }
    DeclarativePointerMoveOwner::Hover
}

pub(in crate::dock::declarative) fn arbitrate_pointer_up(
    snapshot: DeclarativeInteractionSnapshot,
) -> DeclarativePointerUpOwner {
    if snapshot.viewport_capture_for_pointer {
        return DeclarativePointerUpOwner::ViewportCapture;
    }
    if snapshot.floating_close_for_pointer {
        return DeclarativePointerUpOwner::FloatingClose;
    }
    if snapshot.floating_drag_for_pointer {
        return DeclarativePointerUpOwner::FloatingDrag;
    }
    if snapshot.divider_drag_for_pointer {
        return DeclarativePointerUpOwner::DividerDrag;
    }
    if snapshot.pending_panel_drag_for_pointer {
        return DeclarativePointerUpOwner::PendingPanelDrag;
    }
    if snapshot.pending_tabs_drag_for_pointer {
        return DeclarativePointerUpOwner::PendingTabsGroupDrag;
    }
    if snapshot.tab_close_for_pointer {
        return DeclarativePointerUpOwner::TabClose;
    }
    DeclarativePointerUpOwner::None
}

pub(in crate::dock::declarative) fn arbitrate_pointer_cancel(
    snapshot: DeclarativeInteractionSnapshot,
) -> DeclarativePointerCancelOwner {
    if snapshot.viewport_capture_for_pointer {
        return DeclarativePointerCancelOwner::ViewportCapture;
    }
    if snapshot.tab_close_for_pointer
        || snapshot.pending_panel_drag_for_pointer
        || snapshot.pending_tabs_drag_for_pointer
        || snapshot.divider_drag_for_pointer
        || snapshot.floating_close_for_pointer
        || snapshot.floating_drag_for_pointer
    {
        return DeclarativePointerCancelOwner::ActiveDockingOrFloatingSession;
    }
    DeclarativePointerCancelOwner::None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> DeclarativeInteractionSnapshot {
        DeclarativeInteractionSnapshot::default()
    }

    #[test]
    fn pointer_move_arbitration_matches_adr_0072_priority() {
        assert_eq!(
            arbitrate_pointer_move(DeclarativeInteractionSnapshot {
                viewport_capture_for_pointer: true,
                divider_drag_for_pointer: true,
                floating_drag_for_pointer: true,
                pending_panel_drag_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerMoveOwner::ViewportCapture
        );
        assert_eq!(
            arbitrate_pointer_move(DeclarativeInteractionSnapshot {
                viewport_capture_in_window: true,
                divider_drag_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerMoveOwner::BlockedByViewportCapture
        );
        assert_eq!(
            arbitrate_pointer_move(DeclarativeInteractionSnapshot {
                divider_drag_for_pointer: true,
                floating_drag_for_pointer: true,
                pending_panel_drag_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerMoveOwner::DividerDrag
        );
        assert_eq!(
            arbitrate_pointer_move(DeclarativeInteractionSnapshot {
                floating_drag_for_pointer: true,
                pending_panel_drag_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerMoveOwner::FloatingDrag
        );
        assert_eq!(
            arbitrate_pointer_move(DeclarativeInteractionSnapshot {
                pending_panel_drag_for_pointer: true,
                pending_tabs_drag_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerMoveOwner::PendingPanelDrag
        );
        assert_eq!(
            arbitrate_pointer_move(DeclarativeInteractionSnapshot {
                pending_tabs_drag_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerMoveOwner::PendingTabsGroupDrag
        );
        assert_eq!(
            arbitrate_pointer_move(snapshot()),
            DeclarativePointerMoveOwner::Hover
        );
    }

    #[test]
    fn pointer_up_arbitration_commits_or_cleans_the_winning_owner_once() {
        assert_eq!(
            arbitrate_pointer_up(DeclarativeInteractionSnapshot {
                viewport_capture_for_pointer: true,
                floating_close_for_pointer: true,
                floating_drag_for_pointer: true,
                divider_drag_for_pointer: true,
                pending_panel_drag_for_pointer: true,
                pending_tabs_drag_for_pointer: true,
                tab_close_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerUpOwner::ViewportCapture
        );
        assert_eq!(
            arbitrate_pointer_up(DeclarativeInteractionSnapshot {
                floating_close_for_pointer: true,
                floating_drag_for_pointer: true,
                divider_drag_for_pointer: true,
                tab_close_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerUpOwner::FloatingClose
        );
        assert_eq!(
            arbitrate_pointer_up(DeclarativeInteractionSnapshot {
                floating_drag_for_pointer: true,
                divider_drag_for_pointer: true,
                pending_panel_drag_for_pointer: true,
                tab_close_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerUpOwner::FloatingDrag
        );
        assert_eq!(
            arbitrate_pointer_up(DeclarativeInteractionSnapshot {
                divider_drag_for_pointer: true,
                pending_panel_drag_for_pointer: true,
                tab_close_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerUpOwner::DividerDrag
        );
        assert_eq!(
            arbitrate_pointer_up(DeclarativeInteractionSnapshot {
                pending_panel_drag_for_pointer: true,
                pending_tabs_drag_for_pointer: true,
                tab_close_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerUpOwner::PendingPanelDrag
        );
        assert_eq!(
            arbitrate_pointer_up(DeclarativeInteractionSnapshot {
                pending_tabs_drag_for_pointer: true,
                tab_close_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerUpOwner::PendingTabsGroupDrag
        );
        assert_eq!(
            arbitrate_pointer_up(DeclarativeInteractionSnapshot {
                tab_close_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerUpOwner::TabClose
        );
        assert_eq!(
            arbitrate_pointer_up(snapshot()),
            DeclarativePointerUpOwner::None
        );
    }

    #[test]
    fn pointer_cancel_arbitration_clears_capture_before_other_sessions() {
        assert_eq!(
            arbitrate_pointer_cancel(DeclarativeInteractionSnapshot {
                viewport_capture_for_pointer: true,
                pending_panel_drag_for_pointer: true,
                floating_drag_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerCancelOwner::ViewportCapture
        );
        assert_eq!(
            arbitrate_pointer_cancel(DeclarativeInteractionSnapshot {
                pending_panel_drag_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerCancelOwner::ActiveDockingOrFloatingSession
        );
        assert_eq!(
            arbitrate_pointer_cancel(DeclarativeInteractionSnapshot {
                floating_close_for_pointer: true,
                ..snapshot()
            }),
            DeclarativePointerCancelOwner::ActiveDockingOrFloatingSession
        );
        assert_eq!(
            arbitrate_pointer_cancel(snapshot()),
            DeclarativePointerCancelOwner::None
        );
    }
}
