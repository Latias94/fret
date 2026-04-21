# Workstreams

Catalog updated: 2026-04-20
Directory layout last reorganized: 2026-03-12
Date fields in this index are resolved from git history. For files moved during the 2026-03-12
reorganization, the historical tracked path was followed back to the pre-reorg location.

This directory contains implementation workstreams, refactor trackers, audits, and longer-running
design notes. These documents are **not** the sole source of truth for project priorities. For the
current sequencing and active cross-workstream stance, start with:

- `docs/roadmap.md`
- `docs/workstreams/standalone/ecosystem-status.md`
- `docs/workstreams/standalone/icon-system-status.md`
- `docs/todo-tracker.md`

## Layout Snapshot

- Reorganized into dedicated workstream directories on 2026-03-12.
- Dedicated directories: 239
- Standalone markdown files: 47 (see `docs/workstreams/standalone/README.md`)
- Top-level markdown files in `docs/workstreams/`: `README.md` only

## Promotion Rule

- Keep a workstream in `standalone/` only while it is compact and self-contained.
- Promote it into `docs/workstreams/<slug>/` once it gains a main doc plus companions such as TODOs,
  milestones, parity notes, evidence docs, or audit appendices.
- Use git history, not filesystem mtimes, as the canonical archive date source.

Useful commands:

```bash
python3 tools/check_workstream_catalog.py
git log -1 --format=%cs -- docs/workstreams/<path>
git log --format='%cs %h %s' -- docs/workstreams/<path>
git log --since='2026-01-01' --name-only -- docs/workstreams
```

## Machine-readable Lane State

Dedicated workstream directories may include:

- `docs/workstreams/<slug>/WORKSTREAM.json`

Use this as a first-open summary after the repo-wide stance docs and before reading a large lane
folder in detail.

The state file should answer:

- whether the lane is active, maintenance, closed, or historical,
- which docs are authoritative right now,
- which repro/gate surfaces are still canonical,
- and whether future work should continue here or start as a narrower follow-on.

This file is not a second source of truth. If it drifts from a closeout audit or explicit status
note, the markdown authority wins and the state file should be refreshed.

Format note:

- `docs/workstreams/standalone/workstream-state-v1.md`

## Historical Status Note Rule

When a workstream doc remains useful as audit/history context but no longer reflects the shipped
surface, add a short status note near the top instead of silently letting it drift.

Prefer this structure:

1. State whether the file is still active, closed, historical, or partially superseded.
2. Name the current shipped surface or current source-of-truth docs explicitly.
3. Say how to read old API names that still appear below:
   - current recommendation,
   - historical-only,
   - or deleted/superseded.

Suggested template:

```md
Status: Historical reference (partially superseded by <new workstream or doc>)
Last updated: YYYY-MM-DD

Status note (YYYY-MM-DD): this document remains useful for <audit/history scope>, but the current
shipped guidance lives in `<current doc 1>` and `<current doc 2>`. References below to
`<old API name>` should be read as historical/deleted unless explicitly marked as retained.
```

Use this note when:

- a default-path API was renamed, collapsed, or deleted,
- a closeout workstream superseded an earlier planning note,
- or a file is still worth keeping for evidence but should not teach the current golden path.

Do not rewrite every old symbol out of closeout records, migration matrices, or delete audits. In
those files, keep historical names when they are the evidence.

## Immediate-Mode Workstream Map

Current source of truth for the in-tree immediate-mode stack:

- Maintenance umbrella for editor-grade product closure:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/M0_BASELINE_AUDIT_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_FOOTGUN_AUDIT_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_DEMOTE_DELETE_PLAN_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_ROOT_HOSTING_RULE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_STABLE_IDENTITY_RULE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P2_DISCOVERABILITY_ENTRY_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
  - Scope: keep the phase ordering and cross-phase status for the remaining maturity gap after the
    stack-reset closeouts without reopening runtime widening or generic helper-growth by default;
    future `fret-ui-kit::imui` widening still needs the frozen two-surface proof budget before
    review, and implementation-heavy work should stay in narrower follow-ons or the active docking
    parity lane.

- Closed narrow P0 response-status lifecycle closeout record:
  - `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/M0_BASELINE_AUDIT_2026-04-13.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/TODO.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/MILESTONES.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json`
  - Scope: records the landed first `ResponseExt` lifecycle vocabulary after direct pressables,
    menu items, boolean controls, slider, input text, textarea, combo, and combo-model helpers all
    gained focused proof without widening `fret-authoring::Response` or `crates/fret-ui`.

- Closed narrow IMUI control-chrome closeout record:
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/DESIGN.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/M0_BASELINE_AUDIT_2026-04-14.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/TODO.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/MILESTONES.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/WORKSTREAM.json`
  - Scope: records the landed shared `fret-ui-kit::imui` control-chrome rewrite after the compact
    showcase stopped depending on the old fixed-width workaround and the shared button/field
    surface became the default proof path.

- Closed narrow P0 menu/tab trigger response canonicalization closeout record:
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/DESIGN.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/TODO.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/MILESTONES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/WORKSTREAM.json`
  - Scope: records the landed canonical naming cleanup for helper-owned menu/submenu/tab outward
    response APIs after the additive response surface was already accepted.

