# M2 Model and Explicit-ID Label Identity Slice - 2026-04-28

## Goal

Finish the first admitted label-bearing IMUI surface so Dear ImGui-style label identity grammar is
consistent across controls without moving identity policy into `fret-ui`.

## Landed

- Routed checkbox, radio, switch, and slider helpers through the private label identity parser.
- Those model/scalar controls now key their helper-owned subtrees by parsed label identity, while
  rendered labels and default accessibility labels use only the visible label.
- Kept explicit-ID controls keyed by their existing IDs while stripping `##` / `###` suffixes from
  their visible labels:
  - combo and combo-model triggers
  - menu and submenu triggers
  - tab triggers
  - collapsing headers
  - tree nodes
- Routed `separator_text` through visible-label parsing because it renders a label but has no item
  identity to update.
- Disabled incremental test builds for `fret-ui-kit` to avoid unstable Windows/MSVC test-binary
  link artifacts during the parser gate.

## Proof

`ecosystem/fret-imui/src/tests/label_identity.rs` now proves that:

- checkbox labels hide suffixes and `###stable_id` preserves focus across visible-label changes and
  reorder,
- radio, switch, and slider labels hide suffixes,
- combo, menu trigger, tab trigger, collapsing header, tree node, and separator text labels hide
  suffixes while keeping their explicit IDs as the identity owner.

## Gates

- `cargo nextest run -p fret-ui-kit --features imui imui_label_identity --no-fail-fast`
- `cargo nextest run -p fret-imui label_identity --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 1`
- `cargo fmt --package fret-ui-kit --package fret-imui --check`

## Verdict

The current label-bearing IMUI control helpers now either:

- use parsed label identity when the helper has no explicit string ID, or
- use their existing explicit ID while consuming only the parsed visible label.

Future work should not reopen this lane for generic text, table-header naming, localization policy,
`test_id` inference, or runtime ID-stack debugging. Those require narrower follow-ons if product
evidence appears.
