//! Tooltip delay-group logic (Radix TooltipProvider-aligned outcomes).
//!
//! This is a deterministic, tick-based state machine: the first tooltip opens after a delay, but
//! subsequent tooltips opened shortly after a close can skip the delay.

pub use crate::headless::tooltip_delay_group::{TooltipDelayGroupConfig, TooltipDelayGroupState};
