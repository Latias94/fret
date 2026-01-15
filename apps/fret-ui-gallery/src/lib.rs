use fret_app::{App, CommandId, CommandMeta, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Event, SemanticsRole, UiServices};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_markdown as markdown;
use fret_runtime::PlatformCapabilities;
use fret_ui::action::{UiActionHost, UiActionHostAdapter};
use fret_ui::declarative;
use fret_ui::element::SemanticsProps;
use fret_ui::{Invalidation, Theme, UiTree};
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use fret_workspace::{
    WorkspaceFrame, WorkspaceStatusBar, WorkspaceTab, WorkspaceTabStrip, WorkspaceTopBar,
};
use std::sync::Arc;
use std::time::Duration;
use time::Date;

const CMD_NAV_SELECT_PREFIX: &str = "ui_gallery.nav.select.";
const CMD_DATA_GRID_ROW_PREFIX: &str = "ui_gallery.data_grid.row.";

const PAGE_INTRO: &str = "intro";
const PAGE_LAYOUT: &str = "layout";
const PAGE_BUTTON: &str = "button";
const PAGE_OVERLAY: &str = "overlay";
const PAGE_FORMS: &str = "forms";
const PAGE_SELECT: &str = "select";
const PAGE_COMBOBOX: &str = "combobox";
const PAGE_DATE_PICKER: &str = "date_picker";
const PAGE_RESIZABLE: &str = "resizable";
const PAGE_DATA_TABLE: &str = "data_table";
const PAGE_DATA_GRID: &str = "data_grid";
const PAGE_TABS: &str = "tabs";
const PAGE_ACCORDION: &str = "accordion";
const PAGE_TABLE: &str = "table";
const PAGE_PROGRESS: &str = "progress";
const PAGE_MENUS: &str = "menus";
const PAGE_COMMAND: &str = "command";
const PAGE_TOAST: &str = "toast";

const CMD_NAV_INTRO: &str = "ui_gallery.nav.select.intro";
const CMD_NAV_LAYOUT: &str = "ui_gallery.nav.select.layout";
const CMD_NAV_BUTTON: &str = "ui_gallery.nav.select.button";
const CMD_NAV_OVERLAY: &str = "ui_gallery.nav.select.overlay";
const CMD_NAV_FORMS: &str = "ui_gallery.nav.select.forms";
const CMD_NAV_SELECT: &str = "ui_gallery.nav.select.select";
const CMD_NAV_COMBOBOX: &str = "ui_gallery.nav.select.combobox";
const CMD_NAV_DATE_PICKER: &str = "ui_gallery.nav.select.date_picker";
const CMD_NAV_RESIZABLE: &str = "ui_gallery.nav.select.resizable";
const CMD_NAV_DATA_TABLE: &str = "ui_gallery.nav.select.data_table";
const CMD_NAV_DATA_GRID: &str = "ui_gallery.nav.select.data_grid";
const CMD_NAV_TABS: &str = "ui_gallery.nav.select.tabs";
const CMD_NAV_ACCORDION: &str = "ui_gallery.nav.select.accordion";
const CMD_NAV_TABLE: &str = "ui_gallery.nav.select.table";
const CMD_NAV_PROGRESS: &str = "ui_gallery.nav.select.progress";
const CMD_NAV_MENUS: &str = "ui_gallery.nav.select.menus";
const CMD_NAV_COMMAND: &str = "ui_gallery.nav.select.command";
const CMD_NAV_TOAST: &str = "ui_gallery.nav.select.toast";

const CMD_PROGRESS_INC: &str = "ui_gallery.progress.inc";
const CMD_PROGRESS_DEC: &str = "ui_gallery.progress.dec";
const CMD_PROGRESS_RESET: &str = "ui_gallery.progress.reset";

const CMD_MENU_DROPDOWN_APPLE: &str = "ui_gallery.menu.dropdown.apple";
const CMD_MENU_DROPDOWN_ORANGE: &str = "ui_gallery.menu.dropdown.orange";
const CMD_MENU_CONTEXT_ACTION: &str = "ui_gallery.menu.context.action";

const CMD_TOAST_DEFAULT: &str = "ui_gallery.toast.default";
const CMD_TOAST_SUCCESS: &str = "ui_gallery.toast.success";
const CMD_TOAST_ERROR: &str = "ui_gallery.toast.error";
const CMD_TOAST_SHOW_ACTION_CANCEL: &str = "ui_gallery.toast.show_action_cancel";
const CMD_TOAST_ACTION: &str = "ui_gallery.toast.action";
const CMD_TOAST_CANCEL: &str = "ui_gallery.toast.cancel";

const CMD_APP_OPEN: &str = "ui_gallery.app.open";
const CMD_APP_SAVE: &str = "ui_gallery.app.save";
const CMD_APP_SETTINGS: &str = "ui_gallery.app.settings";

