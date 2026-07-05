---
type: Work Progress
title: Custom Effect V2 LUT web model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-custom-effect-lut
tags: fret,ui-framework,public-surface,custom-effect,raw-model,function-driver
---

# Summary

`custom_effect_v2_lut_web_demo` now routes reset/toggle model writes through a local
`CustomEffectV2LutWebModelOwner`, extending the V2 web owner-boundary pattern to the LUT reference
surface while preserving its LUT-specific parameter defaults.

# Decisions

- Keep retained `Model<T>` handles and `models_mut().insert(...)` allocation in the function-driver
  setup path. The demo is still an advanced WebGPU/function-driver reference surface.
- Keep the owner helper demo-local. LUT defaults differ from identity and base V2 web, so a shared
  helper would still be premature.
- Extend the shared source test and grouped-state policy table with LUT-specific reset markers,
  including owner-outside raw `ModelStore` write marker rejection. The grouped-state owner-slice
  policy now fails closed when configured owner markers drift.

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface custom_effect_v2_web_model_writes_stay_behind_owner_helper custom_effect_v2_identity_web_model_writes_stay_behind_owner_helper custom_effect_v2_lut_web_model_writes_stay_behind_owner_helper custom_effect_v2_web_templates_use_shared_text_roles --no-fail-fast`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- Targeted `examples_source_tree_policy.grouped_state` check for `custom_effect_v2_web_demo.rs`,
  `custom_effect_v2_identity_web_demo.rs`, and `custom_effect_v2_lut_web_demo.rs`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Apply the owner-boundary pattern to `custom_effect_v2_glass_chrome_web_demo.rs`.
- Revisit a shared custom-effect parameter binding after all V2 web variants expose the same owner
  boundary and the remaining duplication is measurable.
