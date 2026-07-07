---
type: "Work Progress"
title: "Label docs order gate"
description: "Work Progress for Label docs order gate."
timestamp: 2026-07-07T22:18:08Z
tags: ["fret", "shadcn", "label", "ui-gallery", "public-surface", "documentation-ordering"]
---

# Summary

Closed the next public-surface follow-up slice for Label by converting the gallery docs-order claim
into an executable gate and removing a raw typography escape hatch from the copyable `Label in Field`
example.

# Details

- Added `apps/fret-ui-gallery/tests/label_docs_surface.rs` to lock Label page ordering:
  `Demo`, `Usage`, then explicit Fret follow-ups (`Label in Field`, `RTL`,
  `Composable Content`, `API Reference`).
- The same gate checks copyable association snippets, Label diagnostics anchors, and the narrow
  `Label::children(...)` follow-up.
- Updated `apps/fret-ui-gallery/src/ui/snippets/label/label_in_field.rs` so the explanatory line uses
  `shadcn::FieldDescription::new(...)` instead of `shadcn::raw::typography::muted(...)`.
- Updated `docs/audits/shadcn-label.md` and `docs/shadcn-declarative-progress.md` so the Pass claim
  cites the new docs-surface gate and records the raw-free Field follow-up.

# Next Action

- Run the Label-focused gate and repo boundary checks:
  `cargo nextest run -p fret-ui-gallery --test label_docs_surface`,
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app label --no-fail-fast`,
  `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`,
  `python3 tools/check_layering.py`, `cargo fmt --all --check`, wiki validation, and `git diff --check`.
- If green, commit and push to `origin/main` per the current user instruction.
- Continue the public-surface follow-up lane by scanning the next Pass row that lacks a dedicated
  `*_docs_surface.rs` gate.

# Citations

- `apps/fret-ui-gallery/tests/label_docs_surface.rs`
- `apps/fret-ui-gallery/src/ui/snippets/label/label_in_field.rs`
- `docs/audits/shadcn-label.md`
- `docs/shadcn-declarative-progress.md`
