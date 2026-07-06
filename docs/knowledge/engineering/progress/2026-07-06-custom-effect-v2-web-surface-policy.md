---
type: Work Progress
title: Custom effect v2 web surface policy
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/raw-surface-contracts
tags: fret,ui-framework,public-surface,custom-effects,source-policy,raw-model
---

# Summary

The custom-effect v2 web variants are now covered by the unified public example source-policy gate:

- `custom_effect_v2_web_demo.rs`;
- `custom_effect_v2_identity_web_demo.rs`;
- `custom_effect_v2_lut_web_demo.rs`;
- `custom_effect_v2_glass_chrome_web_demo.rs`.

Each file is classified as an advanced/manual examples surface with an explicit owner, raw seam
allowlist, and retirement condition.

Follow-up on the same day: the unified source-policy gate now also enforces the shared private
owner boundary. Direct reset/toggle writes through `models_mut().update(...)` or UFCS
`ModelStore::update(...)` are rejected outside `custom_effect_v2_web_owner.rs`, while each demo
keeps its variant-specific reset defaults in a local `CustomEffectV2WebControlReset` impl.
Setup-time `models_mut().insert(...)` remains allowed until a public custom-effect
parameter/control binding exists.

# Decision

Do not mechanically migrate these demos to `LocalState` yet. They are manual runner/effect-layer
proofs with raw parameter models and a shared private `ModelStore` owner. The correct cleanup target
is a public custom-effect parameter/control binding that owns parameter models, reset/toggle
actions, and effect-layer composition without exposing the raw owner seam.

# Verification

- `python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_custom_effect_v2_web_direct_model_updates_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_custom_effect_v2_web_owner_helper_updates_are_allowed`
- `python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `python3 -m unittest tools.test_check_surface_policy`
- `python3 tools/check_surface_policy.py`

# Next

`tools/gate_examples_source_tree_policy.py` still has an existing failing baseline and should be
treated as a broader drift report until it is repaired. The owner-helper regression now lives in
`tools/check_surface_policy.py`, including the UFCS-style `ModelStore::update(...)` case.
