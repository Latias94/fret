---
type: Work Progress
title: Docking arbitration controls boundary enters surface policy
tags:
  - fret
  - docking
  - surface-policy
  - model-owner
timestamp: 2026-07-07T13:59:19Z
---

# Summary

Promoted the `docking_arbitration_demo` controls-boundary source test into the global surface
policy gate. The harness still owns manual driver, docking, viewport, and diagnostics seams, but
diagnostic model writes now have a repo-level guard that keeps them behind
`DockingArbitrationControls` and `DockingArbitrationControlsService`.

# Changed Files

- `tools/check_surface_policy.py`: names the docking-arbitration owner constant, keeps the existing
  internal-harness classification, and adds
  `internal_harness-docking-arbitration-controls-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage for direct `models_mut().update*`,
  UFCS `ModelStore::update*`, and `let models = app.models_mut(); models.update(...)` alias
  bypasses, plus an allowed controls-routed fixture.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples docking_arbitration_demo_model_writes_stay_behind_controls_binding --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
