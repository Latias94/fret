//! TanStack-inspired aggregation helpers.
//!
//! This is intentionally small and `u64`-focused for now. The UI layer can format `u64` results
//! (e.g. currency/units) as needed.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Aggregation {
    #[default]
    None,
    Count,
    SumU64,
    MinU64,
    MaxU64,
    MeanU64,
}

pub fn aggregate_u64(
    agg: Aggregation,
    mut values: impl Iterator<Item = u64>,
    count: usize,
) -> Option<u64> {
    match agg {
        Aggregation::None => None,
        Aggregation::Count => Some(count as u64),
        Aggregation::SumU64 => values.try_fold(0u64, |acc, v| acc.checked_add(v)),
        Aggregation::MinU64 => values.min(),
        Aggregation::MaxU64 => values.max(),
        Aggregation::MeanU64 => {
            if count == 0 {
                return None;
            }
            let sum = values.try_fold(0u64, |acc, v| acc.checked_add(v))?;
            Some(sum / (count as u64))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_ignores_values() {
        assert_eq!(
            aggregate_u64(Aggregation::Count, [1, 2, 3].into_iter(), 10),
            Some(10)
        );
    }

    #[test]
    fn sum_checked() {
        assert_eq!(
            aggregate_u64(Aggregation::SumU64, [1, 2, 3].into_iter(), 3),
            Some(6)
        );
        assert_eq!(
            aggregate_u64(Aggregation::SumU64, [u64::MAX, 1].into_iter(), 2),
            None
        );
    }

    #[test]
    fn min_max() {
        assert_eq!(
            aggregate_u64(Aggregation::MinU64, [3, 2, 1].into_iter(), 3),
            Some(1)
        );
        assert_eq!(
            aggregate_u64(Aggregation::MaxU64, [3, 2, 1].into_iter(), 3),
            Some(3)
        );
        assert_eq!(aggregate_u64(Aggregation::MinU64, [].into_iter(), 0), None);
        assert_eq!(aggregate_u64(Aggregation::MaxU64, [].into_iter(), 0), None);
    }

    #[test]
    fn mean_uses_count_and_is_checked() {
        assert_eq!(
            aggregate_u64(Aggregation::MeanU64, [10, 20].into_iter(), 2),
            Some(15)
        );
        assert_eq!(
            aggregate_u64(Aggregation::MeanU64, [10].into_iter(), 0),
            None
        );
        assert_eq!(
            aggregate_u64(Aggregation::MeanU64, [u64::MAX, 1].into_iter(), 2),
            None
        );
    }
}
