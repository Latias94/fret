# Runtime Contract Matrix (fret-ui)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Floating UI: https://github.com/floating-ui/floating-ui
- gpui-component: https://github.com/longbridge/gpui-component
- Tailwind CSS: https://github.com/tailwindlabs/tailwindcss
- shadcn/ui: https://github.com/shadcn-ui/ui
- virtualizer (Rust): https://github.com/Latias94/virtualizer
- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This document is a *living checklist* for what the `crates/fret-ui` runtime provides, why it
exists, and which mature ecosystem reference we align with.

It is intentionally **mechanism-only**: component policies and UI recipes belong in
`ecosystem/fret-ui-kit` and `ecosystem/fret-ui-shadcn`.

For a closure-oriented, module-by-module index (contracts → code → tests → demos), see:

- `docs/ui-closure-map.md`

## Layering (GPUI mapping)

- `gpui` (runtime substrate) ≈ `crates/fret-ui`
- `gpui-component/crates/ui` (policy + recipes) ≈ `ecosystem/fret-ui-kit` (infra) +
  `ecosystem/fret-ui-shadcn` (taxonomy + recipes)

## Contracts (P0)

### Event routing, focus, capture, and semantics

- **Module(s):** `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/widget.rs`, `crates/fret-ui/src/focus_visible.rs`
- **ADR(s):** `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- **Related ADR(s):** `docs/adr/0218-input-dispatch-phases-prevent-default-and-action-availability-v2.md`
- **Reference(s):**
  - WAI-ARIA Authoring Practices (APG): focus/keyboard interaction outcomes (policy lives in components)
- **Pointer capture arbitration (mechanism):**
  - When pointer capture switches to a different node mid-sequence (gesture arbitration), the
    previous capture target receives `PointerCancel` so it can clear pressed/drag state.
  - Evidence anchors:
    - Mechanism: `crates/fret-ui/src/tree/dispatch.rs`
    - Regression test: `crates/fret-ui/src/tree/tests/pointer_move_layers.rs`
- **Capture-phase pointer move opt-in (mechanism):**
  - `PointerRegionProps.capture_phase_pointer_moves` routes `PointerEvent::Move` via the Capture phase
    (root → target) so parent wrappers can observe moves even when a descendant has captured and/or
    stopped bubbling (gesture arena style arbitration).
  - Evidence anchors:
    - Contract surface: `crates/fret-ui/src/element.rs` (`PointerRegionProps`)
    - PointerRegion dispatch: `crates/fret-ui/src/declarative/host_widget/event/pointer_region.rs`
- **Runner snapshot seam (data-only):**
  - `fret-runtime::WindowInputContextService` publishes a window-scoped `InputContext` snapshot for
    runner/platform integration surfaces (OS menubars, etc.).
  - `InputContext.window_arbitration` (`WindowInputArbitrationSnapshot`) is the single source of
    truth for window-level modal/capture/occlusion state. It is published by the UI runtime as part
    of the `InputContext` snapshot (no separate arbitration service).
  - Evidence anchors:
    - Snapshot service: `crates/fret-runtime/src/window_input_context.rs`
    - Snapshot type: `crates/fret-runtime/src/input.rs`
    - Publishing sites: `crates/fret-ui/src/tree/{dispatch.rs,commands.rs,paint.rs}`

### Overlay/layer substrate + modal barrier

- **Module(s):** `crates/fret-ui/src/tree/mod.rs`
- **ADR(s):**
  - `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
  - `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- **Reference(s):**
  - Radix UI primitives outcomes (policy level)
  - Browser inert/modality model: modal barrier makes underlay non-interactive

### Overlay placement solver (anchoring, flip, clamp, size)

- **Module(s):** `crates/fret-ui/src/overlay_placement/mod.rs`
- **ADR(s):** `docs/adr/0064-overlay-placement-contract.md`
- **Reference(s):**
  - Floating UI: `repo-ref/floating-ui` (contract vocabulary; not a DOM implementation target)
- **Related contract(s):**
  - Cross-frame anchor geometry for declarative elements: `crates/fret-ui/src/elements/mod.rs` (`bounds_for_element`, `root_bounds_for_element`)
  - RenderTransform-aware anchoring: `docs/adr/0082-render-transform-hit-testing.md` + `crates/fret-ui/src/elements/mod.rs` (`visual_bounds_for_element`, `last_visual_bounds_for_element`)
  - Stable overlay owner identity for declarative triggers: `crates/fret-ui/src/elements/mod.rs` (`GlobalElementId`), consumed via `ecosystem/fret-ui-kit` (`OverlayOwnerId`)

### Declarative layout vocabulary (Tailwind/CSS semantics)

- **Module(s):** `crates/fret-ui/src/element.rs`, `crates/fret-ui/src/declarative.rs`
- **ADR(s):**
  - `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
  - `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`
