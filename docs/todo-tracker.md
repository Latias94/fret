# Fret TODO Tracker (Review Findings)

This document tracks actionable TODOs discovered during architecture/doc/code review.
It complements (but does not replace) ADRs:

- ADRs define contracts and boundaries.
- This file lists concrete follow-up work, grouped by subsystem, and links back to the relevant ADRs.

## How to use

- Prefer turning P0 items into `Accepted` ADR decisions or conformance tests before adding new feature surface area.
- When an item is resolved, either delete it or move it into `docs/known-issues.md` (if it becomes a long-lived limitation).
- Deep-dive gap/backlog notes live under `docs/archive/backlog/` to keep `docs/` entrypoints small.

## P0 - IME / Text Input

- **Preedit-first key arbitration end-to-end (runner + routing)**
  - Problem: composing IME sessions must not lose `Tab/Space/Enter/NumpadEnter/Escape/Arrows/Backspace/...` to focus traversal or global shortcuts.
  - ADRs: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
  - Code: `crates/fret-launch/src/runner/mod.rs`, `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/text_input/mod.rs`, `crates/fret-ui/src/text_area/mod.rs`
  - Current: `crates/fret-ui/src/tree/mod.rs` defers shortcut matching for reserved keys and only falls back if the widget does not `stop_propagation`. `crates/fret-ui/src/text_input/mod.rs` and `crates/fret-ui/src/text_area/mod.rs` stop propagation for these keys while IME is composing (treat "composing" as `preedit` non-empty **or** preedit cursor metadata present).
  - Current: regression tests exist for:
    - composing: reserved keys suppress traversal/shortcuts (`tab_focus_next_is_suppressed_during_ime_composition`, `reserved_shortcuts_are_suppressed_during_ime_composition`);
    - not composing: `Tab` focus traversal works (`tab_focus_next_runs_when_text_input_not_composing`).

- **Define and validate blur/disable semantics for IME enablement**
  - Problem: ensure loss of focus reliably disables IME where appropriate, and avoid per-widget effect spam.
  - ADRs: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0020-focus-and-command-routing.md`
  - Code: `crates/fret-ui/src/tree/mod.rs`
  - Current: `UiTree` owns `Effect::ImeAllow` and updates it on focus changes and at paint time; widgets only emit `Effect::ImeSetCursorArea` when the caret rect changes.

- **Multiline IME contract + conformance harness**
  - Goal: lock and validate multiline selection/composition/caret-rect behavior (scroll/wrap/DPI/preedit updates).
  - ADRs: `docs/adr/0071-text-input-multiline-composition-contract.md`, `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`, `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
  - Code: `crates/fret-ui/src/text_area/mod.rs`, `crates/fret-render/src/text.rs`

## P0 - Fonts / Fallbacks / Missing Glyphs

- **Make the default font semantic (system UI font alias)**
  - Problem: relying on `FontId::default()` without a defined font family causes platform-dependent tofu and IME provisional-state breakage.
  - ADRs: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0006-text-system.md`, `docs/adr/0162-font-stack-bootstrap-and-textfontstackkey-v1.md`
  - Code: `crates/fret-ui/src/theme.rs`, `crates/fret-render/src/text.rs`
  - Current: `crates/fret-render/src/text.rs` configures both `cosmic-text` fontdb generics and Parley/fontique generic families (keep backend behavior aligned as we converge the font source of truth).
  - Current: `TextStyle.font` is a semantic `FontId` (`Ui/Serif/Monospace/Family(name)`) and maps to generic stacks (`sans-serif`/`serif`/`monospace`) for shaping.
  - TODO: expose a curated default font stack at the theme/settings layer (and decide how user font loading maps to stable `FontId` values).

- **Web/WASM bootstrap fonts are insufficient** (done)
  - Problem: `fret-fonts` currently bundles a mono subset only; general UI text needs a UI sans baseline (and eventually emoji).
  - ADRs: `docs/adr/0162-font-stack-bootstrap-and-textfontstackkey-v1.md`
  - Code: `crates/fret-fonts/src/lib.rs`, `crates/fret-launch/src/runner/web.rs`
  - Current: `fret-fonts` bundles a UI sans + monospace baseline for wasm (`Inter` + `JetBrains Mono` subsets).
  - Current: optional `emoji` font bundle is available (`Noto Color Emoji`), gated behind `fret-fonts/emoji`.
  - Current: optional `cjk-lite` font bundle is available (`Noto Sans CJK SC`), gated behind `fret-fonts/cjk-lite`.
  - Current: web runner seeds `TextFontFamilyConfig` from curated defaults when empty, and bumps `TextFontStackKey` via `apply_font_catalog_update` after font injection.

- **Fallback list participates in `TextBlobId` caching / invalidation**
  - Problem: changing configured fallbacks or font DB state must invalidate cached shaping/rasterization results.
  - ADRs: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0162-font-stack-bootstrap-and-textfontstackkey-v1.md`
  - Code: `crates/fret-render/src/text.rs`
  - Current: `crates/fret-render/src/text.rs` includes a `font_stack_key` (derived from locale + configured generic families + fallback policy) in the `TextBlobKey` cache key.
  - Current: runner font/config mutations go through `fret_runtime::apply_font_catalog_update`, which bumps `TextFontStackKey` to prevent stale layout/raster cache reuse.

