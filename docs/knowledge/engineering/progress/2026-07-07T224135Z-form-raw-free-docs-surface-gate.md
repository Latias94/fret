---
type: "Work Progress"
title: "Form raw-free docs surface gate"
description: "Work Progress for Form raw-free docs surface gate."
timestamp: 2026-07-07T22:41:35Z
tags: ["fret", "shadcn", "form", "ui-gallery", "public-surface", "raw-surface", "documentation-ordering"]
---

# Summary

Closed a Form public-surface follow-up slice by removing `shadcn::raw::typography::*` from the
copyable Form docs snippets and adding a dedicated UI Gallery docs-surface gate.

# Details

- Added `apps/fret-ui-gallery/tests/form_docs_surface.rs` to lock Form page ordering, curated
  Form/Field snippet surfaces, and diagnostics anchors for docs smoke, submit validation, and
  disabled-field action state.
- Replaced Form `notes.rs` raw typography calls with `shadcn::FormDescription::new(...)`.
- Replaced Form `upstream_demo.rs` raw typography calls with curated `FieldDescription` and
  `FieldTitle` usage for mobile settings, sidebar, and email notification copy.
- Updated the Form page intro and tracker wording to name the actual `Form Demo` section and the
  copyable `Usage` / submit-validation follow-ups.
- Updated `docs/audits/shadcn-form.md` and `docs/shadcn-declarative-progress.md` so the Pass claim
  cites the new docs-order/raw-free gate.

# Next Action

- Commit and push this Form slice if the final status check remains clean.
- Continue scanning Pass rows that still lack dedicated docs-surface gates or expose raw/advanced
  APIs in default copyable snippets.

# Citations

- `apps/fret-ui-gallery/tests/form_docs_surface.rs`
- `apps/fret-ui-gallery/src/ui/snippets/form/notes.rs`
- `apps/fret-ui-gallery/src/ui/snippets/form/upstream_demo.rs`
- `apps/fret-ui-gallery/src/ui/pages/form.rs`
- `docs/audits/shadcn-form.md`
- `docs/shadcn-declarative-progress.md`

# Verification

- `cargo nextest run -p fret-ui-gallery --test form_docs_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app form --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `python3 -m json.tool tools/diag-scripts/ui-gallery/form/ui-gallery-form-docs-smoke.json`
- `python3 -m json.tool tools/diag-scripts/ui-gallery/form/ui-gallery-form-submit-validation-semantics.json`
- `python3 -m json.tool tools/diag-scripts/ui-gallery/form/ui-gallery-form-disabled-field-action-state.json`
- `git diff --check`
