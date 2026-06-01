//! Editor-grade numeric control: drag-to-scrub with an optional typing mode.
//!
//! v1 goals (workstream):
//! - scrub (drag-to-change) with Shift slow / Alt fast outcomes,
//! - double-click to switch into a typing mode,
//! - Escape cancels scrub to the pre-edit value (handled by `DragValueCore`).

use std::panic::Location;
use std::sync::{Arc, Mutex};

use crate::controls::numeric_input::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use crate::primitives::numeric_text_entry::NumericTextEntryFocusHandoffState;
use crate::primitives::{EditSessionOutcome, NumericPresentation};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, UiHost};

#[cfg(test)]
mod tests;

mod model;
mod options;
mod scrub;
mod scrub_element;
mod session;
mod typing;

use model::{DragValueMode, DragValueState};
pub use options::DragValueOptions;
use scrub_element::{DragValueScrubElementArgs, drag_value_scrub_element};
use session::hidden_layout;
use typing::{DragValueTypingInputArgs, drag_value_typing_input};

pub type DragValueOutcome = EditSessionOutcome;
pub type OnDragValueOutcome =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, DragValueOutcome) + 'static>;

#[derive(Clone)]
pub struct DragValue<T> {
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    on_outcome: Option<OnDragValueOutcome>,
    options: DragValueOptions,
}

impl<T> DragValue<T>
where
    T: DragValueScalar + Default,
{
    pub fn new(model: Model<T>, format: NumericFormatFn<T>, parse: NumericParseFn<T>) -> Self {
        Self {
            model,
            format,
            parse,
            validate: None,
            on_outcome: None,
            options: DragValueOptions::default(),
        }
    }

    /// Construct a drag value from a shared editor authoring bundle.
    pub fn from_presentation(model: Model<T>, presentation: NumericPresentation<T>) -> Self {
        let mut drag_value = Self::new(model, presentation.format(), presentation.parse());
        drag_value.options.prefix = presentation.chrome_prefix().cloned();
        drag_value.options.suffix = presentation.chrome_suffix().cloned();
        drag_value
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn on_outcome(mut self, on_outcome: Option<OnDragValueOutcome>) -> Self {
        self.on_outcome = on_outcome;
        self
    }

    pub fn options(mut self, options: DragValueOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.drag_value", id_source, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.drag_value", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let state: Arc<Mutex<DragValueState>> = cx.slot_state(
            || Arc::new(Mutex::new(DragValueState::default())),
            |s| s.clone(),
        );
        let focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>> = cx.slot_state(
            || Arc::new(Mutex::new(NumericTextEntryFocusHandoffState::default())),
            |s| s.clone(),
        );
        let on_outcome = self.on_outcome.clone();

        let value = cx
            .get_model_copied(&self.model, Invalidation::Paint)
            .unwrap_or_default();
        let value_text = (self.format)(value);

        let (mode, scrub_revision) = {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            (st.mode, st.scrub_revision)
        };

        let typing = mode == DragValueMode::Typing;
        let (prefix, suffix) = suppress_duplicate_chrome_affixes(
            value_text.as_ref(),
            self.options.prefix.clone(),
            self.options.suffix.clone(),
        );
        let scrub_test_id = self.options.test_id.clone();
        let typing_test_id = derived_test_id(self.options.test_id.as_ref(), "typing");
        let active_typing_test_id = if typing { typing_test_id.clone() } else { None };
        let prefix_test_id = derived_test_id(scrub_test_id.as_ref(), "prefix");
        let suffix_test_id = derived_test_id(scrub_test_id.as_ref(), "suffix");
        let value_test_id = derived_test_id(scrub_test_id.as_ref(), "value");

        let scrub = drag_value_scrub_element(
            cx,
            DragValueScrubElementArgs {
                model: self.model.clone(),
                value,
                value_text: value_text.clone(),
                layout: self.options.layout,
                typing,
                scrub_enabled: mode == DragValueMode::Scrub,
                constraints: self.options.constraints,
                scrub_revision,
                state: state.clone(),
                focus_handoff: focus_handoff.clone(),
                on_outcome: on_outcome.clone(),
                prefix: prefix.clone(),
                suffix: suffix.clone(),
                scrub_test_id: scrub_test_id.clone(),
                prefix_test_id: prefix_test_id.clone(),
                suffix_test_id: suffix_test_id.clone(),
                value_test_id: value_test_id.clone(),
            },
        );

        let mut input_layout = self.options.layout;
        if !typing {
            input_layout = hidden_layout(input_layout);
        }

        let input = drag_value_typing_input(
            cx,
            DragValueTypingInputArgs {
                model: self.model.clone(),
                format: self.format.clone(),
                parse: self.parse.clone(),
                validate: self.validate.clone(),
                constraints: self.options.constraints,
                input_layout,
                typing,
                prefix: prefix.clone(),
                suffix: suffix.clone(),
                selection_behavior: self.options.selection_behavior,
                active_typing_test_id,
                state: state.clone(),
                focus_handoff: focus_handoff.clone(),
                on_outcome: on_outcome.clone(),
            },
        );

        // Render both: scrub stays mounted so focus can restore, input stays mounted so focus
        // requests have a stable target.
        cx.container(Default::default(), move |_cx| vec![scrub, input])
    }
}
