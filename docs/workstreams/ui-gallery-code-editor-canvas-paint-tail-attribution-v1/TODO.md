# UI Gallery Code Editor Canvas Paint Tail Attribution v1 - TODO

Status: Active
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] CPT-010 [owner=planner] [deps=none] [scope=docs/workstreams/ui-gallery-code-editor-canvas-paint-tail-attribution-v1]
  Goal: Open the narrow follow-on lane, freeze the VCRJ-030 bundle as starting evidence, and keep
  `ViewCache`, `Scroll`, and Windows RTX 4090 baselines out of this first slice.
  Validation:
  `python3 -m json.tool docs/workstreams/ui-gallery-code-editor-canvas-paint-tail-attribution-v1/WORKSTREAM.json`;
  `python3 tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence:
  `docs/workstreams/view-cache-resize-jitter-attribution-v1/CLOSEOUT_AUDIT_2026-05-18.md`;
  `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json`;
  `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/layout.perf.summary.v1.json`.
  Handoff: Complete when the lane docs and catalog validate. Continue with CPT-020.

## M1 - Canvas Source Attribution

- [ ] CPT-020 [owner=codex] [deps=CPT-010] [scope=ecosystem/fret-code-editor,ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs,crates/fret-ui,crates/fret-diag]
  Goal: Map the code-editor `Canvas` callback, windowed rows surface, row cache/scene paths, and
  paint diagnostics counters to concrete source owners.
  Validation:
  `rg -n "windowed_rows_surface|Canvas|paint_perf|row_scene|surface_callback|torture" ecosystem/fret-code-editor ecosystem/fret-ui-kit/src/declarative crates/fret-ui crates/fret-diag -S`.
  Evidence: Dated source attribution note in this lane.
  Handoff: Decide whether CPT-030 should rerun perf with extra instrumentation, add diagnostics, or
  start a focused runtime proof.

## M2 - Reproduction Or Instrumented Bundle

- [ ] CPT-030 [owner=codex] [deps=CPT-020] [scope=target/fret-diag,tools/diag-scripts/ui-gallery/code-editor]
  Goal: Capture a fresh bundle, optionally with any added low-overhead attribution, and verify
  whether the `Canvas` paint tail repeats.
  Validation:
  `target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20`.
  Evidence: Fresh bundle path and stats summary.
  Handoff: If the signature repeats, continue with the smallest owner proof. If it does not repeat,
  record a no-change/no-repro verdict.

## M3 - Focused Proof Or Split

- [ ] CPT-040 [owner=codex] [deps=CPT-030] [scope=owner-selected]
  Goal: Land the smallest justified proof or split the real owner into a narrower follow-on.
  Validation: Owner-specific focused test or diag gate.
  Evidence: Focused test path, diagnostics diff, or split follow-on.
  Handoff: Keep renderer/canvas mechanism work separate from code-editor row-surface policy unless
  the source audit proves they are the same owner.

## M4 - Closeout

- [ ] CPT-050 [owner=codex] [deps=CPT-040] [scope=docs/workstreams/ui-gallery-code-editor-canvas-paint-tail-attribution-v1]
  Goal: Close the lane with the final owner verdict, evidence, and next action.
  Validation:
  `python3 tools/check_layering.py`; `cargo fmt --check`; `git diff --check`;
  `python3 tools/check_workstream_catalog.py`.
  Evidence: Closeout audit and final bundle.
  Handoff: Do not reopen VCRJ unless a future bundle again proves `ViewCache` is the top owner.
