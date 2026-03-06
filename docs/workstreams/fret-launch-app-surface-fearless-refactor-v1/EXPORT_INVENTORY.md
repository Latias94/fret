# Fret Launch + App Surface (Fearless Refactor v1) 鈥?Export Inventory

This note captures the Stage 1 export inventory for `crates/fret-launch`.

Goals:

- classify the current root surface before hiding or removing anything,
- make downstream coupling visible,
- turn the launch-surface discussion into a reviewable checklist instead of intuition.

## Snapshot summary

### Main findings

1. `fret-launch` currently exposes a mix of:
   - stable launch entry points,
   - advanced-but-valid integration seams,
   - compatibility-only paths,
   - and implementation-heavy surfaces that should not expand further.
2. `ecosystem/fret` directly depends on a small subset of launch types:
   - `RunnerError`
   - `WindowCreateSpec`
   - `EngineFrameUpdate`
   - `WinitRunnerConfig`
   - `WinitAppDriver`
   - dev-state exports
   - `ViewportRenderTarget`
3. In this worktree, `crates/fret-framework::launch` re-exports a curated subset of the core launch contract instead of mirroring the full `fret_launch::*` root surface.
4. In-tree `apps/` callers no longer need `fret_launch::runner::*`; core launch entry points stay at crate root while specialized interop/media helpers now live under dedicated public submodules.
5. In this worktree, `pub mod runner` has been removed from the public root surface; launch consumers now go through curated crate-root exports plus explicit specialized modules.

### Current recommendation

- App authors should prefer `fret`.
- Manual framework assemblers should use `fret-framework`.
- Advanced launch/integration users may depend on `fret-launch`, using crate-root imports for core launch contracts and dedicated submodules for specialized interop/media helpers.
- Advanced driver recommendation: prefer `FnDriver`; treat `WinitAppDriver` as compatibility surface.

## Root export classification (`crates/fret-launch/src/lib.rs`)

### Categories

- **Stable public contract**: should remain usable as part of the intended long-lived public story.
- **Stable specialized contract**: valid advanced/integration surface, but not part of the first-hour app-author path.
- **Transitional public surface**: currently public and useful, but should not grow or become the default recommendation.
- **Compatibility-only**: keep working for now, but steer users away in docs/examples.

| Public export(s) | Classification | Why |
| --- | --- | --- |
| `RunnerError` | Stable public contract | Clear error boundary for launch/bootstrap callers. |
| `configure_stacksafe_from_env` | Removed from root surface | Kept as crate-internal bootstrap plumbing used by the native run path. |
| `dev_state` module + `DevState*` exports | Transitional public surface | Explicitly dev-only and feature-gated; should remain available without being treated as core runtime contract. |
| `runner` module | Removed from root surface | The module remains internal implementation plumbing and is no longer a public import path. |
| `FnDriver`, `FnDriverHooks` | Stable public contract | Best match for the repo's hotpatch-friendly advanced driver posture. |
| `EngineFrameKeepalive`, `EngineFrameUpdate` | Stable specialized contract | Used by advanced engine-frame and interop paths, including `ecosystem/fret`. |
| `WgpuInit` | Stable specialized contract | Required for host-provided GPU context / factory integration. |
| `WindowCreateSpec`, `WindowLogicalSize`, `WindowPhysicalPosition`, `WindowPosition` | Stable public contract | Core window-creation and geometry contract for advanced/custom launch flows. |
| `WinitCommandContext`, `WinitEventContext`, `WinitGlobalContext`, `WinitHotReloadContext`, `WinitRenderContext`, `WinitWindowContext` | Stable specialized contract | Advanced driver-hook contexts; valid long-lived API for integrators. |
| `WinitRunnerConfig` | Stable public contract (with curation debt) | Widely needed and currently central, but too broad as a single long-term config surface. |
| `run_app`, `run_app_with_event_loop`, `WinitAppBuilder` | Stable public contract | Native launch entry points that advanced callers can reasonably depend on. |
| `WinitRunner` | Removed from root surface | Internal implementation type; not part of the curated public launch contract. |
| `WinitAppDriver` | Compatibility-only | Still heavily used in-tree, but the documented direction is to prefer `FnDriver`. |
| `RunnerUserEvent` | Removed from root surface | Internal runner event type; not part of the intended public launch surface. |
| `ViewportRenderTarget`, `ViewportRenderTargetWithDepth`, `RenderTargetUpdate` | Stable specialized contract | Advanced embedded viewport / offscreen render-target seams that are still general enough to stay near the root launch contract. |
| `imported_viewport_target::{ImportedViewport*, NativeExternalImportOutcome}` | Stable specialized contract | Imported viewport helpers remain public, but now live under an explicit interop-oriented namespace instead of competing with the crate-root story. |
| `native_external_import::{NativeExternalImportError, NativeExternalImportedFrame, NativeExternalTextureFrame, OwnedWgpuTextureFrame}` | Stable specialized contract | Native external texture import contracts remain valid advanced seams while becoming easier to classify. |
| `ViewportOverlay3dHooks*`, `install_viewport_overlay_3d_immediate`, `record_viewport_overlay_3d`, `upload_viewport_overlay_3d_immediate` | Transitional public surface | Real advanced features, but still niche and implementation-sensitive. |
| `media::{windows_mf_video, apple_avfoundation_video, android_mediacodec_video}` | Stable specialized contract | Platform-specific integration helpers; not beginner surface, but legitimate advanced launch helpers now grouped under a dedicated module. |
| `shared_allocation::{SharedAllocationExportError, dx12}` | Transitional specialized contract | Mostly meaningful together with shared-allocation interop helpers; keep available without treating it as a broad top-level abstraction. |
| `WebRunnerHandle`, `run_app_with_handle` | Stable specialized contract | Web-specific launch/interop entry points. |
| `run_app_with_event_loop_and_handle` | Removed from root surface | Unused in-tree root export; retained as internal/runner-level implementation entry. |