- **Emoji / variation selectors policy**
  - Goal: define baseline behavior for emoji fonts and variation selectors, and add a smoke test string that exercises it.
  - ADRs: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0167-polychrome-glyphs-and-emoji-pipeline-v1.md`
  - Code: `crates/fret-render/src/text.rs`
  - Current: optional wasm emoji font bundle (`fret-fonts/emoji` -> `Noto Color Emoji`) and a dedicated conformance demo (`apps/fret-examples/src/emoji_conformance_demo.rs`).
  - Current: automated conformance (unit) covers VS16/ZWJ/flags/keycaps (`crates/fret-render/src/text.rs`).

- **Center baseline within the line box across font swaps**
  - Symptom: switching the UI font in `fret-demo` to fonts with unusual metrics (e.g. Nerd Fonts like "Agave NF") can make text look slightly "up/right" in controls that visually expect centered labels.
  - Root cause: baseline placement derived from ascent only (no distribution of extra line-height padding), plus glyph bitmap bearings can shift perceived ink position vs logical advance metrics.
  - Current: baseline offset is centered within the line box when `line_height > ascent+descent` (see `crates/fret-render/src/text.rs`).
  - Decision: align with the web/shadcn mental model (layout uses the line box + baseline). Do **not** implement default "optical alignment" (ink-bounds-based centering) to compensate for extreme font bearings.
  - Note: some "weird metrics" fonts may still look slightly off-center horizontally. Treat this as expected behavior under the web-aligned model unless we add an explicit per-component opt-in.
  - Option: add an **opt-in** "optical centering" mode for single-line control labels (compute ink bounds per shaped run and apply a small offset at paint time; cache the bounds in the prepared text blob).
  - TODO: add a deterministic regression harness in `apps/fret-examples/src/components_gallery.rs` that toggles a known-problem font and captures a centered-label alignment snapshot (baseline centering regressions only).

## P1 - Text System v2 (Parley / Attributed Spans)

- **Unify rich text under attributed spans (shaping vs paint split)**
  - Goal: make Markdown/code highlighting structurally compatible with wrapping and geometry queries without “many Text nodes”.
  - ADRs: `docs/adr/0157-text-system-v2-parley-attributed-spans-and-quality-baseline.md`
  - Workstream: `docs/workstreams/text-system-v2-parley.md`

- **Stop theme-only changes from forcing reshaping/re-wrapping**
  - Problem: current v1 `RichText` run colors participate in shaping/layout cache keys, so recolors can trigger expensive rework.
  - ADRs: `docs/adr/0157-text-system-v2-parley-attributed-spans-and-quality-baseline.md`, `docs/adr/0109-rich-text-runs-and-text-quality-v2.md`
  - Workstream: `docs/workstreams/text-system-v2-parley.md`

- **Wrapper-owned wrapping and truncation (not backend-owned)**
  - Goal: keep wrapping/ellipsis policy stable and testable across backends and platforms.
  - Reference: Zed/GPUI `LineWrapper` (`repo-ref/zed/crates/gpui/src/text_system/line_wrapper.rs`)
  - Workstream: `docs/workstreams/text-system-v2-parley.md`

- **Text quality baseline: gamma/contrast tuning + subpixel coherence**
  - Problem: current text shaders apply raw atlas coverage without contrast/gamma correction, which can look "soft" under DPI scaling and on light-on-dark surfaces.
  - Problem: subpixel glyph variants are selected during shaping using local glyph positions, but final device-pixel placement also depends on the element origin/transform; a mismatch can cause jitter/blur when scrolling or when origins land on fractional device pixels.
  - ADRs: `docs/adr/0157-text-system-v2-parley-attributed-spans-and-quality-baseline.md`, `docs/adr/0109-rich-text-runs-and-text-quality-v2.md`
  - References:
    - Zed blade shader gamma/contrast helpers: `repo-ref/zed/crates/gpui/src/platform/blade/shaders.wgsl`
    - Zed subpixel variant constants: `repo-ref/zed/crates/gpui/src/text_system.rs`
  - Code (current Fret implementation):
    - Atlas sampler is filtering: `crates/fret-render/src/text.rs`
    - Text shaders: `crates/fret-render/src/renderer/shaders.rs` (`TEXT_SHADER`, `TEXT_SUBPIXEL_SHADER`)
    - Text draw origin uses `origin * scale_factor`: `crates/fret-render/src/renderer/render_scene/encode/draw/text.rs`
  - TODO:
    - Add an explicit "text rendering parameters" uniform (gamma ratios + contrast knobs) and apply it in text fragment shaders (mask + subpixel).
    - Decide and implement a single rule for subpixel variant selection: either snap device-pixel origins (translation-only) or choose variants using the final device-pixel fractional offset at encode time (with a safe fallback under non-translation transforms).
    - Make hinting and subpixel mode policy explicit/configurable (per-platform defaults + conformance strings).

- **Budgeted, evictable glyph atlases**
  - Problem: append-only atlas growth is a long-session risk; eviction must be deterministic and observable.
  - ADRs: `docs/adr/0157-text-system-v2-parley-attributed-spans-and-quality-baseline.md`, `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
  - Workstream: `docs/workstreams/text-system-v2-parley.md`

