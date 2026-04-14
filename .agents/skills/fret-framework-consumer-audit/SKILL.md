---
name: fret-framework-consumer-audit
description: "This skill should be used when the user asks to \"find problems from a Fret user's perspective\", \"audit developer experience\", \"build real examples to expose framework friction\", or \"review onboarding/examples/scaffolds/API ergonomics\". Provides a task-driven workflow that simulates framework consumers shipping a small app slice, records friction by owner layer, and turns the top issues into gates, docs/example fixes, or refactor lanes."
---

# Fret framework consumer audit (developer-experience lens)

This skill treats Fret like a product, not just a codebase.

Start from a **real consumer task** and ask where a framework user would get stuck:

- onboarding and docs,
- templates and examples,
- public authoring surfaces,
- diagnostics and evidence tooling,
- layering and contract ownership.

Goal: turn “the framework feels awkward” into a **prioritized friction register** with owner layers,
smallest proofs, and follow-up lanes.

Because Fret is also positioning itself as a shadcn-aligned ecosystem layer, this audit should not
stop at code ergonomics. When the task touches first-party shadcn surfaces or recipe outcomes, the
audit should also check whether the rendered result is close to the expected visual/interaction
baseline via diagnostics evidence.

Also avoid abstract “framework feels good/bad” judgments. Anchor the audit in at least one real
application probe, ideally one that matches Fret's editor-grade positioning or the product category
the team cares about most.

## When to use

- You want to find problems from a Fret user or framework-consumer perspective.
- You want to build or port a small real app surface to expose authoring friction.
- You want to audit onboarding, examples, templates, or public crate surfaces.
- You want to verify whether Fret's rendered result is visually close to the expected shadcn-style outcome.
- You suspect the framework “works” but still teaches the wrong mental model or costs too much effort.
- You want to know whether real or complex app slices feel awkward to build, scale poorly, or fail to interoperate with expected Rust/app ecosystem pieces.
- You want concrete follow-up slices for API ergonomics, docs, example drift, or tooling gaps.

## Choose this vs adjacent skills

- Use this skill when the main goal is **finding framework friction through a real user journey**.
- Use `fret-app-ui-builder` when the main goal is shipping or refactoring a cohesive app surface.
- Use `fret-ui-review` when the main goal is auditing an existing UI surface or codebase.
- Use `fret-diag-workflow` when the main deliverable is a scripted repro, bundle, triage result, or perf artifact.
- Use `fret-shadcn-source-alignment` or `fret-material-source-alignment` when the issue is already known to be upstream parity drift.

## Inputs to collect (ask the user)

- Which consumer lane matters most:
  - first-hour onboarding,
  - scaffold-to-real-app,
  - comparison/porting,
  - external app repo?
- Which real application probe should anchor the audit?
  - editor notes workbench,
  - workspace shell / IDE-lite,
  - markdown / knowledge viewer,
  - data-heavy admin surface,
  - asset browser / preview surface,
  - node graph / canvas editor,
  - or another explicit real app target?
- What is the concrete task the consumer is trying to complete?
- Which public/product surface should the consumer follow first?
- Are we auditing first-party examples/docs, or a real external app repo?
- Do we need visual proof as part of the audit:
  - `capture_screenshot`,
  - `capture_layout_sidecar`,
  - `capture_bundle`,
  - or all three?
- Is the task intentionally a complex-app / ecosystem-fit probe:
  - async/background work,
  - persistence/config,
  - commands/keymaps,
  - docking/multi-panel shell,
  - third-party Rust crates or external app integration?
- What artifact should this audit leave behind: friction register only, or also example patches / diag scripts / docs fixes?

Defaults if unclear:

- Start with one small consumer task on the default ladder: `hello` -> `simple-todo` -> `todo`.
- Then escalate to at least one real application probe from `references/application-probes.md`; do not close the audit on `todo` alone.
- Prefer the public/product path first; treat maintainer-only shortcuts as friction evidence unless the audit explicitly targets maintainers.
- If the audit makes any visual/parity claim, leave diagnostics evidence instead of prose only:
  - `capture_screenshot` for visible chrome and style drift,
  - `capture_layout_sidecar` for size/stretch/clipping ownership,
  - `capture_bundle` for interaction state machines.

## Smallest starting point (one command)

- `cargo run -p fretboard -- new simple-todo --name audit-simple-todo`

If the audit must happen in an external app repo, start with `fret-external-app-mode` first.

## Quick start

1. Pick one consumer lane, one concrete task, and one real application probe.
2. Execute the task through the public/default surface first.
3. Escalate from warm-up demo to the chosen real application probe.
4. Log every friction point as a broken truth, not just a vague annoyance.
5. Capture bounded diag evidence for the top visual/interaction issues.
6. Classify each issue by owner layer and choose the right follow-up skill.
7. Leave the 3-pack: repro, gate, evidence.

Read these references first:

- `references/probe-lanes.md`
- `references/friction-register.md`
- `references/application-probes.md`
- `.agents/skills/fret-diag-workflow/SKILL.md`

## Workflow

