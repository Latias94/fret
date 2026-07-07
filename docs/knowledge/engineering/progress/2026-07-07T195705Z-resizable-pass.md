---
type: Work Progress
title: Resizable public surface is closed to Pass
timestamp: 2026-07-07T19:57:05Z
tags:
  - shadcn
  - resizable
  - ui-gallery
  - public-surface
status: verified
---

# Summary

Closed the Resizable tracker row to `Pass` by refreshing the audit validation evidence and citing
the current matrix packet.

# Truth

- `docs/shadcn-declarative-progress.md` now marks `resizable` as `Pass`.
- `docs/audits/shadcn-resizable.md` now cites the focused recipe, web-golden, Gallery docs-surface,
  Gallery core-example, and matrix packet gates.
- No runtime component code changed in this slice.

# Verification

- `cargo nextest run -p fret-ui-shadcn --test resizable_panel_group_layout`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_resizable`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout resizable::`
- `cargo nextest run -p fret-ui-gallery --test resizable_docs_surface`
- `cargo nextest run -p fret-ui-gallery --lib gallery_resizable_core_examples_keep_upstream_aligned_targets_present`
- Resizable matrix packet check: all 9 validation gates are `pass`, with no live-measurement,
  mismatch, or blocked status counts.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
  (passes with existing historical warnings)
- `git diff --check`
