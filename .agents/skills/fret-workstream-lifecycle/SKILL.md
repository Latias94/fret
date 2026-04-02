---
name: fret-workstream-lifecycle
description: "This skill should be used when the user asks to \"start a new workstream\", \"resume a fearless refactor lane\", \"record workstream progress\", \"split a narrow follow-on\", or \"close out a lane\". Provides a lightweight workstream lifecycle for Fret: scaffold the right docs, keep lane state explicit, attach repro/gate/evidence to each slice, and leave closeout notes without reopening scope by accident."
---

# Fret workstream lifecycle (create, run, close out)

Use this skill to manage `docs/workstreams/*` as a lightweight execution system rather than a pile
of markdown files.

## When to use

- The user wants to create a new workstream for a feature/refactor.
- The task is to continue or narrow an existing workstream lane.
- The user wants a recommendation on “continue this lane vs create a follow-on”.
- The task is to record progress, gates, evidence, or closeout for a lane.

## Inputs to collect (ask the user)

- What problem/invariant does this lane own?
- Is this a new lane, an existing lane, or a possible follow-on to a closed lane?
- What is the smallest runnable repro target?
- What regression artifact is expected: unit test, diag script, perf gate, ADR/alignment update?
- Which crates/layers are likely in scope?

Defaults if unclear:

- Prefer continuing an active lane.
- Prefer a narrow follow-on instead of reopening a closed broad lane.
- Leave one repro, one gate, and one evidence set for each landable slice.

## Quick start

1. Read `references/workstream-skeleton.md`.
2. Read `references/workstream-state-resolution.md`.
3. Read `docs/workstreams/standalone/workstream-state-v1.md`.
4. Open `docs/roadmap.md`, `docs/workstreams/README.md`, and `docs/todo-tracker.md`.
5. If the lane already exists, resolve its current state before editing code.

## Workflow

### 1) Decide: new lane, current lane, or follow-on

Use this rule:

- **Continue current lane** when the roadmap/workstream index still treats it as active or maintenance.
- **Create a new lane** when the scope is new enough that forcing it into an existing folder would blur ownership.
- **Create a narrow follow-on** when a closed lane has fresh evidence, but the new ask would widen scope if added back to the old folder.

If the lane is already closed, do not silently reopen it from an old TODO item.

### 2) For a new lane, scaffold the smallest useful doc set

Start with:

- `WORKSTREAM.json`
- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`

Add only when the lane needs them:

- `TARGET_INTERFACE_STATE.md`
- `OPEN_QUESTIONS.md`
- dated audit/status notes
- `CLOSEOUT_AUDIT_YYYY-MM-DD.md` or `FINAL_STATUS.md`

Keep the folder small at first. Promote more notes only when they are genuinely needed.

### 3) For an existing lane, resolve authoritative docs first

If `WORKSTREAM.json` exists, read it first as the lane-state index, then open the listed
authoritative docs.

Read the lane in this order:

- repo-wide stance: `docs/roadmap.md`, `docs/workstreams/README.md`, `docs/todo-tracker.md`
- machine-readable lane state: `WORKSTREAM.json` when present
- lane positioning: `README.md` or `<slug>.md`
- current target surface: `DESIGN.md`, `TARGET_INTERFACE_STATE.md` when present
- execution docs: `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`
- shipped verdict: `CLOSEOUT_AUDIT_*.md`, `FINAL_STATUS.md`, or explicit status note

If a closeout note conflicts with an older TODO, the closeout note wins.

### 4) Record progress as bounded state, not chat history

After each meaningful slice:

- update `WORKSTREAM.json` when lane status, authoritative docs, or the primary repro/gate surface changes
- update `TODO.md` and `MILESTONES.md`
- update `EVIDENCE_AND_GATES.md` when the canonical command set changes
- add a dated note only for real decisions, audits, or closeout-worthy turns

Do not dump whole session transcripts into the lane.

### 5) Tie each slice to repro, gate, and evidence

Every non-trivial slice should name:

- one repro target
- one gate command
- one evidence anchor set

If the lane changes a hard contract, update ADR/alignment docs too.

### 6) Close out explicitly

When the lane is no longer an active execution surface:

- add `CLOSEOUT_AUDIT_YYYY-MM-DD.md` or `FINAL_STATUS.md`
- update the lane status note or positioning doc
- make it clear whether the lane is:
  - active,
  - maintenance,
  - closed,
  - or historical

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro, Gate, Evidence. See `fret-skills-playbook`.
- Lane state is explicit.
- `WORKSTREAM.json` reflects the current lane status and first-open doc set when the lane uses a dedicated directory.
- The next maintainer can tell whether to continue the lane or open a follow-on.
- A closed lane stays closed unless fresh evidence explicitly justifies a new follow-on.

## Evidence anchors

- Repo-wide execution stance: `docs/roadmap.md`
- Workstream catalog + status-note rules: `docs/workstreams/README.md`
- Machine-readable lane state contract: `docs/workstreams/standalone/workstream-state-v1.md`
- Review findings / cross-lane backlog: `docs/todo-tracker.md`
- Foundation-first loop: `docs/foundation-first-workflow.md`
- Diagnostics platform workstream example: `docs/workstreams/diag-fearless-refactor-v2/README.md`
- Closeout lane example: `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
- Explicit gates example: `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- ADR alignment matrix: `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

## Examples

- Example: start a new lane
  - User says: "We need a new workstream for this refactor."
  - Actions: decide ownership, create the minimal doc set, name one repro/gate/evidence path, and avoid adding speculative companion docs too early.
  - Result: a small lane that is easy for humans and agents to continue.

- Example: split a follow-on
  - User says: "Reopen this closed fearless-refactor lane and add more API cleanup."
  - Actions: read the closeout note first; if the old lane is closed, create a narrower follow-on instead of widening the historical lane.
  - Result: scope stays reviewable.

- Example: continue an active lane
  - User says: "Continue `ui-focus-overlay-fearless-refactor-v1`."
  - Actions: resolve the current state, pick one smallest slice from `TODO.md` / `MILESTONES.md`, run the named gates, and update the lane docs.
  - Result: one landable step with explicit evidence.

## Troubleshooting

- Symptom: the lane folder has too many notes.
  - Fix: resolve repo stance first, then lane positioning, then execution docs; ignore deep audits until the current state is clear.
- Symptom: `TODO.md` and a closeout note disagree.
  - Fix: prefer the latest closeout/status note and the roadmap stance over an older TODO checklist.
- Symptom: a new request almost fits an old lane.
  - Fix: ask whether continuing the old folder would blur ownership; if yes, create a narrow follow-on.

## Common pitfalls

- Treating every markdown file in a workstream folder as equally authoritative.
- Reopening a closed lane because an old checklist still exists.
- Recording whole conversations instead of bounded decisions/evidence.
- Adding many companion docs before the lane has a real first slice.

## Related skills

- `fret-repo-orientation`
- `fret-framework-maintainer-guide`
- `fret-diag-workflow`
- `fret-skills-playbook`
- `fret-boundary-checks`