static NAV_GROUPS: &[NavGroupSpec] = &[
    NavGroupSpec {
        title: "Core",
        items: &[
            NavItemSpec::new(
                PAGE_INTRO,
                "Introduction",
                "Core contracts",
                CMD_NAV_INTRO,
                &["overview", "contracts"],
            ),
            NavItemSpec::new(
                PAGE_LAYOUT,
                "Layout",
                "Layout system",
                CMD_NAV_LAYOUT,
                &["layout", "flex", "stack"],
            ),
        ],
    },
    NavGroupSpec {
        title: "Shadcn",
        items: &[
            NavItemSpec::new(
                PAGE_BUTTON,
                "Button",
                "fret-ui-shadcn",
                CMD_NAV_BUTTON,
                &["button", "variant"],
            ),
            NavItemSpec::new(
                PAGE_FORMS,
                "Forms",
                "fret-ui-shadcn",
                CMD_NAV_FORMS,
                &["input", "textarea", "checkbox", "switch"],
            ),
            NavItemSpec::new(
                PAGE_SELECT,
                "Select",
                "fret-ui-shadcn",
                CMD_NAV_SELECT,
                &["select", "popover", "listbox"],
            ),
            NavItemSpec::new(
                PAGE_COMBOBOX,
                "Combobox",
                "fret-ui-shadcn",
                CMD_NAV_COMBOBOX,
                &["combobox", "cmdk", "search"],
            ),
            NavItemSpec::new(
                PAGE_DATE_PICKER,
                "Date Picker",
                "fret-ui-shadcn",
                CMD_NAV_DATE_PICKER,
                &["date", "calendar", "popover"],
            ),
            NavItemSpec::new(
                PAGE_RESIZABLE,
                "Resizable",
                "fret-ui-shadcn",
                CMD_NAV_RESIZABLE,
                &["split", "panel", "resize"],
            ),
            NavItemSpec::new(
                PAGE_DATA_TABLE,
                "DataTable",
                "fret-ui-shadcn + fret-ui-headless",
                CMD_NAV_DATA_TABLE,
                &["table", "virtualized", "tanstack"],
            ),
            NavItemSpec::new(
                PAGE_DATA_GRID,
                "DataGrid",
                "fret-ui-shadcn",
                CMD_NAV_DATA_GRID,
                &["grid", "viewport", "virtualized"],
            ),
            NavItemSpec::new(
                PAGE_TABS,
                "Tabs",
                "fret-ui-shadcn",
                CMD_NAV_TABS,
                &["tabs", "roving", "focus"],
            ),
            NavItemSpec::new(
                PAGE_ACCORDION,
                "Accordion",
                "fret-ui-shadcn",
                CMD_NAV_ACCORDION,
                &["accordion", "collapsible"],
            ),
            NavItemSpec::new(
                PAGE_TABLE,
                "Table",
                "fret-ui-shadcn",
                CMD_NAV_TABLE,
                &["table", "grid"],
            ),
            NavItemSpec::new(
                PAGE_PROGRESS,
                "Progress",
                "fret-ui-shadcn",
                CMD_NAV_PROGRESS,
                &["progress"],
            ),
            NavItemSpec::new(
                PAGE_MENUS,
                "Menus",
                "fret-ui-shadcn",
                CMD_NAV_MENUS,
                &["dropdown", "context-menu"],
            ),
            NavItemSpec::new(
                PAGE_COMMAND,
                "Command Palette",
                "fret-ui-shadcn",
                CMD_NAV_COMMAND,
                &["cmdk", "command"],
            ),
            NavItemSpec::new(
                PAGE_TOAST,
                "Toast",
                "fret-ui-shadcn",
                CMD_NAV_TOAST,
                &["sonner", "toast"],
            ),
            NavItemSpec::new(
                PAGE_OVERLAY,
                "Overlay",
                "Radix-shaped primitives",
                CMD_NAV_OVERLAY,
                &["dialog", "popover"],
            ),
        ],
    },
];

#[derive(Clone, Copy)]
struct NavItemSpec {
    id: &'static str,
    label: &'static str,
    origin: &'static str,
    command: &'static str,
    tags: &'static [&'static str],
}

impl NavItemSpec {
    const fn new(
        id: &'static str,
        label: &'static str,
        origin: &'static str,
        command: &'static str,
        tags: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            label,
            origin,
            command,
            tags,
        }
    }
}

#[derive(Clone, Copy)]
struct NavGroupSpec {
    title: &'static str,
    items: &'static [NavItemSpec],
}

struct UiGalleryWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    selected_page: Model<Arc<str>>,
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
        let selected_page = app.models_mut().insert(Arc::<str>::from(PAGE_INTRO));
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
        let _ = app.models_mut().update(&state.selected_page, |v| *v = page);
        true
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

    fn matches_query(query: &str, item: &NavItemSpec) -> bool {
        let q = query.trim();
        if q.is_empty() {
            return true;
        }

        let q_lower = q.to_ascii_lowercase();
        if item.label.to_ascii_lowercase().contains(&q_lower) {
            return true;
        }
        if item.origin.to_ascii_lowercase().contains(&q_lower) {
            return true;
        }
        item.tags
            .iter()
            .any(|t| t.to_ascii_lowercase().contains(&q_lower))
    }

    fn render_ui(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut UiGalleryWindowState,
        bounds: fret_core::Rect,
    ) {
        OverlayController::begin_frame(app, window);

        let selected_page = state.selected_page.clone();
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
                    cx.observe_model(&selected_page, Invalidation::Layout);
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

                    let sidebar = sidebar_view(
                        cx,
                        &theme,
                        selected.as_ref(),
                        query.as_str(),
                        nav_query.clone(),
                    );
                    let content = content_view(
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
                    );

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

                    let tab_strip = WorkspaceTabStrip::new(selected.clone())
                        .tabs([
                            WorkspaceTab::new(PAGE_INTRO, "Intro", CommandId::from(CMD_NAV_INTRO)),
                            WorkspaceTab::new(
                                PAGE_LAYOUT,
                                "Layout",
                                CommandId::from(CMD_NAV_LAYOUT),
                            ),
                            WorkspaceTab::new(
                                PAGE_BUTTON,
                                "Button",
                                CommandId::from(CMD_NAV_BUTTON),
                            ),
                            WorkspaceTab::new(
                                PAGE_OVERLAY,
                                "Overlay",
                                CommandId::from(CMD_NAV_OVERLAY),
                            ),
                            WorkspaceTab::new(
                                PAGE_COMMAND,
                                "Command",
                                CommandId::from(CMD_NAV_COMMAND),
                            ),
                        ])
                        .into_element(cx);

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
                        shadcn::Toaster::new().into_element(cx),
                    ]
                });

        state.ui.set_root(root);
        OverlayController::render(&mut state.ui, app, services, window, bounds);
        state.root = Some(root);
    }
}

fn sidebar_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    query: &str,
    nav_query: Model<String>,
) -> AnyElement {
    let title_row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center(),
        |cx| {
            vec![
                cx.text("Fret UI Gallery"),
                shadcn::Badge::new("WIP")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
            ]
        },
    );

    let query_input = shadcn::Input::new(nav_query)
        .a11y_label("Search components")
        .placeholder("Search… (id / tag)")
        .into_element(cx);

    let mut nav_sections: Vec<AnyElement> = Vec::new();
    for group in NAV_GROUPS {
        let mut group_items: Vec<AnyElement> = Vec::new();
        for item in group.items {
            if !UiGalleryDriver::matches_query(query, item) {
                continue;
            }

            let is_selected = selected == item.id;
            let variant = if is_selected {
                shadcn::ButtonVariant::Secondary
            } else {
                shadcn::ButtonVariant::Ghost
            };

            group_items.push(
                shadcn::Button::new(item.label)
                    .variant(variant)
                    .on_click(item.command)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
            );
        }

        if group_items.is_empty() {
            continue;
        }

        nav_sections.push(cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from(group.title),
            style: None,
            color: Some(theme.color_required("muted-foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        }));

        nav_sections.push(stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N1),
            |_cx| group_items,
        ));
    }

    let nav_scroll = shadcn::ScrollArea::new(vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |_cx| nav_sections,
    )])
    .refine_layout(LayoutRefinement::default().w_full().h_full())
    .into_element(cx);

    let container = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .p(Space::N4),
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(280.0)))
                .h_full(),
        ),
        |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .gap(Space::N4),
                |_cx| vec![title_row, query_input, nav_scroll],
            )]
        },
    );

    container
}

