---
type: Work Progress
title: Bootstrap authoring surface doc order
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/raw-surface-contracts
tags: fret,ui-framework,public-surface,fret-bootstrap,authoring-surface,doc-order
---

# Summary

`fret-bootstrap` now has an explicit source-shape gate for its first-contact documentation order.
The crate rustdoc and README must name `ui_app(...)` / `ui_app_with_hooks(...)` before the
`BootstrapBuilder::new_fn(...)` escape hatch.

# Decision

Keep `fret` as the default batteries-included facade, and keep direct `fret-bootstrap` usage as a
manual assembly layer. Within `fret-bootstrap`, `ui_app(...)` is the author-facing path; FnDriver
construction stays an advanced escape hatch.

# Verification

- `cargo nextest run -p fret-bootstrap --lib authoring_surface_doc_tests --no-fail-fast`
- `cargo check -p fret-bootstrap --locked --no-default-features`
- `python3 tools/check_layering.py`
- `python3 tools/check_consumption_profiles.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next

The next public-surface cleanup should return to code seams. The strongest current candidate is a
multi-grid chart binding for `echarts_multi_grid_demo.rs`, while linked and stress chart demos need
separate contracts before migration.
