use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::{AppWindowId, Px};
use fret_runtime::{Model, ModelId};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length};
use fret_ui::elements::GlobalElementId;
use fret_ui::{CommandAvailability, ElementContext, UiHost};
use fret_ui_kit::IntoUiElement;

use crate::commands::{
    CMD_WORKSPACE_PANE_FOCUS_CONTENT, CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP,
    CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS,
};
use crate::focus_registry::{WorkspaceTabElementKey, workspace_tab_element_registry_model};
use crate::layout::WorkspaceWindowLayout;
use crate::pane_content_focus::{
    WorkspacePaneContentElementKey, workspace_pane_content_element_registry_model,
};

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout.size.min_width = Some(Length::Px(Px(0.0)));
    layout.size.min_height = Some(Length::Px(Px(0.0)));
    layout
}

#[derive(Debug, Default)]
struct WorkspaceCommandScopeFocusState {
    reconciled_layout_revision_by_window: HashMap<AppWindowId, (ModelId, u64)>,
    last_focused_by_window: HashMap<AppWindowId, Option<GlobalElementId>>,
    focused_within_scope_by_window: HashMap<AppWindowId, bool>,
    focus_lane_by_window: HashMap<AppWindowId, Option<WorkspaceCommandScopeFocusLane>>,
    last_non_tabstrip_focused_by_window: HashMap<AppWindowId, GlobalElementId>,
    last_non_tabstrip_lane_by_window: HashMap<AppWindowId, Option<WorkspaceCommandScopeFocusLane>>,
    return_focus_by_window_and_pane:
        HashMap<(AppWindowId, Arc<str>), WorkspaceCommandScopeReturnFocus>,
}

