# ImUi Collection Helper Readiness v1 - Design

Status: active narrow audit lane
Last updated: 2026-04-24

## Problem

The closed collection second proof-surface lane proved that a second real collection surface now
exists, but it also closed on a no-helper-widening verdict: the collection-first asset-browser grid
and the smaller shell-mounted `Scene collection` outline do not yet show the same reusable helper
shape.

This lane owns the next narrower question: can fresh first-party evidence name an exact shared
collection helper that both proof surfaces need, without moving app/product policy into generic
IMUI?

## Scope

Owned here:

1. Compare the two first-party collection proof surfaces from a helper-readiness perspective.
2. Name candidate helper seams only when both surfaces need the same shape.
3. Reject candidate seams that are actually app policy, command policy, selection policy, or recipe
   composition.
4. Leave a gate package that keeps the audit decision source-policy visible.

Not owned here:

1. No `fret-imui` facade widening.
2. No `fret-ui-kit::imui` public helper implementation until the audit names an exact helper.
3. No `crates/fret-ui` runtime/mechanism change.
4. No new dedicated asset-grid/file-browser demo.
5. No reopening of `imui-collection-second-proof-surface-v1`.

## Assumptions

1. The second proof-surface closeout is authoritative.
   Evidence: `docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md`.
   Confidence: Confident.
   Consequence if wrong: this lane would duplicate or reopen a closed folder instead of acting as a
   narrow follow-on.

2. The minimum helper-widening bar is still the proof-budget rule.
   Evidence: `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`.
   Confidence: Confident.
   Consequence if wrong: one proof surface could justify generic IMUI API growth too early.

3. The current candidate surfaces are intentionally different.
   Evidence: `apps/fret-examples/src/imui_editor_proof_demo/collection.rs` carries the dense
   collection-first grid, while `apps/fret-examples/src/editor_notes_demo.rs` carries the compact
   shell-mounted `Scene collection` outline.
   Confidence: Confident.
   Consequence if wrong: the audit may miss a shared primitive that is already obvious.

4. A helper-readiness lane should start with documentation and source-policy proof, not with API
   implementation.
   Evidence: ADR 0066 keeps `fret-ui` as mechanism/contract and previous collection lanes kept
   product policy app-owned.
   Confidence: Likely.
   Consequence if wrong: implementation may be delayed one slice, but avoids committing to the
   wrong public helper.

## Decision Rule

This lane may propose shared helper growth only if a candidate satisfies all criteria:

1. both proof surfaces need the same helper shape,
2. the helper is policy-light enough for `fret-ui-kit::imui`,
3. app-owned command semantics, selection semantics, row content, and layout policy remain outside
   the helper,
4. a focused source-policy gate names the helper boundary,
5. and at least one surface/unit gate proves the shared contract without relying on a new demo.

If no candidate passes, close this lane as another no-helper-widening verdict and continue
product-depth work in app-owned surfaces.
