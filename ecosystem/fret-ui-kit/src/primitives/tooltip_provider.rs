//! Tooltip provider helpers (Radix-aligned outcomes).
//!
//! Radix tooltips commonly share a provider so:
//! - the first tooltip opens after a delay, but
//! - moving between tooltips shortly after closing skips the delay.
//!
//! In Fret:
//! - the deterministic delay-window logic lives in `crate::headless::tooltip_delay_group`,
//! - while the provider stack/service is driven via `crate::tooltip_provider`.

pub use crate::tooltip_provider::{
    TooltipProviderConfig, current_config, is_pointer_in_transit, last_opened_tooltip, note_closed,
    note_opened_tooltip, open_delay_ticks, open_delay_ticks_with_base, pointer_in_transit_model,
    pointer_transit_geometry_model, set_pointer_in_transit, set_pointer_transit_geometry,
    with_tooltip_provider,
};
