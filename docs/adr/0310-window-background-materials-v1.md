# ADR 0310: Window Background Materials v1 (Mica / Acrylic / Vibrancy) — Capability-Gated

Status: Accepted

## Context

ADR 0139 defines a portable `WindowStyleRequest` vocabulary for utility OS windows (frameless,
transparent, always-on-top, skip-taskbar, etc).

For launcher/HUD-style apps, “transparent” alone is not enough: users expect OS-provided materials
such as Windows 11 Mica/Acrylic or macOS Vibrancy, with deterministic degradation and capability
gating.

We want a minimal, portable way to request these materials without:

- leaking backend/window-handle types into portable crates,
- forcing ecosystem crates to fork on `cfg(target_os = ...)`,
- conflating **in-window** backdrop filters (renderer effects) with **OS window background** materials.

Terminology:

- **In-window backdrop effects**: GPU effects applied to content rendered *inside* the window (ADR 0116/0117).
- **Window background materials**: compositor/OS-backed backdrop behind the app’s content (desktop blur/material).

## Goals

1. Define a small, stable vocabulary for **OS window background materials**.
2. Make requests capability-gated and degradable at runtime (ADR 0054).
3. Keep the contract portable and backend-independent.
4. Require observability: callers can inspect the effective/clamped result (ADR 0036).

## Non-goals

- Guarantee that material requests are honored on all platforms/backends.
- Provide per-pixel shaped windows or per-pixel hit testing (still out of scope; see ADR 0139).
- Define exact pixel-level parity across OS materials (tints/blur radii vary by OS version/theme).
- Replace in-window GPU effects (this ADR does not change renderer effect semantics).

## Decision

### 0) Orthogonal facets: decorations, composited alpha surface, OS backdrop material, content background

This ADR intentionally treats window styling as **orthogonal facets** so behavior is portable and
debuggable without platform forks:

- **Decorations** (`decorations`): whether the platform draws a title bar / border chrome.
- **Composited alpha surface** (`transparent`): whether the OS window surface participates in alpha
  composition (create-time; may be sticky for the window lifetime).
- **OS backdrop material** (`background_material`): OS/compositor-provided backdrop behind the app's
  content (Mica/Acrylic/Vibrancy).
- **Content background** (not a window-style facet): whether the app paints an opaque background in
  its root view.

Important clarifications (normative):

- `background_material=None` means **"do not request an OS backdrop material"**. It does not imply
  anything about decorations or whether the window surface is composited.
- `transparent=true` means the **surface is composited**. It does not guarantee that the desktop is
  visible through the app. Visibility depends on renderer clears and whether the app paints an
  opaque root background.

### 1) Extend `WindowStyleRequest` with an optional background material facet

Extend the portable `WindowStyleRequest` (ADR 0139) with:

- `background_material: Option<WindowBackgroundMaterialRequest>`

This is a patchable facet (best-effort) and may be used both at create-time and via
`WindowRequest::SetStyle`.

### 2) Define `WindowBackgroundMaterialRequest` v1

`WindowBackgroundMaterialRequest` is intentionally small and enumerates *intent*, not implementation:

- `None` (default): normal opaque app window background behavior.
- `SystemDefault`: request the platform’s default “utility window” material, if any.
- `Mica`: request Windows 11-style Mica (best-effort; Windows-only in practice).
- `Acrylic`: request acrylic/blurred translucent backdrop (best-effort).
- `Vibrancy`: request macOS vibrancy-style backdrop (best-effort; macOS-only in practice).

Notes:

- These variants are capability-gated. Unsupported variants are clamped to `None`.
- This vocabulary is intentionally conservative; richer material parameterization belongs in a
  follow-up ADR once we have evidence from multiple platforms.

### 3) Interaction with `transparent` and rendering

Window background materials generally require compositor-backed alpha composition. Therefore:

- If `background_material` is `Some(...)` and the backend requires an alpha/composited window, the
  runner may implicitly treat `transparent` as `true` for the purposes of the **composited surface**
  (effective style).
- Runners may keep this implied composited transparency **sticky** for the lifetime of the window,
  even if the background material is later set back to `None`, because some backends treat window
  transparency as a create-time-only attribute.
- If the backend cannot provide a composited alpha surface, it must clamp the request to
  `background_material=None` and report the effective result via diagnostics.

Renderer contract (normative, high-level):

