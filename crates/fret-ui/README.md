# `fret-ui`

`fret-ui` is the UI runtime contract layer for the Fret workspace.

It provides the mechanisms that higher layers build on:

- Declarative element authoring + reconciliation (GPUI-style direction),
- Layout + hit testing + paint orchestration (runner/host driven),
- Cross-frame state storage and stable identity,
- Focus/input routing surfaces and debug snapshots.

This crate is **not** intended to be a policy-heavy component library. Radix/shadcn-style interaction
policies (dismiss, focus trap/restore, hover intent, default sizing/padding conventions, etc.)
belong in the ecosystem layer (`fret-ui-kit`, `fret-ui-shadcn`) rather than here.

## Module ownership map

The crate is large; treat it as a set of subsystems:

- `src/declarative/`: frame-to-frame element tree building and reconciliation.
- `src/tree/`: runtime tree representation, layout/paint passes, hit testing, and debug stats.
- `src/elements/`: builder helpers and element runtime utilities.
- `src/element.rs`: element property types (including layout/text/scroll props) and runtime-facing
  contracts.
- `src/layout/`: layout engine glue (`constraints`, `pass`, `engine`).
- `src/text/`: text input/editing surfaces (IME integration lives here, policy lives above).
- `src/scroll/` + `src/virtual_list/`: imperative scroll handles and the virtual list virtualizer.
- `src/theme/`: token/config/theme snapshot plumbing (policy lives above).
- `src/overlay_placement/`: placement helpers for overlay/popup positioning (arbitration policy is
  tracked separately in workstreams).
- Crate-root modules: thin glue/utility pieces that have not yet been regrouped.

## Public surface

Prefer importing from `fret_ui`’s re-exports in `src/lib.rs` (e.g. `UiTree`, `UiHost`,
`ScrollHandle`, `TextInputStyle`). Internal modules are being regrouped as part of the bottom-up
fearless refactor; relying on deep paths makes churn more likely.

## Local refactor gates

- Fastest loop: `cargo nextest run -p fret-ui`
- Formatting: `cargo fmt`

