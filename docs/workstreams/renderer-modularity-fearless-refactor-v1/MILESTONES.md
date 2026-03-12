# Renderer Modularity (Fearless Refactor v1) — Milestones

Status: In progress

Related:

- Purpose: `docs/workstreams/renderer-modularity-fearless-refactor-v1/README.md`
- Design: `docs/workstreams/renderer-modularity-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/renderer-modularity-fearless-refactor-v1/TODO.md`

Current snapshot (2026-03-12):

- The renderer stack is not a rewrite candidate; it is a staged modularization candidate.
- `fret-render-wgpu` baseline gates are green:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `python3 tools/check_layering.py`: passed
- v1 start decisions are now locked:
  - no new renderer crates in v1,
  - `fret-render` stays the stable default facade,
  - `fret-render-core` stays portable/value-only,
  - host-provided GPU topology closure is P0,
  - render-plan semantics are treated as frozen inputs,
  - `text/mod.rs` is the first high-value breakup target.
- The strongest current risks are:
  - wildcard facade exports,
  - oversized backend public surface,
  - a large multi-domain `Renderer` state owner,
  - and incomplete ergonomic closure for engine-hosted topology helpers.

---

## M0 — Problem and baseline locked

Exit criteria:

- The current public facade and backend export surface are inventoried.
- The current first-party consumers are grouped by usage pattern.
- Baseline gates are green and recorded in this workstream.
- Render-plan semantics are explicitly treated as fixed inputs to the refactor.

---

## M1 — Stable facade contract closed

Exit criteria:

- `crates/fret-render` no longer behaves like a wildcard backend dump.
- The intended stable default renderer surface is explicit.
- Portable value contracts have a clear ownership story.
- Callers can see which surfaces are "default facade contract" vs "backend-specific detail."

---

## M2 — Host-provided GPU topology becomes first-class

Exit criteria:

- Capability/bootstrap helpers no longer force `WgpuContext` as the only ergonomic entrypoint.
- Engine-hosted integration has at least one explicit smoke path or first-party example.
- Docs/examples clearly show both editor-hosted and engine-hosted topology entrypoints.

---

## M3 — Internal domains extracted behind stable semantics

Exit criteria:

- `text/mod.rs` is split into explicit subdomains.
- `Renderer` no longer directly owns every subdomain in one large state block.
- Service trait implementations remain readable and behavior-preserving.
- Conformance and render-plan semantics tests remain green.

Recommended first slice:

- text system breakup
- renderer state/domain breakup
- capability/bootstrap seam cleanup

---

## M4 — Public export tightening completed

Exit criteria:

- Low-value public exports have been reviewed and either justified or removed.
- Cache/registry/internal-only backend types are no longer public by default unless intentionally
  part of the stable story.
- First-party callers compile and run against the curated facade.

---

## M5 — Contract closure and cleanup

Exit criteria:

- Any required ADRs for renderer facade/topology changes have been added or updated.
- Workstream docs reflect the final stable story.
- We can explain, in one short page, which layer owns:
  - portable render contracts,
  - default facade compatibility,
  - backend-specific implementation details.

---

## M6 — Optional follow-up: deeper crate boundary changes

Exit criteria:

- A conscious decision exists on whether further crate splits are still needed.
- If yes, the next split is documented separately with scope and risks.
- If no, this workstream closes with modularity improvements delivered inside the existing crate
  layout.

This milestone is intentionally optional. It should only start after facade closure and internal
domain extraction have already lowered the risk surface.
