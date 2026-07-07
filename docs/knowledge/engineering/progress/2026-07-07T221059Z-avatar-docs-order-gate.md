---
type: "Work Progress"
title: "Avatar docs order gate"
description: "Work Progress for Avatar docs order gate."
timestamp: 2026-07-07T22:10:59Z
tags: ["fret", "shadcn", "avatar", "ui-gallery", "public-surface", "documentation-ordering"]
---

# Summary

Closed the Avatar UI Gallery documentation-ordering gap with a focused docs-surface gate.

The Avatar row was already `Pass`, and the page/audit already said the gallery mirrors the upstream
Avatar docs path before the Fret-only fallback check. This slice makes that claim executable so a
future edit cannot move `Fallback only (Fret)` or `Notes` ahead of `API Reference`, lose the
copyable usage/dropdown snippets, or retarget the dropdown diagnostics at the presentational nested
Avatar instead of the authored Button trigger.

# Details

- Added `apps/fret-ui-gallery/tests/avatar_docs_surface.rs`.
- Locked the page order:
  `Demo -> Usage -> Basic -> Badge -> Badge with Icon -> Avatar Group -> Avatar Group Count ->
  Avatar Group with Icon -> Sizes -> Dropdown -> RTL -> API Reference -> Fallback only (Fret) ->
  Notes`.
- Locked the copyable usage snippet on `fret_ui_shadcn::facade as shadcn` plus
  `Avatar::empty().children([...])`.
- Locked dropdown trigger ownership so the authored Button remains the semantic trigger and the
  nested Avatar remains presentational content.
- Linked the new gate from `docs/audits/shadcn-avatar.md` and
  `docs/shadcn-declarative-progress.md`.

# Next Action

Continue applying this focused docs-order gate pattern to any remaining `Pass` shadcn rows whose
tracker/audit claims are still only covered by broad `ui_authoring_surface_default_app` tests.

# Citations

- `apps/fret-ui-gallery/tests/avatar_docs_surface.rs`
- `apps/fret-ui-gallery/src/ui/pages/avatar.rs`
- `docs/audits/shadcn-avatar.md`
- `docs/shadcn-declarative-progress.md`
- Verification:
  - `cargo nextest run -p fret-ui-gallery --test avatar_docs_surface`
  - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app avatar --no-fail-fast`
  - `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
  - `python3 tools/check_layering.py`
  - `cargo fmt --all --check`
