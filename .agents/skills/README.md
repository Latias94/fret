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

- Generic/shared skills should prefer stable in-repo docs and code anchors.
- Source-alignment skills may prefer local `repo-ref/` mirrors when available, and fall back to public upstream docs/source URLs only when the mirror is absent or insufficient.
- If you are using these skills in an external app repo (not inside the Fret mono-repo), consider keeping a lightweight Fret source checkout available so “evidence anchors” (paths) remain clickable.
  - Option A (recommended): add the Fret repo as a git submodule or keep a sibling checkout for browsing.
  - Option B (fallback): browse dependency sources in the Cargo registry (`~/.cargo/registry/src/...`) for published crates.
  - See `docs/repo-ref.md` for the local-mirror policy and why `repo-ref/` is optional local state.

## Validate skills

Primary (fast, cross-platform, no dependencies):

```bash
python3 .agents/skills/fret_skills.py validate --strict
```

Optional (upstream schema validation):

If you want to validate against the upstream Agent Skills reference schema as well, search GitHub for
`agentskills/agentskills` and run their validator against a single skill folder.

## Skill conventions (Fret)

These conventions keep skills consistent and easy to auto-validate/package:

- Skill directory name: `fret-*`
- Skill entrypoint: `SKILL.md`
- Frontmatter: `name` and `description`
- Skill folder structure: `SKILL.md` (required), optional `references/`, `scripts/`, `assets/`
- Recommended headings: `When to use`, `Quick start`, `Workflow`, `Examples`, `Troubleshooting`, `Evidence anchors`, `Common pitfalls`, `Related skills`
- Evidence anchors: prefer stable file paths + symbol names; avoid fragile line-number anchors

Claude-aligned folder rules (mirrors the upstream “build a skill” guidance):

- `SKILL.md` must be named exactly `SKILL.md` (case-sensitive when uploaded/zipped)
- Do not place a `README.md` inside an individual skill folder (keep docs in `SKILL.md` or `references/`)
- Frontmatter must start at the top of `SKILL.md` and be wrapped in `---` delimiters
- `name` must match the folder name and be lowercase-hyphen (kebab-case)
- `description` must include **what** the skill does + **when** to use it (trigger phrases); keep it `<= 1024` chars
- Treat frontmatter as “system-prompt surface”: avoid XML/angle brackets and other injection-shaped content

## Skill quality bar (engineering discipline)

These rules are adapted from mature skill ecosystems (e.g. Claude Code plugin “skill-development” guidance),
but aligned to how Fret skills are validated and shipped.

### 1) Strong triggers in frontmatter

- Make `description` *specific* about when it should trigger (include concrete user phrases like
  “create a diag script”, “audit UI focus”, “align shadcn component”).
- Prefer third-person trigger wording:
  - ✅ “This skill should be used when…”
  - ❌ Vague one-liners.

### 2) Lean `SKILL.md` + progressive disclosure

- Keep procedural essentials in `SKILL.md`.
- Move long material (schemas, large checklists, deep background) into `references/` and link it.
- If a reference is large, include a suggested `rg` search pattern so agents can load only the relevant slice.

### 3) No duplicated knowledge

- Pick one “owner skill” per concept; other skills should link to it.
- Avoid copy/pasting the same checklist into multiple skills.

### 4) Write for an agent (not a blog post)

- Use imperative / verb-first instructions.
- Avoid second-person narration (“you should…”).
- Prefer checklists + short workflows over long prose.

### 5) Evidence + regression protection

- For non-trivial work, include evidence anchors (stable paths + key symbols).
- Prefer leaving the “3-pack”: Repro + Gate + Evidence (see `fret-skills-playbook`).

### 6) Validate locally

