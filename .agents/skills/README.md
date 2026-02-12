# Agent skills (repo-local)

This directory contains agent skill folders (`SKILL.md`, optional `references/`) maintained in the Fret repo so the team can share consistent debugging + parity workflows.

Most agents load skills from a *project-local* or *user-local* skills directory. Install by copying the skill folders from this repo into your agent's expected location.

## Install (examples)

Project-local (recommended):

- Claude Code: copy skill folders into `<project>/.claude/skills/`
- Codex CLI: copy skill folders into `<project>/.agents/skills/`, or use the global location below

Install script (recommended for consistency, cross-platform via Python):

- List available skills:
  - `python3 .agents/skills/fret_skills.py list`
- Install all skills into a target project:
  - Claude Code: `python3 .agents/skills/fret_skills.py install --agent claude-code --target <project> --force`
  - Codex CLI: `python3 .agents/skills/fret_skills.py install --agent codex --target <project> --force`
  - Gemini CLI: `python3 .agents/skills/fret_skills.py install --agent gemini --target <project> --force`
- Install a profile (recommended for most users):
  - App developers: `python3 .agents/skills/fret_skills.py install --agent codex --target <project> --profile app-dev --force`
  - Framework developers: `python3 .agents/skills/fret_skills.py install --agent codex --target <project> --profile framework-dev --force`
- Install a subset:
  - `python3 .agents/skills/fret_skills.py install --agent codex --target <project> --skills fret-diag-workflow,fret-shadcn-app-recipes --force`

Global (user-local):

