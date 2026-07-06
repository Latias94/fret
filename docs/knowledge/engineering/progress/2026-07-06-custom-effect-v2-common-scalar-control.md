---
type: "Work Progress"
title: "Custom Effect V2 common scalar control binding"
description: "Work Progress for Custom Effect V2 common scalar control binding."
timestamp: 2026-07-06T17:16:06Z
tags: ["fret", "custom-effect", "examples", "binding", "public-surface", "raw-model"]
git_branch: "refactor/custom-effect-common-scalar-control"
verified_by: "cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast"
---

# Summary

The shared Custom Effect V2 web `uv_span` control now uses `CustomEffectV2ScalarControl` instead
of exposing `Model<Vec<f32>>` through `CustomEffectV2WebControlBinding`.

# Details

- Moved `CustomEffectV2WebCommonControls::uv_span` from a raw slider model to the shared scalar
  control wrapper.
- Added a private owner-backed reset path for `CustomEffectV2ScalarControl` so common reset can use
  the same typed scalar default as variant controls.
- Updated the web, identity, LUT, and glass-chrome demos to subscribe through
  `binding.uv_span().values()` and read through `binding.uv_span().clamped_value(...)`.
- Kept `shadcn::Slider::new(...)` compatibility through the existing `IntoFloatVecModel`
  implementation rather than exposing the underlying runtime model.
- Strengthened `custom_effect_overlay_text_surface` structure tests so the common binding cannot
  regress to `uv_span: Model<Vec<f32>>` or raw owner `set_model(... vec![...])` writes.
- Added an owner-local behavior test that mutates common `uv_span`, calls `reset_controls_in(...)`,
  and verifies the scalar-control model returns to its configured default.

# Decision

Use the existing `CustomEffectV2ScalarControl` as the immediate common numeric-control contract.
This keeps the app-facing demo API named and typed without prematurely introducing a full shader
parameter schema. A fuller schema should wait until it can own ranges, UI labels, shader ABI slots,
diagnostic names, and docs together.

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `cargo nextest run -p fret-examples custom_effect_v2_web_owner --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `cargo check -p fret-examples --target wasm32-unknown-unknown`
- `python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

The scalar control later gained `CustomEffectV2ScalarSpec`; see
`2026-07-06-custom-effect-v2-scalar-specs.md`. After that slice, the next custom-effect step should
be a real schema design only if more demos need shared shader ABI/control metadata.

# Citations

- [custom_effect_v2_web_owner.rs](../../../../apps/fret-examples/src/custom_effect_v2_web_owner.rs)
- [custom_effect_overlay_text_surface.rs](../../../../apps/fret-examples/tests/custom_effect_overlay_text_surface.rs)
