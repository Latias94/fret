# ImUi Menu/Tab Trigger Response Canonicalization v1 - TODO

Status: closed closeout lane
Last updated: 2026-04-13

## Lane setup

- [x] Split this cleanup into its own narrow follow-on instead of reopening the closed response
      surface lane.
- [x] Wire the lane into the umbrella docs, `docs/workstreams/README.md`, `docs/roadmap.md`, and
      `docs/todo-tracker.md`.
- [x] Freeze that this lane only canonicalizes the helper-owned outward response API and does not
      widen runtime/mechanism-layer contracts.

## M0 - Canonical target freeze

- [x] Record the duplicate helper surface and why it is worth removing.
      Result: `DESIGN.md` now names the compatibility track explicitly.
- [x] Freeze the intended canonical target surface.
      Result: `begin_menu[_with_options]`, `begin_submenu[_with_options]`, and
      `tab_bar[_with_options]` are the target names for richer outward responses.
- [x] Name the first-open repro/gate surfaces before refactoring call sites.
      Result: `EVIDENCE_AND_GATES.md` records current focused tests plus the source/demo proof
      surface that must move to canonical names.

## M1 - API cleanup

- [x] Remove the duplicate `*_response*` helper entry points once the canonical helper names return
      the richer response values directly.
- [x] Update all in-tree call sites, focused tests, and demo proof to use the canonical names.
- [x] Keep the cleanup facade-only; do not widen `fret-authoring::Response` or `crates/fret-ui`.

## M2 - Proof and closeout

- [x] Refresh focused `fret-imui` tests so they prove behavior through the canonical helper names.
- [x] Refresh `imui_response_signals_demo` and its source gate so the demo teaches only the
      canonical naming story.
- [x] Close the lane explicitly once no duplicate alias surface remains and the repo can point at
      the right historical lane for the original “should this exist?” verdict.
      Result: `FINAL_STATUS.md` now closes the naming cleanup and routes readers back to the
      earlier response-surface lane for the additive existence verdict.
