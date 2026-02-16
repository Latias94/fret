//! Headless UI state machines and small deterministic helpers.
//!
//! This crate intentionally avoids theme/recipe policy and contains only reusable logic that can
//! be shared across UI kits and component ecosystems.

// This crate is intentionally "logic dense" (state machines, snapshot helpers, and parity code),
// where refactors to satisfy certain style lints can harm readability without improving correctness.
#![allow(
    clippy::field_reassign_with_default,
    clippy::too_many_arguments,
    clippy::type_complexity
)]

pub mod calendar;
pub mod calendar_solar_hijri;
pub mod carousel;
pub mod checked_state;
pub mod cmdk_score;
pub mod cmdk_selection;
pub mod easing;
pub mod form_state;
pub mod form_validation;
pub mod grid_viewport;
pub mod hover_intent;
pub mod menu_nav;
pub mod motion;
pub mod presence;
pub mod roving_focus;
pub mod safe_hover;
pub mod scroll_area;
pub mod scroll_area_visibility;
pub mod select_item_aligned;
pub mod slider;
pub mod table;
pub mod tooltip_delay_group;
pub mod tooltip_intent;
pub mod transition;
pub mod typeahead;
