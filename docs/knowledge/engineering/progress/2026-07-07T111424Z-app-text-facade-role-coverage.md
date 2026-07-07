---
type: "Work Progress"
title: "App text facade role coverage"
description: "Work Progress for App text facade role coverage."
timestamp: 2026-07-07T11:14:24Z
tags: ["ui-surface", "facade", "text", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/app-text-facade-role-coverage"
---

# Summary

Extended the app-facing `fret::app::text` facade so default app helpers can use existing kit text
roles without importing raw `fret_ui_kit::declarative::text` or spelling `cx.elements()` directly.

# Details

Changed files:

- `ecosystem/fret/src/lib.rs`
- `docs/crate-usage-guide.md`

Decision:

- Add thin wrappers for existing kit roles: `control_label`, `table_cell`,
  `table_cell_emphasis`, `chrome_glyph`, and `code_block`.
- Keep the wrappers on the explicit `fret::app::text` module rather than widening
  `fret::app::prelude::*`.
- Update the app text facade source-policy test so these roles stay covered by
  `AppRenderContext<'a>` and delegate to the existing kit constructors.
- Update the crate usage guide to point extracted app helpers at `fret::app::text` for common
  text roles before falling back to raw kit text helpers.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret app_text_facade_keeps_first_contact_text_off_raw_element_context --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `git diff --check`

# Next Action

Merge this slice back to `main` and push remote `main`, then migrate examples that were blocked on
`table_cell`, `control_label`, `code_block`, or `chrome_glyph` app text roles.

# Citations

- `ecosystem/fret/src/lib.rs`
- `docs/crate-usage-guide.md`
