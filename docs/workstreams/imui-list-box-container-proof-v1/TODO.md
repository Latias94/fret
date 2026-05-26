# IMUI List Box Container Proof v1 - TODO

Status: Closed
Last updated: 2026-05-25

## LBC-010 - Boundary Decision

- [x] Start a narrow follow-on instead of reopening the closed generic collection helper lane.
- [x] Record that this is a `BeginListBox`-style container, not a selection/collection policy API.

## LBC-020 - Container Surface

- [x] Add `ListBoxOptions`.
- [x] Add private `list_box_controls.rs` renderer.
- [x] Add `UiWriterImUiFacadeExt::list_box` and `list_box_with_options`.
- [x] Export the options type without changing `fret-imui` dependencies.

## LBC-030 - Proof

- [x] Add a focused composition test proving ListBox semantics, scroll/test-id forwarding, and
      selectable row hosting.
- [x] Add source-policy markers preventing this lane from becoming a generic collection helper.
- [x] Run focused Rust/source/catalog/format gates and record evidence.
