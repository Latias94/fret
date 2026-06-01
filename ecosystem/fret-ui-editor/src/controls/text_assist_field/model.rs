//! Text assist field public option records and rendered panel handoff record.

use std::sync::Arc;

use fret_core::Px;
use fret_ui::GlobalElementId;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::AnyElement;
use fret_ui_kit::headless::text_assist::{InputOwnedTextAssistKeyOptions, TextAssistMatch};

use crate::controls::TextFieldOptions;

#[cfg(test)]
mod tests;

pub type OnTextAssistFieldAccept =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, TextAssistMatch) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAssistFieldSurface {
    #[default]
    Inline,
    AnchoredOverlay,
}

#[derive(Debug, Clone)]
pub struct TextAssistFieldOptions {
    /// Base `TextField` options for the owning input.
    ///
    /// Note: this recipe currently forces `buffered = false` because its input-owned key policy
    /// reads and writes the bound query model directly.
    pub field: TextFieldOptions,
    pub surface: TextAssistFieldSurface,
    pub list_label: Arc<str>,
    pub empty_label: Arc<str>,
    pub key_options: InputOwnedTextAssistKeyOptions,
    pub list_test_id: Option<Arc<str>>,
    pub item_test_id_prefix: Option<Arc<str>>,
    pub empty_test_id: Option<Arc<str>>,
    /// Maximum visible list content height before the recipe introduces scrolling.
    ///
    /// For anchored overlays, leaving this unset still applies a conservative editor default so
    /// the popup does not grow to the full window height.
    pub max_list_height: Option<Px>,
}

impl Default for TextAssistFieldOptions {
    fn default() -> Self {
        let field = TextFieldOptions {
            buffered: false,
            ..Default::default()
        };
        Self {
            field,
            surface: TextAssistFieldSurface::Inline,
            list_label: Arc::from("Suggestions"),
            empty_label: Arc::from("No matches"),
            key_options: InputOwnedTextAssistKeyOptions::default(),
            list_test_id: None,
            item_test_id_prefix: None,
            empty_test_id: None,
            max_list_height: None,
        }
    }
}

pub(super) struct RenderedTextAssistPanel {
    pub(super) panel: AnyElement,
    pub(super) listbox_id: Option<GlobalElementId>,
    pub(super) option_elements: Vec<GlobalElementId>,
    pub(super) surface_height: Px,
}
