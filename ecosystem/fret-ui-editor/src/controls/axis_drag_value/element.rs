use std::sync::{Arc, Mutex};

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::ChromeRefinement;

use crate::primitives::chrome::resolve_editor_text_field_style;
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use crate::primitives::numeric_text_entry::NumericTextEntryFocusHandoffState;
use crate::primitives::style::EditorStyle;

use super::AxisDragValue;
use super::ids::axis_drag_value_test_ids;
use super::model::{AxisDragValueMode, AxisDragValueState, axis_drag_value_input_text_style};

mod input;
mod scrub;
mod scrub_element;
mod typing;
mod typing_element;
mod typing_focus;
mod typing_keys;

use scrub_element::{AxisDragValueScrubElementArgs, axis_drag_value_scrub_element};
use typing_element::{AxisDragValueTypingElementArgs, axis_drag_value_typing_element};

impl<T> AxisDragValue<T>
where
    T: DragValueScalar + Default,
{
    pub(super) fn into_element_keyed<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let state: Arc<Mutex<AxisDragValueState>> = cx.slot_state(
            || Arc::new(Mutex::new(AxisDragValueState::default())),
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
        let typing = mode == AxisDragValueMode::Typing;
        let (prefix, suffix) = suppress_duplicate_chrome_affixes(
            value_text.as_ref(),
            self.options.prefix.clone(),
            self.options.suffix.clone(),
        );
        let reset_action = self.options.reset.clone();
        let test_ids = axis_drag_value_test_ids(
            self.options.test_id.clone(),
            reset_action
                .as_ref()
                .and_then(|reset| reset.test_id.clone()),
            typing,
        );
        let scrub_test_id = test_ids.scrub.clone();
        let active_typing_test_id = test_ids.active_typing.clone();
        let scrub_axis_test_id = test_ids.scrub_axis.clone();
        let scrub_value_test_id = test_ids.scrub_value.clone();
        let scrub_prefix_test_id = test_ids.scrub_prefix.clone();
        let scrub_suffix_test_id = test_ids.scrub_suffix.clone();
        let typing_axis_test_id = test_ids.typing_axis.clone();
        let typing_input_test_id = test_ids.typing_input.clone();
        let typing_prefix_test_id = test_ids.typing_prefix.clone();
        let typing_suffix_test_id = test_ids.typing_suffix.clone();
        let typing_error_icon_test_id = test_ids.typing_error_icon.clone();
        let scrub_reset_test_id = test_ids.scrub_reset.clone();
        let typing_reset_test_id = test_ids.typing_reset.clone();

        let (density, frame_chrome, (text_style, input_chrome)) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let frame_chrome = style.frame_chrome_small();
            let (input_chrome, text_style) = resolve_editor_text_field_style(
                theme,
                self.options.size,
                &ChromeRefinement::default(),
            );
            let text_style = axis_drag_value_input_text_style(text_style, style.density.row_height);

            (style.density, frame_chrome, (text_style, input_chrome))
        };

        let scrub = axis_drag_value_scrub_element(
            cx,
            AxisDragValueScrubElementArgs {
                state: state.clone(),
                focus_handoff: focus_handoff.clone(),
                model: self.model.clone(),
                on_outcome: on_outcome.clone(),
                value,
                value_text: value_text.clone(),
                scrub_revision,
                typing,
                mode,
                layout: self.options.layout,
                constraints: self.options.constraints,
                density,
                frame_chrome,
                enabled: self.options.enabled,
                axis_label: self.axis_label.clone(),
                axis_tint: self.axis_tint,
                prefix: prefix.clone(),
                suffix: suffix.clone(),
                reset_action: reset_action.clone(),
                scrub_test_id: scrub_test_id.clone(),
                scrub_axis_test_id: scrub_axis_test_id.clone(),
                scrub_value_test_id: scrub_value_test_id.clone(),
                scrub_prefix_test_id: scrub_prefix_test_id.clone(),
                scrub_suffix_test_id: scrub_suffix_test_id.clone(),
                scrub_reset_test_id: scrub_reset_test_id.clone(),
            },
        );
        let typing_field = axis_drag_value_typing_element(
            cx,
            AxisDragValueTypingElementArgs {
                state: state.clone(),
                focus_handoff: focus_handoff.clone(),
                model: self.model.clone(),
                parse: self.parse.clone(),
                format: self.format.clone(),
                validate: self.validate.clone(),
                on_outcome: on_outcome.clone(),
                value_text: value_text.clone(),
                typing,
                layout: self.options.layout,
                constraints: self.options.constraints,
                density,
                frame_chrome,
                input_chrome,
                text_style,
                enabled: self.options.enabled,
                focusable: self.options.focusable,
                selection_behavior: self.options.selection_behavior,
                axis_label: self.axis_label.clone(),
                axis_tint: self.axis_tint,
                prefix: prefix.clone(),
                suffix: suffix.clone(),
                reset_action: self.options.reset.clone(),
                active_typing_test_id,
                typing_axis_test_id,
                typing_input_test_id,
                typing_prefix_test_id,
                typing_suffix_test_id,
                typing_error_icon_test_id,
                typing_reset_test_id,
            },
        );

        // Render both: scrub stays mounted so focus can restore, typing stays mounted so focus
        // requests have a stable target.
        cx.container(Default::default(), move |_cx| vec![scrub, typing_field])
    }
}
