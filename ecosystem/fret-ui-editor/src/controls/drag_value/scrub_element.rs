use std::sync::{Arc, Mutex};

use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerDownCx, PressablePointerDownResult, UiActionHost};
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use super::model::{DragValueMode, DragValueState};
use super::scrub::{DragValueScrubFrameArgs, drag_value_scrub_frame};
use super::session::emit_drag_value_outcome;
use super::{DragValueOutcome, OnDragValueOutcome};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusHandoffState, arm_numeric_text_entry_focus_handoff,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::{DragValueCore, DragValueCoreOptions, NumericValueConstraints};

pub(super) struct DragValueScrubElementArgs<T> {
    pub(super) model: Model<T>,
    pub(super) value: T,
    pub(super) value_text: Arc<str>,
    pub(super) layout: LayoutStyle,
    pub(super) scrub_enabled: bool,
    pub(super) constraints: NumericValueConstraints,
    pub(super) scrub_revision: u64,
    pub(super) state: Arc<Mutex<DragValueState>>,
    pub(super) focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>>,
    pub(super) on_outcome: Option<OnDragValueOutcome>,
    pub(super) prefix: Option<Arc<str>>,
    pub(super) suffix: Option<Arc<str>>,
    pub(super) scrub_test_id: Option<Arc<str>>,
    pub(super) prefix_test_id: Option<Arc<str>>,
    pub(super) suffix_test_id: Option<Arc<str>>,
    pub(super) value_test_id: Option<Arc<str>>,
}

pub(super) fn drag_value_scrub_element<H, T>(
    cx: &mut ElementContext<'_, H>,
    args: DragValueScrubElementArgs<T>,
) -> AnyElement
where
    H: UiHost,
    T: DragValueScalar + Default,
{
    let DragValueScrubElementArgs {
        model,
        value,
        value_text,
        layout,
        scrub_enabled,
        constraints,
        scrub_revision,
        state,
        focus_handoff,
        on_outcome,
        prefix,
        suffix,
        scrub_test_id,
        prefix_test_id,
        suffix_test_id,
        value_test_id,
    } = args;

    let (density, scrub_chrome) = {
        let theme = Theme::global(&*cx.app);
        let style = EditorStyle::resolve(theme);
        (style.density, style.frame_chrome_small())
    };

    let model_for_change = model.clone();
    let on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static> =
        Arc::new(move |host, action_cx, next| {
            let _ = host.models_mut().update(&model_for_change, |v| *v = next);
            host.request_redraw(action_cx.window);
        });

    let mut scrub_opts = DragValueCoreOptions::default();
    scrub_opts.layout = layout;
    scrub_opts.enabled = scrub_enabled;
    scrub_opts.scrub_on_double_click = false;
    scrub_opts.constraints = constraints;

    let state_for_scrub = state.clone();
    let focus_handoff_for_scrub = focus_handoff.clone();
    let on_outcome_for_scrub = on_outcome.clone();
    let prefix_for_scrub_root = prefix.clone();
    let suffix_for_scrub_root = suffix.clone();
    cx.keyed(
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
    )
}
