# Postman-like API Client — First-Contact Fret Audit

## Scope

This audit applies the `fret-framework-consumer-audit` skill to a concrete product probe:

- a first-time Fret user wants to build a desktop-first API debugging app similar to Postman,
- with collections/history, request editor, response viewer, environments/settings, command palette,
  and async HTTP workflows.

Audit mode:

- lane: `complex app / ecosystem-fit`
- primary real-app probe: `Workspace shell / IDE-lite`
- secondary probes:
  - `Markdown / knowledge viewer` for response/content viewing
  - `Data-heavy admin surface` for history/collections/stateful tables

This is intentionally **not** a component-parity audit and **not** a toy-demo audit.

## Product slice used for judgment

The target “API client workbench” was treated as needing these user-facing capabilities:

1. Left sidebar for collections / history / environments
2. Center tabbed request editor
3. Request form fields (method, URL, headers, auth, body)
4. Async HTTP send / cancel / retry
5. Response viewer (pretty/raw/headers/timing)
6. Settings + persisted app/project configuration
7. Command palette and keyboard-first actions
8. Optional rich preview surfaces (markdown, JSON/code, tables)

## Public surfaces a first-time user would realistically touch

The current public/default route naturally points a new user through:

- `docs/first-hour.md`
- `docs/examples/README.md`
- `fretboard new simple-todo`
- `fretboard new todo --command-palette --ui-assets`

To keep moving toward an API client, that same user would then need to discover and combine:

- `docs/workspace-shell.md`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `docs/integrating-tokio-and-reqwest.md`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-cookbook/examples/data_table_basics.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `docs/adr/0014-settings-and-configuration-files.md`

That jump itself is one of the main findings.

## Findings

### P0 — The default first-contact path is misaligned with the product class Fret claims to target

Broken truth:

- A first-time user trying to build an editor-grade or tool-style app should have one obvious
  public baseline that is closer to a real workbench than a todo list.

Evidence:

- `docs/first-hour.md` explicitly frames the default path as “small native UI app quickly” and
  lists editor-grade features as non-goals.
- `docs/examples/README.md` keeps the default ladder on `hello` -> `simple-todo` -> `todo`.
- `docs/workstreams/example-suite-fearless-refactor-v1/inventory.md` still shows `workbench` only
  as a planned reference app.
- `docs/workspace-shell.md` states that a cohesive workspace shell is still the biggest perceived
  gap versus editor-grade references.

Owner:

- docs + public example/template strategy + app-scale teaching surface

Evidence anchors:

