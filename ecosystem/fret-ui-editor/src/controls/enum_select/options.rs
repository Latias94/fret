use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

#[derive(Debug, Clone)]
pub struct EnumSelectOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub focusable: bool,
    pub placeholder: Arc<str>,
    pub none_label: Arc<str>,
    pub max_list_height: Option<Px>,
    pub a11y_label: Option<Arc<str>>,
    /// Explicit identity source for internal state (open/filter models, overlay root ids).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple enum selects from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub list_test_id: Option<Arc<str>>,
    pub search_test_id: Option<Arc<str>>,
}

impl Default for EnumSelectOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            placeholder: Arc::from("Select…"),
            // In inspectors, `None` often means "mixed/indeterminate" rather than "unset".
            none_label: Arc::from("Mixed"),
            max_list_height: None,
            a11y_label: None,
            id_source: None,
            test_id: None,
            list_test_id: None,
            search_test_id: None,
        }
    }
}
