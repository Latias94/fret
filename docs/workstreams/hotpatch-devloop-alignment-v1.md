# Hotpatch Devloop Alignment v1 (Dioxus-Style)

Status: Draft (workstream notes only; ADRs remain the source of truth)

Tracking files:

- `docs/workstreams/hotpatch-devloop-alignment-v1-todo.md`
- `docs/workstreams/hotpatch-devloop-alignment-v1-milestones.md`

## 0) Why this workstream exists

Fret already has a dev-only Subsecond integration (ADR 0105) and a conservative runner-level hot reload boundary.
However, the **developer experience is not yet “one obvious path”**:

- multiple flags exist (`--hotpatch`, `--hotpatch-devserver`, `--hotpatch-dx`, file triggers),
- “why did my change not show up?” can be ambiguous,
- Windows has a known view-level hotpatch crash mode,
- and common UI tweaks still depend on Rust hotpatch success.

This workstream aligns Fret’s inner loop with a Dioxus-style posture:

1) a single recommended command,
2) clear status + diagnostics,
3) safe, predictable fallback behavior,
4) and “no-compile” hot reload channels for high-frequency UI tweaks.

## 1) Invariants (do not break)

1. **Dev tooling stays out of kernel crates**
   - No `subsecond`, `tungstenite`, or devserver protocol deps in `fret-core` / `fret-runtime` / `fret-app` / `fret-ui`.
   - All hotpatch integration remains feature-gated and glue-layer owned (ADR 0105 / ADR 0092).

2. **Safety-first hot reload boundary**
   - A patch may invalidate retained closures/callbacks; the default must be “drop registrations and rebuild caches”
     rather than attempting state migration (ADR 0105).

3. **Mechanism vs policy split remains**
   - `crates/fret-ui` remains mechanism-only; hot reload policy lives at the runner/app boundary.

## 2) Definitions

- **Hotpatch (Subsecond)**: applying a JumpTable that changes function targets at runtime.
- **Runner reload boundary**: a conservative reset point where retained UI runtime state is discarded/rebuilt
  (`UiTree` reset, overlay/controller cleanup, etc).
- **Full rebuild + restart**: stop the process, rebuild the binary, launch again (fast supervisor UX is preferred).

## 3) Target UX (what users should do)

### 3.1) One recommended command (native)

Users should default to:

`fretboard dev native --bin <demo> --hotpatch`

Policy:

- If `dx` (dioxus-cli) is available and the target is hotpatch-ready, `fretboard` should run `dx serve --hotpatch`
  and enable end-to-end Subsecond delivery.
- Otherwise, `fretboard` should fall back to “runner reload boundary” mode (file trigger), and clearly state that
  Rust hotpatching is not active.

### 3.2) Advanced/explicit modes remain available

- `--hotpatch-reload`: force “reload boundary only” (never run `dx`)
- `--hotpatch-devserver ws://.../_dioxus`: connect to an external devserver (expert mode)
- `--hotpatch-dx`: explicitly run the dx wrapper (mostly for debugging/compat)

## 4) Gap closure plan (the 4 missing pieces)

### 4.1) End-to-end devloop supervision (DX-like)

Goal: reduce “flag soup” and make the build/patch/run lifecycle observable.

Design:

- `fretboard` is the **supervisor**:
  - chooses mode,
  - sets environment,
  - prints a startup summary,
  - provides clear next actions on failure.
- The runner remains a **consumer**:
  - listens for patch events / marker changes,
  - triggers a hot reload boundary,
  - emits lightweight diag logs.

Deliverables:

- A stable “Hotpatch Summary” printed at startup (mode, ws endpoint, build id, trigger path, view-call strategy).
- A stable location for logs:
  - `.fret/hotpatch_runner.log`
  - `.fret/hotpatch_bootstrap.log`
- Optional (later): a small `fretboard hotpatch status` command that summarizes last-known state by reading those logs
  or a simple `.fret/hotpatch.status.json` file written by the runner.

### 4.2) No-compile hot reload channels (RSX-like)

Goal: most “UI tweak” edits should not depend on Rust hotpatch success.

Channels:

1) **Theme reload**
   - tokens, typography, radii, spacing, shadows
   - default file: `.fret/theme.json` (a `fret_ui::ThemeConfig` JSON)
   - reloadable via a polling watcher and applied at a safe frame boundary

2) **Asset reload**
   - svg/png/fonts; invalidate caches and request redraw
   - default trigger file: `.fret/ui_assets.touch` (bump `UiAssetsReloadEpoch`)
   - intended usage: a tooling watcher updates the trigger file when assets change
   - current scope (L1): path-based image decode (`ImageSource::from_path`) and SVG file bytes (`SvgFileSource`)
   - fonts: `.fret/fonts.json` (list of TTF/OTF/TTC files) is applied via `Effect::TextAddFonts` on change or when `ui_assets.touch` bumps

3) **Hot literals**
   - developer strings/labels/tooltips sourced from a data file in `.fret/` or the app root
   - default file: `.fret/literals.json` (string→string map; example key: `demo.headline`)

Enablement (native):

- Enabled automatically in dev/hotpatch contexts (`FRET_HOTPATCH=1`, `DIOXUS_CLI_ENABLED=1`) or explicitly via `FRET_DEV_RELOAD=1`.
- Poll interval: `FRET_DEV_RELOAD_POLL_MS` (default: `250`).
- Path overrides:
  - `FRET_DEV_RELOAD_THEME_PATH`
  - `FRET_DEV_RELOAD_LITERALS_PATH`
  - `FRET_DEV_RELOAD_UI_ASSETS_TRIGGER_PATH`

Principle:

- These channels are orthogonal to Subsecond: they should work in “reload boundary only” mode.

### 4.3) Predictability + fallback (especially Windows)

Goal: eliminate “sometimes it updates, sometimes it crashes” behavior.

Design:

- Define a single “fallback ladder”:
  1) apply patch → request runner reload boundary
  2) if view-level hotpatch is known-unsafe (platform/config) → warn and fall back to boundary-only behavior
  3) if the process crashes repeatedly → `fretboard` suggests/optionally performs fast restart

Windows known issue (ADR 0105):

- Make the “direct view call disables view-level hotpatching” state explicit in logs and in the startup summary.
- Prefer a **supervised restart** UX over asking users to memorize env vars.

### 4.4) Productized scope + guidance (what’s supported)

Goal: make “what hotpatch can/can’t do” obvious.

Deliverables:

- Hotpatch-ready authoring guidance:
  - prefer `UiAppDriver` / `FnDriver`,
  - avoid captured closures as stable entrypoints,
  - keep view entrypoints stable and reachable from the tip crate where possible,
  - expect hard resets in dev mode.
- A short troubleshooting guide:
  - “patch applied but UI didn’t change”
  - “connected to devserver but no JumpTables”
  - “aslr_reference=0”
  - “Windows patched view crash”

## 5) Evidence & regression gates

We want a small, stable validation loop:

- a dedicated “hotpatch smoke” demo that is designed to patch safely,
- a scripted interaction that confirms:
  - patches are applied,
  - reload boundary runs,
  - view code actually changes output.

Potential gates:

- `fretboard dev native --bin hotpatch_smoke_demo --hotpatch` (manual smoke)
- future: a `fretboard diag` script that drives the smoke demo and checks a log or screenshot delta.

## 6) References

- ADR 0105: `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- Dev tooling posture: `docs/adr/0106-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- Dioxus hot reload overview: `repo-ref/dioxus/notes/architecture/07-HOTRELOAD.md`
