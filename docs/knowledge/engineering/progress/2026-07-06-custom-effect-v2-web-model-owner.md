---
type: Work Progress
title: Custom Effect V2 web model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-custom-effect-state
tags: fret,ui-framework,public-surface,custom-effect,raw-model,function-driver
---

# Summary

`custom_effect_v2_web_demo` now routes its raw model writes through a local
`CustomEffectV2WebModelOwner`. The demo remains an advanced function-driver/WebGPU reference
surface, so its retained `Model<T>` graph and `models_mut().insert(...)` allocation stay in the
driver-owned setup path.

# Decisions

- Do not migrate this demo to `LocalState<T>` in this slice. It is not a first-contact app view; it
  validates custom-effect renderer ABI, GPU input images, diagnostics, and function-driver hooks.
- Keep model allocation in `build_ui(...)`, which owns the window state graph.
- Move reset/toggle writes behind an operation-specific owner instead of leaving direct
  `models_mut().update(...)` calls in keyboard and activation handlers.
- Add a source gate that permits the owner boundary while forbidding scattered raw update calls.

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface custom_effect_v2_web_model_writes_stay_behind_owner_helper custom_effect_v2_web_overlay_readouts_use_shared_roles --no-fail-fast`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --test app_import_surface examples_src_keeps_local_state_raw_bridges_out app_state_demos_use_app_local_state_imports --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- Targeted `examples_source_tree_policy.grouped_state` check for `custom_effect_v2_web_demo.rs`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Apply the same owner-boundary pattern to the other `custom_effect_v2_*_web_demo.rs` variants if
  they keep raw function-driver models.
- Consider a shared custom-effect parameter binding only after two or more demos need the same
  app-facing abstraction; avoid widening `fret::app` for one advanced WebGPU reference surface.
