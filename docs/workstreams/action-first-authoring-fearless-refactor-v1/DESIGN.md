# Action-First Authoring + View Runtime (Fearless Refactor v1) — Design

Status: Draft (workstream note; ADRs remain the source of truth)
Last updated: 2026-03-01

This workstream refactors Fret’s **user-facing authoring story** to close the ergonomics + correctness gap vs
Zed/GPUI while preserving Fret’s non-negotiable layering rules:

- `crates/fret-ui` stays **mechanism/contract-only** (ADR 0066).
- policy-heavy interaction lives in ecosystem crates (`fret-ui-kit`, `fret-ui-shadcn`, domain ecosystems).

The key stance:

- Treat “DSL” as an optional **future frontend**, not as the core refactor goal.
- Design the refactor so that a future DSL/frontend can compile into the same IR and reuse the same runtime loop.

---

## 0) Context (Why this is a fearless refactor)

Fret’s ambitions include:

- cross-platform: native desktop + wasm, and a clear path to mobile bring-up,
- editor-grade UX: docking, multiple windows/viewports, large virtualized surfaces, rich text,
- ecosystem scale: node graph, plot/chart, markdown/code surfaces, and room for third-party component libraries.

Today, we already have strong substrate pieces:

- declarative per-frame element tree + cross-frame element state (ADR 0028),
- app-owned model store (`Model<T>`) and explicit invalidation (ADR 0031, ADR 0051),
- diagnostics selectors + scripted interaction tests (ADR 0159),
- immediate-mode authoring frontend (`fret-imui`) that compiles down to the same declarative element taxonomy,
- data-driven GenUI spec rendering (`fret-genui-*`) as a guardrailed “spec → IR” pipeline.

The remaining gap (felt in cookbook/gallery and by new app authors) is mostly:

1) authoring density (too much glue / too many moving parts),
2) a single, cohesive “actions + keymap + command palette + pointer” story,
3) a predictable invalidation + caching loop aligned with GPUI’s “notify → dirty views → reuse ranges unless dirty”.

---

## 1) Goals

### G1 — Action-first authoring

Establish a single authoring mental model:

- UI events and keybindings resolve to **actions** (stable IDs),
- actions are dispatched through the same routing stack as other input (focus/roots),
- action availability is queryable for UX and diagnostics.

This replaces ad-hoc stringly command parsing in user code and reduces boilerplate in demos/templates.

### G2 — A view runtime with hooks, built on app-owned models

Provide a coherent view-level authoring facade that composes:

- local state (element/view scoped),
- derived state (selectors),
- async resources (queries),
- while preserving the app-owned, handle-based paradigm (ADR 0223).

### G3 — Multi-frontend convergence

Make three authoring frontends converge on the same underlying IR and runtime contracts:

1) Rust declarative authoring (components/builders),
2) immediate-mode authoring (`imui` / `UiWriter`),
3) GenUI spec rendering (data-driven UI specs).

### G4 — Keep the kernel mechanism-only

No policy drift into `crates/fret-ui`:

- `fret-ui` provides action routing primitives and observation/invalidations,
- ecosystem crates define dismissal rules, focus restore semantics, hover intent, default spacing, etc.

### G5 — A cleanup phase that leaves the architecture “clean”

Plan an explicit “delete legacy” milestone:

- deprecate/retire redundant routing glue and duplicated surfaces once adoption is sufficient,
- keep docs, examples, and templates aligned to one boring golden path.

---

## 2) Non-goals (v1)

- Building a full end-user plugin sandbox or a “runtime UI editor” product.
- Replacing the shadcn/material3 ecosystem recipes; this workstream focuses on the substrate authoring loop.
- Moving theme policy or interaction policy into `crates/fret-ui`.
- Introducing a global implicit reactive graph (“signals everywhere”); dependencies remain explicit and auditable.
- Committing to a new long-lived public API for third-party crates before we have in-tree adoption evidence.

---

## 2.1) Decision Snapshot (v1)

