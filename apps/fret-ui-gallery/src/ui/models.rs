use fret_app::Model;
use fret_core::ImageId;
#[cfg(feature = "gallery-dev")]
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;
use time::Date;

#[derive(Clone)]
pub(crate) struct UiGalleryModels {
    pub(crate) theme_preset: Model<Option<Arc<str>>>,
    pub(crate) theme_preset_open: Model<bool>,
    pub(crate) motion_preset: Model<Option<Arc<str>>>,
    pub(crate) motion_preset_open: Model<bool>,

    pub(crate) view_cache_enabled: Model<bool>,
    pub(crate) view_cache_cache_shell: Model<bool>,
    pub(crate) view_cache_cache_content: Model<bool>,
    pub(crate) view_cache_inner_enabled: Model<bool>,
    pub(crate) view_cache_popover_open: Model<bool>,
    pub(crate) view_cache_continuous: Model<bool>,
    pub(crate) view_cache_counter: Model<u64>,

    #[cfg(feature = "gallery-dev")]
    pub(crate) popover_open: Model<bool>,
    pub(crate) dialog_open: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) dialog_glass_open: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) alert_dialog_open: Model<bool>,
    #[cfg(any(feature = "gallery-dev", feature = "gallery-material3"))]
    pub(crate) sheet_open: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) portal_geometry_popover_open: Model<bool>,

    pub(crate) combobox_value: Model<Option<Arc<str>>>,
    pub(crate) combobox_open: Model<bool>,
    pub(crate) combobox_query: Model<String>,

    pub(crate) date_picker_open: Model<bool>,
    pub(crate) date_picker_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(crate) date_picker_selected: Model<Option<Date>>,

    pub(crate) data_table_state: Model<fret_ui_headless::table::TableState>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) data_grid_selected_row: Model<Option<u64>>,

    pub(crate) tabs_value: Model<Option<Arc<str>>>,
    pub(crate) accordion_value: Model<Option<Arc<str>>>,

    pub(crate) avatar_demo_image: Model<Option<ImageId>>,
    pub(crate) image_fit_demo_wide_image: Model<Option<ImageId>>,
    pub(crate) image_fit_demo_tall_image: Model<Option<ImageId>>,
    pub(crate) image_fit_demo_streaming_image: Model<Option<ImageId>>,

    pub(crate) progress: Model<f32>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) checkbox: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) switch: Model<bool>,

    #[cfg(feature = "gallery-material3")]
    pub(crate) material3_expressive: Model<bool>,

    pub(crate) text_input: Model<String>,
    pub(crate) text_area: Model<String>,
    pub(crate) input_file_value: Model<String>,

    #[cfg(feature = "gallery-dev")]
    pub(crate) dropdown_open: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) context_menu_open: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) context_menu_edge_open: Model<bool>,

    pub(crate) cmdk_open: Model<bool>,
    pub(crate) cmdk_query: Model<String>,

    pub(crate) last_action: Model<Arc<str>>,
    pub(crate) sonner_position: Model<shadcn::ToastPosition>,

    #[cfg(feature = "gallery-dev")]
    pub(crate) virtual_list_torture_jump: Model<String>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) virtual_list_torture_edit_row: Model<Option<u64>>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) virtual_list_torture_edit_text: Model<String>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) virtual_list_torture_scroll: VirtualListScrollHandle,

    #[cfg(feature = "gallery-dev")]
    pub(crate) code_editor_syntax_rust: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) code_editor_boundary_identifier: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) code_editor_soft_wrap: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) code_editor_folds: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    pub(crate) code_editor_inlays: Model<bool>,

    #[cfg(feature = "gallery-dev")]
    pub(crate) markdown_link_gate_last_activation: Model<Option<Arc<str>>>,
}
