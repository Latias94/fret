# Iconify Presentation Defaults Report v1

Status: Closed
Last updated: 2026-04-09

Status note (2026-04-09): this lane is now closed on an optional
`--report-out <file>` follow-on for
`fretboard icons suggest presentation-defaults ...`. The helper now writes a versioned advisory
report JSON alongside the emitted `presentation-defaults.json` when requested, keeping derivation
evidence reviewable without changing the shipped generator/import contract. See
`docs/workstreams/iconify-presentation-defaults-report-v1/M2_PROOF_SURFACE_2026-04-09.md` and
`docs/workstreams/iconify-presentation-defaults-report-v1/CLOSEOUT_AUDIT_2026-04-09.md`.

This workstream is a narrow follow-on to the closed
`iconify-presentation-defaults-suggestion-v1` lane. It does not reopen generated-pack
presentation policy, runtime icon rendering ownership, or import-time defaulting.

It owns one narrower question:

> once Fret can emit an explicit advisory `presentation-defaults.json` suggestion from Iconify
> acquisition provenance, how should it also leave a committed review artifact that explains the
> decision and its limits without turning the helper into another import policy path?

## Why this lane exists

The closed suggestion lane already shipped the right helper boundary:

- `icons suggest presentation-defaults` is explicit and file-based,
- it consumes explicit acquisition provenance,
- and it remains advisory because import still requires the emitted config file explicitly.

What it did not preserve is the review context around that file:

- the emitted `presentation-defaults.json` records the decision but not why it was suggested,
- CLI stdout is useful during one run but is not a durable audit artifact,
- and future tooling or code review cannot easily recover the helper's evidence and limitations
  from the config file alone.

This lane therefore targets a second explicit artifact:

- optional, not automatic,
- versioned, not ad-hoc text,
- helper-owned in `fretboard`, not folded into the generator contract,
- and review-oriented rather than normative import policy.

## Starting assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The shipped `presentation-defaults.json` contract is already correct and must stay unchanged here. | Confident | `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`, `crates/fret-icons-generator/src/presentation_defaults.rs` | This lane would accidentally become another generator-contract rewrite. |
| The missing piece is a review artifact, not another defaulting rule. | Confident | `docs/workstreams/iconify-presentation-defaults-suggestion-v1/CLOSEOUT_AUDIT_2026-04-09.md`, `crates/fretboard/src/icons/suggest.rs` | We would blur user-review ergonomics with hidden policy changes. |
| The report belongs in `fretboard`, not `fret-icons-generator`, because it is source-specific helper output over explicit provenance. | Likely | `docs/workstreams/iconify-presentation-defaults-suggestion-v1/DESIGN.md`, `crates/fretboard/src/icons/suggest.rs`, `crates/fret-icons-generator/src/contracts.rs` | We would pull convenience/reporting logic into a stable generator surface. |
| The report should be optional and file-based, so existing suggestion workflows remain unchanged unless users request the extra artifact. | Likely | Current helper surface is explicit and path-based: `crates/fretboard/src/icons/contracts.rs`, `crates/fretboard/src/icons/mod.rs` | We would add surprise side effects or widen the default workflow unnecessarily. |
| The smallest useful proof is one new optional CLI flag plus tests/docs that prove the report stays advisory. | Confident | `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M2_PROOF_SURFACE_2026-04-09.md`, `ecosystem/fret/src/lib.rs` source-policy gates | We could over-scope the follow-on into another large tooling lane. |

## In scope

- One optional `--report-out <file>` flag on `icons suggest presentation-defaults`.
- A versioned review-report JSON artifact owned by `fretboard`.
- Summary/evidence/limitations content derived from the same explicit acquisition provenance.
- Path validation that prevents accidental self-overwrite between provenance, config, and report.
- Public docs and source-policy gates that teach the report as advisory only.

## Out of scope

- Changing `presentation-defaults.json`.
- Making import read provenance or reports implicitly.
- SVG analysis, per-icon override inference, or mixed-pack heuristics.
- Reopening runtime icon rendering or acquisition ownership.

## Target shipped state

### Non-negotiable target

- Omitting `--report-out` preserves the previously shipped behavior.
- The report remains optional, advisory, and helper-owned.
- The report records the derivation evidence and explicit limitations.
- Invalid output-path combinations fail before any file is overwritten.

### Shipped direction

This lane closed on:

- an optional `--report-out` flag,
- a versioned JSON report containing source facts, the suggested pack-level default, and review
  limitations,
- and docs/tests that keep the new artifact firmly outside the generator/import contract.