### 1) Choose the right consumer lane before editing code

Use `references/probe-lanes.md` and pick exactly one primary lane:

- **Default onboarding lane**
  - first-time user,
  - public docs and examples,
  - smallest mental-model audit.
- **Scaffold-to-real-slice lane**
  - generate a small app,
  - add one real feature,
  - audit public authoring ergonomics.
- **Comparison / porting lane**
  - port one known UI/app slice,
  - compare authoring density and parity pressure,
  - only after the default lane is already understood.
- **External app lane**
  - use a real consumer repo,
  - audit packaging/tooling/docs/evidence friction,
  - pair with `fret-external-app-mode`.
- **Complex app / ecosystem-fit lane**
  - build or port a denser app slice,
  - stress commands, async, persistence, docking, data-heavy surfaces, or third-party crate integration,
  - audit whether the framework shape still feels composable.

Do not start from advanced or maintainer-heavy surfaces unless the consumer story actually needs them.

### 2) Write the consumer story as 1-3 outcome-shaped tasks

Phrase the task like a framework user would:

- “Generate a simple todo app and add filters.”
- “Build a settings form with dialogs and keyboard-first actions.”
- “Use Fret from an external app repo and keep diagnostics evidence reviewable.”

For each task, write down:

- starting doc/example/template,
- exact command or file the user would touch first,
- chosen real application probe,
- success truth,
- stop condition.

Keep scope narrow. The guide from `The Complete Guide to Building Skills for Claude` applies here too:
iterate on one high-signal task before broadening the audit.

Before moving on, sanity check the task against `references/application-probes.md`.
If the task is still basically “todo but with one more button”, the probe is probably too weak.

### 3) Run the journey using public/product surfaces first

Execute the task as if you were the consumer:

- follow `docs/first-hour.md`, `docs/examples/README.md`, and public `fretboard` commands first,
- prefer generated templates, cookbook examples, and app-facing `fret` surfaces before raw crate internals,
- treat every jump to maintainer-only commands, deep internals, or undocumented concepts as friction evidence.

Recommended pattern:

- use `hello` / `simple-todo` only to establish the baseline mental model,
- then move quickly to one real application probe that reflects the product claim under evaluation.

Record each time the task requires:

- an unexpected concept jump,
- undocumented imports or hidden extension traits,
- switching from public docs to repo spelunking,
- unclear example taxonomy,
- unstable or missing diagnostics surfaces,
- a runtime/policy distinction the user could not reasonably infer.
- ecosystem workarounds that feel framework-caused rather than app-caused.

### 3.5) Prove visual and interaction expectations with diagnostics

If the audit says “this surface matches expectations” or “the styling is off”, prove it.

Preferred evidence order:

- `capture_layout_sidecar` when the issue is stretch, clipping, width ownership, `w_full`, `flex_1`, or `min_w_0` drift
- `capture_screenshot` when the issue is visual chrome, spacing rhythm, focus rings, or shadcn-style parity
- `capture_bundle` when the issue is interaction sequencing, focus restore, dismissal, or command flow

Use `fret-diag-workflow` whenever:

- a first-party shadcn page should look close to the expected outcome,
- a regression claim depends on what is visibly rendered,
- a behavior claim needs a reviewable artifact instead of memory.

Do not accept “looks roughly right” without artifacts on parity-sensitive surfaces.

### 4) Classify friction by owner layer, not by complaint volume

Use `references/friction-register.md`.

For every issue, capture:

- **User symptom**: what blocked or confused the consumer.
- **Broken truth**: what should have been true if the journey were good.
- **Smallest proof**: command, file, example patch, or diag artifact.
- **Likely owner layer**: docs, template, example, ecosystem, runtime, tooling.
- **Next lane**: which skill or workstream should own the fix.

Recommended finding format:

- `P0/P1/P2 - lane - broken truth`
- `Evidence: command / path / artifact`
- `Owner: docs | template | example | ecosystem | crates | tooling`
- `Evidence type: screenshot | layout-sidecar | bundle | test | docs/example patch`
- `Next move: skill or slice`

### 5) Choose the correct follow-up lane

Do not collapse all friction into “fix runtime” or “write more docs”.

Route by owner:

- onboarding/docs drift -> docs/examples update
- missing app recipe or weak first-party exemplar -> `fret-app-ui-builder`
- unstable interaction or perf proof -> `fret-diag-workflow`
- existing UI surface smells -> `fret-ui-review`
- upstream behavior mismatch -> `fret-shadcn-source-alignment` or `fret-material-source-alignment`
- hard contract / cross-crate redesign -> `fret-framework-maintainer-guide` or `fret-workstream-lifecycle`
- external consumer-repo friction -> `fret-external-app-mode`
- complex-app composition or ecosystem-fit pain -> usually `fret-framework-consumer-audit` first, then maintainer lane or builder lane depending on whether the fix is contract-level or recipe-level

When the public API shape is wrong, prefer the correct redesign over a docs-only band-aid.

### 6) Leave durable artifacts, not just prose

Minimum deliverables (3-pack): Repro + Gate + Evidence.

Good audit outputs:

