//! Small, reusable numeric formatting helpers for editor controls.
//!
//! These helpers live in the policy layer (`fret-ui-editor`) so we can iterate on "editor feel"
//! without expanding the `fret-ui` mechanism surface.

use std::sync::Arc;

use crate::controls::numeric_input::{NumericFormatFn, NumericParseFn};
use crate::primitives::drag_value_core::DragValueScalar;

pub fn percent_0_1_format<T>(decimals: usize) -> NumericFormatFn<T>
where
    T: DragValueScalar + Send + Sync + 'static,
{
    let decimals = decimals.min(6);
    Arc::new(move |v| {
        let pct = v.to_f64() * 100.0;
        if decimals == 0 {
            Arc::from(format!("{:.0}%", pct))
        } else {
            Arc::from(format!("{:.*}%", decimals, pct))
        }
    })
}

pub fn percent_0_1_parse<T>() -> NumericParseFn<T>
where
    T: DragValueScalar + Send + Sync + 'static,
{
    Arc::new(|s| {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        if let Some(raw) = s.strip_suffix('%') {
            let v = raw.trim().parse::<f64>().ok()?;
            return Some(T::from_f64(v / 100.0));
        }

        let v = s.parse::<f64>().ok()?;

        // Heuristic: when editing a normalized 0..1 value that is *displayed* as percent, users
        // tend to type `50` (meaning 50%), not `0.5`. Treat values outside [0, 1] as percent.
        if v.abs() > 1.0 {
            return Some(T::from_f64(v / 100.0));
        }

        Some(T::from_f64(v))
    })
}
