# M1 Button Label Identity Slice - 2026-04-28

## Goal

Land the private Dear ImGui-style label identity parser and prove the first label-bearing control
can use it without widening runtime identity contracts.

## Landed

- Added a private `fret-ui-kit::imui` label parser for:
  - plain labels
  - `Label##suffix`
  - `##hidden_id`
  - `Label###stable_id`
  - `###hidden_stable_id`
- Routed the button family through the parser.
- Button visible text and default accessibility labels now use the rendered label, not the identity
  suffix.
- Button identity is keyed by the parsed identity value so `###stable_id` survives visible-label
  changes and reorder.

## Proof

`ecosystem/fret-imui/src/tests/label_identity.rs` proves that:

- `##` and `###` suffixes are not painted,
- `Label###stable_id` can change the visible label while preserving logical identity,
- and focus follows the stable identity across reorder.

## Gates

- `cargo nextest run -p fret-ui-kit --features imui imui_label_identity --no-fail-fast`
- `cargo nextest run -p fret-imui label_identity --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 2`
- `cargo fmt --package fret-ui-kit --package fret-imui --check`

## Remaining M2

Extend the same parser to the next admitted label-bearing controls, starting with selectable and
menu item rows. Keep `test_id` explicit and do not infer diagnostics identity from the label.
