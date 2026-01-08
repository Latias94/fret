//! Searcher (palette) helpers for node insertion and conversion workflows.
//!
//! This module is UI-light: it builds rows for a searcher overlay, while the canvas owns event
//! routing and rendering.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::core::NodeKindKey;
use crate::ui::presenter::InsertNodeCandidate;

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

fn normalize(s: &str) -> String {
    s.trim().to_ascii_lowercase()
}

fn split_category_label(label: &str) -> (Option<&str>, &str) {
    label
        .rsplit_once('/')
        .map(|(cat, name)| (Some(cat), name))
        .unwrap_or((None, label))
}

fn score_candidate(query: &str, label: &str, kind: &str) -> Option<(u8, usize)> {
    if query.is_empty() {
        return Some((0, 0));
    }
    let label_lc = label.to_ascii_lowercase();
    if let Some(ix) = label_lc.find(query) {
        let bucket = if ix == 0 { 0 } else { 1 };
        return Some((bucket, ix));
    }
    let kind_lc = kind.to_ascii_lowercase();
    kind_lc.find(query).map(|ix| (2, ix))
}

pub(crate) fn build_rows(
    candidates: &[InsertNodeCandidate],
    query: &str,
    recent_kinds: &[NodeKindKey],
) -> Vec<SearcherRow> {
    let q = normalize(query);
    let mut rows: Vec<SearcherRow> = Vec::new();

    if !q.is_empty() {
        let mut scored: Vec<(u8, usize, usize)> = Vec::new();
        for (ix, c) in candidates.iter().enumerate() {
            let Some((bucket, pos)) = score_candidate(&q, c.label.as_ref(), c.kind.0.as_str())
            else {
                continue;
            };
            scored.push((bucket, pos, ix));
        }
        scored.sort_by(|a, b| a.cmp(b));

        for (_bucket, _pos, ix) in scored {
            let c = &candidates[ix];
            rows.push(SearcherRow {
                kind: SearcherRowKind::Candidate { candidate_ix: ix },
                label: c.label.clone(),
                enabled: c.enabled,
            });
        }

        if rows.is_empty() {
            rows.push(SearcherRow {
                kind: SearcherRowKind::Header,
                label: Arc::<str>::from("No matches"),
                enabled: false,
            });
        }

        return rows;
    }

    let mut kind_to_ix: HashMap<&str, usize> = HashMap::new();
    for (ix, c) in candidates.iter().enumerate() {
        kind_to_ix.insert(c.kind.0.as_str(), ix);
    }

    let mut used: HashSet<usize> = HashSet::new();

    // Common / recent section.
    let mut common: Vec<usize> = Vec::new();
    for kind in recent_kinds {
        if let Some(ix) = kind_to_ix.get(kind.0.as_str()).copied()
            && used.insert(ix)
        {
            common.push(ix);
        }
    }
    if !common.is_empty() {
        rows.push(SearcherRow {
            kind: SearcherRowKind::Header,
            label: Arc::<str>::from("Recent"),
            enabled: false,
        });
        for ix in common {
            let c = &candidates[ix];
            rows.push(SearcherRow {
                kind: SearcherRowKind::Candidate { candidate_ix: ix },
                label: c.label.clone(),
                enabled: c.enabled,
            });
        }
    }

    // Grouped catalog.
    let mut grouped: BTreeMap<String, Vec<(usize, Arc<str>)>> = BTreeMap::new();
    for (ix, c) in candidates.iter().enumerate() {
        if !used.insert(ix) {
            continue;
        }
        let (cat, name) = split_category_label(c.label.as_ref());
        let display = if cat.is_some() {
            Arc::<str>::from(name)
        } else {
            c.label.clone()
        };
        grouped
            .entry(cat.unwrap_or("Other").to_string())
            .or_default()
            .push((ix, display));
    }

    for (cat, items) in grouped {
        rows.push(SearcherRow {
            kind: SearcherRowKind::Header,
            label: Arc::<str>::from(cat),
            enabled: false,
        });
        for (ix, display) in items {
            let c = &candidates[ix];
            rows.push(SearcherRow {
                kind: SearcherRowKind::Candidate { candidate_ix: ix },
                label: display,
                enabled: c.enabled,
            });
        }
    }

    if rows.is_empty() {
        rows.push(SearcherRow {
            kind: SearcherRowKind::Header,
            label: Arc::<str>::from("No candidates"),
            enabled: false,
        });
    }

    rows
}

pub(crate) fn build_rows_flat(candidates: &[InsertNodeCandidate], query: &str) -> Vec<SearcherRow> {
    let q = normalize(query);
    let mut rows: Vec<SearcherRow> = Vec::new();

    if !q.is_empty() {
        let mut scored: Vec<(u8, usize, usize)> = Vec::new();
        for (ix, c) in candidates.iter().enumerate() {
            let Some((bucket, pos)) = score_candidate(&q, c.label.as_ref(), c.kind.0.as_str())
            else {
                continue;
            };
            scored.push((bucket, pos, ix));
        }
        scored.sort_by(|a, b| a.cmp(b));
        for (_bucket, _pos, ix) in scored {
            let c = &candidates[ix];
            rows.push(SearcherRow {
                kind: SearcherRowKind::Candidate { candidate_ix: ix },
                label: c.label.clone(),
                enabled: c.enabled,
            });
        }
        if rows.is_empty() {
            rows.push(SearcherRow {
                kind: SearcherRowKind::Header,
                label: Arc::<str>::from("No matches"),
                enabled: false,
            });
        }
        return rows;
    }

    for (ix, c) in candidates.iter().enumerate() {
        rows.push(SearcherRow {
            kind: SearcherRowKind::Candidate { candidate_ix: ix },
            label: c.label.clone(),
            enabled: c.enabled,
        });
    }
    if rows.is_empty() {
        rows.push(SearcherRow {
            kind: SearcherRowKind::Header,
            label: Arc::<str>::from("No candidates"),
            enabled: false,
        });
    }
    rows
}
