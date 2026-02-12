# ADR 0054: Platform Capabilities and Portability Matrix (Runtime, Not cfg)

Status: Accepted
Scope: `fret-platform-*` + runner contracts; consumed by commands/UI via `InputContext`/`when`

## Context

Fret targets:

- desktop first (Windows/macOS/Linux),
- wasm/WebGPU as a mid-term target,
- mobile later (out of scope for near-term implementation, but should not be blocked by core contracts).

Today, several assumptions are “implicitly desktop”:

- multi-window is always available,
- external drag-and-drop yields real filesystem paths,
- clipboard supports arbitrary operations,
- IME and text input behaviors are consistent.

These assumptions leak into:

- `fret-core` event payloads (e.g. external drag file paths),
- command gating decisions (`when` expressions),
- demo/editor behaviors.

Makepad’s web backend demonstrates a useful posture:

- a single entry-point message pump for wasm,
- runtime knowledge of OS/browser capabilities,
- dependency/resource loading handled as part of the platform loop.

We want the same *conceptual* advantage without adopting Makepad’s DSL or platform stack.

## Decision

Introduce a **runtime capability model**: `PlatformCapabilities`.

Capabilities are produced by the runner/platform backend and are:

- pure data (serializable/debuggable),
- platform-agnostic (no `winit` types, no `PathBuf`),
- stable inputs to `when` expressions and UI/command gating.

### 1) Capabilities are runtime, not compile-time

We do not rely solely on `cfg(target_arch)` to decide feature availability.

Rationale:

- wasm environments can differ (browser quirks, permission gating),
- desktop environments can differ (Wayland vs X11, sandboxing),
- multi-monitor/multi-window support may be constrained by backend.

### 2) Capabilities are used for gating, not for behavior forks inside widgets

Widgets and commands should not grow ad-hoc platform branches.
Instead:

- commands are enabled/disabled via `when` using capabilities,
- platform-dependent behavior is expressed via effects (ADR 0001 / ADR 0003),
- the platform backend returns events/results (clipboard, file read, etc.).

### 3) Initial capability set (P0)

We define a minimal set of booleans/enums that cover the “hard portability” boundaries:

- **Execution / scheduling** (see ADR 0175; primarily diagnostics and portability guardrails)
  - Note: this reference is `docs/adr/0184-execution-and-concurrency-surface-v1.md`.
  - `exec.background_work`: enum:
    - `none` (no background execution; UI thread only)
    - `cooperative` (best-effort background work; no threads, time-sliced/cooperative)
    - `threads` (background threads available; true multi-thread execution)
  - `exec.wake`: enum:
    - `none`
    - `best_effort` (wake may be coalesced/delayed; may be global-only)
    - `reliable` (wake reliably reaches the next driver boundary)
  - `exec.timers`: enum:
    - `none`
    - `best_effort` (timers exist but precision is limited / clamped)
    - `reliable` (timers are available with typical OS/runtime semantics)
- **Windows**
  - `ui.multi_window`: `bool` (wasm/mobile: false; desktop: true)
  - `ui.window_tear_off`: `bool` (may equal `multi_window`)
  - `ui.cursor_icons`: `bool` (desktop: true; wasm/mobile: false or limited)
  - Windowing quality signals (stable keys; recommended for editor-grade multi-window UX):
    - `ui.window_hover_detection`: enum:
      - `none` (cannot reliably determine window-under-cursor)
      - `best_effort` (may be stale/missing; avoid “hover another window while dragging” UX promises)
      - `reliable` (window-under-cursor selection is accurate enough for tear-off + cross-window docking)
    - `ui.window_set_outer_position`: enum:
      - `none` (cannot programmatically move windows)
      - `best_effort` (movement is possible but may be clamped/denied or visibly delayed; avoid follow-mode)
      - `reliable` (movement works predictably; follow-mode is viable)
    - `ui.window_z_level`: enum:
      - `none` (cannot request OS z-level changes)
      - `best_effort` (may work inconsistently; avoid relying on AlwaysOnTop during drags)
      - `reliable` (z-level requests behave predictably)
