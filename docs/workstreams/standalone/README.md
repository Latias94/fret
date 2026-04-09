# Standalone Workstreams

Catalog updated: 2026-04-09
Moved under `docs/workstreams/standalone/`: 2026-03-12
Date fields in this index are resolved from git history. For files moved during the 2026-03-12
reorganization, the historical tracked path was followed back to the pre-reorg location.
Entries annotated with `history:` point to the pre-reorg tracked path for git lookup only; they are
not current browse targets.

This folder now holds compact workstreams and a small number of shared workstream-adjacent
convention notes that do not yet need their own dedicated subdirectories.
There are no remaining obvious same-prefix multi-file bundles in `standalone/`; those were promoted
back into dedicated folders on 2026-03-12.

## Rules

- Keep single-file workstreams here.
- Keep shared workflow conventions here only when they remain single-file and tightly tied to the
  workstream system itself.
- Promote a note into `docs/workstreams/<slug>/` once it gains TODO/milestone companions or enough
  supporting material that reviewers need one stable directory to inspect the track end to end.
- Use git history, not filesystem mtimes, as the canonical archive date source.

Useful commands:

```bash
git log -1 --follow --format=%cs -- docs/workstreams/standalone/<file>
git log --follow --format='%cs %h %s' -- docs/workstreams/standalone/<file>
git log --since='2026-01-01' --name-only -- docs/workstreams/standalone
```

## Immediate-Mode Note Cluster

Most recent closeout record for the in-tree `imui` retained-compatibility follow-on:

- `docs/workstreams/imui-compat-retained-surface-v1/DESIGN.md`
- `docs/workstreams/imui-compat-retained-surface-v1/TODO.md`
- `docs/workstreams/imui-compat-retained-surface-v1/MILESTONES.md`
- `docs/workstreams/imui-compat-retained-surface-v1/BASELINE_AUDIT_2026-03-31.md`
- `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`

Closed stack reset + teaching-surface closeout record:

- `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`

Standalone `imui` notes in this folder are companion or archive material:

- `imui-imgui-parity-audit-v1.md` is a historical parity audit.
- `imui-ecosystem-facade-perf-v1.md` is a historical companion perf note.
- `imui-shadcn-adapter-v1.md` is a historical adapter companion note.
- `imui-state-integration-v1.md` is a historical service/state compatibility note.

Use them for rationale, parity, and migration history, not as the first stop for current API guidance.

## Icon System Note Cluster

Current closeout + follow-on map for the icon-system workstream family:

- `docs/workstreams/standalone/icon-system-status.md`

Use this note as the first stop when deciding whether new icon work should continue from:

- the base icon contract lane,
- the install/startup reporting branch,
- or the third-party import/presentation branch.

## File Index

