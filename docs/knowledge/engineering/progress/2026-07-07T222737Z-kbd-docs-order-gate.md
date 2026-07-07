---
type: "Work Progress"
title: "Kbd docs order gate"
description: "Work Progress for Kbd docs order gate."
timestamp: 2026-07-07T22:27:37Z
tags: ["fret", "shadcn", "kbd", "ui-gallery", "public-surface", "documentation-ordering"]
---

# Summary

Closed the next public-surface follow-up slice for Kbd by turning the existing docs-order Pass claim
into a dedicated UI Gallery docs-surface gate.

# Details

- Added `apps/fret-ui-gallery/tests/kbd_docs_surface.rs` to lock the Kbd docs path:
  `Demo`, `Usage`, `Group`, `Button`, `Tooltip`, `Input Group`, `API Reference`, then the explicit
  Fret-only `RTL` follow-up.
- The gate checks that copyable Kbd snippets stay on the textual/glyph facade lane and do not teach
  `shadcn::raw::*`, `advanced::`, `compose()`, `asChild`, or `Kbd::from_children(...)` as the default
  docs path.
- Updated `tools/diag-scripts/ui-gallery/kbd/ui-gallery-kbd-docs-smoke.json` so the script waits for
  `API Reference` before `RTL`, matching the page order and tracker claim.
- Updated `docs/audits/shadcn-kbd.md` and `docs/shadcn-declarative-progress.md` to cite the new
  docs-order/raw-free gate.

# Next Action

- Run the Kbd-focused gate and repo guardrails:
  `cargo nextest run -p fret-ui-gallery --test kbd_docs_surface`,
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app kbd --no-fail-fast`,
  `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`,
  `python3 tools/check_layering.py`, `cargo fmt --all --check`, wiki validation, and `git diff --check`.
- If green, commit and push to `origin/main` per the current user instruction.
- Continue this lane by scanning the next Pass row where docs-order or raw exposure evidence is still
  broad or indirect.

# Citations

- `apps/fret-ui-gallery/tests/kbd_docs_surface.rs`
- `tools/diag-scripts/ui-gallery/kbd/ui-gallery-kbd-docs-smoke.json`
- `docs/audits/shadcn-kbd.md`
- `docs/shadcn-declarative-progress.md`
