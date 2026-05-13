# UI Frame Pipeline v2 Fearless Refactor

Status: Active
Last updated: 2026-05-13

## Problem

Fret's current UI runtime can now diagnose and optimize editor-grade surfaces, but the execution
model is still too indirect:

- declarative element build, retained `UiTree` mounting, view-cache reuse, layout containment,
  prepaint-like staging, paint-cache replay, and editor row-scene replay are not one unified model;
- local changes can still amplify into unrelated shell layout or paint work;
- diagnostics report useful counters, but they are not yet centered on stable execution boundaries;
- and old cache/containment paths can accumulate unless deletion is part of the design.

The 2026-05-13 macOS code-editor resize slice proved the direction:

- the script became deterministic,
- `code_editor.paint_perf` became visible,
- code-editor page layout containment reduced p95 total by roughly 34% and layout solve by roughly
  85%,
- and the remaining bottleneck moved to paint/widget row replay and content resolution.

The next step should not be another one-off knob. It should be a frame-pipeline refactor.

## Scope

This lane owns the migration to the contract proposed in
`docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`.

In scope:

- `crates/fret-ui` runtime internals:
  - dirty propagation,
  - view/cache boundary state,
  - layout containment,
  - prepaint phase ownership,
  - paint-cache / scene-fragment replay,
  - diagnostics counters.
- First vertical slice:
  - UI Gallery code-editor content boundary,
  - `ecosystem/fret-code-editor` row paint surface,
  - code-editor resize and paint perf gates.
- Cleanup:
  - remove or retire old internal paths after each migrated slice,
  - delete redundant knobs that the v2 boundary contract replaces.

Out of scope:

- Linux-specific perf closure.
- Replacing Fret with GPUI/Zed code.
- Moving component policy into `crates/fret-ui`.
- Rewriting the renderer contract or `Scene` ordering semantics.

## Target Model

The target frame pipeline is:

```text
schedule / dirty propagation
  -> build
  -> request layout
  -> layout
  -> prepaint
  -> paint
  -> renderer prepare / encode / upload / present
```

The target runtime unit is a `ViewBoundary`.

A boundary should answer:

- did my declarative output need rebuilding?
- did my layout dependency key change?
- did my geometry-derived prepaint state change?
- can my scene fragment be replayed by translation/transform?
- did hit-testing or semantics need refresh?
- why was reuse rejected?

## Ownership

Mechanism ownership:

- `crates/fret-ui`
  - boundary identity,
  - dirty propagation,
  - layout/prepaint/paint phase contracts,
  - cache/replay mechanics,
  - runtime diagnostics.

Policy ownership:

- `ecosystem/fret-ui-kit`, `ecosystem/fret-ui-shadcn`, `ecosystem/fret-docking`
  - component behavior,
  - dismissal/focus policy,
  - default recipes,
  - recipe-level boundary hints.

App/exemplar ownership:

- `apps/fret-ui-gallery`
  - first-party teaching surface,
  - perf stress pages,
  - reproduction scripts.

## Fearless Refactor Rules

1. Internal compatibility is not a goal.
2. Each slice must name the old path it replaces.
3. A slice is not done until the old path is deleted, narrowed, or documented as a temporary
   migration shim with a deletion milestone.
4. Public app-facing churn is allowed only with a migration note and updated examples.
5. Perf claims require a diag perf run and worst-bundle attribution.
6. Correctness claims require focused unit tests, diag scripts, or both.
7. Layer boundaries must stay green with `python3 tools/check_layering.py`.

## First Vertical Slice

Start with code-editor resize/paint because it exercises the full pressure surface:

- nested app shell,
- view-cache content boundary,
- resize-driven layout containment,
- prepaint-like editor frame state,
- row-scene replay,
- text-heavy paint,
- renderer payload counters.

The first slice should target the current paint-dominant post-layout state:

- `paint.widget` p95/max,
- `code_editor.paint_perf.us_total`,
- row content resolution,
- row scene fast path / replay touch / replay ops.

## Completion Criteria

This lane is complete only when:

- ADR 0327 is accepted or superseded by an accepted equivalent.
- `ViewBoundary` or its final named equivalent is the canonical runtime boundary.
- code-editor resize/paint uses the v2 pipeline without relying on the old ad hoc containment path.
- at least one stricter editor paint stressor exists if `ui-code-editor-resize-probes` is no longer
  sufficient to catch regressions.
- old replaced paths are removed or explicitly marked historical with a deletion audit.
- perf gates prove the target improvement and remain reproducible.
