---
type: Work Progress
title: Custom Effect V2 scalar control bindings
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/custom-effect-control-schema
tags: fret,custom-effect,examples,binding,public-surface,raw-model
---

# Summary

The four Custom Effect V2 web demos now keep variant-specific slider state behind
`CustomEffectV2ScalarControl` instead of exposing raw `Model<Vec<f32>>` fields in the demo structs.

# Changes

- Added `CustomEffectV2ScalarControl` to the shared Custom Effect V2 web helper.
- Implemented `IntoFloatVecModel` for the scalar control so it can be passed directly to
  `shadcn::Slider::new(...)` without exposing a raw slider model at the call site.
- Moved scalar defaults, reset writes, clamp fallback, and integer rounding through the binding.
- Migrated the web, identity, LUT, and glass-chrome variants away from direct
  `app.models_mut().insert(vec![...])` scalar allocation and `reset.set_model(&self.foo, ...)`
  reset calls.
- Tightened the source-policy gates to require scalar controls and reject raw scalar model fields
  in those demo files.

# Decision

This is still not a full effect-parameter schema. It is a narrow app-facing binding over the
existing shadcn slider model contract. That is the right intermediate layer: demos stop teaching raw
runtime model plumbing, while shader-specific control names and ranges remain local until a real
effect schema can own UI controls, shader ABI, defaults, diagnostics labels, and docs together.

# Verification

- `cargo nextest run -p fret-examples custom_effect_v2_web_owner --no-fail-fast`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Follow-Up

The shared common `uv_span()` control was later moved behind `CustomEffectV2ScalarControl`; see
`2026-07-06-custom-effect-v2-common-scalar-control.md`. The remaining design question is whether a
future typed effect schema should own shader ABI slots, ranges, labels, defaults, and diagnostics
across more than these four demos.
