---
type: Work Progress
title: Examples docking arbitration owner helper
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/examples-docking-arbitration-owner
tags: fret,ui-framework,public-surface,examples,docking,raw-model,owner
---

# Summary

`apps/fret-examples/src/docking_arbitration_demo.rs` now routes its diagnostic/control model writes
through a demo-local `DockingArbitrationModelOwner`.

The demo remains an advanced docking arbitration harness. It still intentionally exercises docking
runtime state, multiple windows/viewports, overlay arbitration, synthetic pointer diagnostics, and
drop-mask policy hooks. This slice only removes the copyable pattern of writing diagnostic models
directly from UI/event handlers.

# Decision

Keep the harness classification. Do not try to hide docking arbitration internals behind ordinary
app `LocalState`; this example is a maintainer-facing proof surface for low-level docking behavior.
The correct cleanup is a named local owner boundary for the shared diagnostic models.

`DockingArbitrationModelOwner` now owns:

- `toggle_drop_mask_disallow_left_edge(...)`
- `set_synth_pointer_debug(...)`
- `set_last_viewport_input(...)`

# Evidence

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test docking_arbitration_surface docking_arbitration_demo_model_writes_stay_behind_owner_helper --no-fail-fast`
  failed because `DockingArbitrationModelOwner` did not exist.
- The source gate now requires the owner methods and forbids direct, generic, `update_any`, and
  UFCS `ModelStore` bypasses in the production demo source.
- `docking_arbitration_model_owner_preserves_diagnostic_state_transitions` covers drop-mask toggle,
  viewport-input text update, and synthetic-pointer debug text update behavior.
- `tools/check_surface_policy.py` now lists `ModelStore` as an explicit allowed raw seam for this
  internal harness only; the gate continues to reject unclassified raw seams elsewhere.

# Next

Do not promote this helper into `fret::app`. If more docking demos need the same shared diagnostic
model mutation pattern, first design a docking diagnostics binding/owner contract in
`ecosystem/fret-docking` or a diagnostics harness layer.