## P0 - Themes / Token Consistency / shadcn Alignment

- **Enforce token-only reads in shadcn-aligned surfaces**
  - Problem: theme drift occurs when some components read typed fields (`theme.colors.*` / `theme.metrics.*`) while others read semantic tokens (`theme.color_by_key("border")`).

## P1 - Menubar / Commands / Keymap UX

- **Zed-aligned shortcut display policy for menus** (done)
  - Problem: shortcut labels in menus should be stable and understandable; they should not flicker based on live focus, and should remain consistent with command palette display.
  - ADRs: `docs/adr/0183-os-menubar-effect-setmenubar.md`, `docs/adr/0023-command-metadata-and-menus.md`, `docs/adr/0021-keymap-file-format-and-merge.md`, `docs/adr/0022-when-expressions.md`
  - Workstream: `docs/workstreams/os-menubar.md` (MVP 2)
  - Reference: Zed/GPUI `repo-ref/zed/crates/gpui/src/platform/mac/platform.rs` (`bindings_for_action` selection comment)
  - Evidence: `crates/fret-runtime/src/keymap.rs` (`display_shortcut_for_command_sequence`), used by OS menubar + command palette + in-window menu surfaces.

- **Menu bar presentation modes (OS vs in-window)** (done)
  - Goal: let apps choose native OS menubar vs client-side in-window menubar while sharing one data-only `MenuBar` and one keymap/when model.
  - Workstream: `docs/workstreams/os-menubar.md` (MVP 2.5)
  - Evidence: `crates/fret-app/src/settings.rs` (`SettingsFileV1.menu_bar`), `crates/fret-app/src/menu_bar.rs` (`sync_os_menu_bar`), `apps/fret-ui-gallery/src/driver.rs` (in-window fallback decision).
  - Current: `fret-ui-gallery` exposes a basic Settings sheet to toggle `menu_bar.os` / `menu_bar.in_window` and write `.fret/settings.json` (`apps/fret-ui-gallery/src/driver.rs`).

