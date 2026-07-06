---
type: Work Progress
title: Custom Effect V2 named parameter slots
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/custom-effect-param-spec
tags: fret,custom-effect,examples,effect-params,public-surface
---

# Summary

The four Custom Effect V2 web demos now build `EffectParamsV1` through named demo-local parameter
slots instead of anonymous `vec4s` literals at each render site.

# Changes

- Added `CustomEffectV2ParamSlot` and `CustomEffectV2ParamPack` to the shared Custom Effect V2 web
  helper module.
- Kept each demo's shader-specific parameter names next to its WGSL parameter comments.
- Replaced raw `EffectParamsV1 { vec4s: ... }` construction in the web, identity, LUT, and
  glass-chrome variants with `CustomEffectV2ParamPack::new().with_value(...).with_flag(...).finish()`.
- Updated the source-surface regression test to require named slots and reject direct anonymous
  parameter packing in those web demos.

# Decision

This slice intentionally stops short of a full dynamic parameter schema or UI generation system.
The current demos still have shader-specific scalar controls, but the brittle ABI ordering is now
explicitly named and source-gated. A future generic effect-parameter spec should be introduced only
when it can describe runtime UI controls, shader ABI packing, diagnostics labels, and defaults as
one contract.

# Verification

- `cargo nextest run -p fret-examples custom_effect_v2_web_owner --no-fail-fast`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `cargo check -p fret-examples --target wasm32-unknown-unknown`
- `python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`

# Follow-Up

The remaining custom-effect raw model pressure is variant-specific control state. Keep it in the
demo variants until a typed control/schema surface can replace both the runtime models and the
parameter ABI in one design.
