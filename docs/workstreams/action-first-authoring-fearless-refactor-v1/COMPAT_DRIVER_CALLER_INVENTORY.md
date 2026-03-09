# Action-First Authoring + View Runtime (Fearless Refactor v1) — Compat Driver Caller Inventory

Last updated: 2026-03-09

Related:

- Gap analysis: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md`
- Execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`

This note answers one narrow question for Stage 3 of the hard-delete sequence:

> Who still calls `fret::run_native_with_compat_driver(...)`, and do those callers look like
> temporary migration debt or intentional advanced interop surfaces?

---

## Summary

Current in-tree status (2026-03-09):

- **20 direct call sites** still use `fret::run_native_with_compat_driver(...)`.
- Those callers are **not** random leftovers from the app-entry migration.
- They cluster into three recognizable groups:
  1. **plot/chart retained-driver demos**,
  2. **low-level renderer / asset pipeline demos**,
  3. **advanced demo shells that still need explicit runner/driver control**.

Practical interpretation:

- This surface is currently acting more like an **advanced interop/demo runner seam** than a
  stray compatibility leftover.
- That means immediate hard delete would be premature unless the repo first chooses to migrate or
  quarantine these demos somewhere more explicit.

---

## Caller groups

### Group A — Plot / chart retained-driver demos

These demos build a `FnDriver`, keep their own `UiTree` / retained plot state, and run through the
compat driver path instead of the `fret::App` view runtime.

| File | Why it is still on compat driver | Migration pressure |
| --- | --- | --- |
| `apps/fret-examples/src/area_demo.rs` | retained plot canvas demo with explicit `FnDriver` hooks, `UiTree`, and plot output logging | medium |
| `apps/fret-examples/src/candlestick_demo.rs` | retained chart demo with manual runner/driver lifecycle | medium |
| `apps/fret-examples/src/category_line_demo.rs` | retained chart demo with explicit driver lifecycle | medium |
| `apps/fret-examples/src/error_bars_demo.rs` | retained chart demo with explicit driver lifecycle | medium |
| `apps/fret-examples/src/grouped_bars_demo.rs` | retained chart demo with explicit driver lifecycle | medium |
| `apps/fret-examples/src/heatmap_demo.rs` | retained chart demo with explicit driver lifecycle | medium |
| `apps/fret-examples/src/histogram_demo.rs` | retained chart demo with explicit driver lifecycle | medium |
| `apps/fret-examples/src/inf_lines_demo.rs` | retained chart demo with explicit driver lifecycle | medium |
| `apps/fret-examples/src/linked_cursor_demo.rs` | retained chart demo with cross-state cursor behavior and explicit driver lifecycle | medium |
| `apps/fret-examples/src/plot3d_demo.rs` | advanced plot/3D demo still on explicit runner path | medium |
| `apps/fret-examples/src/stacked_bars_demo.rs` | retained chart demo with explicit driver lifecycle | medium |
| `apps/fret-examples/src/stairs_demo.rs` | retained chart demo with explicit driver lifecycle | medium |
| `apps/fret-examples/src/stems_demo.rs` | retained chart demo with explicit driver lifecycle | medium |
| `apps/fret-examples/src/tags_demo.rs` | retained chart demo with explicit driver lifecycle | medium |

Group reading:

- This is the largest cluster.
- These callers look structurally similar enough that they should be treated as **one family**
  during any future migration or quarantine decision.

---

### Group B — Low-level renderer / asset-pipeline demos

These are not “typical app UI” demos. They use the compat driver to keep full control over raw
scene/image/effect handling.

| File | Why it is still on compat driver | Migration pressure |
| --- | --- | --- |
| `apps/fret-examples/src/alpha_mode_demo.rs` | low-level alpha/image registration demo built directly on `SceneOp` + upload effects | low |
| `apps/fret-examples/src/image_upload_demo.rs` | low-level keyed image upload/cache demo with direct asset-cache host interaction | low |
| `apps/fret-cookbook/examples/compositing_alpha_basics.rs` | cookbook example intentionally teaching renderer semantics rather than view-runtime authoring | low |
| `apps/fret-cookbook/examples/image_asset_cache_basics.rs` | cookbook example intentionally teaching keyed image asset cache behavior | low |

Group reading:

- These callers are strong evidence that `run_native_with_compat_driver(...)` still serves a real
  pedagogical purpose for low-level rendering/asset topics.
- They do **not** look like default-path migration debt.

---

### Group C — Advanced shell / multi-window / diagnostics demos

These demos still rely on explicit runner and driver control for reasons that go beyond a normal
view-runtime app.

| File | Why it is still on compat driver | Migration pressure |
| --- | --- | --- |
| `apps/fret-examples/src/components_gallery.rs` | large retained demo shell with diagnostics, accessibility hooks, command wiring, hot reload, and explicit `FnDriverHooks` ownership | low |
| `apps/fret-examples/src/window_hit_test_probe_demo.rs` | multi-window hit-test probe using `ui_app_driver` -> `into_fn_driver()` plus custom window creation/raising behavior | low |

Group reading:

- These are not “simple remaining migrations”.
- If the repo wants to remove the compat driver, these demos need either:
  - a replacement advanced entry story, or
  - a clear compat-only home.

---

## Shared wrapper

There is also one in-tree helper that centralizes many of the example-side calls:

| File | Role |
| --- | --- |
| `apps/fret-examples/src/lib.rs` | local wrapper that forwards to `fret::run_native_with_compat_driver(...)` for example crates |

This wrapper is useful evidence because it shows the examples crate currently treats the compat
driver as a normal advanced demo-running seam rather than a one-off special case.

---

## Evidence anchors

- Public surface:
  - `ecosystem/fret/src/lib.rs`
  - `ecosystem/fret/README.md`
- Example/cookbook callers:
  - `apps/fret-examples/src/lib.rs`
  - `apps/fret-examples/src/area_demo.rs`
  - `apps/fret-examples/src/components_gallery.rs`
  - `apps/fret-examples/src/window_hit_test_probe_demo.rs`
  - `apps/fret-examples/src/alpha_mode_demo.rs`
  - `apps/fret-examples/src/image_upload_demo.rs`
  - `apps/fret-cookbook/examples/compositing_alpha_basics.rs`
  - `apps/fret-cookbook/examples/image_asset_cache_basics.rs`

---

## Recommended interpretation

The current caller set argues against calling this surface “just stale compatibility”.

A more accurate framing today is:

- `run_native_with_compat_driver(...)` is an **advanced low-level runner seam** still used by:
  - retained chart/plot demos,
  - low-level rendering/asset demos,
  - advanced shell/diagnostics demos.

That leaves the repo with two defensible paths:

1. **Keep it intentionally**
   - document it as advanced interop/runner surface,
   - keep it out of default docs/templates,
   - stop treating it as near-term hard-delete target.
2. **Quarantine it intentionally**
   - move it behind a clearer compat/interop namespace or policy boundary,
   - migrate the caller families in batches,
   - only then deprecate/remove the facade-level name.

What does **not** look justified yet:

- deleting it immediately without a family-by-family migration or quarantine plan.

---

## Recommended next step

Based on the current caller inventory, the narrow next decision should be:

1. decide whether `run_native_with_compat_driver(...)` is a **kept advanced surface** or a
   **quarantined compat surface**,
2. then update:
   - `HARD_DELETE_EXECUTION_CHECKLIST.md`,
   - `HARD_DELETE_GAP_ANALYSIS.md`,
   - `ecosystem/fret/README.md`,
   - and any matching docs gate strategy.

Until that policy is explicit, this surface should be treated as **not ready for hard delete**.
