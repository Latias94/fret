# IMUI Editor Proof Grouped Paint Read Audit — 2026-04-16

Status: landed follow-on audit
Last updated: 2026-04-16

Related:

- `TODO.md`
- `MILESTONES.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `IMUI_EDITOR_PROOF_APP_OWNER_AUDIT_2026-04-16.md`
- `QUERY_GROUPED_MAINTENANCE_SURFACE_AUDIT_2026-04-15.md`
- `MODEL_STORE_RENDER_READ_OWNER_AUDIT_2026-04-15.md`

## Why this note exists

After the app-owner audit landed, `imui_editor_proof_demo` still had one remaining mixed signal:

- some seams were correctly classified as explicit app-owned helpers,
- but several paint-only render readouts still used raw `get_model_cloned(...)` /
  `get_model_copied(...)` helpers even though they belonged on the grouped selector lane.

This note records the follow-on split so the demo no longer teaches those paint-only readouts as
raw helper exceptions.

## Assumptions

1. Paint-only render readouts for editor diagnostics or chrome belong on grouped selector helpers,
   not raw `get_model_*` reads.
   Evidence: the lane already froze `cx.data().selector_model_layout(...)` /
   `selector_model_paint(...)` as the render-time grouped lane for explicit shared `Model<T>` bags.
   Confidence: Confident.
   Consequence if wrong: the demo would keep leaking raw tracked-read vocabulary into ordinary
   render diagnostics helpers.

2. The app-owned exceptions recorded in `IMUI_EDITOR_PROOF_APP_OWNER_AUDIT_2026-04-16.md` remain
   valid and should not be folded into this grouped selector cleanup.
   Evidence: outliner reorder math and dock/bootstrap target lookup still run outside the ordinary
   render readout lane and already have explicit helper owners.
   Confidence: Confident.
   Consequence if wrong: we would blur render-time readouts with app/bootstrap logic again.

## Owner resolution

### 1. Text assist / text field / string readouts

Owner: grouped paint selector lane.

Decision:

- `editor_text_assist_readout(...)`,
- `editor_text_field_readout(...)`,
- and `editor_string_model_readout(...)`

now read through `cx.data().selector_model_paint(...)`.

Why:

- these helpers are render-time readouts over explicit shared `Model<T>` bags,
- they are not app/bootstrap logic,
- and they should teach the grouped paint selector surface directly.

### 2. Authoring parity shared state + gradient snapshots

Owner: grouped paint selector lane.

Decision:

- `render_authoring_parity_shared_state(...)`,
- the gradient property-group preview read,
- and `build_authoring_parity_gradient_editor(...)`

now read through `cx.data().selector_model_paint(...)`.

Why:

- these values only shape paint-time diagnostics/readout chrome,
- and the grouped selector lane already exists precisely for explicit model-bag readouts like
  this.

### 3. Dock panel embedded target render read

Owner: grouped paint selector lane inside render.

Decision:

- dock-panel render roots now read `m.target` through
  `cx.data().selector_model_paint(&m.target, |target| target)`.

Why:

- this is still a render-time tracked read,
- even though the non-render bootstrap lookup remains app-owned through
  `embedded_target_for_window(...)`.

## Landed result

This audit lands:

- grouped paint selector reads for the IMUI editor proof readout helpers,
- grouped paint selector reads for gradient-stop snapshots and dock-panel embedded target reads,
- source-policy gates that now accept the grouped selector lane and forbid the previous raw
  `get_model_*` readouts from drifting back into those surfaces.

## Decision from this audit

`imui_editor_proof_demo` now has two explicit owner classes instead of one blurry advanced tail:

- app-owned helpers for reorder/bootstrap-only logic,
- grouped paint selector helpers for render-time shared-model readouts.