fn content_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    content_tab: Model<Option<Arc<str>>>,
    theme_preset: Model<Option<Arc<str>>>,
    theme_preset_open: Model<bool>,
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
) -> AnyElement {
    let (title, origin, docs_md, usage_md) = page_meta(selected);

    let header = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center(),
        |cx| {
            let left = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N1).items_start(),
                |cx| {
                    vec![
                        cx.text(title),
                        cx.text_props(TextProps {
                            layout: Default::default(),
                            text: Arc::from(origin),
                            style: None,
                            color: Some(theme.color_required("muted-foreground")),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Ellipsis,
                        }),
                    ]
                },
            );

            let theme_select = shadcn::Select::new(theme_preset, theme_preset_open)
                .placeholder("Theme preset")
                .items([
                    shadcn::SelectItem::new("zinc/light", "Zinc (light)"),
                    shadcn::SelectItem::new("zinc/dark", "Zinc (dark)"),
                    shadcn::SelectItem::new("slate/light", "Slate (light)"),
                    shadcn::SelectItem::new("slate/dark", "Slate (dark)"),
                    shadcn::SelectItem::new("neutral/light", "Neutral (light)"),
                    shadcn::SelectItem::new("neutral/dark", "Neutral (dark)"),
                ])
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(220.0))))
                .into_element(cx);

            vec![left, theme_select]
        },
    );

    let preview_panel = page_preview(
        cx,
        theme,
        selected,
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
    );
    let docs_panel = markdown::Markdown::new(Arc::from(docs_md)).into_element(cx);
    let usage_panel = markdown::Markdown::new(Arc::from(usage_md)).into_element(cx);

    let tabs = shadcn::Tabs::new(content_tab)
        .refine_layout(LayoutRefinement::default().w_full())
        .list_full_width(true)
        .items([
            shadcn::TabsItem::new("preview", "Preview", vec![preview_panel]),
            shadcn::TabsItem::new("usage", "Usage", vec![usage_panel]),
            shadcn::TabsItem::new("docs", "Notes", vec![docs_panel]),
        ])
        .into_element(cx);

    let content = shadcn::ScrollArea::new(vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N6),
        |_cx| vec![header, tabs],
    )])
    .refine_layout(LayoutRefinement::default().w_full().h_full())
    .into_element(cx);

    cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("background")))
                .p(Space::N6),
            LayoutRefinement::default().w_full().h_full(),
        ),
        |_cx| vec![content],
    )
}

fn page_meta(selected: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    match selected {
        PAGE_LAYOUT => (
            "Layout / Stacks & Constraints",
            "fret-ui + fret-ui-kit",
            DOC_LAYOUT,
            USAGE_LAYOUT,
        ),
        PAGE_BUTTON => ("Button", "fret-ui-shadcn", DOC_BUTTON, USAGE_BUTTON),
        PAGE_FORMS => ("Forms", "fret-ui-shadcn", DOC_FORMS, USAGE_FORMS),
        PAGE_SELECT => ("Select", "fret-ui-shadcn", DOC_SELECT, USAGE_SELECT),
        PAGE_COMBOBOX => ("Combobox", "fret-ui-shadcn", DOC_COMBOBOX, USAGE_COMBOBOX),
        PAGE_DATE_PICKER => (
            "Date Picker",
            "fret-ui-shadcn",
            DOC_DATE_PICKER,
            USAGE_DATE_PICKER,
        ),
        PAGE_RESIZABLE => (
            "Resizable",
            "fret-ui-shadcn",
            DOC_RESIZABLE,
            USAGE_RESIZABLE,
        ),
        PAGE_DATA_TABLE => (
            "DataTable",
            "fret-ui-shadcn + fret-ui-headless",
            DOC_DATA_TABLE,
            USAGE_DATA_TABLE,
        ),
        PAGE_DATA_GRID => ("DataGrid", "fret-ui-shadcn", DOC_DATA_GRID, USAGE_DATA_GRID),
        PAGE_TABS => ("Tabs", "fret-ui-shadcn", DOC_TABS, USAGE_TABS),
        PAGE_ACCORDION => (
            "Accordion",
            "fret-ui-shadcn",
            DOC_ACCORDION,
            USAGE_ACCORDION,
        ),
        PAGE_TABLE => ("Table", "fret-ui-shadcn", DOC_TABLE, USAGE_TABLE),
        PAGE_PROGRESS => ("Progress", "fret-ui-shadcn", DOC_PROGRESS, USAGE_PROGRESS),
        PAGE_MENUS => ("Menus", "fret-ui-shadcn", DOC_MENUS, USAGE_MENUS),
        PAGE_COMMAND => (
            "Command Palette",
            "fret-ui-shadcn",
            DOC_COMMAND,
            USAGE_COMMAND,
        ),
        PAGE_TOAST => ("Toast", "fret-ui-shadcn", DOC_TOAST, USAGE_TOAST),
        PAGE_OVERLAY => (
            "Overlay / Popover & Dialog",
            "fret-ui-shadcn (Radix-shaped primitives)",
            DOC_OVERLAY,
            USAGE_OVERLAY,
        ),
        _ => ("Introduction", "Core contracts", DOC_INTRO, USAGE_INTRO),
    }
}