- Closed narrow P0 menu/tab trigger response-surface follow-on:
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/DESIGN.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/M0_BASELINE_AUDIT_2026-04-13.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/TODO.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/MILESTONES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/WORKSTREAM.json`
  - Scope: records the landed additive outward response surface for helper-owned menu/submenu/tab
    triggers while preserving legacy `bool open` / no-return wrappers and keeping richer menu/tab
    policy out of scope.

- Closed P1 shell follow-on closeout record:
  - `docs/workstreams/imui-workbench-shell-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/TODO.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json`
  - Scope: records the no-new-helper-yet verdict for promoted first-party workbench shell helpers
    after the umbrella lane froze the P1 proof roster and promoted shell diagnostics floor; keep
    active P3 execution in the existing docking parity lane.

- Active P3 docking parity execution lane:
  - `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M4_WAYLAND_DEGRADATION_POLICY_2026-04-21.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
  - Scope: owns the remaining runner/backend multi-window hand-feel closure, starting from the
    bounded P3 package and the mixed-DPI execution slice rather than reopening the umbrella lane.

- Closed narrow diagnostics follow-on for the mixed-DPI automation preflight gap:
  - `docs/workstreams/diag-monitor-topology-environment-v1/WORKSTREAM.json`
  - `docs/workstreams/diag-monitor-topology-environment-v1/DESIGN.md`
  - `docs/workstreams/diag-monitor-topology-environment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
  - Scope: records the shipped runner-owned host monitor-topology environment fingerprint after
    the docking lane froze `scale_factors_seen` as evidence-only; future host-environment
    predicates still need a different follow-on instead of reopening the docking lane or this
    folder.

- Closed narrow diagnostics closeout record for the first environment-predicate contract:
  - `docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json`
  - `docs/workstreams/diag-environment-predicate-contract-v1/DESIGN.md`
  - `docs/workstreams/diag-environment-predicate-contract-v1/BASELINE_AUDIT_2026-04-20.md`
  - `docs/workstreams/diag-environment-predicate-contract-v1/CLOSEOUT_AUDIT_2026-04-20.md`
  - Scope: classifies the current environment snapshot families, lands the first
    `host.monitor_topology` environment-admission contract, and closes with the rule that any
    wider grammar needs a different narrow follow-on instead of a generic erased runtime snapshot
    abstraction.

- Most recent closeout record for the compatibility-retained follow-on:
  - `docs/workstreams/imui-compat-retained-surface-v1/DESIGN.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/TODO.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/MILESTONES.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/BASELINE_AUDIT_2026-03-31.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`

- Closed stack reset + teaching-surface closeout record:
  - `docs/workstreams/imui-stack-fearless-refactor-v2/DESIGN.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v2/TODO.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v2/MILESTONES.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v2/BASELINE_AUDIT_2026-03-31.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v2/TEACHING_SURFACE_AUDIT_2026-03-31.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`

- Completed stack reset / baseline closeout record:
  - `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v1/TODO.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v1/MILESTONES.md`

- Historical authoring-vocabulary closure / closeout record:
  - `docs/workstreams/imui-authoring-vocabulary-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-authoring-vocabulary-closure-v1/TODO.md`
  - `docs/workstreams/imui-authoring-vocabulary-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-authoring-vocabulary-closure-v1/GAP_AUDIT_2026-03-31.md`
  - `docs/workstreams/imui-authoring-vocabulary-closure-v1/CLOSEOUT_AUDIT_2026-03-31.md`

- Closed editor-grade helper closure / closeout record:
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/TODO.md`
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/EDITOR_GRADE_GAP_AUDIT_2026-03-29.md`
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/DRAG_DROP_BOUNDARY_AUDIT_2026-03-29.md`

- Closed reusable immediate sortable/reorder policy / closeout record:
  - `docs/workstreams/imui-sortable-recipe-v1/DESIGN.md`
  - `docs/workstreams/imui-sortable-recipe-v1/TODO.md`
  - `docs/workstreams/imui-sortable-recipe-v1/MILESTONES.md`
  - `docs/workstreams/imui-sortable-recipe-v1/CLOSEOUT_AUDIT_2026-03-30.md`
  - `docs/workstreams/imui-sortable-recipe-v1/SECOND_PROOF_SURFACE_DECISION_2026-03-30.md`

- Closed same-window source-side drag preview ghost / closeout record:
  - `docs/workstreams/imui-drag-preview-ghost-v1/DESIGN.md`
  - `docs/workstreams/imui-drag-preview-ghost-v1/TODO.md`
  - `docs/workstreams/imui-drag-preview-ghost-v1/MILESTONES.md`
  - `docs/workstreams/imui-drag-preview-ghost-v1/UPSTREAM_PARITY_AUDIT_2026-03-30.md`
  - `docs/workstreams/imui-drag-preview-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

- Closed generic cross-window ghost baseline / closeout record:
  - `docs/workstreams/imui-cross-window-ghost-v1/DESIGN.md`
  - `docs/workstreams/imui-cross-window-ghost-v1/TODO.md`
  - `docs/workstreams/imui-cross-window-ghost-v1/MILESTONES.md`
  - `docs/workstreams/imui-cross-window-ghost-v1/M1_CONTRACT_FREEZE_2026-03-30.md`
  - `docs/workstreams/imui-cross-window-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

- Closed shell-aware ghost choreography follow-on / closeout record:
  - `docs/workstreams/imui-shell-ghost-choreography-v1/DESIGN.md`
  - `docs/workstreams/imui-shell-ghost-choreography-v1/TODO.md`
  - `docs/workstreams/imui-shell-ghost-choreography-v1/MILESTONES.md`
  - `docs/workstreams/imui-shell-ghost-choreography-v1/M1_CONTRACT_FREEZE_2026-03-30.md`
  - `docs/workstreams/imui-shell-ghost-choreography-v1/CLOSEOUT_AUDIT_2026-03-30.md`

