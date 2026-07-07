---
type: Work Progress
title: Button Group docs order and action-state gates are locked
timestamp: 2026-07-07T19:14:09Z
tags:
  - shadcn
  - button-group
  - ui-gallery
  - public-surface
status: verified
---

# Summary

Closed the Button Group documentation-ordering and action-state evidence gap by adding a focused UI
Gallery docs-surface gate and updating the shadcn progress tracker from `In review` to `Pass`.

# Truth

- `apps/fret-ui-gallery/tests/button_group_docs_surface.rs` now locks the upstream Button Group docs
  order through `API Reference` before Fret-only follow-ups.
- The same gate locks the ButtonGroupText label/control diagnostic script and suite wiring.
- `docs/shadcn-declarative-progress.md` now cites the docs-order/action-state gate, existing chrome
  gates, the label/control runtime diagnostic, and the button-group matrix packet.
- No runtime component code changed in this slice.

# Artifacts

- `apps/fret-ui-gallery/tests/button_group_docs_surface.rs`
- `docs/audits/shadcn-button-group.md`
- `docs/shadcn-declarative-progress.md`

# Verification

- `cargo nextest run -p fret-ui-gallery --test button_group_docs_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app button_group_`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome button_group`
- Button Group matrix packet check: status is `regression_locked`, validation gates pass, and
  repair, hardening, and gate queues are empty.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
  (passes with existing historical warnings)
- `git diff --check`

# Notes

- This is a documentation/public-surface closeout for Button Group only; other `In review` shadcn
  rows still need component-specific evidence before being upgraded.
