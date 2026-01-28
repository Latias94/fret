//! Headless UI state machines and small deterministic helpers.
//!
//! This crate intentionally avoids theme/recipe policy and contains only reusable logic that can
//! be shared across UI kits and component ecosystems.

pub mod calendar;
pub mod calendar_solar_hijri;
pub mod checked_state;
pub mod cmdk_score;
pub mod cmdk_selection;
pub mod easing;
pub mod form_state;
pub mod form_validation;
pub mod grid_viewport;
pub mod hover_intent;
pub mod menu_nav;
pub mod presence;
pub mod roving_focus;
pub mod safe_hover;
pub mod scroll_area;
pub mod scroll_area_visibility;
pub mod select_item_aligned;
pub mod slider;
pub mod table;
pub mod tooltip_delay_group;
pub mod transition;
pub mod typeahead;
