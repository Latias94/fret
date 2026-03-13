//! Searcher (palette) helpers for node insertion and conversion workflows.
//!
//! This module is UI-light: it builds rows for a searcher overlay, while the canvas owns event
//! routing and rendering.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::core::NodeKindKey;
use crate::ui::presenter::InsertNodeCandidate;

mod searcher_build;
mod searcher_score;
pub(crate) use searcher_build::{build_rows, build_rows_flat};

pub(crate) const SEARCHER_MAX_VISIBLE_ROWS: usize = 12;

#[derive(Debug, Clone)]
pub(crate) enum SearcherRowKind {
    Header,
    Candidate { candidate_ix: usize },
}

#[derive(Debug, Clone)]
pub(crate) struct SearcherRow {
    pub kind: SearcherRowKind,
    pub label: Arc<str>,
    pub enabled: bool,
}
