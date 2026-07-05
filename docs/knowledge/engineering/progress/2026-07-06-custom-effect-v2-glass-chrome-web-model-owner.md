---
type: Work Progress
title: Custom Effect V2 glass chrome web model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-custom-effect-glass
tags: fret,ui-framework,public-surface,custom-effect,raw-model,function-driver
---

# Summary

`custom_effect_v2_glass_chrome_web_demo` now routes reset/toggle model writes through a local
`CustomEffectV2GlassChromeWebModelOwner`, completing the owner-boundary cleanup across the current
Custom Effect V2 web demo family.

# Decisions

- Keep retained `Model<T>` handles and `models_mut().insert(...)` allocation in the function-driver
  setup path. The demo remains an advanced WebGPU/function-driver reference surface.
- Keep the owner helper demo-local. Glass/chrome uses a smaller parameter set than base V2 web and
  LUT, so a shared helper would still obscure the defaults more than it helps.
- Extend the shared source test and grouped-state policy table with glass/chrome-specific reset
  markers. The owner-slice policy remains fail-closed when configured owner markers drift.

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface custom_effect_v2_web_model_writes_stay_behind_owner_helper custom_effect_v2_identity_web_model_writes_stay_behind_owner_helper custom_effect_v2_lut_web_model_writes_stay_behind_owner_helper custom_effect_v2_glass_chrome_web_model_writes_stay_behind_owner_helper custom_effect_v2_web_templates_use_shared_text_roles --no-fail-fast`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- Targeted `examples_source_tree_policy.grouped_state` check for all four `custom_effect_v2_*web_demo.rs`
  owner slices
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Revisit a shared custom-effect parameter binding after the V2 web family has enough repeated
  shape to justify a public app-facing abstraction.
- Keep the advanced function-driver demos as low-level reference surfaces; do not migrate them to
  `LocalState<T>` unless the demo itself changes from renderer authoring to first-contact app UI.
