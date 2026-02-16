---
title: Onboarding Ergonomics + Ecosystem Interop (v1)
status: draft
date: 2026-02-15
scope: first-time app authoring + interop “Tier A” embedding (native-first; wasm follow-up)
---

# Onboarding Ergonomics + Ecosystem Interop (v1) — Workstream

This workstream targets the **first-time developer experience** of Fret:

- “I want to write a small app today” (within 30–60 minutes).
- “I want to grow into editor-grade workflows later” (docking, multi-window, viewports).
- “I want to integrate with other ecosystems” without breaking Fret’s core contracts.

This is a **workstream note**, not an ADR. If we converge on new stable surfaces, we should promote
the minimal, hard-to-change parts to ADRs.

Related reading (existing):

- Repo positioning + quick start: `README.md`
- Public crate surfaces: `docs/README.md`
- Golden-path Todo: `docs/examples/todo-app-golden-path.md`
- Ergonomics + interop note: `docs/ui-ergonomics-and-interop.md`
- Component authoring contracts: `docs/component-authoring-contracts.md`
- Action hooks (policy split): `docs/action-hooks.md`
- Interop helpers (Tier A): `ecosystem/fret/src/interop/embedded_viewport.rs`
- Fluent builder coverage workstream: `docs/workstreams/authoring-ergonomics-fluent-builder.md`

Tracking:

- TODO list: `docs/workstreams/onboarding-ergonomics-v1-todo.md`
- Milestones: `docs/workstreams/onboarding-ergonomics-v1-milestones.md`

---

## Goals

1. Make the “first app” path feel boring and obvious.
2. Reduce “concept overload” in starter templates without watering down the architecture.
3. Keep the kernel/runtime contract surfaces stable; land ergonomics work in ecosystem/tooling/docs.
4. Provide an interop story that is:
   - explicit,
   - debuggable,
   - testable,
   - and compatible with editor-grade constraints (multi-window, focus, IME, overlays, viewports).

## Non-goals (v1)

- Moving policy into `crates/*` (“mechanism-only” kernel remains the posture).
- “Same-tree” interop between full UI runtimes (too many focus/IME/semantics pitfalls).
- Requiring proc-macros to be productive (macros may reduce friction but must remain optional).

---

## Decisions (2026-02-15)

These are the decisions agreed in discussion so far:

- Template naming: **separate template name** `simple-todo` (not a flag on `todo`).
- Demo shells: `apps/fret-demo` (native) + `apps/fret-demo-web` (wasm) are the **canonical** entry points.
- Native demo shell must be able to select and run `ui_gallery` (mirroring `fret-demo-web`).
- Authoring surface posture: keep the ecosystem `UiPatch` / `UiBuilder` design (ADR 0160); do not remove it.
- Macro posture (v1): prioritize “composition macros” that reduce call-site adapters (GPUI/iced style),
  avoid DSL macros that create a hard-to-change dialect.
- Hotpatch posture (v1): ergonomics changes must remain compatible with Subsecond-style reload boundaries (ADR 0105 / 0110).
  - Avoid large stack-heavy view literals in the “first hour” templates (e.g. very large `vec![...]` child lists).
  - Prefer push-based helpers like `ui::children![...]` that are easier on stack probing under Windows hotpatch.

---

## Problem Statement (what a new user feels)

The current golden path is strong, but a new user can still experience:

1. **Concept density too early**
   - A template/demo can introduce: `Model`, explicit invalidation, MVU router, selector-derived
     state, query/async state, tokens/theme, and shadcn recipes in one go.
   - This is great as a “best practice baseline”, but it is not the best *first hour*.

2. **Invalidation mental model is powerful but heavy**
   - `watch_model(...).layout()` vs `.paint()` is a correctness + performance contract.
   - New users often don’t yet know which to choose, and the consequence is not always visible.

3. **Authoring noise**
   - `into_element(cx)` / container closures / `vec![...]` patterns still show up frequently.
   - We already have tools to reduce this (`ui::*` constructors, `AnyElementIterExt::elements()`),
     but onboarding docs/templates don’t always teach them first.

4. **Interop expectations are unclear without a “copy-paste” demo**
   - The recommended path exists (Tier A viewport embedding), but without a minimal runnable demo,
     many users default to “try to mix runtimes in one tree”, which tends to fail.

### Baseline metrics (2026-02-15; rough `rg` counts)

