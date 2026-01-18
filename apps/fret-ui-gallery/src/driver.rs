use fret_app::{App, CommandId, CommandMeta, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Event, SemanticsRole, UiServices};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::action::{UiActionHost, UiActionHostAdapter};
use fret_ui::declarative;
use fret_ui::element::SemanticsProps;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{Invalidation, UiTree};
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use fret_workspace::commands::{
    CMD_WORKSPACE_TAB_CLOSE, CMD_WORKSPACE_TAB_CLOSE_PREFIX, CMD_WORKSPACE_TAB_NEXT,
    CMD_WORKSPACE_TAB_PREV,
};
use fret_workspace::{
    WorkspaceFrame, WorkspaceStatusBar, WorkspaceTab, WorkspaceTabStrip, WorkspaceTopBar,
};
use std::sync::Arc;
use std::time::Duration;
use time::Date;

#[cfg(not(target_arch = "wasm32"))]
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;

use crate::spec::*;
use crate::ui;

#[derive(Default)]
struct DebugHudState {
    last_tick: Option<fret_core::time::Instant>,
    ema_frame_time_us: Option<f64>,
}

impl DebugHudState {
    fn tick(&mut self, now: fret_core::time::Instant) -> Option<Duration> {
        let dt = self.last_tick.map(|prev| now.duration_since(prev));
        self.last_tick = Some(now);

        if let Some(dt) = dt {
            let sample = dt.as_micros() as f64;
            let alpha = 0.1;
            self.ema_frame_time_us = Some(match self.ema_frame_time_us {
                Some(prev) => prev * (1.0 - alpha) + sample * alpha,
                None => sample,
            });
        }

        dt
    }

    fn ema_fps(&self) -> Option<f64> {
        let us = self.ema_frame_time_us?;
        if us <= 0.0 {
            return None;
        }
        Some(1_000_000.0 / us)
    }
}

struct UiGalleryWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    debug_hud: DebugHudState,
    selected_page: Model<Arc<str>>,
    workspace_tabs: Model<Vec<Arc<str>>>,
    workspace_dirty_tabs: Model<Vec<Arc<str>>>,
    nav_query: Model<String>,
    content_tab: Model<Option<Arc<str>>>,
    theme_preset: Model<Option<Arc<str>>>,
    theme_preset_open: Model<bool>,
    applied_theme_preset: Option<Arc<str>>,
    view_cache_enabled: Model<bool>,
    view_cache_cache_shell: Model<bool>,
    view_cache_inner_enabled: Model<bool>,
    view_cache_popover_open: Model<bool>,
    view_cache_continuous: Model<bool>,
    view_cache_counter: Model<u64>,
    inspector_enabled: Model<bool>,
    inspector_last_pointer: Model<Option<fret_core::Point>>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    select_value: Model<Option<Arc<str>>>,
    select_open: Model<bool>,
    combobox_value: Model<Option<Arc<str>>>,
    combobox_open: Model<bool>,
    combobox_query: Model<String>,
    date_picker_open: Model<bool>,
    date_picker_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    date_picker_selected: Model<Option<Date>>,
    resizable_h_fractions: Model<Vec<f32>>,
    resizable_v_fractions: Model<Vec<f32>>,
    data_table_state: Model<fret_ui_headless::table::TableState>,
    data_grid_selected_row: Model<Option<u64>>,
    tabs_value: Model<Option<Arc<str>>>,
    accordion_value: Model<Option<Arc<str>>>,
    progress: Model<f32>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    text_input: Model<String>,
    text_area: Model<String>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
    virtual_list_torture_jump: Model<String>,
    virtual_list_torture_edit_row: Model<Option<u64>>,
    virtual_list_torture_edit_text: Model<String>,
    virtual_list_torture_scroll: VirtualListScrollHandle,
}

#[derive(Default)]
struct UiGalleryDriver;

