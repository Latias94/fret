# Baseline Audit — 2026-04-09

Status: accepted M0 evidence freeze

Related:

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/DESIGN.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/TODO.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/MILESTONES.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `crates/fretboard/src/icons/acquire.rs`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fret-icons-generator/src/presentation_defaults.rs`

## Purpose

Freeze the exact gap before landing any helper:

- generator/import already owns the explicit presentation-defaults contract,
- acquisition already owns the explicit provenance artifact,
- but the repo still lacks a thin bridge that turns provenance into a suggested config file.

## Findings

### 1. The hard contract is already explicit and should not move

Evidence:

- `crates/fret-icons-generator/src/presentation_defaults.rs`
- `crates/fretboard/src/icons/mod.rs`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`

Consequence:

- this lane must not change generator import defaults or add hidden fallback logic.

### 2. Acquisition provenance already records the one v1 hint we can safely reuse

Evidence:

- `crates/fretboard/src/icons/acquire.rs` writes
  `upstream.collection_info.palette` into the provenance sidecar.
- the sidecar is already local and reviewable.

Consequence:

- helper logic can consume provenance without reopening network access or re-fetching metadata.

### 3. No current public helper turns provenance into `presentation-defaults.json`

Evidence:

- `crates/fretboard/src/icons/contracts.rs` currently only exposes `acquire` and `import`.
- `crates/fretboard/src/icons/mod.rs` currently only routes those two command families.

Consequence:

- users must still hand-author a config even when acquisition already captured a strong hint.

### 4. The helper should stay advisory, not normative

Evidence:

- generated-defaults lane explicitly rejected using `palette` as the silent import default:
  `docs/workstreams/generated-icon-presentation-defaults-v1/M1_CONTRACT_FREEZE_2026-04-09.md`.

Consequence:

- the right v1 shape is a separate suggestion command that emits a file;
- the helper should fail when evidence is missing rather than silently widen scope.

## M0 decision from this audit

Treat M0 as closed on these points:

1. keep generator/import defaults explicit and unchanged;
2. consume acquisition provenance only through a separate helper command;
3. use `palette` only as an explicit suggestion input;
4. keep missing-evidence behavior strict in v1.
