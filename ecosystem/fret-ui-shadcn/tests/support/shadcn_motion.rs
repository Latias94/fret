#![allow(dead_code)]

use std::time::Duration;

pub(crate) fn ticks_for_duration(duration: Duration) -> u64 {
    fret_ui_kit::declarative::transition::ticks_60hz_for_duration(duration)
}

pub(crate) fn ticks_100() -> u64 {
    ticks_for_duration(Duration::from_millis(100))
}

pub(crate) fn ticks_200() -> u64 {
    ticks_for_duration(Duration::from_millis(200))
}

pub(crate) fn ticks_300() -> u64 {
    ticks_for_duration(Duration::from_millis(300))
}

pub(crate) fn ticks_500() -> u64 {
    ticks_for_duration(Duration::from_millis(500))
}
