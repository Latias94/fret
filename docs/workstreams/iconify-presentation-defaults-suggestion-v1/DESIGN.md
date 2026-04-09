# Iconify Presentation Defaults Suggestion v1

Status: Closed
Last updated: 2026-04-09

Status note (2026-04-09): this lane is now closed on a thin
`fretboard icons suggest presentation-defaults ...` helper that consumes Iconify acquisition
provenance and emits an explicit versioned `presentation-defaults.json` suggestion without
changing the shipped generator/import contract. See
`docs/workstreams/iconify-presentation-defaults-suggestion-v1/M2_PROOF_SURFACE_2026-04-09.md`
and
`docs/workstreams/iconify-presentation-defaults-suggestion-v1/CLOSEOUT_AUDIT_2026-04-09.md`.

This workstream is a narrow follow-on to the closed
`generated-icon-presentation-defaults-v1` lane. It does not reopen the explicit generated-pack
presentation contract, runtime icon rendering ownership, or the explicit acquisition pre-step.

It owns one narrower question:

> if Fret already has explicit acquisition provenance and an explicit `presentation-defaults.json`
> contract, how should it offer a thin helper that suggests a starter config from acquisition
> evidence without turning that evidence into a hidden import default?

## Why this lane exists

The closed generated-pack presentation-defaults lane already shipped the correct hard contract:

- generated/imported packs use explicit versioned `presentation-defaults.json`,
- generator/import code carries `IconRenderMode` through to generated registration/provenance,
- and runtime widgets only honor explicit `IconPresentation`.

The closed acquisition lane also already shipped a second relevant fact:

- `fretboard icons acquire iconify-collection ...` writes explicit provenance, including
  `upstream.collection_info.palette` when the upstream metadata provides it.

That leaves one narrower convenience problem:

- how to reuse explicit acquisition provenance as a suggestion input,
- without making `palette` the normative generator default,
- and without pushing presentation heuristics back into runtime or import internals.

## Starting assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The explicit `presentation-defaults.json` contract is already correctly closed and should stay unchanged here. | Confident | `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`, `crates/fret-icons-generator/src/presentation_defaults.rs`, `crates/fretboard/src/icons/mod.rs` | This lane would accidentally become another generator-contract rewrite. |
| Acquisition provenance is the right helper input because it is already explicit, local, and reviewable. | Confident | `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`, `crates/fretboard/src/icons/acquire.rs` | We would either duplicate acquisition logic or reintroduce hidden network coupling. |
| The helper belongs in `fretboard`, not in `fret-icons-generator`, because it is convenience logic over one provenance format rather than a stable generator contract. | Likely | `docs/workstreams/generated-icon-presentation-defaults-v1/M1_CONTRACT_FREEZE_2026-04-09.md`, `crates/fretboard/src/icons/acquire.rs`, `crates/fret-icons-generator/src/contracts.rs` | We would blur generator mechanism with source-specific helper policy. |
| v1 should only derive a pack-level default when provenance provides an explicit `palette` hint; missing evidence should fail rather than guess. | Likely | Iconify `palette` is collection-level metadata already recorded in acquisition provenance: `crates/fretboard/src/icons/acquire.rs`; generated-default lane explicitly rejected silent heuristics as the normative default: `docs/workstreams/generated-icon-presentation-defaults-v1/M1_CONTRACT_FREEZE_2026-04-09.md` | We would trade a narrow helper for another hidden or weakly justified defaulting path. |
| The smallest useful proof is an end-to-end CLI chain: provenance -> suggestion JSON -> existing import path. | Confident | `crates/fretboard/src/icons/mod.rs`, `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md` | We might leave the helper unproven relative to the real import workflow. |

## In scope

- A thin `fretboard icons suggest presentation-defaults ...` helper surface.
- Iconify acquisition provenance as the v1 helper input.
- Explicit emission of a versioned `presentation-defaults.json` suggestion.
- One smallest repro/gate/evidence set proving the suggestion flows into the existing import path.

## Out of scope

- Changing `fret-icons-generator`'s explicit config contract.
- Making `palette` the hidden defaulting rule for `icons import`.
- SVG-content heuristics or mixed-pack inference.
- Reopening runtime icon rendering or acquisition ownership.

## Target shipped state

### Non-negotiable target

- The helper remains explicit and file-based.
- The generated/import path still requires an explicit config file to opt into derived defaults.
- Missing provenance evidence fails loudly instead of guessing.
- The helper can be deleted or replaced later without changing the generator contract.

### Shipped direction

This lane closed on:

- a new `icons suggest presentation-defaults` CLI branch,
- `iconify-collection` acquisition provenance as the only v1 input,
- `palette=true -> original-colors` and `palette=false -> mask` as suggestion-only output,
- and erroring when `palette` is unavailable instead of inventing broader inference.
