# ImUi Debug Draw Channel Split v1 Milestones

Status: Closed.

## M0 - Buffering Model

Exit criteria:

- Active channel commands keep using the existing `commands` vector.
- Switching channels saves/restores command buffers without touching individual draw helpers.
- Merge flattens channels by channel index.

Result: Complete.

## M1 - Public API

Exit criteria:

- `ImUiDebugDrawList` exposes split, switch, and merge helpers.
- Open splits are auto-merged before the list is consumed by the Canvas element.
- Invalid channel switches are no-ops.

Result: Complete.

## M2 - Tests and Evidence

Exit criteria:

- Unit tests prove channel order flattening.
- Smoke tests prove the public API compiles.
- Workstream docs record residual DrawList gaps.

Result: Complete.
