use std::sync::{Arc, OnceLock};

use crate::docs;

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

pub(crate) const CMD_NAV_SELECT_PREFIX: &str = "ui_gallery.nav.select.";
pub(crate) const CMD_DATA_GRID_ROW_PREFIX: &str = "ui_gallery.data_grid.row.";

pub(crate) const PAGE_INTRO: &str = "intro";
pub(crate) const PAGE_LAYOUT: &str = "layout";
pub(crate) const PAGE_VIEW_CACHE: &str = "view_cache";
pub(crate) const PAGE_VIRTUAL_LIST_TORTURE: &str = "virtual_list_torture";
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
pub(crate) const PAGE_MATERIAL3_BUTTON: &str = "material3_button";
pub(crate) const PAGE_MATERIAL3_ICON_BUTTON: &str = "material3_icon_button";
pub(crate) const PAGE_MATERIAL3_CHECKBOX: &str = "material3_checkbox";
pub(crate) const PAGE_MATERIAL3_SWITCH: &str = "material3_switch";
pub(crate) const PAGE_MATERIAL3_RADIO: &str = "material3_radio";
pub(crate) const PAGE_MATERIAL3_TEXT_FIELD: &str = "material3_text_field";
pub(crate) const PAGE_MATERIAL3_TABS: &str = "material3_tabs";
pub(crate) const PAGE_MATERIAL3_NAVIGATION_BAR: &str = "material3_navigation_bar";
pub(crate) const PAGE_MATERIAL3_NAVIGATION_RAIL: &str = "material3_navigation_rail";
pub(crate) const PAGE_MATERIAL3_NAVIGATION_DRAWER: &str = "material3_navigation_drawer";
pub(crate) const PAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER: &str = "material3_modal_navigation_drawer";
pub(crate) const PAGE_MATERIAL3_DIALOG: &str = "material3_dialog";
pub(crate) const PAGE_MATERIAL3_MENU: &str = "material3_menu";
pub(crate) const PAGE_MATERIAL3_STATE_MATRIX: &str = "material3_state_matrix";

pub(crate) const CMD_NAV_INTRO: &str = "ui_gallery.nav.select.intro";
pub(crate) const CMD_NAV_LAYOUT: &str = "ui_gallery.nav.select.layout";
pub(crate) const CMD_NAV_VIEW_CACHE: &str = "ui_gallery.nav.select.view_cache";
pub(crate) const CMD_NAV_VIRTUAL_LIST_TORTURE: &str = "ui_gallery.nav.select.virtual_list_torture";
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
pub(crate) const CMD_NAV_MATERIAL3_BUTTON: &str = "ui_gallery.nav.select.material3_button";
pub(crate) const CMD_NAV_MATERIAL3_ICON_BUTTON: &str =
    "ui_gallery.nav.select.material3_icon_button";
pub(crate) const CMD_NAV_MATERIAL3_CHECKBOX: &str = "ui_gallery.nav.select.material3_checkbox";
pub(crate) const CMD_NAV_MATERIAL3_SWITCH: &str = "ui_gallery.nav.select.material3_switch";
pub(crate) const CMD_NAV_MATERIAL3_RADIO: &str = "ui_gallery.nav.select.material3_radio";
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
pub(crate) const CMD_NAV_MATERIAL3_STATE_MATRIX: &str =
    "ui_gallery.nav.select.material3_state_matrix";

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
pub(crate) const CMD_VIRTUAL_LIST_TORTURE_ROW_EDIT_PREFIX: &str =
    "ui_gallery.virtual_list_torture.row.edit.";

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
                PAGE_VIRTUAL_LIST_TORTURE,
                "Virtual List (Torture)",
                "Virtual List / Torture Harness",
                "fret-ui (virtualization contract)",
                CMD_NAV_VIRTUAL_LIST_TORTURE,
                &["virtual_list", "performance", "gpui-parity", "harness"],
                docs::DOC_VIRTUAL_LIST_TORTURE,
                docs::USAGE_VIRTUAL_LIST_TORTURE,
            ),
        ],
    },
    PageGroupSpec {
        title: "Shadcn",
        items: &[
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
        ],
    },
];

pub(crate) fn page_spec(id: &str) -> Option<&'static PageSpec> {
    PAGE_GROUPS
        .iter()
        .flat_map(|group| group.items.iter())
        .find(|item| item.id == id)
}

pub(crate) fn page_meta(
    selected: &str,
) -> (&'static str, &'static str, &'static str, &'static str) {
    let fallback = page_spec(PAGE_INTRO).expect("intro page exists");
    let page = page_spec(selected).unwrap_or(fallback);
    (page.title, page.origin, page.docs_md, page.usage_md)
}
