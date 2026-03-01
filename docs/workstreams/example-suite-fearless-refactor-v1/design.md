# Example Suite (Fearless Refactor v1)

This workstream redesigns Fret’s examples as a **product surface** (Flutter-like): an onboarding ladder,
a cookbook, a component catalog, and a set of regression-ready harnesses.

The goal is not “more demos”. The goal is **fewer, clearer, and more teachable entry points** that scale
from “my first app” to “editor-grade GPU UI + engine embedding”, while keeping the repo maintainable.

This is **not** an ADR. If we change hard-to-change contracts, we must write/adjust ADRs.

## Problem statement

Today, the repo has many runnable apps/demos, but the learning path is easy to lose:

- There are multiple run entry points (native/web) and some demo lists are duplicated.
- “User-facing examples” and “maintainer/stress harnesses” are mixed, which hurts discoverability.
- New users struggle to answer: “Which example should I start from?” and “How do I find a reference for X?”

We want examples to be *intentional teaching artifacts* and *regression evidence anchors*.

## Goals

- Provide an explicit **easy → hard** ladder with a small set of canonical “start here” examples.
- Cover both primary personas:
  - **App authors** (batteries-included, shadcn-first).
  - **Engine embedders / editor builders** (multi-window, docking, viewports, renderer effects).
- Make ecosystem integration teachable:
  - shadcn recipes, Material 3 alignment (in-progress), docking, node graph, charts/plot, markdown/code-view, icons.
- Make renderer/effects capability obvious:
  - built-in effect steps and the custom effect tracks (CustomV1/V2/V3 + pass/plan semantics).
- Attach **gates** to examples:
  - stable `test_id`s, `fretboard diag` scripts/suites, and (where practical) small Rust tests.
- Reduce long-term maintenance cost by consolidating “demo registry” and avoiding duplicated lists.

## Non-goals

- Turning examples into a full tutorial site in v1.
- Free-for-all “shader playground” without budgets/capability gates (effects must remain bounded).
- Moving policy into `crates/fret-ui` (mechanism-only stays mechanism-only).

## Personas (who this is for)

1) **App author (desktop-first)**: wants a fast path with good defaults and a few “copy/paste” patterns.
2) **Engine embedder**: wants viewport embedding, external textures/videos, and GPU context ownership options.
3) **Component author**: wants mechanism vs policy clarity and a cookbook of “correct authoring patterns”.
4) **Maintainer**: wants regression-ready harnesses and a small, stable user-facing story.

## The “Flutter-like” example product surface

We explicitly mirror the high-level learning story many people expect:

- **Templates** (“create an app”): `fretboard new ...`
- **Cookbook** (“how do I do X?”): small, focused runnable examples
- **Gallery** (“what components exist and how do they behave?”): `fret-ui-gallery`
- **Labs** (“cool/experimental, optional”): renderer effects, advanced visuals, experimental components
- **Diagnostics** (“debug + regressions”): `fretboard diag` scripts/suites anchored to stable `test_id`s

### Cookbook vs UI Gallery (positioning)

We already have a UI Gallery. The cookbook is **not a second component catalog**.

- **Cookbook** (this workstream): "how do I do X?" recipes that teach authoring patterns and
  ecosystem seams (commands, overlays, text input, virtualization, effects, etc.).
  - One file ≈ one lesson.
  - Optimized for copy/paste and first-day onboarding.
- **UI Gallery**: "what exists and how does it behave?" component pages + conformance harnesses.
  - Optimized for breadth, parity work, and regression detection (APG/Radix/shadcn alignment).

If we later want a "cookbook gallery" UI surface, it should be a **thin index** (listing + deep
links + "run this example") rather than duplicating the UI Gallery's role.

### Cookbook commands (native)

The cookbook is intended to be GPUI-like: small Cargo `examples/` where one file demonstrates one
concept.

Canonical discovery + run commands:

```bash
fretboard list cookbook-examples
fretboard dev native --example overlay_basics
```

