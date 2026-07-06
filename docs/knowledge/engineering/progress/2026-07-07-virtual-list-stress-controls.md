---
type: "Work Progress"
title: "Virtual list stress controls binding"
description: "Work Progress for the virtual-list stress controls binding cleanup."
timestamp: 2026-07-07T03:05:00Z
tags: ["fret", "examples", "virtual-list", "stress", "public-surface", "raw-model"]
git_branch: "refactor/virtual-list-stress-controls"
verified_by: "cargo nextest run -p fret-examples --test virtual_list_stress_demo_surface virtual_list_stress_demo_model_state_stays_behind_controls_binding --no-fail-fast"
---

# Summary

`virtual_list_stress_demo.rs` now stores its stress controls behind
`VirtualListStressControls` instead of exposing separate `Model<bool>` / `Model<u64>` fields on
`VirtualListStressWindowState`.

# Details

- Replaced `VirtualListStressModelOwner` with `VirtualListStressControls`.
- Moved startup model allocation behind `VirtualListStressControls::new(...)`.
- Routed Space/R key writes through semantic controls methods.
- Added `layout_snapshot(...)` so render can observe the three stress-control values without
  hand-wiring model handles in the render closure.
- Strengthened the source-shape test so production code must keep the controls binding and cannot
  regress to the old owner or raw state fields.

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test virtual_list_stress_demo_surface virtual_list_stress_demo_keeps_fixed_row_text_on_roles virtual_list_stress_demo_model_state_stays_behind_controls_binding --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Apply the same bundle-first cleanup to other examples that still expose several runtime model
handles on a public window/view state even though the models form one local control surface.

# Citations

- [virtual_list_stress_demo.rs](../../../../apps/fret-examples/src/virtual_list_stress_demo.rs)
- [virtual_list_stress_demo_surface.rs](../../../../apps/fret-examples/tests/virtual_list_stress_demo_surface.rs)