- **Reference(s):**
  - Tailwind CSS: `repo-ref/tailwindcss` (semantic target)
  - Taffy: layout engine backend (implementation detail)

### Inherited foreground / `currentColor`-style paint cascade

- **Module(s):**
  - `crates/fret-ui/src/element.rs`
  - `crates/fret-ui/src/declarative/frame.rs`
  - `crates/fret-ui/src/declarative/mount.rs`
  - `crates/fret-ui/src/declarative/paint_helpers.rs`
  - `crates/fret-ui/src/declarative/host_widget/paint.rs`
- **ADR(s):** `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- **Reference(s):**
  - GPUI/Zed style-context traversal outcomes for inherited text/foreground styling
  - shadcn/ui `currentColor` expectations for icon/text composition
- **Contract notes:**
  - `AnyElement::inherit_foreground(...)` stamps inherited foreground on an existing subtree root
    without adding a layout node.
  - Mount/paint traversal carries that foreground as scoped paint state rather than by inserting a
    wrapper-shaped element.
  - Explicit element colors remain authoritative; inherited foreground is a fallback for opt-in
    consumers such as text/icons/spinners.
  - Ecosystem authoring helpers may still expose transitional lowering APIs, but those helpers must
    preserve the original root's layout ownership (`fill`, `shrink`, sibling flow, and width
    constraints).
  - Evidence anchors:
    - Runtime carrier + traversal: `crates/fret-ui/src/element.rs`, `crates/fret-ui/src/declarative/{frame.rs,mount.rs,paint_helpers.rs}`
    - Paint application: `crates/fret-ui/src/declarative/host_widget/paint.rs`
    - Transitional ecosystem helper: `ecosystem/fret-ui-kit/src/declarative/current_color.rs`
    - Regression gates: `ecosystem/fret-ui-shadcn/src/{dropdown_menu.rs,select.rs,tabs.rs,input_group.rs,alert.rs,context_menu.rs,menubar.rs}` and `ecosystem/fret-ui-ai/src/elements/{message.rs,task.rs}`

### Scheduling (redraw, RAF, continuous frames lease)

- **Module(s):**
  - `crates/fret-runtime/src/effect.rs`
  - `crates/fret-launch/src/runner/mod.rs`
  - `crates/fret-ui/src/elements/mod.rs`
- **ADR(s):** `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- **Portability (execution/wake/timers):** Native: `exec.background_work=threads`, `exec.wake=reliable`, `exec.timers=reliable`; wasm: `exec.background_work=cooperative`, `exec.wake=best_effort`, `exec.timers=best_effort`; mobile (future): `exec.background_work=threads`, `exec.wake=reliable`, `exec.timers=reliable` (see `docs/adr/0184-execution-and-concurrency-surface-v1.md` and `docs/adr/0054-platform-capabilities-and-portability-matrix.md`).
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
  - virtualizer (Rust): `repo-ref/virtualizer` (primary)
  - gpui-component virtual list patterns: `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`

### Rendering substrate (scene ops, clipping, shadows)

- **Module(s):**
  - `crates/fret-ui/src/paint.rs`
  - `crates/fret-ui/src/tree/mod.rs` (paint cache + deterministic replay)
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
  - `crates/fret-ui/src/text_input/mod.rs`
  - `crates/fret-ui/src/text_area/mod.rs`
- **ADR(s):**
  - `docs/adr/0044-text-editing-state-and-commands.md`
  - `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
  - `docs/adr/0071-text-input-multiline-composition-contract.md`

## Notes

- `UiTree` + `Widget` exist as an internal hosting mechanism for declarative elements; they are not
  a public component authoring model (see ADR 0066).
