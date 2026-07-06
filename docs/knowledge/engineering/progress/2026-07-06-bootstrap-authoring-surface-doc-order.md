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

The chart cleanup that followed this doc has landed: `echarts_multi_grid_demo.rs` uses
`ChartCanvasMultiGridBinding`, `chart_multi_axis_demo.rs` uses linked chart bindings, and
`chart_stress_demo.rs` uses `ChartCanvasPanelBinding` for stress-harness engine access. The next
public-surface cleanup should scan for remaining app-facing raw model seams and add a narrow owner
or binding only when it preserves the demo's real contract.
