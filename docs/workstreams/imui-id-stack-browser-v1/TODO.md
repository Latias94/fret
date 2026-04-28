# ImUi ID Stack Browser v1 - TODO

Status: closed
Last updated: 2026-04-28

## M0 - Tracking

- [x] Start a narrow follow-on from `imui-id-stack-diagnostics-v1`.
- [x] Record assumptions-first scope and non-goals.
- [x] Add the lane to repo-level workstream indexes.

## M1 - Source Model Audit

- [x] Map the current `identity_warnings` bundle shape into a browser-ready row model.
- [x] Decide whether existing fields are enough for first-use browsing.
- [x] Add fixture coverage for duplicate-key and unkeyed-reorder rows.
- [x] Keep missing capture-side fields as explicit follow-on candidates unless they block browsing.

M1 result:

- `crates/fret-diag/src/identity_browser.rs` owns the browser-ready source model and collector.
- Existing capture fields are sufficient for first-use post-run browsing.
- No capture-side blocker was found; future live/devtools fields remain follow-on candidates.

## M2 - Browser Query Surface

- [x] Add a bounded `fret-diag` identity browser model over schema2 bundle snapshots.
- [x] Support grouping by warning kind, source file, list id, element path, and frame/window.
- [x] Preserve `--json` / `--out` style evidence for automation.
- [x] Add focused contract/cutover tests if the public CLI surface grows.

M2 result:

- `diag query identity-warnings --browser` exposes opt-in `summary` and `groups`.
- Default query output remains row-compatible when `--browser` is absent.
- Grouping is currently over the browser model's stable key: warning kind, window, frame id, source
  file, list id, key hash, and element path.

## M3 - Interactive Experience

- [x] Decide whether the first interactive surface belongs in `diag dashboard`, a browser-ready JSON
  sidecar, or a dedicated `diag identity-browser` command.
- [x] Keep the first surface in pure query mode, so keyboard/filter semantics and HTML/dashboard
  controls are deferred to narrower follow-ons.
- [x] Keep live connected devtools transport out of scope unless bundle-mode browsing proves
  insufficient.

## M4 - Closeout Readiness

- [x] Record final gates and evidence.
- [x] Split live devtools, `test_id` inference, localization, and table column identity into
  separate follow-ons if they remain useful.
- [x] Close or downgrade the lane once a browser-ready identity diagnostics surface is stable.