fn page_preview(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
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
) -> AnyElement {
    let body: Vec<AnyElement> = match selected {
        PAGE_LAYOUT => preview_layout(cx, theme),
        PAGE_BUTTON => preview_button(cx),
        PAGE_OVERLAY => {
            preview_overlay(cx, popover_open, dialog_open, alert_dialog_open, sheet_open)
        }
        PAGE_FORMS => preview_forms(cx, text_input, text_area, checkbox, switch),
        PAGE_SELECT => preview_select(cx, select_value, select_open),
        PAGE_COMBOBOX => preview_combobox(cx, combobox_value, combobox_open, combobox_query),
        PAGE_DATE_PICKER => preview_date_picker(
            cx,
            date_picker_open,
            date_picker_month,
            date_picker_selected,
        ),
        PAGE_RESIZABLE => {
            preview_resizable(cx, theme, resizable_h_fractions, resizable_v_fractions)
        }
        PAGE_DATA_TABLE => preview_data_table(cx, data_table_state),
        PAGE_DATA_GRID => preview_data_grid(cx, data_grid_selected_row),
        PAGE_TABS => preview_tabs(cx, tabs_value),
        PAGE_ACCORDION => preview_accordion(cx, accordion_value),
        PAGE_TABLE => preview_table(cx),
        PAGE_PROGRESS => preview_progress(cx, progress),
        PAGE_MENUS => preview_menus(cx, dropdown_open, context_menu_open, last_action.clone()),
        PAGE_COMMAND => preview_command_palette(cx, cmdk_open, cmdk_query, last_action.clone()),
        PAGE_TOAST => preview_toast(cx, last_action.clone()),
        _ => preview_intro(cx, theme),
    };

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Preview").into_element(cx),
            shadcn::CardDescription::new("Interactive preview for validating behaviors.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(body).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}

fn preview_intro(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let card = |cx: &mut ElementContext<'_, App>, title: &str, desc: &str| -> AnyElement {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![cx.text(desc)]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
    };

    let grid = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        |cx| {
            vec![
                card(
                    cx,
                    "Core",
                    "Window / event / UiTree / renderer contracts (mechanisms & boundaries)",
                ),
                card(
                    cx,
                    "UI Kit",
                    "Headless interaction policies: focus trap, dismiss, hover intent, etc.",
                ),
                card(
                    cx,
                    "Shadcn",
                    "Visual recipes: composed defaults built on the Kit layer",
                ),
            ]
        },
    );

    let note = {
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default().w_full(),
        );
        cx.container(props, |cx| {
            vec![cx.text("Phase 1: fixed two-pane layout + hardcoded docs strings (focus on validating component usability). Docking/multi-window views will come later.")]
        })
    };

    vec![grid, note]
}

fn preview_layout(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let boxy = |cx: &mut ElementContext<'_, App>, label: &str, color: fret_core::Color| {
        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(color))
                    .rounded(Radius::Md)
                    .p(Space::N3),
                LayoutRefinement::default().w_full(),
            ),
            |cx| vec![cx.text(label)],
        )
    };

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_stretch(),
        |cx| {
            vec![
                boxy(cx, "Left (fill)", theme.color_required("accent")),
                boxy(cx, "Center (fill)", theme.color_required("muted")),
                boxy(cx, "Right (fill)", theme.color_required("card")),
            ]
        },
    );

    vec![
        cx.text("Layout mental model: LayoutRefinement (constraints) + stack (composition) + Theme tokens (color/spacing)."),
        row,
    ]
}

fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let variants = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Default").into_element(cx),
                shadcn::Button::new("Secondary")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .into_element(cx),
                shadcn::Button::new("Outline")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::Button::new("Ghost")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .into_element(cx),
                shadcn::Button::new("Destructive")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .into_element(cx),
                shadcn::Button::new("Disabled")
                    .disabled(true)
                    .into_element(cx),
            ]
        },
    );

    let sizes = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Small")
                    .size(shadcn::ButtonSize::Sm)
                    .into_element(cx),
                shadcn::Button::new("Default")
                    .size(shadcn::ButtonSize::Default)
                    .into_element(cx),
                shadcn::Button::new("Large")
                    .size(shadcn::ButtonSize::Lg)
                    .into_element(cx),
            ]
        },
    );

    vec![variants, sizes]
}

fn preview_forms(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    let input = shadcn::Input::new(text_input)
        .a11y_label("Email")
        .placeholder("name@example.com")
        .into_element(cx);

    let textarea = shadcn::Textarea::new(text_area)
        .a11y_label("Message")
        .into_element(cx);

    let toggles = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Checkbox::new(checkbox)
                                .a11y_label("Accept terms")
                                .into_element(cx),
                            cx.text("Accept terms"),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(switch)
                                .a11y_label("Enable feature")
                                .into_element(cx),
                            cx.text("Enable feature"),
                        ]
                    },
                ),
            ]
        },
    );

    vec![
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3),
            |_cx| vec![input, textarea, toggles],
        ),
        cx.text(
            "Tip: these are model-bound controls; values persist while you stay in the window.",
        ),
    ]
}

fn preview_select(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    let select = shadcn::Select::new(value.clone(), open)
        .placeholder("Pick a fruit")
        .items([
            shadcn::SelectItem::new("apple", "Apple"),
            shadcn::SelectItem::new("banana", "Banana"),
            shadcn::SelectItem::new("orange", "Orange"),
        ])
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))))
        .into_element(cx);

    let selected = cx
        .app
        .models()
        .read(&value, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![select, cx.text(format!("Selected: {selected}"))]
}

fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    let combo = shadcn::Combobox::new(value.clone(), open)
        .a11y_label("Combobox")
        .width(Px(240.0))
        .placeholder("Pick a fruit")
        .query_model(query.clone())
        .items([
            shadcn::ComboboxItem::new("apple", "Apple"),
            shadcn::ComboboxItem::new("banana", "Banana"),
            shadcn::ComboboxItem::new("orange", "Orange"),
            shadcn::ComboboxItem::new("disabled", "Disabled").disabled(true),
        ])
        .into_element(cx);

    let selected = cx
        .app
        .models()
        .read(&value, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let query = cx.app.models().get_cloned(&query).unwrap_or_default();

    vec![
        combo,
        cx.text(format!("Selected: {selected}")),
        cx.text(format!("Query: {query}")),
    ]
}

fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    let picker = shadcn::DatePicker::new(open, month, selected.clone())
        .placeholder("Pick a date")
        .into_element(cx);

    let selected_text: Arc<str> = cx
        .app
        .models()
        .read(&selected, |v| v.map(|d| Arc::<str>::from(d.to_string())))
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![picker, cx.text(format!("Selected: {selected_text}"))]
}

