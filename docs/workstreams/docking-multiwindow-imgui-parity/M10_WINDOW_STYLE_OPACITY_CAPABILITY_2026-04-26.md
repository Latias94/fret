# M10 Window Style Opacity Capability - 2026-04-26

Status: accepted source-level closure for `DW-P2-style-001`

Related:

- `docs/adr/0139-window-styles-and-utility-windows.md`
- `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- `crates/fret-runtime/src/capabilities/keys.rs`
- `crates/fret-runtime/src/runner_window_style_diagnostics.rs`
- `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs`
- `crates/fret-launch/src/runner/desktop/runner/effects.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`

## Problem

The DockFloating style request surface had already moved beyond the old `DW-P2-style-001` wording:
portable style requests, z-level capability quality, taskbar/activation policy, transparent
payload hit-testing, background materials, and effective style diagnostics all exist. The remaining
runner-owned gap in the docking parity lane was narrower:

- `WindowStyleRequest::opacity` was a real patch facet used by the ImGui-style transparent moving
  payload path,
- but there was no `ui.window.opacity` capability key,
- unsupported runners could still receive the OS opacity attempt,
- and diagnostics could not assert effective opacity.

That made opacity different from the other style facets in ADR 0139: it was best-effort in code, but
not capability-gated or observable as an effective style outcome.

## Fix

The source surface now treats global window opacity like the other utility-window style facets:

- `PlatformCapabilities.ui.window_opacity` and the `ui.window.opacity` key advertise availability.
- Desktop backends only advertise opacity on Windows/macOS, matching the implemented runner hooks.
- Web and mobile clamp opacity off.
- Create-time and runtime `WindowStyleRequest::opacity` application is gated by capability.
- `RunnerWindowStyleDiagnosticsStore` records effective opacity and keeps unsupported requests at
  the opaque default (`WindowOpacity(255)`).
- Diagnostics `window_style_effective_is` predicates can assert `opacity_alpha_u8`.

This keeps the v1 DockFloating style subset honest without widening `fret-ui` or requiring native
handle access. Remaining backend-specific style behavior stays under ADR 0139 / ADR 0313 rather
than reopening the docking parity lane.

## Gates

```text
cargo nextest run -p fret-runtime -E 'test(opacity_request_degrades_when_unsupported) or test(opacity_request_records_when_supported) or test(capability_key_kind_matches_platform_capabilities_accessors)' --no-fail-fast --jobs 2
cargo nextest run -p fret-diag-protocol --test script_json_roundtrip script_v1_roundtrip_window_style_effective_hit_test --no-fail-fast --jobs 2
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics -E 'test(window_style_effective_matches_opacity_alpha)' --no-fail-fast --jobs 2
cargo check -p fret-launch -p fret-bootstrap --features fret-bootstrap/ui-app-driver,fret-bootstrap/diagnostics --jobs 2
```

## Decision

Mark `DW-P2-style-001` done for the current v1 DockFloating style subset. The remaining open item in
this lane is still the Wayland compositor manual acceptance run; it cannot be converted into a
Windows-host source gate without weakening the evidence.
