# ImUi Identity Browser Visual Gate v1 - Milestones

Status: closed
Last updated: 2026-04-28

## M0 - Tracking

Result:

- Created as a narrow follow-on of the closed offline HTML browser lane.
- Scope is limited to deterministic smoke gates.

## M1 - Structural Visual Anchors

Result:

- Added stable `data-testid` anchors to the generated HTML artifact.
- Fixture tests prove the anchor set is present.

## M2 - Smoke Check Output

Result:

- Added `check.identity_browser_html` JSON smoke output.
- Added `diag query identity-warnings --html-check-out <path>`.
- Query fixture tests cover HTML and check sidecar generation.

## M3 - Closeout

Result:

- Lane closed on 2026-04-28.
- Browser screenshot and dashboard gates remain separate follow-on candidates.
