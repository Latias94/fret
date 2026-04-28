# M2 Selectable and Menu Label Identity Slice - 2026-04-28

## Goal

Extend the label identity grammar from buttons to row-like IMUI controls where same visible labels
are common.

## Landed

- Routed selectable rows through the private label identity parser.
- Routed menu item rows through the same parser, including checkbox/radio/action/submenu menu item
  paths because they share the private menu item implementation.
- Selectable and menu item visible text now hides `##` / `###` suffixes.
- Parsed identity keys now back keyed subtrees for selectable and menu item rows.

## Proof

`ecosystem/fret-imui/src/tests/label_identity.rs` now proves that:

- selectable rows hide `###` suffixes from painted text,
- selectable focus follows `###stable_id` across reorder,
- menu items hide `##` and `###` suffixes from painted text,
- hidden-label menu items do not paint their identity suffix.

## Gates

- `cargo nextest run -p fret-ui-kit --features imui imui_label_identity --no-fail-fast`
- `cargo nextest run -p fret-imui label_identity --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 2`
- `cargo fmt --package fret-ui-kit --package fret-imui --check`

## Remaining

Decide whether checkbox/radio/switch/slider should use label identity as a keyed subtree or only
consume the rendered-label portion. Those controls pair a label with a model or scalar value, so the
next slice should avoid accidentally making the label grammar compete with model identity.
