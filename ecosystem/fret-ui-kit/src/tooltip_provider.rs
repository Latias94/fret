use std::collections::HashMap;

use fret_core::Rect;
use fret_runtime::FrameId;
use fret_runtime::Model;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::headless::tooltip_delay_group::{TooltipDelayGroupConfig, TooltipDelayGroupState};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TooltipProviderConfig {
    pub delay_duration_ticks: u64,
    pub close_delay_duration_ticks: Option<u64>,
    pub skip_delay_duration_ticks: u64,
    pub disable_hoverable_content: bool,
}

impl TooltipProviderConfig {
    pub fn new(delay_duration_ticks: u64, skip_delay_duration_ticks: u64) -> Self {
        Self {
            delay_duration_ticks,
            close_delay_duration_ticks: None,
            skip_delay_duration_ticks,
            disable_hoverable_content: false,
        }
    }

    pub fn close_delay_duration_ticks(mut self, ticks: u64) -> Self {
        self.close_delay_duration_ticks = Some(ticks);
        self
    }

    pub fn disable_hoverable_content(mut self, disable: bool) -> Self {
        self.disable_hoverable_content = disable;
        self
    }
}

#[derive(Debug, Default, Clone)]
struct ProviderState {
    config: TooltipProviderConfig,
    delay_group: TooltipDelayGroupState,
    last_opened_token: u64,
    last_opened_tooltip: Option<GlobalElementId>,
    pointer_in_transit: bool,
    pointer_in_transit_model: Option<Model<bool>>,
    pointer_transit_geometry_model: Option<Model<Option<(Rect, Rect)>>>,
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
            .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
                svc.begin_frame(app.frame_id());
                let entry = svc.providers.entry(provider_id).or_default();
                entry.config = config;
                svc.active_stack.push(provider_id);
            });

        let out = f(cx);

        cx.app
            .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
                svc.begin_frame(app.frame_id());
                let _ = svc.active_stack.pop();
            });

        out
    })
}

pub fn current_config<H: UiHost>(cx: &mut ElementContext<'_, H>) -> TooltipProviderConfig {
    cx.app
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            svc.current_state().config
        })
}

pub fn open_delay_ticks<H: UiHost>(cx: &mut ElementContext<'_, H>, now: u64) -> u64 {
    cx.app
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
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
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
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
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            svc.current_state_mut().delay_group.note_closed(now);
        });
}

pub fn last_opened_tooltip<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Option<(GlobalElementId, u64)> {
    cx.app
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            let st = svc.current_state();
            st.last_opened_tooltip.map(|id| (id, st.last_opened_token))
        })
}

pub fn note_opened_tooltip<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    tooltip: GlobalElementId,
) -> u64 {
    cx.app
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            let st = svc.current_state_mut();
            st.last_opened_token = st.last_opened_token.saturating_add(1);
            st.last_opened_tooltip = Some(tooltip);

            if st.pointer_in_transit {
                st.pointer_in_transit = false;
                if let Some(model) = st.pointer_in_transit_model.clone() {
                    let _ = app.models_mut().update(&model, |v| *v = false);
                }
            }
            if let Some(model) = st.pointer_transit_geometry_model.clone() {
                let _ = app.models_mut().update(&model, |v| *v = None);
            }
            st.last_opened_token
        })
}

pub fn pointer_in_transit_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.app
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            let st = svc.current_state_mut();
            let existing = st.pointer_in_transit_model.clone();
            if let Some(model) = existing {
                return model;
            }

            let model = app.models_mut().insert(st.pointer_in_transit);
            st.pointer_in_transit_model = Some(model.clone());
            model
        })
}

pub fn pointer_transit_geometry_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<(Rect, Rect)>> {
    cx.app
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            let st = svc.current_state_mut();
            let existing = st.pointer_transit_geometry_model.clone();
            if let Some(model) = existing {
                return model;
            }

            let model = app.models_mut().insert(None);
            st.pointer_transit_geometry_model = Some(model.clone());
            model
        })
}

pub fn set_pointer_transit_geometry<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    geometry: Option<(Rect, Rect)>,
) {
    cx.app
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            let st = svc.current_state_mut();
            let model = st
                .pointer_transit_geometry_model
                .clone()
                .unwrap_or_else(|| {
                    let model = app.models_mut().insert(None);
                    st.pointer_transit_geometry_model = Some(model.clone());
                    model
                });

            let _ = app.models_mut().update(&model, |v| *v = geometry);
        });
}

pub fn is_pointer_in_transit<H: UiHost>(cx: &mut ElementContext<'_, H>) -> bool {
    cx.app
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            svc.current_state().pointer_in_transit
        })
}

