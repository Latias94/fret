# Milestone 4 — `fret-demo` lesson-shaped candidates

This document identifies **8–15 “lesson-shaped” demos** currently exposed via `apps/fret-demo/src/bin/*`
that should eventually become **cookbook examples** (`apps/fret-cookbook/examples/*`).

Goal: make `fret-demo` read as **maintainer/labs**, while keeping a smaller, boring, copy/paste-friendly
learning surface in the cookbook.

Non-goals:

- This is **not an implementation plan** (no code moved yet).
- This does **not** change runtime contracts.
- We do **not** attempt to keep `apps/fret-examples` as an onboarding surface (it is intentionally heavy).

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

## Recommended migration list (12 candidates)

Each item is currently reachable via a `fret-demo` bin that calls into `fret_examples::*`.

Legend:

- **Cookbook label**: `Official` vs `Lab` (feature-gated).
- **Cookbook feature**: existing feature gate or a suggested new one.

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

### 1) Toasts / notifications (Sonner)

- Demo bin: [`apps/fret-demo/src/bin/sonner_demo.rs`](../../../apps/fret-demo/src/bin/sonner_demo.rs)
- Implementation: [`apps/fret-examples/src/sonner_demo.rs`](../../../apps/fret-examples/src/sonner_demo.rs)
- Proposed cookbook example: `toast_basics`
- Cookbook label: `Official`
- Cookbook feature: none (prefer to keep this in the default ladder)
- Notes:
  - Teaches: overlay + toasts, promise-style lifecycle, “last action” feedback.
  - Should add stable `test_id`s and a short diag smoke (open toast + screenshot).

### 2) Forms + validation ergonomics

- Demo bin: [`apps/fret-demo/src/bin/form_demo.rs`](../../../apps/fret-demo/src/bin/form_demo.rs)
- Implementation: [`apps/fret-examples/src/form_demo.rs`](../../../apps/fret-examples/src/form_demo.rs)
- Proposed cookbook example: `form_basics`
- Cookbook label: `Official`
- Cookbook feature: none
- Notes:
  - Teaches: inputs, validation feedback, disabled states, submit wiring.
  - Should keep the scope narrow (one form, one validation pattern).

### 3) Drag interactions (pointer capture + hit testing)

- Demo bin: [`apps/fret-demo/src/bin/drag_demo.rs`](../../../apps/fret-demo/src/bin/drag_demo.rs)
- Implementation: [`apps/fret-examples/src/drag_demo.rs`](../../../apps/fret-examples/src/drag_demo.rs)
- Proposed cookbook example: `drag_basics`
- Cookbook label: `Official`
- Cookbook feature: none
- Notes:
  - Teaches: pointer capture, drag handles, hover/active affordances.
  - Good place to standardize `test_id` patterns for draggables.

### 4) Date picker (calendar + popover composition)

- Demo bin: [`apps/fret-demo/src/bin/date_picker_demo.rs`](../../../apps/fret-demo/src/bin/date_picker_demo.rs)
- Implementation: [`apps/fret-examples/src/date_picker_demo.rs`](../../../apps/fret-examples/src/date_picker_demo.rs)
- Proposed cookbook example: `date_picker_basics`
- Cookbook label: `Official` (if compile cost is acceptable), otherwise `Lab`
- Cookbook feature: none (preferred) or a new `cookbook-date-picker`
- Notes:
  - Teaches: overlay placement + focus + calendar selection semantics.
  - Should include: open/close, keyboard focus, and one screenshot via diag.

### 5) Data table basics (small, non-stress)

- Demo bin: [`apps/fret-demo/src/bin/datatable_demo.rs`](../../../apps/fret-demo/src/bin/datatable_demo.rs)
- Implementation: [`apps/fret-examples/src/datatable_demo.rs`](../../../apps/fret-examples/src/datatable_demo.rs)
- Proposed cookbook example: `data_table_basics`
- Cookbook label: `Lab`
- Cookbook feature: a new `cookbook-table` (or reuse an existing UI-kit/table crate gate if introduced)
- Notes:
  - Avoid: `table_stress_demo` style scenarios; keep it “boring”.
  - If this depends on heavy table engines, keep it feature-gated.

### 6) Image asset cache (GPU image upload path)

- Demo bin: [`apps/fret-demo/src/bin/image_upload_demo.rs`](../../../apps/fret-demo/src/bin/image_upload_demo.rs)
- Implementation: [`apps/fret-examples/src/image_upload_demo.rs`](../../../apps/fret-examples/src/image_upload_demo.rs)
- Proposed cookbook example: `image_asset_cache_basics`
- Cookbook label: `Lab`
- Cookbook feature: reuse `cookbook-assets` (preferred) or introduce `cookbook-image-assets`
- Notes:
  - Teaches: `fret-ui-assets` image cache host + keyed asset usage.
  - Should avoid file dialogs; generate synthetic images (like checkerboard) for determinism.

