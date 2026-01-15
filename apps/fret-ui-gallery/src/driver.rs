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
use fret_ui::{Invalidation, Theme, UiTree};
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

use crate::spec::*;
use crate::ui;
struct UiGalleryWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    selected_page: Model<Arc<str>>,
    workspace_tabs: Model<Vec<Arc<str>>>,
    workspace_dirty_tabs: Model<Vec<Arc<str>>>,
    nav_query: Model<String>,
    content_tab: Model<Option<Arc<str>>>,
    theme_preset: Model<Option<Arc<str>>>,
    theme_preset_open: Model<bool>,
    applied_theme_preset: Option<Arc<str>>,
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
}

#[derive(Default)]
struct UiGalleryDriver;

impl UiGalleryDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> UiGalleryWindowState {
        let start_page = ui_gallery_start_page().unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
        let selected_page = app.models_mut().insert(start_page.clone());

        let mut workspace_tabs_init = vec![
            Arc::<str>::from(PAGE_INTRO),
            Arc::<str>::from(PAGE_LAYOUT),
            Arc::<str>::from(PAGE_BUTTON),
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

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        UiGalleryWindowState {
            ui,
            root: None,
            selected_page,
            workspace_tabs,
            workspace_dirty_tabs,
            nav_query,
            content_tab,
            theme_preset,
            theme_preset_open,
            applied_theme_preset: Some(Arc::from("zinc/light")),
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

    fn handle_gallery_command(app: &mut App, state: &UiGalleryWindowState, command: &CommandId) {
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
            _ => {}
        }
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

        let selected_page = state.selected_page.clone();
        let workspace_tabs = state.workspace_tabs.clone();
        let workspace_dirty_tabs = state.workspace_dirty_tabs.clone();
        let nav_query = state.nav_query.clone();
        let content_tab = state.content_tab.clone();
        let theme_preset = state.theme_preset.clone();
        let theme_preset_open = state.theme_preset_open.clone();
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

        Self::sync_shadcn_theme(app, state);

        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("fret-ui-gallery", |cx| {
                    if (bisect & BISECT_MINIMAL_ROOT) != 0 {
                        return vec![cx.text("Hello, fret-ui-gallery")];
                    }

                    cx.observe_model(&selected_page, Invalidation::Layout);
                    cx.observe_model(&workspace_tabs, Invalidation::Layout);
                    cx.observe_model(&workspace_dirty_tabs, Invalidation::Layout);
                    cx.observe_model(&nav_query, Invalidation::Layout);
                    cx.observe_model(&content_tab, Invalidation::Layout);
                    cx.observe_model(&theme_preset, Invalidation::Layout);
                    cx.observe_model(&theme_preset_open, Invalidation::Layout);
                    cx.observe_model(&popover_open, Invalidation::Layout);
                    cx.observe_model(&dialog_open, Invalidation::Layout);
                    cx.observe_model(&alert_dialog_open, Invalidation::Layout);
                    cx.observe_model(&sheet_open, Invalidation::Layout);
                    cx.observe_model(&select_value, Invalidation::Layout);
                    cx.observe_model(&select_open, Invalidation::Layout);
                    cx.observe_model(&combobox_value, Invalidation::Layout);
                    cx.observe_model(&combobox_open, Invalidation::Layout);
                    cx.observe_model(&combobox_query, Invalidation::Layout);
                    cx.observe_model(&date_picker_open, Invalidation::Layout);
                    cx.observe_model(&date_picker_month, Invalidation::Layout);
                    cx.observe_model(&date_picker_selected, Invalidation::Layout);
                    cx.observe_model(&resizable_h_fractions, Invalidation::Layout);
                    cx.observe_model(&resizable_v_fractions, Invalidation::Layout);
                    cx.observe_model(&data_table_state, Invalidation::Layout);
                    cx.observe_model(&data_grid_selected_row, Invalidation::Layout);
                    cx.observe_model(&tabs_value, Invalidation::Layout);
                    cx.observe_model(&accordion_value, Invalidation::Layout);
                    cx.observe_model(&progress, Invalidation::Layout);
                    cx.observe_model(&checkbox, Invalidation::Layout);
                    cx.observe_model(&switch, Invalidation::Layout);
                    cx.observe_model(&text_input, Invalidation::Layout);
                    cx.observe_model(&text_area, Invalidation::Layout);
                    cx.observe_model(&dropdown_open, Invalidation::Layout);
                    cx.observe_model(&context_menu_open, Invalidation::Layout);
                    cx.observe_model(&cmdk_open, Invalidation::Layout);
                    cx.observe_model(&cmdk_query, Invalidation::Layout);
                    cx.observe_model(&last_action, Invalidation::Layout);

                    let theme = Theme::global(&*cx.app).clone();

                    let selected = cx
                        .app
                        .models()
                        .read(&selected_page, |v| v.clone())
                        .ok()
                        .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));

                    let query = cx
                        .app
                        .models()
                        .read(&nav_query, |v| v.clone())
                        .ok()
                        .unwrap_or_default();

                    let sidebar = cx.keyed("ui_gallery.sidebar", |cx| {
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
                            )
                        }
                    });

