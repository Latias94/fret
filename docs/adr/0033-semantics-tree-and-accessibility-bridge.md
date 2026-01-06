# ADR 0033: Semantics Tree and Accessibility Bridge (A11y-Ready UI Infrastructure)

Status: Accepted

## Context

Fret targets editor-grade UI: multi-window docking, multiple viewports, and multi-root overlays (menus, popups, modals).
Even if full accessibility (A11y) is not a day-one deliverable, **the infrastructure contract must be decided early**
to avoid an invasive rewrite later.

Late A11y retrofits typically require changing:

- stable identity rules (so semantics nodes survive reordering),
- focus navigation and keyboard routing,
- text input/IME integration (selection/composition),
- overlay/modal rules (what is reachable and when).

References:

- Multi-root overlays and modal blocking:
  - `docs/adr/0011-overlays-and-multi-root.md`
- Focus and command routing:
  - `docs/adr/0020-focus-and-command-routing.md`
- IME vs shortcuts boundary:
  - `docs/adr/0012-keyboard-ime-and-text-input.md`
- Declarative element identity and cross-frame state:
  - `docs/adr/0028-declarative-elements-and-element-state.md`
- Framework vs editor-app scope boundary:
  - `docs/adr/0027-framework-scope-and-responsibilities.md`
- Practical cross-platform bridge library:
  - https://github.com/AccessKit/accesskit

## Decision

### 1) Fret provides a Semantics Tree as a first-class, renderer-independent output

Each window maintains a **Semantics Tree** that represents UI meaning independently from rendering:

- nodes with stable identity,
- role and state,
- actions,
- label/value text,
- geometry (bounds in window coordinates),
- parent/child relationships,
- z-order/overlay relationships.

The semantics tree is owned by the UI runtime and is consumed by a platform A11y bridge.

Default update strategy (P0):

- `fret-ui` produces a **full semantics snapshot** for each window when it builds UI for a frame.
- `fret-platform` (bridge) is responsible for translating snapshots to OS APIs, and may compute diffs internally
  for efficiency.

### 2) Stable semantics identity follows the UI identity model

Semantics nodes must have stable identity across frames.

- In the retained-widget world, this is naturally `NodeId`.
- In the declarative-element world (ADR 0028), this is the element’s stable key (`GlobalElementId` path + explicit keys).

The semantics contract must not depend on pointer addresses or transient object lifetimes.

### 3) Multi-root overlays are modeled explicitly in semantics

The semantics tree must preserve the overlay stack (ADR 0011):

- base UI root(s),
- overlay/popup roots,
- modal roots.

Modal semantics behavior is defined:

- when a modal root is visible and blocks underlay input, semantics navigation must be restricted to that modal root
  and any roots above it.
- underlay semantics nodes are not reachable until the modal is dismissed.

### 4) Minimal, stable semantics schema

Fret defines a minimal, stable schema that is sufficient for editor workflows:

- Roles: `Window`, `Panel`, `Button`, `Tab`, `Menu`, `MenuItem`, `TextField`, `List`, `TreeItem`, `Viewport`, etc.
- States: `Focused`, `Disabled`, `Selected`, `Expanded`, `Checked`, etc.
- Actions: `Focus`, `Invoke`, `SetValue` (text), `ScrollBy`, `Increment/Decrement` (optional).
- Relations (P0, minimal):
  - `labelled_by`: declare that this node’s accessible name is provided by another node (e.g. a
    `TabPanel` is labelled by its selected `Tab`).
  - `described_by`: declare that this node’s accessible description is provided by another node
    (e.g. a `Dialog` is described by a `DialogDescription` text node).
  - `controls`: declare that this node controls another node (e.g. the selected `Tab` controls its
    active `TabPanel`).

Relationship edges are part of the **portable semantics contract**, not platform-specific glue.
Backends may ignore relations they cannot represent, but `fret-ui` must preserve them in snapshots.

Tabs baseline (P0):

- A tabs widget must expose `TabList` / `Tab` / `TabPanel` roles.
- The active `TabPanel` must be `labelled_by` the selected `Tab`.
- The selected `Tab` must `controls` the active `TabPanel`.

Dialog baseline (P0):

- A dialog content root must expose `Dialog` / `AlertDialog` roles.
- A dialog must be `labelled_by` its title text node when present.
- A dialog may be `described_by` its description text node when present.
- When a dialog is installed as a modal barrier layer (`barrier_root`), the platform bridge should
  mark it as modal where the backend supports it (e.g. AccessKit `modal` flag, comparable to
  `aria-modal="true"` on the web).
- When a modal barrier layer (`barrier_root`) is active, the bridge should also apply "hide others"
  semantics by only exposing roots at-or-above the barrier z-index (Radix uses `hideOthers(content)`
  for `Select` and similar overlays even though the content is not a dialog).

This schema is framework-level infrastructure; editor-domain meanings (e.g. “Gizmo Mode”) remain app-owned.

### 5) Text input semantics include selection and composition hooks

For text-editing widgets, semantics must expose enough data for platform A11y and IME correctness:

- current value (or a safe excerpt policy),
- selection range,
- IME composition range (when applicable).

The exact IME event flow remains defined by ADR 0012, but semantics must not block future correctness work.

Default text exposure policy (P0):

- Short text values are exposed in full.
- For very large text (future code editor), semantics may expose an excerpt plus ranges, but **selection and IME
  composition ranges must remain exact**.

### 6) Platform boundary: bridge is implementation-specific, contract is stable

Platform backend crates (`fret-platform-*`) implement OS-specific accessibility bridges (macOS AX, Windows UIA, Linux AT-SPI, etc.).
The bridge consumes the window’s semantics tree and translates it to platform APIs.

Recommended default (P0):

- Use `accesskit` as the bridge implementation where possible, to avoid per-OS bespoke trees and to keep behavior consistent.
  - In this workspace, the AccessKit mapping logic lives in `crates/fret-a11y-accesskit`, with winit adapter glue in `crates/fret-runner-winit`.
  - Note: `accesskit_winit` support lags behind `winit` releases. While the workspace uses `winit 0.31` (pre-stable), the adapter glue in
    `crates/fret-runner-winit` may remain a stub. We plan to re-enable the real AccessKit adapter once `winit 0.31` is stable and a compatible
    `accesskit_winit` is available.

Fret’s core contracts remain OS-agnostic:

- `fret-core` stores semantics data types (optional),
- `fret-ui` produces per-window semantics snapshots/updates,
- platform backend bridges (e.g. `fret-runner-winit`).

Default geometry unit (P0):

- Semantics bounds are expressed in **logical units** (DIP / logical px) consistent with the UI coordinate space.
- The platform bridge converts to physical pixels only when required by OS APIs.

## Consequences

- Fret can support accessibility and automation (testing tools) without rewriting UI identity/focus/overlay rules later.
- Multi-window + modal semantics become predictable and portable.
- Editor apps can build domain tools without coupling domain meaning into framework semantics.

## Open Questions (To Decide Before Implementation)

### Locked P0 Choices

1) **Snapshot cadence**:
   - Semantics snapshots are produced together with the UI build/layout/paint pipeline.
   - No independent “semantics-only rebuild” loop is introduced in P0 (keeps scheduling and identity consistent).

2) **Automation and inspection**:
   - A semantics inspector is exposed only via observability/inspector hooks (ADR 0036).
   - Fret does not ship a built-in “accessibility inspector UI” as a framework deliverable.

Additional locked behavior:

- Semantics node identity is derived from UI identity (NodeId / GlobalElementId) and must remain stable across frames.
- Modal overlays restrict semantics reachability exactly as they restrict input reachability (ADR 0011).
