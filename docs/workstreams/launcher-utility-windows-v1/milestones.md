# Launcher + Utility Windows v1 — Milestones

Status: Draft

Milestones are landable checkpoints (each should be buildable + gated).

## M0 — Contracts aligned

Exit criteria:

- ADR 0139 updated and internally consistent.
- ADR 0310 and ADR 0311 reviewed as decision gates (still `Proposed` is fine).
- Workstream docs reviewed (README + TODO + milestones).

## M1 — Portable contract complete (no runner work yet)

Exit criteria:

- `WindowStyleRequest` includes the full v1 vocabulary (ADR 0139 + ADR 0310).
- Capability keys exist for all facets (ADR 0054).
- `WindowRequest` includes chrome actions + visibility (ADR 0311).
- Diagnostics groundwork:
  - `capabilities.json` advertises the new `diag.window_style_snapshot` and `diag.window_background_material_snapshot` capabilities (even if initially clamped/empty on unsupported runners).
  - Script predicates exist for asserting effective/clamped style/material (capability-gated; fail-fast when missing).

## M2 — Desktop runner: frameless utility window MVP

Exit criteria:

- A demo (or scripted diag) proves:
  - frameless window creation works,
  - drag + resize work via custom chrome actions (capability-gated),
  - show/hide works without destroying state.

## M3 — Desktop runner: transparent + background materials (best-effort)

Exit criteria:

- Transparent composited window behavior is capability-gated and observable.
- At least one OS material variant is implemented end-to-end (platform-specific).

## M4 — Observability hardened

Exit criteria:

- “effective/clamped window style/material” is visible in diagnostics/inspection for scripted repros.
- Basic regression gates exist (diag script predicates or stable snapshot checks).
