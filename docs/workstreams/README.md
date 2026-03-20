# Workstreams

Catalog updated: 2026-03-20
Directory layout last reorganized: 2026-03-12
Date fields in this index are resolved from git history. For files moved during the 2026-03-12
reorganization, the historical tracked path was followed back to the pre-reorg location.

This directory contains implementation workstreams, refactor trackers, audits, and longer-running
design notes. These documents are **not** the sole source of truth for project priorities. For the
current sequencing and active cross-workstream stance, start with:

- `docs/roadmap.md`
- `docs/workstreams/standalone/ecosystem-status.md`
- `docs/todo-tracker.md`

## Layout Snapshot

- Reorganized into dedicated workstream directories on 2026-03-12.
- Dedicated directories: 168
- Standalone markdown files: 44 (see `docs/workstreams/standalone/README.md`)
- Top-level markdown files in `docs/workstreams/`: `README.md` only

## Promotion Rule

- Keep a workstream in `standalone/` only while it is compact and self-contained.
- Promote it into `docs/workstreams/<slug>/` once it gains a main doc plus companions such as TODOs,
  milestones, parity notes, evidence docs, or audit appendices.
- Use git history, not filesystem mtimes, as the canonical archive date source.

Useful commands:

```bash
git log -1 --format=%cs -- docs/workstreams/<path>
git log --format='%cs %h %s' -- docs/workstreams/<path>
git log --since='2026-01-01' --name-only -- docs/workstreams
```

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

## Directory Index

