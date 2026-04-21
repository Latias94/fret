use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use fret_core::AppWindowId;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ImUiSharedHoverDelayState {
    delay_short_met: bool,
    delay_normal_met: bool,
    short_timer: Option<fret_runtime::TimerToken>,
    normal_timer: Option<fret_runtime::TimerToken>,
    clear_timer: Option<fret_runtime::TimerToken>,
}

#[derive(Default)]
struct ImUiSharedHoverDelayStore {
    by_window: HashMap<AppWindowId, fret_runtime::Model<ImUiSharedHoverDelayState>>,
}

#[derive(Debug, Default, Clone, Copy)]
struct HoverQueryDelayLocalState {
    stationary_met: bool,
    delay_short_met: bool,
    delay_normal_met: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub(in super::super) struct HoverQueryDelayRead {
    pub(in super::super) stationary_met: bool,
    pub(in super::super) delay_short_met: bool,
    pub(in super::super) delay_normal_met: bool,
    pub(in super::super) shared_delay_short_met: bool,
    pub(in super::super) shared_delay_normal_met: bool,
}

const HOVER_TIMER_KIND_STATIONARY: u64 =
    super::super::fnv1a64(b"fret-ui-kit.imui.hover.timer.stationary.v1");
const HOVER_TIMER_KIND_DELAY_SHORT: u64 =
    super::super::fnv1a64(b"fret-ui-kit.imui.hover.timer.delay_short.v1");
const HOVER_TIMER_KIND_DELAY_NORMAL: u64 =
    super::super::fnv1a64(b"fret-ui-kit.imui.hover.timer.delay_normal.v1");

const SHARED_HOVER_CLEAR_DELAY: Duration = Duration::from_millis(250);

fn shared_hover_delay_model_for_window<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> fret_runtime::Model<ImUiSharedHoverDelayState> {
    let window = cx.window;
    cx.app
        .with_global_mut_untracked(ImUiSharedHoverDelayStore::default, |st, app| {
            st.by_window
                .entry(window)
                .or_insert_with(|| {
                    app.models_mut()
                        .insert(ImUiSharedHoverDelayState::default())
                })
                .clone()
        })
}

pub(in super::super) fn hover_blocked_by_active_item_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    active_item_model: &fret_runtime::Model<super::ImUiActiveItemState>,
) -> bool {
    let active = cx
        .read_model(
            active_item_model,
            fret_ui::Invalidation::Paint,
            |_app, st| st.active,
        )
        .ok()
        .flatten();
    active.is_some() && active != Some(id)
}

fn hover_timer_token_for(kind: u64, element: GlobalElementId) -> fret_runtime::TimerToken {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for b in kind.to_le_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    for b in element.0.to_le_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    fret_runtime::TimerToken(hash)
}

fn shared_hover_delay_on_hover_change(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    hovered: bool,
    shared_model: &fret_runtime::Model<ImUiSharedHoverDelayState>,
) {
    let prev = host
        .models_mut()
        .read(shared_model, |st| *st)
        .ok()
        .unwrap_or_default();

    if hovered {
        if let Some(token) = prev.clear_timer {
            host.push_effect(fret_runtime::Effect::CancelTimer { token });
        }

        let mut next = prev;
        next.clear_timer = None;

        if !prev.delay_short_met && prev.short_timer.is_none() {
            let token = host.next_timer_token();
            next.short_timer = Some(token);
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token,
                after: super::super::HOVER_DELAY_SHORT,
                repeat: None,
            });
        }

        if !prev.delay_normal_met && prev.normal_timer.is_none() {
            let token = host.next_timer_token();
            next.normal_timer = Some(token);
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token,
                after: super::super::HOVER_DELAY_NORMAL,
                repeat: None,
            });
        }

        let _ = host.models_mut().update(shared_model, |st| *st = next);
        return;
    }

    if prev.clear_timer.is_some() {
        return;
    }

    let token = host.next_timer_token();
    host.push_effect(fret_runtime::Effect::SetTimer {
        window: Some(action_cx.window),
        token,
        after: SHARED_HOVER_CLEAR_DELAY,
        repeat: None,
    });

    let mut next = prev;
    next.clear_timer = Some(token);
    let _ = host.models_mut().update(shared_model, |st| *st = next);
}

fn shared_hover_delay_on_timer(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    action_cx: fret_ui::action::ActionCx,
    token: fret_runtime::TimerToken,
    shared_model: &fret_runtime::Model<ImUiSharedHoverDelayState>,
) -> bool {
    let prev = host
        .models_mut()
        .read(shared_model, |st| *st)
        .ok()
        .unwrap_or_default();

    if prev.short_timer == Some(token) {
        let mut next = prev;
        next.delay_short_met = true;
        next.short_timer = None;
        let _ = host.models_mut().update(shared_model, |st| *st = next);
        host.notify(action_cx);
        return true;
    }

    if prev.normal_timer == Some(token) {
        let mut next = prev;
        next.delay_normal_met = true;
        next.normal_timer = None;
        let _ = host.models_mut().update(shared_model, |st| *st = next);
        host.notify(action_cx);
        return true;
    }

    if prev.clear_timer == Some(token) {
        if let Some(token) = prev.short_timer {
            host.push_effect(fret_runtime::Effect::CancelTimer { token });
        }
        if let Some(token) = prev.normal_timer {
            host.push_effect(fret_runtime::Effect::CancelTimer { token });
        }
        let _ = host.models_mut().update(shared_model, |st| {
            *st = ImUiSharedHoverDelayState::default()
        });
        host.notify(action_cx);
        return true;
    }

    false
}

