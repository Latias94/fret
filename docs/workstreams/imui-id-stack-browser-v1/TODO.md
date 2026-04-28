# ImUi ID Stack Browser v1 - TODO

Status: active
Last updated: 2026-04-28

## M0 - Tracking

- [x] Start a narrow follow-on from `imui-id-stack-diagnostics-v1`.
- [x] Record assumptions-first scope and non-goals.
- [x] Add the lane to repo-level workstream indexes.

## M1 - Source Model Audit

- [ ] Map the current `identity_warnings` bundle shape into a browser-ready row model.
- [ ] Decide whether existing fields are enough for first-use browsing.
- [ ] Add fixture coverage for duplicate-key and unkeyed-reorder rows.
- [ ] Keep missing capture-side fields as explicit follow-on candidates unless they block browsing.

## M2 - Browser Query Surface

- [ ] Add a bounded `fret-diag` identity browser model or command over schema2 bundle snapshots.
- [ ] Support grouping by warning kind, source file, list id, element path, and frame/window.
- [ ] Preserve `--json` / `--out` style evidence for automation.
- [ ] Add focused contract/cutover tests if the public CLI surface grows.

## M3 - Interactive Experience

- [ ] Decide whether the first interactive surface belongs in `diag dashboard`, a browser-ready JSON
  sidecar, or a dedicated `diag identity-browser` command.
- [ ] Provide keyboard/filter semantics or a bounded HTML/dashboard view if the implementation
  leaves pure query mode.
- [ ] Keep live connected devtools transport out of scope unless bundle-mode browsing proves
  insufficient.

## M4 - Closeout Readiness

- [ ] Record final gates and evidence.
- [ ] Split live devtools, `test_id` inference, localization, and table column identity into
  separate follow-ons if they remain useful.
- [ ] Close or downgrade the lane once a browser-ready identity diagnostics surface is stable.