fn workspace_registry_reconciliation_required(
    state: &WorkspaceCommandScopeFocusState,
    window: AppWindowId,
    layout_revision: Option<(ModelId, u64)>,
) -> bool {
    layout_revision.is_none_or(|revision| {
        state
            .reconciled_layout_revision_by_window
            .get(&window)
            .copied()
            != Some(revision)
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkspaceCommandScopeFocusLane {
    TabStrip(Arc<str>),
    PaneContent(Arc<str>),
}

impl WorkspaceCommandScopeFocusLane {
    pub(crate) fn pane_id(&self) -> &Arc<str> {
        match self {
            Self::TabStrip(pane_id) | Self::PaneContent(pane_id) => pane_id,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceCommandScopeFocusSnapshot {
    pub(crate) target: Option<GlobalElementId>,
    pub(crate) lane: Option<WorkspaceCommandScopeFocusLane>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkspaceCommandScopeReturnFocus {
    target: GlobalElementId,
    lane: Option<WorkspaceCommandScopeFocusLane>,
    tab_id: Option<Arc<str>>,
}

fn last_return_focus_for_pane(
    state: &WorkspaceCommandScopeFocusState,
    window: AppWindowId,
    pane_id: &Arc<str>,
    tab_id: &Arc<str>,
) -> Option<WorkspaceCommandScopeReturnFocus> {
    let target = state
        .last_non_tabstrip_focused_by_window
        .get(&window)
        .copied()?;
    let lane = state
        .last_non_tabstrip_lane_by_window
        .get(&window)
        .cloned()
        .unwrap_or(None);
    let belongs_to_target_pane = match lane.as_ref() {
        Some(WorkspaceCommandScopeFocusLane::PaneContent(owner)) => owner == pane_id,
        Some(WorkspaceCommandScopeFocusLane::TabStrip(_)) => false,
        None => true,
    };
    belongs_to_target_pane.then_some(WorkspaceCommandScopeReturnFocus {
        target,
        lane,
        tab_id: Some(tab_id.clone()),
    })
}

fn take_return_focus_for_active_tab(
    state: &mut WorkspaceCommandScopeFocusState,
    window: AppWindowId,
    pane_id: &Arc<str>,
    tab_id: &Arc<str>,
) -> Option<WorkspaceCommandScopeReturnFocus> {
    state
        .return_focus_by_window_and_pane
        .remove(&(window, pane_id.clone()))
        .filter(|focus| focus.tab_id.as_ref() == Some(tab_id))
}

fn record_requested_focus(
    state: &mut WorkspaceCommandScopeFocusState,
    window: AppWindowId,
    focus: WorkspaceCommandScopeReturnFocus,
) {
    state
        .last_focused_by_window
        .insert(window, Some(focus.target));
    state.focused_within_scope_by_window.insert(window, true);
    state
        .focus_lane_by_window
        .insert(window, focus.lane.clone());
    if !matches!(
        focus.lane,
        Some(WorkspaceCommandScopeFocusLane::TabStrip(_))
    ) {
        state
            .last_non_tabstrip_focused_by_window
            .insert(window, focus.target);
        state
            .last_non_tabstrip_lane_by_window
            .insert(window, focus.lane);
    }
}

#[derive(Default)]
struct WorkspaceCommandScopeFocusGlobal {
    model: Option<Model<WorkspaceCommandScopeFocusState>>,
}

fn workspace_command_scope_focus_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<WorkspaceCommandScopeFocusState> {
    cx.app
        .with_global_mut_untracked(WorkspaceCommandScopeFocusGlobal::default, |global, app| {
            if let Some(model) = global.model.clone() {
                return model;
            }
            let model = app
                .models_mut()
                .insert(WorkspaceCommandScopeFocusState::default());
            global.model = Some(model.clone());
            model
        })
}

pub(crate) fn workspace_command_scope_focus_snapshot<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
) -> WorkspaceCommandScopeFocusSnapshot {
    let model = app
        .with_global_mut_untracked(WorkspaceCommandScopeFocusGlobal::default, |global, _app| {
            global.model.clone()
        });
    let Some(model) = model else {
        return WorkspaceCommandScopeFocusSnapshot::default();
    };
    app.models_mut()
        .read(&model, |state| WorkspaceCommandScopeFocusSnapshot {
            target: state.last_focused_by_window.get(&window).copied().flatten(),
            lane: state.focus_lane_by_window.get(&window).cloned().flatten(),
        })
        .unwrap_or_default()
}

/// Workspace-shell command routing scope.
///
/// This is intended for editor-like shells where pointer interactions should not steal focus from
/// the content surface, but keyboard users still need deterministic focus transfer commands.
///
/// In particular, it handles `workspace.pane.focus_tab_strip` by focusing the active tab in the
/// active pane's `WorkspaceTabStrip` (best-effort, gated by unit tests).
#[derive(Debug)]
pub struct WorkspaceCommandScope<T = AnyElement> {
    window_layout: Model<WorkspaceWindowLayout>,
    child: T,
}

impl<T> WorkspaceCommandScope<T> {
    pub fn new(window_layout: Model<WorkspaceWindowLayout>, child: T) -> Self {
        Self {
            window_layout,
            child,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        T: IntoUiElement<H>,
    {
        let window_layout = self.window_layout;
        let child = self.child.into_element(cx);
        let tab_element_registry = workspace_tab_element_registry_model(cx);
        let pane_content_registry = workspace_pane_content_element_registry_model(cx);
        let focus_state = workspace_command_scope_focus_model(cx);
        let window = cx.window;

        let layout_revision = window_layout
            .revision(cx.app)
            .map(|revision| (window_layout.id(), revision));
        let reconcile_registries = cx
            .app
            .models_mut()
            .read(&focus_state, |state| {
                workspace_registry_reconciliation_required(state, window, layout_revision)
            })
            .unwrap_or(true);
        let live_tab_ids_by_pane = if reconcile_registries {
            cx.app
                .models_mut()
                .read(&window_layout, |layout| {
                    let mut pane_ids = Vec::new();
                    layout.pane_tree.collect_leaf_ids(&mut pane_ids);
                    pane_ids
                        .into_iter()
                        .filter_map(|pane_id| {
                            let pane = layout.pane_tree.find_pane(pane_id.as_ref())?;
                            let tab_ids = pane.tabs.tabs().iter().cloned().collect::<HashSet<_>>();
                            Some((pane_id, tab_ids))
                        })
                        .collect::<HashMap<_, _>>()
                })
                .ok()
        } else {
            None
        };
        if let Some(live_tab_ids_by_pane) = live_tab_ids_by_pane {
            let tab_registry_reconciled = cx
                .app
                .models_mut()
                .read(&tab_element_registry, |registry| {
                    registry.needs_workspace_reconciliation(window, &live_tab_ids_by_pane)
                })
                .map(|needs_reconciliation| {
                    !needs_reconciliation
                        || cx
                            .app
                            .models_mut()
                            .update(&tab_element_registry, |registry| {
                                registry.reconcile_workspace_tabs_for_window(
                                    window,
                                    &live_tab_ids_by_pane,
                                );
                            })
                            .is_ok()
                })
                .unwrap_or(false);
            let live_pane_ids = live_tab_ids_by_pane.keys().cloned().collect::<HashSet<_>>();
            let content_registry_reconciled = cx
                .app
                .models_mut()
                .read(&pane_content_registry, |registry| {
                    registry.needs_workspace_reconciliation(window, &live_pane_ids)
                })
                .map(|needs_reconciliation| {
                    !needs_reconciliation
                        || cx
                            .app
                            .models_mut()
                            .update(&pane_content_registry, |registry| {
                                registry
                                    .reconcile_workspace_panes_for_window(window, &live_pane_ids);
                            })
                            .is_ok()
                })
                .unwrap_or(false);
            if tab_registry_reconciled
                && content_registry_reconciled
                && let Some(layout_revision) = layout_revision
            {
                let _ = cx.app.models_mut().update(&focus_state, |state| {
                    state
                        .reconciled_layout_revision_by_window
                        .insert(window, layout_revision);
                });
            }
        }

        let root = cx.container(
            ContainerProps {
                layout: fill_layout(),
                ..Default::default()
            },
            move |_cx| vec![child],
        );
        cx.action_route_fallback_root(root.id);
        cx.command_on_command_availability_for(
            root.id,
            Arc::new(|_host, acx, command| {
                if matches!(
                    command.as_str(),
                    CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP
                        | CMD_WORKSPACE_PANE_FOCUS_CONTENT
                        | CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS
                ) {
                    if acx.input_ctx.ui_has_modal {
                        CommandAvailability::Blocked
                    } else {
                        CommandAvailability::Available
                    }
                } else {
                    CommandAvailability::NotHandled
                }
            }),
        );

        // Publish the current focus target plus its last workspace-owned lane. Portal overlays are
        // outside this scope, so they update the exact target without erasing the lane that should
        // receive focus after the overlay or dirty-close transaction is removed.
        let focused_now = cx.focused_element();
        let focused_within_scope = focused_now.is_some() && cx.is_focus_within_element(root.id);
        let focused_tabstrip_pane = focused_now.and_then(|_| {
            cx.app
                .models_mut()
                .read(&tab_element_registry, |reg| {
                    reg.pane_elements_for_window(window)
                })
                .unwrap_or_default()
                .into_iter()
                .find_map(|(pane_id, element)| {
                    cx.is_focus_within_element(element).then_some(pane_id)
                })
        });
        let focused_content_pane = focused_now
            .and_then(|_| {
                focused_tabstrip_pane.is_none().then(|| {
                    cx.app
                        .models_mut()
                        .read(&pane_content_registry, |reg| {
                            reg.pane_elements_for_window(window)
                        })
                        .unwrap_or_default()
                        .into_iter()
                        .find_map(|(pane_id, element)| {
                            cx.is_focus_within_element(element).then_some(pane_id)
                        })
                })
            })
            .flatten();
        let focus_lane = focused_tabstrip_pane
            .clone()
            .map(WorkspaceCommandScopeFocusLane::TabStrip)
            .or_else(|| {
                focused_content_pane
                    .clone()
                    .map(WorkspaceCommandScopeFocusLane::PaneContent)
            });
        let focused_is_tabstrip = matches!(
            focus_lane,
            Some(WorkspaceCommandScopeFocusLane::TabStrip(_))
        );
        let needs_focus_state_update = cx
            .app
            .models_mut()
            .read(&focus_state, |st| {
                let focused_changed = st
                    .last_focused_by_window
                    .get(&window)
                    .copied()
                    .unwrap_or(None)
                    != focused_now;
                let scope_changed = st
                    .focused_within_scope_by_window
                    .get(&window)
                    .copied()
                    .unwrap_or(false)
                    != focused_within_scope;
                let focus_lane_changed = st
                    .focus_lane_by_window
                    .get(&window)
                    .cloned()
                    .unwrap_or(None)
                    != focus_lane;
                let non_tabstrip_changed = if let Some(focused) = focused_now {
                    focused_within_scope
                        && !focused_is_tabstrip
                        && (st.last_non_tabstrip_focused_by_window.get(&window).copied()
                            != Some(focused)
                            || st
                                .last_non_tabstrip_lane_by_window
                                .get(&window)
                                .cloned()
                                .unwrap_or(None)
                                != focus_lane)
                } else {
                    false
                };
                focused_changed
                    || scope_changed
                    || (focused_within_scope && focus_lane_changed)
                    || non_tabstrip_changed
            })
            .unwrap_or(true);
        if needs_focus_state_update {
            let _ = cx.app.models_mut().update(&focus_state, |st| {
                let entry = st.last_focused_by_window.entry(window).or_insert(None);
                if *entry != focused_now {
                    *entry = focused_now;
                }
                st.focused_within_scope_by_window
                    .insert(window, focused_within_scope);
                if focused_within_scope {
                    st.focus_lane_by_window.insert(window, focus_lane.clone());
                }

                if let Some(focused) = focused_now
                    && focused_within_scope
                    && !focused_is_tabstrip
                {
                    st.last_non_tabstrip_focused_by_window
                        .insert(window, focused);
                    st.last_non_tabstrip_lane_by_window
                        .insert(window, focus_lane.clone());
                }
            });
        }

        let window_layout_for_command = window_layout.clone();
        let tab_element_registry_for_command = tab_element_registry.clone();
        let pane_content_registry_for_command = pane_content_registry.clone();
        let focus_state_for_command = focus_state.clone();
        cx.command_on_command_for(
            root.id,
            Arc::new(move |host, acx, command| {
                match command.as_str() {
                    CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP => {
                        let active = host.models_mut().read(&window_layout_for_command, |w| {
                            let pane_id = w.active_pane_id().cloned()?;
                            let pane = w.pane_tree.find_pane(pane_id.as_ref())?;
                            let tab_id = pane.tabs.active().cloned()?;
                            Some((pane_id, tab_id))
                        });
                        let Some((pane_id, tab_id)) = active.ok().flatten() else {
                            return false;
                        };

                        let key = WorkspaceTabElementKey {
                            window: acx.window,
                            pane_id: Some(pane_id.clone()),
                            tab_id: tab_id.clone(),
                        };

                        let target: Option<GlobalElementId> = host
                            .models_mut()
                            .read(&tab_element_registry_for_command, |reg| reg.get(&key))
                            .ok()
                            .flatten();
                        let Some(target) = target else {
                            return false;
                        };

                        let content_fallback = host
                            .models_mut()
                            .read(&pane_content_registry_for_command, |reg| {
                                reg.get(&WorkspacePaneContentElementKey {
                                    window: acx.window,
                                    pane_id: pane_id.clone(),
                                })
                            })
                            .ok()
                            .flatten()
                            .map(|target| WorkspaceCommandScopeReturnFocus {
                                target,
                                lane: Some(WorkspaceCommandScopeFocusLane::PaneContent(
                                    pane_id.clone(),
                                )),
                                tab_id: Some(tab_id.clone()),
                            });

                        // Record the last focused element (best-effort) so `focus_content` can
                        // restore it after keyboard use of the tab strip. If no prior focus target
                        // is known, fall back to the pane's registered content target (if any).
                        let last_focus = host
                            .models_mut()
                            .read(&focus_state_for_command, |st| {
                                last_return_focus_for_pane(st, acx.window, &pane_id, &tab_id)
                            })
                            .ok()
                            .flatten()
                            .or(content_fallback);
                        if let Some(last_focus) = last_focus
                            && last_focus.target != target
                        {
                            let _ = host.models_mut().update(&focus_state_for_command, |st| {
                                st.return_focus_by_window_and_pane
                                    .insert((acx.window, pane_id.clone()), last_focus);
                            });
                        }

                        host.request_focus(target);
                        let _ = host.models_mut().update(&focus_state_for_command, |st| {
                            record_requested_focus(
                                st,
                                acx.window,
                                WorkspaceCommandScopeReturnFocus {
                                    target,
                                    lane: Some(WorkspaceCommandScopeFocusLane::TabStrip(
                                        pane_id.clone(),
                                    )),
                                    tab_id: Some(tab_id),
                                },
                            );
                        });
                        host.request_redraw(acx.window);
                        true
                    }
                    CMD_WORKSPACE_PANE_FOCUS_CONTENT => {
                        let active = host.models_mut().read(&window_layout_for_command, |w| {
                            let pane_id = w.active_pane_id().cloned()?;
                            let pane = w.pane_tree.find_pane(pane_id.as_ref())?;
                            let tab_id = pane.tabs.active().cloned()?;
                            Some((pane_id, tab_id))
                        });
                        let Some((pane_id, tab_id)) = active.ok().flatten() else {
                            return false;
                        };

                        let return_focus = host
                            .models_mut()
                            .update(&focus_state_for_command, |st| {
                                take_return_focus_for_active_tab(st, acx.window, &pane_id, &tab_id)
                            })
                            .ok()
                            .flatten();
                        let focus = match return_focus {
                            Some(focus) => Some(focus),
                            None => host
                                .models_mut()
                                .read(&pane_content_registry_for_command, |reg| {
                                    reg.get(&WorkspacePaneContentElementKey {
                                        window: acx.window,
                                        pane_id: pane_id.clone(),
                                    })
                                })
                                .ok()
                                .flatten()
                                .map(|target| WorkspaceCommandScopeReturnFocus {
                                    target,
                                    lane: Some(WorkspaceCommandScopeFocusLane::PaneContent(
                                        pane_id.clone(),
                                    )),
                                    tab_id: Some(tab_id),
                                }),
                        };
                        let Some(focus) = focus else {
                            return false;
                        };

                        host.request_focus(focus.target);
                        let _ = host.models_mut().update(&focus_state_for_command, |st| {
                            record_requested_focus(st, acx.window, focus.clone());
                        });
                        host.request_redraw(acx.window);
                        true
                    }
                    CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS => {
                        let active = host.models_mut().read(&window_layout_for_command, |w| {
                            let pane_id = w.active_pane_id().cloned()?;
                            let pane = w.pane_tree.find_pane(pane_id.as_ref())?;
                            let tab_id = pane.tabs.active().cloned()?;
                            Some((pane_id, tab_id))
                        });
                        let Some((pane_id, tab_id)) = active.ok().flatten() else {
                            return false;
                        };

                        let focused_in_active_pane_tabstrip = host
                            .models_mut()
                            .read(&focus_state_for_command, |st| {
                                st.focus_lane_by_window
                                    .get(&acx.window)
                                    .is_some_and(|lane| {
                                        matches!(
                                            lane,
                                            Some(WorkspaceCommandScopeFocusLane::TabStrip(owner))
                                                if owner == &pane_id
                                        )
                                    })
                            })
                            .unwrap_or(false);

                        // If we're already in the tab strip, this is an "exit" gesture (back to
                        // content). If a return target was recorded, use it; otherwise, fall back
                        // to the registered pane content focus target (if any).
                        if focused_in_active_pane_tabstrip {
                            let return_focus = host
                                .models_mut()
                                .update(&focus_state_for_command, |st| {
                                    take_return_focus_for_active_tab(
                                        st, acx.window, &pane_id, &tab_id,
                                    )
                                })
                                .ok()
                                .flatten()
                                .or_else(|| {
                                    host.models_mut()
                                        .read(&pane_content_registry_for_command, |reg| {
                                            reg.get(&WorkspacePaneContentElementKey {
                                                window: acx.window,
                                                pane_id: pane_id.clone(),
                                            })
                                        })
                                        .ok()
                                        .flatten()
                                        .map(|target| WorkspaceCommandScopeReturnFocus {
                                            target,
                                            lane: Some(
                                                WorkspaceCommandScopeFocusLane::PaneContent(
                                                    pane_id.clone(),
                                                ),
                                            ),
                                            tab_id: Some(tab_id.clone()),
                                        })
                                });
                            let Some(return_focus) = return_focus else {
                                return false;
                            };

                            host.request_focus(return_focus.target);
                            let _ = host.models_mut().update(&focus_state_for_command, |st| {
                                record_requested_focus(st, acx.window, return_focus.clone());
                            });
                            host.request_redraw(acx.window);
                            return true;
                        }

                        let key = WorkspaceTabElementKey {
                            window: acx.window,
                            pane_id: Some(pane_id.clone()),
                            tab_id: tab_id.clone(),
                        };

                        let target: Option<GlobalElementId> = host
                            .models_mut()
                            .read(&tab_element_registry_for_command, |reg| reg.get(&key))
                            .ok()
                            .flatten();
                        let Some(target) = target else {
                            return false;
                        };

                        let content_fallback = host
                            .models_mut()
                            .read(&pane_content_registry_for_command, |reg| {
                                reg.get(&WorkspacePaneContentElementKey {
                                    window: acx.window,
                                    pane_id: pane_id.clone(),
                                })
                            })
                            .ok()
                            .flatten()
                            .map(|target| WorkspaceCommandScopeReturnFocus {
                                target,
                                lane: Some(WorkspaceCommandScopeFocusLane::PaneContent(
                                    pane_id.clone(),
                                )),
                                tab_id: Some(tab_id.clone()),
                            });

                        // Record the last focused element (best-effort) so toggle can restore it.
                        // If no prior focus target is known, fall back to the pane's registered
                        // content target (if any).
                        let focused = host
                            .models_mut()
                            .read(&focus_state_for_command, |st| {
                                last_return_focus_for_pane(st, acx.window, &pane_id, &tab_id)
                            })
                            .ok()
                            .flatten()
                            .or(content_fallback);
                        if let Some(last_focus) = focused
                            && last_focus.target != target
                        {
                            let _ = host.models_mut().update(&focus_state_for_command, |st| {
                                st.return_focus_by_window_and_pane
                                    .insert((acx.window, pane_id.clone()), last_focus);
                            });
                        }

                        host.request_focus(target);
                        let _ = host.models_mut().update(&focus_state_for_command, |st| {
                            record_requested_focus(
                                st,
                                acx.window,
                                WorkspaceCommandScopeReturnFocus {
                                    target,
                                    lane: Some(WorkspaceCommandScopeFocusLane::TabStrip(
                                        pane_id.clone(),
                                    )),
                                    tab_id: Some(tab_id),
                                },
                            );
                        });
                        host.request_redraw(acx.window);
                        true
                    }
                    _ => false,
                }
            }),
        );

        root
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WorkspaceCommandScope, WorkspaceCommandScopeFocusState,
        workspace_registry_reconciliation_required,
    };
    use crate::layout::WorkspaceWindowLayout;
    use fret_app::App;
    use fret_core::AppWindowId;
    use fret_runtime::{Model, ModelStore};
    use fret_ui::ElementContext;
    use fret_ui::element::AnyElement;
    use fret_ui_kit::ui;

    #[allow(dead_code)]
    fn workspace_command_scope_accepts_typed_children(
        cx: &mut ElementContext<'_, App>,
        window_layout: Model<WorkspaceWindowLayout>,
    ) -> AnyElement {
        WorkspaceCommandScope::new(window_layout, ui::text("body")).into_element(cx)
    }

    #[test]
    fn workspace_registry_reconciliation_cache_is_keyed_by_model_and_revision() {
        let window = AppWindowId::default();
        let mut models = ModelStore::default();
        let first_layout = models.insert(());
        let replacement_layout = models.insert(());
        let first_revision = (
            first_layout.id(),
            models
                .revision(&first_layout)
                .expect("inserted layout must have a revision"),
        );
        let mut state = WorkspaceCommandScopeFocusState::default();

        assert!(workspace_registry_reconciliation_required(
            &state,
            window,
            Some(first_revision),
        ));

        state
            .reconciled_layout_revision_by_window
            .insert(window, first_revision);

        let unchanged_layout_requires_reconciliation =
            workspace_registry_reconciliation_required(&state, window, Some(first_revision));
        assert!(
            !unchanged_layout_requires_reconciliation,
            "an unchanged layout must not rebuild the pane/tab reconciliation sets"
        );
        models
            .update(&first_layout, |_| {})
            .expect("layout update must succeed");
        let updated_revision = (
            first_layout.id(),
            models
                .revision(&first_layout)
                .expect("updated layout must retain its revision"),
        );
        assert_ne!(updated_revision, first_revision);
        assert!(workspace_registry_reconciliation_required(
            &state,
            window,
            Some(updated_revision),
        ));
        assert!(workspace_registry_reconciliation_required(
            &state,
            window,
            Some((replacement_layout.id(), first_revision.1)),
        ));
        assert!(
            workspace_registry_reconciliation_required(&state, window, None),
            "a missing revision must retry instead of treating the cache as current"
        );
    }
}
