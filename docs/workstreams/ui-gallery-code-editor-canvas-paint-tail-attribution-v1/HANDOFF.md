# UI Gallery Code Editor Canvas Paint Tail Attribution v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed as the follow-on to
`docs/workstreams/view-cache-resize-jitter-attribution-v1/`.

Starting evidence:

- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json`
- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/layout.perf.summary.v1.json`

Starting verdict:

- The current resize-jitter worst frame is paint-dominated:
  `total=362814us`, `paint=360395us`.
- The top paint hotspot is a `Canvas` node:
  `paint_time_us=360009`, `scene_ops_delta=20009`.
- The source path points through code-editor and windowed rows surface code.
- `code_editor.paint_perf` counters are zero in the same stats output, so source attribution comes
  before runtime changes.

CPT-020 source attribution is complete:

- `CPT_020_SOURCE_ATTRIBUTION_2026-05-18.md`
- The `Canvas` owner is the code-editor windowed rows surface callback.
- The VCRJ-030 bundle has `app_snapshot.code_editor.torture.paint_perf = null` for every snapshot
  because `FRET_CODE_EDITOR_DIAG_PAINT_PERF` was not enabled.
- The all-zero `code_editor.paint_perf frames=10` stats lines are a reporting artifact, not proof
  that row paint did no work.

Final owner proof is complete:

- `CPT_030_CPT_040_OWNER_PROOF_2026-05-18.md`
- `CLOSEOUT_AUDIT_2026-05-18.md`

Final verdict:

- CPT-030 repeated the `Canvas` tail with paint perf enabled:
  `total=380102us`, `paint=376387us`, `rows_painted=20004`.
- The decisive structural evidence was
  `ui-gallery-code-editor-torture-viewport viewport_h=320064 content_h=320064`.
- The root owner was `fret-ui` positioned-container final child sizing for non-absolute
  `Fill` / `Fraction` children under scroll overflow probes.
- CPT-040 after evidence shows the same script bounded the inner viewport:
  `viewport_h=518 content_h=320064`, `rows_painted=289`, worst `total=1425us`,
  worst `paint=398us`.

Runtime/test anchors:

- `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`
- `crates/fret-ui/src/declarative/tests/layout/scroll.rs`

## Next Task

No next task remains in this lane.

Start a new narrow workstream only if future bounded-viewport evidence proves a remaining owner.

## Guardrails

- Keep `ViewCache` out of this lane.
- Keep renderer redesign out of this lane.
- Do not split code-editor row-surface optimization from this evidence set; CPT-040 removed the
  wrong-viewport cause.
- Treat VCRJ-030 as local attribution evidence, not a portable performance baseline.
