---
type: Work Progress
title: Accordion docs order and usage gates support Pass status
timestamp: 2026-07-07T19:20:21Z
tags:
  - shadcn
  - accordion
  - ui-gallery
  - public-surface
status: verified
---

# Summary

Closed the Accordion documentation/public-surface evidence gap by updating the shadcn progress
tracker from `In review` to `Pass` and citing the existing docs, usage, layout, and matrix gates.

# Truth

- `apps/fret-ui-gallery/tests/accordion_docs_surface.rs` already locks the upstream Accordion docs
  order through `API Reference`, the curated facade typed-children usage lane, and the docs/usage
  diagnostic scripts.
- `docs/shadcn-declarative-progress.md` now cites the docs-order/usage gate, runtime diagnostics,
  recipe/layout gates, and the accordion matrix packet.
- `docs/audits/shadcn-accordion.md` now links the matrix packet from its validation evidence.
- No runtime component code changed in this slice.

# Artifacts

- `docs/audits/shadcn-accordion.md`
- `docs/shadcn-declarative-progress.md`

# Verification

- `cargo nextest run -p fret-ui-gallery --test accordion_docs_surface`
- `cargo nextest run -p fret-ui-shadcn --lib accordion`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout accordion`
- Accordion matrix packet check: status is `regression_locked`, validation gates pass, and repair,
  hardening, and gate queues are empty.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
  (passes with existing historical warnings)
- `git diff --check`

# Notes

- This is a documentation/public-surface closeout for Accordion only; other `In review` shadcn rows
  still need component-specific evidence before being upgraded.
