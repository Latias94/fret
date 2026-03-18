# One-Slot Write Audit — 2026-03-17

Status: M1 audit note
Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`

## Why this note exists

The first open question on this lane was whether the current one-slot write family is genuinely too
wide, or whether it is already a small intentional budget that only looked broad when mixed
together with the older action-first and dataflow discussions.

This note audits the current in-tree teaching surface for:

- `local_update::<A>(...)`
- `local_set::<A, T>(...)`
- `toggle_local_bool::<A>(...)`

## Current evidence

### `local_update::<A>(...)`

Observed use:

- `apps/fret-cookbook/examples/hello.rs`
  - canonical "increment a local counter" shape
- `apps/fret-cookbook/examples/imui_action_basics.rs`
  - the same in-place counter-mutation story on a different surface
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
  - "bump a nonce/counter" style local write
- `apps/fret-cookbook/examples/overlay_basics.rs`
  - increment a simple diagnostic underlay counter
- `apps/fretboard/src/scaffold/templates.rs`
  - hello template uses it as the narrow "increment one local value" teaching path
  - todo template uses it for `RefreshTip` nonce bumping

Classification:

- this is the "mutate one local value in place" helper
- it is not acting as a disguised transaction helper
- it is not competing with `local_set` for constant-value writes

### `local_set::<A, T>(...)`

Observed use:

- `apps/fret-examples/src/hello_counter_demo.rs`
  - reset count to a constant
  - set step presets to fixed strings
- `apps/fret-cookbook/examples/hello_counter.rs`
  - same constant reset/preset pattern
- `apps/fret-examples/src/embedded_viewport_demo.rs`
  - choose among fixed size presets
- `apps/fret-cookbook/examples/query_basics.rs`
  - set invalidation request flags to `true`
- `apps/fretboard/src/scaffold/templates.rs`
  - todo template sets filter enum presets
- `docs/examples/todo-app-golden-path.md`
  - draft clear uses the same fixed-value write posture

Classification:

- this is the "set one local to a known target value" helper
- the common in-tree uses are reset/preset/enum selection writes
- it is not being used as a substitute for coordinated multi-local transactions

### `toggle_local_bool::<A>(...)`

Observed use:

- `apps/fret-cookbook/examples/toggle_basics.rs`
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-cookbook/examples/customv1_basics.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`

Classification:

- this is the "flip one local bool" helper
- its callsites are semantically obvious toggles rather than hidden set/update variants
- in current first-party surfaces it improves readability instead of widening the mental model

## First-contact docs and template posture

The current docs/templates do **not** present these helpers as three competing global defaults:

- `docs/crate-usage-guide.md`
  - keeps the broad first-contact handler surface centered on `locals_with((...)).on::<A>(...)`,
    `transient::<A>(...)`, keyed payload row writes, and explicit `models::<A>(...)`
- `docs/first-hour.md`
  - also centers the default ladder on `locals_with((...)).on::<A>(...)` plus payload row writes
- `docs/authoring-golden-path-v2.md`
  - collapses the trio into one row: `cx.actions().local_set/update`
- `apps/fretboard/src/scaffold/templates.rs`
  - hello teaches `local_update::<A>(...)` because the example is specifically "increment one
    local value"
  - todo teaches `local_update` for a nonce bump and `local_set` for fixed filter presets

Conclusion:

- the repo is already teaching the trio as a small, semantics-driven companion family
- it is **not** currently teaching them as three co-equal replacements for
  `locals_with((...)).on::<A>(...)`

## Decision

Freeze the current one-slot family as the intentional default budget:

- `local_update::<A>(...)`
- `local_set::<A, T>(...)`
- `toggle_local_bool::<A>(...)`

Additional interpretation:

- `locals_with((...)).on::<A>(...)` remains the primary explicit transaction story for anything
  that coordinates more than one local or requires cross-field reasoning
- the one-slot trio is a companion family, not a competing "new root default" that should replace
  `locals_with((...)).on::<A>(...)` everywhere

## What this audit rules out

- no new one-slot helper family should be introduced from this evidence alone
- no rename-only churn is justified just because the repo currently has three one-slot verbs
- no workstream time should be spent trying to force all one-slot writes through
  `locals_with((...)).on::<A>(...)` when the existing trio is already small and semantically
  distinct

## What remains open after M1

The next real question on this lane is M2:

- whether the current payload row-write posture (`payload_local_update_if::<A>(...)`,
  `payload_locals::<A>(...)`, and quarantined `payload::<A>()`) is already the right final budget
  or still needs further narrowing.
