# Closeout Audit — 2026-04-09

Status: closed closeout record

Related:

- `docs/workstreams/iconify-acquisition-prestep-v1/DESIGN.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/TODO.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/MILESTONES.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/EVIDENCE_AND_GATES.md`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/acquire.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`

## Verdict

This lane is now closed.

It successfully landed the narrow follow-on that the closed generator lane intentionally deferred:

- an explicit remote acquisition pre-step,
- a separate public `icons acquire ...` command family,
- a generator-compatible local Iconify snapshot artifact,
- a separate provenance sidecar for acquisition facts,
- and proof that the acquired snapshot flows into the existing generator/import path.

## What shipped

### 1) Explicit acquisition stays separate from import

The public surface now distinguishes:

- `fretboard icons acquire iconify-collection ...`
- `fretboard icons import iconify-collection ...`

This preserves the generator lane's “local-input only” contract.

### 2) Subset-first local snapshot workflow

The first shipped workflow is intentionally subset-first:

- users pick one or more explicit icon names,
- acquisition writes a local Iconify-collection-shaped snapshot,
- and the snapshot can be committed, reviewed, and later imported into a pack crate.

This is sufficient for the v1 proof target without prematurely blessing full-collection vendoring
as the default posture.

### 3) Provenance is explicit at acquisition time

Acquisition provenance no longer has to hide inside pack-generation output or transient network
state. The sidecar records deterministic acquisition facts and keeps review-relevant upstream
metadata visible.

### 4) The remaining gaps are follow-ons, not open contract debt

What remains after this lane is narrower than the lane's owned contract:

1. full-collection acquisition mode, if product pressure appears,
2. richer provenance or policy fields, if another concrete consumer demands them,
3. authored-color presentation defaults for imported multicolor icons,
4. or extracting a broader reusable acquisition library if a second consumer appears outside
   `fretboard`.

These are follow-on opportunities, not reasons to keep this lane active.

## Gates that now define the shipped surface

- `cargo nextest run -p fret-icons-generator -p fretboard`
- `cargo check -p fretboard --quiet`

## Follow-on policy

Do not reopen this lane for:

- automatic semantic alias inference,
- moving live fetch into `icons import`,
- or broad icon-pack policy changes unrelated to acquisition.

If future work is needed, open a narrower follow-on such as:

- full-collection acquisition mode,
- richer acquisition provenance consumers,
- or imported-pack presentation defaults for original-color icons.