fn preview_resizable(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    h_fractions: Model<Vec<f32>>,
    v_fractions: Model<Vec<f32>>,
) -> Vec<AnyElement> {
    let boxy = |cx: &mut ElementContext<'_, App>, title: &str, color_key: &str| -> AnyElement {
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required(color_key)))
                .rounded(Radius::Md)
                .p(Space::N3),
            LayoutRefinement::default().w_full().h_full(),
        );
        cx.container(props, move |cx| vec![cx.text(title)])
    };

    let nested_vertical = shadcn::ResizablePanelGroup::new(v_fractions)
        .axis(fret_core::Axis::Vertical)
        .entries(vec![
            shadcn::ResizablePanel::new(vec![boxy(cx, "Viewport", "muted")])
                .min_px(Px(120.0))
                .into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new(vec![boxy(cx, "Console", "card")])
                .min_px(Px(80.0))
                .into(),
        ])
        .into_element(cx);

    let root = shadcn::ResizablePanelGroup::new(h_fractions)
        .axis(fret_core::Axis::Horizontal)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(320.0))),
        )
        .entries(vec![
            shadcn::ResizablePanel::new(vec![boxy(cx, "Explorer", "accent")])
                .min_px(Px(140.0))
                .into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new(vec![nested_vertical])
                .min_px(Px(240.0))
                .into(),
        ])
        .into_element(cx);

    vec![cx.text("Drag the handles to resize panels."), root]
}

#[derive(Debug, Clone)]
struct DemoProcessRow {
    id: u64,
    name: Arc<str>,
    status: Arc<str>,
    cpu: u64,
    mem_mb: u64,
}

#[derive(Debug, Clone)]
struct DemoProcessTableAssets {
    data: Arc<[DemoProcessRow]>,
    columns: Arc<[fret_ui_headless::table::ColumnDef<DemoProcessRow>]>,
}

fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let assets = cx.with_state(
        || {
            let data: Arc<[DemoProcessRow]> = Arc::from(vec![
                DemoProcessRow {
                    id: 1,
                    name: Arc::from("Renderer"),
                    status: Arc::from("Running"),
                    cpu: 12,
                    mem_mb: 420,
                },
                DemoProcessRow {
                    id: 2,
                    name: Arc::from("Asset Cache"),
                    status: Arc::from("Idle"),
                    cpu: 0,
                    mem_mb: 128,
                },
                DemoProcessRow {
                    id: 3,
                    name: Arc::from("Indexer"),
                    status: Arc::from("Running"),
                    cpu: 38,
                    mem_mb: 860,
                },
                DemoProcessRow {
                    id: 4,
                    name: Arc::from("Spellcheck"),
                    status: Arc::from("Disabled"),
                    cpu: 0,
                    mem_mb: 0,
                },
                DemoProcessRow {
                    id: 5,
                    name: Arc::from("Language Server"),
                    status: Arc::from("Running"),
                    cpu: 7,
                    mem_mb: 512,
                },
            ]);

            let columns: Arc<[fret_ui_headless::table::ColumnDef<DemoProcessRow>]> =
                Arc::from(vec![
                    fret_ui_headless::table::ColumnDef::new("name")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.name.cmp(&b.name))
                        .size(220.0),
                    fret_ui_headless::table::ColumnDef::new("status")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.status.cmp(&b.status))
                        .size(140.0),
                    fret_ui_headless::table::ColumnDef::new("cpu%")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.cpu.cmp(&b.cpu))
                        .size(90.0),
                    fret_ui_headless::table::ColumnDef::new("mem_mb")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| {
                            a.mem_mb.cmp(&b.mem_mb)
                        })
                        .size(110.0),
                ]);

            DemoProcessTableAssets { data, columns }
        },
        |st| st.clone(),
    );

    let selected_count = cx
        .app
        .models()
        .read(&state, |st| st.row_selection.len())
        .ok()
        .unwrap_or(0);
    let sorting = cx
        .app
        .models()
        .read(&state, |st| {
            st.sorting.first().map(|s| (s.column.clone(), s.desc))
        })
        .ok()
        .flatten();

    let sorting_text: Arc<str> = sorting
        .map(|(col, desc)| {
            Arc::<str>::from(format!(
                "Sorting: {} {}",
                col,
                if desc { "desc" } else { "asc" }
            ))
        })
        .unwrap_or_else(|| Arc::<str>::from("Sorting: <none>"));

    let table = shadcn::DataTable::new()
        .row_height(Px(36.0))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(280.0))),
        )
        .into_element(
            cx,
            assets.data.clone(),
            1,
            state,
            assets.columns.clone(),
            |row, _index, _parent| fret_ui_headless::table::RowKey(row.id),
            |col| col.id.clone(),
            |cx, col, row| match col.id.as_ref() {
                "name" => cx.text(row.name.as_ref()),
                "status" => cx.text(row.status.as_ref()),
                "cpu%" => cx.text(format!("{}%", row.cpu)),
                "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                _ => cx.text("?"),
            },
        );

    vec![
        cx.text("Click header to sort; click row to toggle selection."),
        cx.text(format!("Selected rows: {selected_count}")),
        cx.text(sorting_text.as_ref()),
        table,
    ]
}

fn preview_data_grid(
    cx: &mut ElementContext<'_, App>,
    selected_row: Model<Option<u64>>,
) -> Vec<AnyElement> {
    let selected = cx.app.models().get_cloned(&selected_row).flatten();

    let grid = shadcn::DataGridElement::new(["PID", "Name", "State", "CPU%"], 200)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(320.0))),
        )
        .into_element(
            cx,
            1,
            1,
            |row| row as u64,
            move |row| {
                let is_selected = selected == Some(row as u64);
                let cmd = CommandId::new(format!("{CMD_DATA_GRID_ROW_PREFIX}{row}"));
                shadcn::DataGridRowState {
                    selected: is_selected,
                    enabled: row % 17 != 0,
                    on_click: Some(cmd),
                }
            },
            |cx, row, col| {
                let pid = 1000 + row as u64;
                match col {
                    0 => cx.text(pid.to_string()),
                    1 => cx.text(format!("Process {row}")),
                    2 => cx.text(if row % 3 == 0 { "Running" } else { "Idle" }),
                    _ => cx.text(((row * 7) % 100).to_string()),
                }
            },
        );

    let selected_text: Arc<str> = selected
        .map(|v| Arc::<str>::from(v.to_string()))
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        cx.text("Virtualized rows/cols viewport; click a row to select (disabled every 17th row)."),
        cx.text(format!("Selected row: {selected_text}")),
        grid,
    ]
}

