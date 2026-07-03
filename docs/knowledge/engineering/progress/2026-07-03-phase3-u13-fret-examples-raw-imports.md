---
type: Work Progress
title: Phase 3 U13 fret-examples explicit raw imports
tags: fret,phase3,u13,fret-examples,advanced-facade,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 tenth slice restores `cargo check -p fret-examples --lib` after raw local-state helper
traits were removed from `fret::advanced::prelude::*`.

# Changes

- Advanced/manual `fret-examples` files now import the exact raw local-state helper traits they use
  from `fret::advanced::raw`.
- The raw traits were not added back to `advanced::prelude::*`; the break remains in force and raw
  usage is visible at each call site.
- The change is mechanical and does not migrate the examples off raw helper usage yet. It restores
  the package-level check so future migrations have a green baseline.

# Verification

Passed:

- `cargo check -p fret-examples --lib`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`

Note:

- `cargo check -p fret-examples --lib` still reports the pre-existing `fret-chart` dead-code
  warning for `visual_map_track_at`, but exits successfully.

# Next Action

Continue U13 by choosing one raw-helper-heavy example cluster and replacing raw model-store access
with app-facing action/data helpers where the file is intended to be copyable. Keep true renderer,
manual driver, and advanced interop proof files explicit under `fret::advanced::raw`.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Advanced raw and driver split](2026-07-03-phase3-u13-advanced-raw-driver-split.md)
- [`fret-examples` precise quarantine](2026-07-03-phase3-u13-fret-examples-precise-quarantine.md)