- `ai-elements-upstream-alignment.md` — first 2026-02-11, latest 2026-03-07 (history: `docs/workstreams/ai-elements-upstream-alignment.md`)
- `command-gating-surface-alignment-v2-todo-input-dispatch-v2.md` — first 2026-01-25, latest 2026-02-11 (history: `docs/workstreams/command-gating-surface-alignment-v2-todo-input-dispatch-v2.md`)
- `default-actions-v2-todo-input-dispatch-v2.md` — first 2026-01-25, latest 2026-02-11 (history: `docs/workstreams/default-actions-v2-todo-input-dispatch-v2.md`)
- `diag-devtools-gui-refresh-v1.md` — first 2026-03-06, latest 2026-03-06 (history: `docs/workstreams/diag-devtools-gui-refresh-v1.md`)
- `docking-multi-window-imgui-alignment-v1.md` — first 2026-02-18, latest 2026-03-04 (history: `docs/workstreams/docking-multi-window-imgui-alignment-v1.md`)
- `ecosystem-status.md` — first 2026-01-13, latest 2026-03-12 (history: `docs/workstreams/ecosystem-status.md`)
- `execution-concurrency-surface-v1.md` — first 2026-01-26, latest 2026-02-11 (history: `docs/workstreams/execution-concurrency-surface-v1.md`)
- `font-catalog-refresh-policy-v1.md` — first 2026-02-11, latest 2026-02-11 (history: `docs/workstreams/font-catalog-refresh-policy-v1.md`)
- `font-fallback-conformance-v1.md` — first 2026-02-12, latest 2026-02-12 (background note; active execution in `docs/workstreams/font-system-fearless-refactor-v1/`) (history: `docs/workstreams/font-fallback-conformance-v1.md`)
- `font-system-audit-zed-parley-xilem-v1.md` — first 2026-02-12, latest 2026-02-20 (history: `docs/workstreams/font-system-audit-zed-parley-xilem-v1.md`)
- `font-system-v1.md` — first 2026-02-11, latest 2026-02-20 (background roadmap; active execution in `docs/workstreams/font-system-fearless-refactor-v1/`) (history: `docs/workstreams/font-system-v1.md`)
- `fret-node-addons-api-m2.md` — first 2026-02-05, latest 2026-02-05 (history: `docs/workstreams/fret-node-addons-api-m2.md`)
- `fret-node-deterministic-patch-units-m6.md` — first 2026-02-06, latest 2026-02-11 (history: `docs/workstreams/fret-node-deterministic-patch-units-m6.md`)
- `fret-node-internals-m0.md` — first 2026-02-03, latest 2026-03-01 (history: `docs/workstreams/fret-node-internals-m0.md`)
- `fret-node-xyflow-parity.md` — first 2026-01-20, latest 2026-02-16 (history: `docs/workstreams/fret-node-xyflow-parity.md`)
- `gpui-default-semantics-alignment.md` — first 2026-02-18, latest 2026-02-18 (history: `docs/workstreams/gpui-default-semantics-alignment.md`)
- `icon-system-status.md` — first 2026-04-09, latest 2026-04-09
- `imui-ecosystem-facade-perf-v1.md` — first 2026-02-06, latest 2026-02-06 (historical imui companion note; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`) (history: `docs/workstreams/imui-ecosystem-facade-perf-v1.md`)
- `imui-imgui-parity-audit-v1.md` — first 2026-02-08, latest 2026-02-16 (historical parity audit; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`) (history: `docs/workstreams/imui-imgui-parity-audit-v1.md`)
- `imui-shadcn-adapter-v1.md` — first 2026-02-06, latest 2026-02-06 (historical imui companion note; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`) (history: `docs/workstreams/imui-shadcn-adapter-v1.md`)
- `imui-state-integration-v1.md` — first 2026-02-06, latest 2026-03-09 (historical imui companion note; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`) (history: `docs/workstreams/imui-state-integration-v1.md`)
- `macos-docking-multiwindow-imgui-parity.md` — first 2026-01-27, latest 2026-02-16 (history: `docs/workstreams/macos-docking-multiwindow-imgui-parity.md`)
- `os-menubar.md` — first 2026-01-20, latest 2026-02-16 (history: `docs/workstreams/os-menubar.md`)
- `overlay-arbitration-polish-todo.md` — first 2026-01-24, latest 2026-02-11 (history: `docs/workstreams/overlay-arbitration-polish-todo.md`)
- `overlay-lifecycle-phases.md` — first 2026-01-24, latest 2026-02-07 (history: `docs/workstreams/overlay-lifecycle-phases.md`)
- `perf-devtools-skills-v1.md` — first 2026-02-09, latest 2026-02-16 (history: `docs/workstreams/perf-devtools-skills-v1.md`)
- `semantics-decorators-adoption-v1-todo.md` — first 2026-02-05, latest 2026-03-05 (history: `docs/workstreams/semantics-decorators-adoption-v1-todo.md`)
- `shadcn-docs-parity-ui-gallery.md` — first 2026-02-05, latest 2026-02-21 (history: `docs/workstreams/shadcn-docs-parity-ui-gallery.md`)
- `state-driven-style-resolution-v1.md` — first 2026-01-25, latest 2026-02-16 (history: `docs/workstreams/state-driven-style-resolution-v1.md`)
- `svg-path-plot.md` — first 2025-12-26, latest 2025-12-30 (history: `docs/workstreams/svg-path-plot.md`)
- `table-forms-calendar.md` — first 2026-01-13, latest 2026-02-24 (history: `docs/workstreams/table-forms-calendar.md`)
- `text-system-v2-parley.md` — first 2026-01-13, latest 2026-02-20 (history: `docs/workstreams/text-system-v2-parley.md`)
- `tooling-python-unification-inventory.md` — first 2026-03-09, latest 2026-03-09 (history: `docs/workstreams/tooling-python-unification-inventory.md`)
- `ui-editor-egui-imgui-gap-v1.md` — first 2026-02-15, latest 2026-02-16 (history: `docs/workstreams/ui-editor-egui-imgui-gap-v1.md`)
- `ui-editor-imgui-alignment-v1.md` — first 2026-02-14, latest 2026-02-16 (history: `docs/workstreams/ui-editor-imgui-alignment-v1.md`)
- `ui-gallery-docs-page-layout-refactor.md` — first 2026-02-16, latest 2026-03-06 (history: `docs/workstreams/ui-gallery-docs-page-layout-refactor.md`)
- `ui-gallery-layout-correctness.md` — first 2026-01-29, latest 2026-03-01 (history: `docs/workstreams/ui-gallery-layout-correctness.md`)
- `ui-gallery-perf-scroll-measure.md` — first 2026-01-29, latest 2026-02-11 (history: `docs/workstreams/ui-gallery-perf-scroll-measure.md`)
- `ui-gallery-shadcn-docs-alignment-v4-todo.md` — first 2026-02-04, latest 2026-03-09 (history: `docs/workstreams/ui-gallery-shadcn-docs-alignment-v4-todo.md`)
- `ui-perf-gpui-gap-v1.md` — first 2026-02-03, latest 2026-02-16 (history: `docs/workstreams/ui-perf-gpui-gap-v1.md`)
- `ui-perf-renderer-profiling-v1.md` — first 2026-02-04, latest 2026-02-23 (history: `docs/workstreams/ui-perf-renderer-profiling-v1.md`)
- `ui-perf-resize-path-v1.md` — first 2026-02-09, latest 2026-02-19 (history: `docs/workstreams/ui-perf-resize-path-v1.md`)
- `ui-perf-setter-idempotency-v1.md` — first 2026-02-09, latest 2026-02-09 (history: `docs/workstreams/ui-perf-setter-idempotency-v1.md`)
- `viewport-gizmo.md` — first 2026-01-10, latest 2026-02-11 (history: `docs/workstreams/viewport-gizmo.md`)
- `workstream-state-v1.md` — shared machine-readable lane-state convention
- `xyflow-gap-analysis.md` — first 2026-02-12, latest 2026-02-16 (history: `docs/workstreams/xyflow-gap-analysis.md`)
