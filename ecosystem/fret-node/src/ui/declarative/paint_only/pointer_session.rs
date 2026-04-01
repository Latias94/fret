use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NodeDragReleaseOutcome {
    pub(super) selection_committed: bool,
    pub(super) drag_committed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LeftPointerReleaseOutcome {
    None,
    NodeDrag(NodeDragReleaseOutcome),
    PendingSelection { selection_committed: bool },
    Marquee { selection_committed: bool },
}

impl LeftPointerReleaseOutcome {
    fn is_handled(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclarativeInteractionCancelMode {
    Escape,
    PointerCancel,
}

impl DeclarativeInteractionCancelMode {
    fn includes_node_drag(self, state: Option<&NodeDragState>) -> bool {
        match self {
            Self::Escape => state.is_some_and(NodeDragState::is_live),
            Self::PointerCancel => state.is_some(),
        }
    }

    fn clear_node_drag(self, state: &mut Option<NodeDragState>) {
        match self {
            Self::Escape => {
                if let Some(state) = state.as_mut() {
                    state.cancel();
                }
            }
            Self::PointerCancel => *state = None,
        }
    }
}

pub(super) fn complete_node_drag_release_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    node_drag: &NodeDragState,
    pending_selection: Option<&PendingSelectionState>,
) -> NodeDragReleaseOutcome {
    let graph = binding.graph_model();
    let view_state = binding.view_state_model();
    let view = host
        .models_mut()
        .read(&view_state, view_from_state)
        .ok()
        .unwrap_or_default();
    let drag_commit_delta = node_drag_commit_delta(view, node_drag);
    let selection_committed = pending_selection
        .is_some_and(|pending| commit_pending_selection_action_host(host, binding, pending));
    let drag_committed = if let Some((dx, dy)) = drag_commit_delta {
        host.models_mut()
            .read(&graph, |graph| {
                build_node_drag_transaction(graph, node_drag.nodes_sorted.as_ref(), dx, dy)
            })
            .ok()
            .is_some_and(|tx| commit_node_drag_transaction(host, binding, &tx))
    } else {
        false
    };
    NodeDragReleaseOutcome {
        selection_committed,
        drag_committed,
    }
}

pub(super) fn handle_node_drag_left_pointer_release_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    binding: &NodeGraphSurfaceBinding,
) -> Option<LeftPointerReleaseOutcome> {
    let node_drag_value = host
        .models_mut()
        .read(node_drag, |state| state.clone())
        .ok()
        .flatten();
    let Some(node_drag_value) = node_drag_value else {
        return None;
    };

    let pending_selection_value = host
        .models_mut()
        .read(pending_selection, |state| state.clone())
        .ok()
        .flatten();
    let outcome = complete_node_drag_release_action_host(
        host,
        binding,
        &node_drag_value,
        pending_selection_value.as_ref(),
    );
    let _ = host
        .models_mut()
        .update(pending_selection, |state| *state = None);
    let _ = host.models_mut().update(node_drag, |state| *state = None);
    Some(LeftPointerReleaseOutcome::NodeDrag(outcome))
}

pub(super) fn handle_pending_selection_left_pointer_release_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    pending_selection: &Model<Option<PendingSelectionState>>,
    binding: &NodeGraphSurfaceBinding,
) -> Option<LeftPointerReleaseOutcome> {
    let pending_selection_value = host
        .models_mut()
        .read(pending_selection, |state| state.clone())
        .ok()
        .flatten();
    let Some(pending_selection_value) = pending_selection_value else {
        return None;
    };

    let selection_committed =
        commit_pending_selection_action_host(host, binding, &pending_selection_value);
    let _ = host
        .models_mut()
        .update(pending_selection, |state| *state = None);
    Some(LeftPointerReleaseOutcome::PendingSelection {
        selection_committed,
    })
}

pub(super) fn handle_marquee_left_pointer_release_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    marquee: &Model<Option<MarqueeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    binding: &NodeGraphSurfaceBinding,
) -> Option<LeftPointerReleaseOutcome> {
    let marquee_value = host
        .models_mut()
        .read(marquee, |state| state.clone())
        .ok()
        .flatten();
    let Some(marquee_value) = marquee_value else {
        return None;
    };

    let selection_committed = if marquee_value.active || !marquee_value.toggle {
        commit_marquee_selection_action_host(host, binding, &marquee_value)
    } else {
        false
    };
    let _ = host
        .models_mut()
        .update(pending_selection, |state| *state = None);
    let _ = host.models_mut().update(marquee, |state| *state = None);
    Some(LeftPointerReleaseOutcome::Marquee {
        selection_committed,
    })
}

