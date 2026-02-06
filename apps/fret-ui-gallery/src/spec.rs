use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::docs;
use fret_runtime::CommandId;

pub(crate) const ENV_UI_GALLERY_BISECT: &str = "FRET_UI_GALLERY_BISECT";
pub(crate) const ENV_UI_GALLERY_START_PAGE: &str = "FRET_UI_GALLERY_START_PAGE";

pub(crate) const BISECT_MINIMAL_ROOT: u32 = 1 << 0;
pub(crate) const BISECT_DISABLE_OVERLAY_CONTROLLER: u32 = 1 << 1;
pub(crate) const BISECT_DISABLE_TOASTER: u32 = 1 << 2;
pub(crate) const BISECT_DISABLE_TAB_STRIP: u32 = 1 << 3;
pub(crate) const BISECT_SIMPLE_SIDEBAR: u32 = 1 << 4;
pub(crate) const BISECT_SIMPLE_CONTENT: u32 = 1 << 5;
pub(crate) const BISECT_DISABLE_SIDEBAR_SCROLL: u32 = 1 << 6;
pub(crate) const BISECT_DISABLE_CONTENT_SCROLL: u32 = 1 << 7;
pub(crate) const BISECT_DISABLE_MARKDOWN: u32 = 1 << 8;
pub(crate) const BISECT_DISABLE_TABS: u32 = 1 << 9;

pub(crate) fn ui_gallery_bisect_flags() -> u32 {
    static FLAGS: OnceLock<u32> = OnceLock::new();
    *FLAGS.get_or_init(|| {
        std::env::var(ENV_UI_GALLERY_BISECT)
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0)
    })
}

pub(crate) fn ui_gallery_start_page() -> Option<Arc<str>> {
    #[cfg(target_arch = "wasm32")]
    {
        ui_gallery_start_page_from_url()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let id = std::env::var(ENV_UI_GALLERY_START_PAGE).ok()?;
        ui_gallery_start_page_from_id(&id)
    }
}

fn ui_gallery_start_page_from_id(id: &str) -> Option<Arc<str>> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }

    if page_spec(id).is_some() {
        Some(Arc::<str>::from(id))
    } else {
        None
    }
}

#[cfg(target_arch = "wasm32")]
fn ui_gallery_start_page_from_url() -> Option<Arc<str>> {
    fn find_key_value(raw: &str, key: &str) -> Option<String> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }

        for part in raw.split('&') {
            let (k, v) = part.split_once('=').unwrap_or((part, ""));
            if k.trim() == key {
                let v = v.trim();
                if v.is_empty() {
                    return None;
                }
                return Some(v.to_string());
            }
        }
        None
    }

    let window = web_sys::window()?;
    let location = window.location();
    let search = location.search().ok().unwrap_or_default();
    let hash = location.hash().ok().unwrap_or_default();

    let search = search.strip_prefix('?').unwrap_or(search.as_str());
    let hash = hash.strip_prefix('#').unwrap_or(hash.as_str());

    let id = find_key_value(search, "page")
        .or_else(|| find_key_value(hash, "page"))
        .or_else(|| find_key_value(search, "start_page"))
        .or_else(|| find_key_value(hash, "start_page"))?;

    ui_gallery_start_page_from_id(&id)
}

pub(crate) const CMD_DATA_GRID_ROW_PREFIX: &str = "ui_gallery.data_grid.row.";
pub(crate) const DATA_GRID_ROWS: usize = 200;

pub(crate) const PAGE_INTRO: &str = "intro";
pub(crate) const PAGE_LAYOUT: &str = "layout";
pub(crate) const PAGE_VIEW_CACHE: &str = "view_cache";
pub(crate) const PAGE_HIT_TEST_TORTURE: &str = "hit_test_torture";
pub(crate) const PAGE_EFFECTS_BLUR_TORTURE: &str = "effects_blur_torture";
pub(crate) const PAGE_SVG_UPLOAD_TORTURE: &str = "svg_upload_torture";
pub(crate) const PAGE_SVG_SCROLL_TORTURE: &str = "svg_scroll_torture";
pub(crate) const PAGE_VIRTUAL_LIST_TORTURE: &str = "virtual_list_torture";
pub(crate) const PAGE_UI_KIT_LIST_TORTURE: &str = "ui_kit_list_torture";
pub(crate) const PAGE_CODE_VIEW_TORTURE: &str = "code_view_torture";
pub(crate) const PAGE_CODE_EDITOR_MVP: &str = "code_editor_mvp";
pub(crate) const PAGE_CODE_EDITOR_TORTURE: &str = "code_editor_torture";
pub(crate) const PAGE_TEXT_SELECTION_PERF: &str = "text_selection_perf";
pub(crate) const PAGE_TEXT_BIDI_RTL_CONFORMANCE: &str = "text_bidi_rtl_conformance";
pub(crate) const PAGE_TEXT_MEASURE_OVERLAY: &str = "text_measure_overlay";
pub(crate) const PAGE_WEB_IME_HARNESS: &str = "web_ime_harness";
pub(crate) const PAGE_CHART_TORTURE: &str = "chart_torture";
pub(crate) const PAGE_CANVAS_CULL_TORTURE: &str = "canvas_cull_torture";
pub(crate) const PAGE_NODE_GRAPH_CULL_TORTURE: &str = "node_graph_cull_torture";
pub(crate) const PAGE_CHROME_TORTURE: &str = "chrome_torture";
pub(crate) const PAGE_WINDOWED_ROWS_SURFACE_TORTURE: &str = "windowed_rows_surface_torture";
pub(crate) const PAGE_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE: &str =
    "windowed_rows_surface_interactive_torture";