                    let content = cx.keyed(("ui_gallery.content", selected.as_ref()), |cx| {
                        if (bisect & BISECT_SIMPLE_CONTENT) != 0 {
                            cx.container(
                                decl_style::container_props(
                                    &theme,
                                    ChromeRefinement::default()
                                        .bg(ColorRef::Color(theme.color_required("background")))
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
                            )
                        }
                    });

                    let menubar = shadcn::Menubar::new(vec![
                        shadcn::MenubarMenu::new("File").entries(vec![
                            shadcn::MenubarEntry::Group(shadcn::MenubarGroup::new(vec![
                                shadcn::MenubarEntry::Item(
                                    shadcn::MenubarItem::new("Open").on_select(CMD_APP_OPEN),
                                ),
                                shadcn::MenubarEntry::Item(
                                    shadcn::MenubarItem::new("Save").on_select(CMD_APP_SAVE),
                                ),
                                shadcn::MenubarEntry::Item(
                                    shadcn::MenubarItem::new("Settings")
                                        .on_select(CMD_APP_SETTINGS),
                                ),
                            ])),
                        ]),
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

                    let tab_strip = if (bisect & BISECT_DISABLE_TAB_STRIP) != 0 {
                        cx.text("Tabs (disabled)")
                    } else {
                        let workspace_tab_ids = cx
                            .app
                            .models()
                            .get_cloned(&workspace_tabs)
                            .unwrap_or_default();
                        let workspace_dirty_ids = cx
                            .app
                            .models()
                            .get_cloned(&workspace_dirty_tabs)
                            .unwrap_or_default();

                        WorkspaceTabStrip::new(selected.clone())
                            .tabs(workspace_tab_ids.iter().map(|tab_id| {
                                let (title, _origin, _docs, _usage) =
                                    ui::page_meta(tab_id.as_ref());
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
                    };

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

                    let status_last_action = cx
                        .app
                        .models()
                        .get_cloned(&last_action)
                        .unwrap_or_else(|| Arc::<str>::from("<none>"));
                    let status_theme = cx
                        .app
                        .models()
                        .get_cloned(&theme_preset)
                        .flatten()
                        .unwrap_or_else(|| Arc::<str>::from("<default>"));

                    let status_bar = WorkspaceStatusBar::new()
                        .left(vec![cx.text(format!(
                            "last action: {}",
                            status_last_action.as_ref()
                        ))])
                        .right(vec![cx.text(format!("theme: {}", status_theme.as_ref()))])
                        .into_element(cx);

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

                    vec![
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
                    ]
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
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

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

        let _ = Self::handle_nav_command(app, state, &command);
        Self::handle_gallery_command(app, state, &command);

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

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
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