Notes:

- Cookbook examples are currently native-only. Web support is tracked separately (see `web-support-tiers.md`).
- `fretboard dev native` only supports one selector at a time: `--demo`, `--bin`, or `--example`.

Supporting appendices for making this plan executable (and preventing drift):

- Current inventory + mapping: `docs/workstreams/example-suite-fearless-refactor-v1/inventory.md`
- Web support tiers: `docs/workstreams/example-suite-fearless-refactor-v1/web-support-tiers.md`
- Official example quality bar + `test_id` conventions: `docs/workstreams/example-suite-fearless-refactor-v1/quality-bar.md`
- Gates + diag script templates: `docs/workstreams/example-suite-fearless-refactor-v1/gates-and-diag-templates.md`
- Catalog single source of truth plan: `docs/workstreams/example-suite-fearless-refactor-v1/catalog-source-of-truth.md`

## Proposed structure (repo-level)

We keep the existing reality (many apps/harnesses), but we reframe and reorganize the “user-facing” slice.

### A) Canonical ladder (the only things we recommend on day 1)

| Stage | Name | Form | Primary crates | Teaches | Run |
|---|---|---|---|---|---|
| 0 | hello | template | `fret` + `fret-ui-shadcn` | minimal UI + layout | `fretboard new hello ...` |
| 1 | simple-todo | template | `fret` + `fret-ui-shadcn` | MVU + models + keyed lists | `fretboard new simple-todo ...` |
| 2 | todo (golden path) | template + doc | + `fret-selector`, `fret-query` | derived/async state baseline | `fretboard new todo ...` |
| 3 | components gallery | demo (cookbook-ish) | shadcn + overlays | “what exists” + overlay behaviors | `fretboard dev native --bin components_gallery` |
| 4 | ui gallery | app (catalog) | `fret-ui-gallery` | per-component pages + conformance | `cargo run -p fret-ui-gallery` |

Rule: stages 0–2 must stay **boring and stable**.

### Reference apps (app-scale examples)

In addition to cookbook-scale examples, we should maintain a small number of “app-scale” reference apps.
These are intentionally larger and opinionated; they serve as:

- realistic integration references (docking + commands + settings + assets + diagnostics),
- architecture templates (“how to structure a real product”),
- regression anchors for editor-grade workflows.

We should keep the set small (2–3) and treat them as product surfaces, not random demos.

Proposed v1 set (names TBD):

| ID | Scope | Primary intent | Should include | Gate posture |
|---|---|---|---|---|
| workbench | editor-grade shell | “the Fret IDE shell” | docking, multi-window (native), command palette, settings, file tree, markdown/code view | diag suites + perf baseline |
| viz-studio | viz + canvas | “interactive viz workspace” | charts/plot, canvas interactions, node graph (optional), perf-friendly virtualization | perf + input scripts |
| shader-lab | renderer/effects | “bounded effects playground” | built-in effect steps, CustomV1/V2/V3 tracks, budget/capability reporting | capability-gated scripts |

Reference app constraints:

- Keep the set small (2-3) and treat each app as a product surface with an owner.
- Do not use reference apps as a dumping ground for experiments: experiments should live in cookbook
  examples or Labs first.
- Prefer reusing cookbook surfaces and diag scripts (or suites) rather than inventing bespoke
  automation per app.
- Keep mechanism vs policy boundaries intact: these apps should depend on ecosystem layers, not
  re-implement policy in app code and not pull backend crates unless the point is interop.

### B) Cookbook tracks (focused examples; “how do I do X?”)

We define three tracks:

- **App Track**: the default “application author” story.
- **Interop Track**: viewports, external resources, embedding.
- **Renderer Track**: effects semantics and custom effects (bounded, capability-gated).

Cookbook implementation (initial):

- In-tree cookbook crate: `apps/fret-cookbook` (Cargo `examples/`)

Suggested v1 sequencing (App Track):

