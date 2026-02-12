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
  - `python3 .agents/skills/fret_skills.py list --with-descriptions`
- Install all skills into a target project:
  - Claude Code: `python3 .agents/skills/fret_skills.py install --agent claude-code --target <project> --force`
  - Codex CLI: `python3 .agents/skills/fret_skills.py install --agent codex --target <project> --force`
  - Gemini CLI: `python3 .agents/skills/fret_skills.py install --agent gemini --target <project> --force`
- Install a profile (recommended for most users):
  - Framework users (external app repos): `python3 .agents/skills/fret_skills.py install --agent codex --target <project> --profile consumer-app-dev --force`
  - Framework developers: `python3 .agents/skills/fret_skills.py install --agent codex --target <project> --profile framework-dev --force`
- Install a subset:
  - `python3 .agents/skills/fret_skills.py install --agent codex --target <project> --skills fret-diag-workflow,fret-app-ui-builder --force`

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
```

Maintainer mode (recommended in the mono-repo; validates anchor paths and a small set of high-signal symbols):

```bash
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
```

## Public distribution (recommended approach)

If you want a lightweight “skills-only” artifact (for framework users who do not want to clone the full repo),
export a bundle zip and attach it to a GitHub Release:

CI helper (recommended): `.github/workflows/skills-bundles.yml` builds and uploads `consumer-app-dev` + `framework-dev`
bundles on every published GitHub Release (and supports manual runs via `workflow_dispatch`).

Auto-release (recommended): `.github/workflows/skills-auto-release.yml` creates a GitHub Release automatically whenever
`.agents/skills/metadata.json`'s `version` changes on `main`. The release tag format is `fret-skills-v<version>`.

```bash
python3 .agents/skills/fret_skills.py package --profile consumer-app-dev --out dist/fret-skills-consumer-app-dev
python3 .agents/skills/fret_skills.py package --profile framework-dev --out dist/fret-skills-framework-dev
```

Each bundle contains only skill folders and can be installed by unzipping and copying into:

- Codex CLI: `<project>/.agents/skills/`
- Claude Code: `<project>/.claude/skills/`

## Skill map (what to use when)

Pick **one primary skill** based on intent, then pull in the adjacent ones only if needed:

- Get oriented / pick the right layer: `fret-repo-orientation` (then `fret-skills-playbook`)
- Build a good-looking app UI (golden path): `fret-app-ui-builder`
- Review/audit a Fret UI: `fret-ui-review`
- Debug a correctness regression (repro + gate + bundle): `fret-diag-workflow`
- Measure or gate performance (numbers/baselines) + attribute worst-frame hitches: `fret-diag-workflow`
- Maintain/author framework components (parity work + gates): `fret-shadcn-source-alignment` (framework/eco authors)
- Maintain the framework safely (contracts + gates): `fret-framework-maintainer-guide`
- Refactor safely across crates/layers: `fret-boundary-checks`
- Ship releases: `fret-release-check-and-publish`

Common adjacent pulls:

- When building: start from `fret-app-ui-builder/references/` (recipes + mind models)
- When auditing: use `fret-ui-review` output format (terse findings)

## Skills

- `fret-repo-orientation`: Find the right layer/crate fast (mono-repo vs external app repo), choose the smallest runnable target, and keep contract-first navigation.
- `fret-skills-playbook`: Shared conventions for layering decisions, regression gates, `test_id`/diag script style, and evidence discipline across all skills.
- `fret-external-app-mode`: Use the skills from an external app repo (outside the mono-repo): what works without `tools/` and `fretboard`, and how to keep anchors and tooling usable via a Fret checkout.
- `fret-app-ui-builder`: Product-oriented golden path: pick a baseline style, apply token overrides, compose shadcn recipes, and leave diag/perf gates early.
- `fret-ui-review`: Review/audit Fret UI code for framework-aligned UX correctness (tokens, focus-visible, overlays, commands gating, `test_id`, and regression gates).
- `fret-framework-maintainer-guide`: Maintainer playbook for contracts/ADRs, boundaries, diagnostics/perf gates, upstream alignment (shadcn/Radix/Base UI), and evidence discipline.
- `fret-diag-workflow`: Diagnostics for correctness + perf: scripted repros, bundles/screenshots, triage/compare, perf gates (`diag perf`), and worst-frame attribution.
- `fret-shadcn-source-alignment`: Align Fret components with upstream shadcn/ui v4 + Radix docs + source (optional local pinned snapshots under `repo-ref/`) and add targeted tests/scripts to prevent regressions even when web goldens are incomplete.
- `fret-crate-audits`: Crate-by-crate code-quality audits for fearless refactors (purpose/exports/deps/hazards) and a small gate set.
- `fret-boundary-checks`: Guardrails for crate boundary/portability refactors (layering, module-size drift, crate audit snapshot).
- `fret-fixture-driven-harnesses`: Convert large test matrices into JSON fixtures + thin harnesses for reviewability and lower merge-conflict risk.
- `fret-release-check-and-publish`: Release workflow for Fret with `release-plz` + crates.io (scope selection, dry-run checks, CI publish flow, and common failure diagnostics).
- `fret-skill-evolution`: Capture reusable learnings as skills (standard headings, references/, plus tests/scripts/gates).
