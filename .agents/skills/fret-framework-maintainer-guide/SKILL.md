---
name: fret-framework-maintainer-guide
description: 'This skill should be used when the user asks to "land a framework change", "change a hard contract", "update/add an ADR", "add diagnostics/perf gates", or "do upstream parity work". Provides a contract-first maintainer playbook for safe evolution (ADRs, boundaries, goal-backward verification, diag/perf, shadcn/Radix/Base UI alignment, evidence discipline).'
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
- Which authoring surface is affected: `fret` facade, direct ecosystem crate usage, or first-party UI Gallery exemplar?

Defaults if unclear:

- Start from the smallest in-tree repro surface, and ship the deliverables 3-pack (Repro + Gate + Evidence).
- If the change affects first-party shadcn authoring or docs, start from the UI Gallery snippet/page surface and the current crate-usage/shadcn docs before touching internals.

## Smallest starting point (one command)

- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

## Quick start (maintainer loop)

1. Decide the ownership layer (mechanism vs policy vs recipe).
2. Decide which public authoring surface is being changed before copying or removing APIs.
3. Read the relevant reference note before coding.
4. Write the must-be-true outcomes before choosing gates.
5. Build the smallest repro and add a regression gate.
6. Update ADR/alignment when the change touches a hard contract.
7. Validate boundaries, diagnostics evidence, docs/exemplars, and release impact.

## Workflow

### 0) Read the relevant reference note first

Use these notes to keep the main skill lean:

- Contract changes, ADRs, diagnostics, perf gates, and refactors:
  - `.agents/skills/fret-framework-maintainer-guide/references/contract-change-checklist.md`
- Shared goal-backward verification:
  - `.agents/skills/fret-skills-playbook/references/goal-backward-verification.md`
- Motion work, upstream reference mapping, and GPU-first translation notes:
  - `.agents/skills/fret-framework-maintainer-guide/references/upstream-and-motion-notes.md`
- Public authoring surface and first-party shadcn exemplar guidance:
  - `docs/crate-usage-guide.md`
  - `docs/shadcn-declarative-progress.md`
  - `.agents/skills/fret-shadcn-source-alignment/references/ui-gallery-exemplar-and-evidence.md`

### 0.25) Decide whether the real change is contract, policy, or teaching surface

Before widening or deleting APIs, answer:

- Is this a runtime/mechanism change, or is the current pain only in the first-party example surface?
- Does `fret` app-facing guidance need to change, or only direct `fret_ui_shadcn` usage?
- Should the first visible fix land in `apps/fret-ui-gallery/src/ui/snippets/**` before deeper refactors?

If the drift is “we are teaching the wrong way to author this in Fret”, fix the exemplar/docs surface
and gates first, then shrink or move internals.

### 1) Decide the ownership layer (non-negotiable)

Rule of thumb:

- `crates/*`: mechanisms + hard contracts (routing, focus primitives, overlay roots, semantics, diagnostics protocol).
- `ecosystem/*`: policy + composition + recipes (dismiss/focus restore rules, roving/typeahead, shadcn recipes).
- `apps/fret-ui-gallery`: first-party exemplar/teaching surface; use it when the public story or diagnostics surface is drifting.

If the change is “interaction policy”, it almost never belongs in `crates/fret-ui`.

### 2) Derive must-be-true outcomes first

Before choosing tests or editing docs, write 3-5 outcome truths:

- what must be true when this framework change is genuinely done,
- which truths are behavior outcomes versus teaching-surface outcomes,
- and what artifacts/wiring each truth depends on.

Do not treat “implemented the refactor” as proof that the contract has changed correctly.

### 3) Lock the smallest repro + gate

- Prefer a smallest runnable surface first (demo/gallery/script).
- Add at least one gate for any behavior change:
  - unit test,
  - diag script,
  - perf gate/baseline.
- Prefer the smallest evidence artifact that explains the change:
  - geometry assertions or `capture_layout_sidecar` for layout ownership/size negotiation
  - `capture_screenshot` for visible chrome/clipping/focus rings
  - `capture_bundle` for interaction state machines and shareable run context
- Choose the gate against the outcome truths, not against the edit list.
- Leave evidence anchors and exact commands so reviewers can reproduce the result quickly.

### 4) Update ADR + alignment when contracts change

If the change touches input, focus, overlays, text, diagnostics, or other hard-to-change contracts:

- update/add an ADR under `docs/adr/`, and
- update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` when the ADR is tracked there.

If the change also alters a public authoring/facade story:

- update `docs/crate-usage-guide.md` when dependency or facade guidance changes,
- update `docs/shadcn-declarative-progress.md` when shadcn golden-path authoring changes,
- update UI Gallery snippet/page exemplars when first-party examples would otherwise teach stale APIs.

### 5) Hand off release-facing changes explicitly

If the framework change affects publishable crates or release automation:

- use `fret-release-check-and-publish` for preflight checks, version-group decisions, and CI publish troubleshooting.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro, Gate, Evidence (anchors + exact commands). See `fret-skills-playbook`.
- The change has an explicit must-be-true outcome set, even if it stays small and local to the current task.
- Contract changes are documented (ADR + alignment update when applicable).
- A regression artifact exists (test/script/perf gate) and is runnable by others.
- Evidence anchors point to the implementation and the gate (no “trust me”).

## Evidence anchors

- Architecture/layering: `docs/architecture.md`, `docs/dependency-policy.md`
- Goal-backward verification note: `.agents/skills/fret-skills-playbook/references/goal-backward-verification.md`
- Crate/layer usage map: `docs/crate-usage-guide.md`
- Shadcn authoring golden path: `docs/shadcn-declarative-progress.md`
- ADRs + alignment: `docs/adr/`, `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- Diagnostics/perf runbook: `.agents/skills/fret-diag-workflow/SKILL.md`, `tools/diag-scripts/`, `tools/perf/`
- Upstream parity: `.agents/skills/fret-shadcn-source-alignment/SKILL.md`
- UI Gallery exemplar + evidence note: `.agents/skills/fret-shadcn-source-alignment/references/ui-gallery-exemplar-and-evidence.md`
- UI Gallery authoring gates: `apps/fret-ui-gallery/src/lib.rs`
- UI Gallery geometry/test-id helpers: `apps/fret-ui-gallery/src/driver/render_flow.rs`
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
- Changing a public authoring surface without updating the docs and first-party exemplars that teach it.
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
