# M1 Contract Freeze — 2026-04-09

Status: accepted v1 decision

Related:

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/DESIGN.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/TODO.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/MILESTONES.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/acquire.rs`
- `crates/fret-icons-generator/src/presentation_defaults.rs`

## Purpose

Freeze the smallest correct v1 command/API contract for provenance-driven presentation-defaults
suggestion before implementation drifts.

## Frozen decisions

### 1) The helper lives in `fretboard icons suggest ...`

Decision:

- add a separate `icons suggest presentation-defaults ...` branch,
- do not add this behavior implicitly to `icons acquire` or `icons import`.

Why:

- that keeps acquisition, suggestion, and import as explicit stages;
- and it preserves the closed ownership of the acquisition and generator lanes.

### 2) The helper emits the existing versioned config file shape

Decision:

- output is still `presentation-defaults.json` in the same v1 schema already consumed by
  `fret-icons-generator`.

Why:

- this keeps the helper deletable and optional;
- and it avoids creating a second config format for the same generator contract.

### 3) The only v1 derivation input is Iconify acquisition provenance with explicit `palette`

Decision:

- v1 reads acquisition provenance written by
  `fretboard icons acquire iconify-collection ...`,
- supports only `acquisition_kind = "iconify-collection"`,
- and only derives a pack-level default when `upstream.collection_info.palette` is present.

Why:

- this is the narrowest strong-evidence path already available in-repo;
- and it avoids pretending that broader or mixed-source inference is solved.

### 4) Missing evidence fails loudly

Decision:

- if `palette` is absent, the helper errors instead of guessing or writing a misleading file.

Why:

- correctness matters more than convenience here;
- and explicit failure is better than silently creating a wrong pack-wide default.

## Immediate consequences

From this point forward:

1. the helper is a thin `fretboard` convenience layer, not a generator contract change;
2. the helper remains suggestion-only because import still requires the emitted file explicitly;
3. broader inference, SVG analysis, or mixed-pack support are follow-ons rather than hidden v1
   behavior.
