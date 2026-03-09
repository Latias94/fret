# Layer B Payload Families Audit V1

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/BUNDLE_ARTIFACT_ALIAS_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/RESIDUAL_NAMING_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`

## Purpose

This note scopes the remaining non-`stats/*` Layer B payload families that still mention
`bundle_json` alongside canonical `bundle_artifact`.

The goal is not to trigger another broad rename pass. The goal is to decide whether any small,
low-risk Layer B follow-up still exists before revisiting deferred Layer A manifest chunk-index
contracts.

## Scope

Included:

- orchestrated payload producers outside `crates/fret-diag/src/stats`,
- small in-repo readers that still influence Layer B payload vocabulary,
- residual non-`stats/*` evidence payloads that can still be handled additively.

Excluded:

- Layer A run-manifest chunk-index storage semantics,
- raw chunk directories such as `chunks/bundle_json`,
- internal app-state or raw JSON holder names in DevTools / MCP,
- already-migrated `stats/*` helper adoption work except as baseline context.

## Audit outcome

Main conclusion:

1. non-`stats/*` Layer B review is now mostly complete,
2. `diag_repro`, `diag_repeat`, and `evidence_index` are already aligned with canonical-first
   additive compatibility,
3. the audit initially identified `crates/fret-diag/src/lint.rs` as the only obvious small
   remaining non-`stats/*` producer, and that follow-up is now landed through the shared helper,
4. Layer A chunk-index surfaces should remain deferred until a dedicated contract note says whether
   `bundle_json` is still intentionally format-specific there.

That means the next code slice, if any, should stay small and local. There is no justification yet
for a new manifest-level rename wave.

## Findings by family

### Aligned — `diag_repro`

Key anchors:

- `crates/fret-diag/src/diag_repro.rs:68`
- `crates/fret-diag/src/diag_repro.rs:78`
- `crates/fret-diag/src/diag_repro.rs:850`
- `crates/fret-diag/src/diag_repro.rs:852`

Observations:

- emitted payload rows already write `bundle_artifact` first,
- legacy `bundle_json` remains as an additive alias,
- top-level selected/packed bundle fields also follow canonical-first ordering.

Recommendation:

- Treat this family as aligned for the current compatibility window.
- Only touch it again if the compatibility window itself is being shortened.

### Aligned — `diag_repeat`

Key anchors:

- `crates/fret-diag/src/diag_repeat.rs:11`
- `crates/fret-diag/src/diag_repeat.rs:12`
- `crates/fret-diag/src/diag_repeat.rs:814`
- `crates/fret-diag/src/diag_repeat.rs:815`

Observations:

- reader-side lookup prefers `bundle_artifact`,
- emitted run rows still retain `bundle_json` as the legacy alias,
- tests already cover both legacy read compatibility and canonical preference.

Recommendation:

- Treat this family as aligned for Layer B review purposes.
- Prefer not to spend more rename budget here.

### Aligned — `evidence_index`

Key anchors:

- `crates/fret-diag/src/evidence_index.rs:5`
- `crates/fret-diag/src/evidence_index.rs:7`
- `crates/fret-diag/src/evidence_index.rs:529`
- `crates/fret-diag/src/evidence_index.rs:542`

Observations:

- summary bundle resolution already prefers canonical `selected_bundle_artifact` /
  `packed_bundle_artifact`,
- legacy `selected_bundle_json` / `packed_bundle_json` remains as bounded fallback only,
- tests already pin canonical-first behavior.

Recommendation:

- Treat reader-side Layer B adoption outside `stats/*` as materially complete.

### Landed follow-up — `lint.rs`

Key anchors:

- `crates/fret-diag/src/lint.rs:427`
- `crates/fret-diag/src/lint.rs:428`

Original observation:

- lint payload emission still hand-writes both `bundle_artifact` and `bundle_json` inline,
- this is still a Layer B payload producer rather than a Layer A manifest concern,
- the remaining drift is small enough to address separately without widening scope.

Current state:

- `lint.rs` now uses the same shared alias helper policy as the rest of the additive Layer B
  cleanup.

Recommendation:

- Treat this follow-up as done.
- Do not reopen non-`stats/*` Layer B review unless a new payload family appears.

### Defer — Layer A chunk-index surfaces

Key anchors:

- `crates/fret-diag/src/artifact_lint.rs:474`
- `crates/fret-diag/src/commands/doctor.rs:719`
- `crates/fret-diag/src/run_artifacts.rs:177`
- `crates/fret-diag/src/pack_zip.rs:757`

Observations:

- these hits belong to persisted manifest/chunk-index semantics,
- they represent storage/index structure, not merely Layer B payload vocabulary,
- changing them now would mix an additive payload review with a harder contract decision.

Recommendation:

- Keep these surfaces deferred.
- Revisit only through an explicit Layer A contract note.

### Out of scope — canonical-only or internal naming cases

Examples:

- `crates/fret-diag/src/commands/ai_packet.rs`
- `crates/fret-diag/src/commands/extensions.rs`
- `crates/fret-diag/src/commands/layout_sidecar.rs`
- `crates/fret-diag/src/layout_perf_summary.rs`

Interpretation:

- these are not part of the residual non-`stats/*` Layer B drift set that motivated this audit,
- internal DevTools / MCP raw JSON variable names remain Layer C concerns and should stay deferred.

## Recommended next move

1. Treat this audit as closing the broad non-`stats/*` Layer B review.
2. Treat the non-`stats/*` Layer B follow-up as closed for now.
3. Do not expand this into Layer A chunk-index renaming.
4. Keep any future naming work focused on new evidence that a real consumer contract still drifts.