impl UiGalleryDriver {
    fn compute_inspector_status(
        app: &mut App,
        ui: &UiTree<App>,
        window: AppWindowId,
        pointer: Option<fret_core::Point>,
    ) -> (Arc<str>, Arc<str>, Arc<str>) {
        let hit = pointer.map(|p| ui.debug_hit_test(p));
        let hit_node = hit.as_ref().and_then(|h| h.hit);
        let hit_layers = hit
            .as_ref()
            .map(|h| h.active_layer_roots.len())
            .unwrap_or(0);
        let hit_barrier = hit.as_ref().and_then(|h| h.barrier_root);

        let (focused_node, focused_element, hovered_pressable, pressed_pressable) = app
            .with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                let state = runtime.diagnostics_snapshot(window);
                (
                    ui.focus(),
                    state.as_ref().and_then(|s| s.focused_element),
                    state.as_ref().and_then(|s| s.hovered_pressable),
                    state.as_ref().and_then(|s| s.pressed_pressable),
                )
            });

        let hit_element = hit_node.and_then(|node| {
            app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                runtime.element_for_node(window, node)
            })
        });

        let hit_path = hit_element.and_then(|element| {
            app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                runtime.debug_path_for_element(window, element)
            })
        });
        let focused_path = focused_element.and_then(|element| {
            app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                runtime.debug_path_for_element(window, element)
            })
        });

        let cursor = if let Some(pos) = pointer {
            Arc::<str>::from(format!("cursor=({:.1},{:.1})", pos.x.0, pos.y.0))
        } else {
            Arc::<str>::from("cursor=<none>")
        };

        let hit = Arc::<str>::from(format!(
            "hit={:?} el={} layers={} barrier={:?} {}",
            hit_node,
            hit_element
                .map(|id| format!("{:#x}", id.0))
                .unwrap_or_else(|| "<none>".to_string()),
            hit_layers,
            hit_barrier,
            hit_path.as_deref().unwrap_or(""),
        ));

        let focus = Arc::<str>::from(format!(
            "focus={:?} el={} hovered={} pressed={} {}",
            focused_node,
            focused_element
                .map(|id| format!("{:#x}", id.0))
                .unwrap_or_else(|| "<none>".to_string()),
            hovered_pressable
                .map(|id| format!("{:#x}", id.0))
                .unwrap_or_else(|| "<none>".to_string()),
            pressed_pressable
                .map(|id| format!("{:#x}", id.0))
                .unwrap_or_else(|| "<none>".to_string()),
            focused_path.as_deref().unwrap_or(""),
        ));

        (cursor, hit, focus)
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> UiGalleryWindowState {
        let start_page = ui_gallery_start_page().unwrap_or_else(|| {
            if std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_DIAG_DIR").is_some_and(|v| !v.is_empty())
            {
                Arc::<str>::from(PAGE_OVERLAY)
            } else {
                Arc::<str>::from(PAGE_INTRO)
            }
        });
        let selected_page = app.models_mut().insert(start_page.clone());

        let mut workspace_tabs_init = vec![
            Arc::<str>::from(PAGE_INTRO),
            Arc::<str>::from(PAGE_LAYOUT),
            Arc::<str>::from(PAGE_VIEW_CACHE),
            Arc::<str>::from(PAGE_BUTTON),
            Arc::<str>::from(PAGE_CARD),
            Arc::<str>::from(PAGE_BADGE),
            Arc::<str>::from(PAGE_AVATAR),
            Arc::<str>::from(PAGE_ICONS),
            Arc::<str>::from(PAGE_FIELD),
            Arc::<str>::from(PAGE_OVERLAY),
            Arc::<str>::from(PAGE_COMMAND),
        ];
        if !workspace_tabs_init
            .iter()
            .any(|page| page.as_ref() == start_page.as_ref())
        {
            workspace_tabs_init.push(start_page);
        }
        let workspace_tabs = app.models_mut().insert(workspace_tabs_init);
        let workspace_dirty_tabs = app
            .models_mut()
            .insert(vec![Arc::<str>::from(PAGE_OVERLAY)]);
        let nav_query = app.models_mut().insert(String::new());
        let content_tab = app.models_mut().insert(Some(Arc::<str>::from("preview")));
        let theme_preset = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("zinc/light")));
        let theme_preset_open = app.models_mut().insert(false);
        let popover_open = app.models_mut().insert(false);
        let dialog_open = app.models_mut().insert(false);
        let alert_dialog_open = app.models_mut().insert(false);
        let sheet_open = app.models_mut().insert(false);
        let select_value = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("apple")));
        let select_open = app.models_mut().insert(false);
        let combobox_value = app.models_mut().insert(None::<Arc<str>>);
        let combobox_open = app.models_mut().insert(false);
        let combobox_query = app.models_mut().insert(String::new());

        let date_picker_open = app.models_mut().insert(false);
        let today = time::OffsetDateTime::now_utc().date();
        let date_picker_month = app
            .models_mut()
            .insert(fret_ui_headless::calendar::CalendarMonth::from_date(today));
        let date_picker_selected = app.models_mut().insert(None::<Date>);

        let resizable_h_fractions = app.models_mut().insert(vec![0.3, 0.7]);
        let resizable_v_fractions = app.models_mut().insert(vec![0.5, 0.5]);

        let data_table_state = app
            .models_mut()
            .insert(fret_ui_headless::table::TableState::default());
        let data_grid_selected_row = app.models_mut().insert(None::<u64>);
        let tabs_value = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("overview")));
        let accordion_value = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("item-1")));
        let progress = app.models_mut().insert(35.0f32);
        let checkbox = app.models_mut().insert(false);
        let switch = app.models_mut().insert(true);
        let text_input = app.models_mut().insert(String::new());
        let text_area = app.models_mut().insert(String::new());
        let dropdown_open = app.models_mut().insert(false);
        let context_menu_open = app.models_mut().insert(false);
        let cmdk_open = app.models_mut().insert(false);
        let cmdk_query = app.models_mut().insert(String::new());
        let last_action = app.models_mut().insert(Arc::<str>::from("<none>"));
        let virtual_list_torture_jump = app.models_mut().insert(String::from("9000"));
        let virtual_list_torture_edit_row = app.models_mut().insert(None::<u64>);
        let virtual_list_torture_edit_text = app.models_mut().insert(String::new());
        let virtual_list_torture_scroll = VirtualListScrollHandle::new();

        let view_cache_enabled = app
            .models_mut()
            .insert(std::env::var_os("FRET_UI_GALLERY_VIEW_CACHE").is_some_and(|v| !v.is_empty()));
        let view_cache_cache_shell = app.models_mut().insert(false);
        let view_cache_inner_enabled = app.models_mut().insert(true);
        let view_cache_popover_open = app.models_mut().insert(false);
        let view_cache_continuous = app.models_mut().insert(false);
        let view_cache_counter = app.models_mut().insert(0u64);

        let inspector_enabled = app.models_mut().insert(
            std::env::var_os("FRET_UI_GALLERY_INSPECTOR").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()),
        );
        let inspector_last_pointer = app.models_mut().insert(None::<fret_core::Point>);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(
            std::env::var_os("FRET_UI_GALLERY_VIEW_CACHE").is_some_and(|v| !v.is_empty()),
        );
        ui.set_debug_enabled(
            std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()),
        );

        UiGalleryWindowState {
            ui,
            root: None,
            debug_hud: DebugHudState::default(),
            selected_page,
            workspace_tabs,
            workspace_dirty_tabs,
            nav_query,
            content_tab,
            theme_preset,
            theme_preset_open,
            applied_theme_preset: None,
            view_cache_enabled,
            view_cache_cache_shell,
            view_cache_inner_enabled,
            view_cache_popover_open,
            view_cache_continuous,
            view_cache_counter,
            inspector_enabled,
            inspector_last_pointer,
            popover_open,
            dialog_open,
            alert_dialog_open,
            sheet_open,
            select_value,
            select_open,
            combobox_value,
            combobox_open,
            combobox_query,
            date_picker_open,
            date_picker_month,
            date_picker_selected,
            resizable_h_fractions,
            resizable_v_fractions,
            data_table_state,
            data_grid_selected_row,
            tabs_value,
            accordion_value,
            progress,
            checkbox,
            switch,
            text_input,
            text_area,
            dropdown_open,
            context_menu_open,
            cmdk_open,
            cmdk_query,
            last_action,
            virtual_list_torture_jump,
            virtual_list_torture_edit_row,
            virtual_list_torture_edit_text,
            virtual_list_torture_scroll,
        }
    }

    fn handle_nav_command(
        app: &mut App,
        state: &UiGalleryWindowState,
        command: &CommandId,
    ) -> bool {
        let Some(page) = command.as_str().strip_prefix(CMD_NAV_SELECT_PREFIX) else {
            return false;
        };

        let page: Arc<str> = Arc::from(page);
        let page_for_tabs = page.clone();
        let _ = app.models_mut().update(&state.selected_page, |v| *v = page);
        let _ = app.models_mut().update(&state.workspace_tabs, |tabs| {
            if !tabs.iter().any(|t| t.as_ref() == page_for_tabs.as_ref()) {
                tabs.push(page_for_tabs);
            }
        });
        true
    }

    fn handle_workspace_tab_command(
        app: &mut App,
        state: &UiGalleryWindowState,
        command: &CommandId,
    ) -> bool {
        let close_tab_by_id = |app: &mut App, tab_id: Arc<str>| -> bool {
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

            let _ = app
                .models_mut()
                .update(&state.workspace_dirty_tabs, |dirty| {
                    dirty.retain(|t| t.as_ref() != tab_id.as_ref());
                });

            if let Some(next) = next_selected {
                let _ = app.models_mut().update(&state.selected_page, |v| *v = next);
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
                    let _ = app.models_mut().update(&state.selected_page, |v| *v = next);
                    return true;
                }
                false
            }
            CMD_WORKSPACE_TAB_CLOSE => {
                let selected = app
                    .models()
                    .get_cloned(&state.selected_page)
                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                close_tab_by_id(app, selected)
            }
            _ => {
                if let Some(suffix) = command
                    .as_str()
                    .strip_prefix(CMD_WORKSPACE_TAB_CLOSE_PREFIX)
                {
                    let suffix = suffix.trim();
                    if suffix.is_empty() {
                        return false;
                    }
                    return close_tab_by_id(app, Arc::<str>::from(suffix));
                }
                false
            }
        }
    }

    fn handle_gallery_command(
        app: &mut App,
        state: &UiGalleryWindowState,
        command: &CommandId,
    ) -> bool {
        match command.as_str() {
            CMD_PROGRESS_INC => {
                let _ = app
                    .models_mut()
                    .update(&state.progress, |v| *v = (*v + 10.0).min(100.0));
            }
            CMD_PROGRESS_DEC => {
                let _ = app
                    .models_mut()
                    .update(&state.progress, |v| *v = (*v - 10.0).max(0.0));
            }
            CMD_PROGRESS_RESET => {
                let _ = app.models_mut().update(&state.progress, |v| *v = 35.0);
            }
            CMD_VIEW_CACHE_BUMP => {
                let _ = app
                    .models_mut()
                    .update(&state.view_cache_counter, |v| *v = v.saturating_add(1));
            }
            CMD_VIEW_CACHE_RESET => {
                let _ = app
                    .models_mut()
                    .update(&state.view_cache_counter, |v| *v = 0);
            }
            _ => return false,
        }
        true
    }

    fn sync_shadcn_theme(app: &mut App, state: &mut UiGalleryWindowState) {
        let preset = app.models().get_cloned(&state.theme_preset).flatten();
        if preset.as_deref() == state.applied_theme_preset.as_deref() {
            return;
        }

        let Some(preset) = preset else {
            return;
        };

        let Some((base, scheme)) = preset.split_once('/') else {
            return;
        };

        let base = match base {
            "neutral" => shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
            "zinc" => shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
            "slate" => shadcn::shadcn_themes::ShadcnBaseColor::Slate,
            "stone" => shadcn::shadcn_themes::ShadcnBaseColor::Stone,
            "gray" => shadcn::shadcn_themes::ShadcnBaseColor::Gray,
            _ => return,
        };

        let scheme = match scheme {
            "light" => shadcn::shadcn_themes::ShadcnColorScheme::Light,
            "dark" => shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            _ => return,
        };

        shadcn::shadcn_themes::apply_shadcn_new_york_v4(app, base, scheme);
        state.applied_theme_preset = Some(preset);
    }

    fn render_ui(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut UiGalleryWindowState,
        bounds: fret_core::Rect,
    ) {
        OverlayController::begin_frame(app, window);
        let bisect = ui_gallery_bisect_flags();

        let cache_enabled = app
            .models()
            .get_copied(&state.view_cache_enabled)
            .unwrap_or(false);
        let cache_shell = app
            .models()
            .get_copied(&state.view_cache_cache_shell)
            .unwrap_or(false);

        if state.ui.view_cache_enabled() != cache_enabled {
            state.ui.set_view_cache_enabled(cache_enabled);
            if let Some(root) = state.root {
                state.ui.invalidate(root, Invalidation::Layout);
            }
        }

        let selected_page = state.selected_page.clone();
        let workspace_tabs = state.workspace_tabs.clone();
        let workspace_dirty_tabs = state.workspace_dirty_tabs.clone();
        let nav_query = state.nav_query.clone();
        let content_tab = state.content_tab.clone();
        let theme_preset = state.theme_preset.clone();
        let theme_preset_open = state.theme_preset_open.clone();
        let view_cache_enabled = state.view_cache_enabled.clone();
        let view_cache_cache_shell = state.view_cache_cache_shell.clone();
        let view_cache_inner_enabled = state.view_cache_inner_enabled.clone();
        let view_cache_popover_open = state.view_cache_popover_open.clone();
        let view_cache_continuous = state.view_cache_continuous.clone();
        let view_cache_counter = state.view_cache_counter.clone();
        let popover_open = state.popover_open.clone();
        let dialog_open = state.dialog_open.clone();
        let alert_dialog_open = state.alert_dialog_open.clone();
        let sheet_open = state.sheet_open.clone();
        let select_value = state.select_value.clone();
        let select_open = state.select_open.clone();
        let combobox_value = state.combobox_value.clone();
        let combobox_open = state.combobox_open.clone();
        let combobox_query = state.combobox_query.clone();
        let date_picker_open = state.date_picker_open.clone();
        let date_picker_month = state.date_picker_month.clone();
        let date_picker_selected = state.date_picker_selected.clone();
        let resizable_h_fractions = state.resizable_h_fractions.clone();
        let resizable_v_fractions = state.resizable_v_fractions.clone();
        let data_table_state = state.data_table_state.clone();
        let data_grid_selected_row = state.data_grid_selected_row.clone();
        let tabs_value = state.tabs_value.clone();
        let accordion_value = state.accordion_value.clone();
        let progress = state.progress.clone();
        let checkbox = state.checkbox.clone();
        let switch = state.switch.clone();
        let text_input = state.text_input.clone();
        let text_area = state.text_area.clone();
        let dropdown_open = state.dropdown_open.clone();
        let context_menu_open = state.context_menu_open.clone();
        let cmdk_open = state.cmdk_open.clone();
        let cmdk_query = state.cmdk_query.clone();
        let last_action = state.last_action.clone();
        let virtual_list_torture_jump = state.virtual_list_torture_jump.clone();
        let virtual_list_torture_edit_row = state.virtual_list_torture_edit_row.clone();
        let virtual_list_torture_edit_text = state.virtual_list_torture_edit_text.clone();
        let virtual_list_torture_scroll = state.virtual_list_torture_scroll.clone();
        let inspector_enabled = state.inspector_enabled.clone();
        let inspector_last_pointer = state.inspector_last_pointer.clone();

        let inspector_on = app.models().get_copied(&inspector_enabled).unwrap_or(false);
        let debug_on = inspector_on
            || std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
            || std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty());
        state.ui.set_debug_enabled(debug_on);
        if debug_on {
            app.request_redraw(window);
        }

        Self::sync_shadcn_theme(app, state);

        let last_debug_stats = state.ui.debug_stats();
        let frame_dt = if debug_on {
            state.debug_hud.tick(fret_core::time::Instant::now())
        } else {
            None
        };
        let fps = state.debug_hud.ema_fps();
        let last_cache_roots = state.ui.debug_cache_root_stats();
        let cache_root_breakdown: Option<Vec<Arc<str>>> = if !last_cache_roots.is_empty() {
            let total = last_cache_roots.len();
            let hits = last_cache_roots.iter().filter(|r| r.reused).count();
            let replayed_ops: u32 = last_cache_roots.iter().map(|r| r.paint_replayed_ops).sum();

            let mut lines: Vec<Arc<str>> = vec![Arc::from(format!(
                "cache_roots total={total} hits={hits} replayed_ops={replayed_ops}"
            ))];

            let max_items = 3usize;
            for (index, root) in last_cache_roots.iter().take(max_items).enumerate() {
                let element_path = root.element.and_then(|element| {
                    app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                        runtime.debug_path_for_element(window, element)
                    })
                });

                lines.push(Arc::from(format!(
                    "cache_root[{index}] node={:?} reused={} contained_layout={} replayed_ops={} el={} {}",
                    root.root,
                    root.reused as u8,
                    root.contained_layout as u8,
                    root.paint_replayed_ops,
                    root.element
                        .map(|id| format!("{:#x}", id.0))
                        .unwrap_or_else(|| "<none>".to_string()),
                    element_path.as_deref().unwrap_or(""),
                )));
            }

            Some(lines)
        } else {
            None
        };
        let hot_model_breakdown: Option<Arc<str>> = {
            let hotspots = state.ui.debug_model_change_hotspots();
            if hotspots.is_empty() {
                None
            } else {
                let mut line = String::from("hot_models");
                for hs in hotspots.iter().take(3) {
                    line.push(' ');
                    line.push_str(&format!("{:?}={}", hs.model, hs.observation_edges));
                }
                Some(Arc::from(line))
            }
        };
        let unobserved_model_breakdown: Option<Arc<str>> = {
            let unobserved = state.ui.debug_model_change_unobserved();
            if unobserved.is_empty() {
                None
            } else {
                let mut line = format!(
                    "unobs_models={}",
                    state.ui.debug_stats().model_change_unobserved_models
                );
                for entry in unobserved.iter().take(3) {
                    let type_name = entry.created.map(|c| c.type_name).unwrap_or("<unknown>");
                    let type_name = type_name.rsplit("::").next().unwrap_or(type_name);
                    line.push(' ');
                    line.push_str(&format!("{:?}={}", entry.model, type_name));
                }
                Some(Arc::from(line))
            }
        };

        let show_debug_hud = debug_on;
        let debug_hud_lines: Vec<Arc<str>> = if show_debug_hud {
            let mut lines: Vec<Arc<str>> = Vec::new();

            lines.push(Arc::from(format!(
                "fps={:.1} frame_dt_ms={:.2} solve_us={}",
                fps.unwrap_or(0.0),
                frame_dt.map(|dt| dt.as_secs_f64() * 1000.0).unwrap_or(0.0),
                last_debug_stats.layout_engine_solve_time.as_micros()
            )));
            lines.push(Arc::from(format!(
                "frame={:?} layout_us={} paint_us={} layout_nodes={}/{} paint_nodes={}/{}",
                last_debug_stats.frame_id,
                last_debug_stats.layout_time.as_micros(),
                last_debug_stats.paint_time.as_micros(),
                last_debug_stats.layout_nodes_performed,
                last_debug_stats.layout_nodes_visited,
                last_debug_stats.paint_nodes_performed,
                last_debug_stats.paint_nodes,
            )));
            lines.push(Arc::from(format!(
                "paint_cache hits={} misses={} replayed_ops={}",
                last_debug_stats.paint_cache_hits,
                last_debug_stats.paint_cache_misses,
                last_debug_stats.paint_cache_replayed_ops
            )));
            lines.push(Arc::from(format!(
                "view_cache active={} trunc={} relayouts={}",
                last_debug_stats.view_cache_active as u8,
                last_debug_stats.view_cache_invalidation_truncations,
                last_debug_stats.view_cache_contained_relayouts
            )));
            lines.push(Arc::from(format!(
                "changes models={} edges={} roots={} walks={} nodes={}",
                last_debug_stats.model_change_models,
                last_debug_stats.model_change_observation_edges,
                last_debug_stats.model_change_invalidation_roots,
                last_debug_stats.invalidation_walk_calls_model_change,
                last_debug_stats.invalidation_walk_nodes_model_change
            )));
            lines.push(Arc::from(format!(
                "globals count={} edges={} roots={} walks={} nodes={}",
                last_debug_stats.global_change_globals,
                last_debug_stats.global_change_observation_edges,
                last_debug_stats.global_change_invalidation_roots,
                last_debug_stats.invalidation_walk_calls_global_change,
                last_debug_stats.invalidation_walk_nodes_global_change
            )));

            if let Some(extra) = cache_root_breakdown.as_ref() {
                lines.extend(extra.iter().cloned());
            }
            if let Some(line) = hot_model_breakdown.as_ref() {
                lines.push(line.clone());
            }
            if let Some(line) = unobserved_model_breakdown.as_ref() {
                lines.push(line.clone());
            }

            lines
        } else {
            Vec::new()
        };
        let inspector_status = if app.models().get_copied(&inspector_enabled).unwrap_or(false) {
            let pointer = app
                .models()
                .get_copied(&inspector_last_pointer)
                .unwrap_or(None);
            Some(Self::compute_inspector_status(
                app, &state.ui, window, pointer,
            ))
        } else {
            None
        };

        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("fret-ui-gallery", |cx| {
                    if (bisect & BISECT_MINIMAL_ROOT) != 0 {
                        return vec![cx.text("Hello, fret-ui-gallery")];
                    }

                    let theme = cx.theme().clone();

                    let sidebar = if cache_shell {
                        cx.view_cache(
                            {
                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.size.width = fret_ui::element::Length::Px(Px(280.0));
                                layout.size.height = fret_ui::element::Length::Fill;
                                fret_ui::element::ViewCacheProps {
                                    layout,
                                    ..Default::default()
                                }
                            },
                            |cx| {
                                let selected = cx
                                    .get_model_cloned(&selected_page, Invalidation::Layout)
                                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                                let query = cx
                                    .get_model_cloned(&nav_query, Invalidation::Layout)
                                    .unwrap_or_default();

                                vec![if (bisect & BISECT_SIMPLE_SIDEBAR) != 0 {
                                    cx.container(
                                        decl_style::container_props(
                                            &theme,
                                            ChromeRefinement::default()
                                                .bg(ColorRef::Color(theme.color_required("muted")))
                                                .p(Space::N4),
                                            LayoutRefinement::default()
                                                .w_px(MetricRef::Px(Px(280.0)))
                                                .h_full(),
                                        ),
                                        |cx| vec![cx.text("Sidebar (disabled)")],
                                    )
                                } else {
                                    ui::sidebar_view(
                                        cx,
                                        &theme,
                                        selected.as_ref(),
                                        query.as_str(),
                                        nav_query.clone(),
                                        selected_page.clone(),
                                        state.workspace_tabs.clone(),
                                    )
                                }]
                            },
                        )
                    } else {
                        cx.keyed("ui_gallery.sidebar", |cx| {
                            let selected = cx
                                .get_model_cloned(&selected_page, Invalidation::Layout)
                                .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                            let query = cx
                                .get_model_cloned(&nav_query, Invalidation::Layout)
                                .unwrap_or_default();

                            if (bisect & BISECT_SIMPLE_SIDEBAR) != 0 {
                                cx.container(
                                    decl_style::container_props(
                                        &theme,
                                        ChromeRefinement::default()
                                            .bg(ColorRef::Color(theme.color_required("muted")))
                                            .p(Space::N4),
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(Px(280.0)))
                                            .h_full(),
                                    ),
                                    |cx| vec![cx.text("Sidebar (disabled)")],
                                )
                            } else {
                                ui::sidebar_view(
                                    cx,
                                    &theme,
                                    selected.as_ref(),
                                    query.as_str(),
                                    nav_query.clone(),
                                    selected_page.clone(),
                                    state.workspace_tabs.clone(),
                                )
                            }
                        })
                    };

                    let content = if cache_shell {
                        cx.view_cache(
                            {
                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.size.width = fret_ui::element::Length::Fill;
                                layout.size.height = fret_ui::element::Length::Fill;
                                layout.flex.grow = 1.0;
                                fret_ui::element::ViewCacheProps {
                                    layout,
                                    ..Default::default()
                                }
                            },
                            |cx| {
                                let selected = cx
                                    .get_model_cloned(&selected_page, Invalidation::Layout)
                                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));

                                vec![cx.keyed(("ui_gallery.content", selected.as_ref()), |cx| {
                                    if (bisect & BISECT_SIMPLE_CONTENT) != 0 {
                                        cx.container(
                                            decl_style::container_props(
                                                &theme,
                                                ChromeRefinement::default()
                                                    .bg(ColorRef::Color(
                                                        theme.color_required("background"),
                                                    ))
                                                    .p(Space::N6),
                                                LayoutRefinement::default().w_full().h_full(),
                                            ),
                                            |cx| vec![cx.text("Content (disabled)")],
                                        )
                                    } else {
                                        ui::content_view(
                                            cx,
                                            &theme,
                                            selected.as_ref(),
                                            content_tab.clone(),
                                            theme_preset.clone(),
                                            theme_preset_open.clone(),
                                            view_cache_enabled.clone(),
                                            view_cache_cache_shell.clone(),
                                            view_cache_inner_enabled.clone(),
                                            view_cache_popover_open.clone(),
                                            view_cache_continuous.clone(),
                                            view_cache_counter.clone(),
                                            popover_open.clone(),
                                            dialog_open.clone(),
                                            alert_dialog_open.clone(),
                                            sheet_open.clone(),
                                            select_value.clone(),
                                            select_open.clone(),
                                            combobox_value.clone(),
                                            combobox_open.clone(),
                                            combobox_query.clone(),
                                            date_picker_open.clone(),
                                            date_picker_month.clone(),
                                            date_picker_selected.clone(),
                                            resizable_h_fractions.clone(),
                                            resizable_v_fractions.clone(),
                                            data_table_state.clone(),
                                            data_grid_selected_row.clone(),
                                            tabs_value.clone(),
                                            accordion_value.clone(),
                                            progress.clone(),
                                            checkbox.clone(),
                                            switch.clone(),
                                            text_input.clone(),
                                            text_area.clone(),
                                            dropdown_open.clone(),
                                            context_menu_open.clone(),
                                            cmdk_open.clone(),
                                            cmdk_query.clone(),
                                            last_action.clone(),
                                            virtual_list_torture_jump.clone(),
                                            virtual_list_torture_edit_row.clone(),
                                            virtual_list_torture_edit_text.clone(),
                                            virtual_list_torture_scroll.clone(),
                                        )
                                    }
                                })]
                            },
                        )
                    } else {
                        cx.keyed("ui_gallery.content_root", |cx| {
                            let selected = cx
                                .get_model_cloned(&selected_page, Invalidation::Layout)
                                .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));

                            cx.keyed(("ui_gallery.content", selected.as_ref()), |cx| {
                                if (bisect & BISECT_SIMPLE_CONTENT) != 0 {
                                    cx.container(
                                        decl_style::container_props(
                                            &theme,
                                            ChromeRefinement::default()
                                                .bg(ColorRef::Color(
                                                    theme.color_required("background"),
                                                ))
                                                .p(Space::N6),
                                            LayoutRefinement::default().w_full().h_full(),
                                        ),
                                        |cx| vec![cx.text("Content (disabled)")],
                                    )
                                } else {
                                    ui::content_view(
                                        cx,
                                        &theme,
                                        selected.as_ref(),
                                        content_tab.clone(),
                                        theme_preset.clone(),
                                        theme_preset_open.clone(),
                                        view_cache_enabled.clone(),
                                        view_cache_cache_shell.clone(),
                                        view_cache_inner_enabled.clone(),
                                        view_cache_popover_open.clone(),
                                        view_cache_continuous.clone(),
                                        view_cache_counter.clone(),
                                        popover_open.clone(),
                                        dialog_open.clone(),
                                        alert_dialog_open.clone(),
                                        sheet_open.clone(),
                                        select_value.clone(),
                                        select_open.clone(),
                                        combobox_value.clone(),
                                        combobox_open.clone(),
                                        combobox_query.clone(),
                                        date_picker_open.clone(),
                                        date_picker_month.clone(),
                                        date_picker_selected.clone(),
                                        resizable_h_fractions.clone(),
                                        resizable_v_fractions.clone(),
                                        data_table_state.clone(),
                                        data_grid_selected_row.clone(),
                                        tabs_value.clone(),
                                        accordion_value.clone(),
                                        progress.clone(),
                                        checkbox.clone(),
                                        switch.clone(),
                                        text_input.clone(),
                                        text_area.clone(),
                                        dropdown_open.clone(),
                                        context_menu_open.clone(),
                                        cmdk_open.clone(),
                                        cmdk_query.clone(),
                                        last_action.clone(),
                                        virtual_list_torture_jump.clone(),
                                        virtual_list_torture_edit_row.clone(),
                                        virtual_list_torture_edit_text.clone(),
                                        virtual_list_torture_scroll.clone(),
                                    )
                                }
                            })
                        })
                    };

                    let menubar = shadcn::Menubar::new(vec![
                        shadcn::MenubarMenu::new("File")
                            .test_id("ui-gallery-menubar-file")
                            .entries(vec![shadcn::MenubarEntry::Group(
                                shadcn::MenubarGroup::new(vec![
                                    shadcn::MenubarEntry::Item(
                                        shadcn::MenubarItem::new("Open")
                                            .test_id("ui-gallery-menubar-open")
                                            .on_select(CMD_APP_OPEN),
                                    ),
                                    shadcn::MenubarEntry::Item(
                                        shadcn::MenubarItem::new("Save").on_select(CMD_APP_SAVE),
                                    ),
                                    shadcn::MenubarEntry::Item(
                                        shadcn::MenubarItem::new("Settings")
                                            .on_select(CMD_APP_SETTINGS),
                                    ),
                                ]),
                            )]),
                        shadcn::MenubarMenu::new("View").entries(vec![
                            shadcn::MenubarEntry::Group(shadcn::MenubarGroup::new(vec![
                                shadcn::MenubarEntry::Item(
                                    shadcn::MenubarItem::new("Command Palette")
                                        .on_select(fret_app::core_commands::COMMAND_PALETTE),
                                ),
                                shadcn::MenubarEntry::Separator,
                                shadcn::MenubarEntry::Item(
                                    shadcn::MenubarItem::new("Toast: Default")
                                        .on_select(CMD_TOAST_DEFAULT),
                                ),
                            ])),
                        ]),
                    ])
                    .into_element(cx);

                    let tab_strip = cx.keyed("ui_gallery.tab_strip", |cx| {
                        if (bisect & BISECT_DISABLE_TAB_STRIP) != 0 {
                            return cx.text("Tabs (disabled)");
                        }

                        let selected = cx
                            .get_model_cloned(&selected_page, Invalidation::Layout)
                            .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                        let workspace_tab_ids = cx
                            .get_model_cloned(&workspace_tabs, Invalidation::Layout)
                            .unwrap_or_default();
                        let workspace_dirty_ids = cx
                            .get_model_cloned(&workspace_dirty_tabs, Invalidation::Layout)
                            .unwrap_or_default();

                        WorkspaceTabStrip::new(selected.clone())
                            .tabs(workspace_tab_ids.iter().map(|tab_id| {
                                let (title, _origin, _docs, _usage) =
                                    crate::spec::page_meta(tab_id.as_ref());
                                let dirty = workspace_dirty_ids
                                    .iter()
                                    .any(|d| d.as_ref() == tab_id.as_ref());
                                WorkspaceTab::new(
                                    tab_id.clone(),
                                    title,
                                    CommandId::new(format!(
                                        "{}{}",
                                        CMD_NAV_SELECT_PREFIX,
                                        tab_id.as_ref()
                                    )),
                                )
                                .close_command(CommandId::new(format!(
                                    "{}{}",
                                    CMD_WORKSPACE_TAB_CLOSE_PREFIX,
                                    tab_id.as_ref()
                                )))
                                .dirty(dirty)
                            }))
                            .into_element(cx)
                    });

                    let top_bar = WorkspaceTopBar::new()
                        .left(vec![menubar])
                        .center(vec![tab_strip])
                        .right(vec![
                            shadcn::Button::new("Command palette")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_click(fret_app::core_commands::COMMAND_PALETTE)
                                .into_element(cx),
                        ])
                        .into_element(cx);

                    let status_bar = cx.keyed("ui_gallery.status_bar", |cx| {
                        let status_last_action = cx
                            .get_model_cloned(&last_action, Invalidation::Layout)
                            .unwrap_or_else(|| Arc::<str>::from("<none>"));
                        let status_theme = cx
                            .get_model_cloned(&theme_preset, Invalidation::Layout)
                            .flatten()
                            .unwrap_or_else(|| Arc::<str>::from("<default>"));
                        let status_view_cache = cx
                            .get_model_copied(&view_cache_enabled, Invalidation::Layout)
                            .unwrap_or(false);
                        let status_cache_shell = cx
                            .get_model_copied(&view_cache_cache_shell, Invalidation::Layout)
                            .unwrap_or(false);

                        let mut right_items: Vec<AnyElement> = vec![cx.text(format!(
                            "theme: {} view_cache={} shell_cache={} layout_us={} paint_us={}",
                            status_theme.as_ref(),
                            status_view_cache as u8,
                            status_cache_shell as u8,
                            last_debug_stats.layout_time.as_micros(),
                            last_debug_stats.paint_time.as_micros()
                        ))];
                        if let Some((cursor, hit, focus)) = inspector_status.as_ref() {
                            right_items.push(cx.text(format!("inspect: {}", cursor.as_ref())));
                            right_items.push(cx.text(format!("inspect: {}", hit.as_ref())));
                            right_items.push(cx.text(format!("inspect: {}", focus.as_ref())));
                        }

                        WorkspaceStatusBar::new()
                            .left(vec![cx.text(format!(
                                "last action: {}",
                                status_last_action.as_ref()
                            ))])
                            .right(right_items)
                            .into_element(cx)
                    });

                    let mut center_layout = fret_ui::element::LayoutStyle::default();
                    center_layout.size.width = fret_ui::element::Length::Fill;
                    center_layout.size.height = fret_ui::element::Length::Fill;
                    center_layout.flex.grow = 1.0;

                    let center = cx.flex(
                        fret_ui::element::FlexProps {
                            layout: center_layout,
                            direction: fret_core::Axis::Horizontal,
                            ..Default::default()
                        },
                        |_cx| vec![sidebar, content],
                    );

                    let frame = WorkspaceFrame::new(center)
                        .top(top_bar)
                        .bottom(status_bar)
                        .into_element(cx);

                    let mut content: Vec<AnyElement> = vec![
                        cx.semantics(
                            SemanticsProps {
                                role: SemanticsRole::Panel,
                                label: Some(Arc::from("fret-ui-gallery")),
                                ..Default::default()
                            },
                            |_cx| vec![frame],
                        ),
                        if (bisect & BISECT_DISABLE_TOASTER) != 0 {
                            cx.text("")
                        } else {
                            shadcn::Toaster::new().into_element(cx)
                        },
                    ];
                    if show_debug_hud {
                        let debug_hud_lines = debug_hud_lines.clone();
                        content.push(cx.keyed("ui_gallery.debug_hud", |cx| {
                            let mut hud_layout = fret_ui::element::LayoutStyle::default();
                            hud_layout.position = fret_ui::element::PositionStyle::Absolute;
                            hud_layout.inset.top = Some(Px(8.0));
                            hud_layout.inset.right = Some(Px(8.0));
                            hud_layout.size.width = fret_ui::element::Length::Px(Px(520.0));
                            hud_layout.size.height = fret_ui::element::Length::Px(Px(220.0));

                            let mut gate = fret_ui::element::InteractivityGateProps::default();
                            gate.layout = hud_layout;
                            gate.present = true;
                            gate.interactive = false;

                            cx.interactivity_gate_props(gate, |cx| {
                                let mut container_props = decl_style::container_props(
                                    &theme,
                                    ChromeRefinement::default()
                                        .bg(ColorRef::Color(theme.color_required("background")))
                                        .border_1()
                                        .rounded(Radius::Md)
                                        .p(Space::N3),
                                    LayoutRefinement::default().w_full().h_full(),
                                );
                                container_props.layout.size.width = fret_ui::element::Length::Fill;
                                container_props.layout.size.height = fret_ui::element::Length::Fill;
                                container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                                let body = stack::vstack(
                                    cx,
                                    stack::VStackProps::default()
                                        .layout(LayoutRefinement::default().w_full())
                                        .gap(Space::N1),
                                    |cx| {
                                        debug_hud_lines
                                            .iter()
                                            .map(|line| {
                                                cx.text_props(TextProps {
                                                    layout: Default::default(),
                                                    text: line.clone(),
                                                    style: None,
                                                    color: Some(theme.color_required("foreground")),
                                                    wrap: TextWrap::Word,
                                                    overflow: TextOverflow::Clip,
                                                })
                                            })
                                            .collect()
                                    },
                                );

                                vec![cx.container(container_props, |cx| {
                                    vec![
                                        shadcn::ScrollArea::new(vec![body])
                                            .refine_layout(
                                                LayoutRefinement::default().w_full().h_full(),
                                            )
                                            .into_element(cx),
                                    ]
                                })]
                            })
                        }));
                    }

                    if cx
                        .get_model_copied(&inspector_enabled, Invalidation::Layout)
                        .unwrap_or(false)
                    {
                        cx.observe_model(&inspector_last_pointer, Invalidation::Paint);

                        let mut props = fret_ui::element::PointerRegionProps::default();
                        props.layout.size.width = fret_ui::element::Length::Fill;
                        props.layout.size.height = fret_ui::element::Length::Fill;

                        let on_pointer_move = {
                            let inspector_last_pointer = inspector_last_pointer.clone();
                            Arc::new(
                                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      cx: fret_ui::action::ActionCx,
                                      mv: fret_ui::action::PointerMoveCx| {
                                    let _ = host.models_mut().update(&inspector_last_pointer, |v| {
                                        *v = Some(mv.position);
                                    });
                                    host.request_redraw(cx.window);
                                    false
                                },
                            )
                        };
                        let on_pointer_down = {
                            let inspector_last_pointer = inspector_last_pointer.clone();
                            Arc::new(
                                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      cx: fret_ui::action::ActionCx,
                                      down: fret_ui::action::PointerDownCx| {
                                    let _ = host.models_mut().update(&inspector_last_pointer, |v| {
                                        *v = Some(down.position);
                                    });
                                    host.request_redraw(cx.window);
                                    false
                                },
                            )
                        };

                        vec![cx.pointer_region(props, |cx| {
                            cx.pointer_region_on_pointer_move(on_pointer_move);
                            cx.pointer_region_on_pointer_down(on_pointer_down);
                            content
                        })]
                    } else {
                        content
                    }
                });

        state.ui.set_root(root);
        if (bisect & BISECT_DISABLE_OVERLAY_CONTROLLER) == 0 {
            OverlayController::render(&mut state.ui, app, services, window, bounds);
        }
        state.root = Some(root);
    }
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
        shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    // Minimal command surface for `CommandDialog::new_with_host_commands`.
    app.commands_mut().register(
        CommandId::new(CMD_APP_OPEN),
        CommandMeta::new("Open")
            .with_category("File")
            .with_keywords(["open", "file"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_APP_SAVE),
        CommandMeta::new("Save")
            .with_category("File")
            .with_keywords(["save", "file"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_APP_SETTINGS),
        CommandMeta::new("Settings")
            .with_category("View")
            .with_keywords(["settings", "preferences"]),
    );

    for group in PAGE_GROUPS {
        for page in group.items {
            let mut keywords: Vec<&'static str> = Vec::new();
            keywords.push(page.id);
            keywords.push(page.origin);
            keywords.extend_from_slice(page.tags);

            app.commands_mut().register(
                CommandId::new(page.command),
                CommandMeta::new(page.label)
                    .with_category(format!("Gallery: {}", group.title))
                    .with_keywords(keywords),
            );
        }
    }

    app.commands_mut().register(
        CommandId::new(CMD_CLIPBOARD_COPY_LINK),
        CommandMeta::new("Copy page link")
            .with_category("Gallery")
            .with_keywords(["copy", "clipboard", "link", "page"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_CLIPBOARD_COPY_USAGE),
        CommandMeta::new("Copy usage")
            .with_category("Gallery")
            .with_keywords(["copy", "clipboard", "usage", "code"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_CLIPBOARD_COPY_NOTES),
        CommandMeta::new("Copy notes")
            .with_category("Gallery")
            .with_keywords(["copy", "clipboard", "notes", "docs"]),
    );

    fret_workspace::commands::register_workspace_commands(app.commands_mut());
    fret_app::install_command_default_keybindings_into_keymap(&mut app);

    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-ui-gallery".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(1080.0, 720.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    UiGalleryDriver::default()
}

#[cfg(test)]
mod stack_overflow_tests;

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();

    fret_bootstrap::BootstrapBuilder::new(app, build_driver())
        .configure(move |c| {
            *c = config;
        })
        .with_default_diagnostics()
        .with_default_config_files()?
        .with_lucide_icons()
        .preload_icon_svgs_on_gpu_ready()
        .run()
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}

impl WinitAppDriver for UiGalleryDriver {
    type WindowState = UiGalleryWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context
            .app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.record_model_changes(context.window, changed);
            });
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context
            .app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
                svc.record_global_changes(app, context.window, changed);
            });
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        if command.as_str() == fret_app::core_commands::COMMAND_PALETTE
            || command.as_str() == fret_app::core_commands::COMMAND_PALETTE_LEGACY
        {
            let _ = app.models_mut().update(&state.cmdk_open, |v| *v = true);
            let _ = app.models_mut().update(&state.cmdk_query, |v| v.clear());
            app.request_redraw(window);
            return;
        }

        if state.ui.dispatch_command(app, services, &command) {
            app.request_redraw(window);
            return;
        }

        if Self::handle_workspace_tab_command(app, state, &command) {
            app.request_redraw(window);
            return;
        }

        let did_nav = Self::handle_nav_command(app, state, &command);
        let did_gallery = Self::handle_gallery_command(app, state, &command);
        if did_nav || did_gallery {
            app.request_redraw(window);
        }

        if command.as_str() == CMD_VIRTUAL_LIST_TORTURE_JUMP {
            let raw = app
                .models()
                .get_cloned(&state.virtual_list_torture_jump)
                .unwrap_or_default();
            let index = raw.trim().parse::<usize>().unwrap_or(0);
            state
                .virtual_list_torture_scroll
                .scroll_to_item(index, fret_ui::scroll::ScrollStrategy::Start);
            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_VIRTUAL_LIST_TORTURE_SCROLL_BOTTOM {
            state.virtual_list_torture_scroll.scroll_to_bottom();
            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_VIRTUAL_LIST_TORTURE_CLEAR_EDIT {
            let _ = app
                .models_mut()
                .update(&state.virtual_list_torture_edit_row, |v| *v = None);
            let _ = app
                .models_mut()
                .update(&state.virtual_list_torture_edit_text, |v| v.clear());
            app.request_redraw(window);
            return;
        }

        if let Some(suffix) = command
            .as_str()
            .strip_prefix(CMD_VIRTUAL_LIST_TORTURE_ROW_EDIT_PREFIX)
        {
            if let Ok(row) = suffix.parse::<u64>() {
                let _ = app
                    .models_mut()
                    .update(&state.virtual_list_torture_edit_row, |v| *v = Some(row));
                let _ = app
                    .models_mut()
                    .update(&state.virtual_list_torture_edit_text, |v| {
                        *v = format!("Row {row}");
                    });
                app.request_redraw(window);
                return;
            }
        }

        if let Some(suffix) = command.as_str().strip_prefix(CMD_DATA_GRID_ROW_PREFIX) {
            if let Ok(row) = suffix.parse::<u64>() {
                let _ = app.models_mut().update(&state.data_grid_selected_row, |v| {
                    if *v == Some(row) {
                        *v = None;
                    } else {
                        *v = Some(row);
                    }
                });
                app.request_redraw(window);
                return;
            }
        }

        match command.as_str() {
            CMD_CLIPBOARD_COPY_LINK | CMD_CLIPBOARD_COPY_USAGE | CMD_CLIPBOARD_COPY_NOTES => {
                let selected = app
                    .models()
                    .get_cloned(&state.selected_page)
                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                let (_title, _origin, notes_md, usage_md) =
                    crate::spec::page_meta(selected.as_ref());

                let text = match command.as_str() {
                    CMD_CLIPBOARD_COPY_LINK => format!("?page={}", selected.as_ref()),
                    CMD_CLIPBOARD_COPY_USAGE => usage_md.to_string(),
                    CMD_CLIPBOARD_COPY_NOTES => notes_md.to_string(),
                    _ => String::new(),
                };

                app.push_effect(Effect::ClipboardSetText { text });

                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast_success_message(
                    &mut host,
                    window,
                    "Copied",
                    shadcn::ToastMessageOptions::new().description("Copied to clipboard."),
                );

                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("clipboard.copy");
                });
            }
            CMD_MENU_DROPDOWN_APPLE => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("menu.dropdown.apple");
                });
            }
            CMD_MENU_DROPDOWN_ORANGE => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("menu.dropdown.orange");
                });
            }
            CMD_MENU_CONTEXT_ACTION => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("menu.context.action");
                });
            }
            CMD_APP_OPEN => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("cmd.open");
                });
            }
            CMD_APP_SAVE => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("cmd.save");
                });
            }
            CMD_APP_SETTINGS => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("cmd.settings");
                });
            }
            CMD_TOAST_DEFAULT => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast_message(
                    &mut host,
                    window,
                    "Default toast",
                    shadcn::ToastMessageOptions::new().description("Hello from fret-ui-gallery."),
                );
                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.default");
                });
            }
            CMD_TOAST_SUCCESS => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast_success_message(
                    &mut host,
                    window,
                    "Success",
                    shadcn::ToastMessageOptions::new().description("Everything worked."),
                );
                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.success");
                });
            }
            CMD_TOAST_ERROR => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast_error_message(
                    &mut host,
                    window,
                    "Error",
                    shadcn::ToastMessageOptions::new().description("Something failed."),
                );
                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.error");
                });
            }
            CMD_TOAST_SHOW_ACTION_CANCEL => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast_message(
                    &mut host,
                    window,
                    "Action toast",
                    shadcn::ToastMessageOptions::new()
                        .description("Try the action/cancel buttons.")
                        .action("Undo", CMD_TOAST_ACTION)
                        .cancel("Cancel", CMD_TOAST_CANCEL)
                        .duration(Duration::from_secs(6)),
                );
                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.action_cancel");
                });
            }
            CMD_TOAST_ACTION => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.action");
                });
            }
            CMD_TOAST_CANCEL => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.cancel");
                });
            }
            _ => {}
        }

        app.request_redraw(window);
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let consumed =
                app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
                    if !svc.is_enabled() {
                        return false;
                    }
                    if svc.maybe_intercept_event_for_inspect_shortcuts(app, window, event) {
                        return true;
                    }
                    svc.maybe_intercept_event_for_picking(app, window, event)
                });
            if consumed {
                return;
            }
        }

        match event {
            Event::WindowCloseRequested => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
            }
        }
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        Self::render_ui(app, services, window, state, bounds);
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let inspection_active = app
                .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                    svc.wants_inspection_active(window)
                });
            state.ui.set_inspection_active(inspection_active);
        }

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let semantics_snapshot = state.ui.semantics_snapshot();
            let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
                let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
                svc.drive_script_for_window(window, semantics_snapshot, element_runtime)
            });

            if drive.request_redraw {
                app.request_redraw(window);
            }

            let mut injected_any = false;
            for event in drive.events {
                injected_any = true;
                state.ui.dispatch_event(app, services, &event);
            }

            if injected_any {
                // Script-driven events bypass the winit event loop, so we must apply any generated
                // command effects (e.g. Tab => focus traversal) before we record snapshots.
                //
                // Keep non-command effects queued for the runner to handle after `render` returns.
                let mut deferred_effects: Vec<Effect> = Vec::new();
                loop {
                    let effects = app.flush_effects();
                    if effects.is_empty() {
                        break;
                    }

                    let mut applied_any_command = false;
                    for effect in effects {
                        match effect {
                            Effect::Command { window: w, command } => {
                                if w.is_none() || w == Some(window) {
                                    let _ = state.ui.dispatch_command(app, services, &command);
                                    applied_any_command = true;
                                } else {
                                    deferred_effects.push(Effect::Command { window: w, command });
                                }
                            }
                            other => deferred_effects.push(other),
                        }
                    }

                    if !applied_any_command {
                        break;
                    }
                }
                for effect in deferred_effects {
                    app.push_effect(effect);
                }

                state.ui.request_semantics_snapshot();
                let mut frame = fret_ui::UiFrameCx::new(
                    &mut state.ui,
                    app,
                    services,
                    window,
                    bounds,
                    scale_factor,
                );
                frame.layout_all();
            }
        }

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.paint_all(scene);

        #[cfg(not(target_arch = "wasm32"))]
        {
            app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
                let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
                svc.record_snapshot(
                    app,
                    window,
                    bounds,
                    scale_factor,
                    &state.ui,
                    element_runtime,
                    scene,
                );
                let _ = svc.maybe_dump_if_triggered();
                if svc.is_enabled() {
                    app.push_effect(Effect::RequestAnimationFrame(window));
                }
            });
        }
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }

    fn accessibility_snapshot(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<Arc<fret_core::SemanticsSnapshot>> {
        state.ui.semantics_snapshot_arc()
    }
}

#[cfg(test)]
mod stack_overflow_repro_tests;
