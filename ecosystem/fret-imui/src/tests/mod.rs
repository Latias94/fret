mod harness;
use harness::*;

mod composition;
mod floating;
#[cfg(feature = "diagnostics")]
mod identity_diagnostics;
mod interaction_drag;
mod interaction_menu_tabs;
mod interaction_press;
mod interaction_shortcuts;
mod label_identity;
mod models_combo;
mod models_controls;
mod models_text_area;
mod models_text_basic;
mod models_text_commands;
mod models_text_filters;
mod models_text_identity;
mod models_text_lifecycle;
mod models_text_modes;
mod models_text_picker;
mod popup_hover;
