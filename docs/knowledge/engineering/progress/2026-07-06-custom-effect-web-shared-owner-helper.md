---
type: Work Progress
title: Custom effect web shared owner helper
timestamp: 2026-07-06T00:00:00Z
git_branch: chore/custom-effect-owner-policy
tags: fret,ui-framework,public-surface,custom-effects,source-policy,raw-model
---

# Summary

The four custom-effect v2 web examples now share one private owner helper:
`apps/fret-examples/src/custom_effect_v2_web_owner.rs`.

This deletes the repeated per-demo `ModelStore` alias and owner struct while keeping each demo's
variant-specific reset defaults in a local `CustomEffectV2WebControlReset` implementation.

# Decision

The helper stays inside `apps/fret-examples`, is compiled only for wasm through the examples crate
root, and uses `pub(crate)` visibility. It is not a public framework or ecosystem API. The current
contract is still an advanced/manual demo quarantine: raw model allocation remains in driver setup,
while reset/toggle writes route through the shared private owner.

# Evidence

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface ...` failed because
  `custom_effect_v2_web_owner.rs` did not exist.
- `tools/check_surface_policy.py` classifies the helper as an internal harness surface and keeps the
  four demos under the custom-effect advanced/manual owner-boundary check.
- `tools/examples_source_tree_policy/grouped_state.py` now checks the shared helper import and
  demo-local reset trait impls instead of the old per-demo owner structs.

# Next

Do not promote this helper into `fret-ui-kit` yet. A public custom-effect parameter/control binding
should wait until the web variants prove a stable parameter graph that is useful outside these
manual WebGPU demos.
