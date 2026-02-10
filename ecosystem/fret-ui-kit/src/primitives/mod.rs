//! Radix-aligned primitive facades.
//!
//! This module exists to keep the `fret-ui-kit` foundation surface organized around
//! Radix UI Primitives concepts, while remaining Rust-native and renderer-agnostic.
//!
//! # Where code should live (anti-duplication rules)
//!
//! This crate has three adjacent layers that can look similar if we are not strict:
//!
//! - `crate::headless`: pure logic / deterministic state machines / index math.
//! - `crate::declarative`: wiring helpers built on `ElementContext` + action hooks.
//! - `crate::primitives` (this module): **Radix-named stable entry points** (thin facades).
//!
//! To avoid drift and duplication:
//!
//! - If it is reusable **logic**, it belongs in `crate::headless` (and should be unit-testable).
//! - If it is reusable **wiring** (hooks, semantics stamping, overlay roots), it belongs in
//!   `crate::declarative` (and should have contract tests where appropriate).
//! - `crate::primitives` should stay thin: re-exports, small adapters, and stable naming aligned
//!   to <https://github.com/radix-ui/primitives> - not a second headless layer.
//!
//! Runtime mechanisms live in `fret-ui`; these facades intentionally port behavior outcomes, not
//! React/DOM APIs.

pub mod accordion;
pub mod active_descendant;
pub mod alert_dialog;
pub mod arrow;
pub mod aspect_ratio;
pub mod avatar;
pub mod checkbox;
pub mod collapsible;
pub mod collection;
pub mod combobox;
pub mod context_menu;
pub mod controllable_state;
pub mod dialog;
pub mod direction;
pub mod dismissable_layer;
pub mod dropdown_menu;
pub mod focus_scope;
pub mod hover_card;
pub mod hover_intent;
pub mod keyboard;
pub mod label;
pub mod menu;
pub mod menubar;
pub mod navigation_menu;
pub mod open_state;
pub mod popover;
pub mod popper;
pub mod popper_arrow;
pub mod popper_content;
pub mod portal;
pub mod presence;
pub mod progress;
pub mod radio_group;
pub mod roving_focus_group;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod toast;
pub mod toggle;
pub mod toggle_group;
pub mod toolbar;
pub mod tooltip;
pub mod tooltip_delay_group;
pub mod tooltip_provider;
pub mod transition;
pub mod trigger_a11y;
pub mod visually_hidden;