pub(super) fn complete_left_pointer_release_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    binding: &NodeGraphSurfaceBinding,
) -> LeftPointerReleaseOutcome {
    if let Some(release) = handle_node_drag_left_pointer_release_action_host(
        host,
        node_drag,
        pending_selection,
        binding,
    ) {
        return release;
    }

    if let Some(release) =
        handle_pending_selection_left_pointer_release_action_host(host, pending_selection, binding)
    {
        return release;
    }

    handle_marquee_left_pointer_release_action_host(host, marquee, pending_selection, binding)
        .unwrap_or(LeftPointerReleaseOutcome::None)
}

fn cancel_declarative_interactions_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    mode: DeclarativeInteractionCancelMode,
) -> bool {
    let drag_active = host
        .models_mut()
        .read(drag, |state| state.is_some())
        .ok()
        .unwrap_or(false);
    let marquee_active = host
        .models_mut()
        .read(marquee, |state| state.is_some())
        .ok()
        .unwrap_or(false);
    let node_drag_active = host
        .models_mut()
        .read(node_drag, |state| mode.includes_node_drag(state.as_ref()))
        .ok()
        .unwrap_or(false);
    let pending_selection_active = host
        .models_mut()
        .read(pending_selection, |state| state.is_some())
        .ok()
        .unwrap_or(false);

    if !drag_active && !marquee_active && !node_drag_active && !pending_selection_active {
        return false;
    }

    let _ = host.models_mut().update(drag, |state| *state = None);
    let _ = host.models_mut().update(marquee, |state| *state = None);
    let _ = host
        .models_mut()
        .update(pending_selection, |state| *state = None);
    let _ = host
        .models_mut()
        .update(node_drag, |state| mode.clear_node_drag(state));
    true
}

pub(super) fn escape_cancel_declarative_interactions_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
) -> bool {
    cancel_declarative_interactions_action_host(
        host,
        drag,
        marquee,
        node_drag,
        pending_selection,
        DeclarativeInteractionCancelMode::Escape,
    )
}

pub(super) fn pointer_cancel_declarative_interactions_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
) -> bool {
    cancel_declarative_interactions_action_host(
        host,
        drag,
        marquee,
        node_drag,
        pending_selection,
        DeclarativeInteractionCancelMode::PointerCancel,
    )
}

pub(super) fn notify_and_redraw_action_host<H>(host: &mut H, action_cx: fret_ui::action::ActionCx)
where
    H: fret_ui::action::UiActionHost + ?Sized,
{
    host.notify(action_cx);
    host.request_redraw(action_cx.window);
}

pub(super) fn invalidate_notify_and_redraw_pointer_action_host<H>(
    host: &mut H,
    action_cx: fret_ui::action::ActionCx,
    invalidation: Invalidation,
) where
    H: fret_ui::action::UiPointerActionHost + ?Sized,
{
    host.invalidate(invalidation);
    notify_and_redraw_action_host(host, action_cx);
}

fn finish_declarative_pointer_session_action_host<H>(
    host: &mut H,
    action_cx: fret_ui::action::ActionCx,
) where
    H: fret_ui::action::UiPointerActionHost + ?Sized,
{
    host.release_pointer_capture();
    invalidate_notify_and_redraw_pointer_action_host(host, action_cx, Invalidation::Layout);
}

pub(super) fn handle_declarative_pointer_up_action_host(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    action_cx: fret_ui::action::ActionCx,
    up: fret_ui::action::PointerUpCx,
    pan_button: MouseButton,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    binding: &NodeGraphSurfaceBinding,
) -> bool {
    if up.button != pan_button {
        if up.button != MouseButton::Left {
            return false;
        }

        let release = complete_left_pointer_release_action_host(
            host,
            node_drag,
            pending_selection,
            marquee,
            binding,
        );
        if !release.is_handled() {
            return false;
        }

        finish_declarative_pointer_session_action_host(host, action_cx);
        return true;
    }

    let _ = host.models_mut().update(drag, |state| *state = None);
    finish_declarative_pointer_session_action_host(host, action_cx);
    true
}

pub(super) fn handle_declarative_pointer_cancel_action_host(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    action_cx: fret_ui::action::ActionCx,
    _cancel: fret_ui::action::PointerCancelCx,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
) -> bool {
    let _ = pointer_cancel_declarative_interactions_action_host(
        host,
        drag,
        marquee,
        node_drag,
        pending_selection,
    );
    finish_declarative_pointer_session_action_host(host, action_cx);
    true
}