- **Standard menu roles and system menus (macOS-first)**
  - Problem: macOS expects standard menus (App/Window/Services) and native edit actions; relying on menu titles is fragile and blocks localization/customization.
  - ADR: `docs/adr/0185-menu-roles-system-menus-and-os-actions.md`
  - Current:
    - Roles/system menus are modeled (`MenuRole`, `SystemMenuType`) and `menubar.json` v2 can express them.
    - macOS runner honors roles (Window/App/Help) and Services system menu, and uses `OsAction` for standard edit selectors.
    - Workspace baseline and `fret-kit` default workspace shell can inject an App menu (About/Preferences/Services/Hide/Hide Others/Show All/Quit) via commands.
    - `fret-bootstrap` handles `app.quit`/`app.hide*` by emitting platform effects (`QuitApp`/`HideApp`/`HideOtherApps`/`UnhideAllApps`), so these commands work by default in the golden path.
  - Remaining TODO:
    - define the remaining macOS App menu conventions (e.g. Hide Others/Show All wording, and standard “Hide Others” placement vs Services) and decide which are command-driven vs runner-native;
    - decide how the App menu title should be derived by default (bundle/app title vs explicit config).
      - Current: `fret-bootstrap` seeds `AppDisplayName` from `WinitRunnerConfig.main_window_title`, and `fret-kit`
        uses it as the default `MenuRole::App` title (fallback `"App"`).

- **Define quit semantics for menu + window close** (done)
  - Problem: `Effect::QuitApp` exits the native event loop immediately; we need a clear policy for "Quit" vs closing windows (and unsaved changes prompts) in the golden path.
  - ADRs: `docs/adr/0001-app-effects.md`, `docs/adr/0094-window-close-and-web-runner-destroy.md`
  - Workstream: `docs/workstreams/os-menubar.md` (MVP 3 gap)
  - Current: native `QuitApp` requests are mediated by `before_close_window` (prompt gate) and then force-close all windows before exiting, so quit works with "unsaved changes" prompts without re-entrancy.
  - Evidence: `crates/fret-launch/src/runner/desktop/mod.rs` (`Effect::QuitApp`, `WindowRequest::Close` + `exit_on_main_window_close`), `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (global `app.quit` handling).

- **Apply shadcn theme presets in shadcn demos by default**
  - Problem: shadcn-aligned components look "off" if the global theme does not provide the expected semantic tokens (or is tuned for a different palette).
  - Current: `apps/fret-examples/src/todo_demo.rs` applies `shadcn/new-york-v4/slate/light` on startup.
  - TODO: decide whether this remains a per-demo choice or becomes a small helper in the bootstrap layer (without making `fret-bootstrap` depend on `fret-ui-shadcn`).

## P0 - shadcn Components / Layout Correctness

- **Tabs can trigger layout recursion / stack overflow**
  - Symptom: `shadcn::Tabs` can crash the app at startup with `thread 'main' has overflowed its stack` (observed on Windows).
  - Hypothesis: a `TabsContent` sizing recipe (`flex: 1` / "fill remaining space") can cause deep layout recursion when composed under parents without a definite main-axis size.
  - ADRs: `docs/adr/0115-available-space-and-non-reentrant-measurement.md`, `docs/adr/0116-window-scoped-layout-engine-and-viewport-roots.md`
  - Roadmap: `docs/layout-engine-refactor-roadmap.md`
  - Code: `ecosystem/fret-ui-shadcn/src/tabs.rs`, `crates/fret-ui/src/declarative/host_widget/layout.rs`
  - Current: `TabsContent` no longer uses a default `flex: 1` sizing recipe (to avoid runaway recursion in invalid compositions).
  - Current: regression test added in `ecosystem/fret-ui-shadcn/src/tabs.rs` (`tabs_layout_regression_does_not_stack_overflow_in_auto_sized_column`).
  - TODO: decide and document the sizing contract for `TabsContent` (when is "fill remaining space" valid, and how do we express it safely in the declarative layout engine?).

- **Typography table (`typography-table`) parity**
  - Current: geometry gate exists (row heights + cell rects) using `goldens/shadcn-web/v4/new-york-v4/typography-table.json` via `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`.
  - Current: shadcn theme defines a `prose` typography baseline (16px/24px) to match web `computedStyle` in typography pages.
  - Current: paint-backed gate exists for `even:bg-muted` (web uses `lab(...)`), using CSS color parsing helpers + scene quad background matching.

- **Progress (`progress-demo`) parity**
  - Current: geometry + paint-backed gates exist for the track (`bg-primary/20`) and indicator (`bg-primary`) using `goldens/shadcn-web/v4/new-york-v4/progress-demo.json` via `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (light+dark).
  - Current: indicator translateX matches upstream percent-based transform (the DOM `w-full` indicator with `translateX(-${100 - value}%)`), validated against the web `getBoundingClientRect` geometry.

