---
type: Work Progress
title: Shadcn rustdoc teaches curated lane before raw escape hatch
tags:
  - fret
  - authoring-surface
  - shadcn
  - docs
timestamp: 2026-07-07T13:41:37Z
---

# Summary

Adjusted the `fret` crate-level rustdoc so the shadcn bullet teaches
`fret::shadcn::{Button, Card, ...}` as the first-contact design-system lane before mentioning
`fret::shadcn::raw::*` as an explicit escape hatch.

# Changed Files

- `ecosystem/fret/src/lib.rs`: reordered the shadcn rustdoc guidance and updated
  `readme_and_rustdoc_expose_curated_shadcn_surface` to assert the curated lane appears before the
  raw escape hatch and that the old `{..., raw::*}` first-contact import does not return.

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret readme_and_rustdoc_expose_curated_shadcn_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`
