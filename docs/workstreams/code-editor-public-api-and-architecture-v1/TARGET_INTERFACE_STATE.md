# Target Interface State

Status: Target contract sketch
Last updated: 2026-05-12

This file describes the intended shape of the code editor public surface. Names below are target
roles, not a promise that every candidate type already exists.

## API Stability Buckets

- Stable: types intended for app authors and ecosystem crates to use directly.
- Experimental: types needed by first-party demos or workstreams, but still allowed to change.
- Internal: implementation details that should stay private or be moved behind a narrower public
  adapter.

Each public item in `fret-code-editor`, `fret-code-editor-buffer`, and `fret-code-editor-view`
should eventually be assigned one of these buckets.

## Buffer/Edit Model

Stable target:

- `DocId`, `DocUri`, `Revision`
- `TextBuffer`
- `Edit`, `AppliedEdit`, `BufferDelta`, `LineDelta`
- `TextBufferTransaction`, `TextBufferTx`

Open design questions:

- Whether `Selection` belongs in `fret-code-editor-buffer`, `fret-code-editor-view`, or remains a
  surface-level single-cursor v1 type.
- Whether multi-cursor uses a new collection type or a versioned extension above the current
  single-selection contract.
- How external workspace owners receive edits without coupling to UI handles.

## Display/View Model

Stable or near-stable target:

- `DisplayMap`
- `DisplayPoint`
- `MaterializedDisplayRow`
- `DisplayRowFragment`
- `DisplayRowSpan`
- `FoldSpan`
- `InlaySpan`
- `InlinePreedit`
- `CodeWrapPolicy`

Required convergence:

- folds, inlays, diagnostics, semantic tokens, and preedit must share one buffer/display mapping
  vocabulary,
- soft wrap and fragment composition must remain deterministic under caret, hit-test, a11y, and
  text input queries,
- view-owned materialization should be the default path for composed rows.

## UI Surface and Controller

Current app-facing surface:

- `CodeEditor`
- `CodeEditorHandle`
- `CodeEditorInteractionOptions`
- `CodeEditorCacheStats`
- `CodeEditorPaintPerfFrame`

Target direction:

- Keep `CodeEditor` as the element/widget builder.
- Keep a handle for imperative integration, but audit each method:
  - model mutation,
  - view configuration,
  - interaction policy,
  - diagnostics/perf readout,
  - debug-only hook.
- Prefer grouped option structs or extension inputs over one setter per feature.
- Make debug-only methods visibly debug-only or move them out of the default public surface.

## Feature Extension Inputs

The editor should accept feature data through explicit owner-layer contracts:

- Diagnostics: ranges, severity, source, message, optional code/action ids, and logical-line
  summaries for gutter/overview consumers.
- Decorations: ranges, visual class, z/order, hover target, hit-test policy.
- Gutter markers: line or display-row attachment, icon/text payload, tooltip/action hooks.
- Semantic tokens: token ranges and semantic classes decoupled from paint colors.
- Completion: request context, candidate model, active candidate, commit policy.
- Hover/signature help: request context and overlay payloads without owning overlay dismissal policy.
- Code actions: range/context payload plus command ids.
- Search highlights: range collection and active-match identity.
- Bracket matching: paired ranges and transient highlight policy.
- Multi-cursor: selection collection and edit transaction semantics.

Not every item needs implementation in this lane, but each needs an owner layer before public API
is widened.

## Commands, Keymap, Undo

Target direction:

- Commands should be named and route through Fret's command/action infrastructure.
- Keymap policy should not be hard-coded into paint or buffer code.
- Undo grouping should remain model-level enough to support app-owned histories and editor-local
  histories.
- Read-only and disabled states should gate edits, command availability, IME/preedit, and a11y
  consistently.

## Diagnostics and Performance Hooks

Stable target:

- cache stats suitable for diagnostics bundles,
- paint frame attribution suitable for p50/p95/max baselines,
- renderer payload evidence for text ops and instance bytes,
- visible-window telemetry for windowed-row surfaces,
- explicit feature payload counters when diagnostics/decorations are added.

Rule: a feature surface is not editor-grade until it has at least one repro, one gate, and one
evidence path.