fn preview_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let selected = cx
        .app
        .models()
        .get_cloned(&value)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let tabs = shadcn::Tabs::new(value)
        .refine_layout(LayoutRefinement::default().w_full())
        .list_full_width(false)
        .items([
            shadcn::TabsItem::new(
                "overview",
                "Overview",
                vec![cx.text("Tabs are stateful and roving-focus friendly.")],
            ),
            shadcn::TabsItem::new(
                "details",
                "Details",
                vec![cx.text("Use Tabs for sections within a single page.")],
            ),
            shadcn::TabsItem::new(
                "notes",
                "Notes",
                vec![cx.text(
                    "In this gallery, the outer shell also uses Tabs (Preview/Usage/Notes).",
                )],
            ),
        ])
        .into_element(cx);

    vec![tabs, cx.text(format!("Selected: {selected}"))]
}

fn preview_accordion(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let open = cx
        .app
        .models()
        .get_cloned(&value)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let accordion = shadcn::Accordion::single(value)
        .collapsible(true)
        .refine_layout(LayoutRefinement::default().w_full())
        .items([
            shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("Item 1")]),
                shadcn::AccordionContent::new(vec![cx.text("This section is collapsible.")]),
            ),
            shadcn::AccordionItem::new(
                "item-2",
                shadcn::AccordionTrigger::new(vec![cx.text("Item 2")]),
                shadcn::AccordionContent::new(vec![
                    cx.text("Keyboard navigation uses roving focus."),
                ]),
            ),
            shadcn::AccordionItem::new(
                "item-3",
                shadcn::AccordionTrigger::new(vec![cx.text("Item 3")]),
                shadcn::AccordionContent::new(vec![
                    cx.text("Content lives in normal layout flow (no portal)."),
                ]),
            ),
        ])
        .into_element(cx);

    vec![accordion, cx.text(format!("Open item: {open}"))]
}

fn preview_table(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let header = shadcn::TableHeader::new(vec![
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableHead::new("Crate").into_element(cx),
                shadcn::TableHead::new("Layer").into_element(cx),
                shadcn::TableHead::new("Notes").into_element(cx),
            ],
        )
        .border_bottom(true)
        .into_element(cx),
    ])
    .into_element(cx);

    let body = shadcn::TableBody::new(vec![
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(cx.text("fret-ui")).into_element(cx),
                shadcn::TableCell::new(cx.text("mechanisms")).into_element(cx),
                shadcn::TableCell::new(cx.text("Element tree + layout")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(cx.text("fret-ui-kit")).into_element(cx),
                shadcn::TableCell::new(cx.text("policies")).into_element(cx),
                shadcn::TableCell::new(cx.text("Dismiss / focus / menu / overlays"))
                    .into_element(cx),
            ],
        )
        .selected(true)
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(cx.text("fret-ui-shadcn")).into_element(cx),
                shadcn::TableCell::new(cx.text("recipes")).into_element(cx),
                shadcn::TableCell::new(cx.text("skinned components + defaults")).into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx);

    let caption = shadcn::TableCaption::new("Tip: TableRow has hover + selected styling parity.")
        .into_element(cx);

    let table = shadcn::Table::new(vec![header, body, caption])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    vec![table]
}

fn preview_progress(cx: &mut ElementContext<'_, App>, progress: Model<f32>) -> Vec<AnyElement> {
    let bar = shadcn::Progress::new(progress).into_element(cx);

    let controls = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("-10")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_PROGRESS_DEC)
                    .into_element(cx),
                shadcn::Button::new("+10")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_PROGRESS_INC)
                    .into_element(cx),
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .on_click(CMD_PROGRESS_RESET)
                    .into_element(cx),
            ]
        },
    );

    vec![bar, controls]
}

fn preview_menus(
    cx: &mut ElementContext<'_, App>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("DropdownMenu")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(dropdown_open.clone())
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Apple").on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Orange").on_select(CMD_MENU_DROPDOWN_ORANGE),
                ),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let context_menu = shadcn::ContextMenu::new(context_menu_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("ContextMenu (right click)")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Action").on_select(CMD_MENU_CONTEXT_ACTION),
                ),
                shadcn::ContextMenuEntry::Separator,
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![dropdown, context_menu],
        ),
        cx.text(format!("last action: {last}")),
    ]
}

fn preview_command_palette(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    query: Model<String>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let cmdk = shadcn::CommandDialog::new_with_host_commands(cx, open.clone(), query)
        .a11y_label("Command palette")
        .into_element(cx, |cx| {
            shadcn::Button::new("Open Command Palette")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open)
                .into_element(cx)
        });

    vec![
        cx.text("Tip: Ctrl/Cmd+P triggers the command palette command."),
        cx.text(format!("last action: {last}")),
        cmdk,
    ]
}

fn preview_toast(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let buttons = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Default")
                    .on_click(CMD_TOAST_DEFAULT)
                    .into_element(cx),
                shadcn::Button::new("Success")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_SUCCESS)
                    .into_element(cx),
                shadcn::Button::new("Error")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_ERROR)
                    .into_element(cx),
                shadcn::Button::new("Action + Cancel")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_SHOW_ACTION_CANCEL)
                    .into_element(cx),
            ]
        },
    );

    vec![buttons, cx.text(format!("last action: {last}"))]
}

