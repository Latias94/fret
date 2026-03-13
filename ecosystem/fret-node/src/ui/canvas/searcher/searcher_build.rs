use super::searcher_score::{normalize, score_candidate, split_category_label};
use super::*;

pub(crate) fn build_rows(
    candidates: &[InsertNodeCandidate],
    query: &str,
    recent_kinds: &[NodeKindKey],
) -> Vec<SearcherRow> {
    let query = normalize(query);
    let mut rows: Vec<SearcherRow> = Vec::new();

    if !query.is_empty() {
        let mut scored: Vec<(u8, usize, usize)> = Vec::new();
        for (index, candidate) in candidates.iter().enumerate() {
            let Some((bucket, position)) =
                score_candidate(&query, candidate.label.as_ref(), candidate.kind.0.as_str())
            else {
                continue;
            };
            scored.push((bucket, position, index));
        }
        scored.sort();

        for (_bucket, _position, index) in scored {
            let candidate = &candidates[index];
            rows.push(SearcherRow {
                kind: SearcherRowKind::Candidate {
                    candidate_ix: index,
                },
                label: candidate.label.clone(),
                enabled: candidate.enabled,
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
    for (index, candidate) in candidates.iter().enumerate() {
        kind_to_ix.insert(candidate.kind.0.as_str(), index);
    }

    let mut used: HashSet<usize> = HashSet::new();
    let mut common: Vec<usize> = Vec::new();
    for kind in recent_kinds {
        if let Some(index) = kind_to_ix.get(kind.0.as_str()).copied()
            && used.insert(index)
        {
            common.push(index);
        }
    }
    if !common.is_empty() {
        rows.push(SearcherRow {
            kind: SearcherRowKind::Header,
            label: Arc::<str>::from("Recent"),
            enabled: false,
        });
        for index in common {
            let candidate = &candidates[index];
            rows.push(SearcherRow {
                kind: SearcherRowKind::Candidate {
                    candidate_ix: index,
                },
                label: candidate.label.clone(),
                enabled: candidate.enabled,
            });
        }
    }

    let mut grouped: BTreeMap<String, Vec<(usize, Arc<str>)>> = BTreeMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if !used.insert(index) {
            continue;
        }
        let (category, name) = split_category_label(candidate.label.as_ref());
        let display = if category.is_some() {
            Arc::<str>::from(name)
        } else {
            candidate.label.clone()
        };
        grouped
            .entry(category.unwrap_or("Other").to_string())
            .or_default()
            .push((index, display));
    }

    for (category, items) in grouped {
        rows.push(SearcherRow {
            kind: SearcherRowKind::Header,
            label: Arc::<str>::from(category),
            enabled: false,
        });
        for (index, display) in items {
            let candidate = &candidates[index];
            rows.push(SearcherRow {
                kind: SearcherRowKind::Candidate {
                    candidate_ix: index,
                },
                label: display,
                enabled: candidate.enabled,
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
    let query = normalize(query);
    let mut rows: Vec<SearcherRow> = Vec::new();

    if !query.is_empty() {
        let mut scored: Vec<(u8, usize, usize)> = Vec::new();
        for (index, candidate) in candidates.iter().enumerate() {
            let Some((bucket, position)) =
                score_candidate(&query, candidate.label.as_ref(), candidate.kind.0.as_str())
            else {
                continue;
            };
            scored.push((bucket, position, index));
        }
        scored.sort();
        for (_bucket, _position, index) in scored {
            let candidate = &candidates[index];
            rows.push(SearcherRow {
                kind: SearcherRowKind::Candidate {
                    candidate_ix: index,
                },
                label: candidate.label.clone(),
                enabled: candidate.enabled,
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

    for (index, candidate) in candidates.iter().enumerate() {
        rows.push(SearcherRow {
            kind: SearcherRowKind::Candidate {
                candidate_ix: index,
            },
            label: candidate.label.clone(),
            enabled: candidate.enabled,
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
