---
type: Work Progress
title: Advanced text facade migrated marquee perf demo
tags:
  - fret
  - authoring-surface
  - text
  - examples
timestamp: 2026-07-07T13:18:04Z
---

# Summary

Added `fret::advanced::text` as the explicit text-role facade for manual `KernelApp` or custom-host
render lanes, mirroring the existing default `fret::app::text` wrappers without adding the module to
`advanced::prelude::*`.

# Changed Files

- `ecosystem/fret/src/lib.rs`: added `advanced::text` wrappers for `control_readout`,
  `compact_paragraph`, `section_chrome_label`, `chrome_glyph`, `code_label`, and `code_block`, plus
  authoring-surface tests.
- `docs/crate-usage-guide.md`: documented `fret::advanced::text` for manual/custom-host helper
  text roles.
- `apps/fret-examples/src/extras_marquee_perf_demo.rs`: replaced direct
  `fret_ui_kit::declarative::text` import with `fret::advanced::text`.
- `apps/fret-examples/tests/extras_marquee_perf_demo_surface.rs`: locked the new import and
  asserted `decl_text` stays out of the example.

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret advanced_text_facade_keeps_manual_text_off_raw_kit_imports --no-fail-fast`
- `cargo nextest run -p fret-examples extras_marquee_perf_demo_keeps_title_on_chrome_role --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Action

Continue migrating the remaining direct `decl_text` example sources to `fret::advanced::text` where
they are manual `KernelApp` / custom-host examples:

- `apps/fret-examples/src/echarts_demo.rs`
- `apps/fret-examples/src/genui_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo/proof_helpers.rs`
- `apps/fret-examples/src/launcher_utility_window_demo.rs`
- `apps/fret-examples/src/launcher_utility_window_materials_demo.rs`
- `apps/fret-examples/src/window_hit_test_probe_demo.rs`
