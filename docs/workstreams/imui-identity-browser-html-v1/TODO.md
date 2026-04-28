# ImUi Identity Browser HTML v1 - TODO

Status: closed
Last updated: 2026-04-28

## M0 - Tracking

- [x] Start a narrow follow-on from `imui-id-stack-browser-v1`.
- [x] Record assumptions-first scope and non-goals.
- [x] Add the lane to repo-level workstream indexes.

## M1 - Renderer Model

- [x] Add a focused offline HTML renderer over the identity browser report.
- [x] Escape source paths, element paths, and unknown strings safely.
- [x] Keep the renderer independent of live devtools transport.
- [x] Add fixture tests for summary, groups, rows, and escaping.

M1 result:

- `crates/fret-diag/src/identity_browser_html.rs` renders a self-contained diagnostic page from the
  shared identity browser report.
- Fixture coverage checks summary/group/row rendering and escaping for source and element strings.

## M2 - CLI Surface

- [x] Add a bounded command/flag to write the HTML sidecar from `diag query identity-warnings`.
- [x] Preserve existing JSON and human output behavior unless HTML output is requested.
- [x] Add contract/cutover tests for the new CLI surface.

M2 result:

- `diag query identity-warnings --html-out <path>` writes the offline HTML sidecar.
- Existing `--json`, `--out`, and human row behavior remain compatible.

## M3 - Closeout Readiness

- [x] Record final gates and evidence.
- [x] Split dashboard/live devtools work into narrower follow-ons if still useful.
- [x] Close or downgrade the lane once offline HTML browsing is stable.
