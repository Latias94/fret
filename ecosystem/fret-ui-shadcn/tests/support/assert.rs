#![allow(dead_code)]

use fret_core::Px;

pub(super) fn assert_close_px(label: &str, actual: Px, expected: f32, tol: f32) {
    let delta = (actual.0 - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected ~{expected:.3}px, got {:.3}px (Δ={delta:.3}px, tol={tol:.3}px)",
        actual.0,
    );
}
