# `UiWriter` / `Response` contract closeout — 2026-03-29

Status: closeout decision
Last updated: 2026-03-29

Related:

- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/imui-stack-fearless-refactor-v1/TODO.md`
- `docs/adr/0223-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- `ecosystem/fret-authoring/src/lib.rs`
- `ecosystem/fret-authoring/tests/contract_surface_policy.rs`
- `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
- `ecosystem/fret-imui/src/lib.rs`

## Why this note exists

The last open M4 question in the active `imui` workstream was:

> Did the fearless refactor materially change the shared `UiWriter` / `Response` contract, such
> that an ADR or contract-tracking note also needed an update?

This note records the answer with evidence instead of leaving it implicit.

## Current conclusion

No material `UiWriter` or `Response` contract change landed as part of the `imui` stack fearless
refactor.

Meaning:

- no ADR update is required for this workstream closeout,
- `fret-imui` still re-exports the same small shared `Response`,
- and the refactor changed the surrounding facade/module shape without changing the underlying
  shared authoring contract.

## What the shared contract still is

`fret-authoring` remains the contract owner for the shared immediate-style authoring surface:

- `Response`
  - `hovered`
  - `pressed`
  - `focused`
  - `clicked`
  - `changed`
  - `rect`
- `UiWriter`
  - `with_cx_mut(...)`
  - `add(...)`
  - `extend(...)`
  - `mount(...)`
  - `keyed(...)`

`fret-imui` still exposes that contract by re-export rather than fork:

- `pub use fret_authoring::Response;`
- `pub use fret_authoring::UiWriter;` through `prelude`

Evidence:

- `ecosystem/fret-authoring/src/lib.rs`
- `ecosystem/fret-imui/src/lib.rs`

## Why this is treated as "no material change"

The current workstream changed:

- compatibility names in `fret-ui-kit::imui`,
- module splits in `fret-imui`,
- editor adapter coverage in `fret-ui-editor::imui`,
- and related closeout docs/tests.

It did **not** change:

- the fields on shared `fret_authoring::Response`,
- the method surface of `fret_authoring::UiWriter`,
- or the ownership boundary recorded in ADR 0223 that keeps shared authoring contracts in
  `ecosystem/fret-authoring`.

Historical evidence:

- `UiWriter` surface was introduced in `bbc3112cb` and extended by `6642bb6b7`
- `Response` moved into `fret-authoring` in `1bee2c783`
- the current `imui` fearless-refactor commit range does not change that contract surface

Evidence:

- `git log -- ecosystem/fret-authoring/src/lib.rs`
- `git log -- ecosystem/fret-imui/src/lib.rs ecosystem/fret-imui/src/frontend.rs`

## Gates that now lock the conclusion

Two tests now guard this boundary from opposite sides:

1. `ecosystem/fret-authoring/tests/contract_surface_policy.rs`
   - locks the small shared `Response` fields and `UiWriter` method surface
2. `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
   - locks the split between shared `Response` and richer facade-only `ResponseExt`

Together they make the closeout claim executable:

- the shared contract stays small,
- richer interaction signals stay in the facade layer,
- and future refactors cannot silently widen `fret-authoring` while still claiming "no contract
  change".

## ADR alignment result

ADR 0223 remains aligned as written:

- shared authoring contracts stay in `ecosystem/fret-authoring`,
- `fret-imui` remains an optional frontend over that contract,
- and ergonomics/policy continue to live in ecosystem facade crates rather than in the shared
  contract crate.

No ADR update is required for this workstream closeout batch.

## Reopen triggers

Reopen ADR/contract-tracking work only if one of these becomes true:

1. `Response` gains or loses shared fields,
2. `UiWriter` gains or loses core methods,
3. `fret-imui` stops re-exporting the shared contract and forks its own shape,
4. richer facade-only response semantics start leaking into `fret-authoring`.

If none of those are true, treat `UiWriter` / `Response` as contract-stable for the purposes of
this workstream.
