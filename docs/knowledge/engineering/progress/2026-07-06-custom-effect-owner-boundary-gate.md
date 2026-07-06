---
type: Work Progress
title: Custom effect owner-boundary gate
timestamp: 2026-07-06T00:00:00Z
git_branch: chore/custom-effect-owner-policy
tags: fret,ui-framework,public-surface,custom-effects,source-policy,raw-model
---

# Summary

The custom-effect v2 web examples now have an owner-boundary regression gate in
`tools/check_surface_policy.py`.

The gate applies only to surfaces owned by `examples-custom-effect-v2-web`. It keeps the current
advanced/manual quarantine honest by rejecting direct reset/toggle writes through
`models_mut().update(...)` or UFCS `ModelStore::update(...)` outside the shared private
`custom_effect_v2_web_owner.rs` owner helper.

# Decision

Do not publish a custom-effect parameter/control binding yet. The web effect demos still need raw
setup-time parameter allocation and low-level effect-layer composition. The clean boundary for the
current architecture is narrower: setup may allocate raw models in the manual driver path, but
runtime reset/toggle writes must pass through the shared private owner helper until a real public
binding can own that model graph. Variant-specific defaults stay in each demo through
`CustomEffectV2WebControlReset`.

# Evidence

- Red proof before implementation:
  `python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_custom_effect_v2_web_direct_model_updates_are_rejected`
  failed with zero owner-boundary violations.
- Positive/negative fixture coverage:
  `test_custom_effect_v2_web_direct_model_updates_are_rejected` and
  `test_custom_effect_v2_web_owner_helper_updates_are_allowed`.

# Next

The next architectural step is a public custom-effect parameter/control binding only after the four
web variants prove which duplicated parameter shapes are stable. Until then, keep the raw
`ModelStore` seam quarantined and source-gated.
