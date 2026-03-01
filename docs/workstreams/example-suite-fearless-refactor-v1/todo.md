# Example Suite (Fearless Refactor v1) — TODO

This is the execution tracker for the workstream described in `design.md`.

Read first:

- Plan/rules: `docs/workstreams/example-suite-fearless-refactor-v1/design.md`
- Milestones: `docs/workstreams/example-suite-fearless-refactor-v1/milestones.md`

Status legend:

- `[ ]` Not started
- `[~]` In progress
- `[x]` Done
- `[?]` Needs triage / unclear ownership

## M0 — Lock the catalog + the ladder (docs-first)

- [ ] Add a short pointer from:
  - [ ] `docs/README.md` (“Start here” section) to this workstream
  - [ ] `docs/first-hour.md` (link to the canonical ladder + cookbook)
- [x] Add appendices to make execution repeatable and prevent drift:
  - [x] `inventory.md` (catalog → current anchors)
  - [x] `web-support-tiers.md`
  - [x] `quality-bar.md`
  - [x] `gates-and-diag-templates.md`
  - [x] `catalog-source-of-truth.md`
  - [x] `gpui-and-flutter-notes.md`
- [ ] Decide the **v1 catalog scope**:
  - [ ] which rows are “official user-facing examples”
  - [ ] which rows are “maintainer harnesses” (still runnable, but hidden by default)
- [ ] Confirm the **easy → hard** ladder is consistent with:
  - [ ] `docs/first-hour.md`
  - [ ] `docs/examples/todo-app-golden-path.md`
  - [ ] ADR 0109/0110 golden-path surfaces

Deliverable:

- [ ] A stable table in `design.md` that is “good enough” to implement against.

## M1 — Cookbook crate (small examples that compile fast)

Goal: create a lightweight, user-facing cookbook that does not pull “everything”.

- [x] Create a new crate for cookbook examples: `apps/fret-cookbook`.
  - [x] Put focused runnable examples under Cargo `examples/`.
  - [~] Keep deps minimal; prefer `fret` + ecosystem surfaces (avoid backend crates unless the example is about that boundary).
- [ ] Implement the “Stage 0–2” ladder examples as `examples/`:
  - [x] `hello`
  - [ ] `simple-todo`
  - [ ] `todo` (or keep as template-only, but ensure the runnable “golden” stays obvious)
- [ ] Add 6–10 focused cookbook examples (App Track):
  - [x] overlays basics (`overlay_basics`)
  - [x] commands + keymap (`commands_keymap_basics`)
  - [x] text input basics (`text_input_basics`)
  - [x] theme switching (`theme_switching_basics`)
  - [x] icons + assets budgets (`icons_and_assets_basics`)
  - [x] virtual list (`virtual_list_basics`)
  - [ ] async inbox + cancellation (`async_inbox_basics`)
  - [x] effects layer basics (`effects_layer_basics`)

Gates:

- [ ] Each example stamps stable `test_id`s for primary controls.
- [ ] Add at least one `fretboard diag run` script per example (or a shared suite).

## M2 — Consolidate the demo registry (reduce duplication)

Goal: avoid duplicated demo lists across native/web/tooling.

- [x] Make `fretboard list native-demos` and `--choose` show:
  - [x] “Official (user-facing)” first
  - [x] “Maintainer/Stress” behind `--all`
- [x] Add cookbook discovery + runner commands:
  - [x] `fretboard list cookbook-examples`
  - [x] `fretboard dev native --example <name>`
- [ ] Reduce or eliminate hand-maintained demo lists where practical:
  - [ ] native: prefer scanning `apps/fret-demo/src/bin` and cookbook `examples/`
  - [ ] web: keep a curated list but generate docs from it
- [ ] Decide whether we want a `tools/examples/catalog.json` manifest:
  - [ ] if yes, define schema (id, level, track, platform, run, gates, owner)
  - [ ] add a small validator (lint) and fail fast on duplicates

## M3 — Track-specific “high ceiling” examples (interop + renderer)

Interop Track:

- [ ] Add/curate “engine embedding” examples:
  - [ ] embedded viewport + input forwarding
  - [ ] external texture import (native + web)
  - [ ] gizmo + viewport integration (native)
- [ ] Docking story:
  - [ ] docking basics (cookbook)
  - [ ] docking arbitration harness (kept as maintainer-grade, but linked as “editor-grade ref”)

Renderer Track:

- [ ] Curate effects examples as “Labs”:
  - [ ] effects basics (built-in steps)
  - [ ] liquid glass / acrylic recipe (bounded)
  - [ ] custom effect tracks: CustomV1/V2/V3 (pass semantics)
- [ ] For each lab:
  - [ ] capability checks are explicit
  - [ ] budgets/degradations are documented
  - [ ] at least one diag script exists

Reference apps (app-scale):

- [ ] Confirm the v1 set (2-3) and their intended scope:
  - [ ] `workbench` (editor-grade shell)
  - [ ] `viz-studio` (viz + canvas)
  - [ ] `shader-lab` (renderer/effects)
- [ ] Decide which reference apps are docs-only in v1 vs implemented in-tree.
- [ ] For each chosen app:
  - [ ] define one end-to-end “smoke workflow” suite (scripted)
  - [ ] define one perf baseline gate (optional, if the app is perf-sensitive by design)

## M4 — Documentation + discoverability polish

- [ ] Add a single “Examples” index doc that links:
  - [ ] the ladder
  - [ ] cookbook topics
  - [ ] UI gallery (component catalog)
  - [ ] diagnostics bundles/scripts
- [ ] Ensure `README.md` only links the *canonical* path (avoid scattering run commands).
- [ ] Add contribution rules:
  - [ ] how to add a new official example
  - [ ] required gates (`test_id`, diag script, owner)

## Evidence / exit criteria (v1 “done enough”)

- [ ] A new user can find the right example in < 60 seconds.
- [ ] The canonical ladder (0–2) is boring, stable, and well-gated.
- [ ] Interop and renderer “high ceiling” are discoverable without polluting onboarding.
- [ ] Demo registry duplication is reduced and drift is prevented.
