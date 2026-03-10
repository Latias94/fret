use super::*;

impl UiGalleryDriver {
    pub(crate) fn build_workspace_window_layout(
        selected_page: Arc<str>,
        workspace_tabs: &[Arc<str>],
        workspace_dirty_tabs: &[Arc<str>],
    ) -> WorkspaceWindowLayout {
        let mut tabs = workspace_tabs.to_vec();
        if tabs.is_empty() {
            tabs.push(selected_page.clone());
        }
        if !tabs
            .iter()
            .any(|tab_id| tab_id.as_ref() == selected_page.as_ref())
        {
            tabs.push(selected_page.clone());
        }

        let mut layout = WorkspaceWindowLayout::new(
            UI_GALLERY_WORKSPACE_WINDOW_LAYOUT_ID,
            UI_GALLERY_WORKSPACE_PANE_ID,
        );
        layout.active_pane = Some(Arc::from(UI_GALLERY_WORKSPACE_PANE_ID));

        if let Some(pane) = layout.pane_tree.find_pane_mut(UI_GALLERY_WORKSPACE_PANE_ID) {
            for tab_id in tabs.iter().cloned() {
                pane.tabs.open_and_activate(tab_id);
            }
            let _ = pane.tabs.activate(selected_page.clone());
            for tab_id in workspace_dirty_tabs.iter().cloned() {
                pane.tabs.set_dirty(tab_id, true);
            }
        }

        layout
    }

    pub(crate) fn workspace_window_layout_is_supported(layout: &WorkspaceWindowLayout) -> bool {
        if layout.active_pane.as_deref() != Some(UI_GALLERY_WORKSPACE_PANE_ID) {
            return false;
        }

        let mut pane_ids = Vec::new();
        layout.pane_tree.collect_leaf_ids(&mut pane_ids);
        pane_ids.len() == 1 && pane_ids[0].as_ref() == UI_GALLERY_WORKSPACE_PANE_ID
    }

    pub(crate) fn workspace_window_layout_snapshot(
        layout: &WorkspaceWindowLayout,
    ) -> Option<(Vec<Arc<str>>, Option<Arc<str>>, Vec<Arc<str>>)> {
        if !Self::workspace_window_layout_is_supported(layout) {
            return None;
        }

        let pane = layout.pane_tree.find_pane(UI_GALLERY_WORKSPACE_PANE_ID)?;
        let workspace_tabs = pane.tabs.tabs().to_vec();
        let selected_page = pane.tabs.active().cloned();
        let workspace_dirty_tabs = workspace_tabs
            .iter()
            .filter(|tab_id| pane.tabs.is_dirty(tab_id.as_ref()))
            .cloned()
            .collect();

        Some((workspace_tabs, selected_page, workspace_dirty_tabs))
    }

    pub(crate) fn rebuild_workspace_tab_close_command_map(
        state: &mut UiGalleryWindowState,
        workspace_tabs: &[Arc<str>],
    ) {
        state.workspace_tab_close_by_command.clear();
        for tab_id in workspace_tabs {
            let cmd = Self::workspace_tab_close_command(tab_id.as_ref());
            state
                .workspace_tab_close_by_command
                .insert(cmd, tab_id.clone());
        }
    }

    pub(crate) fn workspace_tab_close_command(tab_id: &str) -> Arc<str> {
        Arc::from(format!("{}{}", CMD_WORKSPACE_TAB_CLOSE_PREFIX, tab_id))
    }

    pub(crate) fn select_gallery_page_in_models(
        app: &mut App,
        state: &mut UiGalleryWindowState,
        page: Arc<str>,
    ) {
        let page_for_selected = page.clone();
        let page_for_tabs = page.clone();
        let _ = app.models_mut().update(&state.selected_page, |selected| {
            *selected = page_for_selected
        });
        let _ = app.models_mut().update(&state.workspace_tabs, |tabs| {
            if !tabs
                .iter()
                .any(|tab_id| tab_id.as_ref() == page_for_tabs.as_ref())
            {
                tabs.push(page_for_tabs.clone());
            }
        });
        state
            .workspace_tab_close_by_command
            .insert(Self::workspace_tab_close_command(page.as_ref()), page);
        Self::sync_workspace_window_layout_from_models(app, state);
    }

    pub(crate) fn navigate_to_gallery_page(
        app: &mut App,
        state: &mut UiGalleryWindowState,
        window: AppWindowId,
        page: Arc<str>,
        action: NavigationAction,
    ) {
        let page_for_router = page.clone();
        Self::select_gallery_page_in_models(app, state, page);
        apply_page_route_side_effects_via_router(
            app,
            window,
            action,
            page_for_router,
            &mut state.page_router,
        );
    }

