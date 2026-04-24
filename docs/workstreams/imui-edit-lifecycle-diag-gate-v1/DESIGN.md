# ImUi Edit Lifecycle Diag Gate v1

Status: closed follow-on
Last updated: 2026-04-24

## Scope

This narrow follow-on turns the existing IMUI edit lifecycle proof into promoted diagnostics
coverage.

It follows the closed `imui-response-status-lifecycle-v1` lane. That lane landed the first
private `ResponseExt` lifecycle vocabulary; this lane only adds repeatable gate evidence and fixes
proof-surface drift found while making the gate reliable.

## Ownership

- `imui_response_signals_demo` remains the smallest proof/contract surface for response lifecycle
  counters.
- `imui_editor_proof_demo` remains the wider editor-grade proof surface for drag, text, numeric,
  and authoring parity outcomes.
- Diag scripts and suite manifests live under `tools/diag-scripts/`.
- No public `fret-imui`, shared `fret-ui-kit::imui`, `fret-authoring::Response`, or
  `crates/fret-ui` runtime API is widened.

## Target Invariant

The gate must prove actual edit lifecycle outcomes, not just control existence:

- slider pointer editing fires the `after_edit` lifecycle edge once,
- text edit plus blur reports activation, deactivation, and after-edit counters,
- editor-proof DragValue outcomes match the current dense preset,
- text/password outcome readouts use state language (`Committed` / `Canceled`),
- authoring parity controls preserve presentation chrome when `from_presentation(...)` is combined
  with explicit demo-local identity/test selectors.

## Out Of Scope

- Public lifecycle API changes.
- Shared inspector/lifecycle helpers.
- Runtime response contract changes.
- Exhaustive automation for every IMUI widget family.
- Compatibility fallbacks for obsolete proof-demo script expectations.
