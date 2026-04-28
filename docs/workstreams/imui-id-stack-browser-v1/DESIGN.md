# ImUi ID Stack Browser v1

Status: active
Last updated: 2026-04-28

## Why This Lane Exists

`imui-id-stack-diagnostics-v1` closed the structured diagnostics contract for identity footguns:
duplicate keyed-list hashes and unkeyed list reorder drift are now captured, exported in schema2
bundles, and queryable through `diag query identity-warnings`.

That is enough for scripted triage, but it is still not the Dear ImGui-style debugging experience
authors expect when identity state goes wrong. An author should be able to browse identity warnings
as a navigable debugging artifact: window, frame, source location, element path, list id, key hash,
indices, and nearby repeated observations should be visible without opening raw bundle JSON.

## Scope

In scope:

- define a browser-ready identity diagnostics view model over captured schema2 bundle data,
- group and filter identity warnings by window, frame, warning kind, element path, list id, key hash,
  and source file,
- provide a first diagnostics surface that is useful in `fretboard diag` workflows,
- keep the surface anchored to existing structured diagnostics records and bundle snapshots,
- document any truly missing runtime fields before adding capture-side data.

Out of scope:

- public render-pass or evaluation-token APIs,
- label-to-`test_id` inference,
- localization policy,
- sortable/resizable table column identity,
- changing the element identity hashing contract,
- making live connected devtools mandatory for the first slice.

## Starting Assumptions

- Area: lane ownership
  - Assumption: this is a new follow-on, not a reopening of `imui-id-stack-diagnostics-v1`.
  - Evidence: `docs/workstreams/imui-id-stack-diagnostics-v1/CLOSEOUT_AUDIT_2026-04-28.md`.
  - Confidence: Confident.
  - Consequence if wrong: new interactive-browser scope would blur the closed structured
    diagnostics contract.

- Area: data source
  - Assumption: the first useful browser should consume existing schema2 bundle snapshots and
    `debug.element_runtime.identity_warnings` before adding new runtime capture fields.
  - Evidence: `fretboard diag query identity-warnings` already proves the source data is queryable.
  - Confidence: Likely.
  - Consequence if wrong: add a narrow capture-side slice with a clear before/after bundle fixture.

- Area: user journey
  - Assumption: the first workflow is post-run triage, not live mutation of runtime state.
  - Evidence: current diagnostics tooling is bundle/query oriented, and the identity warnings are
    captured in exported snapshots.
  - Confidence: Likely.
  - Consequence if wrong: split live devtools transport into a separate follow-on after the bundle
    browser model is stable.

- Area: policy boundary
  - Assumption: the browser should explain existing identity problems, not infer stable public
    labels or `test_id`s.
  - Evidence: ADR 0319 keeps diagnostics guidance separate from public authoring identity APIs.
  - Confidence: Confident.
  - Consequence if wrong: start a `test_id` inference lane with its own contract review.

## Target Semantics

- Identity warnings can be browsed as a bounded, grouped list rather than raw JSON rows.
- A selected warning shows:
  - window and snapshot anchors,
  - warning kind,
  - element id and element path,
  - list id,
  - key hash and duplicate indices when available,
  - previous/next list lengths when available,
  - source file/line/column.
- Repeated observations across snapshots are visible as timeline/context, not accidental noise.
- The browser can be driven by deterministic fixtures and `fret-diag` tests.

## Exit Criteria

- A browser-ready identity diagnostics model exists with focused Rust tests.
- At least one `fretboard diag` entry point exposes that model without requiring raw bundle JSON.
- The surface has bounded output and stable filters suitable for maintainer triage.
- Any future live UI/devtools panel work has an explicit follow-on boundary.
