# Perf Devtools: Skills and Workflow (v1)

Status: Draft (workstream note; ADRs remain the source of truth)

This workstream documents how we want contributors to **investigate, explain, and prevent** performance regressions
in Fret, using repo-local Agent Skills and `fretboard diag` tooling.

Goal: turn “perf feels off” into a repeatable, commit-addressable loop:

1) reproduce via a stable `tools/diag-scripts/*.json` probe,
2) quantify via `fretboard diag perf` + baselines/gates,
3) attribute via bundles (CPU phases + renderer churn signals, and optional external profilers),
4) prevent regressions via gates + perf log entries.

Related:

- Perf workstream plan: `docs/workstreams/ui-perf-zed-smoothness-v1.md`
- Perf TODO tracker: `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md`
- Perf log: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`
- GPUI gap map: `docs/workstreams/ui-perf-gpui-gap-v1.md`

---

## 0) Current skill landscape

Repo-local skills live under `.agents/skills/`.

- `fret-diag-workflow`: author/run `tools/diag-scripts/*.json`, capture bundles/screenshots, compare and gate
  correctness regressions.
- `fret-perf-workflow`: run perf suites, maintain baselines, run majority-gated resize probes, and record
  commit-addressable evidence in the perf log.

Design intent:

- `fret-diag-workflow` owns “make it reproducible”.
- `fret-perf-workflow` owns “make it measurable and enforceable”.

---

## 1) The gaps (what still slows people down)

Observed friction points:

1) **Attribution is still too manual** for newcomers.
   - After a gate fails, you still need local knowledge to answer:
     “Is this layout solve, text prepare, scene rebuild, atlas upload, or GPU stall?”
2) **Deep profiling recipes are not standardized**.
   - We have strong in-bundle CPU phase counters, but “what next?” (CPU stacks / allocations / GPU capture)
     is inconsistent across contributors and machines.
3) **Probe authoring lacks a single checklist**.
   - People create scripts, but forget stable `test_id`, forget the “worst bundle” link, or skip turning the probe into
     a gate once explainable.

---

## 2) Proposed improvements (skills + tooling)

### 2.1 Extend `fret-perf-workflow` with “attribution recipes”

Add a dedicated section (and optional helper scripts) covering:

- CPU stack profiling:
  - macOS: Instruments Time Profiler (preferred for resize/scroll hitches).
  - Linux: `perf record`/`perf report`.
  - Windows: WPA/ETW (best-effort).
- Allocation profiling:
  - macOS: Instruments Allocations (look for per-frame hot allocations on hitch frames).
- GPU/renderer capture:
  - RenderDoc / Tracy integration (best-effort; document the “works on my machine” constraints).

The skill should be explicit about the handoff:

- start with a failing gate + worst bundle,
- use bundle stats to choose the profiler mode,
- capture only the minimal scenario (one script),
- record the capture metadata in the perf log entry (where it lives, how to reproduce).

### 2.2 Promote “render-time setter idempotency” as a first-class checklist item

Codify a default audit step for declarative render loops:

- If `handle.set_*` can be called during render, it must be a no-op when the next value equals the current value.
- Reference ledger: `docs/workstreams/ui-perf-setter-idempotency-v1.md`.

This turns a high-impact, easy-to-miss footgun into a standard part of perf triage.

### 2.3 Consider a new skill: `fret-perf-attribution`

If `fret-perf-workflow` becomes too large, split out a dedicated attribution skill that focuses on:

- reading bundles (`diag stats`, renderer churn counters),
- deciding what to measure next,
- using external profilers and integrating evidence back into workstreams logs.

---

## 3) Milestones

### M0: Conformance + discoverability

- Validate all `fret-*` skills against the Agent Skills reference validator (`repo-ref/agentskills/skills-ref`).
- Ensure every skill cross-links to:
  - the canonical docs it relies on,
  - the “handoff” skills (diag ↔ perf).

### M1: Attribution playbooks (repeatable)

- Add cross-platform “CPU stack / alloc / GPU capture” recipes to the perf workflow skill.
- Add at least one “worked example” entry linked to an existing perf log bundle (resize or editor probe).

### M2: Gate authoring checklist (reduce footguns)

- Publish a one-page checklist for adding a new perf probe:
  - stable `test_id`,
  - minimal script,
  - baseline creation and validation runs,
  - gate integration,
  - perf log entry template.

---

## 4) Evidence / audit trail

This workstream should stay evidence-linked:

- When a skill changes behavior, record the change in the relevant workstream log and include:
  - commit hash,
  - what it enables (faster triage, fewer manual steps),
  - one “before/after” example invocation.

### Recent changes

- Added a dedicated hitch attribution skill:
  - Skill: `.agents/skills/fret-perf-attribution/SKILL.md`
  - Rationale: keep `fret-perf-workflow` focused on gates/baselines while providing a deeper “tail latency”
    playbook for CPU vs GPU triage and external profilers.
- Streamlined the “P0 resize jank” loop across diag/perf skills:
  - Skills:
    - `.agents/skills/fret-diag-workflow/SKILL.md`
    - `.agents/skills/fret-perf-workflow/SKILL.md`
  - Change: add copy/paste “resize fast path” + document how to append gate attempts into the perf log.
  - Evidence: commit `ea13d4015`.
- Stabilized a flaky steady-suite navigation script (so perf gating does not fail due to `wait_until_timeout`):
  - Scripts:
    - `tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json`
    - `tools/diag-scripts/ui-gallery-material3-tabs-switch-perf.json`
  - Change: add a short settle delay after scrolling/search navigation and extend the page-mount timeout.
  - Evidence: commit `0d168fc5e`.
