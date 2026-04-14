# Probe lanes for framework-consumer audits

Pick one primary lane before doing any audit work.

## 1) Default onboarding lane

Use when:

- the question is “what does a first-time Fret user experience?”
- you want to test the public/default authoring story
- you suspect docs, example taxonomy, or hidden mental-model costs

Start from:

- `docs/examples/README.md`
- `docs/first-hour.md`
- `docs/examples/todo-app-golden-path.md`

Recommended tasks:

- generate `hello` or `simple-todo`
- change one visible behavior
- add one small feature without leaving the default ladder

Look for:

- hidden imports or extension traits
- concept jumps not explained by the docs
- example taxonomy confusion
- public vs maintainer command drift

## 2) Scaffold-to-real-slice lane

Use when:

- the public ladder works, but you want to know where a real app gets awkward
- you want to test the public `fret` authoring surface, not internals

Start from:

- generated `simple-todo` or `todo`
- app recipes under `.agents/skills/fret-app-ui-builder/references/recipes/`

Recommended tasks:

- add filters, dialogs, settings, or command palette flows
- introduce one realistic layout split
- keep the task small enough to finish in one slice

Look for:

- unnecessary concept count
- awkward action/state wiring
- recipe gaps
- places where app authors must copy internal patterns

## 3) Comparison / porting lane

Use when:

- the issue is about ergonomics relative to a known design or framework
- you want to see where authoring density or parity pressure shows up

Start from:

- `docs/ui-ergonomics-and-interop.md`
- a first-party example or a pinned `repo-ref/` source only after the default path is understood

Recommended tasks:

- port one small page or feature
- keep the visual target narrow
- record where the extra work comes from: layout, state, policy, docs, or missing helpers

Look for:

- parity drift that should route to source-alignment skills
- comparison surfaces accidentally used as onboarding
- helper gaps that only exist because the teaching surface is weak

## 4) External app lane

Use when:

- the audit must happen in a real consumer repository
- you want to test install, packaging, repo-local tooling assumptions, or clickable evidence flow

Start from:

- `.agents/skills/fret-external-app-mode/SKILL.md`

Recommended tasks:

- install and run a small app
- reproduce one real issue
- keep a sibling Fret checkout for tools and anchors

Look for:

- mono-repo-only assumptions
- missing guidance for `fretboard` / diag / tools
- unclear split between app repo and Fret repo responsibilities

## 5) Complex app / ecosystem-fit lane

Use when:

- the default ladder is not enough to expose the real pain
- you want to know whether Fret still feels good under denser, more realistic app demands
- you suspect ecosystem support or composition shape is the real issue

Start from:

- a generated `todo`-level app or a narrow real app surface
- `docs/examples/todo-app-golden-path.md`
- `docs/ui-ergonomics-and-interop.md`
- `.agents/skills/fret-app-ui-builder/references/recipes/`

Recommended tasks:

- combine at least two or three real concerns in one slice:
  - forms + dialogs,
  - commands + keyboard gating,
  - async/background work + loading states,
  - persistence/config,
  - docking or multi-panel layout,
  - data-heavy tables or inspectors
- integrate one expected ecosystem dependency or workflow when relevant:
  - `tokio`,
  - `serde`,
  - `tracing`,
  - repo-local or external app integration surfaces

Look for:

- state ownership that becomes awkward under real complexity
- too many escape hatches to keep the slice moving
- default guidance that stops working outside toy demos
- ecosystem or interop gaps that force framework-shaped workarounds
- places where diagnostics are too weak to prove visible or interaction outcomes

## Lane selection rule of thumb

- Start at the lowest lane that can still expose the problem.
- Do not jump to comparison or advanced lanes to excuse a broken default lane.
- But do escalate to this lane before declaring “the framework feels fine” if toy examples are not representative of the actual product goals.
- If the issue is already known to be a parity problem, switch to the relevant source-alignment skill instead of stretching this audit.