- a generated or edited smallest app/example that demonstrates the friction,
- one docs/example/template patch,
- one diag script or test for the highest-risk issue,
- one screenshot/layout-sidecar/bundle evidence set for the highest-risk visual or interaction claim,
- one prioritized friction register with owner layers,
- one recommended next slice or workstream note.

## Definition of done (what to leave behind)

- One concrete consumer journey has been executed end-to-end.
- The highest-signal issues are framed as broken truths with owner layers.
- Visual or interaction claims are backed by diagnostics artifacts rather than prose only.
- At least one issue leaves a durable artifact: docs/example patch, test, diag script, or screenshot/layout-sidecar/bundle evidence set.
- Follow-up work is routed to the correct skill/layer instead of lumped into “framework cleanup”.

## Evidence anchors

- Default example taxonomy: `docs/examples/README.md`
- First-hour onboarding: `docs/first-hour.md`
- Todo golden path: `docs/examples/todo-app-golden-path.md`
- Ergonomics/comparison note: `docs/ui-ergonomics-and-interop.md`
- Crate/layer usage map: `docs/crate-usage-guide.md`
- External consumer repo workflow: `.agents/skills/fret-external-app-mode/SKILL.md`
- UI building golden path: `.agents/skills/fret-app-ui-builder/SKILL.md`
- UI audit lens: `.agents/skills/fret-ui-review/SKILL.md`
- Diagnostics workflow: `.agents/skills/fret-diag-workflow/SKILL.md`
- Shared conventions: `.agents/skills/fret-skills-playbook/SKILL.md`
- Probe lanes: `references/probe-lanes.md`
- Friction categories: `references/friction-register.md`

## Examples

- Example: audit the first-hour experience
  - User says: “From a new Fret user’s perspective, where would they get stuck in the default path?”
  - Actions: follow `docs/first-hour.md`, generate `simple-todo`, note every unexpected concept jump, and route each issue to docs/template/API/tooling.
  - Result: a prioritized onboarding-friction register with at least one proof artifact.

- Example: use a real feature slice to expose API friction
  - User says: “Build a small settings surface and tell me where the framework feels heavier than it should.”
  - Actions: ship one thin surface, measure concept and authoring friction, then decide whether the fix belongs in examples, helpers, or contracts.
  - Result: a real consumer slice plus specific follow-up recommendations.

- Example: verify shadcn-style outcome with evidence
  - User says: “This should feel like the shadcn baseline. Does it actually look right?”
  - Actions: run the smallest first-party target, capture screenshot/layout sidecar/bundle evidence, then classify whether the issue is parity drift, token drift, or docs/example drift.
  - Result: a visual verdict backed by reviewable artifacts.

- Example: audit external app usage
  - User says: “Use Fret from another repo and show me the rough edges.”
  - Actions: pair with `fret-external-app-mode`, keep evidence anchors reviewable, and classify setup/tooling/docs friction separately from runtime bugs.
  - Result: an external-consumer audit that does not hide mono-repo assumptions.

- Example: stress a more realistic app slice
  - User says: “A simple demo is fine, but does a denser app with commands, async work, and settings still feel good?”
  - Actions: use the complex-app lane, build one denser slice, and record where composition, state ownership, async, or ecosystem integration starts to fight the user.
  - Result: framework-shape feedback that is much harder to dismiss as “beginner unfamiliarity”.

## Common pitfalls

- Starting from UI Gallery or maintainer-only demos for a first-contact audit.
- Treating repo-local shortcuts as acceptable public surfaces.
- Filing every annoyance as a runtime bug when the real issue is docs, examples, or template teaching.
- Comparing against upstream or `repo-ref/` too early before the baseline user task is proven.
- Leaving only prose and no smallest proof artifact.
- Calling visual parity “good enough” without screenshots or layout evidence.
- Declaring success after a `todo`-level slice even though the framework is meant to support much richer application shapes.
- Treating complex-app pain as out-of-scope when that pain may be the real API/design bug.
- Papering over a bad public API with more tutorial text instead of fixing the surface.

## Troubleshooting

- Symptom: the audit explodes into too many issues.
  - Fix: keep one consumer lane and one task; rank by broken truth + severity + owner clarity.
- Symptom: it is unclear whether the pain is real or just unfamiliarity.
  - Fix: stay on the public docs/template path longer; if success depends on internal repo knowledge, that is evidence.
- Symptom: the issue only reproduces in a real consumer repo.
  - Fix: switch to `fret-external-app-mode` and keep a sibling Fret checkout for tooling + anchors.
- Symptom: a simple todo-like audit looks fine, but real app slices still feel wrong.
  - Fix: switch to the complex-app / ecosystem-fit lane and probe one denser workflow before closing the audit.
- Symptom: the task still feels too generic.
  - Fix: choose a named real application probe from `references/application-probes.md` and restate the task in product language.

## Related skills

- `fret-skills-playbook`
- `fret-external-app-mode`
- `fret-app-ui-builder`
- `fret-ui-review`
- `fret-diag-workflow`
- `fret-framework-maintainer-guide`