    pub(crate) fn sync_workspace_window_layout_from_models(
        app: &mut App,
        state: &UiGalleryWindowState,
    ) {
        let selected_page = app
            .models()
            .get_cloned(&state.selected_page)
            .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
        let workspace_tabs = app
            .models()
            .get_cloned(&state.workspace_tabs)
            .unwrap_or_default();
        let workspace_dirty_tabs = app
            .models()
            .get_cloned(&state.workspace_dirty_tabs)
            .unwrap_or_default();

        let mut expected_tabs = workspace_tabs.clone();
        if expected_tabs.is_empty() {
            expected_tabs.push(selected_page.clone());
        }
        if !expected_tabs
            .iter()
            .any(|tab_id| tab_id.as_ref() == selected_page.as_ref())
        {
            expected_tabs.push(selected_page.clone());
        }

        let current_layout = app.models().get_cloned(&state.workspace_window_layout);
        let current_snapshot = current_layout
            .as_ref()
            .and_then(Self::workspace_window_layout_snapshot);
        let expected_snapshot = Some((
            expected_tabs.clone(),
            Some(selected_page.clone()),
            workspace_dirty_tabs.clone(),
        ));

        let layout_matches = current_layout
            .as_ref()
            .is_some_and(Self::workspace_window_layout_is_supported)
            && current_snapshot == expected_snapshot;
        if layout_matches {
            return;
        }

        let next_layout = Self::build_workspace_window_layout(
            selected_page,
            &workspace_tabs,
            &workspace_dirty_tabs,
        );
        let _ = app
            .models_mut()
            .update(&state.workspace_window_layout, |layout| {
                *layout = next_layout
            });
    }

