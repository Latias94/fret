---
type: "Work Progress"
title: "Radio Group docs surface gate"
description: "Work Progress for Radio Group docs surface gate."
timestamp: 2026-07-07T23:12:23Z
tags: ["fret", "shadcn", "radio-group", "ui-gallery", "public-surface", "documentation-ordering"]
---

# Summary

Added a focused Radio Group UI Gallery docs-surface gate. The slice keeps the current Pass audit
claim reviewable by locking the page order, the compact Demo vs parts-based Usage/follow-up lanes,
and the existing radio-group diagnostic evidence anchors in one component-owned integration test.

# Details

- Added `apps/fret-ui-gallery/tests/radio_group_docs_surface.rs`.
- Updated `docs/audits/shadcn-radio-group.md` with the new gallery docs-surface validation command.
- Updated the `radio-group` row in `docs/shadcn-declarative-progress.md` to cite the
  docs-order/lane-split gate.
- Verification passed:
  `cargo nextest run -p fret-ui-gallery --test radio_group_docs_surface`;
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app radio_group --no-fail-fast`;
  `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`;
  `python3 tools/check_layering.py`;
  `python3 tools/report_largest_files.py --top 30 --min-lines 800`;
  `cargo fmt --all --check`;
  radio-group diagnostic JSON parse check;
  `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`;
  `git diff --check`.
- The wiki memory validation stayed structurally OK and reported only pre-existing rollup/history
  warnings.

# Next Action

Commit and push `main`.

# Citations

- `apps/fret-ui-gallery/tests/radio_group_docs_surface.rs`
- `apps/fret-ui-gallery/src/ui/pages/radio_group.rs`
- `apps/fret-ui-gallery/src/ui/snippets/radio_group/`
- `tools/diag-scripts/ui-gallery/radio-group/`
- `docs/audits/shadcn-radio-group.md`
- `docs/shadcn-declarative-progress.md`
