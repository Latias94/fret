# Milestone 4 — `fret-demo` lesson-shaped candidates (migrated)

This document started as a short-list of **lesson-shaped** `fret-demo` bins we wanted to migrate into
the cookbook (`apps/fret-cookbook/examples/*`), so new users would not have to start in the
maintainer-grade demo harnesses.

Status (2026-03-04): **the initial batch has been migrated** and most of the redundant
`apps/fret-demo/src/bin/*` wrappers were removed.

Notes:

- `apps/fret-examples` remains a maintainer-grade crate (intentionally heavy).
- A few `fret-demo` bins remain as **contract/perf probes** where referenced by ADRs/workstreams.

## Selection rubric (what counts as “lesson-shaped”)

A good migration candidate is:

- **One concept per file** (or can be made so without losing intent).
- **App-author relevant** (things people actually need when building apps).
- **Not a stress harness** (no perf torture loops, huge datasets, or conformance-only scenarios).
- **Not platform-specific** (avoid MF/AVF video imports, etc.).
- **Works with the cookbook dependency budget**:
  - Prefer `fret` (with `["desktop", "shadcn"]`) + `fret-ui-shadcn` surfaces.
  - If it needs heavier deps, it must be **feature-gated** in the cookbook.
- Prefer **action-first / view runtime** authoring.
  - If the upstream demo is MVU/legacy, migration implies rewriting (or keeping it as feature-gated “Legacy MVU” until action-first lands).

## Initial migration batch (implemented in cookbook)

These are the first “lesson-shaped” examples we actually landed in `fret-cookbook` as part of M4.

- Toasts: [`apps/fret-cookbook/examples/toast_basics.rs`](../../../apps/fret-cookbook/examples/toast_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/toast-basics/cookbook-toast-basics-smoke.json`](../../../tools/diag-scripts/cookbook/toast-basics/cookbook-toast-basics-smoke.json)
- Date picker: [`apps/fret-cookbook/examples/date_picker_basics.rs`](../../../apps/fret-cookbook/examples/date_picker_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/date-picker-basics/cookbook-date-picker-basics-smoke.json`](../../../tools/diag-scripts/cookbook/date-picker-basics/cookbook-date-picker-basics-smoke.json)
- Form basics: [`apps/fret-cookbook/examples/form_basics.rs`](../../../apps/fret-cookbook/examples/form_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/form-basics/cookbook-form-basics-smoke.json`](../../../tools/diag-scripts/cookbook/form-basics/cookbook-form-basics-smoke.json)
- Drag basics: [`apps/fret-cookbook/examples/drag_basics.rs`](../../../apps/fret-cookbook/examples/drag_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/drag-basics/cookbook-drag-basics-smoke.json`](../../../tools/diag-scripts/cookbook/drag-basics/cookbook-drag-basics-smoke.json)
- Assets reload epoch: [`apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`](../../../apps/fret-cookbook/examples/assets_reload_epoch_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/assets-reload-epoch-basics/cookbook-assets-reload-epoch-basics-smoke.json`](../../../tools/diag-scripts/cookbook/assets-reload-epoch-basics/cookbook-assets-reload-epoch-basics-smoke.json)
- Compositing alpha: [`apps/fret-cookbook/examples/compositing_alpha_basics.rs`](../../../apps/fret-cookbook/examples/compositing_alpha_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/compositing-alpha-basics/cookbook-compositing-alpha-basics-baseline.json`](../../../tools/diag-scripts/cookbook/compositing-alpha-basics/cookbook-compositing-alpha-basics-baseline.json)
- Drop shadow: [`apps/fret-cookbook/examples/drop_shadow_basics.rs`](../../../apps/fret-cookbook/examples/drop_shadow_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/drop-shadow-basics/cookbook-drop-shadow-basics-baseline.json`](../../../tools/diag-scripts/cookbook/drop-shadow-basics/cookbook-drop-shadow-basics-baseline.json)
- Data table basics: [`apps/fret-cookbook/examples/data_table_basics.rs`](../../../apps/fret-cookbook/examples/data_table_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/data-table-basics/cookbook-data-table-basics-baseline.json`](../../../tools/diag-scripts/cookbook/data-table-basics/cookbook-data-table-basics-baseline.json)
- Image asset cache: [`apps/fret-cookbook/examples/image_asset_cache_basics.rs`](../../../apps/fret-cookbook/examples/image_asset_cache_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/image-asset-cache-basics/cookbook-image-asset-cache-basics-baseline.json`](../../../tools/diag-scripts/cookbook/image-asset-cache-basics/cookbook-image-asset-cache-basics-baseline.json)
- Query basics: [`apps/fret-cookbook/examples/query_basics.rs`](../../../apps/fret-cookbook/examples/query_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/query-basics/cookbook-query-basics-baseline.json`](../../../tools/diag-scripts/cookbook/query-basics/cookbook-query-basics-baseline.json)
- Router basics: [`apps/fret-cookbook/examples/router_basics.rs`](../../../apps/fret-cookbook/examples/router_basics.rs)
  - Script: [`tools/diag-scripts/cookbook/router-basics/cookbook-router-basics-smoke.json`](../../../tools/diag-scripts/cookbook/router-basics/cookbook-router-basics-smoke.json)

