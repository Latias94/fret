# Foreground inheritance (late binding) v2 - Milestones

Last updated: 2026-02-24

This file defines “fearless refactor v2” milestones and exit criteria for late-bound foreground
inheritance via `ForegroundScope` (paint-time), replacing build-time inherited state.

## Milestones

### M0 - Mechanism landed (core plumbing)

Status: In progress

Exit criteria:
- [x] `ForegroundScope` exists as a layout-transparent wrapper element kind.
- [x] Paint traversal carries `PaintStyleState { foreground }`.
- [x] `PaintCacheKey` includes inherited foreground to avoid incorrect replay.
- [ ] One paint-time regression test exists that asserts inherited foreground is used for at least:
  - [ ] `SvgIcon(inherit_color=true)`
  - [ ] `Text(color=None)`

### M1 - Ecosystem leaves opt into late-binding (no build-time color baking)

Status: In progress

Exit criteria:
- [x] Icons default to `inherit_color=true` with a stable fallback.
- [ ] UI-kit text helpers do not read build-time inherited state to decide `TextProps.color`.
- [ ] Spinner defaults align (when `color=None`, prefer inherited foreground).
- [ ] Any legacy tests asserting build-time inheritance are updated to v2 semantics.

### M2 - shadcn hosts install `ForegroundScope` boundaries

Status: In progress

Exit criteria:
- [ ] High-ROI hosts migrated (buttons, menus, selects, tabs/toggles).
- [ ] No production callsites rely on v1 build-time `with_current_color_provider` for fg inheritance.
- [ ] Host boundaries are scoped correctly (no overly-wide scopes that accidentally affect unrelated descendants).

### M3 - Gallery parity + regression gates

Status: Not started

Exit criteria:
- [ ] UI gallery component pages match shadcn docs expectations for:
  - [ ] Dark background icon visibility (no “black-on-black” regressions).
  - [ ] Copy/paste-ready code snippets for the aligned components.
- [ ] Screenshot diag scripts exist for at least the most common icon-bearing surfaces (button/menu/loading).

### M4 - Cleanup (v1 provider deprecation path)

Status: Not started

Exit criteria:
- [ ] Workstream docs describe v1 vs v2 clearly with a migration path.
- [ ] v1 provider APIs are either:
  - [ ] explicitly marked “compat only” and removed from primary authoring docs, or
  - [ ] removed after downstream migration (separate milestone/PR).