pub(crate) const PAGE_DATA_TABLE_TORTURE: &str = "data_table_torture";
pub(crate) const PAGE_TREE_TORTURE: &str = "tree_torture";
pub(crate) const PAGE_TABLE_RETAINED_TORTURE: &str = "table_retained_torture";
pub(crate) const PAGE_AI_TRANSCRIPT_TORTURE: &str = "ai_transcript_torture";
pub(crate) const PAGE_AI_CHAT_DEMO: &str = "ai_chat_demo";
pub(crate) const PAGE_INSPECTOR_TORTURE: &str = "inspector_torture";
pub(crate) const PAGE_FILE_TREE_TORTURE: &str = "file_tree_torture";
pub(crate) const PAGE_BUTTON: &str = "button";
pub(crate) const PAGE_CARD: &str = "card";
pub(crate) const PAGE_BADGE: &str = "badge";
pub(crate) const PAGE_AVATAR: &str = "avatar";
pub(crate) const PAGE_SKELETON: &str = "skeleton";
pub(crate) const PAGE_SCROLL_AREA: &str = "scroll_area";
pub(crate) const PAGE_TOOLTIP: &str = "tooltip";
pub(crate) const PAGE_SLIDER: &str = "slider";
pub(crate) const PAGE_ICONS: &str = "icons";
pub(crate) const PAGE_FIELD: &str = "field";
pub(crate) const PAGE_OVERLAY: &str = "overlay";
pub(crate) const PAGE_FORMS: &str = "forms";
pub(crate) const PAGE_SELECT: &str = "select";
pub(crate) const PAGE_COMBOBOX: &str = "combobox";
pub(crate) const PAGE_DATE_PICKER: &str = "date_picker";
pub(crate) const PAGE_RESIZABLE: &str = "resizable";
pub(crate) const PAGE_DATA_TABLE: &str = "data_table";
pub(crate) const PAGE_DATA_GRID: &str = "data_grid";
pub(crate) const PAGE_TABS: &str = "tabs";
pub(crate) const PAGE_ACCORDION: &str = "accordion";
pub(crate) const PAGE_TABLE: &str = "table";
pub(crate) const PAGE_PROGRESS: &str = "progress";
pub(crate) const PAGE_MENUS: &str = "menus";
pub(crate) const PAGE_COMMAND: &str = "command";
pub(crate) const PAGE_TOAST: &str = "toast";
pub(crate) const PAGE_ALERT: &str = "alert";
pub(crate) const PAGE_ALERT_DIALOG: &str = "alert_dialog";
pub(crate) const PAGE_ASPECT_RATIO: &str = "aspect_ratio";
pub(crate) const PAGE_BREADCRUMB: &str = "breadcrumb";
pub(crate) const PAGE_BUTTON_GROUP: &str = "button_group";
pub(crate) const PAGE_CALENDAR: &str = "calendar";
pub(crate) const PAGE_CAROUSEL: &str = "carousel";
pub(crate) const PAGE_CHART: &str = "chart";
pub(crate) const PAGE_CHECKBOX: &str = "checkbox";
pub(crate) const PAGE_COLLAPSIBLE: &str = "collapsible";
pub(crate) const PAGE_CONTEXT_MENU: &str = "context_menu";
pub(crate) const PAGE_DIALOG: &str = "dialog";
pub(crate) const PAGE_DRAWER: &str = "drawer";
pub(crate) const PAGE_DROPDOWN_MENU: &str = "dropdown_menu";
pub(crate) const PAGE_EMPTY: &str = "empty";
pub(crate) const PAGE_FORM: &str = "form";
pub(crate) const PAGE_HOVER_CARD: &str = "hover_card";
pub(crate) const PAGE_INPUT: &str = "input";
pub(crate) const PAGE_INPUT_GROUP: &str = "input_group";
pub(crate) const PAGE_INPUT_OTP: &str = "input_otp";
pub(crate) const PAGE_ITEM: &str = "item";
pub(crate) const PAGE_KBD: &str = "kbd";
pub(crate) const PAGE_LABEL: &str = "label";
pub(crate) const PAGE_MENUBAR: &str = "menubar";
pub(crate) const PAGE_NATIVE_SELECT: &str = "native_select";
pub(crate) const PAGE_NAVIGATION_MENU: &str = "navigation_menu";
pub(crate) const PAGE_PAGINATION: &str = "pagination";
pub(crate) const PAGE_POPOVER: &str = "popover";
pub(crate) const PAGE_RADIO_GROUP: &str = "radio_group";
pub(crate) const PAGE_SEPARATOR: &str = "separator";
pub(crate) const PAGE_SHEET: &str = "sheet";
pub(crate) const PAGE_SIDEBAR: &str = "sidebar";
pub(crate) const PAGE_SONNER: &str = "sonner";
pub(crate) const PAGE_SPINNER: &str = "spinner";
pub(crate) const PAGE_SWITCH: &str = "switch";
pub(crate) const PAGE_TEXTAREA: &str = "textarea";
pub(crate) const PAGE_TOGGLE: &str = "toggle";
pub(crate) const PAGE_TOGGLE_GROUP: &str = "toggle_group";
pub(crate) const PAGE_TYPOGRAPHY: &str = "typography";
pub(crate) const PAGE_MATERIAL3_GALLERY: &str = "material3_gallery";
pub(crate) const PAGE_MATERIAL3_BUTTON: &str = "material3_button";
pub(crate) const PAGE_MATERIAL3_ICON_BUTTON: &str = "material3_icon_button";
pub(crate) const PAGE_MATERIAL3_CHECKBOX: &str = "material3_checkbox";
pub(crate) const PAGE_MATERIAL3_SWITCH: &str = "material3_switch";
pub(crate) const PAGE_MATERIAL3_RADIO: &str = "material3_radio";
pub(crate) const PAGE_MATERIAL3_BADGE: &str = "material3_badge";
pub(crate) const PAGE_MATERIAL3_SEGMENTED_BUTTON: &str = "material3_segmented_button";
pub(crate) const PAGE_MATERIAL3_TOP_APP_BAR: &str = "material3_top_app_bar";
pub(crate) const PAGE_MATERIAL3_BOTTOM_SHEET: &str = "material3_bottom_sheet";
pub(crate) const PAGE_MATERIAL3_DATE_PICKER: &str = "material3_date_picker";
pub(crate) const PAGE_MATERIAL3_TIME_PICKER: &str = "material3_time_picker";
pub(crate) const PAGE_MATERIAL3_AUTOCOMPLETE: &str = "material3_autocomplete";
pub(crate) const PAGE_MATERIAL3_SELECT: &str = "material3_select";
pub(crate) const PAGE_MATERIAL3_TEXT_FIELD: &str = "material3_text_field";
pub(crate) const PAGE_MATERIAL3_TABS: &str = "material3_tabs";
pub(crate) const PAGE_MATERIAL3_NAVIGATION_BAR: &str = "material3_navigation_bar";
pub(crate) const PAGE_MATERIAL3_NAVIGATION_RAIL: &str = "material3_navigation_rail";
pub(crate) const PAGE_MATERIAL3_NAVIGATION_DRAWER: &str = "material3_navigation_drawer";
pub(crate) const PAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER: &str = "material3_modal_navigation_drawer";
pub(crate) const PAGE_MATERIAL3_DIALOG: &str = "material3_dialog";
pub(crate) const PAGE_MATERIAL3_MENU: &str = "material3_menu";
pub(crate) const PAGE_MATERIAL3_LIST: &str = "material3_list";
pub(crate) const PAGE_MATERIAL3_SNACKBAR: &str = "material3_snackbar";
pub(crate) const PAGE_MATERIAL3_TOOLTIP: &str = "material3_tooltip";
pub(crate) const PAGE_MATERIAL3_STATE_MATRIX: &str = "material3_state_matrix";
pub(crate) const PAGE_MATERIAL3_TOUCH_TARGETS: &str = "material3_touch_targets";

pub(crate) const CMD_NAV_INTRO: &str = "ui_gallery.nav.select.intro";
pub(crate) const CMD_NAV_LAYOUT: &str = "ui_gallery.nav.select.layout";
pub(crate) const CMD_NAV_VIEW_CACHE: &str = "ui_gallery.nav.select.view_cache";
pub(crate) const CMD_NAV_HIT_TEST_TORTURE: &str = "ui_gallery.nav.select.hit_test_torture";
pub(crate) const CMD_NAV_VIRTUAL_LIST_TORTURE: &str = "ui_gallery.nav.select.virtual_list_torture";
pub(crate) const CMD_NAV_UI_KIT_LIST_TORTURE: &str = "ui_gallery.nav.select.ui_kit_list_torture";
pub(crate) const CMD_NAV_CODE_VIEW_TORTURE: &str = "ui_gallery.nav.select.code_view_torture";
pub(crate) const CMD_NAV_CODE_EDITOR_MVP: &str = "ui_gallery.nav.select.code_editor_mvp";
pub(crate) const CMD_NAV_CODE_EDITOR_TORTURE: &str = "ui_gallery.nav.select.code_editor_torture";
pub(crate) const CMD_NAV_TEXT_SELECTION_PERF: &str = "ui_gallery.nav.select.text_selection_perf";
pub(crate) const CMD_NAV_TEXT_BIDI_RTL_CONFORMANCE: &str =
    "ui_gallery.nav.select.text_bidi_rtl_conformance";
pub(crate) const CMD_NAV_TEXT_MEASURE_OVERLAY: &str = "ui_gallery.nav.select.text_measure_overlay";
pub(crate) const CMD_NAV_WEB_IME_HARNESS: &str = "ui_gallery.nav.select.web_ime_harness";
pub(crate) const CMD_NAV_CHART_TORTURE: &str = "ui_gallery.nav.select.chart_torture";
pub(crate) const CMD_NAV_CANVAS_CULL_TORTURE: &str = "ui_gallery.nav.select.canvas_cull_torture";
pub(crate) const CMD_NAV_NODE_GRAPH_CULL_TORTURE: &str =
    "ui_gallery.nav.select.node_graph_cull_torture";
pub(crate) const CMD_NAV_CHROME_TORTURE: &str = "ui_gallery.nav.select.chrome_torture";
pub(crate) const CMD_NAV_WINDOWED_ROWS_SURFACE_TORTURE: &str =
    "ui_gallery.nav.select.windowed_rows_surface_torture";
pub(crate) const CMD_NAV_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE: &str =
    "ui_gallery.nav.select.windowed_rows_surface_interactive_torture";
pub(crate) const CMD_NAV_DATA_TABLE_TORTURE: &str = "ui_gallery.nav.select.data_table_torture";
pub(crate) const CMD_NAV_TREE_TORTURE: &str = "ui_gallery.nav.select.tree_torture";
pub(crate) const CMD_NAV_TABLE_RETAINED_TORTURE: &str =
    "ui_gallery.nav.select.table_retained_torture";
pub(crate) const CMD_NAV_AI_TRANSCRIPT_TORTURE: &str =
    "ui_gallery.nav.select.ai_transcript_torture";