- [docs/first-hour.md](/Users/frankorz/Documents/projects/rust/fret/docs/first-hour.md#L1)
- [docs/examples/README.md](/Users/frankorz/Documents/projects/rust/fret/docs/examples/README.md#L18)
- [docs/workstreams/example-suite-fearless-refactor-v1/inventory.md](/Users/frankorz/Documents/projects/rust/fret/docs/workstreams/example-suite-fearless-refactor-v1/inventory.md#L29)
- [docs/workspace-shell.md](/Users/frankorz/Documents/projects/rust/fret/docs/workspace-shell.md#L1)

Next move:

- ship one public “tool app / workbench” baseline instead of asking users to bridge from `todo`

### P0 — The async/networking story is documented, but still too indirect for a network-centric first app

Broken truth:

- For an API client, the first public path to “send an HTTP request and render async state” should
  be obvious and near the default app lane.

Evidence:

- `docs/examples/README.md` classifies `query_basics` as a lab/high-ceiling example instead of a
  default starting point.
- `docs/integrating-tokio-and-reqwest.md` is marked `Draft`.
- Async fetch requires installing a `FutureSpawnerHandle` global and knowing about
  `cx.data().query_async(...)`; this is reasonable architecture, but not first-contact-simple.
- No scaffold template includes an async fetch or persistence lane by default.

Owner:

- docs + app-scale recipes + scaffold strategy

Evidence anchors:

- [docs/examples/README.md](/Users/frankorz/Documents/projects/rust/fret/docs/examples/README.md#L107)
- [docs/integrating-tokio-and-reqwest.md](/Users/frankorz/Documents/projects/rust/fret/docs/integrating-tokio-and-reqwest.md#L1)
- [apps/fret-cookbook/examples/query_basics.rs](/Users/frankorz/Documents/projects/rust/fret/apps/fret-cookbook/examples/query_basics.rs)

Next move:

- add an app-scale “HTTP workbench” recipe or scaffold slice that installs the spawner and leaves a
  request/response/query example behind

### P1 — Public scaffolds are still todo-centric; they do not give a convincing baseline for a tool app

Broken truth:

- A user should not need to stitch together four unrelated examples just to reach the first
  believable version of an API tool.

Evidence:

- `fretboard new` only exposes `hello`, `simple-todo`, `todo`, and `empty`.
- `todo` does support `--command-palette` and `--ui-assets`, which is good, but the generated
  baseline still centers on todo-specific state and one small query slice rather than shell,
  settings, history, and async workflows.

Owner:

- scaffold templates

Evidence anchors:

- [apps/fretboard/src/cli/contracts.rs](/Users/frankorz/Documents/projects/rust/fret/apps/fretboard/src/cli/contracts.rs#L290)
- [crates/fretboard/src/scaffold/templates.rs](/Users/frankorz/Documents/projects/rust/fret/crates/fretboard/src/scaffold/templates.rs#L1461)
- [crates/fretboard/src/scaffold/templates.rs](/Users/frankorz/Documents/projects/rust/fret/crates/fretboard/src/scaffold/templates.rs#L317)

Next move:

- add a new scaffold tier oriented around “tool app” or “workbench”, not around list CRUD

### P1 — The user must manually compose too many app-scale concerns from disconnected surfaces

Broken truth:

- A first-time user building a Postman-like app should find one cohesive “how these pieces fit
  together” story for shell + commands + async + settings + content preview.

Evidence:

- Shell guidance lives in `docs/workspace-shell.md` and `workspace_shell_demo`.
- Command palette guidance lives in a recipe note and optional scaffold flag.
- Async HTTP guidance lives in a separate draft integration note.
- Data-heavy and content-heavy viewing live in separate cookbook/examples.
- Settings/files are locked in an ADR, but not connected to a first public scaffolded UI path.

Owner:

- docs + examples + builder recipes

Evidence anchors:

- [docs/workspace-shell.md](/Users/frankorz/Documents/projects/rust/fret/docs/workspace-shell.md)
- [apps/fret-examples/src/workspace_shell_demo.rs](/Users/frankorz/Documents/projects/rust/fret/apps/fret-examples/src/workspace_shell_demo.rs)
- [apps/fret-examples/src/editor_notes_demo.rs](/Users/frankorz/Documents/projects/rust/fret/apps/fret-examples/src/editor_notes_demo.rs)
- [apps/fret-examples/src/markdown_demo.rs](/Users/frankorz/Documents/projects/rust/fret/apps/fret-examples/src/markdown_demo.rs)
- [apps/fret-cookbook/examples/data_table_basics.rs](/Users/frankorz/Documents/projects/rust/fret/apps/fret-cookbook/examples/data_table_basics.rs)
- [docs/adr/0014-settings-and-configuration-files.md](/Users/frankorz/Documents/projects/rust/fret/docs/adr/0014-settings-and-configuration-files.md)

Next move:

- publish one combined “API workbench lite” recipe that explicitly names the owner layer of each concern

### P1 — The architecture has a sensible settings model, but the first-contact UX for settings/env/history is still weak

Broken truth:

- A Postman-like app should have an obvious story for persisted settings, environments, layout, and
  history from the moment the app becomes real.

Evidence:

- ADR 0014 clearly defines file-backed settings and strong typing.
- The first-hour path and default scaffolds do not bridge that architecture into a ready-to-copy
  settings/editor workflow.

Owner:

- examples + settings UI recipe + scaffolds

Evidence anchors:

- [docs/adr/0014-settings-and-configuration-files.md](/Users/frankorz/Documents/projects/rust/fret/docs/adr/0014-settings-and-configuration-files.md)
- [docs/first-hour.md](/Users/frankorz/Documents/projects/rust/fret/docs/first-hour.md)

Next move:

- add a first-party settings/environment example that pairs settings files with a real UI surface

## What already looks promising

- The default action-first local-state story is coherent for request forms and simple pane-local UI.
- The public scaffold already supports `--command-palette` and `--ui-assets`, so some “tool app”
  concerns are not starting from zero.
- There are real building blocks in-tree for shell, content, assets, tables, and async integration;
  the main gap is **cohesive first-contact composition**, not total absence of capability.

## Implementation probe notes (`api-workbench-lite`)

This audit was pushed past document review into a concrete first-contact probe:

- example: `apps/fret-examples/src/api_workbench_lite_demo.rs`
- native entry: `apps/fret-demo/src/bin/api_workbench_lite_demo.rs`
- shell baseline diag script:
  `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-baseline.json`
- shell + response diag repro:
  `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json`

The probe exposed additional consumer-facing friction that is easy to miss in abstract review.

### P0 — Tool-app commands are easy to scope incorrectly on first contact

Broken truth:

- A first-time user should not have to internalize Fret's command scope routing model before they
  can wire a top toolbar button like “Send request” or “Open environments”.

What the probe exposed:

- The first probe version registered `SendRequest` and `OpenSettings` as `CommandScope::Widget`
  because that pattern is highly visible in cookbook-style action examples.
- In this tool-app shell, that decision caused toolbar buttons to render disabled under command
  gating even though the actions themselves existed.
- The fix was to move those commands back to the default window-level scope, which is the correct
  owner for workbench chrome actions.

Evidence:

- compile/authoring path: `apps/fret-examples/src/api_workbench_lite_demo.rs`
- failed diag bundle before the scope correction:
  `target/fret-diag-api-workbench-lite/sessions/1776156011819-54145/1776156166669-script-step-0005-wait_until-timeout/`
- later diag traces show the click dispatch being handled after the scope correction:
  `target/fret-diag-api-workbench-lite/sessions/1776156651145-57384/1776156779847/script.result.json`

Owner:

- examples + docs + action/command teaching surface

### P1 — The default `locals_with((...))` story does not scale cleanly to the first real request flow

Broken truth:

- The first realistic request-submit path in a tool app should stay on the default local-state
  surface without surprising tuple-size or transaction-shape pressure.

What the probe exposed:

- The initial request-submit implementation naturally wanted to coordinate more than eight
  `LocalState<T>` slots at once.
- That pushed the probe off the neat `locals_with((...)).on::<A>(...)` lane and into
  `cx.actions().models::<A>(...)` even though the state was still view-local.

Evidence:

- current implementation fallback:
  `apps/fret-examples/src/api_workbench_lite_demo.rs`
- hidden capture tuple support lives in:
  `ecosystem/fret/src/view.rs`

Owner:

- app authoring ergonomics + docs

### P1 — Public example wiring did not include the command-palette feature on the first try

Broken truth:

- A first-party example crate that wants to demonstrate `.command_palette(true)` should not require
  a consumer to rediscover the feature gate by trial and compile error.

What the probe exposed:

- The probe compiled against `fret` without the `command-palette` feature enabled in
  `apps/fret-examples`, so `.command_palette(true)` initially failed as a method-not-found error.
- The fix was straightforward, but only after repo spelunking confirmed that the surface existed
  elsewhere.

Evidence:

- feature wiring:
  `apps/fret-examples/Cargo.toml`
- public facade surface:
  `ecosystem/fret/src/lib.rs`

Owner:

- example wiring + first-contact feature discoverability

### P0 — Click-driven request flows can easily pick the wrong query policy and silently replay side-effecting HTTP calls

Broken truth:

- Clicking “Send request” once in a Postman-like tool should issue one bounded request, converge to
  one reviewable terminal state, and avoid silently replaying `POST` traffic from the render path.

What the probe exposed:

- The first implementation used `query_async(...)` with `stale_time = 0` and retry enabled for a
  submission keyed by request sequence.
- In a real window, the next redraw after success can arrive with a frame gap, which makes the
  query look like a stale remount and starts a fresh fetch before the UI reads the previous
  terminal state.
- On a manual request surface this is a serious product hazard: a single click can replay a
  side-effecting `POST`, while the response area still appears stuck in `Loading`.
- The fix was not a transport rewrite. The fix was to treat this lane as a manual, one-shot
  observation:
  - give the response query a non-zero `stale_time`,
  - disable automatic retries for click-driven requests,
  - clear the observed submission once the response has been materialized into local UI state.

What this means for framework consumers:

- The underlying `query_async` mechanism works.
- The first-contact guidance for “user-triggered network mutation/result materialization” is still
  easy to misuse if the example only teaches query syntax and not query policy semantics.
- For a tool app, “wrong policy but valid code” is a real ergonomics problem because the user sees
  duplicated requests and unstable terminal state instead of a compile error.

Evidence:

- failing diag script:
  `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json`
- failing run result before the policy correction:
  `target/fret-diag-api-workbench-lite/sessions/1776157395729-62219/1776157525075/script.result.json`
- failing bundle before the policy correction:
  `target/fret-diag-api-workbench-lite/sessions/1776157395729-62219/1776157540345-script-step-0005-wait_until-timeout/`
- passing run result after the policy correction:
  `target/fret-diag-api-workbench-lite-logprobe-3/sessions/1776159135060-71714/script.result.json`
- passing bundle after the policy correction:
  `target/fret-diag-api-workbench-lite-logprobe-3/sessions/1776159135060-71714/1776159262898-api-workbench-lite.shell-and-response/`
- passing screenshot after the policy correction:
  `target/fret-diag-api-workbench-lite-logprobe-3/sessions/1776159135060-71714/screenshots/1776159262780-api-workbench-lite.shell-and-response/window-4294967297-tick-178-frame-178.png`

Owner:

- async recipe/docs + example authoring pattern + query-policy teaching surface

### Shell and first-response evidence are now reviewable

What worked:

- The shell-mounted baseline script passed and captured bounded diagnostics artifacts for the
  sidebar/header/request/response shell before any network interaction.
- After correcting the manual-request query policy, the shell + response script also passes and
  captures a reviewable first-response state.
- This gives the team both a stable shell proof and a bounded “first response landed” proof for
  the “does this look and behave like the intended shadcn-style tool shell?” question.

Evidence:

- passing baseline script:
  `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-baseline.json`
- passing run directory:
  `target/fret-diag-api-workbench-lite-shell/sessions/1776157657074-63698/1776157786971-api-workbench-lite.shell-baseline/`
- screenshot artifact:
  `target/fret-diag-api-workbench-lite-shell/sessions/1776157657074-63698/screenshots/1776157786864-api-workbench-lite.shell-baseline/window-4294967297-tick-26-frame-26.png`
- passing shell + response script:
  `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json`
- passing shell + response run directory:
  `target/fret-diag-api-workbench-lite-logprobe-3/sessions/1776159135060-71714/1776159262898-api-workbench-lite.shell-and-response/`
- passing shell + response screenshot:
  `target/fret-diag-api-workbench-lite-logprobe-3/sessions/1776159135060-71714/screenshots/1776159262780-api-workbench-lite.shell-and-response/window-4294967297-tick-178-frame-178.png`

Owner:

- diagnostics + design review loop

## Recommended next probe

The next artifact should not be another todo.

Build and gate a narrow first-party example or scaffold named something like:

- `api-workbench-lite`

Minimum surface:

1. Left sidebar: collections + history
2. Center tabs: one request editor tab and one response tab
3. Request area: method + URL + headers/body
4. Async send: `reqwest`/Tokio-backed query or mutation path
5. Response area: pretty/raw tabs plus status/time badges
6. Settings/env dialog
7. Command palette

Evidence to leave behind:

- one runnable example or scaffold
- one diag script with `capture_bundle`
- one screenshot set for the shell + request/response states
- one layout sidecar for the main shell split

## Short verdict

Fret does not look incapable of supporting a Postman-like app.

The first-contact problem is that the current public path still teaches “small app authoring”
better than it teaches “real tool app composition”. For a first-time user, the main pain is not
that a single API is obviously broken; it is that the public story for assembling shell, async,
settings, and content surfaces into one believable product is still too distributed.
