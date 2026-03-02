# ADR 0307: Action Registry and Typed Action Dispatch (v1)

Status: Accepted

## Context

Fret’s current app/UX integration story is command-oriented:

- keymap bindings produce `CommandId` (ADR 0021),
- commands are routed via focus/window/app scopes (ADR 0020),
- command metadata drives menus and the command palette (ADR 0023),
- UI event handlers frequently use stringly command IDs or MVU-style typed routing (`MessageRouter`).

This has proven workable, but it leaves a few persistent problems in user-facing code:

1) **Stringly glue**: apps and ecosystem widgets still tend to accumulate `"prefix.{id}"` patterns,
   especially for dynamic per-item actions.
2) **No single mental model**: pointer events, keybindings, and palette commands are “almost the same”
   conceptually, but are not expressed as one coherent “action” vocabulary.
3) **Future frontend constraints**: if we want optional data-driven frontends (GenUI specs, future DSLs),
   we need stable event identities that can be referenced without embedding Rust closures.

GPUI/Zed’s reference model (non-normative) treats commands as **typed actions** routed through the dispatch path,
with queryable availability and key contexts.

Fret already has relevant substrate:

- explicit input dispatch phases and action availability semantics (ADR 0218),
- diagnostics selectors and scripted repros that want stable, explainable targets (ADR 0159),
- GenUI spec rendering which binds events to a guardrailed action vocabulary (`fret-genui-core`).

We want to introduce a typed action vocabulary that:

- stays portable (no backend types),
- preserves layering (no policy in `crates/fret-ui`),
- converges metadata/availability/dispatch semantics across pointer + key + palette.

## Decision

### D1 — Introduce a stable `ActionId` (v1)

Define `ActionId` as the canonical identity for “an invocable UI intent”.

Requirements:

- stable and deterministic,
- string-addressable (for keymap JSON, diagnostics, and data-driven frontends),
- namespaced and versionable by convention (e.g. `"workspace.tabs.close.v1"`).

**v1 compatibility strategy**:

- Implement `ActionId` as a thin wrapper around `CommandId` (or a type alias),
  so we can adopt action-first authoring without immediate keymap schema churn.

Naming convention (recommended):

- Use a dot-separated namespace with a version suffix:
  - `"workspace.tabs.close.v1"`
  - `"app.editor.save.v1"`

**v1 decision (locked)**:

- `ActionId` is implemented as an alias/wrapper over `CommandId`.
- Keymap bindings continue to reference the same string IDs (no keymap schema changes in v1).
- “Action-first” is primarily an authoring + observability + convergence refactor:
  - pointer-triggered UI, keymap, and command palette should dispatch through the same pipeline.

### D2 — Converge action metadata with command metadata

Actions need the same metadata currently used for commands:

- title, description, category,
- scope (widget/window/app),
- default keybindings.

Decision:

- Treat action metadata as a specialization of the existing command metadata registry.
- Avoid duplicating registries.

### D3 — Dispatch actions through the existing routing stack

Action dispatch semantics align with ADR 0020:

- focused widget scope,
- window scope,
- app scope.

Pointer-triggered UI events (e.g. button click) should dispatch actions through the same “command/action” pipeline,
so keybindings, UI triggers, and the palette converge on the same handler semantics and diagnostics.

### D4 — Availability is queryable and auditable

Action availability must be queryable:

- for UX (disabled states),
- for diagnostics (explain why a keybinding did not fire),
- for policy-layer gating (modal barriers, focus scopes).

Availability semantics must remain compatible with Input Dispatch v2 (ADR 0218).

### D5 — Typed unit actions are the v1 authoring surface (no payload)

For v1, we standardize on **unit actions**:

- typed Rust actions map to a stable `ActionId`,
- keymap bindings reference the action ID string,
- dispatch does not include structured payloads.

Structured payload actions are explicitly deferred to v2+ because they require:

- strict deterministic serialization,
- validation rules,
- clear policy on where payload types live (kernel vs ecosystem).

## Contract Shape (bikesheddable, but concept is fixed)

This ADR intentionally specifies the **shape**, not the exact names.

### C1 — Identity

- `ActionId`: stable identity, string-addressable.
  - v1: `ActionId` may be an alias/wrapper over `CommandId`.

### C2 — Metadata

An action registry entry should carry:

- title/description/category,
- scope (widget/window/app),
- default keybindings,
- (optional) “capability requirements” for portable degradation (future).

In v1, this should reuse the existing command metadata surface (ADR 0023) to avoid duplicated registries.

### C3 — Binding points (where actions can be triggered)

Actions can be triggered from:

1) **Keymap bindings** (ADR 0021)
2) **Command palette / menus** (ADR 0023)
3) **Pointer-triggered UI widgets** (buttons, menu items, etc.)
4) **Data-driven frontends** (GenUI) gated by catalog approval

### C4 — Dispatch + availability queries

Dispatch must:

- resolve handler scope using the same scope stack as commands (ADR 0020),
- support availability queries compatible with Input Dispatch v2 (ADR 0218),
- remain effect-driven / non-reentrant (ADR 0020).

### C5 — Typed Rust actions (ecosystem authoring sugar)

We want a stable, zero-boilerplate way to define unit actions:

```rust,ignore
mod act {
    actions!(fret, [
        WorkspaceTabClose, // "workspace.tabs.close.v1"
        EditorSave,        // "app.editor.save.v1"
    ]);
}
```

Where:

- the macro produces unit marker types plus an `ActionId` mapping,
- apps can reference actions without string constants,
- keymap bindings can reference the same string IDs (deterministic, debug-friendly).

## Layering and Ownership

To preserve ADR 0066 (mechanism vs policy):

- `crates/fret-ui` owns only the **mechanism** to:
  - attach an `ActionId` to an interactive element,
  - route a triggered action through the dispatch stack,
  - query availability at a point in the tree.
- ecosystem crates own:
  - policy for which actions exist,
  - when they are available (modal barriers, domain state),
  - UX chrome outcomes (disabled appearance, hover intent, focus restore).

## Observability Requirements (non-optional)

To keep action-first refactors safe, diagnostics must be able to answer:

- Which action ID was triggered?
- Which binding/key context matched (or why none matched)?
- Why was it blocked (availability)?
- Which handler scope handled it (focused widget vs window vs app)?

This aligns with the scripted interaction testing model (ADR 0159): action IDs become first-class evidence.

## Consequences

### Benefits

- One coherent “action” mental model across pointer + key + palette.
- Reduced stringly glue in apps and ecosystem widgets.
- Better alignment with diagnostics and scripted repros (action IDs become traceable artifacts).
- A cleaner seam for data-driven frontends (GenUI today; optional future DSLs).

### Costs / Risks

- Requires convergence work across command/keymap/dispatch codepaths (even if v1 is a thin wrapper).
- Introduces a new “name” (`ActionId`) that must be taught and documented.
- Payload actions are deferred; some use-cases may still need typed routing for dynamic items until v2.

## Alternatives Considered

- **Keep commands only**:
  - Pros: no new vocabulary.
  - Cons: does not solve the “single mental model” problem; does not provide a clear seam for spec/DSL frontends.

- **Introduce a full reactive action graph**:
  - Pros: high ergonomics.
  - Cons: conflicts with Fret’s explicit, auditable invalidation stance (ADR 0051, ADR 0223).

## Migration Plan (v1)

1) Land `ActionId` and typed unit action helpers as additive APIs.
2) Update one or two cookbook and ui-gallery examples to use action-first authoring.
3) Expose diagnostics traces for action dispatch and availability.
4) After adoption, consider (separately) whether payload actions are needed (ADR v2).
