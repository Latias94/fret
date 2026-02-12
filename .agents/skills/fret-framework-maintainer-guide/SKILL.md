---
name: fret-framework-maintainer-guide
description: "Maintainer playbook for evolving the Fret UI framework safely: contracts/ADRs, layering boundaries, diagnostics + perf gates, shadcn/Radix/Base UI alignment, and evidence discipline. Use when landing framework changes (mechanisms, policies, tooling, or contracts)."
---

# Fret framework maintainer guide (contract-first)

This skill is the **maintainer entrypoint** for framework development in the Fret mono-repo.

It is intentionally guideline/checklist-oriented (Vercel-style): the goal is to make changes
reviewable and reversible by leaving the right artifacts behind.

## When to use

- You are changing a hard-to-change contract (input/focus/overlays/text/diagnostics/perf protocol).
- You are moving code across crates and need to preserve boundaries (mechanism vs policy vs recipes).
- You are landing a diagnostics/perf workflow change and need to update scripts/gates/evidence.
- You are doing upstream parity work (shadcn/Radix/Base UI) and want to lock it with gates.

## Inputs to collect (ask the user)

- What is the user-facing invariant (what must always be true)?
- Which layer should own the change (`crates/*` mechanism vs `ecosystem/*` policy/recipes)?
- What is the smallest runnable target (demo/gallery/script) that shows the behavior?
- What regression protection is required (unit test, diag script, perf gate/baseline)?
- What upstream reference is in scope (Radix semantics, shadcn composition, Base UI headless patterns)?

Defaults if unclear:

- Start from the smallest in-tree repro surface, and ship the deliverables 3-pack (Repro + Gate + Evidence).

## Smallest starting point (one command)

- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

## Quick start (maintainer loop)

1) Decide the ownership layer (mechanism vs policy vs recipe).
2) Build a smallest repro (demo/gallery/script).
3) Add a regression gate (test/script/perf).
4) Record evidence anchors (paths + key functions + commands).
5) Run boundary checks and validate skills/docs drift.

## Workflow

### 1) Layering decision (non-negotiable)

Rule of thumb:

- `crates/*`: mechanisms + hard contracts (routing, focus primitives, overlay roots, semantics, diagnostics protocol).
- `ecosystem/*`: policy + composition + recipes (dismiss/focus restore rules, roving/typeahead, shadcn recipes).

If the change is “interaction policy”, it almost never belongs in `crates/fret-ui`.

### 2) Contract-first changes (ADRs + alignment)

If you change a contract surface (input/focus/overlays/text/diagnostics):

- Update or add an ADR under `docs/adr/`.
- If the ADR is already tracked, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with:
  - `Aligned` / `Partially aligned` / `Not implemented`,
  - 1–3 evidence anchors (paths + tests/scripts).

### 3) Diagnostics and perf gates (evidence discipline)

Treat diagnostics artifacts as first-class regression protection:

- Correctness: `tools/diag-scripts/*.json` (schema v2 preferred) + `capture_bundle`.
- Perf: `fretboard diag perf` suites or `tools/perf/*` gate scripts + worst bundle paths.
- Attribution: `fretboard diag stats <bundle.json> --sort time --top 30` plus the exact failing metric/threshold key.

Use `fret-diag-workflow` as the canonical runbook.

### 4) Upstream alignment scope (Radix + shadcn + Base UI)

Use `fret-shadcn-source-alignment` when you want parity work plus gates.

Practical mapping:

- **Radix**: semantics + state-machine outcomes (dismiss/focus/keyboard nav/placement).
- **shadcn**: composition + taxonomy + sizing defaults (recipes).
- **Base UI**: headless accessibility patterns and part composition (unstyled primitives, event/state flows).

Use Base UI as an additional reference when DOM-centric assumptions need translating to Fret’s GPU-first
custom renderer (semantics tree, hit-testing, focus routing, text/IME).

### 5) Refactors (fearless, but gated)

Before/after a refactor that may cross boundaries:

- Run layering checks: `python3 tools/check_layering.py`
- Add at least one gate for any behavior change (unit test or diag script).
- If perf is in scope, record a perf gate run + worst bundles (commit-addressable evidence).

### 6) Release-facing changes

If the change affects publishable crates or release automation:

- Use `fret-release-check-and-publish` for preflight checks and publish troubleshooting.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro, Gate, Evidence (anchors + exact commands). See `fret-skills-playbook`.
- Contract changes are documented (ADR + alignment update when applicable).
- A regression artifact exists (test/script/perf gate) and is runnable by others.
- Evidence anchors point to the implementation and the gate (no “trust me”).

## Evidence anchors

- Architecture/layering: `docs/architecture.md`, `docs/dependency-policy.md`
- ADRs + alignment: `docs/adr/`, `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- Diagnostics/perf runbook: `.agents/skills/fret-diag-workflow/SKILL.md`, `tools/diag-scripts/`, `tools/perf/`
- Upstream parity: `.agents/skills/fret-shadcn-source-alignment/SKILL.md`
- Base UI snapshot (optional): `repo-ref/base-ui/packages/react/src/`, `repo-ref/base-ui/docs/`

## Common pitfalls

- Landing a behavior change without a gate (regressions return as human-only bugs).
- Fixing policy mismatches by adding runtime knobs in `crates/fret-ui`.
- Not leaving worst bundle paths for perf work (attribution becomes non-deterministic).
- Doing parity work without locking it with a test/script (drift returns quickly).

## Related skills

- `fret-repo-orientation`
- `fret-skills-playbook`
- `fret-diag-workflow`
- `fret-ui-review`
- `fret-shadcn-source-alignment`
- `fret-boundary-checks`
- `fret-crate-audits`
- `fret-fixture-driven-harnesses`
- `fret-release-check-and-publish`