pub(crate) const CMD_NAV_AI_CHAT_DEMO: &str = "ui_gallery.nav.select.ai_chat_demo";
pub(crate) const CMD_NAV_INSPECTOR_TORTURE: &str = "ui_gallery.nav.select.inspector_torture";
pub(crate) const CMD_NAV_FILE_TREE_TORTURE: &str = "ui_gallery.nav.select.file_tree_torture";
pub(crate) const CMD_NAV_BUTTON: &str = "ui_gallery.nav.select.button";
pub(crate) const CMD_NAV_CARD: &str = "ui_gallery.nav.select.card";
pub(crate) const CMD_NAV_BADGE: &str = "ui_gallery.nav.select.badge";
pub(crate) const CMD_NAV_AVATAR: &str = "ui_gallery.nav.select.avatar";
pub(crate) const CMD_NAV_SKELETON: &str = "ui_gallery.nav.select.skeleton";
pub(crate) const CMD_NAV_SCROLL_AREA: &str = "ui_gallery.nav.select.scroll_area";
pub(crate) const CMD_NAV_TOOLTIP: &str = "ui_gallery.nav.select.tooltip";
pub(crate) const CMD_NAV_SLIDER: &str = "ui_gallery.nav.select.slider";
pub(crate) const CMD_NAV_ICONS: &str = "ui_gallery.nav.select.icons";
pub(crate) const CMD_NAV_FIELD: &str = "ui_gallery.nav.select.field";
pub(crate) const CMD_NAV_OVERLAY: &str = "ui_gallery.nav.select.overlay";
pub(crate) const CMD_NAV_FORMS: &str = "ui_gallery.nav.select.forms";
pub(crate) const CMD_NAV_SELECT: &str = "ui_gallery.nav.select.select";
pub(crate) const CMD_NAV_COMBOBOX: &str = "ui_gallery.nav.select.combobox";
pub(crate) const CMD_NAV_DATE_PICKER: &str = "ui_gallery.nav.select.date_picker";
pub(crate) const CMD_NAV_RESIZABLE: &str = "ui_gallery.nav.select.resizable";
pub(crate) const CMD_NAV_DATA_TABLE: &str = "ui_gallery.nav.select.data_table";
pub(crate) const CMD_NAV_DATA_GRID: &str = "ui_gallery.nav.select.data_grid";
pub(crate) const CMD_NAV_TABS: &str = "ui_gallery.nav.select.tabs";
pub(crate) const CMD_NAV_ACCORDION: &str = "ui_gallery.nav.select.accordion";
pub(crate) const CMD_NAV_TABLE: &str = "ui_gallery.nav.select.table";
pub(crate) const CMD_NAV_PROGRESS: &str = "ui_gallery.nav.select.progress";
pub(crate) const CMD_NAV_MENUS: &str = "ui_gallery.nav.select.menus";
pub(crate) const CMD_NAV_COMMAND: &str = "ui_gallery.nav.select.command";
pub(crate) const CMD_NAV_TOAST: &str = "ui_gallery.nav.select.toast";
pub(crate) const CMD_NAV_ALERT: &str = "ui_gallery.nav.select.alert";
pub(crate) const CMD_NAV_ALERT_DIALOG: &str = "ui_gallery.nav.select.alert_dialog";
pub(crate) const CMD_NAV_ASPECT_RATIO: &str = "ui_gallery.nav.select.aspect_ratio";
pub(crate) const CMD_NAV_BREADCRUMB: &str = "ui_gallery.nav.select.breadcrumb";
pub(crate) const CMD_NAV_BUTTON_GROUP: &str = "ui_gallery.nav.select.button_group";
pub(crate) const CMD_NAV_CALENDAR: &str = "ui_gallery.nav.select.calendar";
pub(crate) const CMD_NAV_CAROUSEL: &str = "ui_gallery.nav.select.carousel";
pub(crate) const CMD_NAV_CHART: &str = "ui_gallery.nav.select.chart";
pub(crate) const CMD_NAV_CHECKBOX: &str = "ui_gallery.nav.select.checkbox";
pub(crate) const CMD_NAV_COLLAPSIBLE: &str = "ui_gallery.nav.select.collapsible";
pub(crate) const CMD_NAV_CONTEXT_MENU: &str = "ui_gallery.nav.select.context_menu";
pub(crate) const CMD_NAV_DIALOG: &str = "ui_gallery.nav.select.dialog";
pub(crate) const CMD_NAV_DRAWER: &str = "ui_gallery.nav.select.drawer";
pub(crate) const CMD_NAV_DROPDOWN_MENU: &str = "ui_gallery.nav.select.dropdown_menu";
pub(crate) const CMD_NAV_EMPTY: &str = "ui_gallery.nav.select.empty";
pub(crate) const CMD_NAV_FORM: &str = "ui_gallery.nav.select.form";
pub(crate) const CMD_NAV_HOVER_CARD: &str = "ui_gallery.nav.select.hover_card";
pub(crate) const CMD_NAV_INPUT: &str = "ui_gallery.nav.select.input";
pub(crate) const CMD_NAV_INPUT_GROUP: &str = "ui_gallery.nav.select.input_group";
pub(crate) const CMD_NAV_INPUT_OTP: &str = "ui_gallery.nav.select.input_otp";
pub(crate) const CMD_NAV_ITEM: &str = "ui_gallery.nav.select.item";
pub(crate) const CMD_NAV_KBD: &str = "ui_gallery.nav.select.kbd";
pub(crate) const CMD_NAV_LABEL: &str = "ui_gallery.nav.select.label";
pub(crate) const CMD_NAV_MENUBAR: &str = "ui_gallery.nav.select.menubar";
pub(crate) const CMD_NAV_NATIVE_SELECT: &str = "ui_gallery.nav.select.native_select";
pub(crate) const CMD_NAV_NAVIGATION_MENU: &str = "ui_gallery.nav.select.navigation_menu";
pub(crate) const CMD_NAV_PAGINATION: &str = "ui_gallery.nav.select.pagination";
pub(crate) const CMD_NAV_POPOVER: &str = "ui_gallery.nav.select.popover";
pub(crate) const CMD_NAV_RADIO_GROUP: &str = "ui_gallery.nav.select.radio_group";
pub(crate) const CMD_NAV_SEPARATOR: &str = "ui_gallery.nav.select.separator";
pub(crate) const CMD_NAV_SHEET: &str = "ui_gallery.nav.select.sheet";
pub(crate) const CMD_NAV_SIDEBAR: &str = "ui_gallery.nav.select.sidebar";
pub(crate) const CMD_NAV_SONNER: &str = "ui_gallery.nav.select.sonner";
pub(crate) const CMD_NAV_SPINNER: &str = "ui_gallery.nav.select.spinner";
pub(crate) const CMD_NAV_SWITCH: &str = "ui_gallery.nav.select.switch";
pub(crate) const CMD_NAV_TEXTAREA: &str = "ui_gallery.nav.select.textarea";
pub(crate) const CMD_NAV_TOGGLE: &str = "ui_gallery.nav.select.toggle";
pub(crate) const CMD_NAV_TOGGLE_GROUP: &str = "ui_gallery.nav.select.toggle_group";
pub(crate) const CMD_NAV_TYPOGRAPHY: &str = "ui_gallery.nav.select.typography";
pub(crate) const CMD_NAV_MATERIAL3_GALLERY: &str = "ui_gallery.nav.select.material3_gallery";
pub(crate) const CMD_NAV_MATERIAL3_BUTTON: &str = "ui_gallery.nav.select.material3_button";
pub(crate) const CMD_NAV_MATERIAL3_ICON_BUTTON: &str =
    "ui_gallery.nav.select.material3_icon_button";
pub(crate) const CMD_NAV_MATERIAL3_CHECKBOX: &str = "ui_gallery.nav.select.material3_checkbox";
pub(crate) const CMD_NAV_MATERIAL3_SWITCH: &str = "ui_gallery.nav.select.material3_switch";
pub(crate) const CMD_NAV_MATERIAL3_RADIO: &str = "ui_gallery.nav.select.material3_radio";
pub(crate) const CMD_NAV_MATERIAL3_BADGE: &str = "ui_gallery.nav.select.material3_badge";
pub(crate) const CMD_NAV_MATERIAL3_SEGMENTED_BUTTON: &str =
    "ui_gallery.nav.select.material3_segmented_button";
pub(crate) const CMD_NAV_MATERIAL3_TOP_APP_BAR: &str =
    "ui_gallery.nav.select.material3_top_app_bar";
pub(crate) const CMD_NAV_MATERIAL3_BOTTOM_SHEET: &str =
    "ui_gallery.nav.select.material3_bottom_sheet";
pub(crate) const CMD_NAV_MATERIAL3_DATE_PICKER: &str =
    "ui_gallery.nav.select.material3_date_picker";
pub(crate) const CMD_NAV_MATERIAL3_TIME_PICKER: &str =
    "ui_gallery.nav.select.material3_time_picker";
pub(crate) const CMD_NAV_MATERIAL3_AUTOCOMPLETE: &str =
    "ui_gallery.nav.select.material3_autocomplete";
pub(crate) const CMD_NAV_MATERIAL3_SELECT: &str = "ui_gallery.nav.select.material3_select";
pub(crate) const CMD_NAV_MATERIAL3_TEXT_FIELD: &str = "ui_gallery.nav.select.material3_text_field";
pub(crate) const CMD_NAV_MATERIAL3_TABS: &str = "ui_gallery.nav.select.material3_tabs";
pub(crate) const CMD_NAV_MATERIAL3_NAVIGATION_BAR: &str =
    "ui_gallery.nav.select.material3_navigation_bar";
pub(crate) const CMD_NAV_MATERIAL3_NAVIGATION_RAIL: &str =
    "ui_gallery.nav.select.material3_navigation_rail";
pub(crate) const CMD_NAV_MATERIAL3_NAVIGATION_DRAWER: &str =
    "ui_gallery.nav.select.material3_navigation_drawer";
pub(crate) const CMD_NAV_MATERIAL3_MODAL_NAVIGATION_DRAWER: &str =
    "ui_gallery.nav.select.material3_modal_navigation_drawer";
pub(crate) const CMD_NAV_MATERIAL3_DIALOG: &str = "ui_gallery.nav.select.material3_dialog";
pub(crate) const CMD_NAV_MATERIAL3_MENU: &str = "ui_gallery.nav.select.material3_menu";
pub(crate) const CMD_NAV_MATERIAL3_LIST: &str = "ui_gallery.nav.select.material3_list";
pub(crate) const CMD_NAV_MATERIAL3_SNACKBAR: &str = "ui_gallery.nav.select.material3_snackbar";
pub(crate) const CMD_NAV_MATERIAL3_TOOLTIP: &str = "ui_gallery.nav.select.material3_tooltip";
pub(crate) const CMD_NAV_MATERIAL3_STATE_MATRIX: &str =
    "ui_gallery.nav.select.material3_state_matrix";
pub(crate) const CMD_NAV_MATERIAL3_TOUCH_TARGETS: &str =
    "ui_gallery.nav.select.material3_touch_targets";

pub(crate) const CMD_PROGRESS_INC: &str = "ui_gallery.progress.inc";
pub(crate) const CMD_PROGRESS_DEC: &str = "ui_gallery.progress.dec";
pub(crate) const CMD_PROGRESS_RESET: &str = "ui_gallery.progress.reset";

