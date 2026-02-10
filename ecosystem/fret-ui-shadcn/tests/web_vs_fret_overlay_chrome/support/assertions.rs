use super::*;

pub(crate) fn assert_close(label: &str, actual: f32, expected: f32, tol: f32) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected={expected} (tol={tol}) got={actual} (delta={delta})"
    );
}

pub(crate) fn assert_rgba_close(
    label: &str,
    actual: css_color::Rgba,
    expected: css_color::Rgba,
    tol: f32,
) {
    assert_close(&format!("{label}.r"), actual.r, expected.r, tol);
    assert_close(&format!("{label}.g"), actual.g, expected.g, tol);
    assert_close(&format!("{label}.b"), actual.b, expected.b, tol);
    assert_close(&format!("{label}.a"), actual.a, expected.a, tol);
}
