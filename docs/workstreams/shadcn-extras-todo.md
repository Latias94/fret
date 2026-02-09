# Shadcn Extras (`fret-ui-shadcn::extras`) — TODO Tracker

This document tracks executable TODOs for adding a small “extras” surface to `fret-ui-shadcn`.

Narrative + boundaries: `docs/workstreams/shadcn-extras.md`  
Milestones (one-screen): `docs/workstreams/shadcn-extras-milestones.md`

## Non-goals (keep this explicit)

- Do not change `crates/fret-ui` public contracts for extras; propose ADRs separately if needed.
- Do not duplicate shadcn/ui v4 components already present in `fret-ui-shadcn` root modules.
- Do not move AI-native / policy-heavy surfaces into extras. Those are owned by `ecosystem/fret-ui-ai`
  (`docs/workstreams/ai-elements-port.md`).

## M0 — Extras skeleton + gates

- [x] Add `ecosystem/fret-ui-shadcn/src/extras/mod.rs` with module docs and export policy.
- [x] Add `pub mod extras;` to `ecosystem/fret-ui-shadcn/src/lib.rs` (no root re-exports).
- [x] Add snapshot coverage for at least one extras root (new snapshot file under
      `ecosystem/fret-ui-shadcn/tests/snapshots/*.json`).
- [x] Add a small “component template” section to `docs/workstreams/shadcn-extras.md` (or a dedicated
      appendix) that standardizes:
      - controlled/uncontrolled pattern (use `fret-ui-kit::declarative::controllable_state`),
      - semantics roles + labels,
      - `test_id` conventions,
      - and required gates.

## Candidate components (staged)

Legend:

- **Source**: upstream inspiration (local snapshots under `repo-ref/` when available)
- **Owner**: target layer for the state machine/policy split
- **Gate**: minimum regression gate required for landing

### M1: Low-risk composition blocks

| Component | Source | Owner split | Gate |
| --- | --- | --- | --- |
| `Banner` | `repo-ref/kibo/packages/banner` (MIT) | policy in extras, uses `Button` from shadcn | snapshot |
| `Announcement` | `repo-ref/kibo/packages/announcement` (MIT) | pure composition over `Badge` | snapshot |
| `Tags` (static list) | (choose permissive source) | pure composition over `Badge/Button` | snapshot |
| `Rating` | `repo-ref/kibo/packages/rating` (MIT) | headless bits may move to `fret-ui-headless` later | snapshot + scripted keyboard |
| `RelativeTime` (display-only) | `repo-ref/kibo/packages/relative-time` (MIT) | no timers in M1 | snapshot |

### M2: Medium complexity

| Component | Source | Owner split | Gate |
| --- | --- | --- | --- |
| `AvatarStack` | `repo-ref/kibo/packages/avatar-stack` (MIT) | composition; avoid web-only mask tricks | snapshot |
| `Snippet` / `CodeBlock` | (coordinate with existing crates) | may live outside extras | TBD |

### M3: Scheduling/animation-heavy

| Component | Source | Owner split | Gate |
| --- | --- | --- | --- |
| `RelativeTime` (auto-updating) | `repo-ref/kibo/packages/relative-time` (MIT) | scheduling policy; perf risk | scripted + perf note |
| `Marquee` / `Ticker` | `repo-ref/kibo/packages/marquee`, `.../ticker` (MIT) | continuous frames lease | scripted + perf gate |

## Per-component checklist (copy/paste into PRs)

- [ ] Clear API name and module path under `extras`.
- [ ] Controlled vs uncontrolled story is explicit (models, defaults).
- [ ] Semantics role/label is present and stable (selectors for diag scripts).
- [ ] `test_id` hooks exist for key parts when needed.
- [ ] At least one regression gate lands (snapshot or diag script).
- [ ] No new runtime contract changes; no platform deps.
- [ ] Rustdoc includes upstream inspiration + license note (short).
