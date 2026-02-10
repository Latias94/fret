# Agent skills (repo-local)

This directory contains agent skill folders (`SKILL.md`, optional `references/`) maintained in the Fret repo so the team can share consistent debugging + parity workflows.

Most agents load skills from a *project-local* or *user-local* skills directory. Install by copying the skill folders from this repo into your agent's expected location.

## Install (examples)

Project-local (recommended):

- Claude Code: copy skill folders into `<project>/.claude/skills/`
- Codex CLI: copy skill folders into `<project>/.agents/skills/`, or use the global location below

Install scripts (recommended for consistency):

- PowerShell (Windows): `powershell -ExecutionPolicy Bypass -File .\\.agents\\skills\\install.ps1 -Agent claude -Force`
- Bash (macOS/Linux): `./.agents/skills/install.sh --agent claude-code --force`

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

## Validate skills (agentskills spec)

This repo vendors the Agent Skills reference implementation under `repo-ref/agentskills/skills-ref`.

Example (macOS/Linux, using `uv`):

```bash
cd repo-ref/agentskills/skills-ref
uv sync
source .venv/bin/activate
skills-ref validate ../../../.agents/skills/fret-diag-workflow
skills-ref validate ../../../.agents/skills/fret-perf-workflow
```

## Skill map (what to use when)

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
