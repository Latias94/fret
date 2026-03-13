# Font System (Fearless Refactor v1) — Milestones

This file defines the milestones for:

- `docs/workstreams/font-system-fearless-refactor-v1/DESIGN.md`

## Milestone 0 — Turn the audit into an execution lane

Outcome:

- The font-system reset has one canonical workstream home.
- The repository distinguishes background audit material from the live execution tracker.

Deliverables:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- docs index links

Exit criteria:

- Maintainers know where to put font-system refactor rationale versus live migration status.

## Milestone 1 — Make font-state publication atomic

Outcome:

- Desktop and web publish a **settled** renderer font environment instead of partial intermediate
  states.

Deliverables:

- one shared publication helper,
- removal of stale publication ordering,
- tests for startup/update publication correctness.

Exit criteria:

- The runtime global `TextFontStackKey` always matches the renderer's current effective locale +
  family-config environment at publication time.

## Milestone 2 — Make bundled profiles a real contract

Outcome:

- Bundled fonts stop being implicit family-name folklore and become a manifest-backed product
  surface.

Deliverables:

- typed bundled roles,
- profile metadata,
- manifest-backed bootstrap logic,
- deterministic bundled-only test coverage.

Exit criteria:

- For every bundled profile we can answer, in code and docs:
  - which roles it provides,
  - which family names are expected,
  - which generic-family guarantees it makes.

## Milestone 3 — Make rescan semantics cheap and honest

Outcome:

- System-font rescans only trigger expensive invalidation when the effective font environment
  actually changed.

Deliverables:

- rescan-result/environment fingerprint,
- no-op apply path,
- regression tests and diagnostics evidence.

Exit criteria:

- A no-op rescan apply does not bump `TextFontStackKey`, does not clear text caches, and does not
  reset glyph atlases.

## Milestone 4 — Close the diagnostics and fallback loop

Outcome:

- The post-refactor font system is still easy to audit, debug, and reason about.

Deliverables:

- updated fallback-policy diagnostics,
- profile-aware mixed-script conformance coverage,
- native mixed-script locale-switch conformance for system-font builds on the
  platform-default/system-fallback lane,
- clear linkage back to the relevant ADRs and implementation-alignment notes.

Exit criteria:

- A maintainer can explain any font-selection outcome using:
  - the bundled profile contract,
  - the renderer fallback-policy snapshot,
  - the font trace,
  - the published `TextFontStackKey`.
- Native locale switching on the mixed-script fallback page can be proven by diagnostics evidence
  alone without relying on ad-hoc settings UI or the UI Gallery's Windows-only curated fallback
  override.
