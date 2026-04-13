# ImUi Menu/Tab Trigger Response Surface v1 - TODO

Status: active execution lane
Last updated: 2026-04-13

## Lane setup

- [x] Create the lane as a narrow P0 follow-on under the active immediate-mode product-closure
      umbrella.
- [x] Wire the lane into the umbrella docs, `docs/workstreams/README.md`, `docs/roadmap.md`, and
      `docs/todo-tracker.md`.
- [x] Freeze that this lane does not widen `fret-authoring::Response`, `crates/fret-ui`, or the
      broader menu/tab policy backlog.

## M0 - Baseline and owner freeze

- [x] Write one assumptions-first baseline audit that re-reads:
      - `begin_menu[_with_options]`,
      - `begin_submenu[_with_options]`,
      - `tab_bar[_with_options]` / `begin_tab_item[_with_options]`,
      - the current helper behavior tests,
      - and the current lifecycle-lane defer notes.
      Result: `M0_BASELINE_AUDIT_2026-04-13.md`.
- [x] Freeze the default owner split for this lane.
      Result: `DESIGN.md` now keeps helper-owned trigger response shape in `fret-ui-kit::imui`,
      focused trigger behavior proof in `fret-imui`, and source/demo proof in
      `apps/fret-examples`.
- [x] Name the first-open proof and gate surfaces before deciding any API shape.
      Result: `EVIDENCE_AND_GATES.md` now uses existing menu/submenu/tab helper tests plus one
      source-policy freeze gate as the lane floor.

## M1 - Contract decision

- [ ] Decide whether helper-owned menu/submenu/tab triggers should keep the current `bool open` /
      no-return posture or gain a narrow outward response surface.
- [ ] If a new outward surface is justified, freeze the smallest return-shape budget without
      widening `fret-authoring::Response` or inventing a second response transport.
- [ ] Keep richer menu-bar/submenu/tab policy depth and key-owner semantics out of this lane even
      if a new outward trigger surface lands.

## M2 - Proof and closeout

- [ ] If the result is a no-new-API verdict, add one explicit source/proof note plus focused gates
      and close the lane.
- [ ] If the result is a new outward response surface, add focused `fret-imui` tests and one
      first-open demo/source gate before claiming the surface is real.
- [ ] Start another narrower follow-on instead of widening this lane if the pressure shifts from
      trigger response shape to richer menu/tab policy.
