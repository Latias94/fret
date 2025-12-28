# Runtime Contract Matrix (fret-ui)

This document is a *living checklist* for what the `crates/fret-ui` runtime provides, why it
exists, and which mature ecosystem reference we align with.

It is intentionally **mechanism-only**: component policies and UI recipes belong in
`crates/fret-components-ui` and `crates/fret-components-shadcn`.

## Layering (GPUI mapping)

- `gpui` (runtime substrate) ≈ `crates/fret-ui`
- `gpui-component/crates/ui` (policy + recipes) ≈ `crates/fret-components-ui` (infra) +
  `crates/fret-components-shadcn` (taxonomy + recipes)

## Contracts (P0)

### Event routing, focus, capture, and semantics

- **Module(s):** `crates/fret-ui/src/tree.rs`, `crates/fret-ui/src/widget.rs`, `crates/fret-ui/src/focus_visible.rs`
- **ADR(s):** `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- **Reference(s):**
  - WAI-ARIA Authoring Practices (APG): focus/keyboard interaction outcomes (policy lives in components)

### Overlay/layer substrate + modal barrier

- **Module(s):** `crates/fret-ui/src/tree.rs`
- **ADR(s):**
  - `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
  - `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- **Reference(s):**
  - Radix UI primitives outcomes (policy level)
  - Browser inert/modality model: modal barrier makes underlay non-interactive

### Overlay placement solver (anchoring, flip, clamp, size)

- **Module(s):** `crates/fret-ui/src/overlay_placement.rs`
- **ADR(s):** `docs/adr/0064-overlay-placement-contract.md`
- **Reference(s):**
  - Floating UI: `repo-ref/floating-ui` (contract vocabulary; not a DOM implementation target)

### Declarative layout vocabulary (Tailwind/CSS semantics)

- **Module(s):** `crates/fret-ui/src/element.rs`, `crates/fret-ui/src/declarative.rs`
- **ADR(s):**
  - `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
  - `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`
- **Reference(s):**
  - Tailwind CSS: `repo-ref/tailwindcss` (semantic target)
  - Taffy: layout engine backend (implementation detail)

### Scheduling (redraw, RAF, continuous frames lease)

- **Module(s):**
  - `crates/fret-runtime/src/effect.rs`
  - `crates/fret-runner-winit-wgpu/src/runner.rs`
  - `crates/fret-ui/src/elements.rs`
- **ADR(s):** `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- **Reference(s):**
  - GPUI/Zed `Window::refresh()` mental model: `repo-ref/zed/crates/gpui/src/window.rs`

### Scroll + virtualization contracts

- **Module(s):**
  - `crates/fret-ui/src/scroll.rs`
  - `crates/fret-ui/src/virtual_list.rs`
  - `crates/fret-ui/src/declarative.rs` (VirtualList element implementation)
- **ADR(s):**
  - `docs/adr/0047-virtual-list-data-source-and-stable-item-keys.md`
  - `docs/adr/0070-virtualization-contract.md`
- **Reference(s):**
  - TanStack Virtual: `repo-ref/virtual` (contract vocabulary + behaviors)
  - gpui-component virtual list patterns: `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`

### Rendering substrate (scene ops, clipping, shadows)

- **Module(s):**
  - `crates/fret-ui/src/paint.rs`
  - `crates/fret-ui/src/tree.rs` (paint cache + deterministic replay)
- **ADR(s):**
  - `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
  - `docs/adr/0060-shadows-and-elevation.md`
  - `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`

### Theme/tokens baseline (shadcn semantic keys)

- **Module(s):** `crates/fret-ui/src/theme.rs`
- **ADR(s):** `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- **Reference(s):**
  - shadcn/ui token semantics: `repo-ref/ui` (v4 taxonomy surface)

### Text input engine (IME/caret/selection; hard semantics)

- **Module(s):**
  - `crates/fret-ui/src/text_input.rs`
  - `crates/fret-ui/src/text_area.rs`
- **ADR(s):**
  - `docs/adr/0044-text-editing-state-and-commands.md`
  - `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
  - `docs/adr/0071-text-input-multiline-composition-contract.md`

## Compatibility-only (non-contract)

### Retained widgets (temporary)

- **Location:** `crates/fret-components-ui/src/widget_primitives`
- **Rule:** retained widgets are not part of the runtime contract surface; they exist only as a temporary compatibility layer while declarative authoring fully replaces widget-based authoring.
- **ADR:** `docs/adr/0066-fret-ui-runtime-contract-surface.md`
