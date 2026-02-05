use std::sync::Arc;

use super::TanStackValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltInAggregationFn {
    Sum,
    Min,
    Max,
    Extent,
    Mean,
    Median,
    Unique,
    UniqueCount,
    Count,
}

impl BuiltInAggregationFn {
    pub fn from_tanstack_key(key: &str) -> Option<Self> {
        match key {
            "sum" => Some(Self::Sum),
            "min" => Some(Self::Min),
            "max" => Some(Self::Max),
            "extent" => Some(Self::Extent),
            "mean" => Some(Self::Mean),
            "median" => Some(Self::Median),
            "unique" => Some(Self::Unique),
            "uniqueCount" => Some(Self::UniqueCount),
            "count" => Some(Self::Count),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregationFnSpec {
    /// TanStack `aggregationFn: 'auto'` (default).
    Auto,
    BuiltIn(BuiltInAggregationFn),
    /// TanStack `aggregationFn: <string>` resolved via `options.aggregationFns[key] ?? builtIn[key]`.
    Named(Arc<str>),
    /// No aggregation for this column.
    None,
}

impl Default for AggregationFnSpec {
    fn default() -> Self {
        Self::Auto
    }
}

pub type AggregationFn = Arc<dyn Fn(&str, &[TanStackValue]) -> TanStackValue + Send + Sync>;

fn value_key(v: &TanStackValue) -> String {
    match v {
        TanStackValue::Undefined => "u".into(),
        TanStackValue::Null => "n".into(),
        TanStackValue::Bool(b) => format!("b:{b}"),
        TanStackValue::Number(n) => {
            if n.is_nan() {
                "num:nan".into()
            } else if *n == 0.0 {
                // SameValueZero: -0 and 0 are treated as equal.
                "num:0".into()
            } else {
                format!("num:{:016x}", n.to_bits())
            }
        }
        TanStackValue::String(s) => format!("s:{s}"),
        TanStackValue::Array(arr) => {
            let inner = arr.iter().map(value_key).collect::<Vec<_>>().join(",");
            format!("a:[{inner}]")
        }
        TanStackValue::DateTime(n) => {
            if n.is_nan() {
                "dt:nan".into()
            } else if *n == 0.0 {
                "dt:0".into()
            } else {
                format!("dt:{:016x}", n.to_bits())
            }
        }
    }
}

fn as_f64_strict(v: &TanStackValue) -> Option<f64> {
    match v {
        TanStackValue::Number(n) => Some(*n),
        TanStackValue::DateTime(n) => Some(*n),
        _ => None,
    }
}

fn to_number_like(v: &TanStackValue) -> Option<f64> {
    match v {
        TanStackValue::Number(n) => Some(*n),
        TanStackValue::DateTime(n) => Some(*n),
        TanStackValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        TanStackValue::Null => Some(0.0),
        TanStackValue::String(s) => s.parse::<f64>().ok(),
        TanStackValue::Undefined | TanStackValue::Array(_) => None,
    }
}

pub fn apply_builtin_aggregation(
    agg: BuiltInAggregationFn,
    values: &[TanStackValue],
) -> TanStackValue {
    match agg {
        BuiltInAggregationFn::Count => TanStackValue::Number(values.len() as f64),
        BuiltInAggregationFn::Sum => {
            let mut sum = 0.0;
            for v in values {
                let Some(n) = as_f64_strict(v) else {
                    continue;
                };
                if n.is_nan() {
                    continue;
                }
                sum += n;
            }
            TanStackValue::Number(sum)
        }
        BuiltInAggregationFn::Min => {
            let mut min: Option<f64> = None;
            for v in values {
                let Some(n) = as_f64_strict(v) else {
                    continue;
                };
                if n.is_nan() {
                    continue;
                }
                min = Some(match min {
                    Some(acc) => acc.min(n),
                    None => n,
                });
            }
            min.map(TanStackValue::Number)
                .unwrap_or(TanStackValue::Undefined)
        }
        BuiltInAggregationFn::Max => {
            let mut max: Option<f64> = None;
            for v in values {
                let Some(n) = as_f64_strict(v) else {
                    continue;
                };
                if n.is_nan() {
                    continue;
                }
                max = Some(match max {
                    Some(acc) => acc.max(n),
                    None => n,
                });
            }
            max.map(TanStackValue::Number)
                .unwrap_or(TanStackValue::Undefined)
        }
        BuiltInAggregationFn::Extent => {
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;
            for v in values {
                let Some(n) = as_f64_strict(v) else {
                    continue;
                };
                if n.is_nan() {
                    continue;
                }
                if min.is_none() {
                    min = Some(n);
                    max = Some(n);
                } else {
                    min = Some(min.unwrap().min(n));
                    max = Some(max.unwrap().max(n));
                }
            }
            TanStackValue::Array(vec![
                min.map(TanStackValue::Number)
                    .unwrap_or(TanStackValue::Undefined),
                max.map(TanStackValue::Number)
                    .unwrap_or(TanStackValue::Undefined),
            ])
        }
        BuiltInAggregationFn::Mean => {
            let mut sum = 0.0;
            let mut count = 0usize;
            for v in values {
                let Some(n) = to_number_like(v) else {
                    continue;
                };
                if n.is_nan() {
                    continue;
                }
                count += 1;
                sum += n;
            }
            if count == 0 {
                return TanStackValue::Undefined;
            }
            TanStackValue::Number(sum / (count as f64))
        }
        BuiltInAggregationFn::Median => {
            if values.is_empty() {
                return TanStackValue::Undefined;
            }
            let mut nums: Vec<f64> = Vec::with_capacity(values.len());
            for v in values {
                let Some(n) = as_f64_strict(v) else {
                    return TanStackValue::Undefined;
                };
                if n.is_nan() {
                    return TanStackValue::Undefined;
                }
                nums.push(n);
            }

            if nums.len() == 1 {
                return TanStackValue::Number(nums[0]);
            }

            nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let mid = nums.len() / 2;
            if nums.len() % 2 == 1 {
                TanStackValue::Number(nums[mid])
            } else {
                TanStackValue::Number((nums[mid - 1] + nums[mid]) / 2.0)
            }
        }
        BuiltInAggregationFn::Unique => {
            let mut seen: std::collections::HashSet<String> = Default::default();
            let mut out: Vec<TanStackValue> = Vec::new();
            for v in values {
                let key = value_key(v);
                if !seen.insert(key) {
                    continue;
                }
                out.push(v.clone());
            }
            TanStackValue::Array(out)
        }
        BuiltInAggregationFn::UniqueCount => {
            let mut seen: std::collections::HashSet<String> = Default::default();
            for v in values {
                seen.insert(value_key(v));
            }
            TanStackValue::Number(seen.len() as f64)
        }
    }
}

pub fn resolve_auto_aggregation(values: &[TanStackValue]) -> Option<BuiltInAggregationFn> {
    for v in values {
        match v {
            TanStackValue::Undefined | TanStackValue::Null => continue,
            TanStackValue::Number(_) => return Some(BuiltInAggregationFn::Sum),
            TanStackValue::DateTime(_) => return Some(BuiltInAggregationFn::Extent),
            _ => return None,
        }
    }
    None
}
