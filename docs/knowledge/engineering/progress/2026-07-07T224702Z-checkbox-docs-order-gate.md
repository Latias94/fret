---
type: "Work Progress"
title: "Checkbox docs order gate"
description: "Work Progress for Checkbox docs order gate."
timestamp: 2026-07-07T22:47:02Z
tags: ["fret", "shadcn", "checkbox", "ui-gallery", "public-surface", "documentation-ordering"]
---

# Summary

- Added a dedicated UI Gallery Checkbox docs-surface gate for the existing public-surface cleanup
  lane.
- Scope is intentionally a gate/documentation slice: no component implementation changes.

# Details

- `apps/fret-ui-gallery/tests/checkbox_docs_surface.rs` locks the Checkbox page render order through
  `API Reference` before `Label Association (Fret)` / `With Title (Fret)`.
- The same gate keeps copyable snippets on curated `Checkbox`, `Field`, `FieldLabel`,
  `FieldDescription`, `FieldLegend`, and `FieldTitle` surfaces, with no `shadcn::raw::*`,
  `advanced::*`, generic `Checkbox::children(...)`, or checkbox parts promotion on the default docs
  lane.
- The gate anchors existing disabled, required-disabled, table mixed-state, label-click, and RTL
  diagnostic scripts so the tracker claim has one focused regression target.
- Updated `docs/audits/shadcn-checkbox.md` and the Checkbox row in
  `docs/shadcn-declarative-progress.md` to cite the new gate.

# Verification

- Passed: `cargo nextest run -p fret-ui-gallery --test checkbox_docs_surface`.
- Passed:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app checkbox --no-fail-fast`.
- Passed: `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`.
- Passed: `python3 tools/check_layering.py`.
- Passed: `python3 tools/report_largest_files.py --top 30 --min-lines 800` (report only; no failure).
- Passed: `cargo fmt --all --check`.
- Passed: JSON parse for the cited Checkbox diag scripts and `ui-gallery-checkbox-semantics`
  suite.
- Passed: `git diff --check`.
- Passed with existing warnings only: engineering wiki memory validation for
  `docs/knowledge/engineering`.

# Next Action

- Commit and push this Checkbox docs-order gate slice.
- Remaining similar candidates from the tracker scan include `input_group` and `radio_group`.

# Citations

- `apps/fret-ui-gallery/tests/checkbox_docs_surface.rs`
- `apps/fret-ui-gallery/src/ui/pages/checkbox.rs`
- `apps/fret-ui-gallery/src/ui/snippets/checkbox/`
- `tools/diag-scripts/suites/ui-gallery-checkbox-semantics/suite.json`
- `docs/audits/shadcn-checkbox.md`
- `docs/shadcn-declarative-progress.md`