## Downstream coupling inventory

### `ecosystem/fret` direct dependencies on `fret-launch`

Current direct references in `ecosystem/fret/src`:

- `fret_launch::RunnerError`
- `fret_launch::WindowCreateSpec`
- `fret_launch::EngineFrameUpdate`
- `fret_launch::WinitRunnerConfig`
- `fret_launch::WinitAppDriver`
- `fret_launch::dev_state::*`
- `fret_launch::ViewportRenderTarget`

Implication:

- `fret` currently depends on both the stable launch-entry layer and the compatibility-era driver trait.
- We cannot remove `WinitAppDriver` from public view until `fret-bootstrap` / `fret` stop requiring it generically.

### `crates/fret-framework` exposure

Current posture:

- `crates/fret-framework/src/lib.rs` exposes a curated `launch` module behind `feature = "launch"`.
- The facade includes driver/core-context/config/app-entry types such as `FnDriver`, `WinitAppDriver`, `WinitRunnerConfig`, `WindowCreateSpec`, `WgpuInit`, and top-level `run_app*` / `WinitAppBuilder` entry wiring.
- Specialized media / interop / imported-viewport helpers remain available from explicit `fret-launch` submodules rather than the compact framework facade.

Implication:

- accidental root-export growth in `fret-launch` no longer automatically becomes part of the manual-assembly facade,
- `fret-framework` can stay a compact umbrella for common advanced assembly,
- callers that need specialized launch integration can still opt into explicit `fret-launch` submodules directly.

## Immediate conclusions

### What we can do now with low risk

1. Keep `fret-framework::launch` limited to the curated core contract unless a new export has explicit facade-level justification.
2. Keep new core launch-facing code on curated crate-root imports only.
3. Add new specialized helpers under explicit public submodules instead of widening the root contract.
4. Stop teaching `WinitAppDriver` as the first recommendation in new docs/examples.
5. Avoid adding new public exports unless they are explicitly classified first.

### What is not yet safe to do

1. Remove more implementation-shaped root exports once in-tree callers are proven absent.
2. Remove `WinitAppDriver` from the public surface.
3. Treat `WinitRunnerConfig` as if its current shape were already the final public config story.

## Recommended next cuts

### Cut 1 鈥?Keep shrinking in-tree `runner::*` callers

Target:

- make `runner::*` primarily an external compatibility path rather than an actively growing in-tree dependency.

### Cut 2 鈥?Document `FnDriver` as the advanced recommendation everywhere new

Target:

- examples,
- docs,
- workstream notes,
- bootstrap-facing guidance.

### Cut 3 鈥?Classify `WinitRunnerConfig` by subdomain

Target groups:

- app/window defaults,
- render/backend tuning,
- streaming/media tuning,
- web/platform specifics.

This can be done as documentation first, before any breaking config refactor.

## Evidence anchors

- Launch root surface: `crates/fret-launch/src/lib.rs`
- Launch ownership/readme: `crates/fret-launch/README.md`
- Driver direction note: `crates/fret-launch/src/runner/common/winit_app_driver.rs`
- `fret` direct launch references: `ecosystem/fret/src/lib.rs`
- `fret` interop reference: `ecosystem/fret/src/interop/embedded_viewport.rs`
- Framework launch exposure: `crates/fret-framework/src/lib.rs`
