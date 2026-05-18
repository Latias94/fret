# UI Gallery Code Editor Canvas Paint Tail Attribution v1 - Milestones

Status: Active
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Exit criteria:

- Lane docs exist and name the frozen VCRJ-030 evidence.
- `Canvas` paint tail is the declared owner under investigation.
- `ViewCache`, `Scroll`, and RTX 4090 baselines are explicitly out of this slice.

Status: Complete.

## M1 - Source Attribution

Exit criteria:

- Code-editor canvas/root surface, windowed rows surface, row scene/cache paths, and diagnostics
  counters are mapped to source owners.
- The lane records why `code_editor.paint_perf` is zero while `Canvas` paint owns the tail.

Status: Complete.

Evidence:

- `CPT_020_SOURCE_ATTRIBUTION_2026-05-18.md`

## M2 - Fresh Repro Or Instrumentation

Exit criteria:

- A fresh bundle confirms or rejects the VCRJ-030 `Canvas` paint signature.
- Any missing diagnostics needed for owner attribution are added or explicitly split.

Status: Complete.

Evidence:

- `CPT_030_CPT_040_OWNER_PROOF_2026-05-18.md`
- `target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030/1779092655064/bundle.schema2.json`

## M3 - Proof Or Split

Exit criteria:

- A runtime optimization lands only after a focused owner proof.
- Otherwise, the lane records a no-change verdict or splits the actual owner.

Status: Complete.

Evidence:

- `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`
- `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
- `target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt040/1779099328829/bundle.schema2.json`

## M4 - Closeout

Exit criteria:

- Workstream docs and gates reflect the final owner verdict.
- Remaining work is split by ownership boundary.

Status: Complete.

Evidence:

- `CLOSEOUT_AUDIT_2026-05-18.md`
