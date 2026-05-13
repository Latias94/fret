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
- `Selection`
- `Edit`, `AppliedEdit`, `BufferDelta`, `LineDelta`
- `TextBufferTransaction`, `TextBufferTx`

Open design questions:

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

Coordinate vocabulary:

- buffer UTF-8 byte ranges are the default storage contract for source-backed feature payloads,
- logical lines are for line aggregates and source-stable gutter summaries,
- display points and display rows are `DisplayMap` projections for current-view anchors,
  hit-testing, wrapped-row gutters, a11y, paint, and perf evidence,
- window-space rects are UI/overlay geometry only and must not become model data.

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
- Keep feature payload readouts diagnostic-friendly: the handle exposes a feature payload snapshot
  and derived diagnostic line summaries without exposing the internal store.

## Feature Extension Inputs

The editor should accept feature data through explicit owner-layer contracts:

- Diagnostics: ranges, severity, source, message, optional code/action ids, and logical-line
  summaries for gutter/overview consumers.
- Decorations: buffer ranges, semantic visual class, layer/z-order hints, hover ids, and hit-test
  policy.
- Gutter markers: logical-line or display-row attachment, semantic kind, optional icon/text
  payload, tooltip/action ids, and explicit hit-target intent.
- Semantic tokens: non-empty buffer ranges, semantic classes, and unordered modifiers decoupled
  from paint colors.
- Completion: revision-aware request context, candidate payloads, active candidate identity, and
  commit intent vocabulary without owning listbox, focus, dismissal, or placement policy.
- Hover/signature help: request context, payload ids, and anchor facts without owning overlay
  dismissal, focus, hover-intent, or placement policy.
- Code actions: range/context payload, related diagnostic ids, and command ids without owning the
  menu/popover/lightbulb policy.
- Search highlights: range collection and active-match identity.
- Bracket matching: paired ranges and transient highlight policy.
- Multi-cursor: selection collection and edit transaction semantics.

Not every item needs implementation in this lane, but each needs an owner layer before public API
is widened.

Current v1 surface:

- `CodeEditorHandle` accepts diagnostic spans, range decorations, gutter markers, and semantic
  tokens through public setters.
- Source-backed payloads are normalized against the current `TextBuffer` revision.
- Buffer mutations clear payloads instead of attempting unproven range remapping.
- Display-row gutter markers are validated against `DisplayMap` and pruned when display-map changes
  make them invalid.
- `fret-code-editor-view` exposes revision-aware assist request and payload contracts for
  completion, hover, and code actions:
  - `EditorAssistRequest`,
  - `CompletionList` / `CompletionCandidate`,
  - `HoverPayload`,
  - `CodeActionList` / `CodeAction`.
- These assist contracts carry buffer ranges, display points, ids, active candidate identity,
  command ids, and diagnostic ids as data only. They do not own overlay placement, dismissal,
  focus, listbox navigation, hover intent, or command execution policy.

## Commands, Keymap, Undo

Target direction:

- Baseline text editing intent should use ADR 0044 `text.*` command ids where semantics match.
- Editor-only behaviors should use `editor.*` ids and must be discoverable through command metadata
  when they are menu/palette/keymap facing.
- Keymap policy should route through Fret's command/action infrastructure; widget-local `KeyDown`
  handling is a compatibility fallback, not the long-term extension surface.
- Undo grouping should remain model-level enough to support app-owned document histories and
  editor-local histories.
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