- `docs/workstreams/a11y-accesskit-xplat-bridge-v1/` — first 2026-02-16, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/a11y-range-semantics-fearless-refactor-v1/` — first 2026-02-23, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/a11y-semantics-closure-v1/` — first 2026-02-23, latest 2026-02-23, 3 markdown docs
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-16, 50 markdown docs
- `docs/workstreams/action-write-surface-fearless-refactor-v1/` — first 2026-03-17, latest 2026-03-17, 8 markdown docs
- `docs/workstreams/ai-elements-port/` — first 2026-02-05, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/animata-recipes-v1/` — first 2026-02-13, latest 2026-02-27, 2 markdown docs
- `docs/workstreams/app-entry-builder-v1/` — first 2026-02-26, latest 2026-03-12, 3 markdown docs
- `docs/workstreams/app-iteration-fast-restart-v1/` — first 2026-02-15, latest 2026-02-15, 3 markdown docs
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/` — first 2026-03-16, latest 2026-03-20, 10 markdown docs
- `docs/workstreams/authoring-ergonomics-fluent-builder/` — first 2026-01-21, latest 2026-03-12, 2 markdown docs
- `docs/workstreams/authoring-paradigm-gpui-style-v1/` — first 2026-02-05, latest 2026-03-06, 2 markdown docs
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/` — first 2026-03-10, latest 2026-03-12, 5 markdown docs
- `docs/workstreams/bottom-up-fearless-refactor-v1/` — first 2026-02-07, latest 2026-03-09, 5 markdown docs
- `docs/workstreams/canvas-world-layer-v1/` — first 2026-02-12, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/carousel-embla-fearless-refactor-v1/` — first 2026-02-26, latest 2026-03-02, 11 markdown docs
- `docs/workstreams/carousel-embla-parity-v1/` — first 2026-02-13, latest 2026-02-27, 3 markdown docs
- `docs/workstreams/carousel-embla-parity-v2/` — first 2026-02-28, latest 2026-03-03, 5 markdown docs
- `docs/workstreams/code-editor-ecosystem-v1/` — first 2026-01-27, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/component-ecosystem-state-integration-v1/` — first 2026-02-06, latest 2026-02-14, 2 markdown docs
- `docs/workstreams/container-queries-v1/` — first 2026-02-09, latest 2026-02-11, 3 markdown docs
- `docs/workstreams/control-chrome-normalization-audit-v1/` — first 2026-02-18, latest 2026-02-19, 3 markdown docs
- `docs/workstreams/control-id-form-association-v1/` — first 2026-03-06, latest 2026-03-08, 3 markdown docs
- `docs/workstreams/crate-audits/` — first 2026-02-08, latest 2026-03-12, 24 markdown docs
- `docs/workstreams/creative-recipes-v1/` — first 2026-02-10, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/current-color-inheritance-fearless-refactor-v1/` — first 2026-02-23, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/` — first n/a, latest n/a, 5 markdown docs
- `docs/workstreams/delinea-engine-contract-closure-v1/` — first 2026-02-09, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/diag-ai-agent-debugging-v1/` — first 2026-02-21, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/diag-architecture-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-06, 20 markdown docs
- `docs/workstreams/diag-bundle-schema-v2/` — first 2026-02-21, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/diag-devtools-gui-v1/` — first 2026-02-07, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/diag-extensibility-and-capabilities-v1/` — first 2026-02-10, latest 2026-02-28, 9 markdown docs
- `docs/workstreams/diag-fearless-refactor-v1/` — first 2026-02-21, latest 2026-03-06, 16 markdown docs
- `docs/workstreams/diag-fearless-refactor-v2/` — first 2026-03-06, latest 2026-03-10, 35 markdown docs
- `docs/workstreams/diag-perf-attribution-v1/` — first 2026-02-14, latest 2026-02-14, 4 markdown docs
- `docs/workstreams/diag-perf-profiling-infra-v1/` — first 2026-02-15, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/diag-simplification-v1/` — first 2026-02-13, latest 2026-03-09, 4 markdown docs
- `docs/workstreams/diag-v2-hardening-and-switches-v1/` — first 2026-02-26, latest 2026-03-03, 10 markdown docs
- `docs/workstreams/docking-arbitration-diag-hardening-v1/` — first 2026-02-28, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/docking-hovered-window-contract-v1/` — first 2026-02-17, latest 2026-02-18, 2 markdown docs
- `docs/workstreams/docking-multiviewport-arbitration-v1/` — first 2026-01-27, latest 2026-03-02, 2 markdown docs
- `docs/workstreams/docking-multiwindow-imgui-parity/` — first 2026-01-27, latest 2026-03-04, 2 markdown docs
- `docs/workstreams/docking-nary-split-graph-v1/` — first 2026-02-11, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/docking-tabbar-fearless-refactor-v1/` — first 2026-02-28, latest 2026-03-05, 9 markdown docs
- `docs/workstreams/ecosystem-integration-traits-v1/` — first 2026-03-11, latest 2026-03-12, 5 markdown docs
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/` — first 2026-03-09, latest 2026-03-10, 7 markdown docs
- `docs/workstreams/editor-tabstrip-unification-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-05, 7 markdown docs
- `docs/workstreams/editor-text-pipeline-v1/` — first 2026-02-14, latest 2026-03-03, 3 markdown docs
- `docs/workstreams/environment-queries-v1/` — first 2026-02-09, latest 2026-03-12, 6 markdown docs
- `docs/workstreams/environment-queries-v1-extensions/` — first n/a, latest n/a, 0 markdown docs
- `docs/workstreams/example-suite-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-12, 9 markdown docs
- `docs/workstreams/external-texture-imports-v1/` — first 2026-02-13, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/external-texture-imports-v2-zero-low-copy/` — first 2026-02-16, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/foreground-inheritance-late-binding-v2/` — first 2026-02-24, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/foreground-style-context-fearless-refactor-v1/` — first 2026-03-06, latest 2026-03-06, 3 markdown docs
- `docs/workstreams/foundation-closure-p0/` — first 2026-01-28, latest 2026-02-11, 2 markdown docs
- `docs/workstreams/font-system-fearless-refactor-v1/` — first 2026-03-13, latest 2026-03-13, 3 markdown docs
- `docs/workstreams/framework-modularity-fearless-refactor-v1/` — first 2026-02-27, latest 2026-02-27, 3 markdown docs
- `docs/workstreams/fret-interaction-kernel-v1/` — first 2026-02-10, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/` — first 2026-03-06, latest 2026-03-12, 7 markdown docs
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/` — first 2026-03-13, latest 2026-03-13, 3 markdown docs
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-06, 3 markdown docs
- `docs/workstreams/fret-node-style-skinning-v1/` — first 2026-02-27, latest 2026-03-01, 7 markdown docs
- `docs/workstreams/fret-node-style-skinning-v2/` — first 2026-03-01, latest 2026-03-01, 3 markdown docs
- `docs/workstreams/fret-node-style-skinning-v3/` — first 2026-03-02, latest 2026-03-02, 6 markdown docs
- `docs/workstreams/genui-json-render-v1/` — first 2026-02-14, latest 2026-03-02, 3 markdown docs
- `docs/workstreams/gesture-recognizers-v1/` — first 2026-02-11, latest 2026-02-11, 3 markdown docs
- `docs/workstreams/gpui-parity-refactor/` — first 2026-01-15, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/headless-table-tanstack-parity/` — first 2026-02-04, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/hotpatch-devloop-alignment-v1/` — first 2026-02-15, latest 2026-03-01, 4 markdown docs
- `docs/workstreams/image-source-view-cache-v1/` — first 2026-02-13, latest 2026-02-13, 3 markdown docs
- `docs/workstreams/image-support-v1/` — first 2026-02-09, latest 2026-02-11, 2 markdown docs
- `docs/workstreams/imui-authoring-facade-v1/` — first 2026-02-03, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/imui-authoring-facade-v2/` — first 2026-02-03, latest 2026-03-02, 2 markdown docs
- `docs/workstreams/imui-ecosystem-facade-v1/` — first 2026-02-05, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/imui-ecosystem-facade-v2/` — first 2026-02-06, latest 2026-02-08, 8 markdown docs
- `docs/workstreams/imui-ecosystem-facade-v3/` — first 2026-02-06, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/input-dispatch-v2/` — first 2026-01-22, latest 2026-02-14, 3 markdown docs
- `docs/workstreams/into-element-surface-fearless-refactor-v1/` — first 2026-03-12, latest 2026-03-12, 5 markdown docs
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
- `docs/workstreams/open-source-onboarding-fearless-refactor-v1/` — first 2026-03-04, latest 2026-03-04, 3 markdown docs
- `docs/workstreams/open-source-readiness-fearless-refactor-v1/` — first 2026-03-04, latest 2026-03-12, 4 markdown docs
- `docs/workstreams/overlay-input-arbitration-v2/` — first 2026-01-24, latest 2026-02-11, 3 markdown docs
- `docs/workstreams/paint-eval-space-v1/` — first 2026-02-28, latest 2026-03-02, 3 markdown docs
- `docs/workstreams/path-paint-surface-v1/` — first 2026-02-16, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/path-stroke-style-v2/` — first 2026-02-16, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/perf-baselines/` — first 2026-02-06, latest 2026-02-10, 1 markdown docs
- `docs/workstreams/primitives-interaction-semantics-alignment-v1/` — first 2026-02-09, latest 2026-02-17, 19 markdown docs
- `docs/workstreams/quad-border-styles-v1/` — first 2026-02-13, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/query-lifecycle-v1/` — first 2026-02-06, latest 2026-02-11, 2 markdown docs
- `docs/workstreams/renderer-clip-mask-closure-v1/` — first 2026-02-17, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/renderer-drop-shadow-effect-v1/` — first 2026-02-17, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/renderer-effect-backdrop-warp-v1/` — first 2026-02-17, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/renderer-effect-backdrop-warp-v2/` — first 2026-02-18, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/` — first 2026-02-25, latest 2026-03-03, 7 markdown docs
- `docs/workstreams/renderer-execute-pass-recorders-modularization-v1/` — first 2026-02-22, latest 2026-02-22, 5 markdown docs
- `docs/workstreams/renderer-paint-gpu-storage-unification-v1/` — first 2026-02-16, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/renderer-render-plan-semantics-audit-v1/` — first 2026-02-22, latest 2026-02-23, 3 markdown docs
- `docs/workstreams/renderer-scene-encoding-semantics-audit-v1/` — first 2026-02-23, latest 2026-02-23, 3 markdown docs
- `docs/workstreams/renderer-upstream-semantics-parity-v1/` — first 2026-02-22, latest 2026-02-22, 3 markdown docs
- `docs/workstreams/renderer-vnext-fearless-refactor-v1/` — first 2026-02-14, latest 2026-02-23, 4 markdown docs
- `docs/workstreams/retained-bridge-exit-v1/` — first 2026-02-07, latest 2026-02-08, 2 markdown docs
- `docs/workstreams/router-tanstack-parity-v1/` — first 2026-02-07, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/router-ui-v1/` — first 2026-02-08, latest 2026-03-12, 2 markdown docs
- `docs/workstreams/router-v1/` — first 2026-02-06, latest 2026-03-11, 2 markdown docs
- `docs/workstreams/runtime-safety-hardening-v1/` — first 2026-02-13, latest 2026-02-14, 3 markdown docs
- `docs/workstreams/scroll-extents-dom-parity/` — first 2026-02-01, latest 2026-03-09, 2 markdown docs
- `docs/workstreams/scroll-optimization-v1/` — first 2026-03-02, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/` — first 2026-03-20, latest 2026-03-20, 6 markdown docs
- `docs/workstreams/select-combobox-deep-redesign-v1/` — first 2026-03-02, latest 2026-03-03, 3 markdown docs
- `docs/workstreams/shadcn-component-surface-audit-v1/` — first 2026-03-02, latest 2026-03-03, 3 markdown docs
- `docs/workstreams/shadcn-extras/` — first 2026-02-09, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/shadcn-motion-parity-audit-v1/` — first 2026-03-03, latest 2026-03-04, 5 markdown docs
- `docs/workstreams/shadcn-part-surface-alignment-v1/` — first 2026-03-01, latest 2026-03-11, 7 markdown docs
- `docs/workstreams/shadcn-semantic-drift-sweep-v1/` — first 2026-02-24, latest 2026-02-26, 3 markdown docs
- `docs/workstreams/shadcn-source-alignment-v1/` — first 2026-03-08, latest 2026-03-08, 3 markdown docs
- `docs/workstreams/shadcn-web-goldens-v4/` — first 2026-01-31, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/shimmer-text-style-source-fearless-refactor-v1/` — first 2026-03-07, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/state-management-v1/` — first 2026-02-05, latest 2026-03-12, 3 markdown docs
- `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-02, 6 markdown docs
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
- `docs/workstreams/unified-authoring-builder-v1/` — first 2026-01-20, latest 2026-03-12, 2 markdown docs
- `docs/workstreams/webview-wry-v1/` — first 2026-02-11, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/workspace-crate-boundaries-v1/` — first 2026-02-07, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/workspace-shell-tabstrip-fearless-refactor-v1/` — first 2026-02-28, latest 2026-03-05, 6 markdown docs
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/` — first 2026-03-01, latest 2026-03-05, 8 markdown docs
- `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-03, 10 markdown docs

## Standalone Bucket

- `docs/workstreams/standalone/README.md` — first 2026-03-12, latest 2026-03-12, 44 markdown docs
- Use this folder for compact loose notes that still do not justify a dedicated subdirectory.
