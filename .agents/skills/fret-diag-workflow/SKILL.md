---
name: fret-diag-workflow
description: "Runs and triages Fret UI diagnostics via `fretboard diag` (scripts, bundles, shareable artifacts, compare/triage, perf gates). Use when user asks to reproduce a flaky UI bug, write a diag script, capture/share a diagnostics bundle, triage/compare bundles, or add a perf gate."
---

# Fret diagnostics workflow (correctness + perf)

## What this skill does

- Turns flaky UI bugs into deterministic repros (scripts + stable selectors).
- Produces portable, shareable artifacts (bundles, sidecars, AI packets, zips).
- Helps triage quickly without opening/grepping a huge `bundle.json`.
- Supports both correctness debugging and perf gating/attribution.

Use `fret-ui-review` when the goal is an architecture/UX audit rather than producing repro artifacts.

## When to use

- You want to reproduce a flaky UI bug with a deterministic script.
- You need to capture or share a bounded diagnostics artifact.
- You want to triage a bundle/run directory without opening raw `bundle.json`.
- You need a perf gate, worst-bundle attribution, or a conformance script.

## Choose this vs adjacent skills

- Use this skill when the main deliverable is a repro artifact, script, bundle, or triage result.
- Use `fret-framework-consumer-audit` when the main goal is to discover broad consumer-journey friction; `fret-diag-workflow` should then supply the proof artifact for the top issue rather than own the whole audit.
- Use `fret-shadcn-source-alignment` or `fret-material-source-alignment` when the main goal is upstream parity work.
- Use `fret-ui-review` when the main goal is an audit, not an artifact-producing workflow.

## Inputs to collect (ask the user)

- What is the smallest runnable target (demo/gallery/app) that shows the issue?
- Is the task about correctness repro, triage, or perf attribution?
- Do we need a new script, or are we running an existing promoted script/suite?
- What stable `test_id` selectors or semantics selectors already exist?
- What artifact form is needed: packed bundle, AI packet, screenshots, layout sidecars, or bounded sidecars only?
- Is this a layout-ownership/clipping issue where `capture_layout_sidecar` should be part of the first repro?

Defaults if unclear:

- Start with a smallest deterministic script, run it with `--launch`, and leave a bounded share artifact plus a gate.
- If the issue is on a first-party shadcn page, start from the existing UI Gallery script corpus and snippet/page `test_id` surfaces before inventing a new target.

## Workflow

## Quick start (native, recommended)

- Start here for run hygiene + bounded artifacts:
  - `references/launch-and-artifact-hygiene.md`
- Start here for bundle triage + maintainer notes:
  - `references/triage-and-maintainer-notes.md`

Recommended first command:

- `cargo run -p fretboard -- diag config doctor --mode launch --print-launch-policy`

## Workflow

### 1) Choose transport and launch strategy

- Native/filesystem-trigger is the day-to-day default.
- Web/WASM transport details live in `references/web-runner.md`.
- Prefer `--launch` and a unique `--dir` / session boundary for deterministic runs.

### 2) Author or select the smallest script

- Prefer schema v2 for new scripts.
- Use stable `test_id` selectors rather than pixel coordinates.
- Keep scripts minimal: one scenario, one or two assertions, at least one `capture_bundle`.
- For first-party component pages, prefer the existing UI Gallery corpus under `tools/diag-scripts/ui-gallery/<family>/`.
- If the failure is about width/flex/min-size ownership, clipping, or viewport negotiation, add `capture_layout_sidecar` near the interesting step.
- Capability-specific authoring guidance stays in `references/launch-and-artifact-hygiene.md`.

### 3) Run and share bounded artifacts

- Prefer packed, bounded outputs over raw `bundle.json`.
- Use AI packets / sidecars when sharing in chat or review loops.
- Pick the smallest artifact that explains the failure:
  - `capture_layout_sidecar` for layout-tree ownership
  - `capture_screenshot` for chrome/clipping/focus-visible evidence
  - `capture_bundle` for interaction state machines and shareable context
- Use `references/launch-and-artifact-hygiene.md` for session hygiene, artifact-size hygiene, and the recommended run/share flow.

