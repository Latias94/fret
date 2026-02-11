#[derive(Debug, Clone)]
pub(super) struct DdminReduction {
    pub(super) removed: Vec<usize>,
    pub(super) kept_len: usize,
    pub(super) granularity: usize,
}

/// Delta debugging ("ddmin") over an ordered set of step indices.
///
/// Starts from `0..total_len` and tries removing contiguous chunks while `test(keep)` remains true.
/// Returns the minimal-ish set of kept indices (not necessarily globally minimal, but usually good),
/// plus a small reduction history for auditability.
pub(super) fn ddmin_keep_indices(
    total_len: usize,
    min_len: usize,
    max_iters: u64,
    mut test: impl FnMut(&[usize]) -> bool,
) -> (Vec<usize>, Vec<DdminReduction>, u64) {
    let mut keep: Vec<usize> = (0..total_len).collect();
    let min_len = min_len.min(total_len);

    let mut reductions: Vec<DdminReduction> = Vec::new();
    let mut granularity: usize = 2;
    let mut iters: u64 = 0;

    while keep.len() > min_len && iters < max_iters {
        iters += 1;

        let len = keep.len();
        let chunks = granularity.min(len).max(2);
        let chunk_size = (len + chunks - 1) / chunks;

        let mut reduced_this_round = false;
        let mut chunk_start: usize = 0;
        while chunk_start < len {
            let chunk_end = (chunk_start + chunk_size).min(len);
            let removed: Vec<usize> = keep[chunk_start..chunk_end].to_vec();

            if len.saturating_sub(removed.len()) < min_len {
                chunk_start = chunk_end;
                continue;
            }

            let mut candidate = keep.clone();
            candidate.drain(chunk_start..chunk_end);

            if test(&candidate) {
                keep = candidate;
                reductions.push(DdminReduction {
                    removed,
                    kept_len: keep.len(),
                    granularity,
                });
                granularity = (granularity.saturating_sub(1)).max(2);
                reduced_this_round = true;
                break;
            }

            chunk_start = chunk_end;
        }

        if !reduced_this_round {
            if granularity >= keep.len() {
                break;
            }
            granularity = (granularity.saturating_mul(2)).min(keep.len());
        }
    }

    (keep, reductions, iters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ddmin_reduces_to_minimal_pair() {
        let (keep, _reductions, _iters) =
            ddmin_keep_indices(8, 1, 1_000, |keep| keep.contains(&2) && keep.contains(&6));
        assert_eq!(keep, vec![2, 6]);
    }

    #[test]
    fn ddmin_can_reduce_to_single_step_when_any_nonempty_passes() {
        let (keep, _reductions, _iters) = ddmin_keep_indices(10, 1, 1_000, |keep| !keep.is_empty());
        assert_eq!(keep.len(), 1);
    }
}