- When the effective style has an OS backdrop material enabled, the runner+renderer must avoid
  “accidentally opaque” clears. The default clear should preserve alpha (clear alpha = 0) unless
  the app draws an opaque root background.
- When the window surface is composited **only** due to a prior implied material request and the
  effective material is later `None`, runners may choose an **opaque default clear** (clear alpha =
  1) so the default appearance matches a typical opaque window. Apps that want visual transparency
  without OS materials must request `transparent=true` explicitly and paint a transparent root.

This ADR intentionally does not prescribe the exact renderer knobs; it only requires that the
window-level transparency/material intent can be honored without leaking backend handles.

### 4) Capability keys (ADR 0054)

Add capability keys to gate requests in portable `when` expressions:

- `ui.window.transparent` (already defined by ADR 0139)
- `ui.window.background_material.system_default`
- `ui.window.background_material.mica`
- `ui.window.background_material.acrylic`
- `ui.window.background_material.vibrancy`

Runners/backends must:

1. Advertise available keys at startup.
2. Clamp requests to available keys.
3. Expose effective/clamped style to diagnostics/inspection (ADR 0036, ADR 0139).

### 5) Observability

To make “why didn’t I get Mica?” debuggable, runners must expose per-window information that can
answer:

- What was requested?
- What was applied?
- Which capability keys were available?

The mechanism is runner-defined (log line, inspector entry, diagnostics store), but the information
must be available without native handle access.

Additional observability requirements (normative):

- Runners must distinguish between:
  - the **composited surface** decision (create-time; sticky),
  - the **effective backdrop material** decision (runtime-clamped),
  - the **default visual transparency** decision (whether the renderer preserves alpha by default).
- Runners must be able to explain *why* the surface is composited (explicit vs implied-by-material
  vs sticky create-time).

### 6) Expected combinations (guide for callers and diagnostics)

The following table describes the intended relationship between facets. It is informational, but
serves as a review checklist for runner implementations.

| Request intent | Composited surface (effective) | Backdrop material (effective) | Default clear alpha |
| --- | --- | --- | --- |
| `transparent=false`, `background_material=Mica/Acrylic/Vibrancy` | false | `None` (degraded) | 1 |
| `transparent` omitted, `background_material=Mica/Acrylic/Vibrancy` | true (implied; sticky) | requested (if supported) | 0 |
| `transparent` omitted, later set `background_material=None` | true (sticky) | `None` | 1 (runner may choose opaque default) |
| `transparent=true`, `background_material=None` | true (explicit) | `None` | 0 (caller-owned; app must paint) |

## Consequences

Pros:

- Locks a portable vocabulary early (hard-to-change UX surface).
- Keeps OS-specific material branching out of ecosystem crates.
- Makes degradation explicit and testable via capabilities + diagnostics.

Cons:

- Still best-effort: OS materials are platform/version/theme dependent.
- Requires runner+renderer coordination for transparent composition to avoid “opaque clears”.

## Alternatives considered

1) “Just use in-window GPU acrylic everywhere”
- Rejected: in-window effects cannot reproduce OS desktop/backdrop materials behind the window.

2) Expose native window handles and let apps call OS APIs directly
- Rejected: violates the portable contract boundary (ADR 0090) and forces platform forks.

3) Encode material requests as opaque strings
- Rejected: loses reviewability and makes capability gating harder to standardize.

## Implementation notes (non-normative)

Likely touch points:

- Portable contract:
  - `crates/fret-runtime/src/effect.rs` (`WindowStyleRequest`)
  - `crates/fret-runtime/src/capabilities/keys.rs` (new keys)
  - `crates/fret-runtime/src/capabilities/platform.rs` (key mapping)
- Desktop runner (winit + platform glue):
  - `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` (apply at create-time)
  - `crates/fret-launch/src/runner/desktop/runner/effects.rs` (apply via `SetStyle`)
- Renderer/surface:
  - ensure the clear/alpha mode selection is compatible with composited windows when requested.

Recommended validation:

- A desktop demo (or scripted diag) that opens a frameless transparent always-on-top window and
  reports the effective clamped material + transparency in diagnostics (ADR 0036).

## References

- Utility window styles: `docs/adr/0139-window-styles-and-utility-windows.md`
- Capability gating: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Renderer effect substrate (in-window backdrop filters): `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`,
  `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
