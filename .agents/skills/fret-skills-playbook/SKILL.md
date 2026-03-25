---
name: fret-skills-playbook
description: "This skill should be used when the user asks to \"write or update a skill\", \"define regression gates\", \"add a diag script\", or \"standardize `test_id` conventions\". Provides shared conventions for layering decisions (mechanism vs policy), regression gate types, diag scripts, and evidence discipline."
---

# Fret skills playbook (shared conventions)

This skill is the shared “glue” for the rest of the Fret skill set: it defines conventions that keep
agent work **reviewable**, **reproducible**, and **architecture-aligned**.

## When to use

- You are writing or updating a skill.
- You want consistent outputs across different contributors/agents.
- You are unsure what regression gate + evidence artifacts to leave behind.

## Inputs to collect (ask the user)

- What is the user-facing invariant (correctness, UX, parity, perf)?
- What is the smallest runnable target (demo/gallery/script) that shows it?
- Is the work about a first-party teaching surface (for example UI Gallery) or an internal recipe/runtime surface?
- Which layer should own the change (mechanism vs policy vs recipe)?
- What artifacts must be produced (gate + evidence)?

Defaults if unclear:

- Start from the smallest runnable demo and leave a regression artifact + evidence anchors.
- When first-party examples are involved, treat UI Gallery snippet files as the exemplar source of truth.

## Smallest starting point (one command)

- `python3 .agents/skills/fret_skills.py validate --strict`

## Quick start

- Use the “deliverables 3-pack” for any non-trivial change:
  - Repro (smallest target or script)
  - Gate (test/script/perf)
  - Evidence (anchors + command)

## Workflow

### 1) Layering decision (non-negotiable)

Use this rule of thumb:

- `crates/*`: mechanisms and hard-to-change contracts (routing, focus primitives, overlay roots, layout/semantics).
- `ecosystem/*`: policy + composition + recipes (dismiss/focus restore rules, roving/typeahead, shadcn recipes).
- `apps/fret-ui-gallery`: first-party exemplar/teaching surface; if the drift is “how should users author this in Fret?”, fix the snippet/page surface first.

If the change is “interaction policy” (dismiss rules, focus restore, keyboard nav rules), it almost never belongs in
`crates/fret-ui`.

### 2) The deliverables 3-pack (Repro + Gate + Evidence)

Every non-trivial change should leave these three deliverables:

- **Repro**: a smallest runnable target (demo/gallery page) or a `tools/diag-scripts/*.json` script.
- **Gate**: at least one regression gate:
  - unit/integration test for deterministic logic, and/or
  - `fretboard diag` script for event sequences/state machines, and/or
  - perf gate/baseline when perf is the goal.
- **Evidence**: 1–3 evidence anchors (file paths + key functions/tests/scripts) so reviewers can verify quickly.
  - Prefer the smallest deterministic proof:
    - geometry assertions or `capture_layout_sidecar` for layout ownership/size negotiation
    - `capture_screenshot` for visible chrome/clipping/focus rings
    - `capture_bundle` for interaction state machines and shareable run context

### 3) `test_id` conventions (automation stability)

Goal: scripts should select **intent-level** targets, not pixel coordinates.

- Put `test_id` at the highest stable semantic surface that survives refactors:
  - reusable component crates → recipe/component layer
  - first-party UI Gallery surfaces → snippet/page/driver seam used by diagnostics
- Use stable, namespaced ids (examples):
  - `ui-gallery-command-palette-trigger`
  - `ui-gallery-select-trigger`
  - `ui-gallery-docking-tab-bar-drag-anchor`
- Avoid using list indices as ids; use model ids or stable row/item keys.

### 4) Diag script conventions (reviewable and gate-friendly)

- Prefer schema v2 for new scripts.
- Prefer selectors by `test_id`.
- Name scripts so they can be used as a gate label:
  - `ui-gallery-<surface>-<behavior>-<expectation>.json`
  - `docking-<scenario>-<expectation>.json`
