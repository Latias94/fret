# M2 Proof Surface — 2026-04-09

Status: accepted proof slice

Related:

- `docs/workstreams/iconify-acquisition-prestep-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/TODO.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/MILESTONES.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/EVIDENCE_AND_GATES.md`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/acquire.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/lib.rs`

## What landed

### 1) Separate public acquisition CLI surface

`fretboard` now exposes a separate acquisition family:

```bash
fretboard icons acquire iconify-collection \
  --collection mdi \
  --icon home \
  --out ./iconify/mdi-home.json
```

This remains intentionally separate from `fretboard icons import ...`.

### 2) Generator-compatible local snapshot artifact

The acquisition command now writes a local Iconify-collection-shaped JSON snapshot that the
existing generator can consume without contract changes.

Current proof posture:

- subset-first acquisition,
- one or more explicit `--icon <NAME>` selections,
- local JSON output under `--out`,
- no hidden tool cache required for the generator step.

### 3) Explicit provenance sidecar

Acquisition also writes a provenance sidecar JSON file (defaulting to a sibling
`<snapshot>.provenance.json`) that records:

- API base URL,
- collection metadata URL,
- icon subset URL,
- requested icon set,
- upstream collection metadata when available,
- and a digest of the emitted snapshot artifact.

### 4) End-to-end proof into the existing import path

The proof does not stop at writing files. The new acquisition tests prove that an acquired local
snapshot can flow into the existing repo import path and generate a real pack crate without manual
cleanup.

## Proof anchors

- CLI contract / help:
  - `crates/fretboard/src/icons/contracts.rs`
  - `crates/fretboard/src/cli/contracts.rs`
  - `crates/fretboard/src/cli/help.rs`
- Acquisition implementation:
  - `crates/fretboard/src/icons/acquire.rs`
- Existing generator/import path kept intact:
  - `crates/fretboard/src/icons/mod.rs`
  - `crates/fret-icons-generator/src/contracts.rs`
  - `crates/fret-icons-generator/src/lib.rs`

## Validation used

- `cargo nextest run -p fret-icons-generator -p fretboard`
- `cargo check -p fretboard --quiet`

Important proof tests:

- `fretboard icons::acquire::tests::acquire_iconify_collection_writes_subset_snapshot_and_provenance`
- `fretboard icons::acquire::tests::acquired_snapshot_flows_into_existing_repo_import_path`
- `fretboard cli::contracts::tests::root_contract_parses_icons_acquire_iconify_collection_subcommand`
- `fretboard cli::help::tests::root_help_mentions_public_commands`

## Deliberate non-goals of this proof

- full-collection acquisition mode,
- automatic semantic alias inference,
- authored-color presentation defaults for imported multicolor icons,
- or moving acquisition into `fret-icons-generator`.