## Migration status (source → cookbook)

The original lesson-shaped demos were implemented in `apps/fret-examples/src/*_demo.rs` and exposed
via thin `apps/fret-demo/src/bin/*` wrappers. After migration:

- the cookbook example is the canonical user-facing entry,
- the `fret-examples` implementation remains available as a maintainer reference,
- most `fret-demo` wrapper bins were removed (except where used as probe harnesses).

| Source (maintainer) | Old `fret-demo` bin | Cookbook example | Wrapper status |
|---|---|---|---|
| [`apps/fret-examples/src/sonner_demo.rs`](../../../apps/fret-examples/src/sonner_demo.rs) | `sonner_demo` | [`apps/fret-cookbook/examples/toast_basics.rs`](../../../apps/fret-cookbook/examples/toast_basics.rs) | Removed |
| [`apps/fret-examples/src/form_demo.rs`](../../../apps/fret-examples/src/form_demo.rs) | `form_demo` | [`apps/fret-cookbook/examples/form_basics.rs`](../../../apps/fret-cookbook/examples/form_basics.rs) | Removed |
| [`apps/fret-examples/src/drag_demo.rs`](../../../apps/fret-examples/src/drag_demo.rs) | `drag_demo` | [`apps/fret-cookbook/examples/drag_basics.rs`](../../../apps/fret-cookbook/examples/drag_basics.rs) | Kept (probe harness) |
| [`apps/fret-examples/src/date_picker_demo.rs`](../../../apps/fret-examples/src/date_picker_demo.rs) | `date_picker_demo` | [`apps/fret-cookbook/examples/date_picker_basics.rs`](../../../apps/fret-cookbook/examples/date_picker_basics.rs) | Removed |
| [`apps/fret-examples/src/datatable_demo.rs`](../../../apps/fret-examples/src/datatable_demo.rs) | `datatable_demo` | [`apps/fret-cookbook/examples/data_table_basics.rs`](../../../apps/fret-cookbook/examples/data_table_basics.rs) | Removed |
| [`apps/fret-examples/src/image_upload_demo.rs`](../../../apps/fret-examples/src/image_upload_demo.rs) | `image_upload_demo` | [`apps/fret-cookbook/examples/image_asset_cache_basics.rs`](../../../apps/fret-cookbook/examples/image_asset_cache_basics.rs) | Removed |
| [`apps/fret-examples/src/drop_shadow_demo.rs`](../../../apps/fret-examples/src/drop_shadow_demo.rs) | `drop_shadow_demo` | [`apps/fret-cookbook/examples/drop_shadow_basics.rs`](../../../apps/fret-cookbook/examples/drop_shadow_basics.rs) | Kept (perf/contract probe) |
| [`apps/fret-examples/src/assets_demo.rs`](../../../apps/fret-examples/src/assets_demo.rs) | `assets_demo` | [`apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`](../../../apps/fret-cookbook/examples/assets_reload_epoch_basics.rs) | Kept (ADR probe) |
| [`apps/fret-examples/src/alpha_mode_demo.rs`](../../../apps/fret-examples/src/alpha_mode_demo.rs) | `alpha_mode_demo` | [`apps/fret-cookbook/examples/compositing_alpha_basics.rs`](../../../apps/fret-cookbook/examples/compositing_alpha_basics.rs) | Removed |
| [`apps/fret-examples/src/query_demo.rs`](../../../apps/fret-examples/src/query_demo.rs) | `query_demo` | [`apps/fret-cookbook/examples/query_basics.rs`](../../../apps/fret-cookbook/examples/query_basics.rs) | Removed |