pub(crate) const CMD_VIEW_CACHE_BUMP: &str = "ui_gallery.view_cache.bump";
pub(crate) const CMD_VIEW_CACHE_RESET: &str = "ui_gallery.view_cache.reset";

pub(crate) const CMD_VIRTUAL_LIST_TORTURE_JUMP: &str = "ui_gallery.virtual_list_torture.jump";
pub(crate) const CMD_VIRTUAL_LIST_TORTURE_SCROLL_BOTTOM: &str =
    "ui_gallery.virtual_list_torture.scroll_bottom";
pub(crate) const CMD_VIRTUAL_LIST_TORTURE_CLEAR_EDIT: &str =
    "ui_gallery.virtual_list_torture.clear_edit";

pub(crate) const CMD_MENU_DROPDOWN_APPLE: &str = "ui_gallery.menu.dropdown.apple";
pub(crate) const CMD_MENU_DROPDOWN_ORANGE: &str = "ui_gallery.menu.dropdown.orange";
pub(crate) const CMD_MENU_CONTEXT_ACTION: &str = "ui_gallery.menu.context.action";

pub(crate) const CMD_TOAST_DEFAULT: &str = "ui_gallery.toast.default";
pub(crate) const CMD_TOAST_SUCCESS: &str = "ui_gallery.toast.success";
pub(crate) const CMD_TOAST_ERROR: &str = "ui_gallery.toast.error";
pub(crate) const CMD_TOAST_SHOW_ACTION_CANCEL: &str = "ui_gallery.toast.show_action_cancel";
pub(crate) const CMD_TOAST_ACTION: &str = "ui_gallery.toast.action";
pub(crate) const CMD_TOAST_CANCEL: &str = "ui_gallery.toast.cancel";

pub(crate) const CMD_APP_OPEN: &str = "ui_gallery.app.open";
pub(crate) const CMD_APP_SAVE: &str = "ui_gallery.app.save";
pub(crate) const CMD_APP_SETTINGS: &str = "ui_gallery.app.settings";
pub(crate) const CMD_APP_SETTINGS_APPLY: &str = "ui_gallery.app.settings.apply";
pub(crate) const CMD_APP_SETTINGS_WRITE_PROJECT: &str = "ui_gallery.app.settings.write_project";
pub(crate) const CMD_APP_TOGGLE_PREFERENCES_ENABLED: &str =
    "ui_gallery.app.preferences.toggle_enabled";

pub(crate) const CMD_CLIPBOARD_COPY_LINK: &str = "ui_gallery.clipboard.copy_link";
pub(crate) const CMD_CLIPBOARD_COPY_USAGE: &str = "ui_gallery.clipboard.copy_usage";
pub(crate) const CMD_CLIPBOARD_COPY_NOTES: &str = "ui_gallery.clipboard.copy_notes";

pub(crate) const CMD_CODE_EDITOR_LOAD_FONTS: &str = "ui_gallery.code_editor.load_fonts";
pub(crate) const CMD_CODE_EDITOR_DUMP_TAFFY: &str = "ui_gallery.code_editor.dump_taffy";

#[derive(Clone, Copy)]
pub(crate) struct PageSpec {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) title: &'static str,
    pub(crate) origin: &'static str,
    pub(crate) command: &'static str,
    pub(crate) tags: &'static [&'static str],
    pub(crate) docs_md: &'static str,
    pub(crate) usage_md: &'static str,
}