These v1 decisions are locked to keep the implementation plan executable:

- `ActionId` is an alias/wrapper over `CommandId` (no keymap schema churn in v1).
- The v1 view runtime lands in `ecosystem/fret` (golden path). Splitting into dedicated crates is deferred.
- Typed unit actions only (no structured payload actions in v1).

---

## 3) References (Sources of truth)

### ADRs

- Declarative elements + element state: `docs/adr/0028-declarative-elements-and-element-state.md`
- Component authoring model: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- App-owned models: `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- Observation + invalidation: `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
- Focus + command routing: `docs/adr/0020-focus-and-command-routing.md`
- Keymap: `docs/adr/0021-keymap-file-format.md`
- Command metadata/palette: `docs/adr/0023-command-metadata-menus-and-palette.md`
- Unified builder surface (ecosystem-only): `docs/adr/0160-unified-authoring-builder-surface-v1.md`
- Cache roots/view cache semantics: `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`
- Authoring paradigm (state helpers): `docs/adr/0223-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- Input dispatch v2 (prevent_default/action availability): `docs/adr/0218-input-dispatch-phases-prevent-default-and-action-availability-v2.md`
- Diagnostics + scripted UI tests: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`

### Workstreams (existing)

- GPUI parity (experience + performance): `docs/workstreams/gpui-parity-refactor.md`
- Authoring paradigm consolidation: `docs/workstreams/authoring-paradigm-gpui-style-v1.md`
- Fluent builder ergonomics: `docs/workstreams/authoring-ergonomics-fluent-builder.md`
- imui facade (implemented): `docs/workstreams/imui-authoring-facade-v2.md`
- GenUI spec rendering (implemented): `docs/workstreams/genui-json-render-v1.md`

### Upstream references (non-normative)

- Zed/GPUI (typed actions + key dispatch): `repo-ref/zed/crates/gpui`
- gpui-component (ergonomic builder patterns): `repo-ref/gpui-component/crates/ui`

---

## 4) Proposed Architecture (IR-first, action-first)

### 4.1 Layering overview (conceptual)

```
           ┌────────────────────────────────────────────────────────────┐
           │ App / Ecosystem policy                                     │
           │  - fret-ui-shadcn / fret-ui-material3 / docking / editors   │
           │  - action handlers, focus/overlay policies, recipes         │
           └───────────────▲───────────────────────────────▲────────────┘
                           │                               │
             (authoring frontends)                         │
     ┌─────────────────────┼─────────────────────┐         │
     │ Rust declarative     │ Immediate-mode      │ GenUI   │
     │ (View + hooks)       │ (UiWriter / imui)   │ (spec)  │
     └───────────────▲──────┴──────────────▲──────┴────▲────┘
                     │                     │            │
                     └──────────────┬──────┴──────┬─────┘
                                    │
                         (shared: UI IR + actions)
                                    │
           ┌────────────────────────▼────────────────────────┐
           │ Kernel runtime substrate (`crates/fret-ui`)      │
           │  - AnyElement / ElementKind / stable identity    │
           │  - input routing + action dispatch primitives    │
           │  - model observation + invalidation              │
           │  - cache roots / view cache reuse                │
           └──────────────────────────────────────────────────┘
```

The refactor goal is not to add a new runtime. It is to:

- stabilize **action identity** and action routing contracts,
- make the default authoring story feel cohesive across all frontends.

### 4.2 “Action-first” means: IDs, not closures, in the element IR

To keep the door open for future DSL/hot-reloadable frontends, the UI IR should prefer:

- “button triggers action `app.editor.save`”
  over
- “button stores an arbitrary Rust closure”.

Rust closures remain a convenience surface, but the runtime should be able to lower them to a
stable action ID + handler table.

### 4.3 View runtime responsibilities (ecosystem-level)

The new “view authoring runtime” should:

