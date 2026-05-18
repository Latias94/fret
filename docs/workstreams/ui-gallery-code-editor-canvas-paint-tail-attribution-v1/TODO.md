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

- [x] CPT-020 [owner=codex] [deps=CPT-010] [scope=ecosystem/fret-code-editor,ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs,crates/fret-ui,crates/fret-diag]
  Goal: Map the code-editor `Canvas` callback, windowed rows surface, row cache/scene paths, and
  paint diagnostics counters to concrete source owners.
  Validation:
  `rg -n "windowed_rows_surface|Canvas|paint_perf|row_scene|surface_callback|torture" ecosystem/fret-code-editor ecosystem/fret-ui-kit/src/declarative crates/fret-ui crates/fret-diag -S`.
  Evidence: `CPT_020_SOURCE_ATTRIBUTION_2026-05-18.md`.
  Handoff: CPT-030 should rerun the same perf script with
  `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`. Do not optimize runtime until that bundle splits the
  `Canvas` hotspot into windowed-surface and row-scene counters.

## M2 - Reproduction Or Instrumented Bundle

- [x] CPT-030 [owner=codex] [deps=CPT-020] [scope=target/fret-diag,tools/diag-scripts/ui-gallery/code-editor]
  Goal: Capture a fresh bundle, optionally with any added low-overhead attribution, and verify
  whether the `Canvas` paint tail repeats.
  Validation:
  `target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20`.
  Evidence:
  `target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030/1779092655064/bundle.schema2.json`;
  `CPT_030_CPT_040_OWNER_PROOF_2026-05-18.md`.
  Handoff: Complete. The signature repeated, but the decisive clue was the inner scroll viewport:
  `viewport_h=320064`, matching `content_h=320064`. Continue with CPT-040 owner proof.

## M3 - Focused Proof Or Split

- [x] CPT-040 [owner=codex] [deps=CPT-030] [scope=crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs,crates/fret-ui/src/declarative/tests/layout/scroll.rs]
  Goal: Land the smallest justified proof or split the real owner into a narrower follow-on.
  Validation:
  `cargo nextest run -p fret-ui scroll_viewport_for_tall_canvas_child`;
  `target/release/fretboard-dev diag stats target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt040/1779099328829/bundle.schema2.json --sort time --top 20`.
  Evidence:
  `CPT_030_CPT_040_OWNER_PROOF_2026-05-18.md`;
  `target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt040/1779099328829/bundle.schema2.json`.
  Handoff: Complete. The real owner was `fret-ui` positioned-container final sizing for
  non-absolute `Fill` / `Fraction` children under scroll overflow probes. No renderer or
  code-editor follow-on is split from this evidence set.

## M4 - Closeout

- [x] CPT-050 [owner=codex] [deps=CPT-040] [scope=docs/workstreams/ui-gallery-code-editor-canvas-paint-tail-attribution-v1]
  Goal: Close the lane with the final owner verdict, evidence, and next action.
  Validation:
  `python3 tools/check_layering.py`; `cargo fmt --check`; `git diff --check`;
  `python3 tools/check_workstream_catalog.py`.
  Evidence:
  `CLOSEOUT_AUDIT_2026-05-18.md`;
  `CPT_030_CPT_040_OWNER_PROOF_2026-05-18.md`.
  Handoff: Closed. Do not reopen VCRJ, renderer, or code-editor row-surface work from this lane
  unless a future fresh bundle with a bounded viewport proves a new owner.