1) `cookbook.hello` -> first run, layout + theme baseline
2) `cookbook.hello_counter` -> MVU + Model + keyed UI updates
3) `cookbook.commands_keymap_basics` -> commands + shortcuts + availability gating
4) `cookbook.text_input_basics` -> text input + submit/clear patterns
5) `cookbook.overlay_basics` -> overlay lifecycle + focus restore baseline
6) `cookbook.effects_layer_basics` -> renderer "wow" without leaving the safe path
7) `cookbook.theme_switching_basics` -> token-driven theming + light/dark verification
8) `cookbook.icons_and_assets_basics` -> icons + assets + currentColor + budgets
9) `cookbook.virtual_list_basics` -> perf + virtualization basics
10) `cookbook.async_inbox_basics` -> async workflows + cancellation patterns

## Example catalog (v1 proposal)

This table is the “source of truth” for what we *intend* to maintain as user-facing examples.
Implementation may live in different harnesses initially, but the intent and run commands should be stable.

Legend:

- **Level**: 0 (first run) → 4 (editor-grade)
- **Form**: Template / Example / Gallery Page / Lab / Harness

| ID | Level | Track | Theme | Form | Native | Web | Teaches | Proposed gate |
|---|---:|---|---|---|---|---|---|---|
| hello | 0 | App | layout | Template | ✅ | ✅ | window + layout + text | smoke (first frame) |
| simple-todo | 1 | App | state | Template | ✅ | ✅ | MVU + keyed lists | diag script: add/remove rows |
| todo | 2 | App | state | Template | ✅ | (optional) | selectors + queries baseline | diag script: “golden path” actions |
| cookbook.commands_keymap_basics | 2 | App | input | Example | ✅ | ✅ | commands + shortcuts | diag script: key injection |
| cookbook.text_input_basics | 2 | App | input | Example | ✅ | ✅ | text input + submit/clear commands | diag script: submit + value gate |
| cookbook.overlay_basics | 2 | App | overlays | Example | ✅ | ✅ | dialog basics + focus restore | diag suite: overlay conformance |
| cookbook.virtual_list_basics | 2 | App | perf | Example | ✅ | ✅ | virtualization + stable identity | perf gate (worst-frame) |
| cookbook.async_inbox_basics | 2 | App | async | Example | ✅ | ✅ | inbox drain + cancellation | unit test + diag run |
| cookbook.theme_switching_basics | 2 | App | theming | Example | ✅ | ✅ | preset switch + token reads | screenshot gate (light/dark) |
| cookbook.icons_and_assets_basics | 2 | App | assets | Example | ✅ | ✅ | icons + image/SVG budgets | screenshot gate |
| data-table | 3 | App | data | Gallery Page | ✅ | ✅ | headless table + sizing/pinning | layout gate + perf baseline |
| markdown-and-code | 3 | App | docs | Example | ✅ | ✅ | markdown + syntax + copy button | screenshot gate |
| docking-basics | 3 | Interop | docking | Example | ✅ | ⛔ | dock model + UI policy | diag script + checklist anchor |
| docking-arbitration | 4 | Interop | docking | Harness | ✅ | ⛔ | multi-root overlays + input arbitration | diag suite (ADR checklist) |
| multi-window-tearoff | 4 | Interop | windows | Lab/Harness | ✅ | ⛔ | tear-off + DPI + drag | manual + diag evidence |
| embedded-viewport | 3 | Interop | viewport | Example | ✅ | (optional) | viewport surface + input forwarding | diag script: pointer mapping |
| gizmo-viewport | 4 | Interop | viewport | Example | ✅ | ⛔ | gizmo + viewport tool math | screenshot + interaction script |
| external-texture-import | 3 | Interop | render I/O | Example | ✅ | ✅ | external texture paths | diag run + capability check |
| external-video-import | 4 | Interop | render I/O | Lab | ✅ | ⛔ | video import (platform-specific) | platform-gated smoke |
| cookbook.effects_layer_basics | 2 | Renderer | effects | Example | ✅ | ✅ | built-in effect steps + semantics | screenshot gate |
| liquid-glass | 4 | Renderer | effects | Lab | ✅ | ⛔ | bounded glass/acrylic recipe | perf + screenshot gate |
| customv1 | 4 | Renderer | custom effects | Lab | ✅ | ✅ | CustomV1 bounded ABI | capability-gated smoke |
| customv2 | 4 | Renderer | custom effects | Lab | ✅ | ✅ | CustomV2 authoring + presets | script-driven repro |
| customv3-pass-graph | 4 | Renderer | custom effects | Lab | ✅ | ✅ | CustomV3 pass planning semantics | script + perf attribution |
| node-graph-basics | 3 | App | canvas | Example | ✅ | ⛔ | node graph navigation + selection | screenshot gate |
| charts-plot | 3 | App | viz | Example | ✅ | ✅ | charts/plot interactions | screenshot + drag script |
| inspector-and-diag | 3 | Maint | diagnostics | Harness | ✅ | ✅ | inspector + bundle export | `fretboard diag` docs gate |

