---
type: Work Progress
title: Fret examples raw decl_text cleanout
tags:
  - fret
  - authoring-surface
  - text
  - examples
timestamp: 2026-07-07T13:33:33Z
---

# Summary

Finished the direct `fret_ui_kit::declarative::text as decl_text` cleanup in
`apps/fret-examples/src`. The remaining examples now use explicit `fret::advanced::text` wrappers
for generic/manual render lanes.

# Changed Files

- `apps/fret-examples/src/echarts_demo.rs`
- `apps/fret-examples/src/genui_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo/proof_helpers.rs`
- `apps/fret-examples/tests/echarts_demo_surface.rs`
- `apps/fret-examples/tests/genui_demo_surface.rs`
- `apps/fret-examples/tests/imui_editor_proof_text_roles_surface.rs`

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples echarts_demo_chart_titles_use_section_chrome_role --no-fail-fast`
- `cargo nextest run -p fret-examples genui_demo_keeps_tool_text_on_roles --no-fail-fast`
- `cargo nextest run -p fret-examples imui_editor_proof_main_fixed_text_uses_shared_roles --no-fail-fast`
- `cargo nextest run -p fret-examples genui_demo_uses_explicit_public_surfaces --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`
- `rg --files apps/fret-examples/src | xargs rg -l "use fret_ui_kit::declarative::text as decl_text|decl_text::" | sort` returned no files.
