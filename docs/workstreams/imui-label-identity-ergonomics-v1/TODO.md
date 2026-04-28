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

- [ ] Add a small `fret-ui-kit::imui` label identity parser.
- [ ] Cover no marker, `##`, `##hidden`, `###`, and empty-visible-label cases.
- [ ] Keep the parser private unless a real cross-crate consumer appears.

## M2 - Control Adoption

- [ ] Route the first admitted label-bearing controls through the parser.
- [ ] Hide `##` / `###` suffixes from rendered labels.
- [ ] Preserve explicit `a11y_label` and `test_id` override behavior.
- [ ] Add one `fret-imui` authoring proof for stable `###` identity across visible-label changes.

## M3 - Closeout

- [ ] Update `EVIDENCE_AND_GATES.md` with the final gate set.
- [ ] Add a closeout note naming adopted and deferred controls.
- [ ] Update repo-level workstream indexes with the closeout state.