- run on top of app-owned models (`Model<T>`) and explicit invalidation,
- provide hooks for derived/query state,
- register action handlers in a view-scoped handler table,
- define a stable view-cache boundary story (align with ADR 0213 + GPUI parity).

---

## 5) Design: Action System (v1)

This workstream proposes a typed action system aligned with GPUI’s mental model while integrating
with Fret’s existing command/keymap/palette infrastructure.

### D1 — Action identity is stable and string-addressable

Introduce a stable `ActionId` that is:

- deterministic,
- debug-friendly,
- used by: keymap bindings, command palette, scripted diagnostics, GenUI specs.

In v1, we can implement `ActionId` as a thin wrapper over `CommandId` (or make them type aliases)
to avoid churn while still tightening semantics at the authoring level.

### D2 — Action metadata and command metadata converge

Actions need the same metadata you already store for commands:

- title/description/category,
- scope (widget/window/app),
- default keybindings.

The intent is to converge the registries, not duplicate them.

### D3 — Availability and routing are queryable

Action dispatch must support:

- “is action available?” queries for UX (disabled buttons, menu items),
- diagnostics tracing (why a keybinding did not fire),
- policy-layer gating (e.g. modal barriers).

Action availability must stay compatible with Input Dispatch v2 semantics (ADR 0218).

---

## 6) Design: View Runtime + Hooks (v1)

### D4 — View authoring is ecosystem-level, built on kernel contracts

The view runtime should live in ecosystem crates (e.g. `ecosystem/fret`) and be implemented in terms of:

- `ElementContext` (construction),
- `UiTree` (frame orchestration),
- `Model<T>` + invalidation observation (state),
- cache roots / view-cache boundaries (caching).

### D5 — Hooks are explicit, keyed, and auditable

We want “hooks ergonomics” without implicit reactive graphs:

- `use_selector`: built on `fret-selector` (deps signature + observation rails),
- `use_query`: built on `fret-query`,
- `use_state`: stored in element/view state slots using stable identity and keyed variants for loops.

### D6 — Dirty/notify closes the loop

Define a canonical way for view logic to request a refresh:

- `cx.notify()` marks the view/cache root dirty,
- view cache reuse happens unless dirty or in “inspection/picking” mode,
- diagnostics can report “why did we rebuild?” at the cache root level.

---

## 7) Multi-frontend convergence

### 7.1 Immediate-mode (imui)

imui remains:

- an authoring frontend,
- compiled down to declarative IR (`AnyElement`),
- using the same state helpers via `UiWriter` adapters (`fret-authoring`).

This workstream adds:

- a first-class “dispatch action” seam usable from imui widgets without string commands,
- stable conventions for attaching `test_id`/semantics to imui output for diag scripts.

### 7.2 GenUI (spec rendering)

GenUI already provides a guardrailed “spec → IR” path and an action queue vocabulary.

This workstream aligns “actions as stable IDs” so that:

- GenUI specs can trigger the same action IDs as Rust-authored UI,
- action metadata/availability can be surfaced in the same diagnostics tooling.

---

## 8) Migration Plan (fearless, but staged)

See:

- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- Tracker: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`

High-level plan:

1) Lock the contracts (ADRs) and landing sites (crate placement).
2) Land minimal action registry + dispatch glue (additive).
3) Land a minimal view runtime and migrate 2–3 cookbook demos.
4) Expand adoption to ui-gallery + docking/workspace shells.
5) Cleanup/retire redundant routing glue and legacy surfaces once adoption is sufficient.

---

## 9) Cleanup (“leave it clean”)

The workstream is not complete until we explicitly:

- remove or quarantine legacy glue surfaces that are no longer recommended,
- update templates/docs to the single golden path,
- keep diagnostics and scripted repros stable (selectors and action traces).

---

## 10) Proposed v1 API Shapes (Illustrative, not final)

This section is intentionally concrete so implementation tasks are reviewable.

### 10.1 Typed unit actions

Target authoring outcome (Rust):

```rust,ignore
mod act {
    actions!(fret, [
        EditorSave,        // "app.editor.save.v1"
        WorkspaceTabClose, // "workspace.tabs.close.v1"
    ]);
}
```

Where:

- the macro defines unit marker types (no payload),
- each marker type maps to a stable `ActionId`,
- the `ActionId` string becomes the keymap/diagnostics-visible identity.

### 10.2 Binding UI triggers to actions

Target authoring outcome:

```rust,ignore
shadcn::Button::new("Save")
    .action(act::EditorSave)
    .disabled(!cx.is_action_available(act::EditorSave))
