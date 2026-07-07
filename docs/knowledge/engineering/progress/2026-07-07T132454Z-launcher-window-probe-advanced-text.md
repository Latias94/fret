---
type: Work Progress
title: Launcher and window probe demos use advanced text facade
tags:
  - fret
  - authoring-surface
  - text
  - examples
timestamp: 2026-07-07T13:24:54Z
---

# Summary

Migrated three manual `KernelApp` / compat-driver examples from direct
`fret_ui_kit::declarative::text` imports to the explicit `fret::advanced::text` facade added in
commit `a55e731e34`.

# Changed Files

- `apps/fret-examples/src/launcher_utility_window_demo.rs`
- `apps/fret-examples/src/launcher_utility_window_materials_demo.rs`
- `apps/fret-examples/src/window_hit_test_probe_demo.rs`
- `apps/fret-examples/tests/launcher_utility_window_demo_surface.rs`
- `apps/fret-examples/tests/launcher_utility_window_materials_demo_surface.rs`
- `apps/fret-examples/tests/window_hit_test_probe_demo_surface.rs`

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples keeps_fixed_text_on_roles --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Remaining Direct Text Imports

- `apps/fret-examples/src/echarts_demo.rs`
- `apps/fret-examples/src/genui_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo/proof_helpers.rs`