### 4) Triage with bounded queries

- Use `meta`, `windows`, `dock-routing`, `query test-id`, and `slice` instead of grepping raw bundle files.
- For fast query/slice helpers, troubleshooting signatures, and maintainer notes, use `references/triage-and-maintainer-notes.md`.
- For evidence-first failure explanation, also read `references/evidence-triage.md`.

### 5) Escalate to conformance or perf when needed

- Component conformance playbooks:
  - `references/select-conformance.md`
  - `references/combobox-conformance.md`
  - `references/layout-sweep.md`
- Perf handoff and worst-bundle attribution:
  - `references/perf-handoff.md`

## Performance gates (when the issue is a hitch)

Use `diag perf` + worst-bundle evidence, then inspect the worst frames:

- `fretboard diag stats <bundle.json> --sort time --top 30`

See: `references/perf-handoff.md`.

## Definition of done (what to leave behind)

Ship a result that is reviewable and reusable:

- Minimum deliverables (3-pack): Repro (script), Gate (script/test), Evidence (bundle + anchors). See `fret-skills-playbook`.
- A minimal script under `tools/diag-scripts/` (schema v2 for new work) that reproduces the issue deterministically.
- Stable selectors (`test_id`) added/updated so the script survives refactors.
- One portable artifact path to share:
  - native: packed bundle dir (optional screenshots), or
  - web: `.fret/diag/exports/<timestamp>/bundle.json` via `fret-diag-export`.
- If you changed behavior: at least one regression gate (script and/or Rust test) linked from the PR/commit message.

## Evidence anchors

- This skill’s workflow: `references/launch-and-artifact-hygiene.md`, `references/triage-and-maintainer-notes.md`
- Evidence-first triage: `references/evidence-triage.md`
- Web/WASM workflow: `references/web-runner.md`
- Perf handoff: `references/perf-handoff.md`
- Public diagnostics CLI help surface: `crates/fretboard/src/cli/help.rs`
- Workspace-dev diagnostics CLI help surface: `apps/fretboard/src/cli/help.rs`
- UI Gallery script corpus: `tools/diag-scripts/ui-gallery/`
- UI Gallery geometry/test-id helpers: `apps/fret-ui-gallery/src/driver/render_flow.rs`
- Layout sidecar writer: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
- Conformance playbooks:
  - `references/select-conformance.md`
  - `references/combobox-conformance.md`
  - `references/layout-sweep.md`
- Tooling and artifacts: `tools/diag-scripts/`, `tools/rg-safe.ps1`

## Examples

- Example: capture a bounded repro artifact
  - User says: "This UI bug flakes—give me something I can share."
  - Actions: write or select a smallest script, run with `--launch`, pack a bounded artifact, then explain the failure with reason codes and sidecars.
  - Result: a reviewable repro + gate + evidence bundle.

- Example: triage without opening `bundle.json`
  - User says: "What happened in this run directory?"
  - Actions: use bounded meta/query/slice commands, inspect reason codes and traces, and summarize the likely root cause.
  - Result: a bounded triage result without output explosion.

## Common pitfalls

- Grepping or opening raw `bundle.json` by default.
- Reusing the same output directory across concurrent runs.
- Leaving no stable selectors behind, so scripts rot immediately.
- Jumping straight to screenshots when a layout sidecar would explain width/flex/clipping ownership faster.
- Treating CI or large artifacts as the first place to discover what happened.
- Mixing the public `fretboard diag ...` evidence owner with the workspace-dev
  `fretboard-dev diag ...` help surface; use `crates/fretboard/src/cli/help.rs` for the public CLI
  and `apps/fretboard/src/cli/help.rs` for the mono-repo developer wrapper.

## Troubleshooting

- Symptom: diagnostics output explodes.
  - Fix: switch to bounded queries and use the launch/artifact hygiene note.
- Symptom: the run directory exists but the failure is still unclear.
  - Fix: use the triage note plus evidence-first traces before adding more logging.

## Related skills

- `fret-skills-playbook`
- `fret-framework-consumer-audit`
- `fret-ui-review`
- `fret-shadcn-source-alignment`
- `fret-material-source-alignment`
