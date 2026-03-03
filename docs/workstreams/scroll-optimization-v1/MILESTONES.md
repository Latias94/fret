# Scroll Optimization Workstream (v1) — Milestones

Date: 2026-03-03  
Status: Draft

## M0 — Baseline + evidence (1–2 days)

- Establish a minimal “scroll correctness” script set (ui-gallery).
- Add a thumb-drag stability repro + gate script.
- Document current invariants (HitTestOnly scrolling, nested wheel routing).

## M1 — Mechanism hardening (2–4 days)

- Reduce barrier/scroll foot-guns (single helper paths where possible).
- Add unit tests around barrier relayout + subtree dirty aggregation.

## M2 — Wheel/trackpad coalescing prototype (3–5 days)

- Implement an opt-in coalescing mode.
- Add a torture script for wheel input and basic perf telemetry capture (bundle capture; perf threshold TBD).
- Ensure nested scrollables still route correctly (deepest-first).

## M3 — Scrollbar drag baseline lock (2–4 days)

- Stabilize thumb while dragging under content changes.
- Add a deterministic gate (diag script + bounded assertions on semantics).

## M4 — Extents observation hardening (2–4 days)

- Expand post-layout overflow observation coverage with gates.
- Validate budget-hit fallback probes prevent pinned extents.
