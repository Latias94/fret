use std::sync::Arc;

use fret_core::Modifiers;

use super::{ImUiMultiSelectState, apply_click};

fn keys() -> Vec<Arc<str>> {
    vec![
        Arc::from("alpha"),
        Arc::from("beta"),
        Arc::from("gamma"),
        Arc::from("delta"),
    ]
}

mod clicks;
mod ordered_selection;