### 10) Plot tags overlays (feature-gated)

- Demo bin: [`apps/fret-demo/src/bin/tags_demo.rs`](../../../apps/fret-demo/src/bin/tags_demo.rs)
- Implementation: [`apps/fret-examples/src/tags_demo.rs`](../../../apps/fret-examples/src/tags_demo.rs)
- Proposed cookbook example: `plot_tags_overlays_basics`
- Cookbook label: `Lab`
- Cookbook feature: a future `cookbook-plot` gate (would pull `fret-plot` into the cookbook)
- Notes:
  - This demo is about plot overlays (`TagX`/`TagY`/`PlotText`), not “tags input”.
  - Keep it out of the onboarding ladder until we have a small, feature-gated plot surface in cookbook.

## Remaining candidates (not migrated)

These are still considered “interesting”, but intentionally remain maintainer-grade for now:

- Plot overlay tags: `apps/fret-examples/src/tags_demo.rs` (exposed via `tags_demo`).
  - Rationale: we do not want to pull a plot stack into the cookbook by default.
  - If/when migrated, introduce a feature-gated `cookbook-plot` surface.

## Explicit non-candidates (keep in maintainer/labs)

These should stay in `fret-demo` (or other maintainer harnesses) because they are either stress,
platform-specific, or too large/multi-topic for cookbook:

- Stress/perf harnesses: `*_stress_demo`, `extras_marquee_perf_demo`,
  `virtual_list_stress_demo`, `table_stress_demo`, `chart_stress_demo`, `plot_stress_demo`.
- Media / platform imports: `external_video_imports_*`, `streaming_*`.
- Conformance-only: `cjk_conformance_demo`, `emoji_conformance_demo`, `ime_smoke_demo`.
- Large multi-surface harnesses: `components_gallery`, `workspace_shell_demo`,
  `docking_arbitration_demo`, `liquid_glass_demo`.
- Node graph demos: keep as ecosystem-level stress + parity surfaces (`node_graph_*`).

## Cookbook gating plan (for future migrations)

If/when these are migrated, keep the cookbook “cold compile” story intact:

- Prefer no new default dependencies for `Official` examples.
- For `Lab` examples, add feature gates (similar to existing `cookbook-*` gates in
  [`apps/fret-cookbook/Cargo.toml`](../../../apps/fret-cookbook/Cargo.toml)).

Suggested new gates (only if needed):

- `cookbook-table`
- `cookbook-query`
- `cookbook-router`

## Post-migration checklist (when future code work starts)

- Add stable `test_id`s for each new cookbook example.
- Add a minimal diag script per migrated example under:
  - `tools/diag-scripts/cookbook/<example>/...`
- Add (or extend) a suite manifest under:
  - `tools/diag-scripts/suites/cookbook-<example>/suite.json`
- Remove migrated demos from the **default** `fretboard list native-demos` surface (keep behind `--all`).