pub(in super::super) fn install_hover_query_hooks_for_pressable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    hovered_raw: bool,
    long_press_signal_model: Option<fret_runtime::Model<super::LongPressSignalState>>,
) -> HoverQueryDelayRead {
    let shared_delay_model = shared_hover_delay_model_for_window(cx);
    let shared_delay_model_for_hover = shared_delay_model.clone();
    cx.pressable_on_hover_change(Arc::new(move |host, action_cx, hovered| {
        let stationary = hover_timer_token_for(HOVER_TIMER_KIND_STATIONARY, action_cx.target);
        let delay_short = hover_timer_token_for(HOVER_TIMER_KIND_DELAY_SHORT, action_cx.target);
        let delay_normal = hover_timer_token_for(HOVER_TIMER_KIND_DELAY_NORMAL, action_cx.target);

        if hovered {
            shared_hover_delay_on_hover_change(
                host,
                action_cx,
                true,
                &shared_delay_model_for_hover,
            );
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token: stationary,
                after: super::super::HOVER_STATIONARY_DELAY,
                repeat: None,
            });
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token: delay_short,
                after: super::super::HOVER_DELAY_SHORT,
                repeat: None,
            });
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token: delay_normal,
                after: super::super::HOVER_DELAY_NORMAL,
                repeat: None,
            });
            return;
        }

        shared_hover_delay_on_hover_change(host, action_cx, false, &shared_delay_model_for_hover);
        host.push_effect(fret_runtime::Effect::CancelTimer { token: stationary });
        host.push_effect(fret_runtime::Effect::CancelTimer { token: delay_short });
        host.push_effect(fret_runtime::Effect::CancelTimer {
            token: delay_normal,
        });
    }));

    let long_press_signal_model_for_timer = long_press_signal_model.clone();
    let shared_delay_model_for_timer = shared_delay_model.clone();
    cx.timer_add_on_timer_for(
        id,
        Arc::new(move |host, action_cx, token| {
            let stationary = hover_timer_token_for(HOVER_TIMER_KIND_STATIONARY, action_cx.target);
            if token == stationary {
                host.record_transient_event(action_cx, super::super::KEY_HOVER_STATIONARY_MET);
                host.notify(action_cx);
                return true;
            }
            let delay_short = hover_timer_token_for(HOVER_TIMER_KIND_DELAY_SHORT, action_cx.target);
            if token == delay_short {
                host.record_transient_event(action_cx, super::super::KEY_HOVER_DELAY_SHORT_MET);
                host.notify(action_cx);
                return true;
            }
            let delay_normal =
                hover_timer_token_for(HOVER_TIMER_KIND_DELAY_NORMAL, action_cx.target);
            if token == delay_normal {
                host.record_transient_event(action_cx, super::super::KEY_HOVER_DELAY_NORMAL_MET);
                host.notify(action_cx);
                return true;
            }

            if shared_hover_delay_on_timer(host, action_cx, token, &shared_delay_model_for_timer) {
                return true;
            }

            if let Some(model) = long_press_signal_model_for_timer.as_ref() {
                return emit_long_press_if_matching(host, action_cx, model, token);
            }

            false
        }),
    );

    let stationary_fired = cx.take_transient_for(id, super::super::KEY_HOVER_STATIONARY_MET);
    let delay_short_fired = cx.take_transient_for(id, super::super::KEY_HOVER_DELAY_SHORT_MET);
    let delay_normal_fired = cx.take_transient_for(id, super::super::KEY_HOVER_DELAY_NORMAL_MET);

    let local = cx.state_for(id, HoverQueryDelayLocalState::default, |st| {
        if stationary_fired {
            st.stationary_met = true;
        }
        if delay_short_fired {
            st.delay_short_met = true;
        }
        if delay_normal_fired {
            st.delay_normal_met = true;
        }

        if !hovered_raw {
            *st = HoverQueryDelayLocalState::default();
        }

        *st
    });

    let shared = cx
        .read_model(
            &shared_delay_model,
            fret_ui::Invalidation::Paint,
            |_app, st| (st.delay_short_met, st.delay_normal_met),
        )
        .unwrap_or((false, false));

    HoverQueryDelayRead {
        stationary_met: local.stationary_met,
        delay_short_met: local.delay_short_met,
        delay_normal_met: local.delay_normal_met,
        shared_delay_short_met: shared.0,
        shared_delay_normal_met: shared.1,
    }
}

fn emit_long_press_if_matching(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    model: &fret_runtime::Model<super::LongPressSignalState>,
    token: fret_runtime::TimerToken,
) -> bool {
    let fired = host
        .update_model(model, |state| {
            if state.timer != Some(token) {
                return false;
            }
            state.timer = None;
            state.holding = true;
            true
        })
        .unwrap_or(false);
    if fired {
        host.record_transient_event(action_cx, super::super::KEY_LONG_PRESSED);
        host.notify(action_cx);
    }
    fired
}