```

Policy note:

- `disabled`/availability is ecosystem-owned; the kernel only provides the query/dispatch substrate.

### 10.3 Registering action handlers (view/app layer)

Target authoring outcome:

```rust,ignore
cx.on_action(act::EditorSave, |app, cx| {
    app.save_current_buffer();
    cx.notify();
});
```

Important constraints (to keep future frontends possible):

- handlers are registered in a table keyed by `ActionId`,
- IR nodes reference only `ActionId`, not arbitrary captured closures.

### 10.4 View runtime shape (ecosystem)

Target authoring outcome:

```rust,ignore
struct MyView {
    st: Model<MyState>,
}

impl View for MyView {
    fn render(&mut self, cx: &mut ViewCx<'_, App>) -> Elements {
        let st = cx.watch_model(&self.st).layout().cloned_or_default();

        let derived = cx.use_selector(
            |cx| DepsBuilder::new(cx).model_rev(&self.st).finish(),
            |_cx| expensive_derive(&st),
        );

        ui::v_flex(cx, |cx| {
            ui::children![
                cx;
                shadcn::Label::new(derived.title),
                shadcn::Button::new("Save").action(act::EditorSave),
            ]
        })
        .into_element(cx)
        .into()
    }
}
```

Notes:

- views still use the app-owned `Model<T>` paradigm (ADR 0031/0223),
- selectors/queries remain explicit and auditable (no implicit reactive graph).

---

## 11) Data flow (keybinding + pointer) — what should be observable

### 11.1 Keybinding → action

Desired traceable steps:

1) key event occurs
2) keymap resolution selects a binding
3) binding resolves to `ActionId`
4) availability is checked
5) dispatch resolves the handler scope (focused widget → window → app)
6) handler runs

Diagnostics must be able to explain:

- “why did this not fire?” (no binding vs blocked availability vs dispatch path mismatch)

### 11.2 Pointer click → action

Desired traceable steps:

1) pointer event hits an interactive element
2) the element triggers an `ActionId`
3) availability is checked (if applicable)
4) the same dispatch pipeline runs (scope resolution)

The key invariant:

- pointer-triggered action and keymap-triggered action must converge on the same handler semantics.

---

## 12) Landing sites (proposed)

This is a proposed mapping. The ADRs must lock it.

- `crates/fret-runtime`:
  - `ActionId` as a portable identity type (v1 may alias `CommandId`).
- `crates/fret-app`:
  - action/command metadata registration (unified registry).
- `crates/fret-ui`:
  - mechanism primitives to attach `ActionId` to interactive elements,
  - dispatch hooks and availability query plumbing (no policy).
- `ecosystem/fret` (or `ecosystem/fret-view`):
  - `View` + `ViewCx` authoring runtime,
  - action handler tables,
  - hook sugar (`use_selector`, `use_query` re-exports).
- `ecosystem/fret-imui` / `ecosystem/fret-authoring`:
  - ability to emit actions from immediate-mode widgets without string glue.
- `ecosystem/fret-genui-*`:
  - align action IDs and metadata where possible; keep spec guardrails.

---

## 13) Cleanup constraints (do not skip)

The cleanup milestone must:

- delete or quarantine legacy glue only after adoption,
- leave a single recommended golden path in docs/templates,
- keep diagnostics selectors stable and update scripts when IDs move.

This workstream should treat cleanup as a first-class deliverable, not “nice to have”.
