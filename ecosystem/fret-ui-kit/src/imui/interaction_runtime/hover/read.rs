use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Debug, Default, Clone, Copy)]
struct HoverQueryDelayLocalState {
    stationary_met: bool,
    delay_short_met: bool,
    delay_normal_met: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub(in crate::imui) struct HoverQueryDelayRead {
    pub(in crate::imui) stationary_met: bool,
    pub(in crate::imui) delay_short_met: bool,
    pub(in crate::imui) delay_normal_met: bool,
    pub(in crate::imui) shared_delay_short_met: bool,
    pub(in crate::imui) shared_delay_normal_met: bool,
}

pub(super) fn read_hover_query_delay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    hovered_raw: bool,
    shared_delay_model: &fret_runtime::Model<super::shared_delay::ImUiSharedHoverDelayState>,
) -> HoverQueryDelayRead {
    let stationary_fired = cx.take_transient_for(id, crate::imui::KEY_HOVER_STATIONARY_MET);
    let delay_short_fired = cx.take_transient_for(id, crate::imui::KEY_HOVER_DELAY_SHORT_MET);
    let delay_normal_fired = cx.take_transient_for(id, crate::imui::KEY_HOVER_DELAY_NORMAL_MET);

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
            shared_delay_model,
            fret_ui::Invalidation::Paint,
            |_app, st| st.delay_flags(),
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
