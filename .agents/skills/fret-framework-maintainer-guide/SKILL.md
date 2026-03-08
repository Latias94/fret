---
name: fret-framework-maintainer-guide
description: 'This skill should be used when the user asks to "land a framework change", "change a hard contract", "update/add an ADR", "add diagnostics/perf gates", or "do upstream parity work". Provides a contract-first maintainer playbook for safe evolution (ADRs, boundaries, diag/perf, shadcn/Radix/Base UI alignment, evidence discipline).'
---

# Fret framework maintainer guide (contract-first)

This skill is the **maintainer entrypoint** for framework development in the Fret mono-repo.

It is intentionally checklist-oriented: the goal is to make changes reviewable and reversible by
leaving the right artifacts behind.

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

1. Decide the ownership layer (mechanism vs policy vs recipe).
2. Read the relevant reference note before coding.
3. Build the smallest repro and add a regression gate.
4. Update ADR/alignment when the change touches a hard contract.
5. Validate boundaries, diagnostics evidence, and release impact.

## Workflow

### 0) Read the relevant reference note first

Use these notes to keep the main skill lean:

- Contract changes, ADRs, diagnostics, perf gates, and refactors:
  - `.agents/skills/fret-framework-maintainer-guide/references/contract-change-checklist.md`
- Motion work, upstream reference mapping, and GPU-first translation notes:
  - `.agents/skills/fret-framework-maintainer-guide/references/upstream-and-motion-notes.md`

### 1) Decide the ownership layer (non-negotiable)

Rule of thumb:

- `crates/*`: mechanisms + hard contracts (routing, focus primitives, overlay roots, semantics, diagnostics protocol).
- `ecosystem/*`: policy + composition + recipes (dismiss/focus restore rules, roving/typeahead, shadcn recipes).

If the change is “interaction policy”, it almost never belongs in `crates/fret-ui`.

### 2) Lock the smallest repro + gate

- Prefer a smallest runnable surface first (demo/gallery/script).
- Add at least one gate for any behavior change:
  - unit test,
  - diag script,
  - perf gate/baseline.
- Leave evidence anchors and exact commands so reviewers can reproduce the result quickly.

### 3) Update ADR + alignment when contracts change

If the change touches input, focus, overlays, text, diagnostics, or other hard-to-change contracts:

- update/add an ADR under `docs/adr/`, and
- update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` when the ADR is tracked there.

### 4) Hand off release-facing changes explicitly

If the framework change affects publishable crates or release automation:

- use `fret-release-check-and-publish` for preflight checks, version-group decisions, and CI publish troubleshooting.

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
- This skill’s references:
  - `.agents/skills/fret-framework-maintainer-guide/references/contract-change-checklist.md`
  - `.agents/skills/fret-framework-maintainer-guide/references/upstream-and-motion-notes.md`

## Examples

- Example: landing a contract change safely
  - User says: "We need to change input/focus behavior—how do we do it safely?"
  - Actions: update the ADR, add a repro + gate + evidence anchors, then validate boundaries.
  - Result: a durable contract change that is reviewable and reversible.

## Common pitfalls

- Landing a behavior change without a gate.
- Fixing policy mismatches by adding runtime knobs in `crates/fret-ui`.
- Doing parity work without locking it with a test/script.
- Tweaking motion “by feel” without fixed-delta diag gates.

## Troubleshooting

- Symptom: behavior change is hard to validate.
  - Fix: add a minimal scripted repro under `tools/diag-scripts/` and keep the packed bundle as evidence.
- Symptom: refactor crosses multiple crates.
  - Fix: pull in `fret-boundary-checks` early to avoid portability regressions.

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
