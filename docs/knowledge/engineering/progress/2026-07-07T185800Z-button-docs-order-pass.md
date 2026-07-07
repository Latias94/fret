---
type: Work Progress
title: Button docs order is gated for Pass status
timestamp: 2026-07-07T18:58:00Z
tags:
  - shadcn
  - button
  - ui-gallery
  - public-surface
status: verified
---

# Summary

Closed the Button documentation-ordering gap by adding a focused UI Gallery docs-surface gate and
updating the shadcn progress tracker from `In review` to `Pass`.

# Truth

- `apps/fret-ui-gallery/tests/button_docs_surface.rs` now locks the upstream Button docs order
  through `API Reference` before Fret-only follow-ups.
- Existing button semantic-link action-state tests in the same file remain intact.
- `docs/shadcn-declarative-progress.md` now cites the docs-order/action-state gate, the semantic-link
  runtime diagnostic, existing chrome/layout gates, and the button matrix packet.
- No runtime component code changed in this slice.

# Artifacts

- `apps/fret-ui-gallery/tests/button_docs_surface.rs`
- `docs/audits/shadcn-button.md`
- `docs/shadcn-declarative-progress.md`

# Verification

- `cargo nextest run -p fret-ui-gallery --test button_docs_surface`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout button`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_button`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome button_demo`
- Button matrix packet check: status is `regression_locked`, validation gates pass, and repair,
  hardening, and gate queues are empty.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
  (passes with existing historical warnings)
- `git diff --check`

# Notes

- This is a documentation/public-surface closeout for Button only; other `In review` shadcn rows
  still need component-specific evidence before being upgraded.