- **Golden-path window close behavior**
  - Symptom: clicking the window close button (X) does nothing in minimal `UiAppDriver` apps unless the app explicitly handles `Event::WindowCloseRequested`.
  - ADRs: `docs/adr/0020-focus-and-command-routing.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`
  - Code: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
  - Current: `UiAppDriver` closes windows by default on `Event::WindowCloseRequested`, with an opt-out for "unsaved changes" prompts.
  - Current: documented in `docs/examples/todo-app-golden-path.md`.

## P0 - Radix/shadcn Overlay Conformance (Goldens + Downshift)

- **Downshift hover-overlay intent drivers into `fret-ui-kit::headless`**
  - Problem: hover-driven overlays (Tooltip/HoverCard) currently contain substantial state/intent logic in shadcn recipes, which makes long-term 1:1 Radix matching harder (logic drift is easy when it is not shared/reused).
  - ADRs: `docs/adr/0090-radix-aligned-headless-primitives-in-fret-components-ui.md`, `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
  - Targets (examples to audit/move):
    - `ecosystem/fret-ui-shadcn/src/hover_card.rs` (`HoverCardIntentDriverState`, frame-tick fallback, close suppression heuristics).
    - `ecosystem/fret-ui-shadcn/src/tooltip.rs` (pointermove gating + suppress-after-pointerdown/focus heuristics).
  - Approach:
    - keep wiring in shadcn recipes, but move the deterministic state machine and timers into `ecosystem/fret-ui-kit/src/headless/*` (or extend existing headless primitives like `hover_intent`).
    - add unit tests at the headless layer for the intent driver (open/close timing, suppression edges), then keep only "wiring smoke" in shadcn.

- **Expand overlay goldens to cover submenu and non-click open paths**
  - Goal: lock down the highest-drift overlay behaviors (submenu grace corridor, delayed opens, focus transfer) with upstream web goldens.
  - Upstream references:
    - `repo-ref/primitives/packages/react/menu/src/menu.tsx` (submenu pointer grace + focus transfer rules).
  - Current:
    - Added dropdown-menu submenu hover-open + select timeline: `goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-hover-select.light.json`.
    - Added Fret gate covering submenu open + close-on-select: `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`.
    - Added context-menu submenu hover-open + select timeline: `goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-hover-select.light.json`.
    - Added menubar submenu hover-open + select timeline: `goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-hover-select.light.json`.
    - Added submenu pointer-grace corridor timelines:
      - `goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-grace-corridor.light.json`
      - `goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-grace-corridor.light.json`
      - `goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-grace-corridor.light.json`
    - Added Fret gates covering pointer-grace corridor staying open: `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`.
    - Added submenu keyboard open/close timelines:
      - `goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-keyboard-open-close.light.json`
      - `goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-keyboard-open-close.light.json`
      - `goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-keyboard-open-close.light.json`

## P0 - Docking / Multi-Window Tear-off

- **ImGui-style multi-window tear-off parity (macOS-first, but cross-platform)**
  - Goal: editor-grade “tear off → hover another window → re-dock → close empty window” experience.
  - Workstream:
    - Narrative: `docs/workstreams/docking-multiwindow-imgui-parity.md`
    - TODO tracker: `docs/workstreams/docking-multiwindow-imgui-parity-todo.md`
    - macOS detail: `docs/workstreams/macos-docking-multiwindow-imgui-parity.md`
  - Contract gates:
    - `docs/adr/0013-docking-ops-and-persistence.md`
    - `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
    - `docs/adr/0072-docking-interaction-arbitration-matrix.md`
    - Added Fret gates covering submenu ArrowRight open + ArrowLeft close + focus restore:
      `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`.
    - Added/updated Radix Vega timeline state gates for:
      - tooltip hover open/close + Escape dismissal,
      - hover-card hover-out (content remains mounted with `data-state=closed`),
      - navigation-menu Escape close clears selected value.
    - Normalized Radix web `press` simulation in state gates to dispatch `KeyDown`+`KeyUp` (so
      activation semantics match web timelines consistently, without per-test patches).
    - Done: Added explicit portal size gates (`portal_w`/`portal_h`) for `Menu` and `ListBox`
      overlays in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` and aligned
      shadcn Select listbox sizing via a width probe (longest item label + padding).
    - Added shadcn-web tiny-viewport (`vp1440x240`) open goldens for common overlay recipes and
      extended Fret gates for placement/insets + menu sizing signals. Extended overlay chrome gates
      to assert `computedStyle`-derived surface colors (background + border) for `dialog-content`,
      `sheet-content`, `popover-content`, `dropdown-menu-content`, `dropdown-menu-sub-content`,
      `context-menu-content`, `context-menu-sub-content`, `menubar-content`, `menubar-sub-content`,
      `navigation-menu-content`, `select-content`, `hover-card-content`, `tooltip-content`, and
      `drawer-content` (light/dark where available).
    - Extended overlay chrome gates to cover constrained-viewport submenu keyboard variants
      (`*.submenu-kbd-vp1440x240.open.json`) for `dropdown-menu-sub-content`, `context-menu-sub-content`,
      and `menubar-sub-content`.
    - Extended ScrollArea conformance to gate thumb background/alpha against web `computedStyle`
      in hover-visible states (light/dark), and aligned the shadcn ScrollArea default thumb alpha to 1.0.
    - Added `table-demo` layout conformance gates (header/body/footer row heights + caption gap) and aligned
      shadcn `TableRow` height behavior by removing the unconditional `min_h=40` default.
    - Added `data-table-demo` layout conformance gates for row height and key control sizing
      (checkbox 16x16, action button 32x32).
    - Implemented `TableCell::col_span` for shadcn Table primitives (required for `table-demo` footer and
      the upcoming data-table empty state `colSpan={columns.length}` parity).
    - Added `data-table-demo.empty` web golden + layout gate for empty state `td` geometry (`colSpan` + `h-24`).
  - Goldens to expand:
    - `goldens/shadcn-web/v4/new-york-v4/*.open.json`: add open snapshots for pages that require non-click input and/or submenu open states.
  - Fret gates to add:
    - behavior/semantics sequence parity: `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` (new scenarios).
    - placement/chrome parity: extend `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_*` to cover submenu content and multi-layer placement.

## P0 - Docking / Overlays / Viewport Capture

- **Dock host keep-alive and early submission**
  - Goal: ensure dock hosts remain stable targets and do not "drop" docked content due to conditional submission.
  - ADRs: `docs/adr/0013-docking-ops-and-persistence.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`
  - Code: `ecosystem/fret-docking/src/dock/space.rs`, `ecosystem/fret-docking/src/dock/manager.rs`, runner/driver UI build order.

- **Programmatic close without one-frame tab "hole"**
  - Goal: add a `DockOp`/notify pattern so closing tabs from commands does not produce a transient no-selection/flicker.
  - ADRs: `docs/adr/0013-docking-ops-and-persistence.md`
  - Code: `ecosystem/fret-docking/src/dock/space.rs`, app integration applying `DockOp` + invalidation.

## P0 - Scheduling / Render Lifecycle

- **Adopt the continuous frames lease contract across high-frequency subsystems**
  - Goal: use RAII `begin_continuous_frames` leases (ADR 0034) for viewport rendering, drags, and animations, and avoid ad-hoc RAF loops.
  - ADRs: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`
  - Code: `crates/fret-ui/src/elements/mod.rs`, `crates/fret-launch/src/runner/mod.rs`

- **Investigate "doesn't draw until hover" initial render regressions**
  - Symptom: some demo surfaces appear blank on startup and only render after pointer movement/hover.
  - Hypothesis: missing initial invalidation/redraw request, or render_root/layout/paint ordering drift.
  - ADRs: `docs/adr/0028-declarative-elements-and-element-state.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`, `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
  - TODO: add a tiny regression harness in `fret-demo` and lock this down with a deterministic first-frame draw rule.

## P0 - Performance / Invalidation & Cache Boundaries

- **Enforce "hover/focus/pressed is Paint-only" across primitives and ecosystem**
  - Goal: pointer hover changes should not trigger `Invalidation::Layout` (avoid view-cache busting and layout solve churn).
  - ADRs: `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`, `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`
  - TODO:
    - add a diagnostic report for "Hover → Layout invalidations" (top offenders + element paths);
    - add a regression test that a `HoverRegion` toggle only invalidates paint unless a component opts in;
    - document an authoring rule: do not change subtree root kind/shape on hover; use `Opacity` or `InteractivityGate`.

- **Standardize stable identity (keying) + cache boundaries for expensive subtrees**
  - Goal: ensure per-frame rebuild does not allocate/re-measure large subtrees unnecessarily (markdown/code-view/tab strips/lists).
  - ADRs: `docs/adr/1152-view-cache-subtree-reuse-and-state-retention.md`, `docs/adr/1155-cache-root-tracing-contract-v1.md`
  - TODO:
    - require `cx.keyed(...)` for list-like rendering and block rendering (e.g. Markdown blocks via `BlockId`);
    - promote `ViewCache` usage in demos for heavy blocks (Markdown, code-view) and audit hover does not bust cache roots;
    - add a small "cache boundary checklist" for component authors (what must be inside/outside a cache root).

## P1 - Accessibility (A11y) Conformance

- **Define minimum semantics for text fields (value/selection/composition)**
  - Goal: Narrator/AccessKit correctness for text editing and IME interaction.
  - ADRs: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`, `docs/adr/0071-text-input-multiline-composition-contract.md`
  - Code: `crates/fret-ui/src/tree/mod.rs`, `crates/fret-a11y-accesskit/src/lib.rs`, `crates/fret-runner-winit/src/accessibility.rs`

- **Viewport semantics contract**
  - Goal: decide viewport role/actions (focus, scroll, basic labeling) and validate reachability under modal barriers.
  - ADRs: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`, `docs/adr/0007-viewport-surfaces.md`

## P1 - Tooling / Regression Harness

- **Hotpatch "golden path" validation loop (dx + smoke demo)**
  - Goal: keep an always-working end-to-end Subsecond patch loop for native dev.
  - ADRs: `docs/adr/0107-dev-hotpatch-subsecond-and-hot-reload-safety.md`
  - Tooling: `fretboard dev native --bin hotpatch_smoke_demo --hotpatch-dx`
  - TODO: add a short checklist and expected log markers (devserver connect, patch applied, safe reload boundary).
  - Bug: after `dx` reports `Hot-patching: ...`, the demo may crash with `thread 'main' has overflowed its stack`.
  - Update: `subsecond::apply_patch` succeeds and the runner completes `hot_reload_all_windows`, but the next `ViewFn` call via `subsecond::HotFn` overflows the stack before returning.
  - Diagnostics:
    - `.fret/hotpatch_runner.log` confirms `apply_patch ok` + runner window reset.
    - `.fret/hotpatch_bootstrap.log` confirms the `ViewFn` is mapped into the patch DLL (`mapped_module=...libhotpatch_smoke_demo-patch-*.dll`) and the crash happens during `hotfn.call(...)`.
    - If `FRET_HOTPATCH_DIAG_BYTES=1` is set, the log captures the patched prologue:
      - both old and new `view` start with a large stack frame (e.g. `mov eax, 0x30f0; call ...`), which implies stack probing (`__chkstk` / `__rust_probestack`-style) is involved;
      - the patched call target is a ThinLink thunk in the patch DLL that jumps to an absolute address in the base EXE.
  - Hypothesis: a Windows/ThinLink edge case around stack-probe thunks or other absolute-call stubs causes recursion inside patched code (manifesting as stack overflow).
  - Workarounds:
    - Set `FRET_HOTPATCH_VIEW_CALL_DIRECT=1` to bypass `HotFn` for the `ViewFn` call (prevents the crash but disables view-level hotpatching).
    - Reduce stack usage in hotpatched functions (especially avoid large `vec![...]` literals of `AnyElement`/large value types that force stack probing) to see if the crash is tied to the probe thunk path.

- **Add a repeatable IME regression checklist to the demo**
  - Goal: a short "manual test script" that can later be automated (Windows Japanese IME, caret placement, commit/cancel).
  - ADRs: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0071-text-input-multiline-composition-contract.md`
  - Code: `apps/fret-examples/src/components_gallery.rs` (stable harness location).

- **Prefer `cargo nextest` for workspace tests**
  - Goal: make it easy to run conformance tests consistently.
  - Docs: `docs/README.md`, `docs/adr/README.md`

- **Harden radix-web golden extraction (determinism + Windows dev loop)**
  - Problem: upstream examples can include external images (e.g. avatar images), which makes DOM
    timelines nondeterministic when the image load races snapshots; Windows shell semantics can
    also break parallel dev scripts (e.g. `&` not backgrounding).
  - Tooling: `goldens/radix-web/scripts/extract-behavior.mts`, `goldens/radix-web/README.md`
  - TODO: keep extractor deterministic (block images / settle timing), and document a known-good
    Windows command sequence for starting the preview server + regenerating goldens.

## P1 - Core Contract Drift

- **Formalize the vector path contract now that `SceneOp::Path` exists**
  - Problem: `fret-core::vector_path` and `SceneOp::Path` are implemented, but the contract is not yet locked at the ADR level (stroke joins/caps, AA expectations, transform interaction, caching keys).
  - ADRs: `docs/adr/0080-vector-path-contract.md`, `docs/adr/0002-display-list.md`, `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
  - Code: `crates/fret-core/src/vector_path.rs`, `crates/fret-core/src/scene.rs`, `crates/fret-render/src/renderer/mod.rs`
  - Update: contract locked (ADR 0080). Follow-up work is conformance testing and any v2 surface expansion (joins/caps/dashes).

- **Clarify the runner vs platform split in docs and code**
  - Problem: winit glue (including optional AccessKit bridge) lives in `fret-runner-winit`, while effect draining/presentation live in `fret-launch`; keep responsibilities crisp to avoid duplicating window registries and event translation.
  - ADRs: `docs/adr/0003-platform-boundary.md`
  - Code: `crates/fret-platform/src/*`, `crates/fret-runner-winit/src/accessibility.rs`, `crates/fret-runner-winit/src/lib.rs`, `crates/fret-launch/src/runner/*`

- **Decide whether `fret-runner-winit` grows into a broader native boundary**
  - Problem: `crates/fret-platform` is now intentionally portable contracts-only, while the concrete native backend lives in `crates/fret-platform-native` and the event loop/effect draining live in `crates/fret-launch`; decide how much window registry/event translation should live in the runner as more backends (web/mobile) arrive.
  - ADRs: `docs/adr/0003-platform-boundary.md`
  - Code: `crates/fret-platform-native/src/*`, `crates/fret-runner-winit/src/lib.rs`, `crates/fret-launch/src/runner/*`