- **Clipboard**
  - `clipboard.text`: `bool`
  - `clipboard.files`: `bool` (future; often false on web)
- **External drag-and-drop**
  - `dnd.external`: `bool`
  - `dnd.external_payload`: enum:
    - `none`
    - `file_token` (web/sandbox; portable handle)
    - `text` (web/desktop)
  - `dnd.external_position`: enum (quality/degradation signal):
    - `none` (no usable external drag position updates)
    - `best_effort` (positions may be stale/missing; do not rely on hover UX)
    - `continuous` (reliable hover position updates)
- **IME/text input**
  - `ime`: `bool`
  - `ime.set_cursor_area`: `bool`
- **Filesystem semantics**
  - `fs.real_paths`: `bool` (false on web)
  - `fs.file_dialogs`: `bool` (may be permission-gated)
- **Rendering backend**
  - `gfx.webgpu`: `bool`
  - `gfx.native_gpu`: `bool`

This list is intentionally small; it can expand later, but early tokens should remain stable.

### 4) Integration points

#### a) `InputContext` / `when`

Capabilities become part of the command resolution input context (ADR 0022 / ADR 0023):

- `when` expressions may gate on capability keys (stringly keys at the expression level),
- command palette and menus use the same gating so UI stays consistent.

#### b) External drag payload portability (ADR 0053)

Capabilities explicitly define the external drag payload contract (token/text/none) and any
degradation signals for hover position quality.

This is a contract guardrail:

- core events must not expose `PathBuf`,
- access to dropped data must go through effect-driven APIs (token/handle).

#### c) Docking and multi-window (ADR 0013 / ADR 0017)

When `ui.multi_window == false`:

- tear-off commands should be disabled,
- docking remains single-window, but logical layouts can still exist (persistence remains useful).

When `ui.multi_window == true` but windowing quality signals are not `reliable`:

- policy layers (docking tear-off follow, cross-window hover selection, z-order nudges) must degrade gracefully
  using the capability quality signals above, instead of introducing platform branches inside widgets.

### 5) Observability

Capabilities must be visible in diagnostics:

- log at startup (one line),
- expose in the debug HUD/inspector (ADR 0036), so portability issues are obvious.

## Consequences

- wasm portability becomes a planned contract instead of a later rewrite.
- core events and payloads can be designed without desktop-only leakage.
- UI/menus/commands remain consistent via shared gating rules (`when`).

## Alternatives Considered

### A) Pure `cfg` compile-time decisions

Pros: simple.

Cons:

- insufficient for web permission gating and backend differences,
- encourages platform-specific code paths scattered across widgets.

### B) Allow widgets to probe platform features ad-hoc

Pros: flexible.

Cons:

- leads to inconsistent UX and hard-to-debug behavior drift,
- breaks the “commands/menus/palette share one gating model” goal.

## Next Steps

Implemented:

1. Added a `PlatformCapabilities` data type at the platform boundary.
2. Threaded it into `InputContext` and `when` evaluation inputs.
3. Updated external DnD contracts to respect the capability matrix (ADR 0053).

Remaining:

1. Add a small debug surface (HUD/inspector) that prints capabilities (ADR 0036).

## References

- ADR 0001: `docs/adr/0001-app-effects.md`
- ADR 0003: `docs/adr/0003-platform-boundary.md`
- ADR 0013: `docs/adr/0013-docking-ops-and-persistence.md`
- ADR 0017: `docs/adr/0017-multi-window-display-and-dpi.md`
- ADR 0022: `docs/adr/0022-when-expressions.md`
- ADR 0175 (execution): `docs/adr/0184-execution-and-concurrency-surface-v1.md`
- ADR 0034: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- ADR 0036: `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- ADR 0041: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- ADR 0053: `docs/adr/0053-external-drag-payload-portability.md`
- Makepad web single entrypoint message pump:
  - `repo-ref/makepad/platform/src/os/web/web.rs`

