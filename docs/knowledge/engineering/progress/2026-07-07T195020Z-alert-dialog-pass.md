---
type: Work Progress
title: Alert Dialog public surface is closed to Pass
timestamp: 2026-07-07T19:50:20Z
tags:
  - shadcn
  - alert-dialog
  - ui-gallery
  - public-surface
status: verified
---

# Summary

Closed the Alert Dialog public-surface pass by adding a focused overlay placement fixture and
updating the tracker/audit evidence to cite rerunnable component-specific gates.

# Truth

- `docs/shadcn-declarative-progress.md` now marks `alert-dialog` as `Pass`.
- `docs/audits/shadcn-alert-dialog.md` now cites the Gallery docs-surface gate, focused web-golden
  chrome gate, focused misc-overlay placement gate, Radix state/geometry gates, and matrix packet.
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement/misc_overlays/fixtures.rs` now has
  `web_vs_fret_misc_overlays_alert_dialog_cases_match_web_fixtures`, so alert-dialog placement
  evidence is not blocked by unrelated misc overlay fixture drift.
- No runtime component code changed in this slice.

# Verification

- `cargo nextest run -p fret-ui-gallery --test alert_dialog_docs_surface`
- `cargo nextest run -p fret-ui-shadcn --lib alert_dialog`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome alert_dialog::fixtures`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement misc_overlays::fixtures::web_vs_fret_misc_overlays_alert_dialog_cases_match_web_fixtures`
- `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry radix_web_alert_dialog_open_geometry_matches_fret`
- `cargo nextest run -p fret-ui-shadcn --test radix_web_primitives_state radix_web_alert_dialog_open_cancel_matches_fret`
- Alert Dialog matrix packet check: all validation gates are `pass`.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
  (passes with existing historical warnings)
- `git diff --check`

# Known Context

- The full `web_vs_fret_overlay_chrome` web-goldens suite currently fails a combobox
  active-descendant semantics case after the alert-dialog chrome case passes.
- The full `web_vs_fret_overlay_placement` web-goldens suite currently fails earlier unrelated
  calendar/dropdown/date-picker/context-menu cases before the alert-dialog misc overlay case runs.
  Use the focused alert-dialog placement gate above for this component's pass evidence.
