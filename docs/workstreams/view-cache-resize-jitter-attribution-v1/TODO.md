# ViewCache Resize-Jitter Attribution v1 - TODO

Status: Active
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] VCRJ-010 [owner=planner] [deps=none] [scope=docs/workstreams/view-cache-resize-jitter-attribution-v1]
  Goal: Open the narrow follow-on lane, freeze the starting evidence, declare `ViewCache` as a
  side-effect boundary risk, and define the first source-audit tasks without runtime changes.
  Validation:
  `python3 -m json.tool docs/workstreams/view-cache-resize-jitter-attribution-v1/WORKSTREAM.json`;
  `python3 tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence:
  `docs/workstreams/view-cache-resize-jitter-attribution-v1/DESIGN.md`;
  `docs/workstreams/pressable-clean-geometry-propagation-v1/EVIDENCE_AND_GATES.md`;
  `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/layout.perf.summary.v1.json`.
  Handoff: Complete when the new lane docs and catalog entry validate. Continue with VCRJ-020.

## M1 - ViewCache Source Audit

- [ ] VCRJ-020 [owner=codex] [deps=VCRJ-010] [scope=crates/fret-ui/src/{element.rs,elements/runtime.rs,tree/view_boundary.rs,tree/layout}]
  Goal: Map `ViewCache` geometry, cache-root reuse, boundary dirty tracking, contained relayout,
  root-bound repair, and scroll follow-up responsibilities to concrete source owners.
  Validation:
  `rg -n "ViewCache|view_cache|ViewBoundaryKind::ViewCacheRoot|contained_relayout|cache_root" crates/fret-ui/src/element.rs crates/fret-ui/src/elements/runtime.rs crates/fret-ui/src/tree/view_boundary.rs crates/fret-ui/src/tree/layout -S`.
  Evidence:
  `docs/workstreams/view-cache-resize-jitter-attribution-v1/EVIDENCE_AND_GATES.md`.
  Handoff: Do not edit runtime code until the audit can name which phase owns the `ViewCache`
  hotspot.

## M2 - Fresh Attribution Bundle

- [ ] VCRJ-030 [owner=codex] [deps=VCRJ-020] [scope=target/fret-diag,tools/diag-scripts/ui-gallery]
  Goal: Capture or reuse a fresh UI Gallery resize-jitter bundle and record whether `ViewCache`
  remains the top owner with current code after the source audit.
  Validation:
  `target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20`.
  Evidence:
  `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030-*/bundle.schema2.json`;
  `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030-*/layout.perf.summary.v1.json`.
  Handoff: If `ViewCache` is no longer the top owner, close with a no-change verdict or split the
  actual owner.

## M3 - First Proof Or No-Change Verdict

- [ ] VCRJ-040 [owner=codex] [deps=VCRJ-030] [scope=crates/fret-ui/src/tree/layout,crates/fret-ui/src/declarative/tests]
  Goal: Add a focused proof for the smallest safe `ViewCache` invariant, or record a no-change
  verdict if the source/evidence review shows the hotspot is legitimate or not attributable.
  Validation:
  `cargo nextest run -p fret-ui view_cache layout_engine --no-fail-fast`.
  Evidence: Focused test path or dated no-change note.
  Handoff: A runtime change must preserve cache-root liveness, state retention, boundary tracing,
  and scroll extent repair.

## M4 - Perf Confirmation And Closeout

- [ ] VCRJ-050 [owner=codex] [deps=VCRJ-040] [scope=docs/workstreams/view-cache-resize-jitter-attribution-v1,target/fret-diag]
  Goal: Confirm the final owner verdict with gates, update `WORKSTREAM.json`, and close or split
  follow-on work.
  Validation:
  `python3 tools/check_layering.py`; `cargo fmt --check`; `git diff --check`;
  `python3 tools/check_workstream_catalog.py`.
  Evidence: Final bundle, closeout/no-change note, and updated lane state.
  Handoff: Split `Scroll` or diagnostics attribution only as a separate lane with fresh evidence.
