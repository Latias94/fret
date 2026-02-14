//! Editor-grade numeric control: drag-to-scrub with an optional typing mode.
//!
//! v1 goals (workstream):
//! - scrub (drag-to-change) with Shift slow / Alt fast outcomes,
//! - double-click to switch into a typing mode,
//! - Escape cancels scrub to the pre-edit value (handled by `DragValueCore`).

use std::sync::{Arc, Mutex};

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerDownCx, PressablePointerDownResult, UiActionHost};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle};
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::controls::numeric_input::{
    NumericFormatFn, NumericInput, NumericInputOptions, NumericInputOutcome, NumericParseFn,
    NumericValidateFn,
};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::{DragValueCore, DragValueCoreOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragValueMode {
    Scrub,
    Typing,
}

#[derive(Debug)]
struct DragValueState {
    mode: DragValueMode,
    scrub_id: Option<fret_ui::GlobalElementId>,
    input_id: Option<fret_ui::GlobalElementId>,
}

impl Default for DragValueState {
    fn default() -> Self {
        Self {
            mode: DragValueMode::Scrub,
            scrub_id: None,
            input_id: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DragValueOptions {
    pub layout: LayoutStyle,
    pub test_id: Option<Arc<str>>,
}

#[derive(Clone)]
pub struct DragValue<T> {
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
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
            options: DragValueOptions::default(),
        }
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn options(mut self, options: DragValueOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let state: Arc<Mutex<DragValueState>> = cx.with_state(
                || Arc::new(Mutex::new(DragValueState::default())),
                |s| s.clone(),
            );

            let value = cx
                .get_model_copied(&self.model, Invalidation::Paint)
                .unwrap_or_default();
            let value_text = (self.format)(value);

            let mode = state.lock().unwrap_or_else(|e| e.into_inner()).mode;

            let typing = mode == DragValueMode::Typing;

            let model_for_change = self.model.clone();
            let on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static> =
                Arc::new(move |host, action_cx, next| {
                    let _ = host.models_mut().update(&model_for_change, |v| *v = next);
                    host.request_redraw(action_cx.window);
                });

            let mut scrub_opts = DragValueCoreOptions::default();
            scrub_opts.layout = if typing {
                hidden_layout(self.options.layout)
            } else {
                self.options.layout
            };
            scrub_opts.enabled = mode == DragValueMode::Scrub;
            scrub_opts.scrub_on_double_click = false;

            let state_for_scrub_record = state.clone();
            let scrub = DragValueCore::new(value, on_change_live)
                .a11y_label(value_text.clone())
                .options(scrub_opts)
                .into_element(cx, move |cx, _resp| {
                    // Record the scrub element id for focus restore from typing mode.
                    let scrub_id = cx.root_id();
                    let mut st = state_for_scrub_record
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    st.scrub_id = Some(scrub_id);

                    let state_for_double_click = state_for_scrub_record.clone();
                    cx.pressable_add_on_pointer_down(Arc::new(
                        move |host, action_cx, down: PointerDownCx| {
                            if down.click_count < 2 {
                                return PressablePointerDownResult::Continue;
                            }

                            let mut st = state_for_double_click
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            st.mode = DragValueMode::Typing;
                            if let Some(input_id) = st.input_id {
                                host.request_focus(input_id);
                            }
                            host.request_redraw(action_cx.window);
                            PressablePointerDownResult::SkipDefaultAndStopPropagation
                        },
                    ));

                    vec![cx.text(value_text.as_ref())]
                });

            let mut input_layout = self.options.layout;
            if !typing {
                input_layout = hidden_layout(input_layout);
            }

            let state_for_input = state.clone();
            let input =
                NumericInput::new(self.model.clone(), self.format.clone(), self.parse.clone())
                    .validate(self.validate.clone())
                    .options(NumericInputOptions {
                        layout: input_layout,
                        enabled: typing,
                        focusable: typing,
                        test_id: self.options.test_id.clone(),
                        ..Default::default()
                    })
                    .on_outcome(Some(Arc::new(move |host, action_cx, outcome| {
                        let mut st = state_for_input.lock().unwrap_or_else(|e| e.into_inner());
                        match outcome {
                            NumericInputOutcome::Committed | NumericInputOutcome::Canceled => {
                                st.mode = DragValueMode::Scrub;
                                if let Some(scrub_id) = st.scrub_id {
                                    host.request_focus(scrub_id);
                                }
                                host.request_redraw(action_cx.window);
                            }
                        }
                    })))
                    .into_element(cx);

            {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                st.input_id = Some(input.id);
            }

            // Render both: scrub stays mounted so focus can restore, input stays mounted so focus
            // requests have a stable target.
            cx.container(Default::default(), move |_cx| vec![scrub, input])
        })
    }
}

fn hidden_layout(mut layout: LayoutStyle) -> LayoutStyle {
    layout.size = SizeStyle {
        width: Length::Px(Px(0.0)),
        height: Length::Px(Px(0.0)),
        ..Default::default()
    };
    layout
}
