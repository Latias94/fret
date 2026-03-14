# Font Mainline Fearless Refactor v1

Status: In progress

This workstream defines the **owner map, refactor order, and non-negotiable boundaries** for the
mainline font system refactor. It assumes a fearless posture: we prefer the correct ownership model
over compatibility scaffolding.

Related inputs:

- `docs/workstreams/standalone/font-system-audit-zed-parley-xilem-v1.md`
- `docs/workstreams/standalone/font-catalog-refresh-policy-v1.md`
- `docs/workstreams/standalone/font-fallback-conformance-v1.md`
- `docs/workstreams/crate-audits/fret-fonts.l0.md`
- `docs/workstreams/crate-audits/fret-render-text.l0.md`
- `docs/workstreams/crate-audits/fret-launch.l1.md`

Related ADRs:

- `docs/adr/0257-font-selection-fallback-and-variable-font-instances-v1.md`
- `docs/adr/0258-font-catalog-refresh-and-revisioning-v1.md`
- `docs/adr/0259-system-font-rescan-and-injected-font-retention-v1.md`

## Target ownership model

### `crates/fret-fonts`

Owns:

- bundled font bytes
- bundled profile manifest
- role/generic guarantees of bundled profiles

Does not own:

- locale policy
- platform fallback heuristics
- startup sequencing
- renderer cache invalidation rules

### `crates/fret-render-text`

Owns:

- font DB access and catalog extraction
- fallback policy composition and fingerprinting
- variable font instance identity inputs
- shaping, wrapping, metrics, and text-specific caches
- rescan seed/result application semantics

Does not own:

- runner startup policy
- app-global publication rules
- platform event-loop orchestration

### `crates/fret-launch`

Owns:

- async startup/rescan orchestration
- publication of renderer-derived font environment into runtime globals
- wiring desktop/web runner effects to renderer font operations

Does not own:

- bundled profile policy
- fallback-policy derivation
- renderer-internal cache decisions

## Current audit summary

1. `fret-fonts` is structurally clean; the main remaining risk is feature-matrix drift rather than
   module ownership, so the workstream now relies on an explicit feature-matrix gate.
2. `fret-render-text` is the real refactor hotspot: it has the right ownership direction, but too
   many responsibilities are concentrated in `parley_shaper.rs` and `wrapper.rs`.
3. `fret-launch` already has a usable font boundary in `runner/font_catalog.rs`, but startup seeding
   and platform-specific orchestration still need a stricter "wiring only" posture.

## Refactor rules

1. Do not move locale or platform fallback policy into `fret-fonts`.
2. Do not add new fallback heuristics to `fret-launch`; move policy either to runtime globals or
   renderer-owned text policy, depending on who actually consumes it.
3. Do not let `fret-render-text` grow new runner/platform dependencies during the split.
4. Prefer smaller crate-private modules and explicit facades over more `pub mod` re-exports.
5. Every ownership move should leave behind one executable gate or one evidence-backed audit update.

## Deliverables in this folder

- `README.md`: owner map and refactor rules
- `TODO.md`: landable refactor steps
- `MILESTONES.md`: staged exit criteria and gates