These are not “success metrics”, but they help quantify where authoring friction currently shows up.

| Area | Metric | Count | Why it matters |
| --- | --- | ---:| --- |
| `apps/fret-examples/src` | `.into_element(cx)` call sites | 338 | Call-site adapter noise dominates readability for new users. |
| `ecosystem/fret-ui-shadcn/src` | `.into_element(cx)` call sites | 650 | Ecosystem code still pays a lot of conversion boilerplate today. |
| `ecosystem/fret-ui-shadcn/src/ui_ext` | `impl_ui_patch_*!(Ty)` invocations | 178 | Internal precedent: “1 line per type” patch wiring is already valuable. |
| `ecosystem/fret-ui-shadcn/src` | explicit `impl UiPatchTarget for ...` | 1 | Most patch wiring is already macro-compressed internally. |

Targets (v1 direction):

- **`simple-todo` readability**: aim for near-zero explicit `.into_element(cx)` in app code; conversions should happen inside
  ecosystem constructors (`ui::*`) and/or via `children![...]`.
- **Third-party integration**: opt into `.ui()` in *one line per component type* (no hand-written `apply_ui_patch`),
  plus an “IntoElement-like” posture that avoids per-type boilerplate where possible (blanket impl or derive).

---

## Design Principles (v1)

### 1) Progressive disclosure (“laddered learning”)

We should explicitly support an onboarding ladder:

- **Step 0**: “Hello UI” (no selector/query; minimal state).
- **Step 1**: “Local state” with `Model<T>` + typed commands (MVU).
- **Step 2**: “Derived state” (selectors) once the app needs projections/counters.
- **Step 3**: “Async state” (queries) once the app needs caching + lifecycle.
- **Step 4**: “Editor-grade” modules (docking, multi-window, viewports).

The “best practice baseline” remains valid, but it becomes *Step 2/3*, not *Step 0*.

### 2) Keep the kernel stable; iterate in ecosystem + tooling

Ergonomics work should bias toward:

- `fret` (app facade),
- `fretboard` (templates + runner UX),
- `fret-ui-kit` / `fret-ui-shadcn` (authoring vocabulary),
- documentation and runnable examples.

Kernel crates (`crates/fret-ui`, `crates/fret-runtime`) should change only when we have a locked
contract rationale + evidence gates.

### 3) Interop strategy: Tier A embedding as the golden path

Interop should be “hosting”, not “mixing”:

- Foreign UI renders into an offscreen `RenderTargetId`.
- Fret hosts that surface as a viewport element.
- Input is forwarded as `ViewportInputEvent` (explicit mapping + focus policy stays in Fret).

This matches the editor/engine topology and avoids undefined behavior around IME, focus capture,
semantics, and scheduling.

We should treat Tier A as **the default recommendation** and invest in:

- a minimal demo,
- a short cookbook,
- diagnostics scripts for repro/snapshots.

---

## Proposed Changes (v1 direction)

### A) Tiered templates (“simple-todo” and friends)

Introduce template variants that map to the onboarding ladder:

- `fretboard new hello` (already minimal) → ensure it uses the most ergonomic primitives.
- `fretboard new simple-todo`:
  - `Model<T>` + typed MVU messages + shadcn components,
  - no selectors, no queries,
  - focuses on identity (`cx.keyed`) and basic layout.
- `fretboard new todo` (current golden baseline):
  - includes selector + query integration as the “official best practice baseline”.

### B) Onboarding docs: explicit “first hour” path

Add a short, friendly “first hour” doc that teaches:

- which crate to depend on (`fret`),
- the minimal mental model for `Model<T>` and typed commands,
- how to choose invalidation levels (rule-of-thumb table),
- how to build lists with keys (and why),
- how to use `ui::*` constructors to avoid props noise.

### C) Interop: a minimal runnable demo + cookbook

Add one runnable demo showing the full Tier A loop:

1) allocate/resize an offscreen target,
2) render something into it (even a simple custom pass),
3) embed it as a panel in regular UI,
4) forward input and show visible feedback.

Then add a short cookbook page with the same steps + common pitfalls.

### D) Reduce demo surface duplication (native + wasm)

We currently have a healthy separation between:

- **demo implementations** (mostly in `apps/fret-examples`, plus larger apps like `apps/fret-ui-gallery`), and
- **demo shells** (native + wasm launchers).

However, “examples vs demos vs gallery” can still feel redundant for a new contributor.

