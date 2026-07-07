---
type: Work Progress
title: Badge docs order is gated for Pass status
timestamp: 2026-07-07T18:50:49Z
tags:
  - shadcn
  - badge
  - ui-gallery
  - public-surface
status: verified
---

# Summary

Closed the Badge documentation-ordering gap by adding a focused UI Gallery docs-surface gate and
updating the shadcn progress tracker from `In review` to `Pass`.

# Truth

- `apps/fret-ui-gallery/tests/badge_docs_surface.rs` now locks the upstream Badge docs order through
  `API Reference` before the Fret-only `Counts (Fret)` follow-up.
- Existing badge link action-state tests in the same file remain intact.
- `docs/shadcn-declarative-progress.md` now cites the docs-order gate, existing chrome/layout
  gates, and the badge matrix packet.
- No runtime component code changed in this slice.

# Artifacts

- `apps/fret-ui-gallery/tests/badge_docs_surface.rs`
- `docs/audits/shadcn-badge.md`
- `docs/shadcn-declarative-progress.md`

# Verification

- `cargo nextest run -p fret-ui-gallery --test badge_docs_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app badge_`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout badge`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome badge`
- Badge matrix packet check: no mismatches, blockers, repair queue, hardening queue, or gate queue.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
  (passes with existing historical warnings)
- `git diff --check`

# Notes

- This is a documentation/public-surface closeout for Badge only; other `In review` shadcn rows
  still need component-specific evidence before being upgraded.
