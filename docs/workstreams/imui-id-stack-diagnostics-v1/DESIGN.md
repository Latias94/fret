# ImUi ID Stack Diagnostics v1

Status: active
Last updated: 2026-04-28

## Why This Lane Exists

`imui-label-identity-ergonomics-v1` closed the Dear ImGui-style label grammar for admitted IMUI
controls. It deliberately deferred runtime ID-stack debugging, ID conflict diagnostics,
localization, and `test_id` inference.

The remaining high-value gap is not another label parser. It is developer feedback when identity
state would drift:

- duplicate keys inside a keyed repeated subtree,
- unkeyed repeated subtrees whose item order changes between frames,
- future stack-path visibility that helps an IMUI author locate the offending callsite.

Dear ImGui makes this class of problem debuggable by showing the ID stack and by making ID
collisions concrete. Fret already has the mechanism substrate (`ElementContext::keyed`,
`named`, `for_each_keyed`, `for_each_unkeyed`, and diagnostics debug paths), but the current
highest-signal warnings are mostly transient tracing output.

## Scope

In scope:

- record structured diagnostics for existing duplicate-key and unkeyed-reorder warnings,
- expose those diagnostics through the existing element runtime diagnostics snapshot,
- prove the behavior through a runtime test and an IMUI authoring test,
- keep `ui.id`, `ui.push_id`, `ui.for_each_keyed`, and `ui.for_each_unkeyed` as the IMUI-facing
  identity vocabulary.

Out of scope:

- public `render_pass_id` / evaluation-token APIs,
- `test_id` inference from label identity,
- localization policy,
- sortable/resizable table column identity,
- a full interactive devtools ID-stack browser,
- replacing the current element identity hashing contract.

## Starting Assumptions

- Area: lane ownership
  - Assumption: this is a narrow follow-on rather than a reopening of the closed label identity
    or table header lanes.
  - Evidence: `docs/workstreams/imui-label-identity-ergonomics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
    and `docs/workstreams/imui-table-header-label-policy-v1/CLOSEOUT_AUDIT_2026-04-28.md`.
  - Confidence: Confident.
  - Consequence if wrong: work could be filed into a closed historical lane and blur closeout
    scope.

- Area: contract boundary
  - Assumption: the first slice should reuse `crates/fret-ui` diagnostics machinery rather than
    inventing an IMUI-only warning registry.
  - Evidence: `crates/fret-ui/src/elements/cx.rs` already owns duplicate-key and unkeyed-reorder
    detection, while `ecosystem/fret-imui/src/frontend.rs` delegates `ui.for_each_unkeyed` to that
    runtime mechanism.
  - Confidence: Confident.
  - Consequence if wrong: move the recorder behind an ecosystem-level adapter, but keep the
    structured evidence shape.

- Area: public API posture
  - Assumption: diagnostics snapshot fields are acceptable for this slice, but public authoring
    identity APIs are not.
  - Evidence: ADR 0319 keeps evaluation tokens internal diagnostics machinery and says diagnostics
    should guide users toward keyed identity.
  - Confidence: Likely.
  - Consequence if wrong: promote a focused ADR update before expanding the snapshot contract.

- Area: IMUI teaching path
  - Assumption: the first actionable diagnostic should point authors from `ui.for_each_unkeyed`
    toward `ui.for_each_keyed` / `ui.id`, not toward lower-level `ElementContext` callsite details.
  - Evidence: `ecosystem/fret-imui/src/frontend.rs` documents that dynamic collections should use
    keyed identity.
  - Confidence: Likely.
  - Consequence if wrong: add a narrower authoring-doc update after the runtime proof lands.

## Target Semantics

- A duplicate key in `ElementContext::for_each_keyed` is still logged in debug builds and is also
  captured as a structured identity diagnostic.
- A changed fingerprint sequence in `ElementContext::for_each_unkeyed` is still logged in debug
  builds and is also captured as a structured identity diagnostic.
- Diagnostics include a source location, the current element path when available, and enough
  hashed identity data to identify the failure without embedding user-visible labels.
- IMUI authors get the same diagnostics because `ui.for_each_unkeyed` delegates to the runtime
  mechanism.

## Exit Criteria

- Runtime diagnostics snapshot includes identity warnings for duplicate keyed-list hashes and
  unkeyed reorder.
- `fret-imui` has a focused proof that `ui.for_each_unkeyed` reorder reaches that structured
  warning path.
- `fret-imui` has a focused proof that `ui.for_each_keyed` duplicate keys reach the same
  structured warning path.
- `fretboard diag query identity-warnings` can inspect captured warning rows without opening raw
  bundle JSON.
- The lane records current gates and leaves future `test_id` inference / full ID-stack browser
  work as explicit follow-ons.
