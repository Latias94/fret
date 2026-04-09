# Example Suite v1 — Catalog Single Source of Truth

This appendix proposes how we prevent drift between:

- docs tables,
- native demo binaries,
- web demo selection,
- `fretboard-dev list ...` discovery surfaces.

## Problem: duplicated registries drift

Today, some demo/executable lists exist in multiple places (native vs web vs tooling).
This makes it easy to:

- forget to add/remove a demo in one place,
- create mismatched IDs between native/web,
- accidentally hide a user-facing example behind “maintainer noise”.

## v1 direction: one catalog surface, multiple views

We want a single catalog that can drive:

- CLI discovery (`fretboard-dev list ...`, `--choose`)
- docs tables (or at least a single authoritative mapping doc)
- optional CI lint (duplicate IDs, missing owner, missing gates)

## Implementation options

### Option A — Directory scan only (fast start)

Source(s):

- native: scan `apps/fret-demo/src/bin/*.rs` (already done for `fretboard-dev list native-demos`)
- cookbook: scan `<cookbook-crate>/examples/*.rs`
- suites: scan `tools/diag-scripts/suites/` to infer “gated + curated” scripts
- web: keep a curated list in one place (temporary)

Pros:

- minimal up-front work.

Cons:

- metadata (level/track/web tier/owner) is not embedded.

### Option B — Manifest file (recommended once the catalog stabilizes)

Add a repo-owned manifest, e.g. `tools/examples/catalog.json`:

- `id`, `title`, `track`, `level`
- `platforms`: `native`/`web` tiers
- `run`: command templates
- `anchors`: file paths (evidence)
- `gates`: suite names and/or canonical script paths
- `owner`: crate/layer responsibility

Then:

- `fretboard` reads the manifest to render discovery views and to validate `--demo`/`--example`.
- CI runs a linter:
  - duplicate IDs
  - missing gate for “official”
  - web tier mismatch
  - broken anchors

Pros:

- explicit metadata + drift prevention.

Cons:

- requires process discipline (but that is the point).

### Option C — Rust registry module (type-safe, heavier)

Implement the manifest as Rust structs in `apps/fretboard`.

Pros:

- compile-time refactors.

Cons:

- makes “edit the catalog” a compile step and increases friction for doc-driven changes.

## Recommended sequencing

1) Start with Option A for quick wins (reduce duplication where it is already painful).
2) Introduce Option B once the example catalog is stable enough to justify enforcement.

## “Official first” discovery

Whatever the backing store is, the CLI should present:

- “Official” examples first (the onboarding/product surface)
- “Maintainer/Stress” behind `--all`

This should be a hard rule to keep the repo approachable.

