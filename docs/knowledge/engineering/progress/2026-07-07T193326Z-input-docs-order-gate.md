---
type: Work Progress
title: Input docs order gate is locked with a remaining layout gap
timestamp: 2026-07-07T19:33:26Z
tags:
  - shadcn
  - input
  - ui-gallery
  - public-surface
status: verified_with_gap
---

# Summary

Closed the Input documentation-ordering risk by adding a focused UI Gallery docs-surface gate.
Input remains `In review` because current live layout evidence contradicts a safe `Pass` upgrade.

# Truth

- `apps/fret-ui-gallery/tests/input_docs_surface.rs` now locks the upstream Input docs order through
  `RTL` before Fret-only follow-ups.
- The same gate locks the docs screenshot script plus label-click-focus and deterministic file
  browse diagnostics.
- `docs/shadcn-declarative-progress.md` now cites the docs-order/runtime follow-up gate, runtime
  diagnostics, the input matrix packet, and the remaining live layout gap.
- `docs/audits/shadcn-input.md` now links the docs-order gate and matrix packet from its validation
  evidence.
- No runtime component code changed in this slice.

# Artifacts

- `apps/fret-ui-gallery/tests/input_docs_surface.rs`
- `docs/audits/shadcn-input.md`
- `docs/shadcn-declarative-progress.md`

# Verification

- `cargo nextest run -p fret-ui-gallery --test input_docs_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app input_`
- `cargo nextest run -p fret-ui-shadcn --lib input::tests`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome input`
- Input matrix packet check: status is `regression_locked`, validation gates pass, and repair,
  hardening, and gate queues are empty.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
  (passes with existing historical warnings)
- `git diff --check`

# Known Gap

- Do not upgrade Input to `Pass` yet: live `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout input`
  currently fails the aggregate `layout_input_fixtures::web_vs_fret_layout_input_geometry_matches_web_fixtures`
  case at `input-with-label input w` (`expected≈384`, got `0`).
- Broad `cargo nextest run -p fret-ui-shadcn --lib input` also matches related combobox/input-group
  tests; use the explicit `input::tests` filter for component-local recipe evidence.