fn preview_overlay(
    cx: &mut ElementContext<'_, App>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
) -> Vec<AnyElement> {
    let tooltip = shadcn::Tooltip::new(
        shadcn::Button::new("Tooltip (hover)")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx),
        shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
            cx,
            "Tooltip: hover intent + placement",
        )])
        .into_element(cx),
    )
    .arrow(true)
    .open_delay_frames(10)
    .close_delay_frames(10)
    .side(shadcn::TooltipSide::Top)
    .into_element(cx);

    let hover_card = shadcn::HoverCard::new(
        shadcn::Button::new("HoverCard (hover)")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx),
        shadcn::HoverCardContent::new(vec![
            cx.text("HoverCard content (overlay-root)"),
            cx.text("Move pointer from trigger to content."),
        ])
        .into_element(cx),
    )
    .close_delay_frames(10)
    .into_element(cx);

    let popover = shadcn::Popover::new(popover_open.clone())
        .auto_focus(true)
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(popover_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::PopoverContent::new(vec![
                    cx.text("Popover content"),
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .toggle_model(popover_open.clone())
                        .into_element(cx),
                ])
                .into_element(cx)
            },
        );

    let dialog = shadcn::Dialog::new(dialog_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("Dialog")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(dialog_open.clone())
                .into_element(cx)
        },
        |cx| {
            shadcn::DialogContent::new(vec![
                shadcn::DialogHeader::new(vec![
                    shadcn::DialogTitle::new("Dialog").into_element(cx),
                    shadcn::DialogDescription::new("Escape / overlay click closes")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DialogFooter::new(vec![
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .toggle_model(dialog_open.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        },
    );

    let alert_dialog = shadcn::AlertDialog::new(alert_dialog_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("AlertDialog")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(alert_dialog_open.clone())
                .into_element(cx)
        },
        |cx| {
            shadcn::AlertDialogContent::new(vec![
                shadcn::AlertDialogHeader::new(vec![
                    shadcn::AlertDialogTitle::new("Are you absolutely sure?").into_element(cx),
                    shadcn::AlertDialogDescription::new("This is non-closable by overlay click.")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::AlertDialogFooter::new(vec![
                    shadcn::AlertDialogCancel::new("Cancel", alert_dialog_open.clone())
                        .into_element(cx),
                    shadcn::AlertDialogAction::new("Continue", alert_dialog_open.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        },
    );

    let sheet = shadcn::Sheet::new(sheet_open.clone())
        .side(shadcn::SheetSide::Right)
        .size(Px(360.0))
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Sheet")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(sheet_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::SheetContent::new(vec![
                    shadcn::SheetHeader::new(vec![
                        shadcn::SheetTitle::new("Sheet").into_element(cx),
                        shadcn::SheetDescription::new("A modal side panel.").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::SheetFooter::new(vec![
                        shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .toggle_model(sheet_open.clone())
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx)
            },
        );

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![tooltip, hover_card],
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![popover, dialog],
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![alert_dialog, sheet],
        ),
    ]
}

const DOC_INTRO: &str = r#"
## Goals

This is an **editor-grade UI** gallery app used to:

- Validate that `fret-ui-shadcn` / `fret-ui-kit` / ecosystem components work under real composition.
- Provide a component-doc-site browsing experience (left navigation, right preview + docs).

Phase 1 intentionally uses hardcoded doc strings to validate the interaction path end-to-end.
"#;

const USAGE_INTRO: &str = r#"
```rust
// Native
cargo run -p fret-ui-gallery

// Web (via fret-demo-web host)
cd apps/fret-demo-web
trunk serve --open
// open: http://127.0.0.1:8080/?demo=ui_gallery
```
"#;

const DOC_LAYOUT: &str = r#"
## LayoutRefinement + stack

The gallery shell is a common editor-like layout:

- Fixed-width left navigation (scrollable)
- Right content area (scrollable)

In Fret, this is typically expressed with:

- `LayoutRefinement`: width/height/min/max/fill constraints
- `stack::{hstack,vstack}`: row/column composition & alignment
- `Theme` tokens: design system values like spacing/color/radius
"#;

const USAGE_LAYOUT: &str = r#"
```rust
let root = stack::hstack(
    cx,
    stack::HStackProps::default()
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_stretch(),
    |_cx| vec![sidebar, content],
);
```
"#;

const DOC_BUTTON: &str = r#"
## Button

Validate `variant` / `size` behaviors and default styling consistency.

This layer is **visual recipes**. Interaction policies (hover intent, focus trap, etc.) should live in `fret-ui-kit` / ecosystem crates.
"#;

const USAGE_BUTTON: &str = r#"
```rust
use fret_ui_shadcn as shadcn;

let btn = shadcn::Button::new("Save")
    .variant(shadcn::ButtonVariant::Default)
    .into_element(cx);
```
"#;

const DOC_FORMS: &str = r#"
## Forms

This page validates the basic form building blocks:

- `Input` / `Textarea`
- `Checkbox` / `Switch`

These are model-bound controls: the UI is driven by `Model<T>` updates.
"#;

const USAGE_FORMS: &str = r#"
```rust
let email = app.models_mut().insert(String::new());
let input = shadcn::Input::new(email).a11y_label("Email");
```
"#;

const DOC_SELECT: &str = r#"
## Select

`Select` is an overlay-driven component (listbox in a popover-like layer).

This page validates:

- value model binding (`Model<Option<Arc<str>>>`)
- open/close model binding (`Model<bool>`)
"#;

const USAGE_SELECT: &str = r#"
```rust
let value = app.models_mut().insert(Some(Arc::<str>::from("apple")));
let open = app.models_mut().insert(false);

let select = shadcn::Select::new(value, open)
    .placeholder("Pick a fruit")
    .items([shadcn::SelectItem::new("apple", "Apple")]);
```
"#;

const DOC_COMBOBOX: &str = r#"
## Combobox

Combobox is a shadcn recipe: Popover + Command list + optional search.

This page validates:

- value model (`Model<Option<Arc<str>>>`)
- open model (`Model<bool>`)
- query model (`Model<String>`)
"#;

const USAGE_COMBOBOX: &str = r#"
```rust
let value = app.models_mut().insert(None::<Arc<str>>);
let open = app.models_mut().insert(false);
let query = app.models_mut().insert(String::new());

let combo = shadcn::Combobox::new(value, open)
    .query_model(query)
    .items([shadcn::ComboboxItem::new("apple", "Apple")]);
```
"#;

const DOC_DATE_PICKER: &str = r#"
## Date Picker

Date picker is a Popover + Calendar integration.

This page validates:

- selected date model (`Model<Option<time::Date>>`)
- month model (`Model<CalendarMonth>`)
- open model (`Model<bool>`)
"#;

const USAGE_DATE_PICKER: &str = r#"
```rust
let open = app.models_mut().insert(false);
let month = app
    .models_mut()
    .insert(fret_ui_headless::calendar::CalendarMonth::from_date(
        time::OffsetDateTime::now_utc().date(),
    ));
let selected = app.models_mut().insert(None::<time::Date>);

let picker = shadcn::DatePicker::new(open, month, selected);
```
"#;

const DOC_RESIZABLE: &str = r#"
## Resizable

Resizable panel groups are runtime-owned drag surfaces (splitter handles).

This page validates:

- fraction model (`Model<Vec<f32>>`) persistence
- nested groups (horizontal + vertical)
"#;

const USAGE_RESIZABLE: &str = r#"
```rust
let fractions = app.models_mut().insert(vec![0.3, 0.7]);

let group = shadcn::ResizablePanelGroup::new(fractions).entries(vec![
    shadcn::ResizablePanel::new(vec![/* ... */]).into(),
    shadcn::ResizableHandle::new().into(),
    shadcn::ResizablePanel::new(vec![/* ... */]).into(),
]);
```
"#;

const DOC_DATA_TABLE: &str = r#"
## DataTable

`DataTable` integrates the TanStack-aligned headless engine (ADR 0101):

- headless: sorting / filtering / selection state (`TableState`)
- UI: fixed header + virtualized body
"#;

const USAGE_DATA_TABLE: &str = r#"
```rust
let state = app.models_mut().insert(fret_ui_headless::table::TableState::default());

let table = shadcn::DataTable::new().into_element(
    cx,
    data,
    data_revision,
    state,
    columns,
    get_row_key,
    header_label,
    cell_at,
);
```
"#;

const DOC_DATA_GRID: &str = r#"
## DataGrid

`DataGrid` is a viewport-driven, virtualized rows/cols surface.

This page validates:

- large row counts without allocating all row widgets
- per-row hover/selected styling
"#;

const USAGE_DATA_GRID: &str = r#"
```rust
let grid = shadcn::DataGrid::new(["A", "B", "C"], 10_000).into_element(
    cx,
    rows_revision,
    cols_revision,
    row_key_at,
    row_state_at,
    cell_at,
);
```
"#;

const DOC_TABS: &str = r#"
## Tabs

Tabs are a roving-focus friendly navigation surface within a page.

This page validates:

- controlled selection model (`Model<Option<Arc<str>>>`)
- tab list layout and content switching
"#;

const USAGE_TABS: &str = r#"
```rust
let tab = app.models_mut().insert(Some(Arc::<str>::from("overview")));

let tabs = shadcn::Tabs::new(tab).items([
    shadcn::TabsItem::new("overview", "Overview", vec![cx.text("...")]),
    shadcn::TabsItem::new("details", "Details", vec![cx.text("...")]),
]);
```
"#;

const DOC_ACCORDION: &str = r#"
## Accordion

Accordion is a collapsible section list with keyboard navigation (roving focus).

This page validates:

- controlled open item model (`Model<Option<Arc<str>>>`)
- `collapsible` (allow close -> `None`)
"#;

const USAGE_ACCORDION: &str = r#"
```rust
let open_item = app.models_mut().insert(Some(Arc::<str>::from("item-1")));

let accordion = shadcn::Accordion::single(open_item)
    .collapsible(true)
    .items([
        shadcn::AccordionItem::new(
            "item-1",
            shadcn::AccordionTrigger::new(vec![cx.text("Item 1")]),
            shadcn::AccordionContent::new(vec![cx.text("...")]),
        ),
    ]);
```
"#;

const DOC_TABLE: &str = r#"
## Table

`Table` is a layout + styling facade (not HTML). `TableRow` is pressable for hover/selected parity.
"#;

const USAGE_TABLE: &str = r#"
```rust
let table = shadcn::Table::new(vec![
    shadcn::TableHeader::new(vec![/* rows */]).into_element(cx),
    shadcn::TableBody::new(vec![/* rows */]).into_element(cx),
]);
```
"#;

const DOC_PROGRESS: &str = r#"
## Progress

`Progress` is a purely visual indicator bound to a numeric model (default 0..=100).
"#;

const USAGE_PROGRESS: &str = r#"
```rust
let progress = app.models_mut().insert(35.0f32);
let bar = shadcn::Progress::new(progress);
```
"#;

const DOC_MENUS: &str = r#"
## Menus

This page validates two common overlay menu primitives:

- `DropdownMenu` (triggered by a button)
- `ContextMenu` (triggered by right click)
"#;

const USAGE_MENUS: &str = r#"
```rust
let open = app.models_mut().insert(false);
let menu = shadcn::DropdownMenu::new(open).into_element(cx, trigger, |_cx| entries);
```
"#;

const DOC_COMMAND: &str = r#"
## Command Palette

`CommandDialog` (cmdk) renders a searchable list of host commands.

In this gallery we register a small command surface (`File`, `View`) so cmdk has something to show.
"#;

const USAGE_COMMAND: &str = r#"
```rust
let open = app.models_mut().insert(false);
let query = app.models_mut().insert(String::new());
let cmdk = shadcn::CommandDialog::new_with_host_commands(cx, open, query);
```
"#;

const DOC_TOAST: &str = r#"
## Toast (Sonner)

Toasts are queued via `Sonner::global(app)` and rendered by a `Toaster` element (overlay layer).
"#;

const USAGE_TOAST: &str = r#"
```rust
let sonner = shadcn::Sonner::global(app);
sonner.toast_success_message(&mut host, window, "Done!", shadcn::ToastMessageOptions::new());
```
"#;

const DOC_OVERLAY: &str = r#"
## Overlay / Portal

Tooltip/HoverCard/Popover/Dialog/Sheet are rendered through overlay/portal mechanisms, outside the normal layout flow.

Goals:

- open/close state model binding
- basic policies (ESC, overlay click, focus behavior)
"#;

const USAGE_OVERLAY: &str = r#"
```rust
let open = app.models_mut().insert(false);

let dialog = shadcn::Dialog::new(open.clone()).into_element(
    cx,
    |cx| shadcn::Button::new("Open").toggle_model(open.clone()).into_element(cx),
    |cx| shadcn::DialogContent::new(vec![cx.text("Hello")]).into_element(cx),
);
```
"#;

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