    pub(crate) fn sync_workspace_models_from_window_layout(
        app: &mut App,
        state: &mut UiGalleryWindowState,
        window: AppWindowId,
    ) -> bool {
        let Some(layout) = app.models().get_cloned(&state.workspace_window_layout) else {
            return false;
        };
        let Some((workspace_tabs, selected_page, workspace_dirty_tabs)) =
            Self::workspace_window_layout_snapshot(&layout)
        else {
            Self::sync_workspace_window_layout_from_models(app, state);
            return false;
        };
        let Some(selected_page) = selected_page.or_else(|| workspace_tabs.first().cloned()) else {
            Self::sync_workspace_window_layout_from_models(app, state);
            return false;
        };
        if workspace_tabs.is_empty() {
            Self::sync_workspace_window_layout_from_models(app, state);
            return false;
        }

        let prev_selected_page = app
            .models()
            .get_cloned(&state.selected_page)
            .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
        let prev_workspace_tabs = app
            .models()
            .get_cloned(&state.workspace_tabs)
            .unwrap_or_default();
        let prev_workspace_dirty_tabs = app
            .models()
            .get_cloned(&state.workspace_dirty_tabs)
            .unwrap_or_default();

        let mut did_change = false;

        if prev_workspace_tabs != workspace_tabs {
            let tabs_for_model = workspace_tabs.clone();
            let _ = app
                .models_mut()
                .update(&state.workspace_tabs, |tabs| *tabs = tabs_for_model);
            Self::rebuild_workspace_tab_close_command_map(state, &workspace_tabs);
            did_change = true;
        }

        if prev_workspace_dirty_tabs != workspace_dirty_tabs {
            let dirty_for_model = workspace_dirty_tabs.clone();
            let _ = app
                .models_mut()
                .update(&state.workspace_dirty_tabs, |dirty| {
                    *dirty = dirty_for_model
                });
            did_change = true;
        }

        if prev_selected_page != selected_page {
            let selected_for_model = selected_page.clone();
            let _ = app.models_mut().update(&state.selected_page, |selected| {
                *selected = selected_for_model
            });
            apply_page_route_side_effects_via_router(
                app,
                window,
                NavigationAction::Replace,
                selected_page,
                &mut state.page_router,
            );
            did_change = true;
        }

        did_change
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn sync_page_router_from_external_history(
        app: &mut App,
        window: AppWindowId,
        state: &mut UiGalleryWindowState,
    ) {
        let Ok(update) = state.page_router.sync_with_prefetch_intents() else {
            return;
        };

        if !update.update.changed() {
            return;
        }

        let next_page = page_from_gallery_location(&state.page_router.state().location)
            .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
        Self::select_gallery_page_in_models(app, state, next_page.clone());

        apply_page_router_update_side_effects(
            app,
            window,
            next_page.clone(),
            &mut state.page_router,
            Ok(update),
        );

        let _ = app.models_mut().update(&state.last_action, |v| {
            *v = Arc::<str>::from(format!("gallery.page_history.sync({})", next_page.as_ref()));
        });
    }

    pub(crate) fn handle_nav_command(
        app: &mut App,
        state: &mut UiGalleryWindowState,
        window: AppWindowId,
        command: &CommandId,
    ) -> bool {
        if matches!(
            command.as_str(),
            CMD_GALLERY_PAGE_BACK | CMD_GALLERY_PAGE_FORWARD
        ) {
            let action = if command.as_str() == CMD_GALLERY_PAGE_BACK {
                NavigationAction::Back
            } else {
                NavigationAction::Forward
            };
            let update = state
                .page_router
                .navigate_with_prefetch_intents(action, None);

            let next_page = page_from_gallery_location(&state.page_router.state().location)
                .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
            Self::select_gallery_page_in_models(app, state, next_page.clone());

            apply_page_router_update_side_effects(
                app,
                window,
                next_page.clone(),
                &mut state.page_router,
                update,
            );

            let _ = app.models_mut().update(&state.last_action, |v| {
                *v = Arc::<str>::from(format!(
                    "gallery.page_history.{}({})",
                    action,
                    next_page.as_ref()
                ));
            });

            return true;
        }

        let Some(page) = page_id_for_nav_command(command.as_str()) else {
            return false;
        };

        let page: Arc<str> = Arc::from(page);
        Self::navigate_to_gallery_page(app, state, window, page, NavigationAction::Push);
        true
    }

    pub(crate) fn handle_workspace_tab_command(
        app: &mut App,
        state: &mut UiGalleryWindowState,
        window: AppWindowId,
        command: &CommandId,
    ) -> bool {
        let close_tab_by_id = |app: &mut App,
                               state: &mut UiGalleryWindowState,
                               tab_id: Arc<str>|
         -> bool {
            let selected = app
                .models()
                .get_cloned(&state.selected_page)
                .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));

            let mut closed = false;
            let mut next_selected: Option<Arc<str>> = None;

            let _ = app.models_mut().update(&state.workspace_tabs, |tabs| {
                let Some(index) = tabs.iter().position(|t| t.as_ref() == tab_id.as_ref()) else {
                    return;
                };
                if tabs.len() <= 1 {
                    return;
                }

                tabs.remove(index);
                closed = true;

                if selected.as_ref() == tab_id.as_ref() {
                    let next_index = index.min(tabs.len().saturating_sub(1));
                    next_selected = tabs.get(next_index).cloned();
                }
            });

            if !closed {
                return false;
            }

            let cmd = Self::workspace_tab_close_command(tab_id.as_ref());
            state.workspace_tab_close_by_command.remove(cmd.as_ref());

            let _ = app
                .models_mut()
                .update(&state.workspace_dirty_tabs, |dirty| {
                    dirty.retain(|t| t.as_ref() != tab_id.as_ref());
                });

            if let Some(next) = next_selected {
                Self::navigate_to_gallery_page(app, state, window, next, NavigationAction::Replace);
            } else {
                Self::sync_workspace_window_layout_from_models(app, state);
            }

            true
        };

        match command.as_str() {
            CMD_WORKSPACE_TAB_NEXT | CMD_WORKSPACE_TAB_PREV => {
                let selected = app
                    .models()
                    .get_cloned(&state.selected_page)
                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                let tabs = app
                    .models()
                    .get_cloned(&state.workspace_tabs)
                    .unwrap_or_default();
                if tabs.is_empty() {
                    return false;
                }
                let Some(index) = tabs.iter().position(|t| t.as_ref() == selected.as_ref()) else {
                    return false;
                };

                let next_index = if command.as_str() == CMD_WORKSPACE_TAB_NEXT {
                    (index + 1) % tabs.len()
                } else {
                    (index + tabs.len() - 1) % tabs.len()
                };
                if let Some(next) = tabs.get(next_index).cloned() {
                    Self::navigate_to_gallery_page(
                        app,
                        state,
                        window,
                        next,
                        NavigationAction::Replace,
                    );
                    return true;
                }
                false
            }
            CMD_WORKSPACE_TAB_CLOSE => {
                let selected = app
                    .models()
                    .get_cloned(&state.selected_page)
                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                close_tab_by_id(app, state, selected)
            }
            _ => {
                if let Some(tab_id) = state
                    .workspace_tab_close_by_command
                    .get(command.as_str())
                    .cloned()
                {
                    return close_tab_by_id(app, state, tab_id);
                }
                false
            }
        }
    }
}
