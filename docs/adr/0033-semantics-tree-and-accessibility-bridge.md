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

`fret-platform` implements an OS-specific accessibility bridge (macOS AX, Windows UIA, Linux AT-SPI, etc.).
The bridge consumes the window’s semantics tree and translates it to platform APIs.

Recommended default (P0):

- Use `accesskit` as the bridge implementation where possible, to avoid per-OS bespoke trees and to keep behavior consistent.

Fret’s core contracts remain OS-agnostic:

- `fret-core` stores semantics data types (optional),
- `fret-ui` produces per-window semantics snapshots/updates,
- `fret-platform` bridges.

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
