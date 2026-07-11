//! Reusable editor text field control.
//!
//! Scope:
//! - single-line input (`TextInput`)
//! - optional multiline mode (`TextArea`) with a minimum height
//! - optional clear affordance

mod buffered;
mod element;
#[cfg(test)]
mod tests;

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{NodeId, Px};
use fret_runtime::{CommandId, Model};
use fret_ui::GlobalElementId;
use fret_ui::action::{ActionCx, UiFocusActionHost};
use fret_ui::element::{LayoutStyle, Length, SizeStyle};
use fret_ui_kit::Size;

use crate::primitives::EditSessionOutcome;
use crate::primitives::text_entry::{EditorTextCancelBehavior, EditorTextSelectionBehavior};

pub use buffered::TextFieldDraftController;
pub use buffered::TextFieldDraftSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFieldMode {
    #[default]
    PlainText,
    Password,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFieldBlurBehavior {
    /// Accept the current draft when focus leaves a buffered field.
    #[default]
    Commit,
    /// Restore the pre-edit value when focus leaves a buffered field.
    Cancel,
    /// Leave the draft session open even after blur.
    ///
    /// This preserves the old deferred policy for specialized surfaces that want an external
    /// owner to decide how blur should finish the session.
    PreserveDraft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextFieldAssistiveSemantics {
    /// Active-descendant semantics for an assistive surface such as completion or history.
    pub active_descendant: Option<NodeId>,
    /// Declarative element id for the current active assistive option.
    pub active_descendant_element: Option<u64>,
    /// Declarative element id for an assistive surface controlled by this field.
    pub controls_element: Option<u64>,
    /// Whether the assistive surface is currently expanded.
    pub expanded: Option<bool>,
}

pub type TextFieldOutcome = EditSessionOutcome;
pub type OnTextFieldOutcome =
    Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, TextFieldOutcome) + 'static>;

#[derive(Debug, Clone)]
pub struct TextFieldOptions {
    pub layout: LayoutStyle,
    pub size: Size,
    pub placeholder: Option<Arc<str>>,
    /// Explicit identity source for internal buffered state.
    ///
    /// Use this when helper code renders multiple text fields from the same callsite and model
    /// identity alone is not enough to distinguish their edit sessions.
    pub id_source: Option<Arc<str>>,
    pub enabled: bool,
    pub focusable: bool,
    pub clear_button: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub clear_test_id: Option<Arc<str>>,
    /// Optional sink for the outer joined-field root id.
    ///
    /// Recipes such as anchored completion/history popups use this to keep the whole field chrome
    /// in the same dismissable branch even when the assistive surface is anchored to the input.
    pub field_id_out: Option<Rc<Cell<Option<GlobalElementId>>>>,
    /// Optional sink for the inner text-entry element id.
    ///
    /// Recipes use this to anchor input-owned assistive surfaces and restore focus to the actual
    /// text entry node rather than the surrounding field container.
    pub input_id_out: Option<Rc<Cell<Option<GlobalElementId>>>>,
    /// Visual text mode for the single-line input surface.
    ///
    /// Password mode currently maps to `TextInputProps::obscure_text`; multiline text areas remain
    /// plain text until editor-owned multiline/password policy is defined.
    pub mode: TextFieldMode,
    /// When true, single-line fields edit a local draft and only commit on explicit accept.
    ///
    /// Both single-line and multiline editor fields can use a local draft session. Multiline
    /// commit remains editor-owned: blur commits by default, while Ctrl/Cmd+Enter acts as an
    /// explicit commit shortcut.
    pub buffered: bool,
    /// How a buffered field should finish its local draft when focus leaves the editing surface.
    pub blur_behavior: TextFieldBlurBehavior,
    /// Opaque operation handle for app-authored commit/discard controls.
    ///
    /// This intentionally does not expose the internal draft model. Use it only when a buffered
    /// field with preserved draft behavior needs explicit external actions in the same editor
    /// surface.
    pub draft_controller: Option<TextFieldDraftController>,
    /// Optional submit command for Enter on single-line text inputs.
    ///
    /// For buffered fields this runs after the local draft has been committed into the bound
    /// model. Multiline text areas intentionally do not route Enter through this field today.
    pub submit_command: Option<CommandId>,
    /// Placeholder semantics for completion/history popups owned outside the field.
    ///
    /// This does not implement those surfaces; it only exposes the relationship hooks needed to
    /// wire them later without changing the public editor control surface again.
    pub assistive_semantics: TextFieldAssistiveSemantics,
    pub selection_behavior: EditorTextSelectionBehavior,
    pub cancel_behavior: EditorTextCancelBehavior,

    /// When true, uses `TextArea` (multiline) instead of `TextInput`.
    pub multiline: bool,
    /// If true, opt into stable multiline line boxes (fixed line height + forced strut).
    ///
    /// This is intended for UI/form surfaces where baseline stability matters more than avoiding
    /// ink clipping for tall fallback glyphs.
    pub stable_line_boxes: bool,
    /// Minimum height for multiline text areas.
    pub min_height: Option<Px>,
}

impl Default for TextFieldOptions {
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
            size: Size::Small,
            placeholder: None,
            id_source: None,
            enabled: true,
            focusable: true,
            clear_button: false,
            a11y_label: None,
            test_id: None,
            clear_test_id: None,
            field_id_out: None,
            input_id_out: None,
            mode: TextFieldMode::PlainText,
            buffered: true,
            blur_behavior: TextFieldBlurBehavior::Commit,
            draft_controller: None,
            submit_command: None,
            assistive_semantics: TextFieldAssistiveSemantics::default(),
            selection_behavior: EditorTextSelectionBehavior::PreserveSelection,
            cancel_behavior: EditorTextCancelBehavior::None,
            multiline: false,
            stable_line_boxes: true,
            min_height: None,
        }
    }
}

#[derive(Clone)]
pub struct TextField {
    model: Model<String>,
    on_outcome: Option<OnTextFieldOutcome>,
    options: TextFieldOptions,
}
