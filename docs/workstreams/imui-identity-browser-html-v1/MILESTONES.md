# ImUi Identity Browser HTML v1 - Milestones

Status: closed
Last updated: 2026-04-28

## M0 - Tracking

Exit criteria:

- the lane is documented as a narrow follow-on of the closed browser model/query lane,
- repo-level indexes point to the new workstream,
- and the first slice is limited to offline HTML over captured diagnostics.

## M1 - Renderer Model

Exit criteria:

- a self-contained HTML renderer exists over the identity browser report,
- escaping tests cover source and element strings,
- and the renderer reuses the shared browser model rather than parsing bundle JSON directly.

Result:

- Landed in `crates/fret-diag/src/identity_browser_html.rs`.
- Tests cover summary, groups, rows, JSON detail rendering, and HTML escaping.
- The renderer consumes `IdentityWarningBrowserReport`.

## M2 - CLI Surface

Exit criteria:

- `fret-diag` can write the offline identity browser HTML artifact,
- existing `identity-warnings` query JSON remains compatible,
- and focused tests cover the new flag/command wiring.

Result:

- Landed as `diag query identity-warnings --html-out <path>`.
- CLI contract and cutover tests cover the new flag.
- Query fixture tests cover sidecar writing.

## M3 - Follow-on Decision

Exit criteria:

- final gates are recorded,
- `WORKSTREAM.json` reflects the current lane state,
- and richer dashboard/live ideas are split into narrower follow-ons.

Result:

- Lane closed on 2026-04-28.
- Dashboard and live devtools work remain separate follow-on candidates.
