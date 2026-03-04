use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

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
    last_focused_by_window: HashMap<AppWindowId, Option<GlobalElementId>>,
    last_non_tabstrip_focused_by_window: HashMap<AppWindowId, GlobalElementId>,
    return_focus_by_window_and_pane: HashMap<(AppWindowId, Arc<str>), GlobalElementId>,
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

/// Workspace-shell command routing scope.
///
/// This is intended for editor-like shells where pointer interactions should not steal focus from
/// the content surface, but keyboard users still need deterministic focus transfer commands.
///
/// In particular, it handles `workspace.pane.focus_tab_strip` by focusing the active tab in the
/// active pane's `WorkspaceTabStrip` (best-effort, gated by unit tests).
#[derive(Debug)]
pub struct WorkspaceCommandScope {
    window_layout: Model<WorkspaceWindowLayout>,
    child: AnyElement,
}

impl WorkspaceCommandScope {
    pub fn new(window_layout: Model<WorkspaceWindowLayout>, child: AnyElement) -> Self {
        Self {
            window_layout,
            child,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let window_layout = self.window_layout;
        let child = self.child;
        let tab_element_registry = workspace_tab_element_registry_model(cx);
        let pane_content_registry = workspace_pane_content_element_registry_model(cx);
        let focus_state = workspace_command_scope_focus_model(cx);

        // Best-effort: keep a snapshot of the latest focused element for the window so command
        // handlers can record/restore focus transfer outcomes without needing a runtime focus query.
        let focused_now = cx.focused_element();
        let window = cx.window;
        let focused_is_tabstrip = focused_now.is_some_and(|focused| {
            cx.app
                .models_mut()
                .read(&tab_element_registry, |reg| {
                    reg.contains_element_for_window(window, focused)
                })
                .unwrap_or(false)
        });
        let _ = cx.app.models_mut().update(&focus_state, |st| {
            let entry = st.last_focused_by_window.entry(window).or_insert(None);
            if *entry != focused_now {
                *entry = focused_now;
            }

            if let Some(focused) = focused_now
                && !focused_is_tabstrip
            {
                st.last_non_tabstrip_focused_by_window
                    .insert(window, focused);
            }
        });

        let root = cx.container(
            ContainerProps {
                layout: fill_layout(),
                ..Default::default()
            },
            move |_cx| vec![child],
        );

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
                            tab_id,
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
                            .flatten();

                        // Record the last focused element (best-effort) so `focus_content` can
                        // restore it after keyboard use of the tab strip. If no prior focus target
                        // is known, fall back to the pane's registered content target (if any).
                        let last_focus = host
                            .models_mut()
                            .read(&focus_state_for_command, |st| {
                                st.last_non_tabstrip_focused_by_window
                                    .get(&acx.window)
                                    .copied()
                            })
                            .ok()
                            .flatten()
                            .or(content_fallback);
                        if let Some(last_focus) = last_focus {
                            if last_focus != target {
                                let _ = host.models_mut().update(&focus_state_for_command, |st| {
                                    st.return_focus_by_window_and_pane
                                        .insert((acx.window, pane_id.clone()), last_focus);
                                });
                            }
                        }

                        host.request_focus(target);
                        let _ = host.models_mut().update(&focus_state_for_command, |st| {
                            st.last_focused_by_window.insert(acx.window, Some(target));
                        });
                        host.request_redraw(acx.window);
                        true
                    }
                    CMD_WORKSPACE_PANE_FOCUS_CONTENT => {
                        let pane_id = host
                            .models_mut()
                            .read(&window_layout_for_command, |w| w.active_pane_id().cloned())
                            .ok()
                            .flatten();
                        let Some(pane_id) = pane_id else {
                            return false;
                        };

                        let target = host
                            .models_mut()
                            .read(&focus_state_for_command, |st| {
                                st.return_focus_by_window_and_pane
                                    .get(&(acx.window, pane_id.clone()))
                                    .copied()
                            })
                            .ok()
                            .flatten();
                        let target = match target {
                            Some(target) => {
                                let _ = host.models_mut().update(&focus_state_for_command, |st| {
                                    st.return_focus_by_window_and_pane
                                        .remove(&(acx.window, pane_id.clone()));
                                });
                                Some(target)
                            }
                            None => host
                                .models_mut()
                                .read(&pane_content_registry_for_command, |reg| {
                                    reg.get(&WorkspacePaneContentElementKey {
                                        window: acx.window,
                                        pane_id: pane_id.clone(),
                                    })
                                })
                                .ok()
                                .flatten(),
                        };
                        let Some(target) = target else {
                            return false;
                        };

                        host.request_focus(target);
                        let _ = host.models_mut().update(&focus_state_for_command, |st| {
                            st.last_focused_by_window.insert(acx.window, Some(target));
                            st.last_non_tabstrip_focused_by_window
                                .insert(acx.window, target);
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

                        let focused_now = host
                            .models_mut()
                            .read(&focus_state_for_command, |st| {
                                st.last_focused_by_window
                                    .get(&acx.window)
                                    .copied()
                                    .flatten()
                            })
                            .ok()
                            .flatten();
                        let focused_in_active_pane_tabstrip = focused_now.is_some_and(|focused| {
                            host.models_mut()
                                .read(&tab_element_registry_for_command, |reg| {
                                    reg.contains_element_for_window_and_pane(
                                        acx.window, &pane_id, focused,
                                    )
                                })
                                .unwrap_or(false)
                        });

                        // If we're already in the tab strip, this is an "exit" gesture (back to
                        // content). If a return target was recorded, use it; otherwise, fall back
                        // to the registered pane content focus target (if any).
                        if focused_in_active_pane_tabstrip {
                            let target = host
                                .models_mut()
                                .read(&focus_state_for_command, |st| {
                                    st.return_focus_by_window_and_pane
                                        .get(&(acx.window, pane_id.clone()))
                                        .copied()
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
                                });
                            let Some(target) = target else {
                                return false;
                            };

                            let _ = host.models_mut().update(&focus_state_for_command, |st| {
                                st.return_focus_by_window_and_pane
                                    .remove(&(acx.window, pane_id));
                            });

                            host.request_focus(target);
                            let _ = host.models_mut().update(&focus_state_for_command, |st| {
                                st.last_focused_by_window.insert(acx.window, Some(target));
                                st.last_non_tabstrip_focused_by_window
                                    .insert(acx.window, target);
                            });
                            host.request_redraw(acx.window);
                            return true;
                        }

                        let key = WorkspaceTabElementKey {
                            window: acx.window,
                            pane_id: Some(pane_id.clone()),
                            tab_id,
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
                            .flatten();

                        // Record the last focused element (best-effort) so toggle can restore it.
                        // If no prior focus target is known, fall back to the pane's registered
                        // content target (if any).
                        let focused = host
                            .models_mut()
                            .read(&focus_state_for_command, |st| {
                                st.last_non_tabstrip_focused_by_window
                                    .get(&acx.window)
                                    .copied()
                            })
                            .ok()
                            .flatten()
                            .or(content_fallback);
                        if let Some(last_focus) = focused {
                            if last_focus != target {
                                let _ = host.models_mut().update(&focus_state_for_command, |st| {
                                    st.return_focus_by_window_and_pane
                                        .insert((acx.window, pane_id.clone()), last_focus);
                                });
                            }
                        }

                        host.request_focus(target);
                        let _ = host.models_mut().update(&focus_state_for_command, |st| {
                            st.last_focused_by_window.insert(acx.window, Some(target));
                        });
                        host.request_redraw(acx.window);
                        true
                    }
                    _ => {
                        if !command.as_str().starts_with("workspace.") {
                            return false;
                        }

                        let applied = host
                            .models_mut()
                            .update(&window_layout_for_command, |w| w.apply_command(&command))
                            .unwrap_or(false);
                        if applied {
                            host.request_redraw(acx.window);
                        }
                        applied
                    }
                }
            }),
        );

        root
    }
}
