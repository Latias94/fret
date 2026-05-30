use super::super::{ImUiHoveredFlags, ResponseExt};

pub(super) fn nav_override_satisfied(response: ResponseExt, flags: ImUiHoveredFlags) -> bool {
    !flags.contains(ImUiHoveredFlags::NO_NAV_OVERRIDE) && response.nav_highlighted
}

pub(super) fn pointer_hovered_for_query(response: ResponseExt, flags: ImUiHoveredFlags) -> bool {
    let allow_disabled = flags.contains(ImUiHoveredFlags::ALLOW_WHEN_DISABLED);
    let allow_blocked_by_popup = flags.contains(ImUiHoveredFlags::ALLOW_WHEN_BLOCKED_BY_POPUP);

    let mut pointer_hovered = if allow_disabled {
        response.pointer_hovered_raw
    } else if response.enabled() {
        response.core.hovered
    } else {
        false
    };

    if allow_blocked_by_popup {
        pointer_hovered |= if allow_disabled || response.enabled {
            response.pointer_hovered_raw_below_barrier
        } else {
            false
        };
    }

    pointer_hovered
}

pub(super) fn active_item_allows_hover(response: ResponseExt, flags: ImUiHoveredFlags) -> bool {
    !response.hover_blocked_by_active_item
        || flags.contains(ImUiHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM)
}
