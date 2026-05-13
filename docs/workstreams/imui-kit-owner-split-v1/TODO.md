# ImUi Kit Owner Split v1 - TODO

Status: closed
Last updated: 2026-05-13

## P0 - Lane Setup

- [x] Create a narrow follow-on from `imui-imgui-gap-closure-v1`.
- [x] Record the no-public-API-widening and no-runtime-contract-widening rules.
- [x] Capture the current `fret-ui-kit::imui` source-size baseline.
- [x] Run the baseline gates listed in `EVIDENCE_AND_GATES.md`.
- [x] Wire this lane into the IMUI workstream map and tracker highlights.

## P1 - First Owner Split

- [x] Read `facade_writer.rs` and pick one private implementation cluster.
- [x] Move the selected cluster behind a private owner module without public method renames.
- [x] Delete local duplication only when the replacement helper is private and behavior-equivalent.
      M1 moved the button/action/command-button facade wrappers without introducing a replacement
      behavior helper or deleting canonical public APIs.
- [x] Run focused `fret-ui-kit` IMUI gates.
- [x] Add an `M1_*_SLICE_YYYY-MM-DD.md` status note with evidence.

## P2 - Response / Status Path Audit

- [x] Audit `facade_support.rs`, `interaction_runtime/*`, and `response/hover.rs` for duplicated
      response-status assembly.
- [x] Decide whether a private typed status helper reduces risk without widening
      `fret-authoring::Response` or public `ResponseExt`.
- [x] If yes, land the smallest private helper split and record an M-note.
      M2 moved the shared pressable response assembly path into
      `interaction_runtime::populate_pressable_response(...)` without widening public IMUI names
      or runtime contracts.
- No-change verdict was not needed for this slice because the helper path landed.

## P3 - Menu Facade Owner Split

- [x] Move the menu item / begin-menu inherent facade wrappers behind a private owner module.
- [x] Keep public `ImUiFacade` method names and `fret::imui` paths stable.
- [x] Keep `fret-imui` and `crates/fret-ui` runtime contracts unchanged.
- [x] Run focused `fret-ui-kit` IMUI gates.
- [x] Add an `M3_*_SLICE_YYYY-MM-DD.md` status note with evidence.

## P4 - Selection / Combo Facade Owner Split

- [x] Move selectable, multi-selectable, and combo inherent facade wrappers behind a private owner
      module.
- [x] Keep public `ImUiFacade` method names and `fret::imui` paths stable.
- [x] Keep `fret-imui` and `crates/fret-ui` runtime contracts unchanged.
- [x] Run focused `fret-ui-kit` IMUI gates.
- [x] Add an `M4_*_SLICE_YYYY-MM-DD.md` status note with evidence.

## P5 - Closeout

- [x] Add a closeout audit.
- [x] Mark `WORKSTREAM.json` closed with `stay_closed`.
- [x] Name the next narrower follow-on for disclosure wrappers.
- [x] Keep remaining additive widgets, docking, multi-window, and runtime contract work out of
      this lane.

## Guardrails

- [x] Keep `fret-imui` thin.
- [x] Keep public `fret-ui-kit::imui` names stable.
- [x] Keep app-facing teaching on `fret::imui`.
- [x] Keep additive Dear ImGui component work in separate follow-ons.
- [x] Keep docking/multi-window work in `fret-docking` / runner lanes.
