//! AxisDragValue scrub DragValueCore owner.

use std::sync::{Arc, Mutex};

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerDownCx, PressablePointerDownResult, UiActionHost};
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusHandoffState, arm_numeric_text_entry_focus_handoff,
};
use crate::primitives::{
    DragValueCore, DragValueCoreOptions, EditorDensity, NumericValueConstraints,
};

use super::super::model::{
    AxisDragValueMode, AxisDragValueOutcome, AxisDragValueResetAction, AxisDragValueState,
    OnAxisDragValueOutcome,
};
use super::super::session::emit_axis_drag_value_outcome;
use super::scrub::{AxisDragValueScrubFrameArgs, axis_drag_value_scrub_frame};

pub(super) struct AxisDragValueScrubElementArgs<T> {
    pub(super) state: Arc<Mutex<AxisDragValueState>>,
    pub(super) focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>>,
    pub(super) model: Model<T>,
    pub(super) on_outcome: Option<OnAxisDragValueOutcome>,
    pub(super) value: T,
    pub(super) value_text: Arc<str>,
    pub(super) scrub_revision: u64,
    pub(super) mode: AxisDragValueMode,
    pub(super) layout: LayoutStyle,
    pub(super) constraints: NumericValueConstraints,
    pub(super) density: EditorDensity,
    pub(super) frame_chrome: ResolvedEditorFrameChrome,
    pub(super) enabled: bool,
    pub(super) axis_label: Arc<str>,
    pub(super) axis_tint: Color,
    pub(super) prefix: Option<Arc<str>>,
    pub(super) suffix: Option<Arc<str>>,
    pub(super) reset_action: Option<AxisDragValueResetAction>,
    pub(super) scrub_test_id: Option<Arc<str>>,
    pub(super) scrub_axis_test_id: Option<Arc<str>>,
    pub(super) scrub_value_test_id: Option<Arc<str>>,
    pub(super) scrub_prefix_test_id: Option<Arc<str>>,
    pub(super) scrub_suffix_test_id: Option<Arc<str>>,
    pub(super) scrub_reset_test_id: Option<Arc<str>>,
}

pub(super) fn axis_drag_value_scrub_element<T, H>(
    cx: &mut ElementContext<'_, H>,
    args: AxisDragValueScrubElementArgs<T>,
) -> AnyElement
where
    T: DragValueScalar + Default,
    H: UiHost,
{
    let AxisDragValueScrubElementArgs {
        state,
        focus_handoff,
        model,
        on_outcome,
        value,
        value_text,
        scrub_revision,
        mode,
        layout,
        constraints,
        density,
        frame_chrome,
        enabled,
        axis_label,
        axis_tint,
        prefix,
        suffix,
        reset_action,
        scrub_test_id,
        scrub_axis_test_id,
        scrub_value_test_id,
        scrub_prefix_test_id,
        scrub_suffix_test_id,
        scrub_reset_test_id,
    } = args;

    let mut scrub_opts = DragValueCoreOptions::default();
    scrub_opts.layout = layout;
    scrub_opts.enabled = enabled && mode == AxisDragValueMode::Scrub;
    scrub_opts.scrub_on_double_click = false;
    scrub_opts.constraints = constraints;

    let model_for_change = model.clone();
    let on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static> =
        Arc::new(move |host, action_cx, next| {
            let _ = host.models_mut().update(&model_for_change, |v| *v = next);
            host.request_redraw(action_cx.window);
        });

    cx.keyed(
        ("fret-ui-editor.axis_drag_value.scrub", scrub_revision),
        move |cx| {
            let state_for_scrub_record = state.clone();
            let focus_handoff_for_double_click = focus_handoff.clone();
            let prefix_for_scrub = prefix.clone();
            let suffix_for_scrub = suffix.clone();
            let on_outcome_for_scrub_commit = on_outcome.clone();
            let on_outcome_for_scrub_cancel = on_outcome.clone();
            let value_text_for_scrub = value_text.clone();
            DragValueCore::new(value, on_change_live)
                .on_commit(Some(Arc::new(move |host, action_cx| {
                    emit_axis_drag_value_outcome(
                        host,
                        action_cx,
                        on_outcome_for_scrub_commit.as_ref(),
                        AxisDragValueOutcome::Committed,
                    );
                })))
                .on_cancel(Some(Arc::new(move |host, action_cx| {
                    emit_axis_drag_value_outcome(
                        host,
                        action_cx,
                        on_outcome_for_scrub_cancel.as_ref(),
                        AxisDragValueOutcome::Canceled,
                    );
                })))
                .a11y_label(value_text.clone())
                .options(scrub_opts)
                .into_element(cx, move |cx, resp| {
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
                            st.mode = AxisDragValueMode::Typing;
                            st.seen_input_focus = false;
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

                    let scrub_frame = axis_drag_value_scrub_frame(
                        cx,
                        AxisDragValueScrubFrameArgs {
                            density,
                            frame_chrome,
                            hovered: resp.hovered(),
                            pressed: resp.dragging() || resp.pressed(),
                            focused: resp.focused() || cx.is_focused_element(scrub_id),
                            enabled,
                            axis_label: axis_label.clone(),
                            axis_tint,
                            value_text: value_text_for_scrub.clone(),
                            prefix: prefix_for_scrub.clone(),
                            suffix: suffix_for_scrub.clone(),
                            reset_action: reset_action.clone(),
                            scrub_test_id: scrub_test_id.clone(),
                            scrub_axis_test_id: scrub_axis_test_id.clone(),
                            scrub_value_test_id: scrub_value_test_id.clone(),
                            scrub_prefix_test_id: scrub_prefix_test_id.clone(),
                            scrub_suffix_test_id: scrub_suffix_test_id.clone(),
                            scrub_reset_test_id: scrub_reset_test_id.clone(),
                        },
                    );
                    vec![scrub_frame]
                })
        },
    )
}