v1 direction:

- Treat `apps/fret-demo` (native) and `apps/fret-demo-web` (wasm) as the **canonical demo shells**.
  - `fretboard dev web` already shells into `apps/fret-demo-web`.
- Treat `apps/fret-examples` as the **canonical demo implementation crate** (build app + driver per demo).
- Keep larger apps (e.g. UI gallery) as independent crates, but make them runnable through the same shells where practical
  (especially on wasm).

Concrete cleanup targets (decision gates; do not land blindly):

- ensure every “core onboarding” demo (e.g. `todo_demo`, `simple-todo`) can run both native and wasm through the same selection mechanism;
- evaluate whether we still need separate `*-web` shells for individual apps once they are selectable in `fret-demo-web`.
- add `ui_gallery` to native demo selection so “one shell” works for both platforms.

### E) Optional proc-macro derive for component authors (ecosystem only)

We should evaluate adding an ecosystem proc-macro crate (e.g. `fret-ui-kit-macros`) to reduce
third-party integration friction:

- derive `UiPatchTarget` / `UiIntoElement` (or a minimal “IntoElement-like” wrapper) for common patterns,
- keep it opt-in; the ecosystem should remain usable without macros.

This aligns with the motivation in ADR 0039 (“macros planned, not required”).

Macro design best practices (v1 posture):

1) Prefer **blanket impls** over derives when they are unambiguous and low-risk.
   - Example candidate: implement `fret_ui_kit::UiIntoElement` for any `T: fret_ui::element::RenderOnce`.
   - This reduces boilerplate for third-party crates immediately and avoids proc-macro churn.

2) Use derives for **patch wiring** only (where blanket impls cannot know your fields).
   - `#[derive(UiPatchTarget)]` with explicit attributes:
     - `#[ui_patch(chrome = \"chrome\", layout = \"layout\")]` (field names configurable; no “magic” convention required).
   - Optionally allow `chrome_only` / `layout_only` modes.

3) Keep generated code boring and inspectable.
   - Always emit straightforward `merge(...)` calls into user-owned fields.
   - Avoid generating runtime behavior, globals, or policy hooks.

4) Make it work for **third-party ecosystem crates** first.
   - Target “plain builder structs” that store:
     - `{ chrome, layout }` patches
     - `children: Vec<AnyElement>` or `Option<Vec<AnyElement>>`
   - Avoid requiring apps to depend on internal crate names.

5) For `imui` integration, focus macros on patch glue, not rendering.
   - Immediate-mode wrappers often need custom rendering glue (`UiWriter`, retained bridges, viewport embedding).
   - Derives should not try to encode those policies; instead, they should help wrapper types opt into `.ui()` consistently.

Macro code sketches (what the code should look like)

Baseline today (no macros; typical patch wiring):

```rust
use fret_ui_kit::{
    ChromeRefinement, LayoutRefinement, UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout,
};

#[derive(Debug, Default, Clone)]
pub struct Dummy {
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl UiPatchTarget for Dummy {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl UiSupportsChrome for Dummy {}
impl UiSupportsLayout for Dummy {}
```

Proposed (derive just the patch wiring; marker traits stay explicit):

```rust
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, UiPatchTarget, UiSupportsChrome, UiSupportsLayout};

#[derive(Debug, Default, Clone, UiPatchTarget)]
#[ui_patch(chrome = "chrome", layout = "layout")]
pub struct Dummy {
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl UiSupportsChrome for Dummy {}
impl UiSupportsLayout for Dummy {}
```

Proposed (reduce boilerplate for `into_element(cx)` by adding a blanket impl):

```rust
use fret_ui::element::RenderOnce;
use fret_ui::{ElementContext, UiHost};
use fret_ui::element::AnyElement;
use fret_ui_kit::UiIntoElement;

impl<T: RenderOnce> UiIntoElement for T {
    #[track_caller]
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.render_once(cx)
    }
}
```

Note: this blanket impl is very ergonomic for third-party crates, but it must be validated for
coherence conflicts (if any existing types already implement `UiIntoElement` and also implement
`RenderOnce`). If it conflicts, the fallback is:

- provide `#[derive(UiIntoElement)]` that forwards to `RenderOnce::render_once`, or
- introduce a separate trait (e.g. `UiIntoElementAuto`) and have `UiBuilder` accept it in addition
  to `UiIntoElement`.

Effect on component authoring (before → after):

