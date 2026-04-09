# Iconify Acquisition Pre-step v1 — Evidence and Gates

Status: Active
Last updated: 2026-04-09

## Smallest current repro

Use this sequence before changing code:

```bash
cargo nextest run -p fret-icons-generator -p fretboard
cargo check -p fretboard --quiet
```

What this proves today:

- the closed generator lane remains the stable baseline,
- `fretboard` still exposes the current local-input producer story,
- and any acquisition follow-on must preserve compatibility with the shipped generator/import path.

## Current evidence set

- `docs/workstreams/iconify-acquisition-prestep-v1/BASELINE_AUDIT_2026-04-09.md` freezes the M0
  baseline:
  - core multicolor support already exists,
  - generated Iconify imports preserve multicolor SVG bytes but still default to mask-mode
    presentation,
  - third-party local-input pack generation is already a public library + CLI surface,
  - and the remaining contract debt is explicit remote acquisition plus provenance.
- `docs/workstreams/iconify-acquisition-prestep-v1/M1_CONTRACT_FREEZE_2026-04-09.md` freezes the
  v1 acquisition contract:
  - generator-compatible local snapshot,
  - separate provenance sidecar,
  - separate `icons acquire ...` public CLI family,
  - and subset snapshot as the first proof target.
- `iconify-import-pack-generator-v1` is now closed on the v1 local-input producer contract.
- The current generator accepts:
  - local SVG directory input,
  - local Iconify collection snapshot input,
  - explicit semantic alias config.
- The closed lane explicitly requires any future remote acquisition convenience to live as a
  separate pre-step/follow-on rather than inside `icons import`.
- `repo-ref/dioxus-iconify` exists as a local, optional reference for fetch/acquisition ergonomics
  only.

## Gate set

### Generator/import baseline

```bash
cargo nextest run -p fret-icons-generator -p fretboard
```

### Public CLI baseline

```bash
cargo check -p fretboard --quiet
```

## Evidence anchors

- `docs/workstreams/iconify-acquisition-prestep-v1/DESIGN.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/TODO.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/MILESTONES.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fretboard/src/icons/mod.rs`
- `repo-ref/dioxus-iconify/README.md`
- `repo-ref/dioxus-iconify/src/api.rs`

## Reference posture

- Optional local reference:
  - `repo-ref/dioxus-iconify/README.md`
  - `repo-ref/dioxus-iconify/src/api.rs`
- Read them for acquisition-workflow ideas only.
- Do not treat `repo-ref/` contents as dependencies or as the source of truth for Fret's public
  API shape.
