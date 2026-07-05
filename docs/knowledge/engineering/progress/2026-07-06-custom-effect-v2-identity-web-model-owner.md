---
type: Work Progress
title: Custom Effect V2 identity web model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-custom-effect-identity
tags: fret,ui-framework,public-surface,custom-effect,raw-model,function-driver
---

# Summary

`custom_effect_v2_identity_web_demo` now routes reset/toggle model writes through a local
`CustomEffectV2IdentityWebModelOwner`, matching the baseline custom-effect V2 web owner boundary
without introducing a shared abstraction for the still-divergent parameter sets.

# Decisions

- Keep retained `Model<T>` handles and `models_mut().insert(...)` allocation in the function-driver
  setup path. This demo remains an advanced WebGPU/function-driver reference surface.
- Do not extract a shared custom-effect owner yet. Identity, LUT, and glass/chrome differ enough
  that a shared helper would hide parameter defaults before the repeated shape is proven.
- Strengthen Rust source tests and the grouped-state source policy so owner-reset coverage is
  field-specific and owner-outside raw `ModelStore` write markers, including aliased, turbofish,
  `update_any`, and UFCS forms, are rejected.

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface custom_effect_v2_web_model_writes_stay_behind_owner_helper custom_effect_v2_identity_web_model_writes_stay_behind_owner_helper custom_effect_v2_web_templates_use_shared_text_roles --no-fail-fast`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- Targeted `examples_source_tree_policy.grouped_state` check for `custom_effect_v2_web_demo.rs`
  and `custom_effect_v2_identity_web_demo.rs`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Apply the owner-boundary pattern to `custom_effect_v2_lut_web_demo.rs`.
- Apply the owner-boundary pattern to `custom_effect_v2_glass_chrome_web_demo.rs`.
- Revisit a shared custom-effect parameter binding after all V2 web variants expose the same owner
  boundary and the remaining duplication is measurable.
