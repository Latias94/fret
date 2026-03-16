# Hotpatch Devloop Alignment v1 — Refactor Design Notes

Status: Draft (design notes; ADRs remain the source of truth)

This workstream targets **L1 only**: a Dioxus-style inner loop that is predictable, observable, and safe, with a small set of “no-compile” hot reload channels for high-frequency UI tweaks.

Non-goals (explicitly out of scope for v1):

- “Bevy/ZST-level” pervasive hotpatching across the full workspace
- automatic state migration across ABI breaks
- patching structural type/layout changes (expect rebuild/restart instead)

## 0) Mental model: Detect vs Apply vs Observe

We treat devloop as three independent subsystems:

1) **Detect**: notice that something changed (Rust code, theme/assets/literals, etc.)
2) **Apply**: safely incorporate the change (patch, invalidate caches, trigger redraw)
3) **Observe**: make the current state and last change visible (summary/status/logs)

Key principle: **Apply should be stable and reusable** even if we later swap out Detect (polling → notify, `dx serve` → custom devserver, etc.).

## 1) Apply surface (L1)

### 1.1) Rust code hotpatch (Subsecond)

Mechanics:

- **Integration points** are explicit call sites that go through `subsecond::HotFn`.
- Subsecond updates a global JumpTable; runtime call targets change without rewriting the original executable.

Fret apply boundaries (native):

- Runner boundary: after a patch is observed/applied, schedule a conservative “reload boundary” that discards retained UI registrations/caches.
- View boundary: view invocation can be either:
  - `hotfn` (view-level hotpatching enabled): `subsecond::HotFn::current(driver.view).call(..)`
  - `direct` (view-level hotpatching disabled): call the function pointer directly and rely on reload-boundary behavior.

Evidence anchors:

- `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (view call strategy + HotFn call site)
- `crates/fret-launch/src/runner/common/fn_driver.rs` (HotFn call sites for driver callbacks)
- `crates/fret-launch/src/runner/desktop/runner/mod.rs` (reload boundary scheduling)

### 1.2) No-compile hot reload channels

These channels must work even when Rust hotpatch is not active (reload-boundary-only mode).

- Theme: `.fret/theme.json` → `Theme::apply_config`
- Hot literals: `.fret/literals.json` → global `HotLiterals`
- Assets: `.fret/asset_reload.touch` bumps the shared `AssetReloadEpoch` to invalidate path-based asset caches
  - file-path images: `ImageSource::from_file_path` cache key includes the epoch
  - SVG files: `SvgFileSource` file bytes cache respects the epoch
- Fonts: `.fret/fonts.json` (TTF/OTF/TTC list) applied via `Effect::TextAddFonts`

Evidence anchors:

- `ecosystem/fret-bootstrap/src/dev_reload.rs` (polling watcher + apply)
- `crates/fret-runtime/src/asset_reload.rs` (`AssetReloadEpoch`)
- `ecosystem/fret-ui-assets/src/image_source.rs` (epoch in cache key)
- `ecosystem/fret-ui-assets/src/svg_file.rs` (epoch-gated file bytes cache)
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` (`Effect::TextAddFonts`)

## 2) Detect surface (L1)

We intentionally keep Detect “swappable”:

- Rust hotpatch:
  - preferred: `dx serve --hotpatch` (supervised by `fretboard`)
  - fallback: external devserver (`--hotpatch-devserver`) or reload-boundary-only mode
- No-compile channels:
  - polling watcher in `fret-bootstrap` (`FRET_DEV_RELOAD_POLL_MS`, default 250ms)

Evidence anchors:

- `apps/fretboard/src/dev.rs` (mode selection and summary)
- `ecosystem/fret-bootstrap/src/dev_reload.rs` (polling watcher)

## 3) Observe surface (L1)

We standardize “what happened” and “where to look”:

- Stable log files:
  - `.fret/hotpatch_runner.log`
  - `.fret/hotpatch_bootstrap.log`
- Status command: `fretboard hotpatch status --tail N`

Evidence anchors:

- `apps/fretboard/src/hotpatch.rs`
- `apps/fretboard/src/dev.rs`

## 4) Platform posture: Windows safety default

Windows has a known view-level hotpatch crash mode (ADR 0105). L1 posture:

- default to `direct` view call strategy on Windows when hotpatch is enabled
- make this explicit in the startup summary/logs
- allow override for experiments via `FRET_HOTPATCH_VIEW_CALL_STRATEGY=hotfn`

Evidence anchors:

- `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`

## 5) Developer guidance (L1)

Hotpatch-ready authoring is about keeping integration points stable:

- prefer a small number of stable entrypoints (view root + driver callbacks)
- assume “reload boundary” resets retained registrations/caches
- treat structural/layout-breaking Rust changes as rebuild/restart events
