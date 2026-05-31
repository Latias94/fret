use std::sync::Arc;

use crate::controls::numeric_input::{NumericParseFn, NumericValidateFn};
use crate::primitives::drag_value_core::DragValueScalar;

use super::value_math::quantize_value;

#[cfg(test)]
mod tests;

pub(super) fn slider_typing_parse<T: DragValueScalar>(
    parse: NumericParseFn<T>,
    min: f64,
    max: f64,
    clamp: bool,
    step: Option<f64>,
) -> NumericParseFn<T> {
    Arc::new(move |s| {
        let v = parse(s)?;
        let next = quantize_value(min, max, clamp, step, v.to_f64());
        Some(T::from_f64(next))
    })
}

pub(super) fn slider_typing_validate<T: DragValueScalar>(
    validate: Option<NumericValidateFn<T>>,
    min: f64,
    max: f64,
    clamp: bool,
) -> Option<NumericValidateFn<T>> {
    if clamp {
        return validate;
    }

    Some(Arc::new(move |v| {
        let f = v.to_f64();
        if f < min || f > max {
            return Some(Arc::from("Out of range"));
        }
        if let Some(validate) = validate.as_ref() {
            validate(v)
        } else {
            None
        }
    }))
}
