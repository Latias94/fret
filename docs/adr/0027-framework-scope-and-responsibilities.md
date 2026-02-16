# ADR 0027: Framework Scope and Responsibilities (Fret vs Editor App)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Fret’s goal is to be a **game-editor-grade UI framework** (Unity/Unreal-like docking, multi-window,
multiple embedded engine viewports, layered rendering) rather than “a full editor”.

If we mix *framework responsibilities* with *editor application / engine responsibilities* inside
the same “must implement” contract set, we will:

- lock ourselves into engine/editor-specific policy too early,
- over-couple `fret-ui` / `fret-render` to domain logic (assets, tools, undo),
- increase the chance of large rewrites when building real editors.

At the same time, the framework must proactively reserve the **extension points** needed by a real
engine editor, so that the editor app can be built *on top* without forking the UI runtime later.

References / inspiration:

- Zed/GPUI ownership and “app-owned models” patterns:
  - https://zed.dev/blog/gpui-ownership
- Zed performance work (frame scheduling + in-flight resources):
  - https://zed.dev/blog/120fps
- Zed settings UX and strong-typed configuration motivation:
  - https://zed.dev/blog/settings-ui
- Zed/GPUI code anchors (non-normative):
  - ownership + update closures + notify/effect cycle:
    `repo-ref/zed/crates/gpui/src/view.rs`, `repo-ref/zed/crates/gpui/src/app.rs`
  - window scheduling and draw/present split:
    `repo-ref/zed/crates/gpui/src/window.rs`
  - scene/list ordering and batching strategy:
    `repo-ref/zed/crates/gpui/src/scene.rs`
  - file-backed settings + keymap layering:
    `repo-ref/zed/crates/settings`

## Decision

### 1) Fret is responsible for “UI infrastructure”, not “editor domain logic”

**In scope (Fret framework responsibilities):**

- **Platform boundary**: multi-window lifecycle, event translation, clipboard/IME plumbing hooks
  (ADR 0003).
- **App-owned model infrastructure**: typed handles + borrow-friendly update APIs for shared state
  (ADR 0031).
- **Retained UI runtime**: tree, layout, invalidation, hit-testing, focus/capture, event routing
  (ADR 0005, ADR 0020).
- **Docking UX infrastructure**: dock data model + operations + persistence contract, tear-off windows,
  drag previews as overlays (ADR 0011, ADR 0013, ADR 0017).
- **Commands / keymap / discoverability**: command metadata, scope-aware routing, keymap file format,
  `when` gating (ADR 0020, ADR 0021, ADR 0022, ADR 0023).
- **Styling infrastructure**: style tokens and theme resolution rules (theme content remains app-owned)
  (ADR 0032).
- **Rendering infrastructure**: backend-agnostic display list + ordering semantics + GPU resource handle
  boundary (ADR 0002, ADR 0004, ADR 0009).
- **Viewport embedding contracts**: render target handles + host-provided WGPU context topologies +
  submission ordering + input forwarding contract (ADR 0007, ADR 0010, ADR 0015, ADR 0025).
- **Settings/persistence infrastructure for UI**: file-scoped keymap/layout/settings loading, layering,
  and update propagation (ADR 0014). (The *schema/content* remains app-owned.)

**Out of scope (Editor app / engine responsibilities):**

- asset database, import pipeline, dependency tracking,
- scene/entity/component model, serialization formats,
- selection/picking, gizmos, tool modes, snapping policies,
- undo/redo history policy and storage (beyond what’s needed for docking layout edits),
- project management, build systems, compilation, indexing.

### 2) Fret exposes contracts; apps provide policy and state

The framework provides **stable contracts** (IDs, events, effects, persistence shapes), and the
editor app owns:

- the models/state (selection, project state, tool state),
- the policies (undo coalescing, asset import rules, tool behaviors),
- the integrations (engine callbacks, plugin ecosystem policy).

### 3) Editor features must remain buildable as a separate layer

When a requirement is “editor feature” rather than “UI infrastructure”, it should live as:

- an **example editor app** (e.g. `fret-demo` extended, or a future `fret-editor-demo` crate), or
- a separate repo/crate that depends on `fret-*`,

not as a hard dependency of `fret-core` / `fret-ui` / `fret-render`.

## Consequences

- Fret stays focused and portable (desktop now, wasm later), while still being editor-grade.
- We avoid forcing all users into a particular engine/editor architecture.
- The framework can be iterated and stabilized independently of editor domain experiments.

## Future Work

- Decide whether to create a dedicated “example editor” crate tree (and where its docs live).
- Define minimal plugin integration points that remain UI-only (panels/commands/menus), while leaving
  engine/editor plugin policy app-owned.