- Before: implement `UiIntoElement` manually for every component type (even when it already implements `RenderOnce`).
- After: components only implement `RenderOnce`; `.ui().into_element(cx)` works automatically.

### Macro stability rule (avoid future rewrites)

For v1, we should only stabilize macros that meet all of:

1) **Thin plumbing**: no hidden policy; no implicit globals; no runtime behavior injection.
2) **Local meaning**: expanding the macro shows you exactly what you get (easy to reason about).
3) **Composition win**: it removes adapters at the call site (GPUI/iced style).
4) **Ecosystem-only**: lives in `ecosystem/*` and can evolve without kernel contract churn.

This explicitly rules out “RSX/HTML-like” macros as the golden path in v1 (Dioxus/Yew/Leptos style):
those create a dominant dialect that is hard to evolve without breaking downstream code, and they can
accidentally obscure identity/keying rules that matter for focus/IME stability.

Why GPUI macros “feel big” even when the generated code is tiny

GPUI’s `#[derive(IntoElement)]` is intentionally *minimal plumbing*:

- Source: `repo-ref/zed/crates/gpui_macros/src/derive_into_element.rs`
- It simply wraps `self` into `gpui::Component<Self>` so the type participates in the uniform `IntoElement` pipeline.

The ergonomics win comes from *ubiquity*:

- components in `gpui-component` derive `IntoElement` widely (e.g. `repo-ref/gpui-component/crates/ui/src/button/button.rs`),
- which makes `.child(...)` / `.children(...)` accept components without adapters,
- and keeps authoring code “always in one fluent chain”.

For Fret, the analogous “big win” is:

- make `.ui()` participation ubiquitous across ecosystem + third-party components,
- and make “into element” adapter code disappear (blanket impl or derive).

What to copy from GPUI more directly (high impact for Fret)

GPUI’s biggest ergonomic win is not that `#[derive(IntoElement)]` saves a few lines — it is that it
enables **uniform composition** without call-site adapters.

In Fret today, new users see a lot of:

- `... .into_element(cx)`

If we want a GPUI-like experience, the v1 direction should be:

1) Ensure third-party and ecosystem components implement a single trait (`UiIntoElement` or `RenderOnce`) consistently.
2) Change ecosystem-level layout constructors (`ui::h_flex`, `ui::v_flex`, `ui::container_build`, etc.) to accept
   children as `impl UiIntoElement` (not `AnyElement`), and perform the `.into_element(cx)` conversion internally.

This moves the verbosity from *every callsite* into *one library boundary*, which is exactly the “macro feels big”
effect in GPUI.

Call-site before (today):

```rust
let input = shadcn::Input::new(st.draft.clone())
    .placeholder("Add a task")
    .submit_command(add_cmd.clone())
    .into_element(cx);

let add_button = shadcn::Button::new("Add")
    .on_click(add_cmd)
    .into_element(cx);

let row = ui::h_flex(cx, |_cx| [input, add_button])
    .gap(Space::N2)
    .into_element(cx);
```

Call-site after (target posture):

```rust
let input = shadcn::Input::new(st.draft.clone())
    .placeholder("Add a task")
    .submit_command(add_cmd.clone());

let add_button = shadcn::Button::new("Add")
    .on_click(add_cmd);

let row = ui::h_flex(cx, |_cx| [input, add_button])
    .gap(Space::N2)
    .into_element(cx);
```

Notes:

- This requires `ui::h_flex` (ecosystem) to accept `IntoIterator<Item = impl UiIntoElement>` (or a generic equivalent),
  then call `child.into_element(cx)` internally.
- This is ecosystem-only, so it does not violate the “mechanism-only kernel” rule.
- This is also the most important prerequisite for `simple-todo` to feel truly “newbie-friendly”.

### F) MVU ergonomics: document the “tick refresh” posture and a future escape hatch

`fret::mvu` currently optimizes for “it works” by forcing a full layout refresh after each update.
That is a valid onboarding tradeoff, but we should:

- document it clearly as a v1 default,
- define a path to opt into more precise invalidation later (if needed),
- add perf/diagnostic guidance for when MVU apps grow.

---

## Evidence / Regression Gates (what we should leave behind)

We should measure onboarding improvements with concrete artifacts:

- templates compile and run on the golden path (native).
- `fret-ui-gallery` / `todo_demo` remain intact (no regressions).
- interop demo has at least one `fretboard diag` scripted repro (screenshot / input trace).
- docs include a small “rule-of-thumb” section for invalidation and identity.

