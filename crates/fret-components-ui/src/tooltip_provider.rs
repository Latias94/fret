use std::collections::HashMap;

use fret_runtime::FrameId;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::headless::tooltip_delay_group::{TooltipDelayGroupConfig, TooltipDelayGroupState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooltipProviderConfig {
    pub delay_duration_ticks: u64,
    pub skip_delay_duration_ticks: u64,
}

impl TooltipProviderConfig {
    pub fn new(delay_duration_ticks: u64, skip_delay_duration_ticks: u64) -> Self {
        Self {
            delay_duration_ticks,
            skip_delay_duration_ticks,
        }
    }
}

impl Default for TooltipProviderConfig {
    fn default() -> Self {
        Self {
            delay_duration_ticks: 0,
            skip_delay_duration_ticks: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ProviderState {
    config: TooltipProviderConfig,
    delay_group: TooltipDelayGroupState,
}

impl Default for ProviderState {
    fn default() -> Self {
        Self {
            config: TooltipProviderConfig::default(),
            delay_group: TooltipDelayGroupState::default(),
        }
    }
}

#[derive(Default)]
struct TooltipProviderService {
    frame_id: Option<FrameId>,
    active_stack: Vec<GlobalElementId>,
    providers: HashMap<GlobalElementId, ProviderState>,
    root: ProviderState,
}

impl TooltipProviderService {
    fn begin_frame(&mut self, frame_id: FrameId) {
        if self.frame_id == Some(frame_id) {
            return;
        }
        self.frame_id = Some(frame_id);
        self.active_stack.clear();
    }

    fn current_provider_id(&self) -> Option<GlobalElementId> {
        self.active_stack.last().copied()
    }

    fn current_state_mut(&mut self) -> &mut ProviderState {
        let Some(id) = self.current_provider_id() else {
            return &mut self.root;
        };
        self.providers.entry(id).or_default()
    }

    fn current_state(&self) -> &ProviderState {
        let Some(id) = self.current_provider_id() else {
            return &self.root;
        };
        self.providers.get(&id).unwrap_or(&self.root)
    }
}

pub fn with_tooltip_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    config: TooltipProviderConfig,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    cx.scope(|cx| {
        let provider_id = cx.root_id();

        cx.app
            .with_global_mut(TooltipProviderService::default, |svc, app| {
                svc.begin_frame(app.frame_id());
                let entry = svc.providers.entry(provider_id).or_default();
                entry.config = config;
                svc.active_stack.push(provider_id);
            });

        let out = f(cx);

        cx.app
            .with_global_mut(TooltipProviderService::default, |svc, app| {
                svc.begin_frame(app.frame_id());
                let _ = svc.active_stack.pop();
            });

        out
    })
}

pub fn current_config<H: UiHost>(cx: &mut ElementContext<'_, H>) -> TooltipProviderConfig {
    cx.app
        .with_global_mut(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            svc.current_state().config
        })
}

pub fn open_delay_ticks<H: UiHost>(cx: &mut ElementContext<'_, H>, now: u64) -> u64 {
    cx.app
        .with_global_mut(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            let st = svc.current_state();
            st.delay_group.open_delay_ticks(
                now,
                TooltipDelayGroupConfig::new(
                    st.config.delay_duration_ticks,
                    st.config.skip_delay_duration_ticks,
                ),
            )
        })
}

pub fn open_delay_ticks_with_base<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    now: u64,
    base_delay_ticks: u64,
) -> u64 {
    cx.app
        .with_global_mut(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            let st = svc.current_state();
            st.delay_group.open_delay_ticks(
                now,
                TooltipDelayGroupConfig::new(base_delay_ticks, st.config.skip_delay_duration_ticks),
            )
        })
}

pub fn note_closed<H: UiHost>(cx: &mut ElementContext<'_, H>, now: u64) {
    cx.app
        .with_global_mut(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            svc.current_state_mut().delay_group.note_closed(now);
        });
}

