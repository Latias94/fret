# IMUI Editor Proof App Owner Audit — 2026-04-16

Status: landed follow-on audit
Last updated: 2026-04-16

Related:

- `TODO.md`
- `MILESTONES.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `IMUI_IMMEDIATE_LOCALSTATE_BRIDGE_OWNER_AUDIT_2026-04-15.md`
- `APP_DRIVER_RAW_MODEL_OWNER_AUDIT_2026-04-15.md`

## Why this note exists

After the IMUI immediate-mode closure-local bridge slice landed, `imui_editor_proof_demo` still
had a very small raw `app.models()` tail.

That tail did not justify new framework surface. It belonged to two explicit app-owned seams:

- outliner reorder math inside the editor proof,
- dock/bootstrap target lookup outside the render tree.

This audit records that split and leaves the file teaching those owners explicitly.

## Assumptions

1. The outliner reorder proof is intentionally app-owned logic even though it runs inside an IMUI
   surface.
   Evidence: the file already states “Sortable math stays app-owned. `imui` only provides typed
   payloads + drop positions.”
   Confidence: Confident.
   Consequence if wrong: moving the math toward IMUI-specific helpers would widen the wrong layer.

2. Dock graph/bootstrap target lookup is a driver/bootstrap owner, not render authoring.
   Evidence: `ensure_dock_graph_inner(...)` runs with `&mut KernelApp`, `DockManager`, and window
   bootstrap metadata before configuring docking graph state.
   Confidence: Confident.
   Consequence if wrong: we would confuse dock/runtime setup with render-time tracked ownership.

3. No framework change is needed for this slice.
   Evidence: demo-local helpers are sufficient to make the owner explicit without adding new
   `fret_imui`, docking, or app-facade API.
   Confidence: Confident.
   Consequence if wrong: this slice would widen framework surface for a single advanced proof.

## Owner resolution

### 1. IMUI outliner reorder math

Owner: explicit app-owned demo helper.

Decision:

- keep the model-store read on the app owner,
- but route it through `proof_outliner_items_snapshot(...)` and
  `proof_outliner_order_line_for_model(...)`.

Why:

- the proof is intentionally demonstrating that reorder math remains app-owned,
- the helper names make that owner visible,
- and they avoid scattering raw `app.models()` calls through the IMUI closure body.

### 2. Dock/bootstrap embedded target lookup

Owner: explicit app-owned demo helper.

Decision:

- keep the target lookup on raw `app.models()` through `embedded_target_for_window(...)`.

Why:

- the caller is dock/bootstrap setup code,
- not render-time authoring,
- so a demo-local app-owned helper is the correct abstraction.

## Landed result

This audit lands:

- demo-local app-owned helpers for outliner snapshots/order readout,
- a demo-local app-owned helper for dock/bootstrap embedded target lookup,
- source-policy tests that lock those helper names and forbid the previous inline raw reads from
  drifting back.

## Decision from this audit

`imui_editor_proof_demo` outliner reorder math and dock bootstrap still belong to explicit
app-owned helpers.

More concretely: outliner reorder math and dock bootstrap still belong to explicit app-owned
helpers, and they do not justify new framework surface.

Reference sentence: outliner reorder math and dock bootstrap still belong to explicit app-owned helpers, and they do not justify new framework surface.

They do not justify new framework surface, and they should remain separate from both:

- IMUI closure-local `LocalState` bridge reads,
- and pure driver/app-loop owner classes already frozen elsewhere.
