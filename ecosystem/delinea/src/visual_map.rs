use crate::engine::model::{VisualMapDomain, VisualMapModel, VisualMapRange};
use crate::ids::PaintId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualMapBucket {
    pub bucket: u16,
    pub in_range: bool,
}

pub fn bucket_index_for_value(map: &VisualMapModel, value: f64) -> u16 {
    let buckets = map.buckets.max(1) as f64;
    bucket_index(map.domain, buckets, value)
}

pub fn eval_bucket_for_value(
    map: &VisualMapModel,
    selected_range: Option<VisualMapRange>,
    selected_piece_mask: Option<u64>,
    value: f64,
) -> VisualMapBucket {
    let bucket = bucket_index_for_value(map, value);
    let in_range = match map.mode {
        crate::spec::VisualMapMode::Continuous => match selected_range {
            Some(r) => value.is_finite() && value >= r.min && value <= r.max,
            None => value.is_finite(),
        },
        crate::spec::VisualMapMode::Piecewise => {
            if !value.is_finite() {
                return VisualMapBucket {
                    bucket,
                    in_range: false,
                };
            }
            let buckets = map.buckets.clamp(1, 64) as u32;
            let full_mask = if buckets >= 64 {
                u64::MAX
            } else {
                (1u64 << buckets) - 1
            };
            let mask = selected_piece_mask.unwrap_or(full_mask);
            ((mask >> (bucket as u32)) & 1) == 1
        }
    };
    VisualMapBucket { bucket, in_range }
}

pub fn paint_id_for_bucket(bucket: VisualMapBucket) -> PaintId {
    PaintId::new(bucket.bucket as u64)
}

fn bucket_index(domain: VisualMapDomain, buckets: f64, value: f64) -> u16 {
    let Some(domain) = domain.sanitize() else {
        return 0;
    };
    if !value.is_finite() {
        return 0;
    }
    let span = domain.max - domain.min;
    if !span.is_finite() || span <= 0.0 || buckets <= 1.0 {
        return 0;
    }
    let t = ((value - domain.min) / span).clamp(0.0, 1.0);
    let mut idx = (t * buckets).floor() as u16;
    let max_idx = buckets as u16;
    if idx >= max_idx {
        idx = max_idx.saturating_sub(1);
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::model::VisualMapDomain;
    use crate::ids::VisualMapId;

    fn vm() -> VisualMapModel {
        VisualMapModel {
            id: VisualMapId::new(1),
            mode: crate::spec::VisualMapMode::Continuous,
            field: crate::ids::FieldId::new(1),
            domain: VisualMapDomain {
                min: 0.0,
                max: 10.0,
            },
            initial_range: None,
            initial_piece_mask: None,
            point_radius_mul_range: None,
            buckets: 5,
            out_of_range_opacity: 0.25,
        }
    }

    #[test]
    fn bucket_assignment_is_deterministic() {
        let vm = vm();
        let a = eval_bucket_for_value(&vm, None, None, 0.0);
        let b = eval_bucket_for_value(&vm, None, None, 0.0);
        assert_eq!(a, b);

        assert_eq!(eval_bucket_for_value(&vm, None, None, -1.0).bucket, 0);
        assert_eq!(eval_bucket_for_value(&vm, None, None, 10.0).bucket, 4);
        assert_eq!(eval_bucket_for_value(&vm, None, None, 9.999_9).bucket, 4);
    }

    #[test]
    fn range_classification_is_applied() {
        let vm = vm();
        let range = VisualMapRange { min: 2.0, max: 4.0 };

        assert!(eval_bucket_for_value(&vm, Some(range), None, 3.0).in_range);
        assert!(!eval_bucket_for_value(&vm, Some(range), None, 1.0).in_range);
        assert!(!eval_bucket_for_value(&vm, Some(range), None, f64::NAN).in_range);
    }

    #[test]
    fn piecewise_mask_selects_buckets() {
        let mut vm = vm();
        vm.mode = crate::spec::VisualMapMode::Piecewise;
        vm.buckets = 5;

        let mask = 0b0_01010u64;
        assert!(eval_bucket_for_value(&vm, None, Some(mask), 3.0).in_range);
        assert!(!eval_bucket_for_value(&vm, None, Some(mask), 1.0).in_range);
    }
}