pub fn set_pointer_in_transit<H: UiHost>(cx: &mut ElementContext<'_, H>, in_transit: bool) {
    cx.app
        .with_global_mut_untracked(TooltipProviderService::default, |svc, app| {
            svc.begin_frame(app.frame_id());
            let st = svc.current_state_mut();
            if st.pointer_in_transit == in_transit {
                return;
            }
            st.pointer_in_transit = in_transit;

            let model = st.pointer_in_transit_model.clone().unwrap_or_else(|| {
                let model = app.models_mut().insert(in_transit);
                st.pointer_in_transit_model = Some(model.clone());
                model
            });

            let _ = app.models_mut().update(&model, |v| *v = in_transit);
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_runtime::{FrameId, TickId};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn provider_stack_overrides_and_restores_config() {
        let window = AppWindowId::default();
        let mut app = App::new();
        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));

        let b = bounds();
        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let outer = TooltipProviderConfig::new(10, 30);
            with_tooltip_provider(cx, outer, |cx| {
                assert_eq!(current_config(cx), outer);

                let inner = TooltipProviderConfig::new(5, 6).disable_hoverable_content(true);
                with_tooltip_provider(cx, inner, |cx| {
                    assert_eq!(current_config(cx), inner);
                });

                assert_eq!(current_config(cx), outer);
            });

            assert_eq!(current_config(cx), TooltipProviderConfig::default());
        });
    }

    #[test]
    fn delay_group_is_scoped_to_provider() {
        let window = AppWindowId::default();
        let mut app = App::new();
        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));

        let b = bounds();
        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let cfg = TooltipProviderConfig::new(10, 30);
            with_tooltip_provider(cx, cfg, |cx| {
                assert_eq!(open_delay_ticks(cx, 100), 10);
                note_closed(cx, 120);
                assert_eq!(open_delay_ticks(cx, 121), 0);
                assert_eq!(open_delay_ticks(cx, 151), 10);
            });
        });
    }

    #[test]
    fn provider_close_delay_ticks_are_exposed_in_config() {
        let window = AppWindowId::default();
        let mut app = App::new();
        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));

        let b = bounds();
        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let cfg = TooltipProviderConfig::new(10, 30).close_delay_duration_ticks(7);
            with_tooltip_provider(cx, cfg, |cx| {
                let got = current_config(cx);
                assert_eq!(got.delay_duration_ticks, 10);
                assert_eq!(got.skip_delay_duration_ticks, 30);
                assert_eq!(got.close_delay_duration_ticks, Some(7));
            });
        });
    }

    #[test]
    fn provider_stack_is_cleared_each_frame() {
        let window = AppWindowId::default();
        let mut app = App::new();
        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));

        let b = bounds();
        fret_ui::elements::with_element_cx(&mut app, window, b, "frame1", |cx| {
            let cfg = TooltipProviderConfig::new(10, 30);
            with_tooltip_provider(cx, cfg, |_cx| {});
        });

        app.set_frame_id(FrameId(2));
        fret_ui::elements::with_element_cx(&mut app, window, b, "frame2", |cx| {
            assert_eq!(current_config(cx), TooltipProviderConfig::default());
        });
    }

    #[test]
    fn note_opened_tracks_last_opened_tooltip() {
        let window = AppWindowId::default();
        let mut app = App::new();
        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));

        let b = bounds();
        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let cfg = TooltipProviderConfig::new(10, 30);
            with_tooltip_provider(cx, cfg, |cx| {
                let t1 = GlobalElementId(0x101);
                let t2 = GlobalElementId(0x202);

                assert_eq!(last_opened_tooltip(cx), None);
                let tok1 = note_opened_tooltip(cx, t1);
                assert_eq!(last_opened_tooltip(cx), Some((t1, tok1)));
                let tok2 = note_opened_tooltip(cx, t2);
                assert_eq!(last_opened_tooltip(cx), Some((t2, tok2)));
                assert!(tok2 > tok1);
            });
        });
    }

    #[test]
    fn pointer_in_transit_model_tracks_state_changes() {
        let window = AppWindowId::default();
        let mut app = App::new();
        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));

        let b = bounds();
        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            assert!(!is_pointer_in_transit(cx));
            let model = pointer_in_transit_model(cx);
            assert_eq!(
                cx.app.models().read(&model, |v| *v).ok(),
                Some(false),
                "expected model to reflect initial transit state"
            );

            set_pointer_in_transit(cx, true);
            assert!(is_pointer_in_transit(cx));
            assert_eq!(cx.app.models().read(&model, |v| *v).ok(), Some(true));

            set_pointer_in_transit(cx, false);
            assert!(!is_pointer_in_transit(cx));
            assert_eq!(cx.app.models().read(&model, |v| *v).ok(), Some(false));
        });
    }
}
