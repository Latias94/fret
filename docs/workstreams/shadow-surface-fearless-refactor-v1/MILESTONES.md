# Shadow Surface (Fearless Refactor v1) — Milestones

Status: Complete (v1 closure landed; future upgrades need a new workstream or ADR)

Last updated: 2026-04-01

Related:

- Design: `docs/workstreams/shadow-surface-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/shadow-surface-fearless-refactor-v1/TODO.md`

## M0 - Baseline freeze

Status note (2026-04-01): exit criteria are now satisfied. The active `new-york-v4` shadow preset
lanes used by first-party recipes are captured in source/golden-backed evidence, current
`ShadowStyle` / `DropShadowV1` consumers are classified in the workstream notes, and shadow-facing
audit docs now point at explicit gates or intentional retained baselines instead of unclassified
drift.

Exit criteria:

- Upstream shadow values for the active shadcn preset family are captured in evidence, not memory.
- First-party `ShadowStyle` and `DropShadowV1` consumers are inventoried.
- Stale docs that currently overstate shadow parity have at least a status note.

## M1 - Preset parity closure

Status note (2026-04-01): Exit criteria are now satisfied. The preset audit/alignment for
`shadow-xs/sm/md/lg/xl` is complete, and shadcn new-york now seeds explicit shadow geometry metrics
for those audited lanes while keeping one semantic `shadow` color token plus preset-owned alpha.

Exit criteria:

- `shadow-xs/sm/md/lg/xl` are audited and aligned to the chosen source of truth for the
  first-party shadcn baseline.
- shadcn theme seeding either:
  - provides explicit `component.shadow.*` values, or
  - records an explicit decision for why fallback-only remains acceptable.

## M2 - Gate closure

Status note (2026-04-01): Exit criteria are currently satisfied by the dedicated card shadow gate,
the additional calendar shadow gate, and the linked audit evidence anchors.

Exit criteria:

- Card shadow parity is protected by a dedicated gate.
- At least one additional non-overlay shadow surface is also protected by a gate.
- Audit docs point to those gates as evidence.

## M3 - Contract decision

Status note (2026-04-01): Exit criteria are now satisfied. ADR 0060 now documents the explicit
coexistence posture with ADR 0286, and the alignment matrix points to the current `ShadowStyle` /
`DropShadowV1` evidence anchors.

Exit criteria:

- The repo has one explicit current statement for the relationship between:
  - `ShadowStyle`,
  - portable no-blur baseline rendering,
  - and `DropShadowV1`.
- If ADR 0060 no longer describes reality, it is updated or superseded.

## M4 - Cleanup and delete-ready closure

Status note (2026-04-01): exit criteria are now satisfied. Stale-doc cleanup and the remaining
manual `ShadowStyle` site audit are complete, most surviving sites are documented as intentional
product-owned surfaces or animation helpers, and the previously open generic toast fallback shadow
is now proven source-aligned by dedicated Sonner light/dark gates and retained intentionally as
shared toast chrome.

Exit criteria:

- Delete-ready stale shadow mappings or stale docs have been removed or reclassified as intentional
  retained behavior with explicit evidence.
- Any remaining lower-fidelity portable path is documented as intentional, not accidental.
- The workstream leaves behind a stable enough shadow story that future shadcn parity work does not
  need to rediscover the same contract split.