---

## Open Questions / Decision Gates

1. Invalidation ergonomics:
   - should we add any new sugar APIs (ecosystem-level), or only document best practices?
2. Macro delivery mechanism:
   - for third-party integration, do we prefer:
     - `macro_rules!` helpers exported from `fret-ui-kit` (lowest risk), and/or
     - proc-macro derives (higher maintenance, nicer DX)?
3. Trait posture:
   - do we converge on:
     - `UiIntoElement` as the ecosystem “IntoElement-like” surface, or
     - `RenderOnce` as the single authoring trait, with `UiIntoElement` implemented as a thin adapter?
4. Interop ownership:
   - where should the minimal interop demo live (`apps/fret-examples` vs `apps/fret-demo`)?
5. Demo shell consolidation:
   - should we move the authoritative demo registry into `fret-examples` and share it between native CLI + wasm URL selection?

---

## Fearless refactor candidates (high leverage for onboarding)

These are intentionally “big-ish” cleanups that can simplify the repo’s mental model for new contributors.
They should be approached as refactors with explicit acceptance criteria and evidence gates.

1) Single demo registry, used everywhere
   - Move the authoritative list of demo IDs → build functions into `apps/fret-examples` (or a tiny `demo-registry` module).
   - Both shells consume it:
     - native: `apps/fret-demo` CLI selection
     - wasm: `apps/fret-demo-web` URL selection
   - Goal: add/remove a demo once, not twice.

2) “One shell” story for UI gallery (native + wasm)
   - Make `ui_gallery` selectable via `apps/fret-demo` as well (mirrors existing wasm path).
   - Goal: new users always run the same thing: `fretboard dev native ...` / `fretboard dev web ...`.

3) Split `apps/fret-examples` into a small wasm-friendly core + optional heavy feature bundles
   - Today, `fret-examples` is a large dependency surface; it is useful for the repo, but it is intimidating for onboarding.
   - Proposed: keep the demo registry + onboarding demos lightweight by default; gate heavy demos behind features.

4) Align demo naming conventions and discoverability
   - Canonicalize demo IDs (snake_case) and keep consistent across:
     - CLI args
     - wasm URL (`?demo=...`)
     - docs references
   - Add `--list`/`--choose` parity between native and web (where applicable).

---

## Should we remove the patch-trait design?

Recommendation: **No** (do not remove `UiPatch` / `UiBuilder` / `UiPatchTarget`).

Rationale:

- It is the ecosystem analogue of “typed style refinement” used by GPUI (`StyleRefinement`) and by
  many builder-style Rust UI libraries: it provides a uniform, token-first authoring vocabulary.
- It is already locked in directionally by ADR 0160 (unified builder surface). Removing it would:
  - fragment the authoring dialect again,
  - push styling decisions into per-component ad-hoc APIs,
  - and likely require another convergence refactor later.
- It is an **ecosystem-only** surface, so we can iterate without destabilizing kernel contracts.

What we *should* change is not “remove the patch traits”, but “reduce call-site adapter noise” by
copying the macro + conversion patterns from other Rust UI ecosystems.

### Quantifying impact: code size + third-party integration

The “patch traits” design is not only about saving a few characters — it is about whether third-party crates can
participate in a **single, uniform authoring dialect** without bespoke glue.

As a rough rule of thumb for an external component type `Ty` that should support `.ui()`:

- **Without helpers** (today’s public posture for third-party code):
  - `impl UiPatchTarget for Ty { fn apply_ui_patch(...) { ... } }` (~8–15 LOC)
  - marker traits: `impl UiSupportsChrome/Layout/... for Ty {}` (2–6 LOC)
  - `impl UiIntoElement for Ty` (if needed) (~6–15 LOC)
  - Total: commonly **~16–36 LOC per type**.
- **With ecosystem helpers** (recommended v1 posture):
  - `fret_ui_kit::ui_component_chrome_layout!(Ty);` (or similar) **1 LOC per type**
  - plus either a blanket impl `UiIntoElement for T: RenderOnce` (0 LOC per type), or `#[derive(UiIntoElement)]` (1 LOC).

This difference compounds quickly:

- 20 component types → **~320–720 LOC** of repetitive glue vs **~20–40 LOC** of 1-line declarations.

