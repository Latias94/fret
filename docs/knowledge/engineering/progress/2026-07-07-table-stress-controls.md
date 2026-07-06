---
type: "Work Progress"
title: "Table stress controls binding"
description: "Work Progress for bundling retained table-stress state behind a local controls binding."
timestamp: 2026-07-07T04:20:00Z
tags: ["fret", "examples", "table", "stress", "public-surface", "raw-model"]
git_branch: "refactor/table-stress-controls"
verified_by: "cargo nextest run -p fret-examples --test table_stress_demo_surface table_stress_demo_model_state_stays_behind_controls_binding --no-fail-fast"
---

# Summary

`table_stress_demo.rs` now stores retained table stress state behind `TableStressControls` instead
of exposing separate `Model<TableState>` and `Model<u64>` fields on `TableStressWindowState`.

# Details

- Added `TableStressControls` to own table state and item revision models.
- Moved startup model allocation into `TableStressControls::new(...)`.
- Routed keyboard commands through semantic controls methods.
- Added `render_snapshot(...)` so render reads selected/sort/filter/revision state through one
  local snapshot API.
- Kept `table_model()` as the narrow retained table component seam for
  `table_virtualized(...)`.
- Strengthened source-shape tests so production code cannot regress to window-level raw table
  model fields or driver helpers that pass model handles around.

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test table_stress_demo_surface table_stress_demo_keeps_fixed_table_text_on_roles table_stress_demo_model_state_stays_behind_controls_binding --no-fail-fast`
- `cargo nextest run -p fret-examples table_stress_controls_preserve_command_state_transitions --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Continue applying this bundle-first pattern to stress or harness demos that still expose multiple
runtime model handles on their app/window state. Avoid converting retained perf harness state to
`LocalState` unless the component contract already supports that authoring surface.

# Citations

- [table_stress_demo.rs](../../../../apps/fret-examples/src/table_stress_demo.rs)
- [table_stress_demo_surface.rs](../../../../apps/fret-examples/tests/table_stress_demo_surface.rs)
