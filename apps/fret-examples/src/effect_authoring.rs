//! Small authoring helpers for effect demos.
//!
//! These helpers are demo/policy-level conveniences (not renderer contracts). Their main goal is
//! to make it harder to accidentally under-estimate `max_sample_offset_px` and introduce
//! scissor/padding artifacts when sources degrade (e.g. `src_raw` aliases to `src`).

use fret_core::geometry::Px;

pub(crate) fn custom_effect_warp_max_sample_offset_px(
    strength_px: f32,
    chromatic_aberration_px: f32,
) -> Px {
    let strength_px = if strength_px.is_finite() {
        strength_px.max(0.0)
    } else {
        0.0
    };
    let chromatic_aberration_px = if chromatic_aberration_px.is_finite() {
        chromatic_aberration_px.max(0.0)
    } else {
        0.0
    };

    // Keep a small safety pad for bilinear sampling and authoring drift.
    Px((strength_px + chromatic_aberration_px + 8.0).min(256.0))
}

pub(crate) fn custom_effect_v3_lens_max_sample_offset_px(
    refraction_amount_px: f32,
    dispersion: f32,
) -> Px {
    let refraction_amount_px = if refraction_amount_px.is_finite() {
        refraction_amount_px.max(0.0)
    } else {
        0.0
    };
    let dispersion = if dispersion.is_finite() {
        dispersion.clamp(0.0, 1.0)
    } else {
        0.0
    };

    // The lens shader samples `pos_px + refract ± disp`, where:
    // - `|refract| <= refraction_amount_px`
    // - `disp = refract * disp_k`, and conservatively `disp_k <= dispersion`
    //
    // So an upper bound on the maximum read radius is:
    // `refraction_amount_px * (1 + dispersion)`.
    //
    // Keep a small safety pad for bilinear sampling and future authoring tweaks.
    Px((refraction_amount_px * (1.0 + dispersion) + 2.0).min(256.0))
}
