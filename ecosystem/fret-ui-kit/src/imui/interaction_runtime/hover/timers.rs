use fret_ui::GlobalElementId;

const HOVER_TIMER_KIND_STATIONARY: u64 =
    crate::imui::fnv1a64(b"fret-ui-kit.imui.hover.timer.stationary.v1");
const HOVER_TIMER_KIND_DELAY_SHORT: u64 =
    crate::imui::fnv1a64(b"fret-ui-kit.imui.hover.timer.delay_short.v1");
const HOVER_TIMER_KIND_DELAY_NORMAL: u64 =
    crate::imui::fnv1a64(b"fret-ui-kit.imui.hover.timer.delay_normal.v1");

pub(super) fn stationary_token_for(element: GlobalElementId) -> fret_runtime::TimerToken {
    hover_timer_token_for(HOVER_TIMER_KIND_STATIONARY, element)
}

pub(super) fn delay_short_token_for(element: GlobalElementId) -> fret_runtime::TimerToken {
    hover_timer_token_for(HOVER_TIMER_KIND_DELAY_SHORT, element)
}

pub(super) fn delay_normal_token_for(element: GlobalElementId) -> fret_runtime::TimerToken {
    hover_timer_token_for(HOVER_TIMER_KIND_DELAY_NORMAL, element)
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
