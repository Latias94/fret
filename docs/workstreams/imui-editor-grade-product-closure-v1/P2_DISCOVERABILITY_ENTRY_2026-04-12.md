# P2 Discoverability Entry - 2026-04-12

Status: focused P2 discoverability decision / first-open entry freeze

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
- `P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`
- `docs/diagnostics-first-open.md`
- `docs/debugging-ui-with-inspector-and-scripts.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-fearless-refactor-v2/START_HERE.md`
- `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`

## Purpose

P2 still needed one discoverability answer:

> what single document should a maintainer or app author open first so they can follow the shared
> diagnostics/devtools loop without hopping across multiple diagnostics notes?

This note freezes that entry before P2 turns into a broader tooling-product lane.

## Audited evidence

- `P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
- `P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`
- `docs/debugging-ui-with-inspector-and-scripts.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-fearless-refactor-v2/START_HERE.md`
- `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`
- `docs/README.md`

## Current gap

The repo already has enough diagnostics writing, but not enough first-open guidance.

Today a reader still has to infer the entry order from several notes:

- inspect/pick guidance in one top-level doc,
- bundle/script guidance in another top-level doc,
- workstream navigation in a v2 diagnostics note,
- GUI dogfood in a separate workstream note.

That is acceptable as a reference library, but it is not acceptable as the default first-open path
for this lane.

## Frozen discoverability entry

From this point forward, the canonical first-open diagnostics entry is:

- `docs/diagnostics-first-open.md`

That page must do one job well:

1. state that it is the default first-open entry,
2. name the default CLI-first loop:
   inspect -> selector -> script -> launched run -> bounded evidence -> compare/summarize/dashboard,
3. keep GUI and MCP as branches after the shared artifacts root exists,
4. tell the reader exactly which deeper note to open next for each branch.

## Frozen branch map

Use this branch map by default.

### `docs/diagnostics-first-open.md`

Owns:

- the canonical first-open reading order,
- the default command loop,
- the branch map into inspect, bundles/scripts, GUI, and workstream planning.

Does **not** own:

- the full picker UX reference,
- the full script/bundle schema reference,
- GUI-specific dogfood details,
- diagnostics roadmap history or migration intent.

### `docs/debugging-ui-with-inspector-and-scripts.md`

Owns:

- interactive inspect/pick workflow details,
- picker shortcuts,
- inspect-first debugging recipes.

It should be read as the inspect branch, not as the only first-open diagnostics doc.

### `docs/ui-diagnostics-and-scripted-tests.md`

Owns:

- bundle/script schema and sidecar details,
- launched run/session hygiene,
- bounded artifact triage commands.

It should be read as the bundles/scripts branch, not as the first page a new reader must discover
 on their own.

### `docs/workstreams/diag-fearless-refactor-v2/START_HERE.md`

Owns:

- diagnostics workstream navigation,
- deeper contract note routing,
- maintainer/history orientation.

It is not the default first-open app-author diagnostics page.

### `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`

Owns:

- the GUI consumer branch over the shared contracts.

It is not the default first-open entry and should assume the shared artifacts-first posture is
already understood.

## Decision

From this point forward:

1. `docs/diagnostics-first-open.md` is the canonical first-open diagnostics/devtools entry.
2. Existing diagnostics notes remain, but each should say what branch it owns.
3. `START_HERE.md` remains a maintainer/workstream navigation note, not the default app-author
   onboarding entry.
4. `DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md` remains a consumer-specific branch, not a second diagnostics
   architecture.
5. P2 should now reject new discoverability fixes that add another parallel entry instead of
   strengthening the canonical one.

## Immediate execution consequence

For this lane:

- route first-open diagnostics readers through `docs/diagnostics-first-open.md`,
- keep `docs/README.md` and the branch docs aligned with that entry,
- and treat future discoverability edits as branch clarity work rather than as permission to
  recreate another competing "start here" page.
