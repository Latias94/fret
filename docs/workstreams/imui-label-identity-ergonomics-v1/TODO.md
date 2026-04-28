# ImUi Label Identity Ergonomics v1 - TODO

Status: active execution lane
Last updated: 2026-04-28

## M0 - Baseline

- [x] Confirm the geometry lane is closed and this is a new narrow follow-on.
- [x] Confirm the old public authoring identity lane is closed and not the right execution owner.
- [x] Confirm existing IMUI has explicit `ui.id` / `ui.push_id` but no label grammar.
- [x] Add the lane to repo-level workstream indexes.
- [x] Run the initial catalog / JSON / diff hygiene gates.

## M1 - Parser Contract

- [x] Add a small `fret-ui-kit::imui` label identity parser.
- [x] Cover no marker, `##`, `##hidden`, `###`, and empty-visible-label cases.
- [x] Keep the parser private unless a real cross-crate consumer appears.

## M2 - Control Adoption

- [x] Route the button family through the parser.
- [x] Hide `##` / `###` suffixes from button rendered labels.
- [x] Preserve explicit button `a11y_label` and `test_id` override behavior.
- [x] Add one `fret-imui` authoring proof for stable `###` identity across visible-label changes
      and reorder.
- [x] Extend parser adoption to selectable rows.
- [x] Extend parser adoption to menu item rows.
- [ ] Decide whether checkbox/radio/switch/slider should key by label grammar or only render the
      visible label in this lane.

## M3 - Closeout

- [ ] Update `EVIDENCE_AND_GATES.md` with the final gate set.
- [ ] Add a closeout note naming adopted and deferred controls.
- [ ] Update repo-level workstream indexes with the closeout state.