Internal precedent already exists: `fret-ui-shadcn` uses `impl_ui_patch_*!(Ty)` widely (178 invocations) to avoid duplicating
patch wiring per component type. The key v1 gap is exporting a similar helper surface from `fret-ui-kit` for third-party crates.

### Impact on ecosystem interop (e.g. immediate-mode / “imui” wrappers)

For “imui” integrations, patch traits are still useful, but we should be explicit about what they do **and do not** provide:

- Patch traits help wrapper types participate in `.ui()` consistently (tokens/layout/chrome patches apply uniformly).
- They do not solve rendering integration by themselves. The recommended interop path remains **Tier A embedding**
  (offscreen surface + input forwarding), or a deliberately scoped wrapper element that owns its scheduling and focus policy.

---

## Macro suitability: what to copy from other Rust UI frameworks

This section is about **which macros are worth making stable**, and which ones we should avoid
because they lock us into a hard-to-change authoring dialect.

### GPUI (Zed)

What GPUI proves:

- A tiny derive macro can have a massive impact if it makes composition uniform.
- `#[derive(IntoElement)]` is minimal plumbing that removes adapters at the call site.

What to copy:

- A stable “component → element” conversion boundary (ADR 0039).
- A minimal derive (or blanket impl) that makes third-party components participate in that boundary.

What not to copy (for v1):

- Style-method generation macros at the kernel layer. In Fret, the typed “Tailwind-ish” vocabulary is
  intentionally ecosystem-owned (`fret-ui-kit` / `fret-ui-shadcn`).

### iced

What iced proves:

- A small macro like `row![ ... ]` / `column![ ... ]` can dramatically reduce boilerplate, especially
  for **heterogeneous child lists**.
- The macro does not need to be “a DSL”; it just applies a uniform `Element::from(...)` conversion
  inside an array. See: `repo-ref/iced/widget/src/helpers.rs`.

What to copy:

- A `children![ ... ]`-style macro that converts each item into a type-erased child (`AnyElement`)
  internally, so call sites do not need to write `.into_element(cx)` repeatedly.

This is likely the *lowest-risk, highest-ROI* macro for `simple-todo`.

### egui

What egui proves:

- Immediate-mode authoring can be extremely approachable, but it trades away “hard contracts” in
  identity/focus/IME/semantics when compared to editor-grade retained/declarative systems.

What to copy:

- Nothing macro-related. The value here is mainly UX discipline and API simplicity, not a macro pattern.

### Dioxus / Yew / Leptos (RSX / HTML-like macros)

What they prove:

- Macro DSLs can produce very dense UI code and are familiar to web developers.

Why it is risky for Fret (v1):

- It becomes a *dominant dialect* that is hard to evolve without breaking downstream code.
- It can hide identity/keying rules (which are critical for editor-grade focus/IME stability).

Recommendation:

- Do **not** introduce an RSX-like macro as the golden path in v1.
- If we ever add one, it should be a thin ecosystem-only layer and never replace the explicit
  contract vocabulary.

### Makepad (`live_design!`)

What it proves:

- A macro-driven UI DSL can enable powerful live-edit workflows.

Why it is risky for Fret (v1):

- It couples authoring syntax, theme schema, and runtime behavior tightly.
- It would be a large bet that is hard to unwind if it does not align with Fret’s contracts.

Recommendation:

- Treat this as inspiration for devloop tooling, not for the core authoring surface.

---

## Proposed macro set for Fret (stable-ish, ecosystem-only)

If we pick a small macro surface, the recommended order is:

1) `children![]` for heterogeneous child lists (iced-like)
   - Objective: remove repetitive `.into_element(cx)` at call sites.
   - Scope: `fret-ui-kit` (re-export from `fret-ui-shadcn::prelude`).
2) `#[derive(UiIntoElement)]` (GPUI-like, minimal)
   - Objective: let a `RenderOnce` component participate in `UiIntoElement` without boilerplate.
   - Scope: ecosystem proc-macro crate (optional).
3) `commands!{...}` / `#[derive(Action)]`-style macros (GPUI-like)
   - Objective: reduce boilerplate for `CommandId` constants + `CommandMeta` registration + default keybindings.
   - Scope: ecosystem proc-macro crate (optional; likely very valuable for third-party crates).

In addition, we should export a **macro_rules! integration helper** for patch wiring so third-party crates can
opt into `.ui()` in one line (mirrors how `fret-ui-shadcn` does it internally today).
