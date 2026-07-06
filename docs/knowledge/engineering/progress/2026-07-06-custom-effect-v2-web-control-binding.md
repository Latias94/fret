---
type: Work Progress
title: Custom Effect V2 web common-control binding
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/custom-effect-parameter-binding
tags: fret,custom-effect,examples,binding,public-surface
---

# Summary

The four Custom Effect V2 web demos now share a private app-facing control binding for duplicated
common controls instead of exposing the private model owner at each demo call site.

# Changes

- Added `CustomEffectV2WebControlBinding` in `custom_effect_v2_web_owner.rs`.
- Moved the common show/enabled/mode/quality/sampling/UV/debug models into the shared binding.
- Routed toggle/reset through `toggle_surface_in(...)` and `reset_controls_in(...)`.
- Kept variant-specific parameter models in each demo behind `CustomEffectV2WebVariantReset`.
- Updated source-policy gates and focused source tests from the old owner-boundary contract to the
  new binding-boundary contract.

# Verification

- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo check -p fret-examples --lib --tests`
- `cargo check -p fret-examples --target wasm32-unknown-unknown`

# Follow-up

This slice intentionally does not introduce a full `EffectParamsV1` parameter-spec abstraction.
The next custom-effect cleanup should decide whether named `ParamSlot` metadata is worth adding for
the remaining variant-specific scalar models and params packing.