- Closed transparent moving-window payload overlap follow-on / closeout record:
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/DESIGN.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/TODO.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/MILESTONES.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/M1_CONTRACT_FREEZE_2026-03-30.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/M2_LAUNCHED_PROOF_READ_2026-03-30.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/CLOSEOUT_AUDIT_2026-03-30.md`

Historical archive directories retained for rationale/audit history only:

- `docs/workstreams/imui-authoring-facade-v1/`
- `docs/workstreams/imui-authoring-facade-v2/`
- `docs/workstreams/imui-ecosystem-facade-v1/`
- `docs/workstreams/imui-ecosystem-facade-v2/`
- `docs/workstreams/imui-ecosystem-facade-v3/`

Current + historical companion/audit notes retained under `standalone/`:

- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `docs/workstreams/standalone/imui-ecosystem-facade-perf-v1.md`
- `docs/workstreams/standalone/imui-shadcn-adapter-v1.md`
- `docs/workstreams/standalone/imui-state-integration-v1.md`

Rule:

- Prefer the active P0 response-status lifecycle follow-on for current `ResponseExt` lifecycle
  vocabulary work.
- Prefer the active P0 menu/tab trigger response canonicalization follow-on for current cleanup of
  helper-owned menu/submenu/tab outward response naming.
- Prefer the closed P0 menu/tab trigger response-surface follow-on for the latest helper-owned
  menu/submenu/tab outward-response verdict; start a narrower follow-on if broader policy work is
  still missing.
- Prefer the active product-closure follow-on for the current "what is still missing relative to an
  editor-grade Dear ImGui-class experience?" answer and for the current phase ordering across
  authoring, shell, tooling, and multi-window hand-feel.
- Prefer the closed P1 shell follow-on only for the latest no-new-helper-yet verdict on promoted
  first-party shell helpers.
- Prefer the existing docking parity lane for the next active P3 multi-window hand-feel work.
- Prefer the compatibility-retained follow-on lane for the latest keep/delete verdict on retained
  compatibility surfaces that leaked into public/proof `imui` paths.
- Prefer the v2 workstream for the closed stack reset, editor adapter closure, and teaching-surface
  cleanup record.
- Prefer the completed stack-reset docs for baseline API/ownership guidance and the first fearless
  cleanup pass.
- Prefer the authoring-vocabulary closeout docs only as historical evidence for what the repo once
  considered missing before the current baseline audit.
- Prefer the editor-grade closeout docs for what landed and what was intentionally deferred.
- Prefer the sortable recipe closeout docs for the shipped v1 row-level recipe boundary.
- Prefer the drag preview ghost closeout docs for the shipped same-window source-side preview boundary.
- Prefer the cross-window ghost closeout docs for the shipped generic multi-window transfer
  baseline.
- Prefer the shell ghost choreography closeout docs for the current docking-owned shell ghost owner
  split and first-shell-rule proof.
- Prefer the transparent payload z-order closeout docs for the diagnostics/runtime closure of the
  transparent moving-window overlap lane.
- Prefer the M1 freeze record in the shell ghost choreography lane for the docking-first owner
  split and proof baseline.
- Prefer the M1 freeze record in the cross-window closeout lane for the generic owner split and
  fallback baseline.
- Read the older `imui-*` files only as archive evidence, parity notes, or migration history.
- Treat any old symbol names in those archive files as historical unless explicitly marked as retained.

## Directory Index

- `docs/workstreams/a11y-accesskit-xplat-bridge-v1/` — first 2026-02-16, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/a11y-range-semantics-fearless-refactor-v1/` — first 2026-02-23, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/a11y-semantics-closure-v1/` — first 2026-02-23, latest 2026-02-23, 3 markdown docs
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-16, 50 markdown docs
- `docs/workstreams/action-payload-surface-dedup-v1/` — first n/a, latest n/a, 0 markdown docs
- `docs/workstreams/action-write-surface-fearless-refactor-v1/` — first 2026-03-17, latest 2026-03-17, 8 markdown docs
- `docs/workstreams/adaptive-layout-contract-closure-v1/` — first 2026-04-10, latest 2026-04-10, 13 markdown docs
- `docs/workstreams/adaptive-presentation-surface-v1/` — first 2026-04-11, latest 2026-04-11, 8 markdown docs
- `docs/workstreams/ai-elements-port/` — first 2026-02-05, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/animata-recipes-v1/` — first 2026-02-13, latest 2026-02-27, 2 markdown docs
- `docs/workstreams/app-composition-density-follow-on-v1/` — first 2026-03-17, latest 2026-03-20, 7 markdown docs
- `docs/workstreams/app-entry-builder-v1/` — first 2026-02-26, latest 2026-03-12, 3 markdown docs
- `docs/workstreams/app-iteration-fast-restart-v1/` — first 2026-02-15, latest 2026-02-15, 3 markdown docs
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/` — first 2026-03-16, latest 2026-03-20, 10 markdown docs
- `docs/workstreams/authoring-ergonomics-fluent-builder/` — first 2026-01-21, latest 2026-03-12, 2 markdown docs
- `docs/workstreams/authoring-paradigm-gpui-style-v1/` — first 2026-02-05, latest 2026-03-06, 2 markdown docs
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/` — first 2026-03-10, latest 2026-03-12, 5 markdown docs
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs (closed closeout record for one bootstrap-level known startup failure taxonomy that unifies returned settings/keymap/menu/assets startup failures with panic-only explicit icon install failures without reopening lifecycle return types or widening the root `fret` re-export budget)
- `docs/workstreams/bottom-up-fearless-refactor-v1/` — first 2026-02-07, latest 2026-03-09, 5 markdown docs
- `docs/workstreams/canvas-world-layer-v1/` — first 2026-02-12, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/carousel-embla-fearless-refactor-v1/` — first 2026-02-26, latest 2026-03-02, 11 markdown docs
- `docs/workstreams/carousel-embla-parity-v1/` — first 2026-02-13, latest 2026-02-27, 3 markdown docs
- `docs/workstreams/carousel-embla-parity-v2/` — first 2026-02-28, latest 2026-03-03, 5 markdown docs
- `docs/workstreams/clipboard-write-completion-fearless-refactor-v1/` — first 2026-03-25, latest 2026-03-25, 5 markdown docs
- `docs/workstreams/code-editor-ecosystem-v1/` — first 2026-01-27, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/component-ecosystem-state-integration-v1/` — first 2026-02-06, latest 2026-02-14, 2 markdown docs
- `docs/workstreams/container-aware-editor-rail-helper-shape-v1/` — first 2026-04-11, latest 2026-04-11, 6 markdown docs
- `docs/workstreams/container-aware-editor-rail-surface-v1/` — first 2026-04-11, latest 2026-04-11, 11 markdown docs
- `docs/workstreams/container-queries-v1/` — first 2026-02-09, latest 2026-02-11, 3 markdown docs
- `docs/workstreams/control-chrome-normalization-audit-v1/` — first 2026-02-18, latest 2026-02-19, 3 markdown docs
- `docs/workstreams/control-id-form-association-v1/` — first 2026-03-06, latest 2026-03-08, 3 markdown docs
- `docs/workstreams/crate-audits/` — first 2026-02-08, latest 2026-03-12, 24 markdown docs
- `docs/workstreams/creative-recipes-v1/` — first 2026-02-10, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/current-color-inheritance-fearless-refactor-v1/` — first 2026-02-23, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/` — first n/a, latest n/a, 5 markdown docs
- `docs/workstreams/default-app-productization-fearless-refactor-v1/` — first 2026-04-02, latest 2026-04-09, 7 markdown docs
- `docs/workstreams/device-shell-adaptive-facade-promotion-v1/` — first 2026-04-11, latest 2026-04-11, 6 markdown docs
- `docs/workstreams/device-shell-recipe-wrapper-surface-v1/` — first 2026-04-11, latest 2026-04-11, 6 markdown docs
- `docs/workstreams/device-shell-strategy-surface-v1/` — first 2026-04-10, latest 2026-04-11, 10 markdown docs
- `docs/workstreams/delinea-engine-contract-closure-v1/` — first 2026-02-09, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/diag-ai-agent-debugging-v1/` — first 2026-02-21, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/diag-architecture-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-06, 20 markdown docs
- `docs/workstreams/diag-bundle-schema-v2/` — first 2026-02-21, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/diag-cli-fearless-refactor-v1/` — first 2026-03-26, latest 2026-03-26, 5 markdown docs
- `docs/workstreams/diag-cli-first-party-migration-v1/` — first 2026-03-26, latest 2026-03-26, 1 markdown docs
- `docs/workstreams/diag-cli-help-and-gates-v1/` — first 2026-03-26, latest 2026-03-26, 1 markdown docs
- `docs/workstreams/diag-cli-main-lanes-hardening-v1/` — first 2026-03-26, latest 2026-03-26, 1 markdown docs
- `docs/workstreams/diag-devtools-gui-v1/` — first 2026-02-07, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/diag-extensibility-and-capabilities-v1/` — first 2026-02-10, latest 2026-02-28, 9 markdown docs
- `docs/workstreams/diag-environment-predicate-contract-v1/` — first n/a, latest n/a, 11 markdown docs (closed narrow diagnostics closeout record that classifies existing environment snapshot families, lands the first `host.monitor_topology` environment-admission contract, and defers wider grammar until a second admitted source exists)
- `docs/workstreams/diag-fearless-refactor-v1/` — first 2026-02-21, latest 2026-03-06, 16 markdown docs
- `docs/workstreams/diag-fearless-refactor-v2/` — first 2026-03-06, latest 2026-03-10, 35 markdown docs
- `docs/workstreams/diag-monitor-topology-environment-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on that adds a runner-owned host monitor-topology environment fingerprint without adding mixed-DPI-only campaign gating or environment predicates)
- `docs/workstreams/diag-perf-attribution-v1/` — first 2026-02-14, latest 2026-02-14, 4 markdown docs
- `docs/workstreams/diag-perf-profiling-infra-v1/` — first 2026-02-15, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/diag-simplification-v1/` — first 2026-02-13, latest 2026-03-09, 4 markdown docs
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the narrow `fret-diag-workflow` evidence-owner correction between public `fretboard` and workspace-dev `fretboard-dev` diagnostics help surfaces)
- `docs/workstreams/diag-v2-hardening-and-switches-v1/` — first 2026-02-26, latest 2026-03-03, 10 markdown docs
- `docs/workstreams/docking-arbitration-diag-hardening-v1/` — first 2026-02-28, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/docking-hovered-window-contract-v1/` — first 2026-02-17, latest 2026-02-18, 2 markdown docs
- `docs/workstreams/docking-multiviewport-arbitration-v1/` — first 2026-01-27, latest 2026-03-02, 2 markdown docs
- `docs/workstreams/docking-multiwindow-imgui-parity/` — first 2026-01-27, latest 2026-04-21, 8 markdown docs
- `docs/workstreams/docking-nary-split-graph-v1/` — first 2026-02-11, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/docking-tabbar-fearless-refactor-v1/` — first 2026-02-28, latest 2026-03-05, 9 markdown docs
- `docs/workstreams/ecosystem-integration-traits-v1/` — first 2026-03-11, latest 2026-03-12, 5 markdown docs
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/` — first 2026-03-09, latest 2026-03-10, 7 markdown docs
- `docs/workstreams/editor-tabstrip-unification-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-05, 7 markdown docs
- `docs/workstreams/editor-text-pipeline-v1/` — first 2026-02-14, latest 2026-03-03, 3 markdown docs
- `docs/workstreams/environment-queries-v1/` — first 2026-02-09, latest 2026-03-12, 6 markdown docs
- `docs/workstreams/example-suite-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-12, 9 markdown docs
- `docs/workstreams/external-texture-imports-v1/` — first 2026-02-13, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/external-texture-imports-v2-zero-low-copy/` — first 2026-02-16, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/executor-backed-mutation-surface-v1/` — first n/a, latest n/a, 8 markdown docs (closed narrow closeout lane for the default app-facing async submit/mutation split on `fret-mutation` + `fret`; keeps `fret-query` read-only and records why GenUI/Sonner executor-backed side flows stay recipe/app-owned exceptions)
- `docs/workstreams/mutation-toast-feedback-golden-path-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow closeout lane that turns the closed submit-owner verdict into a copyable cookbook + docs + screenshot path while keeping Sonner as feedback-only chrome above the authoritative mutation lane)
- `docs/workstreams/foreground-inheritance-late-binding-v2/` — first 2026-02-24, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/foreground-style-context-fearless-refactor-v1/` — first 2026-03-06, latest 2026-03-06, 3 markdown docs
- `docs/workstreams/foundation-closure-p0/` — first 2026-01-28, latest 2026-02-11, 2 markdown docs
- `docs/workstreams/font-bundle-release-boundary-v1/` — first 2026-04-08, latest 2026-04-08, 4 markdown docs
- `docs/workstreams/font-system-fearless-refactor-v1/` — first 2026-03-13, latest 2026-03-13, 3 markdown docs
- `docs/workstreams/font-mainline-fearless-refactor-v1/` — first 2026-03-14, latest 2026-03-14, 3 markdown docs
- `docs/workstreams/framework-modularity-fearless-refactor-v1/` — first 2026-02-27, latest 2026-02-27, 3 markdown docs
- `docs/workstreams/fretboard-cli-fearless-refactor-v1/` — first 2026-03-26, latest 2026-03-26, 4 markdown docs
- `docs/workstreams/fretboard-public-app-author-surface-v1/` — first 2026-04-09, latest 2026-04-09, 10 markdown docs
- `docs/workstreams/fretboard-public-dev-implementation-v1/` — first 2026-04-09, latest 2026-04-09, 6 markdown docs
- `docs/workstreams/fretboard-public-diag-implementation-v1/` — first 2026-04-09, latest 2026-04-09, 5 markdown docs
- `docs/workstreams/fret-interaction-kernel-v1/` — first 2026-02-10, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/` — first 2026-03-06, latest 2026-03-12, 7 markdown docs
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/` — first 2026-03-13, latest 2026-03-13, 3 markdown docs
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-06, 3 markdown docs
- `docs/workstreams/fret-node-style-skinning-v1/` — first 2026-02-27, latest 2026-03-01, 7 markdown docs
- `docs/workstreams/fret-node-style-skinning-v2/` — first 2026-03-01, latest 2026-03-01, 3 markdown docs
- `docs/workstreams/fret-node-style-skinning-v3/` — first 2026-03-02, latest 2026-03-02, 6 markdown docs
- `docs/workstreams/genui-json-render-v1/` — first 2026-02-14, latest 2026-03-02, 3 markdown docs
- `docs/workstreams/generated-icon-presentation-defaults-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for explicit versioned `OriginalColors` vs `Mask` defaults in generated/imported icon packs without reopening acquisition or runtime icon contracts)
- `docs/workstreams/grid-track-and-slot-placement-parity-v1/` — first 2026-04-07, latest 2026-04-07, 4 markdown docs
- `docs/workstreams/gesture-recognizers-v1/` — first 2026-02-11, latest 2026-02-11, 3 markdown docs
- `docs/workstreams/gpui-parity-refactor/` — first 2026-01-15, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/headless-dnd-fearless-refactor-v1/` — first 2026-03-13, latest 2026-03-13, 12 markdown docs
- `docs/workstreams/headless-table-tanstack-parity/` — first 2026-02-04, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/hotpatch-devloop-alignment-v1/` — first 2026-02-15, latest 2026-03-01, 4 markdown docs
- `docs/workstreams/image-source-view-cache-v1/` — first 2026-02-13, latest 2026-02-13, 3 markdown docs
- `docs/workstreams/image-support-v1/` — first 2026-02-09, latest 2026-02-11, 2 markdown docs
- `docs/workstreams/icon-install-error-reporting-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs (closed closeout record for known icon-install failure reports plus diagnostics-aware panic-hook logging without changing setup/bootstrap return types)
- `docs/workstreams/icon-install-health-hardening-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs (closed closeout record for fail-fast explicit icon-pack install semantics plus best-effort partial helper fallback without reopening the closed runtime icon contract)
- `docs/workstreams/icon-system-extension-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs
- `docs/workstreams/iconify-acquisition-prestep-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for subset-first explicit remote/pinned Iconify acquisition as a separate pre-step that writes local snapshot + provenance artifacts for the closed generator lane)
- `docs/workstreams/iconify-import-pack-generator-v1/` — first n/a, latest n/a, 9 markdown docs (closed closeout record for the v1 third-party icon-pack generator contract: reusable generator + public CLI, local SVG/Iconify snapshot inputs, explicit semantic alias config, and deterministic proof gates all landed)
- `docs/workstreams/iconify-presentation-defaults-report-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs (closed closeout record for optional versioned review-report output from the thin presentation-defaults suggestion helper without changing import defaults)
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/` — first n/a, latest n/a, 9 markdown docs (closed closeout record for the thin `icons suggest presentation-defaults` helper that derives advisory config from explicit Iconify acquisition provenance without changing import defaults)
- `docs/workstreams/imui-authoring-facade-v1/` — first 2026-02-03, latest 2026-02-16, 2 markdown docs (historical archive; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-authoring-facade-v2/` — first 2026-02-03, latest 2026-03-02, 2 markdown docs (historical archive; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-authoring-vocabulary-closure-v1/` — first n/a, latest n/a, 5 markdown docs (closed closeout record for the shipped generic immediate helper vocabulary relative to Dear ImGui/egui after the editor-grade and ghost closeouts)
- `docs/workstreams/imui-compat-retained-surface-v1/` — first n/a, latest n/a, 5 markdown docs (closed closeout record for deleting public/proof retained-compatibility `imui` facades while keeping one declarative node-graph proof seam)
- `docs/workstreams/imui-cross-window-ghost-v1/` — first n/a, latest n/a, 5 markdown docs (closed closeout record for the shipped generic cross-window ghost baseline; M1 contract freeze accepted)
- `docs/workstreams/imui-drag-preview-ghost-v1/` — first n/a, latest n/a, 5 markdown docs (closed closeout record for the shipped same-window source-side drag preview ghost)
- `docs/workstreams/imui-ecosystem-facade-v1/` — first 2026-02-05, latest 2026-02-16, 2 markdown docs (historical archive; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-ecosystem-facade-v2/` — first 2026-02-06, latest 2026-02-08, 8 markdown docs (historical archive; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-ecosystem-facade-v3/` — first 2026-02-06, latest 2026-02-16, 2 markdown docs (historical archive; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/` — first n/a, latest n/a, 7 markdown docs (closed closeout record for the shared IMUI control-chrome rewrite after the compact showcase fixed-width workaround was deleted)
- `docs/workstreams/imui-editor-grade-product-closure-v1/` — first n/a, latest n/a, 20 markdown docs
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for the canonical helper naming cleanup after the additive trigger-response lane landed)
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/` — first n/a, latest n/a, 6 markdown docs
- `docs/workstreams/imui-response-status-lifecycle-v1/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for the first `ResponseExt` lifecycle vocabulary after single-line and multiline text-entry lifecycle proof landed)
- `docs/workstreams/imui-workbench-shell-closure-v1/` — first n/a, latest n/a, 5 markdown docs
- `docs/workstreams/imui-editor-grade-surface-closure-v1/` — first 2026-03-29, latest 2026-03-29, 6 markdown docs (closed closeout record; sortable recipe follow-on lives in `docs/workstreams/imui-sortable-recipe-v1/`)
- `docs/workstreams/imui-shell-ghost-choreography-v1/` — first n/a, latest n/a, 5 markdown docs (closed closeout record for the shipped docking-owned shell ghost choreography follow-on)
- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for the transparent moving-window payload overlap diagnostics/runtime lane)
- `docs/workstreams/imui-sortable-recipe-v1/` — first 2026-03-29, latest 2026-03-30, 5 markdown docs (closed closeout record for the shipped v1 row-level sortable recipe)
- `docs/workstreams/imui-stack-fearless-refactor-v1/` — first 2026-03-26, latest 2026-03-27, 3 markdown docs (completed stack-reset baseline; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-stack-fearless-refactor-v2/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for the immediate-mode doc reset, editor adapter closure, and proof-surface cleanup lane)
- `docs/workstreams/input-dispatch-v2/` — first 2026-01-22, latest 2026-02-14, 3 markdown docs
- `docs/workstreams/into-element-surface-fearless-refactor-v1/` — first 2026-03-12, latest 2026-03-20, 6 markdown docs
- `docs/workstreams/launcher-utility-windows-v1/` — first 2026-03-03, latest 2026-03-03, 4 markdown docs
- `docs/workstreams/length-percentage-semantics-v1/` — first 2026-02-23, latest 2026-02-27, 3 markdown docs
- `docs/workstreams/localization-i18n-v1/` — first 2026-02-06, latest 2026-02-07, 2 markdown docs
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/` — first 2026-03-16, latest 2026-03-16, 7 markdown docs
- `docs/workstreams/local-state-facade-boundary-hardening-v1/` — first 2026-03-16, latest 2026-03-16, 5 markdown docs
- `docs/workstreams/material3/` — first 2026-01-22, latest 2026-02-24, 5 markdown docs
- `docs/workstreams/material3-expressive-alignment-v1/` — first 2026-02-18, latest 2026-02-18, 4 markdown docs
- `docs/workstreams/material3-icon-toggle-button-expressive-v1/` — first 2026-02-18, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/menu-surfaces-alignment-v1/` — first 2026-02-05, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/mobile-bringup-v1/` — first 2026-02-11, latest 2026-02-12, 4 markdown docs
- `docs/workstreams/mobile-contracts-v1/` — first 2026-02-12, latest 2026-02-12, 3 markdown docs
- `docs/workstreams/mobile-gfx-backend-v1/` — first 2026-02-12, latest 2026-02-24, 6 markdown docs
- `docs/workstreams/mobile-share-and-clipboard-v1/` — first 2026-02-12, latest 2026-02-12, 3 markdown docs
- `docs/workstreams/motion-foundation-v1/` — first 2026-02-12, latest 2026-02-27, 3 markdown docs
- `docs/workstreams/onboarding-ergonomics-v1/` — first 2026-02-16, latest 2026-03-08, 3 markdown docs
- `docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/` — first 2026-04-11, latest 2026-04-11, 6 markdown docs
- `docs/workstreams/open-source-onboarding-fearless-refactor-v1/` — first 2026-03-04, latest 2026-03-04, 3 markdown docs
- `docs/workstreams/open-source-readiness-fearless-refactor-v1/` — first 2026-03-04, latest 2026-03-12, 4 markdown docs
- `docs/workstreams/overlay-input-arbitration-v2/` — first 2026-01-24, latest 2026-02-11, 3 markdown docs
- `docs/workstreams/paint-eval-space-v1/` — first 2026-02-28, latest 2026-03-02, 3 markdown docs
- `docs/workstreams/path-paint-surface-v1/` — first 2026-02-16, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/path-stroke-style-v2/` — first 2026-02-16, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/perf-baselines/` — first 2026-02-06, latest 2026-02-10, 1 markdown docs
- `docs/workstreams/primitives-interaction-semantics-alignment-v1/` — first 2026-02-09, latest 2026-02-17, 19 markdown docs
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/` — first 2026-04-02, latest 2026-04-15, 6 markdown docs
- `docs/workstreams/quad-border-styles-v1/` — first 2026-02-13, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/query-lifecycle-v1/` — first 2026-02-06, latest 2026-02-11, 2 markdown docs
- `docs/workstreams/release-surface-fearless-refactor-v1/` — first 2026-03-31, latest 2026-04-02, 4 markdown docs
- `docs/workstreams/renderer-clip-mask-closure-v1/` — first 2026-02-17, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/renderer-drop-shadow-effect-v1/` — first 2026-02-17, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/renderer-effect-backdrop-warp-v1/` — first 2026-02-17, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/renderer-effect-backdrop-warp-v2/` — first 2026-02-18, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/` — first 2026-02-25, latest 2026-03-03, 7 markdown docs
- `docs/workstreams/renderer-execute-pass-recorders-modularization-v1/` — first 2026-02-22, latest 2026-02-22, 5 markdown docs
- `docs/workstreams/renderer-modularity-fearless-refactor-v1/` — first 2026-03-12, latest 2026-03-13, 8 markdown docs
- `docs/workstreams/renderer-paint-gpu-storage-unification-v1/` — first 2026-02-16, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/renderer-render-plan-semantics-audit-v1/` — first 2026-02-22, latest 2026-02-23, 3 markdown docs
- `docs/workstreams/renderer-scene-encoding-semantics-audit-v1/` — first 2026-02-23, latest 2026-02-23, 3 markdown docs
- `docs/workstreams/renderer-upstream-semantics-parity-v1/` — first 2026-02-22, latest 2026-02-22, 3 markdown docs
- `docs/workstreams/renderer-vnext-fearless-refactor-v1/` — first 2026-02-14, latest 2026-02-23, 4 markdown docs
- `docs/workstreams/resizable-adaptive-panel-proof-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow closeout lane that promotes ADR 0325's fixed-window panel-resize/container-query proof into the first-party `Resizable` gallery/docs surface)
- `docs/workstreams/resource-loading-fearless-refactor-v1/` — first 2026-03-15, latest 2026-03-16, 7 markdown docs
- `docs/workstreams/resource-loading-release-readiness-fearless-refactor-v1/` — first n/a, latest n/a, 3 markdown docs
- `docs/workstreams/retained-bridge-exit-v1/` — first 2026-02-07, latest 2026-02-08, 2 markdown docs
- `docs/workstreams/router-tanstack-parity-v1/` — first 2026-02-07, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/router-ui-v1/` — first 2026-02-08, latest 2026-03-12, 2 markdown docs
- `docs/workstreams/router-v1/` — first 2026-02-06, latest 2026-03-11, 2 markdown docs
- `docs/workstreams/runtime-safety-hardening-v1/` — first 2026-02-13, latest 2026-02-14, 3 markdown docs
- `docs/workstreams/scroll-extents-dom-parity/` — first 2026-02-01, latest 2026-03-09, 2 markdown docs
- `docs/workstreams/scroll-optimization-v1/` — first 2026-03-02, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow closeout lane that aligns the sidebar app-shell provider/context surface on shared `device_shell_*` vocabulary without reopening panel/container adaptive work)
- `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/` — first 2026-03-20, latest 2026-03-20, 6 markdown docs
- `docs/workstreams/select-combobox-deep-redesign-v1/` — first 2026-03-02, latest 2026-03-03, 3 markdown docs
- `docs/workstreams/shadcn-component-surface-audit-v1/` — first 2026-03-02, latest 2026-03-03, 3 markdown docs
- `docs/workstreams/shadcn-extras/` — first 2026-02-09, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/shadcn-motion-parity-audit-v1/` — first 2026-03-03, latest 2026-03-04, 5 markdown docs
- `docs/workstreams/shadcn-recipe-focus-and-builder-render-closure-v1/` — first 2026-04-01, latest 2026-04-01, 3 markdown docs
- `docs/workstreams/shadcn-part-surface-alignment-v1/` — first 2026-03-01, latest 2026-03-11, 7 markdown docs
- `docs/workstreams/shadcn-semantic-drift-sweep-v1/` — first 2026-02-24, latest 2026-02-26, 3 markdown docs
- `docs/workstreams/shadcn-source-alignment-v1/` — first 2026-03-08, latest 2026-03-08, 3 markdown docs
- `docs/workstreams/shadcn-web-goldens-v4/` — first 2026-01-31, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/shadow-portable-softness-fearless-refactor-v1/` — first 2026-04-01, latest 2026-04-01, 3 markdown docs
- `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/` — first 2026-04-01, latest 2026-04-02, 3 markdown docs
- `docs/workstreams/shadow-surface-fearless-refactor-v1/` — first 2026-04-01, latest 2026-04-01, 3 markdown docs
- `docs/workstreams/shell-composition-fearless-refactor-v1/` — first 2026-04-02, latest 2026-04-02, 4 markdown docs
- `docs/workstreams/shimmer-text-style-source-fearless-refactor-v1/` — first 2026-03-07, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/state-management-v1/` — first 2026-02-05, latest 2026-03-12, 3 markdown docs
- `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-02, 6 markdown docs
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs (closed closeout record for conservative local SVG-directory analysis that scaffolds per-icon `original-colors` overrides and an optional review report without inferring pack-level defaults or changing import behavior)
- `docs/workstreams/text-infrastructure-v1/` — first 2026-02-19, latest 2026-02-22, 2 markdown docs
- `docs/workstreams/text-interactive-spans-v1/` — first 2026-02-19, latest 2026-02-28, 2 markdown docs
- `docs/workstreams/text-intrinsic-sizing-and-wrap-v1/` — first 2026-02-19, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/text-layout-integration-v1/` — first 2026-01-30, latest 2026-02-20, 2 markdown docs
- `docs/workstreams/text-line-breaking-v1/` — first 2026-02-14, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/text-outline-stroke-surface-v1/` — first 2026-02-18, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/text-paint-surface-v1/` — first 2026-02-16, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/text-parley-layout-alignment-v1/` — first 2026-02-20, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/text-parley-unification-v1/` — first 2026-02-20, latest 2026-02-21, 3 markdown docs
- `docs/workstreams/text-shaping-surface-v1/` — first 2026-02-14, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/text-strut-and-leading-distribution-v1/` — first 2026-02-22, latest 2026-02-22, 3 markdown docs
- `docs/workstreams/text-style-cascade-fearless-refactor-v1/` — first 2026-03-07, latest 2026-03-07, 4 markdown docs
- `docs/workstreams/theme-token-alignment-v1/` — first 2026-02-27, latest 2026-02-28, 4 markdown docs
- `docs/workstreams/ui-assets-image-loading-v1/` — first 2026-02-13, latest 2026-02-13, 3 markdown docs
- `docs/workstreams/ui-automation-and-debug-recipes-v1/` — first 2026-01-30, latest 2026-02-24, 2 markdown docs
- `docs/workstreams/ui-diagnostics-inspector-v1/` — first 2026-01-16, latest 2026-03-03, 2 markdown docs
- `docs/workstreams/ui-diagnostics-timebase-decoupling-v1/` — first 2026-03-03, latest 2026-03-07, 4 markdown docs
- `docs/workstreams/ui-direction-and-rtl-fearless-refactor-v1/` — first 2026-03-04, latest 2026-03-04, 3 markdown docs
- `docs/workstreams/ui-editor-v1/` — first 2026-02-14, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-03, 8 markdown docs
- `docs/workstreams/ui-gallery-fearless-refactor/` — first 2026-03-01, latest 2026-03-11, 7 markdown docs
- `docs/workstreams/ui-gallery-view-cache-web-perf-stabilization-v1/` — first 2026-02-23, latest 2026-03-10, 3 markdown docs
- `docs/workstreams/ui-gallery-visual-parity/` — first 2026-02-01, latest 2026-02-24, 2 markdown docs
- `docs/workstreams/ui-launch-modularization-v1/` — first 2026-02-12, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/ui-memory-footprint-closure-v1/` — first 2026-03-04, latest 2026-03-10, 17 markdown docs
- `docs/workstreams/ui-perf-paint-pass-breakdown-v1/` — first 2026-02-05, latest 2026-02-05, 2 markdown docs
- `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1/` — first 2026-02-12, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/ui-perf-zed-smoothness-v1/` — first 2026-02-02, latest 2026-02-24, 4 markdown docs
- `docs/workstreams/ui-typography-presets-v1/` — first 2026-02-22, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/uicx-compat-alias-release-retirement-v1/` — first n/a, latest n/a, 6 markdown docs
- `docs/workstreams/unified-authoring-builder-v1/` — first 2026-01-20, latest 2026-03-12, 2 markdown docs
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/` — first 2026-03-20, latest 2026-03-20, 5 markdown docs
- `docs/workstreams/webview-wry-v1/` — first 2026-02-11, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/workstream-catalog-integrity-gate-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the section-aware gate that keeps workstream directory/file catalog indexes aligned with actual `docs/workstreams` contents and common maintainer gate entrypoints)
- `docs/workstreams/workspace-crate-boundaries-v1/` — first 2026-02-07, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/workspace-shell-tabstrip-fearless-refactor-v1/` — first 2026-02-28, latest 2026-03-05, 6 markdown docs
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/` — first 2026-03-01, latest 2026-03-05, 8 markdown docs
- `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-03, 10 markdown docs

## Standalone Bucket

- `docs/workstreams/standalone/README.md` — first 2026-03-12, latest 2026-03-12, 47 markdown docs
- `docs/workstreams/standalone/workstream-state-v1.md` — shared machine-readable lane-state convention
- Use this folder for compact loose notes that still do not justify a dedicated subdirectory.