impl PageSpec {
    pub(crate) const fn new(
        id: &'static str,
        label: &'static str,
        title: &'static str,
        origin: &'static str,
        command: &'static str,
        tags: &'static [&'static str],
        docs_md: &'static str,
        usage_md: &'static str,
    ) -> Self {
        Self {
            id,
            label,
            title,
            origin,
            command,
            tags,
            docs_md,
            usage_md,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PageGroupSpec {
    pub(crate) title: &'static str,
    pub(crate) items: &'static [PageSpec],
}

pub(crate) static PAGE_GROUPS: &[PageGroupSpec] = &[
    PageGroupSpec {
        title: "Core",
        items: &[
            PageSpec::new(
                PAGE_INTRO,
                "Introduction",
                "Introduction",
                "Core contracts",
                CMD_NAV_INTRO,
                &["overview", "contracts"],
                docs::DOC_INTRO,
                docs::USAGE_INTRO,
            ),
            PageSpec::new(
                PAGE_LAYOUT,
                "Layout",
                "Layout / Stacks & Constraints",
                "Layout system",
                CMD_NAV_LAYOUT,
                &["layout", "flex", "stack"],
                docs::DOC_LAYOUT,
                docs::USAGE_LAYOUT,
            ),
            PageSpec::new(
                PAGE_VIEW_CACHE,
                "View Cache",
                "View Cache / Subtree Reuse",
                "fret-ui (runtime experiments)",
                CMD_NAV_VIEW_CACHE,
                &["cache", "performance", "gpui-parity"],
                docs::DOC_VIEW_CACHE,
                docs::USAGE_VIEW_CACHE,
            ),
            PageSpec::new(
                PAGE_HIT_TEST_TORTURE,
                "Hit Test (Torture)",
                "Hit Test / Spatial Index Harness",
                "fret-ui (hit testing)",
                CMD_NAV_HIT_TEST_TORTURE,
                &["hit_test", "pointer", "dispatch", "performance", "harness"],
                docs::DOC_HIT_TEST_TORTURE,
                docs::USAGE_HIT_TEST_TORTURE,
            ),
            PageSpec::new(
                PAGE_VIRTUAL_LIST_TORTURE,
                "Virtual List (Torture)",
                "Virtual List / Torture Harness",
                "fret-ui (virtualization contract)",
                CMD_NAV_VIRTUAL_LIST_TORTURE,
                &["virtual_list", "performance", "gpui-parity", "harness"],
                docs::DOC_VIRTUAL_LIST_TORTURE,
                docs::USAGE_VIRTUAL_LIST_TORTURE,
            ),
            PageSpec::new(
                PAGE_UI_KIT_LIST_TORTURE,
                "List (UI Kit Torture)",
                "List / UI Kit Retained Virtualization Harness",
                "fret-ui-kit (retained-host list surface)",
                CMD_NAV_UI_KIT_LIST_TORTURE,
                &[
                    "list",
                    "virtual_list",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_UI_KIT_LIST_TORTURE,
                docs::USAGE_UI_KIT_LIST_TORTURE,
            ),
            PageSpec::new(
                PAGE_CODE_VIEW_TORTURE,
                "Code View (Torture)",
                "Code View / Large Document Harness",
                "fret-code-view (windowed surface candidate)",
                CMD_NAV_CODE_VIEW_TORTURE,
                &[
                    "code",
                    "text",
                    "scroll",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_CODE_VIEW_TORTURE,
                docs::USAGE_CODE_VIEW_TORTURE,
            ),
            PageSpec::new(
                PAGE_CODE_EDITOR_MVP,
                "Code Editor (MVP)",
                "Code Editor / TextInputRegion MVP",
                "fret-code-editor (ecosystem surface)",
                CMD_NAV_CODE_EDITOR_MVP,
                &["code", "editor", "ime", "text-input", "windowed-rows"],
                docs::DOC_CODE_EDITOR_MVP,
                docs::USAGE_CODE_EDITOR_MVP,
            ),
            PageSpec::new(
                PAGE_CODE_EDITOR_TORTURE,
                "Code Editor (Torture)",
                "Code Editor / Scroll Stability Harness",
                "fret-code-editor (windowed surface + caching)",
                CMD_NAV_CODE_EDITOR_TORTURE,
                &[
                    "code",
                    "editor",
                    "scroll",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_CODE_EDITOR_TORTURE,
                docs::USAGE_CODE_EDITOR_TORTURE,
            ),
            PageSpec::new(
                PAGE_TEXT_SELECTION_PERF,
                "Text Selection (Perf)",
                "Text Selection / Selection Rect Culling",
                "Text integration workstream",
                CMD_NAV_TEXT_SELECTION_PERF,
                &["text", "selection", "performance", "diagnostics", "tli1"],
                docs::DOC_TEXT_SELECTION_PERF,
                docs::USAGE_TEXT_SELECTION_PERF,
            ),
            PageSpec::new(
                PAGE_TEXT_BIDI_RTL_CONFORMANCE,
                "Text BiDi/RTL",
                "Text / BiDi + RTL Conformance Harness",
                "Text integration workstream",
                CMD_NAV_TEXT_BIDI_RTL_CONFORMANCE,
                &["text", "bidi", "rtl", "geometry", "diagnostics", "tli1"],
                docs::DOC_TEXT_BIDI_RTL_CONFORMANCE,
                docs::USAGE_TEXT_BIDI_RTL_CONFORMANCE,
            ),
            PageSpec::new(
                PAGE_TEXT_MEASURE_OVERLAY,
                "Text Measure (Overlay)",
                "Text / Measured Bounds Overlay",
                "Text integration workstream",
                CMD_NAV_TEXT_MEASURE_OVERLAY,
                &["text", "layout", "measure", "diagnostics", "tli1"],
                docs::DOC_TEXT_MEASURE_OVERLAY,
                docs::USAGE_TEXT_MEASURE_OVERLAY,
            ),
            PageSpec::new(
                PAGE_WEB_IME_HARNESS,
                "Web IME (Harness)",
                "Web / IME + TextInput Bridge Harness",
                "fret-platform-web (textarea bridge, v1)",
                CMD_NAV_WEB_IME_HARNESS,
                &["web", "ime", "text-input", "wasm", "harness"],
                docs::DOC_WEB_IME_HARNESS,
                docs::USAGE_WEB_IME_HARNESS,
            ),
            PageSpec::new(
                PAGE_CHART_TORTURE,
                "Chart (Torture)",
                "Chart / Pan-Zoom Canvas Harness",
                "fret-chart + delinea (sampling/window candidate)",
                CMD_NAV_CHART_TORTURE,
                &[
                    "chart",
                    "plot",
                    "canvas",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_CHART_TORTURE,
                docs::USAGE_CHART_TORTURE,
            ),
            PageSpec::new(
                PAGE_CANVAS_CULL_TORTURE,
                "Canvas Cull (Torture)",
                "Canvas / Pan-Zoom Culling Harness",
                "fret-canvas (viewport culling candidate)",
                CMD_NAV_CANVAS_CULL_TORTURE,
                &[
                    "canvas",
                    "node_graph",
                    "culling",
                    "pan_zoom",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_CANVAS_CULL_TORTURE,
                docs::USAGE_CANVAS_CULL_TORTURE,
            ),
            PageSpec::new(
                PAGE_NODE_GRAPH_CULL_TORTURE,
                "Node Graph Cull (Torture)",
                "Node Graph / Pan-Zoom Culling Harness",
                "fret-node (viewport culling candidate)",
                CMD_NAV_NODE_GRAPH_CULL_TORTURE,
                &[
                    "node_graph",
                    "canvas",
                    "culling",
                    "pan_zoom",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_NODE_GRAPH_CULL_TORTURE,
                docs::USAGE_NODE_GRAPH_CULL_TORTURE,
            ),
            PageSpec::new(
                PAGE_CHROME_TORTURE,
                "Chrome (Torture)",
                "Chrome / Hover-Focus Overlay Harness",
                "fret-ui-shadcn + fret-ui (paint-only candidate)",
                CMD_NAV_CHROME_TORTURE,
                &[
                    "hover",
                    "focus",
                    "overlay",
                    "chrome",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_CHROME_TORTURE,
                docs::USAGE_CHROME_TORTURE,
            ),
            PageSpec::new(
                PAGE_WINDOWED_ROWS_SURFACE_TORTURE,
                "Windowed Rows Surface",
                "Windowed Rows Surface / Scroll + Canvas Harness",
                "fret-ui-kit (scroll + canvas pattern)",
                CMD_NAV_WINDOWED_ROWS_SURFACE_TORTURE,
                &["scroll", "performance", "gpui-parity", "canvas", "harness"],
                docs::DOC_WINDOWED_ROWS_SURFACE_TORTURE,
                docs::USAGE_WINDOWED_ROWS_SURFACE_TORTURE,
            ),
            PageSpec::new(
                PAGE_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE,
                "Windowed Rows (Interactive)",
                "Windowed Rows Surface / Pointer + Paint-only Chrome",
                "fret-ui-kit (windowed surface + pointer hit testing)",
                CMD_NAV_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE,
                &[
                    "scroll",
                    "performance",
                    "gpui-parity",
                    "canvas",
                    "pointer",
                    "harness",
                ],
                docs::DOC_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE,
                docs::USAGE_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE,
            ),
            PageSpec::new(
                PAGE_DATA_TABLE_TORTURE,
                "DataTable (Torture)",
                "DataTable / Virtualized Table Harness",
                "fret-ui-shadcn + fret-ui-kit (virtualized table)",
                CMD_NAV_DATA_TABLE_TORTURE,
                &[
                    "table",
                    "virtualized",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_DATA_TABLE_TORTURE,
                docs::USAGE_DATA_TABLE_TORTURE,
            ),
            PageSpec::new(
                PAGE_TREE_TORTURE,
                "Tree (Torture)",
                "Tree / Virtualized Tree Harness",
                "fret-ui-kit (virtualized tree)",
                CMD_NAV_TREE_TORTURE,
                &[
                    "tree",
                    "virtualized",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_TREE_TORTURE,
                docs::USAGE_TREE_TORTURE,
            ),
            PageSpec::new(
                PAGE_TABLE_RETAINED_TORTURE,
                "Table (Retained Torture)",
                "UI Kit Table / Retained Host Harness",
                "fret-ui-kit (virt-003 retained table v0)",
                CMD_NAV_TABLE_RETAINED_TORTURE,
                &[
                    "table",
                    "virtualized",
                    "retained",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_TABLE_RETAINED_TORTURE,
                docs::USAGE_TABLE_RETAINED_TORTURE,
            ),
            PageSpec::new(
                PAGE_AI_TRANSCRIPT_TORTURE,
                "AI Transcript (Torture)",
                "AI Transcript / Long Conversation Harness",
                "fret-ui-ai (conversation surface)",
                CMD_NAV_AI_TRANSCRIPT_TORTURE,
                &[
                    "ai",
                    "chat",
                    "conversation",
                    "scroll",
                    "virtualized",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_AI_TRANSCRIPT_TORTURE,
                docs::USAGE_AI_TRANSCRIPT_TORTURE,
            ),
            PageSpec::new(
                PAGE_AI_CHAT_DEMO,
                "AI Chat (Demo)",
                "AI Chat / Conversation + PromptInput Demo",
                "fret-ui-ai (chat surfaces)",
                CMD_NAV_AI_CHAT_DEMO,
                &[
                    "ai",
                    "chat",
                    "conversation",
                    "prompt",
                    "input",
                    "interaction",
                    "demo",
                ],
                docs::DOC_AI_CHAT_DEMO,
                docs::USAGE_AI_CHAT_DEMO,
            ),
            PageSpec::new(
                PAGE_INSPECTOR_TORTURE,
                "Inspector (Torture)",
                "Inspector / Property List Harness",
                "virtualized property list (retained host)",
                CMD_NAV_INSPECTOR_TORTURE,
                &[
                    "inspector",
                    "properties",
                    "outline",
                    "virtualized",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_INSPECTOR_TORTURE,
                docs::USAGE_INSPECTOR_TORTURE,
            ),
            PageSpec::new(
                PAGE_FILE_TREE_TORTURE,
                "File Tree (Torture)",
                "File Tree / Outline Harness",
                "virtualized tree rows (retained host)",
                CMD_NAV_FILE_TREE_TORTURE,
                &[
                    "file",
                    "tree",
                    "outline",
                    "virtualized",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
                docs::DOC_FILE_TREE_TORTURE,
                docs::USAGE_FILE_TREE_TORTURE,
            ),
        ],
    },
    PageGroupSpec {
        title: "Shadcn",
        items: &[
            PageSpec::new(
                PAGE_ACCORDION,
                "Accordion",
                "Accordion",
                "fret-ui-shadcn",
                CMD_NAV_ACCORDION,
                &["accordion", "collapsible"],
                docs::DOC_ACCORDION,
                docs::USAGE_ACCORDION,
            ),
            PageSpec::new(
                PAGE_ALERT,
                "Alert",
                "Alert",
                "fret-ui-shadcn",
                CMD_NAV_ALERT,
                &["alert", "feedback"],
                docs::DOC_ALERT,
                docs::USAGE_ALERT,
            ),
            PageSpec::new(
                PAGE_ALERT_DIALOG,
                "Alert Dialog",
                "Alert Dialog",
                "fret-ui-shadcn",
                CMD_NAV_ALERT_DIALOG,
                &["alert_dialog", "dialog", "overlay"],
                docs::DOC_ALERT_DIALOG,
                docs::USAGE_ALERT_DIALOG,
            ),
            PageSpec::new(
                PAGE_ASPECT_RATIO,
                "Aspect Ratio",
                "Aspect Ratio",
                "fret-ui-shadcn",
                CMD_NAV_ASPECT_RATIO,
                &["aspect_ratio", "layout"],
                docs::DOC_ASPECT_RATIO,
                docs::USAGE_ASPECT_RATIO,
            ),
            PageSpec::new(
                PAGE_AVATAR,
                "Avatar",
                "Avatar",
                "fret-ui-shadcn",
                CMD_NAV_AVATAR,
                &["avatar", "image", "fallback"],
                docs::DOC_AVATAR,
                docs::USAGE_AVATAR,
            ),
            PageSpec::new(
                PAGE_BADGE,
                "Badge",
                "Badge",
                "fret-ui-shadcn",
                CMD_NAV_BADGE,
                &["badge", "status", "tag"],
                docs::DOC_BADGE,
                docs::USAGE_BADGE,
            ),
            PageSpec::new(
                PAGE_BREADCRUMB,
                "Breadcrumb",
                "Breadcrumb",
                "fret-ui-shadcn",
                CMD_NAV_BREADCRUMB,
                &["breadcrumb", "navigation"],
                docs::DOC_BREADCRUMB,
                docs::USAGE_BREADCRUMB,
            ),
            PageSpec::new(
                PAGE_BUTTON,
                "Button",
                "Button",
                "fret-ui-shadcn",
                CMD_NAV_BUTTON,
                &["button", "variant"],
                docs::DOC_BUTTON,
                docs::USAGE_BUTTON,
            ),
            PageSpec::new(
                PAGE_BUTTON_GROUP,
                "Button Group",
                "Button Group",
                "fret-ui-shadcn",
                CMD_NAV_BUTTON_GROUP,
                &["button", "group"],
                docs::DOC_BUTTON_GROUP,
                docs::USAGE_BUTTON_GROUP,
            ),
            PageSpec::new(
                PAGE_CALENDAR,
                "Calendar",
                "Calendar",
                "fret-ui-shadcn",
                CMD_NAV_CALENDAR,
                &["calendar", "date"],
                docs::DOC_CALENDAR,
                docs::USAGE_CALENDAR,
            ),
            PageSpec::new(
                PAGE_CARD,
                "Card",
                "Card",
                "fret-ui-shadcn",
                CMD_NAV_CARD,
                &["card", "layout", "surface"],
                docs::DOC_CARD,
                docs::USAGE_CARD,
            ),
            PageSpec::new(
                PAGE_CAROUSEL,
                "Carousel",
                "Carousel",
                "fret-ui-shadcn",
                CMD_NAV_CAROUSEL,
                &["carousel", "scroll"],
                docs::DOC_CAROUSEL,
                docs::USAGE_CAROUSEL,
            ),
            PageSpec::new(
                PAGE_CHART,
                "Chart",
                "Chart",
                "fret-ui-shadcn",
                CMD_NAV_CHART,
                &["chart", "data_viz"],
                docs::DOC_CHART,
                docs::USAGE_CHART,
            ),
            PageSpec::new(
                PAGE_CHECKBOX,
                "Checkbox",
                "Checkbox",
                "fret-ui-shadcn",
                CMD_NAV_CHECKBOX,
                &["checkbox", "input"],
                docs::DOC_CHECKBOX,
                docs::USAGE_CHECKBOX,
            ),
            PageSpec::new(
                PAGE_COLLAPSIBLE,
                "Collapsible",
                "Collapsible",
                "fret-ui-shadcn",
                CMD_NAV_COLLAPSIBLE,
                &["collapsible", "disclosure"],
                docs::DOC_COLLAPSIBLE,
                docs::USAGE_COLLAPSIBLE,
            ),
            PageSpec::new(
                PAGE_COMBOBOX,
                "Combobox",
                "Combobox",
                "fret-ui-shadcn",
                CMD_NAV_COMBOBOX,
                &["combobox", "cmdk", "search"],
                docs::DOC_COMBOBOX,
                docs::USAGE_COMBOBOX,
            ),
            PageSpec::new(
                PAGE_COMMAND,
                "Command Palette",
                "Command Palette",
                "fret-ui-shadcn",
                CMD_NAV_COMMAND,
                &["cmdk", "command"],
                docs::DOC_COMMAND,
                docs::USAGE_COMMAND,
            ),
            PageSpec::new(
                PAGE_CONTEXT_MENU,
                "Context Menu",
                "Context Menu",
                "fret-ui-shadcn",
                CMD_NAV_CONTEXT_MENU,
                &["context_menu", "menu"],
                docs::DOC_CONTEXT_MENU,
                docs::USAGE_CONTEXT_MENU,
            ),
            PageSpec::new(
                PAGE_DATA_TABLE,
                "DataTable",
                "DataTable",
                "fret-ui-shadcn + fret-ui-headless",
                CMD_NAV_DATA_TABLE,
                &["table", "virtualized", "tanstack"],
                docs::DOC_DATA_TABLE,
                docs::USAGE_DATA_TABLE,
            ),
            PageSpec::new(
                PAGE_DATE_PICKER,
                "Date Picker",
                "Date Picker",
                "fret-ui-shadcn",
                CMD_NAV_DATE_PICKER,
                &["date", "calendar", "popover"],
                docs::DOC_DATE_PICKER,
                docs::USAGE_DATE_PICKER,
            ),
            PageSpec::new(
                PAGE_DIALOG,
                "Dialog",
                "Dialog",
                "fret-ui-shadcn",
                CMD_NAV_DIALOG,
                &["dialog", "overlay"],
                docs::DOC_DIALOG,
                docs::USAGE_DIALOG,
            ),
            PageSpec::new(
                PAGE_DRAWER,
                "Drawer",
                "Drawer",
                "fret-ui-shadcn",
                CMD_NAV_DRAWER,
                &["drawer", "overlay"],
                docs::DOC_DRAWER,
                docs::USAGE_DRAWER,
            ),
            PageSpec::new(
                PAGE_DROPDOWN_MENU,
                "Dropdown Menu",
                "Dropdown Menu",
                "fret-ui-shadcn",
                CMD_NAV_DROPDOWN_MENU,
                &["dropdown_menu", "menu"],
                docs::DOC_DROPDOWN_MENU,
                docs::USAGE_DROPDOWN_MENU,
            ),
            PageSpec::new(
                PAGE_EMPTY,
                "Empty",
                "Empty",
                "fret-ui-shadcn",
                CMD_NAV_EMPTY,
                &["empty", "state"],
                docs::DOC_EMPTY,
                docs::USAGE_EMPTY,
            ),
            PageSpec::new(
                PAGE_FIELD,
                "Field",
                "Field",
                "fret-ui-shadcn",
                CMD_NAV_FIELD,
                &["field", "form", "label", "error"],
                docs::DOC_FIELD,
                docs::USAGE_FIELD,
            ),
            PageSpec::new(
                PAGE_FORM,
                "Form",
                "Form",
                "fret-ui-shadcn",
                CMD_NAV_FORM,
                &["form", "field"],
                docs::DOC_FORM,
                docs::USAGE_FORM,
            ),
            PageSpec::new(
                PAGE_HOVER_CARD,
                "Hover Card",
                "Hover Card",
                "fret-ui-shadcn",
                CMD_NAV_HOVER_CARD,
                &["hover_card", "overlay"],
                docs::DOC_HOVER_CARD,
                docs::USAGE_HOVER_CARD,
            ),
            PageSpec::new(
                PAGE_INPUT,
                "Input",
                "Input",
                "fret-ui-shadcn",
                CMD_NAV_INPUT,
                &["input", "text"],
                docs::DOC_INPUT,
                docs::USAGE_INPUT,
            ),
            PageSpec::new(
                PAGE_INPUT_GROUP,
                "Input Group",
                "Input Group",
                "fret-ui-shadcn",
                CMD_NAV_INPUT_GROUP,
                &["input", "group"],
                docs::DOC_INPUT_GROUP,
                docs::USAGE_INPUT_GROUP,
            ),
            PageSpec::new(
                PAGE_INPUT_OTP,
                "Input OTP",
                "Input OTP",
                "fret-ui-shadcn",
                CMD_NAV_INPUT_OTP,
                &["input", "otp"],
                docs::DOC_INPUT_OTP,
                docs::USAGE_INPUT_OTP,
            ),
            PageSpec::new(
                PAGE_ITEM,
                "Item",
                "Item",
                "fret-ui-shadcn",
                CMD_NAV_ITEM,
                &["item", "layout"],
                docs::DOC_ITEM,
                docs::USAGE_ITEM,
            ),
            PageSpec::new(
                PAGE_KBD,
                "Kbd",
                "Kbd",
                "fret-ui-shadcn",
                CMD_NAV_KBD,
                &["kbd", "text"],
                docs::DOC_KBD,
                docs::USAGE_KBD,
            ),
            PageSpec::new(
                PAGE_LABEL,
                "Label",
                "Label",
                "fret-ui-shadcn",
                CMD_NAV_LABEL,
                &["label", "form"],
                docs::DOC_LABEL,
                docs::USAGE_LABEL,
            ),
            PageSpec::new(
                PAGE_MENUBAR,
                "Menubar",
                "Menubar",
                "fret-ui-shadcn",
                CMD_NAV_MENUBAR,
                &["menubar", "menu"],
                docs::DOC_MENUBAR,
                docs::USAGE_MENUBAR,
            ),
            PageSpec::new(
                PAGE_NATIVE_SELECT,
                "Native Select",
                "Native Select",
                "fret-ui-shadcn",
                CMD_NAV_NATIVE_SELECT,
                &["native_select", "select"],
                docs::DOC_NATIVE_SELECT,
                docs::USAGE_NATIVE_SELECT,
            ),
            PageSpec::new(
                PAGE_NAVIGATION_MENU,
                "Navigation Menu",
                "Navigation Menu",
                "fret-ui-shadcn",
                CMD_NAV_NAVIGATION_MENU,
                &["navigation_menu", "menu"],
                docs::DOC_NAVIGATION_MENU,
                docs::USAGE_NAVIGATION_MENU,
            ),
            PageSpec::new(
                PAGE_PAGINATION,
                "Pagination",
                "Pagination",
                "fret-ui-shadcn",
                CMD_NAV_PAGINATION,
                &["pagination"],
                docs::DOC_PAGINATION,
                docs::USAGE_PAGINATION,
            ),
            PageSpec::new(
                PAGE_POPOVER,
                "Popover",
                "Popover",
                "fret-ui-shadcn",
                CMD_NAV_POPOVER,
                &["popover", "overlay"],
                docs::DOC_POPOVER,
                docs::USAGE_POPOVER,
            ),
            PageSpec::new(
                PAGE_PROGRESS,
                "Progress",
                "Progress",
                "fret-ui-shadcn",
                CMD_NAV_PROGRESS,
                &["progress"],
                docs::DOC_PROGRESS,
                docs::USAGE_PROGRESS,
            ),
            PageSpec::new(
                PAGE_RADIO_GROUP,
                "Radio Group",
                "Radio Group",
                "fret-ui-shadcn",
                CMD_NAV_RADIO_GROUP,
                &["radio", "group"],
                docs::DOC_RADIO_GROUP,
                docs::USAGE_RADIO_GROUP,
            ),
            PageSpec::new(
                PAGE_RESIZABLE,
                "Resizable",
                "Resizable Panels",
                "fret-ui-shadcn",
                CMD_NAV_RESIZABLE,
                &["split", "panel", "resize"],
                docs::DOC_RESIZABLE,
                docs::USAGE_RESIZABLE,
            ),
            PageSpec::new(
                PAGE_SCROLL_AREA,
                "Scroll Area",
                "Scroll Area",
                "fret-ui-shadcn",
                CMD_NAV_SCROLL_AREA,
                &["scroll", "scrollbar", "virtual"],
                docs::DOC_SCROLL_AREA,
                docs::USAGE_SCROLL_AREA,
            ),
            PageSpec::new(
                PAGE_SELECT,
                "Select",
                "Select",
                "fret-ui-shadcn",
                CMD_NAV_SELECT,
                &["select", "popover", "listbox"],
                docs::DOC_SELECT,
                docs::USAGE_SELECT,
            ),
            PageSpec::new(
                PAGE_SEPARATOR,
                "Separator",
                "Separator",
                "fret-ui-shadcn",
                CMD_NAV_SEPARATOR,
                &["separator"],
                docs::DOC_SEPARATOR,
                docs::USAGE_SEPARATOR,
            ),
            PageSpec::new(
                PAGE_SHEET,
                "Sheet",
                "Sheet",
                "fret-ui-shadcn",
                CMD_NAV_SHEET,
                &["sheet", "overlay"],
                docs::DOC_SHEET,
                docs::USAGE_SHEET,
            ),
            PageSpec::new(
                PAGE_SIDEBAR,
                "Sidebar",
                "Sidebar",
                "fret-ui-shadcn",
                CMD_NAV_SIDEBAR,
                &["sidebar", "navigation"],
                docs::DOC_SIDEBAR,
                docs::USAGE_SIDEBAR,
            ),
            PageSpec::new(
                PAGE_SKELETON,
                "Skeleton",
                "Skeleton",
                "fret-ui-shadcn",
                CMD_NAV_SKELETON,
                &["skeleton", "loading", "animation"],
                docs::DOC_SKELETON,
                docs::USAGE_SKELETON,
            ),
            PageSpec::new(
                PAGE_SLIDER,
                "Slider",
                "Slider",
                "fret-ui-shadcn",
                CMD_NAV_SLIDER,
                &["slider", "range", "input"],
                docs::DOC_SLIDER,
                docs::USAGE_SLIDER,
            ),
            PageSpec::new(
                PAGE_SONNER,
                "Sonner",
                "Sonner",
                "fret-ui-shadcn",
                CMD_NAV_SONNER,
                &["sonner", "toast"],
                docs::DOC_SONNER,
                docs::USAGE_SONNER,
            ),
            PageSpec::new(
                PAGE_SPINNER,
                "Spinner",
                "Spinner",
                "fret-ui-shadcn",
                CMD_NAV_SPINNER,
                &["spinner", "loading"],
                docs::DOC_SPINNER,
                docs::USAGE_SPINNER,
            ),
            PageSpec::new(
                PAGE_SWITCH,
                "Switch",
                "Switch",
                "fret-ui-shadcn",
                CMD_NAV_SWITCH,
                &["switch", "input"],
                docs::DOC_SWITCH,
                docs::USAGE_SWITCH,
            ),
            PageSpec::new(
                PAGE_TABLE,
                "Table",
                "Table",
                "fret-ui-shadcn",
                CMD_NAV_TABLE,
                &["table", "grid"],
                docs::DOC_TABLE,
                docs::USAGE_TABLE,
            ),
            PageSpec::new(
                PAGE_TABS,
                "Tabs",
                "Tabs",
                "fret-ui-shadcn",
                CMD_NAV_TABS,
                &["tabs", "roving", "focus"],
                docs::DOC_TABS,
                docs::USAGE_TABS,
            ),
            PageSpec::new(
                PAGE_TEXTAREA,
                "Textarea",
                "Textarea",
                "fret-ui-shadcn",
                CMD_NAV_TEXTAREA,
                &["textarea", "input"],
                docs::DOC_TEXTAREA,
                docs::USAGE_TEXTAREA,
            ),
            PageSpec::new(
                PAGE_TOAST,
                "Toast",
                "Toast",
                "fret-ui-shadcn",
                CMD_NAV_TOAST,
                &["sonner", "toast"],
                docs::DOC_TOAST,
                docs::USAGE_TOAST,
            ),
            PageSpec::new(
                PAGE_TOGGLE,
                "Toggle",
                "Toggle",
                "fret-ui-shadcn",
                CMD_NAV_TOGGLE,
                &["toggle"],
                docs::DOC_TOGGLE,
                docs::USAGE_TOGGLE,
            ),
            PageSpec::new(
                PAGE_TOGGLE_GROUP,
                "Toggle Group",
                "Toggle Group",
                "fret-ui-shadcn",
                CMD_NAV_TOGGLE_GROUP,
                &["toggle_group"],
                docs::DOC_TOGGLE_GROUP,
                docs::USAGE_TOGGLE_GROUP,
            ),
            PageSpec::new(
                PAGE_TOOLTIP,
                "Tooltip",
                "Tooltip",
                "fret-ui-shadcn",
                CMD_NAV_TOOLTIP,
                &["tooltip", "overlay", "hover"],
                docs::DOC_TOOLTIP,
                docs::USAGE_TOOLTIP,
            ),
            PageSpec::new(
                PAGE_TYPOGRAPHY,
                "Typography",
                "Typography",
                "fret-ui-shadcn",
                CMD_NAV_TYPOGRAPHY,
                &["typography", "text"],
                docs::DOC_TYPOGRAPHY,
                docs::USAGE_TYPOGRAPHY,
            ),
        ],
    },
    PageGroupSpec {
        title: "Shadcn (Extras)",
        items: &[
            PageSpec::new(
                PAGE_DATA_GRID,
                "DataGrid",
                "DataGrid",
                "fret-ui-shadcn",
                CMD_NAV_DATA_GRID,
                &["grid", "viewport", "virtualized"],
                docs::DOC_DATA_GRID,
                docs::USAGE_DATA_GRID,
            ),
            PageSpec::new(
                PAGE_FORMS,
                "Forms",
                "Inputs / TextArea / Checkbox / Switch",
                "fret-ui-shadcn",
                CMD_NAV_FORMS,
                &["input", "textarea", "checkbox", "switch"],
                docs::DOC_FORMS,
                docs::USAGE_FORMS,
            ),
            PageSpec::new(
                PAGE_ICONS,
                "Icons",
                "Icons",
                "fret-icons + fret-icons-lucide",
                CMD_NAV_ICONS,
                &["icon", "svg", "lucide"],
                docs::DOC_ICONS,
                docs::USAGE_ICONS,
            ),
            PageSpec::new(
                PAGE_MENUS,
                "Menus",
                "Menus (Dropdown / Context)",
                "fret-ui-shadcn",
                CMD_NAV_MENUS,
                &["dropdown", "context-menu"],
                docs::DOC_MENUS,
                docs::USAGE_MENUS,
            ),
            PageSpec::new(
                PAGE_OVERLAY,
                "Overlay",
                "Overlay / Popover & Dialog",
                "Radix-shaped primitives",
                CMD_NAV_OVERLAY,
                &["dialog", "popover"],
                docs::DOC_OVERLAY,
                docs::USAGE_OVERLAY,
            ),
        ],
    },
    PageGroupSpec {
        title: "Material 3",
        items: &[
            PageSpec::new(
                PAGE_MATERIAL3_GALLERY,
                "Gallery",
                "Material 3 Gallery (outcomes-first snapshot surface)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_GALLERY,
                &["material3", "gallery", "regression", "outcomes"],
                docs::DOC_MATERIAL3_GALLERY,
                docs::USAGE_MATERIAL3_GALLERY,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TOP_APP_BAR,
                "Top App Bar",
                "Material 3 Top App Bar (primitives)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TOP_APP_BAR,
                &["material3", "top-app-bar", "toolbar", "app-bar"],
                docs::DOC_MATERIAL3_TOP_APP_BAR,
                docs::USAGE_MATERIAL3_TOP_APP_BAR,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_BOTTOM_SHEET,
                "Bottom Sheet",
                "Material 3 Bottom Sheet (modal + standard)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_BOTTOM_SHEET,
                &["material3", "bottom-sheet", "sheet", "overlay"],
                docs::DOC_MATERIAL3_BOTTOM_SHEET,
                docs::USAGE_MATERIAL3_BOTTOM_SHEET,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_DATE_PICKER,
                "Date Picker",
                "Material 3 Date Picker (modal + docked)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_DATE_PICKER,
                &["material3", "date-picker", "calendar", "overlay"],
                docs::DOC_MATERIAL3_DATE_PICKER,
                docs::USAGE_MATERIAL3_DATE_PICKER,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TIME_PICKER,
                "Time Picker",
                "Material 3 Time Picker (modal + docked)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TIME_PICKER,
                &["material3", "time-picker", "clock", "overlay"],
                docs::DOC_MATERIAL3_TIME_PICKER,
                docs::USAGE_MATERIAL3_TIME_PICKER,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_AUTOCOMPLETE,
                "Autocomplete",
                "Material 3 Autocomplete (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_AUTOCOMPLETE,
                &[
                    "material3",
                    "autocomplete",
                    "combobox",
                    "listbox",
                    "overlay",
                    "a11y",
                ],
                docs::DOC_MATERIAL3_AUTOCOMPLETE,
                docs::USAGE_MATERIAL3_AUTOCOMPLETE,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_STATE_MATRIX,
                "State Matrix",
                "Material 3 State Matrix (manual regression harness)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_STATE_MATRIX,
                &["material3", "states", "regression", "matrix"],
                docs::DOC_MATERIAL3_STATE_MATRIX,
                docs::USAGE_MATERIAL3_STATE_MATRIX,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TOUCH_TARGETS,
                "Touch Targets",
                "Material 3 Touch Targets (minimum interactive size)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TOUCH_TARGETS,
                &[
                    "material3",
                    "touch-target",
                    "interactive-size",
                    "regression",
                ],
                docs::DOC_MATERIAL3_TOUCH_TARGETS,
                docs::USAGE_MATERIAL3_TOUCH_TARGETS,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_BUTTON,
                "Button",
                "Material 3 Button (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_BUTTON,
                &["material3", "button", "state-layer", "ripple", "motion"],
                docs::DOC_MATERIAL3_BUTTON,
                docs::USAGE_MATERIAL3_BUTTON,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_ICON_BUTTON,
                "Icon Button",
                "Material 3 Icon Button (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_ICON_BUTTON,
                &[
                    "material3",
                    "icon-button",
                    "state-layer",
                    "ripple",
                    "motion",
                ],
                docs::DOC_MATERIAL3_ICON_BUTTON,
                docs::USAGE_MATERIAL3_ICON_BUTTON,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_CHECKBOX,
                "Checkbox",
                "Material 3 Checkbox (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_CHECKBOX,
                &["material3", "checkbox", "state-layer", "ripple", "forms"],
                docs::DOC_MATERIAL3_CHECKBOX,
                docs::USAGE_MATERIAL3_CHECKBOX,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_SWITCH,
                "Switch",
                "Material 3 Switch (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_SWITCH,
                &["material3", "switch", "state-layer", "ripple", "forms"],
                docs::DOC_MATERIAL3_SWITCH,
                docs::USAGE_MATERIAL3_SWITCH,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_RADIO,
                "Radio",
                "Material 3 Radio (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_RADIO,
                &["material3", "radio", "state-layer", "ripple", "forms"],
                docs::DOC_MATERIAL3_RADIO,
                docs::USAGE_MATERIAL3_RADIO,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_BADGE,
                "Badge",
                "Material 3 Badge (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_BADGE,
                &["material3", "badge", "status", "navigation"],
                docs::DOC_MATERIAL3_BADGE,
                docs::USAGE_MATERIAL3_BADGE,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_SEGMENTED_BUTTON,
                "Segmented Button",
                "Material 3 Segmented Button (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_SEGMENTED_BUTTON,
                &["material3", "segmented-button", "roving-focus", "selection"],
                docs::DOC_MATERIAL3_SEGMENTED_BUTTON,
                docs::USAGE_MATERIAL3_SEGMENTED_BUTTON,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_SELECT,
                "Select",
                "Material 3 Select (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_SELECT,
                &["material3", "select", "listbox", "forms", "overlay"],
                docs::DOC_MATERIAL3_SELECT,
                docs::USAGE_MATERIAL3_SELECT,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TEXT_FIELD,
                "Text Field",
                "Material 3 Text Field (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TEXT_FIELD,
                &["material3", "text-field", "forms"],
                docs::DOC_MATERIAL3_TEXT_FIELD,
                docs::USAGE_MATERIAL3_TEXT_FIELD,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TABS,
                "Tabs",
                "Material 3 Tabs (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TABS,
                &["material3", "tabs", "state-layer", "ripple", "roving-focus"],
                docs::DOC_MATERIAL3_TABS,
                docs::USAGE_MATERIAL3_TABS,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_NAVIGATION_BAR,
                "Navigation Bar",
                "Material 3 Navigation Bar (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_NAVIGATION_BAR,
                &[
                    "material3",
                    "navigation-bar",
                    "state-layer",
                    "ripple",
                    "roving-focus",
                    "motion",
                ],
                docs::DOC_MATERIAL3_NAVIGATION_BAR,
                docs::USAGE_MATERIAL3_NAVIGATION_BAR,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_NAVIGATION_RAIL,
                "Navigation Rail",
                "Material 3 Navigation Rail (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_NAVIGATION_RAIL,
                &[
                    "material3",
                    "navigation-rail",
                    "state-layer",
                    "ripple",
                    "roving-focus",
                    "motion",
                ],
                docs::DOC_MATERIAL3_NAVIGATION_RAIL,
                docs::USAGE_MATERIAL3_NAVIGATION_RAIL,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_NAVIGATION_DRAWER,
                "Navigation Drawer",
                "Material 3 Navigation Drawer (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_NAVIGATION_DRAWER,
                &[
                    "material3",
                    "navigation-drawer",
                    "state-layer",
                    "ripple",
                    "roving-focus",
                ],
                docs::DOC_MATERIAL3_NAVIGATION_DRAWER,
                docs::USAGE_MATERIAL3_NAVIGATION_DRAWER,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER,
                "Modal Navigation Drawer",
                "Material 3 Modal Navigation Drawer (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_MODAL_NAVIGATION_DRAWER,
                &[
                    "material3",
                    "navigation-drawer",
                    "modal",
                    "overlay",
                    "scrim",
                    "focus-trap",
                    "motion",
                ],
                docs::DOC_MATERIAL3_MODAL_NAVIGATION_DRAWER,
                docs::USAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_DIALOG,
                "Dialog",
                "Material 3 Dialog (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_DIALOG,
                &[
                    "material3",
                    "dialog",
                    "modal",
                    "overlay",
                    "scrim",
                    "focus-trap",
                    "motion",
                ],
                docs::DOC_MATERIAL3_DIALOG,
                docs::USAGE_MATERIAL3_DIALOG,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_MENU,
                "Menu",
                "Material 3 Menu (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_MENU,
                &[
                    "material3",
                    "menu",
                    "list",
                    "state-layer",
                    "ripple",
                    "roving-focus",
                    "typeahead",
                ],
                docs::DOC_MATERIAL3_MENU,
                docs::USAGE_MATERIAL3_MENU,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_LIST,
                "List",
                "Material 3 List (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_LIST,
                &["material3", "list", "roving-focus", "selection"],
                docs::DOC_MATERIAL3_LIST,
                docs::USAGE_MATERIAL3_LIST,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_SNACKBAR,
                "Snackbar",
                "Material 3 Snackbar (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_SNACKBAR,
                &["material3", "snackbar", "toast-layer"],
                docs::DOC_MATERIAL3_SNACKBAR,
                docs::USAGE_MATERIAL3_SNACKBAR,
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TOOLTIP,
                "Tooltip",
                "Material 3 Tooltip (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TOOLTIP,
                &["material3", "tooltip", "overlay", "motion"],
                docs::DOC_MATERIAL3_TOOLTIP,
                docs::USAGE_MATERIAL3_TOOLTIP,
            ),
        ],
    },
];

pub(crate) fn page_spec(id: &str) -> Option<&'static PageSpec> {
    PAGE_GROUPS
        .iter()
        .flat_map(|group| group.items.iter())
        .find(|item| item.id == id)
}

pub(crate) fn page_id_for_nav_command(command: &str) -> Option<&'static str> {
    static BY_COMMAND: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let by_command = BY_COMMAND.get_or_init(|| {
        let mut map: HashMap<&'static str, &'static str> = HashMap::new();
        for group in PAGE_GROUPS {
            for page in group.items {
                map.insert(page.command, page.id);
            }
        }
        map
    });

    by_command.get(command).copied()
}

struct DataGridCommands {
    by_row: Vec<CommandId>,
    row_by_command: HashMap<Arc<str>, u64>,
}

fn data_grid_commands() -> &'static DataGridCommands {
    static COMMANDS: OnceLock<DataGridCommands> = OnceLock::new();
    COMMANDS.get_or_init(|| {
        let mut by_row: Vec<CommandId> = Vec::with_capacity(DATA_GRID_ROWS);
        let mut row_by_command: HashMap<Arc<str>, u64> = HashMap::with_capacity(DATA_GRID_ROWS);

        for row in 0..DATA_GRID_ROWS {
            let cmd = CommandId::new(format!("{CMD_DATA_GRID_ROW_PREFIX}{row}"));
            row_by_command.insert(cmd.0.clone(), row as u64);
            by_row.push(cmd);
        }

        DataGridCommands {
            by_row,
            row_by_command,
        }
    })
}

pub(crate) fn data_grid_row_command(row: usize) -> Option<CommandId> {
    data_grid_commands().by_row.get(row).cloned()
}

pub(crate) fn data_grid_row_for_command(command: &str) -> Option<u64> {
    data_grid_commands().row_by_command.get(command).copied()
}

pub(crate) fn page_meta(
    selected: &str,
) -> (&'static str, &'static str, &'static str, &'static str) {
    let fallback = page_spec(PAGE_INTRO).expect("intro page exists");
    let page = page_spec(selected).unwrap_or(fallback);
    (page.title, page.origin, page.docs_md, page.usage_md)
}
