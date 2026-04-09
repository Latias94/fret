# Iconify Acquisition Pre-step v1

Status: Active
Last updated: 2026-04-09

This workstream is a narrow follow-on to the closed
`iconify-import-pack-generator-v1` lane. It does not reopen the shipped v1 generator contract, the
generated pack-crate shape, or the explicit semantic alias policy that already closed there.

It owns one narrower question:

> if Fret wants a convenience path for acquiring Iconify data from remote upstreams, how should
> that happen as an explicit, pinned, auditable pre-step that produces local artifacts for the
> already-shipped generator, rather than hiding network state inside `icons import`?

## Why this lane exists

The closed generator lane already shipped the correct producer boundary:

- `fret-icons-generator` consumes local SVG inventories and local Iconify collection snapshots,
- `fretboard icons import ...` emits a real pack crate,
- semantic `ui.*` alias policy is explicit and versioned,
- and generated packs target the current app/bootstrap install contract directly.

That lane also intentionally refused to bless live network fetch as part of the stable generator
surface. This leaves one narrower follow-on:

- how a remote acquisition step should produce pinned local Iconify artifacts without changing the
  generator's local-input contract.

This is a different problem from pack emission, so it should live in a separate lane.

## Starting assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The generator lane is correctly closed and should stay closed. | Confident | `docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md`, `docs/workstreams/iconify-import-pack-generator-v1/WORKSTREAM.json` | This follow-on would blur ownership and reopen a closed producer lane. |
| Any network step must remain visibly separate from pack generation. | Confident | `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md` | We would accidentally hide nondeterministic fetch state inside `icons import`. |
| The first useful output of an acquisition step is still a local, repo-committable Iconify collection snapshot JSON file. | Likely | `crates/fret-icons-generator/src/iconify.rs`, `docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md` | We may need a different artifact shape or a two-file acquisition result instead of one snapshot file. |
| The smallest protection while exploring acquisition remains the current generator proof gate. | Confident | `docs/workstreams/iconify-import-pack-generator-v1/EVIDENCE_AND_GATES.md`, `crates/fret-icons-generator/src/lib.rs`, `crates/fretboard/src/icons/mod.rs` | We would lack a stable regression baseline for follow-on work. |
| `repo-ref/dioxus-iconify` is a useful workflow reference for acquisition ergonomics, but not a normative source for Fret command/API shape. | Likely | `repo-ref/dioxus-iconify/README.md`, `repo-ref/dioxus-iconify/src/api.rs`, `docs/repo-ref.md` | We could either ignore a helpful precedent or overfit to another framework's runtime assumptions. |

## In scope

- Freeze the boundary for a remote acquisition pre-step that stays separate from pack generation.
- Decide what pinned local artifact(s) the pre-step should emit.
- Decide where the public acquisition surface should live.
- Leave one smallest repro/gate/evidence set for the first acquisition proof slice.

## Out of scope

- Reopening the generator output contract.
- Reintroducing live fetch into `fretboard icons import ...`.
- Adding a runtime Iconify HTTP client to framework crates.
- Auto-inferring semantic `ui.*` aliases.
- Redesigning `IconPackImportModel`, `PACK_METADATA`, or the current pack install seams.

## Target shipped state

### Non-negotiable target

- A developer can run an explicit acquisition step that produces pinned local Iconify artifact(s).
- The acquisition result is reviewable and repo-committable.
- The existing generator can consume that local artifact without changing its local-input contract.
- Network state, source provenance, and pinning details stay explicit rather than hidden in caches.

### Likely direction

The most plausible v1 direction is:

- a thin public acquisition command under `fretboard`,
- backed by a reusable helper/library surface if the proof demands it,
- producing a local Iconify collection snapshot JSON file plus explicit provenance metadata when
  needed,
- and keeping pack generation as a second, explicit step.

This lane exists to verify that direction before code starts drifting.