### 7) Renderer semantics: drop shadow

- Demo bin: [`apps/fret-demo/src/bin/drop_shadow_demo.rs`](../../../apps/fret-demo/src/bin/drop_shadow_demo.rs)
- Implementation: [`apps/fret-examples/src/drop_shadow_demo.rs`](../../../apps/fret-examples/src/drop_shadow_demo.rs)
- Proposed cookbook example: `drop_shadow_basics`
- Cookbook label: `Lab` (until compile cost is confirmed)
- Cookbook feature: reuse `cookbook-bootstrap` or `cookbook-customv1` depending on implementation needs
- Notes:
  - Teaches: shadow semantics, clipping interactions, and deterministic degradation expectations.

### 8) Assets + reload epoch (authoring loop basics)

- Demo bin: [`apps/fret-demo/src/bin/assets_demo.rs`](../../../apps/fret-demo/src/bin/assets_demo.rs)
- Implementation: [`apps/fret-examples/src/assets_demo.rs`](../../../apps/fret-examples/src/assets_demo.rs)
- Proposed cookbook example: `assets_reload_epoch_basics`
- Cookbook label: `Lab`
- Cookbook feature: reuse `cookbook-assets`
- Notes:
  - Teaches: assets wiring + reload epoch, “edit and see change” patterns (without hotpatch).
  - Keep it deterministic; avoid too many asset types in one file.

### 9) Opacity / alpha compositing (small, visual lesson)

- Demo bin: [`apps/fret-demo/src/bin/alpha_mode_demo.rs`](../../../apps/fret-demo/src/bin/alpha_mode_demo.rs)
- Implementation: [`apps/fret-examples/src/alpha_mode_demo.rs`](../../../apps/fret-examples/src/alpha_mode_demo.rs)
- Proposed cookbook example: `compositing_alpha_basics`
- Cookbook label: `Lab`
- Cookbook feature: likely none (verify), otherwise reuse `cookbook-bootstrap`
- Notes:
  - Teaches: compositing groups / isolated opacity expectations.
  - Great candidate for a screenshot-only diag script.

### 10) Plot tags overlays (feature-gated)

- Demo bin: [`apps/fret-demo/src/bin/tags_demo.rs`](../../../apps/fret-demo/src/bin/tags_demo.rs)
- Implementation: [`apps/fret-examples/src/tags_demo.rs`](../../../apps/fret-examples/src/tags_demo.rs)
- Proposed cookbook example: `plot_tags_overlays_basics`
- Cookbook label: `Lab`
- Cookbook feature: a future `cookbook-plot` gate (would pull `fret-plot` into the cookbook)
- Notes:
  - This demo is about plot overlays (`TagX`/`TagY`/`PlotText`), not “tags input”.
  - Keep it out of the onboarding ladder until we have a small, feature-gated plot surface in cookbook.

### 11) Query basics

- Demo bin: [`apps/fret-demo/src/bin/query_demo.rs`](../../../apps/fret-demo/src/bin/query_demo.rs)
- Implementation: [`apps/fret-examples/src/query_demo.rs`](../../../apps/fret-examples/src/query_demo.rs)
- Proposed cookbook example: `query_basics`
- Cookbook label: `Lab`
- Cookbook feature: `cookbook-query`
- Notes:
  - Implemented as an action-first view runtime example in `fret-cookbook`.
  - Keep it small and deterministic: one query key, invalidate, namespace invalidation, and a stable baseline screenshot.

### 12) Router basics

- Demo bin: [`apps/fret-demo/src/bin/router_query_demo.rs`](../../../apps/fret-demo/src/bin/router_query_demo.rs)
- Implementation: [`apps/fret-examples/src/router_query_demo.rs`](../../../apps/fret-examples/src/router_query_demo.rs)
- Proposed cookbook example: `router_basics`
- Cookbook label: `Lab`
- Cookbook feature: `cookbook-router`
- Notes:
  - Implemented as an action-first router-only lesson in `fret-cookbook`.
  - Query integration is covered separately by `query_basics`.

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

## Cookbook gating plan (doc-only)

If/when these are migrated, keep the cookbook “cold compile” story intact:

- Prefer no new default dependencies for `Official` examples.
- For `Lab` examples, add feature gates (similar to existing `cookbook-*` gates in
  [`apps/fret-cookbook/Cargo.toml`](../../../apps/fret-cookbook/Cargo.toml)).

Suggested new gates (only if needed):

- `cookbook-table`
- `cookbook-query`
- `cookbook-router`

## Post-migration checklist (when code work starts)

- Add stable `test_id`s for each new cookbook example.
- Add a minimal diag script per migrated example under:
  - `tools/diag-scripts/cookbook/<example>/...`
- Add (or extend) a suite manifest under:
  - `tools/diag-scripts/suites/cookbook-<example>/suite.json`
- Remove migrated demos from the **default** `fretboard list native-demos` surface (keep behind `--all`).
