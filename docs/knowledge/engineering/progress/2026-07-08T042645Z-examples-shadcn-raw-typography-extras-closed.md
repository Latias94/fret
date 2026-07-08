---
type: Work Progress
title: Examples shadcn raw typography and extras closed
timestamp: 2026-07-08T04:26:45Z
tags:
  - fret-examples
  - fret-ui-shadcn
  - public-surface
  - typography
  - extras
status: verified
---

# Examples shadcn raw typography/extras closed

## Summary

The first-party examples source tree no longer uses `shadcn::raw::typography::*` or
`shadcn::raw::extras::*` for prose and extras helpers.

Updated examples now use the explicit facade helper namespaces:

- `shadcn::typography::*` for typography helpers in the custom-effect, postprocess, drop-shadow,
  and liquid-glass demos.
- `shadcn::extras::*` for the extras marquee perf probe.

## Policy Update

The examples source-tree policy no longer whitelists raw typography or raw extras as documented raw
escape hatches. `shadcn::typography::*` is also no longer classified as a forbidden curated marker,
because it is now the explicit facade lane.

The remaining shadcn raw usage in `apps/fret-examples/src` is the retained advanced service seam:
`fret::shadcn::raw::advanced::sync_theme_from_environment(...)`.

## Verification

Completed:

- `rg -n "shadcn::raw::(typography|extras)::" apps/fret-examples/src tools/examples_source_tree_policy`
- `cargo check -p fret-examples --lib`
- `cargo nextest run -p fret-examples --lib --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`

Also ran `PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 tools/examples_source_tree_policy/gate.py`.
It still fails on the existing 230 unrelated source-policy problems; the shadcn raw
typography/extras allowance was removed and no migrated source callsite remains.

## Residual Risk

`tools/examples_source_tree_policy/gate.py` has pre-existing unrelated failures in this workstream.
This slice should not attempt to close those broad source-policy issues unless they become necessary
to validate the shadcn raw cleanup.
