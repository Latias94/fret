# Default Lane LocalState + Keyed Identity Freeze Audit — 2026-04-16

Status: Frozen

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/authoring-golden-path-v2.md`
- `docs/first-hour.md`
- `docs/examples/README.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `crates/fretboard/src/scaffold/templates.rs`

## Scope

Close the two remaining default-lane wording items in M1:

1. `LocalState<T>` is the only blessed first-contact local-state story.
2. keyed identity is the only taught dynamic-list/subtree rule on the default lane.

This audit does not reopen the advanced raw-model lane, runtime substrate convergence, or IMUI.

## Assumptions-first checkpoint

1. The remaining gap is wording/gates, not runtime mechanism.
   Confidence: Confident.
   Evidence: `docs/authoring-golden-path-v2.md`, `docs/first-hour.md`,
   `docs/examples/todo-app-golden-path.md`, `apps/fret-examples/src/lib.rs`,
   `crates/fretboard/src/scaffold/templates.rs`.
2. Raw `Model<T>` and `clone_model()` are still valid, but only as explicit advanced/shared-owner
   seams rather than first-contact teaching material.
   Confidence: Confident.
   Evidence: `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`,
   `ecosystem/fret/tests/raw_state_advanced_surface_docs.rs`.
3. “Keyed identity is the default rule” does not mean unkeyed iteration must disappear entirely;
   it means unkeyed iteration must remain an explicit static-list exception rather than a peer
   teaching story for dynamic collections.
   Confidence: Confident.
   Evidence: `docs/first-hour.md`, `docs/examples/README.md`,
   `docs/examples/todo-app-golden-path.md`.

## Findings

### 1) The default docs already converge on one LocalState-first story

The current first-contact docs all point at the same default authoring posture:

- `docs/authoring-golden-path-v2.md` explicitly says this is the only blessed first-contact
  local-state story.
- `docs/first-hour.md` teaches `simple-todo` as `LocalState + view runtime + typed actions +
  keyed lists` only.
- `docs/examples/README.md` keeps the onboarding ladder on `LocalState` first and quarantines
  `cx.raw_model::<T>()` behind the advanced lane.
- `docs/examples/todo-app-golden-path.md` teaches one default path only: LocalState-first app
  code on `fret::app`.

This means the remaining work was not to invent another local-state owner, but to freeze the
wording and keep it from drifting.

### 2) The scaffold surface already matches the same contract

The generated onboarding templates already reinforce the same posture:

- `simple-todo` keeps the second rung on `LocalState<Vec<_>>` + payload row actions.
- `todo` keeps the richer third rung LocalState-first while making selector/query slices
  explicitly deletable.
- scaffold tests already forbid `clone_model()` / `LocalState::from_model(...)` from drifting
  into those default templates.

That makes the freeze evidence stronger than “docs say so”: the shipped starter surfaces already
teach the same contract.

### 3) Keyed identity is already the default dynamic-list rule, with one explicit exception

The default-lane identity teaching surface is now coherent:

- `docs/authoring-golden-path-v2.md` teaches keyed lists via `ui::for_each_keyed(...)` by default.
- `docs/first-hour.md` says lists that can change shape should assume keys.
- `docs/examples/todo-app-golden-path.md` teaches stable keys for dynamic lists.
- `docs/examples/README.md` keeps unkeyed iteration only as an explicit static-list exception.

Therefore the contract is:

- dynamic/reorderable/per-row-state collections teach keyed identity,
- unkeyed iteration remains allowed only when the list is static and order-stable,
- and that exception should not be promoted back into the default onboarding story.

## Evidence

- `docs/authoring-golden-path-v2.md`
- `docs/first-hour.md`
- `docs/examples/README.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/README.md`
- `crates/fretboard/src/scaffold/templates.rs`
- `ecosystem/fret/tests/default_state_identity_docs.rs`
- `apps/fret-examples/src/lib.rs`

## Gate commands

- `cargo check -p fret --test default_state_identity_docs --test raw_state_advanced_surface_docs`
- `cargo check -p fretboard --tests`
- `cargo check -p fret-examples --lib`

## Outcome

The M1 default-lane wording is now frozen:

1. `LocalState<T>` is the only blessed first-contact local-state story.
2. Raw `Model<T>` / `clone_model()` / `cx.raw_model::<T>()` stay explicit advanced/shared-owner
   seams.
3. Keyed identity is the only default dynamic-list/subtree rule on the default lane.
4. Unkeyed iteration remains an explicit static-list exception, not a peer onboarding path.
