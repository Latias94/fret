# Material3 Token Resolver Fallback v1 TODO

Status: Closed
Last updated: 2026-05-31

Task IDs use `M3TRF-*`.

## Tasks

- [x] M3TRF-010: Open the resolver/fallback workstream.
  - Scope: `docs/workstreams/material3-token-resolver-fallback-v1`, `docs/workstreams/README.md`.
  - Expected result: narrow lane exists with resolver/fallback scope, gates, and follow-on context.
  - Gate: JSON/catalog/diff hygiene.

- [x] M3TRF-020: Move pure color composition helpers behind the token resolver module.
  - Scope: `foundation::token_resolver`, token modules with local `alpha_mul` / `blend_over`
    copies, fixture runner helper if useful.
  - Expected result: duplicated alpha/blend helpers are deleted from component token modules while
    visual fixture outcomes remain unchanged.
  - Gate: token visual fixture runner plus focused package check/clippy.
  - Note: `alpha_mul` and `blend_over` now live in `foundation::token_resolver`; local copies were
    removed from field, selection, slider/list/switch token modules and the visual fixture runner.

- [x] M3TRF-030: Add resolver helpers for common component-to-system fallback chains.
  - Scope: high-duplication color/number fallback paths in field and selection token modules.
  - Expected result: token modules describe Material token role/state selection while resolver owns
    primitive fallback semantics.
  - Gate: token visual fixtures plus targeted tests for migrated families.
  - Note: `MaterialTokenResolver` now owns Material state-layer interaction opacity and disabled
    state-layer opacity fallback semantics. Checkbox, Radio, Switch, and Slider token modules now
    keep only component role/state key selection for migrated state-layer color/opacity paths.
    Field-family migration is intentionally left to M3TRF-040.

- [x] M3TRF-040: Migrate the heaviest field-family fallback modules.
  - Scope: TextField, Select, Autocomplete token modules.
  - Expected result: field-family token fallbacks use shared resolver helpers without visual drift.
  - Gate: token visual fixtures plus TextField/Select RTL and field-family tests.
  - Note: `MaterialTokenResolver` now owns field-family component-to-system color fallback,
    multi-system fallback chains, optional opacity lookup, and explicit fallback-color lookup.
    TextField, Select, and Autocomplete token modules now retain state/key selection while resolver
    handles migrated color/opacity fallback mechanics.

- [x] M3TRF-050: Verify and close or split remaining fallback hardening.
  - Scope: docs, package gates, catalog, layering, diff hygiene.
  - Expected result: lane closes or splits a smaller follow-on for any remaining broad token
    fallback families.
  - Gate: all commands in `EVIDENCE_AND_GATES.md`.
  - Note: closeout verifies this lane's target scope and records remaining raw color fallback
    families in non-field components as a future follow-on, not additional M3TRF scope.

## Notes

- Do not change public recipe behavior in this lane.
- Do not move generated Material Web v30 data or import policy unless a task explicitly scopes it.
- Keep changes fixture-first: every migration should preserve existing token outcome rows.
