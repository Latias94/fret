# A11y + AccessKit xplat bridge v1 — Milestones

Status: Draft

Tracking doc: `docs/workstreams/a11y-accesskit-xplat-bridge-v1.md`

## M0 — Audit snapshot (portable layers)

Done when:

- We have an explicit “what exists today” audit (schema/snapshot/mapping/tests) with evidence
  anchors.
- Gaps and risks are written down and link to follow-up trackers.

## M1 — AccessKit upgrade compiles (no behavior change)

Done when:

- Workspace uses `accesskit 0.24.0`.
- `cargo nextest run -p fret-a11y-accesskit` is green.
- No runner glue behavior changes are landed yet (pure dependency + compile fixes).

## M2 — xplat adapter implemented and wired (behavior change)

Done when:

- `crates/fret-runner-winit/src/accessibility.rs` is no longer a no-op.
- OS accessibility activation triggers redraw and consumes `TreeUpdate`s.
- Action requests are drained and routed through existing driver hooks.
- Gates are green:
  - `cargo nextest run -p fret-runner-winit`
  - `cargo nextest run -p fret-a11y-accesskit`
  - `python3 tools/check_layering.py`

## M3 — Manual acceptance closure (Windows + 1)

Done when:

- `docs/a11y-acceptance-checklist.md` passes on Windows.
- It passes on at least one additional desktop platform (macOS or Linux).

## M4 — Hardening + cleanup

Done when:

- Extra semantics validation checks are added (cheap and portable).
- A bridge kill-switch exists for debugging.
- We decide whether to keep or remove the legacy `accesskit_winit`-based file.

## M5 — Optional follow-ups (separate workstreams)

These should not block M2/M3:

- Text editor semantics closure for large buffers (excerpt policy + stable ranges).
- Table/grid semantics work (requires schema expansion + ADR).
- Web accessibility bridge (out of scope for v1; see ADR 0180 context).

