---
name: fret-external-app-mode
description: "Use Fret skills from an external app repo (outside the Fret mono-repo): what works without `tools/` and `fretboard`, how to keep anchors clickable, and how to run mono-repo tooling when needed."
---

# Fret external app mode (framework users)

This skill is for **framework users** building an app *outside* the Fret mono-repo. Many other skills
reference repo-local tooling (`tools/`, `apps/fretboard`) and in-tree demos; those do not exist in a
consumer app repo by default.

Goal: keep the skills useful by making “what to run where” explicit.

## When to use

- You are using Fret from an external app repo (published crates or a git dependency).
- A skill tells you to run `fretboard` / `tools/*` / `tools/diag-scripts/*`, but your repo does not have them.
- You want evidence anchors (paths) to stay clickable and reviewable.

## Inputs to collect (ask the user)

- Are you willing to keep a lightweight Fret mono-repo checkout (sibling clone or submodule)?
- Do you need runnable tooling (diag/perf/fretboard) or only docs/source browsing?
- Is the app target native only, or native + wasm?

Defaults if unclear:

- Keep a sibling Fret repo checkout for tooling + anchors, and keep your app repo clean.

## Smallest starting point (one command)

- Install the skills bundle into your app repo:
  - `python3 /path/to/fret/.agents/skills/fret_skills.py install --agent codex --target /path/to/my-app --force`

## Quick start

Recommended setup (sibling checkouts):

- `~/work/fret/` (Fret mono-repo; tooling + anchors)
- `~/work/my-app/` (your app repo)

Then:

1. Install Fret skills into your app repo (so your agent can load them).
2. Keep the Fret repo checkout available for:
   - clickable evidence anchors (`docs/`, `crates/`, `ecosystem/`),
   - running repo-local tools (`fretboard`, `tools/perf/*`, `tools/diag-scripts/*`) when needed.

## Workflow

### 1) Decide what you need (docs-only vs tooling)

- Docs/source browsing only:
  - You can rely on published docs + Cargo registry sources.
  - Evidence anchors in skills won’t be clickable unless you also have a Fret checkout.
- Tooling (diag/perf/fretboard/scripts):
  - Keep a Fret checkout (sibling clone or submodule). The tools live in that repo.

### 2) Keep anchors clickable

Preferred:

- Keep a Fret checkout and point contributors/agents at it for evidence anchors.

Fallback:

- Browse dependency sources in the Cargo registry (`~/.cargo/registry/src/...`).
  - Note: you will not have `apps/` and `tools/` in registry sources.

### 3) Running repo-local tooling from the Fret checkout

If a skill says to run `fretboard` or `tools/*`, run it **in the Fret repo**, not your app repo.

Typical pattern:

- Repro a behavior in your app repo (native/web).
- If you need Fret’s diag/perf tooling, reproduce the minimal scenario in a Fret demo/gallery or add a small demo in a temporary branch.
- Leave the gate/evidence in Fret (script/test) *and* leave the app-side change in your app repo.

### 4) Keep deliverables reviewable (3-pack)

Even in external app repos, keep the “deliverables 3-pack”:

- Repro (smallest app surface or minimal Fret demo),
- Gate (test/script/perf where applicable),
- Evidence (anchors + command).

See: `fret-skills-playbook`.

## Definition of done (what to leave behind)

- The team has a clear “where to run commands” rule (app repo vs Fret repo checkout).
- Evidence anchors are reviewable (either via a Fret checkout or explicit registry paths).
- Any behavior change has a regression artifact (test/script) in the appropriate repo.

## Evidence anchors

- This repo’s skills entrypoint: `.agents/skills/fret_skills.py`
- Shared conventions: `.agents/skills/fret-skills-playbook/SKILL.md`
- Orientation: `.agents/skills/fret-repo-orientation/SKILL.md`

## Common pitfalls

- Trying to run `tools/*` commands inside the external app repo (they don’t exist there).
- Assuming `fretboard` is available without a Fret checkout.
- Making app-side changes without leaving any gate/evidence behind.

## Related skills

- `fret-repo-orientation`
- `fret-skills-playbook`
- `fret-diag-workflow`
- `fret-perf-workflow`