- Keep scripts minimal: one scenario, one or two assertions, at least one `capture_bundle`.
- When proving layout ownership or size negotiation, add `capture_layout_sidecar` before falling back to screenshots.
- For first-party component pages, prefer the canonical nested corpus under `tools/diag-scripts/ui-gallery/<family>/`.

### 5) Evidence discipline (make it reversible)

When you fix a tricky issue, record:

- exact command(s) used,
- output dir / bundle path(s),
- the smallest script/test added,
- the conclusion (“what changed” + “why it’s correct”).

### 6) Skill testing discipline (triggering + behavior)

When creating or refreshing a skill, run a small manual test set before calling it done:

- 2-3 positive trigger prompts:
  - an obvious phrasing,
  - a paraphrased phrasing,
  - a repo-specific phrasing when the skill is Fret-only
- 1-2 negative prompts that should **not** load the skill
- 1 functional path:
  - the skill's main workflow completes with the expected repo command / artifact / code change
- 1 failure path when applicable:
  - missing tool, missing repo-ref mirror, or unsupported platform leads to a bounded fallback instead of hand-wavy output

### 7) External app repos (framework users)

If you are using these skills outside the Fret mono-repo:

- Expect that repo-local commands (`fretboard`, `tools/*`, `tools/diag-scripts/*`) must be run from a Fret checkout.
- Keep a sibling clone or submodule so evidence anchors remain clickable.

See: `fret-external-app-mode`.

## Definition of done (what to leave behind)

Minimum deliverables (3-pack):

- Repro: smallest runnable target or diag script.
- Gate: test/script/perf gate that fails before and passes after.
- Evidence: 1–3 anchors (paths/functions/tests) and a copy/pasteable command.

## Evidence anchors

- Layering and contracts: `docs/architecture.md`, `docs/runtime-contract-matrix.md`
- Crate/layer usage map: `docs/crate-usage-guide.md`
- Canonical shadcn migration status: `docs/shadcn-declarative-progress.md`
- Upstream local-mirror policy: `docs/repo-ref.md`
- Diag scripts and workflows: `tools/diag-scripts/`, `.agents/skills/fret-diag-workflow/SKILL.md`
- UI Gallery exemplar + evidence note: `.agents/skills/fret-shadcn-source-alignment/references/ui-gallery-exemplar-and-evidence.md`
- UI Gallery authoring gates: `apps/fret-ui-gallery/src/lib.rs`
- UI Gallery geometry/test-id helpers: `apps/fret-ui-gallery/src/driver/render_flow.rs`
- Perf gates and baselines: `tools/perf/`, `docs/workstreams/perf-baselines/` (see `.agents/skills/fret-diag-workflow/SKILL.md`)

## Examples

- Example: add a new skill with a durable outcome
  - User says: "Turn this workflow into a repeatable skill."
  - Actions: define triggers, keep SKILL.md lean, move deep material into references, and leave the 3-pack (Repro + Gate + Evidence).
  - Result: a skill that is easy to load and hard to regress.

## Common pitfalls

- Fixing policy mismatches by adding runtime knobs in `crates/fret-ui`.
- Leaving no gate behind (“works on my machine” regressions).
- Unstable selectors (`test_id` missing/duplicated), leading to flaky scripts.
- Writing long narratives instead of a small reproducible repro + gate + evidence anchors.

## Troubleshooting

- Symptom: `validate --strict` fails.
  - Fix: ensure frontmatter has `---` delimiters, `name` matches the folder, and required headings exist.
- Symptom: anchor checks fail in the mono-repo.
  - Fix: replace directory-only anchors with stable file paths.
- Symptom: a shared/meta skill points to `repo-ref/` paths and becomes noisy in external repos.
  - Fix: keep `repo-ref/` anchors for source-alignment skills only; when they are required, pair them with `docs/repo-ref.md` and state that the mirrors are optional local state.

## Related skills

- `fret-repo-orientation`
- `fret-diag-workflow`
- `fret-shadcn-source-alignment`
