pub type Direction = i8;

pub const DIRECTION_NONE: Direction = 0;
pub const DIRECTION_FORWARD: Direction = -1;
pub const DIRECTION_BACKWARD: Direction = 1;

#[inline]
pub fn math_abs(v: f32) -> f32 {
    v.abs()
}

#[inline]
pub fn math_sign(v: f32) -> f32 {
    v.signum()
}

#[inline]
pub fn delta_abs(b: f32, a: f32) -> f32 {
    (b - a).abs()
}

/// Ported from Embla `factorAbs(inputB, inputA)`.
///
/// Upstream: `repo-ref/embla-carousel/packages/embla-carousel/src/components/utils.ts`
#[inline]
pub fn factor_abs(input_b: f32, input_a: f32) -> f32 {
    if input_b == 0.0 || input_a == 0.0 {
        return 0.0;
    }
    if input_b.abs() <= input_a.abs() {
        return 0.0;
    }
    let diff = delta_abs(input_b.abs(), input_a.abs());
    (diff / input_b).abs()
}

#[inline]
pub fn array_last<T: Copy>(items: &[T]) -> T {
    items[items.len().saturating_sub(1)]
}