- Codex CLI: copy skill folders into `%USERPROFILE%\.agents\skills\`
- Claude Code: copy skill folders into `%USERPROFILE%\.claude\skills\`

PowerShell example (copy all Fret skills into a project for Claude Code):

```powershell
New-Item -ItemType Directory -Force .\.claude\skills | Out-Null
Copy-Item -Recurse -Force .\.agents\skills\fret-* .\.claude\skills\
```

## Notes on upstream references

- Skills point to **public upstream docs and source URLs** by default.
- Some developer checkouts may include optional pinned snapshots under `repo-ref/` for quick local diffs; this folder is not necessarily present on GitHub checkouts of this repo.
- If you are using these skills in an external app repo (not inside the Fret mono-repo), consider keeping a lightweight Fret source checkout available so “evidence anchors” (paths) remain clickable.
  - Option A (recommended): add the Fret repo as a git submodule or keep a sibling checkout for browsing.
  - Option B (fallback): browse dependency sources in the Cargo registry (`~/.cargo/registry/src/...`) for published crates.

## Validate skills

Primary (fast, cross-platform, no dependencies):

```bash
python3 .agents/skills/fret_skills.py validate --strict
```

Optional (upstream Agent Skills reference validator):

This repo vendors the Agent Skills reference implementation under `repo-ref/agentskills/skills-ref`.

Example (macOS/Linux, using `uv`):

```bash
cd repo-ref/agentskills/skills-ref
uv sync
source .venv/bin/activate
skills-ref validate ../../../.agents/skills/fret-diag-workflow
skills-ref validate ../../../.agents/skills/fret-perf-workflow
```

Maintainer mode (recommended in the mono-repo; validates anchor paths and a small set of high-signal symbols):

```bash
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
```

## Public distribution (recommended approach)

If you want a lightweight “skills-only” artifact (for framework users who do not want to clone the full repo),
export a bundle zip and attach it to a GitHub Release:

CI helper (recommended): `.github/workflows/skills-bundles.yml` builds and uploads `app-dev` + `framework-dev`
bundles on every published GitHub Release (and supports manual runs via `workflow_dispatch`).

```bash
python3 .agents/skills/fret_skills.py package --profile app-dev --out dist/fret-skills-app-dev
python3 .agents/skills/fret_skills.py package --profile framework-dev --out dist/fret-skills-framework-dev
```

Each bundle contains only skill folders and can be installed by unzipping and copying into:

- Codex CLI: `<project>/.agents/skills/`
- Claude Code: `<project>/.claude/skills/`

## Skill map (what to use when)

- Get oriented / find the right layer: `fret-repo-orientation`
- Shared conventions (layering, gates, evidence): `fret-skills-playbook`
- Using skills from an external app repo (framework users): `fret-external-app-mode`
- Build a cohesive app UI fast (product-oriented): `fret-app-ui-builder`
- Review/audit Fret UI code (best practices): `fret-ui-review`
- Make the UI look good: `fret-ui-ux-guidelines` + `fret-design-system-styles` + `fret-shadcn-app-recipes`
- App architecture + side effects (persistence, background work): `fret-app-architecture-and-effects`
- State stack defaults (typed routing + selector + query): `fret-app-architecture-and-effects` + `fret-component-authoring`
- Align behavior with shadcn/Radix: `fret-shadcn-source-alignment` (then add invariant tests + `fretboard diag` repros)
- Debug UI regressions: `fret-diag-workflow` (capture bundle/screenshot, script repro, turn into a gate)
- Refactor audits + guardrails: `fret-crate-audits` + `fret-boundary-checks` + `fret-fixture-driven-harnesses`
- Profile + gate performance (numbers/baselines): `fret-perf-workflow` (resize/scroll/pointer-move probes, baseline selection, perf log evidence)
- Attribute perf hitches (root cause playbook): `fret-perf-attribution` (read bundles, decide CPU vs GPU, pick the next profiler, record evidence)
- Turn one-off fixes into reusable workflows: `fret-skill-evolution` (update skills + add tests/scripts/gates)
- Build complex editor shells: `fret-docking-and-viewports` + `fret-commands-and-keymap` + overlay/layout skills as needed
- Prepare and run releases: `fret-release-check-and-publish` (`release-plz` scope, preflight checks, release-pr/release troubleshooting)

## Skills

- `fret-repo-orientation`: Find the right layer/crate fast (mono-repo vs external app repo), choose the smallest runnable target, and keep contract-first navigation.
- `fret-skills-playbook`: Shared conventions for layering decisions, regression gates, `test_id`/diag script style, and evidence discipline across all skills.
- `fret-external-app-mode`: Use the skills from an external app repo (outside the mono-repo): what works without `tools/` and `fretboard`, and how to keep anchors and tooling usable via a Fret checkout.
- `fret-app-ui-builder`: Product-oriented golden path: pick a baseline style, apply token overrides, compose shadcn recipes, and leave diag/perf gates early.
- `fret-ui-review`: Review/audit Fret UI code for framework-aligned UX correctness (tokens, focus-visible, overlays, commands gating, `test_id`, and regression gates).
- `fret-component-authoring`: Declarative component authoring in `fret-ui` + `fret-ui-kit` (identity, element-local state, model observation, `ui()` builder surface).
- `fret-action-hooks`: Component-owned interaction policy (press/dismiss/roving/typeahead/timers) via runtime action hooks (ADR 0074).
- `fret-app-architecture-and-effects`: App-level structure (Models + Commands + Effects), typed routing (`MessageRouter`/`KeyedMessageRouter`), and async state (`fret-query`/`fret-selector`) on top of runner-owned concurrency (Dispatcher + Inbox).
- `fret-design-system-styles`: Apply a cohesive visual style via shadcn presets + `ThemeConfig` token overrides (density/radius/shadows/rings).
- `fret-ui-ux-guidelines`: App-level UX + visual hierarchy playbook (spacing rhythm, editor shell patterns, polish) that composes with token/theming skills.
- `fret-commands-and-keymap`: Commands/menus/palette + `keymap.json` (focus-aware routing, `when` gating, platform bindings).
- `fret-text-input-and-ime`: Text input + IME composition contracts (caret geometry feedback, command-vs-text arbitration, a11y semantics).
- `fret-scroll-and-virtualization`: Scrolling + virtualized large lists (stable item keys, measurement modes, scroll-to-item).
- `fret-layout-and-style`: Token-driven layout + styling (`LayoutRefinement`, `ChromeRefinement`, `UiBuilder`) and common overflow/clipping patterns.
- `fret-overlays-and-focus`: Overlay orchestration + Radix-aligned dismiss/focus behavior (`OverlayController`, placement/anchoring).
- `fret-animation-and-scheduling`: Runner-owned scheduling, continuous frames leases, and transition/presence helpers (RAF/timers).
- `fret-docking-and-viewports`: Docking/multi-window/viewport concepts and conformance harness entry points.
- `fret-diag-workflow`: Use `fretboard diag` + `tools/diag-scripts/*.json` to reproduce UI issues, capture bundles/screenshots, triage regressions, and turn bugs into stable repro gates.
- `fret-perf-workflow`: Profile and gate performance with `fretboard diag perf` + baselines + `tools/perf/*` helpers, and record commit-addressable evidence in workstreams logs.
- `fret-perf-attribution`: Attribute and explain performance hitches (tail latency) using diag bundles + perf gates, then choose the right next profiler (CPU stacks, allocations, GPU capture) and integrate evidence back into workstreams logs.
- `fret-shadcn-source-alignment`: Align Fret components with upstream shadcn/ui v4 + Radix docs + source (optional local pinned snapshots under `repo-ref/`) and add targeted tests/scripts to prevent regressions even when web goldens are incomplete.
- `fret-shadcn-app-recipes`: Build good-looking apps with `fret-ui-shadcn` by translating shadcn/Tailwind mental models into Fret patterns, and pairing recipes with tests + `fretboard diag` scripts to avoid regressions.
- `fret-crate-audits`: Crate-by-crate code-quality audits for fearless refactors (purpose/exports/deps/hazards) and a small gate set.
- `fret-boundary-checks`: Guardrails for crate boundary/portability refactors (layering, module-size drift, crate audit snapshot).
- `fret-fixture-driven-harnesses`: Convert large test matrices into JSON fixtures + thin harnesses for reviewability and lower merge-conflict risk.
- `fret-release-check-and-publish`: Release workflow for Fret with `release-plz` + crates.io (scope selection, dry-run checks, CI publish flow, and common failure diagnostics).
- `fret-skill-evolution`: Capture reusable learnings as skills (standard headings, references/, plus tests/scripts/gates).
