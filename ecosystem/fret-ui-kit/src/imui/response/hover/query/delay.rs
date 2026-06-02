use super::super::{ImUiHoveredFlags, ResponseExt};

pub(super) fn delay_requirements_met(response: ResponseExt, flags: ImUiHoveredFlags) -> bool {
    let delay_normal = flags.contains(ImUiHoveredFlags::DELAY_NORMAL);
    let delay_short = flags.contains(ImUiHoveredFlags::DELAY_SHORT);
    let stationary = flags.contains(ImUiHoveredFlags::STATIONARY);
    let no_shared_delay = flags.contains(ImUiHoveredFlags::NO_SHARED_DELAY);

    if delay_normal {
        let delay_met = if no_shared_delay {
            response.hover_delay_normal_met
        } else {
            response.hover_delay_normal_shared_met || response.hover_delay_normal_met
        };
        return response.hover_stationary_met && delay_met;
    }

    if delay_short {
        let delay_met = if no_shared_delay {
            response.hover_delay_short_met
        } else {
            response.hover_delay_short_shared_met || response.hover_delay_short_met
        };
        return response.hover_stationary_met && delay_met;
    }

    !stationary || response.hover_stationary_met
}
