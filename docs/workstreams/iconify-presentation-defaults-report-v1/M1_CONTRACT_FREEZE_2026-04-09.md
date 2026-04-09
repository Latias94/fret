# M1 Contract Freeze — 2026-04-09

Status: accepted freeze

Related:

- `docs/workstreams/iconify-presentation-defaults-report-v1/DESIGN.md`
- `docs/workstreams/iconify-presentation-defaults-report-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/suggest.rs`

## Frozen decisions

### 1. The new public surface is one optional flag

The only new CLI surface is:

- `fretboard icons suggest presentation-defaults ... --report-out <file>`

No new subcommand or generator flag is introduced.

### 2. The report is advisory and helper-owned

The report is written only when explicitly requested, and it stays owned by the `fretboard`
helper. Import still consumes only `presentation-defaults.json`.

### 3. The report gets its own versioned contract

The report is a versioned JSON artifact that records:

- the provenance source facts,
- the suggested pack-level default,
- and the helper limitations/review notes.

This contract is separate from `presentation-defaults.json`.

### 4. Path validation happens before writes

The helper must reject conflicting provenance/config/report paths before writing any file.

## Rejected alternatives

- auto-emitting a report for every suggestion run,
- extending `presentation-defaults.json` with helper rationale,
- making import read the report implicitly,
- or moving report logic into `fret-icons-generator`.
