//! Editor-grade numeric control: drag-to-scrub with an optional typing mode.
//!
//! v1 goals (workstream):
//! - scrub (drag-to-change) with Shift slow / Alt fast outcomes,
//! - double-click to switch into a typing mode,
//! - Escape cancels scrub to the pre-edit value (handled by `DragValueCore`).

use std::panic::Location;
use std::sync::{Arc, Mutex};

use crate::controls::numeric_input::{
    NumericFormatFn, NumericInput, NumericInputErrorDisplay, NumericInputOptions,
    NumericInputOutcome, NumericInputSelectionBehavior, NumericParseFn, NumericValidateFn,
};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusHandoffState, arm_numeric_text_entry_focus_handoff,
    sync_numeric_text_entry_focus_handoff,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::{
    DragValueCore, DragValueCoreOptions, EditSessionOutcome, NumericPresentation,
    NumericValueConstraints, constrain_numeric_value,
};
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerDownCx, PressablePointerDownResult, UiActionHost};
use fret_ui::element::{
    AnyElement, FlexItemStyle, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

#[cfg(test)]
mod tests;

mod scrub;

use scrub::{DragValueScrubFrameArgs, drag_value_scrub_frame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragValueMode {
    Scrub,
    Typing,
}

#[derive(Debug)]
struct DragValueState {
    mode: DragValueMode,
    scrub_id: Option<fret_ui::GlobalElementId>,
    scrub_revision: u64,
}

impl Default for DragValueState {
    fn default() -> Self {
        Self {
            mode: DragValueMode::Scrub,
            scrub_id: None,
            scrub_revision: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DragValueOptions {
    pub layout: LayoutStyle,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    /// Shared numeric edit constraints applied to scrub and typed commit paths.
    pub constraints: NumericValueConstraints,
    pub selection_behavior: NumericInputSelectionBehavior,
    /// Explicit identity source for internal state (scrub/typing focus restore).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple drag values from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for DragValueOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            prefix: None,
            suffix: None,
            constraints: NumericValueConstraints::default(),
            selection_behavior: NumericInputSelectionBehavior::ReplaceAllOnFocus,
            id_source: None,
            test_id: None,
        }
    }
}

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

        let (density, scrub_chrome) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            (style.density, style.frame_chrome_small())
        };

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
        scrub_opts.constraints = self.options.constraints;

        let state_for_scrub = state.clone();
        let focus_handoff_for_scrub = focus_handoff.clone();
        let on_outcome_for_scrub = on_outcome.clone();
        let prefix_for_scrub_root = prefix.clone();
        let suffix_for_scrub_root = suffix.clone();
        let scrub = cx.keyed(
            ("fret-ui-editor.drag_value.scrub", scrub_revision),
            move |cx| {
                let prefix_for_scrub = prefix_for_scrub_root.clone();
                let suffix_for_scrub = suffix_for_scrub_root.clone();
                let state_for_scrub_record = state_for_scrub.clone();
                let focus_handoff_for_double_click = focus_handoff_for_scrub.clone();
                let on_outcome_for_scrub_commit = on_outcome_for_scrub.clone();
                let on_outcome_for_scrub_cancel = on_outcome_for_scrub.clone();
                DragValueCore::new(value, on_change_live)
                    .on_commit(Some(Arc::new(move |host, action_cx| {
                        emit_drag_value_outcome(
                            host,
                            action_cx,
                            on_outcome_for_scrub_commit.as_ref(),
                            DragValueOutcome::Committed,
                        );
                    })))
                    .on_cancel(Some(Arc::new(move |host, action_cx| {
                        emit_drag_value_outcome(
                            host,
                            action_cx,
                            on_outcome_for_scrub_cancel.as_ref(),
                            DragValueOutcome::Canceled,
                        );
                    })))
                    .a11y_label(value_text.clone())
                    .options(scrub_opts)
                    .into_element(cx, move |cx, resp| {
                        // Record the scrub element id for focus restore from typing mode.
                        let scrub_id = cx.root_id();
                        let mut st = state_for_scrub_record
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        st.scrub_id = Some(scrub_id);

                        let state_for_double_click = state_for_scrub_record.clone();
                        let focus_handoff_for_double_click = focus_handoff_for_double_click.clone();
                        cx.pressable_add_on_pointer_down(Arc::new(
                            move |host, action_cx, down: PointerDownCx| {
                                if down.click_count < 2 {
                                    return PressablePointerDownResult::Continue;
                                }

                                let mut st = state_for_double_click
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                st.mode = DragValueMode::Typing;
                                {
                                    let mut handoff = focus_handoff_for_double_click
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner());
                                    arm_numeric_text_entry_focus_handoff(&mut handoff);
                                }
                                host.request_redraw(action_cx.window);
                                PressablePointerDownResult::SkipDefaultAndStopPropagation
                            },
                        ));

                        let scrub_frame = drag_value_scrub_frame(
                            cx,
                            DragValueScrubFrameArgs {
                                density,
                                scrub_chrome,
                                hovered: resp.hovered(),
                                pressed: resp.dragging() || resp.pressed(),
                                focused: resp.focused() || cx.is_focused_element(scrub_id),
                                value_text: value_text.clone(),
                                prefix: prefix_for_scrub.clone(),
                                suffix: suffix_for_scrub.clone(),
                                scrub_test_id: scrub_test_id.clone(),
                                prefix_test_id: prefix_test_id.clone(),
                                suffix_test_id: suffix_test_id.clone(),
                                value_test_id: value_test_id.clone(),
                            },
                        );
                        vec![scrub_frame]
                    })
            },
        );

        let mut input_layout = self.options.layout;
        if !typing {
            input_layout = hidden_layout(input_layout);
        }

        let state_for_input = state.clone();
        let on_outcome_for_input = on_outcome.clone();
        let input_focus_target: Arc<Mutex<Option<fret_ui::GlobalElementId>>> =
            Arc::new(Mutex::new(None));
        let constraints = self.options.constraints;
        let parse = self.parse.clone();
        let constrained_parse: NumericParseFn<T> = Arc::new(move |text| {
            parse(text).map(|value| constrain_numeric_value(constraints, value))
        });
        let input = NumericInput::new(self.model.clone(), self.format.clone(), constrained_parse)
            .validate(self.validate.clone())
            .focus_target(input_focus_target.clone())
            .options(NumericInputOptions {
                layout: input_layout,
                enabled: typing,
                focusable: typing,
                prefix: prefix.clone(),
                suffix: suffix.clone(),
                selection_behavior: self.options.selection_behavior,
                test_id: active_typing_test_id,
                // Avoid growing the row height when a commit-time validation error occurs.
                // A small trailing status icon keeps the inspector layout stable.
                error_display: NumericInputErrorDisplay::TrailingIcon,
                ..Default::default()
            })
            .on_outcome(Some(Arc::new(move |host, action_cx, outcome| {
                let mut st = state_for_input.lock().unwrap_or_else(|e| e.into_inner());
                match outcome {
                    NumericInputOutcome::Committed | NumericInputOutcome::Canceled => {
                        st.mode = DragValueMode::Scrub;
                        st.scrub_revision = st.scrub_revision.wrapping_add(1);
                        if let Some(scrub_id) = st.scrub_id {
                            host.request_focus(scrub_id);
                        }
                        emit_drag_value_outcome(
                            host,
                            action_cx,
                            on_outcome_for_input.as_ref(),
                            drag_value_outcome_from_numeric_input(outcome),
                        );
                        host.request_redraw(action_cx.window);
                    }
                }
            })))
            .into_element(cx);

        if let Some(input_id) = input_focus_target
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .copied()
        {
            let is_focused = cx.is_focused_element(input_id);
            sync_numeric_text_entry_focus_handoff(
                cx,
                input.id,
                &focus_handoff,
                typing,
                input_id,
                is_focused,
            );
        }

        // Render both: scrub stays mounted so focus can restore, input stays mounted so focus
        // requests have a stable target.
        cx.container(Default::default(), move |_cx| vec![scrub, input])
    }
}

fn hidden_layout(mut layout: LayoutStyle) -> LayoutStyle {
    layout.size = SizeStyle {
        width: Length::Px(Px(0.0)),
        height: Length::Px(Px(0.0)),
        min_width: Some(Length::Px(Px(0.0))),
        min_height: Some(Length::Px(Px(0.0))),
        ..Default::default()
    };
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
        ..Default::default()
    };
    layout.overflow = Overflow::Clip;
    layout
}

fn drag_value_outcome_from_numeric_input(outcome: NumericInputOutcome) -> DragValueOutcome {
    match outcome {
        NumericInputOutcome::Committed => DragValueOutcome::Committed,
        NumericInputOutcome::Canceled => DragValueOutcome::Canceled,
    }
}

fn emit_drag_value_outcome(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    on_outcome: Option<&OnDragValueOutcome>,
    outcome: DragValueOutcome,
) {
    if let Some(cb) = on_outcome {
        cb(host, action_cx, outcome);
    }
}