Notes:

- “Web” support is intentionally selective: we should keep a small, high-signal subset running in
  `fretboard dev web --demo ...`, and treat the rest as native-first until contracts mature.
- Some rows map to existing demos/pages today; v1 is about making the catalog stable and discoverable.

## How GPUI does examples (useful reference pattern)

We keep a pinned local reference for GPUI in `repo-ref/zed/crates/gpui/`.

GPUI’s example strategy is instructive:

- It keeps many small, topic-focused runnable examples under `repo-ref/zed/crates/gpui/examples/`.
- Each file typically demonstrates one concept (input, popover, scrollable, data table, painting, etc.).
- The “real app” reference is Zed itself (the editor), which acts as the app-scale integration example.

For Fret, this suggests a healthy split:

- **Cookbook**: small Cargo `examples/` (GPUI-like).
- **Gallery**: `fret-ui-gallery` (component catalog + conformance).
- **Reference apps**: 2–3 app-scale shells (Zed-like “real product” anchors).

## Single source of truth (demo registry)

We want one “catalog” to drive:

- `fretboard list ...` (native/web/examples)
- `fretboard dev native --...` and `fretboard dev web --...` selection UX
- docs tables (generated or manually kept in sync)

Implementation options (to decide in TODO):

1) **Directory scan** (low effort): scan `apps/fret-demo/src/bin`, `apps/*/examples`, plus a static web list.
2) **Manifest file** (recommended long-term): a `tools/examples/catalog.json` that includes:
   - id, name, track, level, platform support, run command template, gating anchors, owning layer.
3) **Rust registry module** (type-safe but heavier): `apps/fretboard/src/catalog.rs`.

Recommended direction: start with (1) to reduce duplication, then migrate to (2) when the table stabilizes.

### Official vs maintainer demos

`apps/fret-demo/src/bin` currently contains many maintainer/stress demos. For onboarding, we keep a
small “official” list surfaced by default:

```bash
fretboard list native-demos
fretboard list native-demos --all
```

## Quality bars (what makes an example “official”)

An “official” example must have:

- a stable **ID** (used in commands, diag scripts, docs),
- a short **purpose statement** and “what to edit” notes,
- stable `test_id`s for the primary interactive controls,
- at least one **gate**:
  - `fretboard diag run ...` script, and/or
  - a small Rust test asserting a contract outcome, and/or
  - a perf baseline (for perf-sensitive examples).

## Open questions / decision gates

- Where should the cookbook live?
  - A new lightweight crate with `examples/` (recommended), or
  - keep using `apps/fret-examples` and split modules (higher compile cost).
- How strict should web parity be for v1?
  - A curated subset (recommended), or attempt broad parity early (risk: churn + slow iteration).
- Do we want a dedicated “Labs” app (like Flutter Gallery), or keep labs as separate examples?
