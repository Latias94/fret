# Rich Template Productization Audit — 2026-04-02

Status: M2 baseline classification and slimming pass landed

## Scope

This audit answers one release-facing question:

- what should the richer `todo` scaffold keep as the third rung product baseline,
- and what should remain explicitly deletable so the template does not read like a framework wall?

## Classification

### Product baseline

These belong in the third rung by default because they read like ordinary product behavior:

- centered card page shell owned by the app/template,
- title + summary + progress snapshot,
- add-task input row,
- keyed list rows with clear completion state,
- clear-completed action,
- optional filter chips for a realistic Todo surface.

### Optional richer example material

These are still valid on the third rung, but they should remain easy to delete:

- selector-derived read-side projection (`TodoDerived`) for filtered rows and counters,
- query-backed focus note (`tip_nonce`, `tip_key`, `tip_policy`, `tip_handle`, `tip_callout`),
- command palette enablement when the scaffold option is turned on.

### Framework-showcase drift

These were classified as noise for the generated product starting point:

- framework-specific seed copy,
- app-level command palette button inside the Todo card,
- badge-heavy footer chrome when plain summary text is enough,
- query content occupying primary screen real estate instead of a secondary support surface.

## Changes landed

- Replaced framework-specific seed tasks with ordinary product-like starter copy.
- Kept selector/query in the scaffold, but demoted the query slice into a secondary "Focus note"
  callout instead of treating it as primary content.
- Removed the in-card command palette button from the richer Todo surface; the feature can stay
  enabled via shortcut without becoming app chrome.
- Simplified the footer from multiple badges to text summary plus the destructive maintenance
  action.
- Added explicit "first cuts" guidance to the generated `README.md` so app authors know which
  slices to delete first.

## Ownership conclusions

- No universal `AppShell` is needed for this work.
- The richer third rung may keep app-owned page/card composition as a teaching surface.
- Selector/query remain example slices on top of the LocalState-first default path, not a reason to
  widen the default facade.

## Evidence

- `apps/fretboard/src/scaffold/templates.rs`
- `docs/examples/README.md`
- `docs/examples/todo-app-golden-path.md`
- `ecosystem/fret/README.md`

## Validation targets

- scaffold source-policy tests in `apps/fretboard/src/scaffold/templates.rs`
- generated README assertions in `apps/fretboard/src/scaffold/templates.rs`