- Fast: `python3 .agents/skills/fret_skills.py validate --strict`
- Maintainer mode (mono-repo): `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

Maintainer mode (recommended in the mono-repo; validates anchor paths and a small set of high-signal symbols):

```bash
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
```

### 7) Test triggering and task fit

- Before calling a skill update “done”, test:
  - one obvious trigger prompt,
  - one paraphrased trigger prompt,
  - one negative prompt that should not trigger.
- For workflow-heavy skills, also test one happy path and one bounded failure/fallback path.
- If a skill depends on optional local mirrors (`repo-ref/`) or external tools, state the fallback explicitly in `SKILL.md`.

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
- Start, continue, or close out a workstream lane with assumptions-first resume: `fret-workstream-lifecycle`
- Audit framework developer experience through a real consumer journey: `fret-framework-consumer-audit`
- Build a good-looking app UI (golden path): `fret-app-ui-builder`
- Review/audit a Fret UI: `fret-ui-review`
- Debug a correctness regression (repro + gate + bundle): `fret-diag-workflow`
- Measure or gate performance (numbers/baselines) + attribute worst-frame hitches: `fret-diag-workflow`
- Optimize performance + make it a contract (tail vs typical, suite normalization, reversible fixes): `fret-perf-optimization` (then `fret-diag-workflow`)
- Bridge perf bundles to Tracy timelines (deep attribution + instrumentation discipline): `fret-perf-tracy-bridge` (then `fret-diag-workflow`)
- Maintain/author framework components (parity work + gates): `fret-shadcn-source-alignment` (framework/eco authors)
- Maintain the framework safely (contracts + gates): `fret-framework-maintainer-guide`
- Debug on real mobile devices (Android + iOS evidence): `fret-mobile-real-device-debug`
- Refactor safely across crates/layers: `fret-boundary-checks`
- Ship releases: `fret-release-check-and-publish`

Common adjacent pulls:

- When building: start from `fret-app-ui-builder/references/` (recipes + mind models)
- When auditing: use `fret-ui-review` output format (terse findings)
- When auditing framework DX through a real user journey: start with `fret-framework-consumer-audit`, then pull `fret-app-ui-builder`, `fret-diag-workflow`, or `fret-external-app-mode` only as needed

## Skills

- `fret-repo-orientation`: Find the right layer/crate fast (mono-repo vs external app repo), choose the smallest runnable target, and keep contract-first navigation.
- `fret-workstream-lifecycle`: Manage the lifecycle of a workstream lane: create the minimal doc set, reopen existing lanes with an assumptions-first evidence pass, keep status explicit, decide continue vs follow-on, and close out with gates/evidence.
- `fret-skills-playbook`: Shared conventions for execution-mode selection, goal-backward verification, layering decisions, regression gates, `test_id`/diag script style, and evidence discipline across all skills.
- `fret-external-app-mode`: Use the skills from an external app repo (outside the mono-repo): what works without `tools/` and `fretboard`, and how to keep anchors and tooling usable via a Fret checkout.
- `fret-framework-consumer-audit`: Audit Fret from a framework-consumer/developer-experience perspective by running a real user journey, classifying friction by owner layer, and leaving proof artifacts instead of vague complaints.
- `fret-app-ui-builder`: Product-oriented golden path: pick a baseline style, apply token overrides, compose shadcn recipes, and leave diag/perf gates early.
- `fret-ui-review`: Review/audit Fret UI code for framework-aligned UX correctness (tokens, focus-visible, overlays, commands gating, `test_id`, and regression gates).
- `fret-framework-maintainer-guide`: Maintainer playbook for contracts/ADRs, boundaries, goal-backward verification, diagnostics/perf gates, upstream alignment (shadcn/Radix/Base UI), and evidence discipline.
- `fret-mobile-real-device-debug`: Real-device mobile debugging workflow (Android + iOS): run the smallest mobile target, verify Vulkan/Metal constraints, and capture diagnostics bundle evidence for ADRs/workstreams.
- `fret-diag-workflow`: Diagnostics for correctness + perf: scripted repros, bundles/screenshots, triage/compare, perf gates (`diag perf`), and worst-frame attribution.
- `fret-perf-optimization`: Perf optimization workflow: turn “jank” into a durable perf contract (tail vs typical), normalize suites, attribute worst bundles, and land reversible fixes with evidence.
- `fret-perf-tracy-bridge`: Bridge perf gates (diag perf + bundles) with Tracy timeline profiling: reproduce worst bundles, capture traces, and correlate spans with bundle stats.
- `fret-shadcn-source-alignment`: Align Fret components with upstream shadcn/ui v4 + Radix docs + source and add targeted tests/scripts to prevent regressions even when web goldens are incomplete.
- `fret-material-source-alignment`: Align Fret components with upstream Material 3 (Expressive) references (spec + MUI + Compose Material3 + Base UI) and lock outcomes with targeted tests and `fretboard diag` scripts.
- `fret-crate-audits`: Crate-by-crate code-quality audits for fearless refactors (purpose/exports/deps/hazards) and a small gate set.
- `fret-boundary-checks`: Guardrails for crate boundary/portability refactors (layering, module-size drift, crate audit snapshot).
- `fret-fixture-driven-harnesses`: Convert large test matrices into JSON fixtures + thin harnesses for reviewability and lower merge-conflict risk.
- `fret-release-check-and-publish`: Release workflow for Fret with `release-plz` + crates.io (scope selection, dry-run checks, CI publish flow, and common failure diagnostics).
- `fret-skill-evolution`: Capture reusable learnings as skills (standard headings, references/, plus tests/scripts/gates).
