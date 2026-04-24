# ImUi Editor Notes Draft Actions v1 - Milestones

Status: closed narrow P1 lane
Last updated: 2026-04-24

## M0 - Lane Opened

Status: complete

- Started from the closed `imui-next-gap-audit-v1` recommendation.
- Kept persistence, dirty-close, command bus, and public API widening out of scope.

## M1 - App-Owned Draft Actions

Status: complete

Goal: add inspector-local draft action buttons with stable test IDs and app-owned feedback state.

Evidence: `M1_APP_OWNED_DRAFT_ACTIONS_SLICE_2026-04-24.md`.

## M2 - Closeout Verdict

Status: complete

Goal: close if local draft actions are enough, or split any real `TextField` draft-buffer contract
into a separate narrow API-proof lane.

Verdict: close this lane and require a new narrow API-proof lane for any future preserved draft-buffer contract.
