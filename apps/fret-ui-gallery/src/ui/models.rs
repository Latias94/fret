use fret_app::Model;
use fret_core::ImageId;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui_shadcn as shadcn;
use std::sync::Arc;
use time::Date;

#[derive(Clone)]
pub(crate) struct UiGalleryModels {
    pub(crate) content_tab: Model<Option<Arc<str>>>,

    pub(crate) theme_preset: Model<Option<Arc<str>>>,
    pub(crate) theme_preset_open: Model<bool>,

    pub(crate) view_cache_enabled: Model<bool>,
    pub(crate) view_cache_cache_shell: Model<bool>,
    pub(crate) view_cache_inner_enabled: Model<bool>,
    pub(crate) view_cache_popover_open: Model<bool>,
    pub(crate) view_cache_continuous: Model<bool>,
    pub(crate) view_cache_counter: Model<u64>,

    pub(crate) popover_open: Model<bool>,
    pub(crate) dialog_open: Model<bool>,
    pub(crate) alert_dialog_open: Model<bool>,
    pub(crate) sheet_open: Model<bool>,
    pub(crate) portal_geometry_popover_open: Model<bool>,

    pub(crate) select_value: Model<Option<Arc<str>>>,
    pub(crate) select_open: Model<bool>,

    pub(crate) combobox_value: Model<Option<Arc<str>>>,
    pub(crate) combobox_open: Model<bool>,
    pub(crate) combobox_query: Model<String>,

    pub(crate) date_picker_open: Model<bool>,
    pub(crate) date_picker_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(crate) date_picker_selected: Model<Option<Date>>,

    pub(crate) time_picker_open: Model<bool>,
    pub(crate) time_picker_selected: Model<time::Time>,

    pub(crate) resizable_h_fractions: Model<Vec<f32>>,
    pub(crate) resizable_v_fractions: Model<Vec<f32>>,

    pub(crate) data_table_state: Model<fret_ui_headless::table::TableState>,
    pub(crate) data_grid_selected_row: Model<Option<u64>>,

    pub(crate) tabs_value: Model<Option<Arc<str>>>,
    pub(crate) accordion_value: Model<Option<Arc<str>>>,

    pub(crate) avatar_demo_image: Model<Option<ImageId>>,
    pub(crate) image_fit_demo_wide_image: Model<Option<ImageId>>,
    pub(crate) image_fit_demo_tall_image: Model<Option<ImageId>>,
    pub(crate) image_fit_demo_streaming_image: Model<Option<ImageId>>,

    pub(crate) progress: Model<f32>,
    pub(crate) checkbox: Model<bool>,
    pub(crate) switch: Model<bool>,

    pub(crate) material3_checkbox: Model<bool>,
    pub(crate) material3_switch: Model<bool>,
    pub(crate) material3_radio_value: Model<Option<Arc<str>>>,
    pub(crate) material3_tabs_value: Model<Arc<str>>,
    pub(crate) material3_list_value: Model<Arc<str>>,
    pub(crate) material3_expressive: Model<bool>,
    pub(crate) material3_navigation_bar_value: Model<Arc<str>>,
    pub(crate) material3_navigation_rail_value: Model<Arc<str>>,
    pub(crate) material3_navigation_drawer_value: Model<Arc<str>>,
    pub(crate) material3_modal_navigation_drawer_open: Model<bool>,
    pub(crate) material3_dialog_open: Model<bool>,
    pub(crate) material3_text_field_value: Model<String>,
    pub(crate) material3_text_field_disabled: Model<bool>,
    pub(crate) material3_text_field_error: Model<bool>,
    pub(crate) material3_autocomplete_value: Model<String>,
    pub(crate) material3_autocomplete_disabled: Model<bool>,
    pub(crate) material3_autocomplete_error: Model<bool>,
    pub(crate) material3_autocomplete_dialog_open: Model<bool>,
    pub(crate) material3_menu_open: Model<bool>,

    pub(crate) text_input: Model<String>,
    pub(crate) text_area: Model<String>,

    pub(crate) dropdown_open: Model<bool>,
    pub(crate) context_menu_open: Model<bool>,
    pub(crate) context_menu_edge_open: Model<bool>,

    pub(crate) cmdk_open: Model<bool>,
    pub(crate) cmdk_query: Model<String>,

    pub(crate) last_action: Model<Arc<str>>,
    pub(crate) sonner_position: Model<shadcn::ToastPosition>,

    pub(crate) virtual_list_torture_jump: Model<String>,
    pub(crate) virtual_list_torture_edit_row: Model<Option<u64>>,
    pub(crate) virtual_list_torture_edit_text: Model<String>,
    pub(crate) virtual_list_torture_scroll: VirtualListScrollHandle,

    pub(crate) code_editor_syntax_rust: Model<bool>,
    pub(crate) code_editor_boundary_identifier: Model<bool>,
    pub(crate) code_editor_soft_wrap: Model<bool>,
    pub(crate) code_editor_folds: Model<bool>,
    pub(crate) code_editor_inlays: Model<bool>,
}
