# API Ergonomics Audit (Draft)

Date: 2026-01-21  
Scope: authoring ergonomics for app and ecosystem component code

This audit focuses on whether Fret鈥檚 public surfaces are ergonomic enough for:

- application authors (fast iteration, high code density),
- ecosystem authors (third-party crates composing cleanly),
- 鈥渇oreign UI鈥?integration (reusing other UI ecosystems without collapsing Fret鈥檚 contracts).

Repository principle reminders:

- Kernel vs ecosystem boundaries must remain intact (see `docs/adr/0066-fret-ui-runtime-contract-surface.md`).
- Prefer a single 鈥済olden path鈥?authoring surface in ecosystem crates; keep kernel contracts stable and minimal.

## Executive Summary

Fret already has the core primitives required for a GPUI-style authoring experience:

- Declarative per-frame element tree + stable identity (`ElementContext::{scope,keyed,named}`, `GlobalElementId`).
- Externalized cross-frame element state (`with_state_for`).
- Explicit model/global observation with invalidation strength (ADR 0051).
- Ecosystem-level unified patch/builder surface (`UiExt::ui()` / `UiBuilder`, ADR 0175 Proposed).

However, ergonomics are currently limited by:

- pervasive `Vec<AnyElement>` requirements (children closures and component constructors),
- inconsistent use of the unified builder surface in examples/templates,
- lack of a single 鈥渄efault鈥?composition path that third-party crates can implement once and reuse everywhere.

## Progress (Living Checklist)

This document is meant to stay 鈥渓ive鈥?while we iterate. Update this section whenever we land meaningful ergonomics work.

- [x] ADR 0189 clarifies the policy: foreign UI integration is isolated surfaces only.
- [x] `fret-kit` provides an ecosystem-level golden-path helper for embedded viewports:
  - host-recorded surface: `EmbeddedViewportRecord` + `drive_embedded_viewport()`
  - foreign-hosted surface: `EmbeddedViewportForeignUi` + `set_foreign_ui(...)` + `drive_embedded_viewport_foreign()`
  - Evidence: `ecosystem/fret-kit/src/interop/embedded_viewport.rs`
- [x] Examples for both integration styles exist:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs` (host-recorded embedded surface)
  - `ecosystem/fret-kit/src/interop/embedded_viewport.rs` (`EmbeddedViewportForeignUi` + `drive_embedded_viewport_foreign` for foreign-hosted path)
- [x] Command palette gating consumes the data-only snapshots:
  - `WindowCommandEnabledService` (explicit overrides) and `InputContext`/`WhenExpr` (catalog gating).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`, `ecosystem/fret-ui-shadcn/src/command.rs`.
- [x] Desktop system menus consume the data-only snapshots (Windows/macOS):
  - Evidence: `crates/fret-launch/src/runner/desktop/windows_menu.rs`, `crates/fret-launch/src/runner/desktop/macos_menu.rs`.
- [x] A consumption-focused per-window gating snapshot aggregates all data-only inputs (P0):
  - `WindowCommandGatingSnapshot` unifies `InputContext` + overrides (+ optional dispatch-path availability snapshot).
  - Evidence: `crates/fret-runtime/src/window_command_gating.rs`,
    `crates/fret-runtime/src/window_command_action_availability.rs`,
    `crates/fret-ui/src/tree/commands.rs`,
    `crates/fret-launch/src/runner/desktop/windows_menu.rs`,
    `crates/fret-launch/src/runner/desktop/macos_menu.rs`,
    `ecosystem/fret-bootstrap/src/ui_app_driver.rs`.
- [x] Declarative components can participate in dispatch-path availability queries (without new kernel widget types).
  - Evidence: `crates/fret-ui/src/action.rs` (`OnCommandAvailability`, `CommandAvailabilityActionCx`),
    `crates/fret-ui/src/declarative/host_widget.rs` (invokes availability hook during queries)
  - Example surfaces: `ecosystem/fret-ui-kit/src/declarative/{list,table}.rs` (`*_virtualized_copyable`)
- [x] Cross-surface clipboard commands exist beyond text widgets (`edit.copy`) with availability + effects wired.
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/{list,table}.rs`, `ecosystem/fret-node/src/ui/canvas/widget.rs`
  - Tests: `ecosystem/fret-ui-kit/src/declarative/{list,table}.rs` (`*_reports_availability_and_emits_clipboard_text`),
    `ecosystem/fret-node/src/ui/canvas/widget/tests/edit_command_availability_conformance.rs`
- [x] Scroll-into-view is stable when already scrolled (prevents focus traversal 鈥渄rift鈥?.
  - Evidence: `crates/fret-ui/src/declarative/host_widget.rs` (`ElementHostWidget::scroll_descendant_into_view`)
  - Tests: `crates/fret-ui/src/tree/tests/scroll_into_view.rs` (`scroll_into_view_does_not_drift_*`)
- [x] Representative todo demo adopts `ModelWatchExt` to reduce observe+read boilerplate.
  - Evidence: `apps/fret-examples/src/todo_demo.rs`
- [x] `fretboard new` templates demonstrate iterator-friendly children composition (no forced `vec![...]` in child closures).
  - Evidence: `apps/fretboard/src/scaffold/templates.rs`
- [x] Globals can be observed/read ergonomically (`GlobalWatchExt`).
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/global_watch.rs`
  - Example: `apps/fret-examples/src/assets_demo.rs`
  - Example: `apps/fret-examples/src/markdown_demo.rs`
- [x] Make a single 鈥渄efault authoring dialect鈥?the norm in examples/templates (ADR 0175 + `UiExt::ui()`).
  - Templates: `apps/fretboard/src/scaffold/templates.rs` (todo/hello use `ui::*` + `.ui()`).
  - Demos: `apps/fret-examples/src/todo_demo.rs`.
- [x] Migrate remaining demos to the default authoring dialect (`UiExt::ui()` / `ui::*`).
  - Migrated: `apps/fret-examples/src/assets_demo.rs`, `apps/fret-examples/src/cjk_conformance_demo.rs`, `apps/fret-examples/src/emoji_conformance_demo.rs`.
  - Migrated: `apps/fret-examples/src/todo_demo.rs`.
  - Migrated: `apps/fret-examples/src/components_gallery.rs`, `apps/fret-examples/src/markdown_demo.rs`.
  - Remaining: none.
- [x] Reduce Vec-first friction (P1, first batch): accept `IntoIterator<Item = AnyElement>` across high-frequency APIs.
  - Evidence: `crates/fret-ui/src/elements/cx.rs` (`pressable_with_id_props`), `ecosystem/fret-ui-kit/src/overlay_controller.rs` (`OverlayController::hover`)
  - Evidence: `crates/fret-ui/src/element.rs` (`Elements` collection wrapper + conversions)
  - Evidence: `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (`ViewElements = Elements` for `UiAppDriver` view fns)
  - Evidence: `ecosystem/fret-kit/src/mvu.rs` (`Program::view -> Elements`)
  - Evidence: `ecosystem/fret-ui-kit/src/tree.rs` (`TreeRowRenderer::* -> Elements`)
  - Evidence: `ecosystem/fret-node/src/ui/registry.rs` (`NodeGraphNodeRenderer -> Elements`)
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/requests.rs` (`*Request::new(..., children: impl IntoIterator<Item = AnyElement>)`)
  - Evidence: `ecosystem/fret-ui-kit/src/{ui,ui_builder}.rs`, `ecosystem/fret-ui-kit/src/declarative/{cached_subtree,chrome,dismissible,glass,pixelate,scroll,stack,visually_hidden}.rs`, `ecosystem/fret-ui-kit/src/primitives/{accordion,dismissable_layer,menu/*,popover,popper_content,roving_focus_group,tabs,toggle,toolbar}.rs`, `ecosystem/fret-ui-primitives/src/focus_scope.rs`
  - Evidence: `ecosystem/fret-ui-shadcn/src/{breadcrumb,card,command,field,input_group,item,scroll_area,select,toggle,tooltip}.rs`,
    `ecosystem/fret-ui-shadcn/src/ui_builder_ext/breadcrumb.rs`
- [x] Fill P0 authoring gaps discovered during demo migration: px/metric-aware `gap` and one-value padding shorthands.
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs` (`FlexBox` uses `MetricRef` for `gap`)
  - Evidence: `ecosystem/fret-ui-kit/src/style/refs.rs` (`MetricRef: From<Px/Space/Radius>`)
  - Evidence: `ecosystem/fret-ui-kit/src/style/layout.rs` (`w_px`/`min_w`/`max_w`/`basis_px` accept `Into<MetricRef>`)
  - Evidence: `ecosystem/fret-ui-kit/src/ui_builder.rs` (`gap` accepts `Into<MetricRef>`, `padding_px`)
  - Evidence: `ecosystem/fret-ui-kit/src/overlay_controller.rs` (`OverlayRequest::*` accepts `IntoIterator<Item=AnyElement>`)
  - Example: `apps/fret-ui-gallery/src/ui.rs` (uses `.w_px(Px(..))`/`.h_px(Px(..))` without `MetricRef::Px(...)`)
  - Example: `ecosystem/fret-ui-shadcn/src/spinner.rs` (uses `.w_px(Px(..))`/`.h_px(Px(..))` without `MetricRef::Px(...)`)
  - Example: `ecosystem/fret-ui-kit/src/declarative/collapsible_motion.rs` (uses `.h_px(Px(..))` without `MetricRef::Px(...)`)
- [x] Provide a fluent scroll-area wrapper in the default authoring dialect (`ui::*`).
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs` (`ui::scroll_area`)
  - Example: `apps/fret-examples/src/markdown_demo.rs`
- [x] Consolidate third-party 鈥渃omponent integration contract鈥?guidance (P2) with a short checklist.
  - Evidence: `docs/component-authoring-contracts.md`
  - Evidence: `docs/component-author-guide.md`

## Comparison Matrix (Iced vs GPUI vs Fret)

This table is intentionally focused on 鈥渨hat an app author experiences鈥?and 鈥渉ow an ecosystem author plugs in鈥?

| Dimension | Iced | GPUI | Fret (current) | Fret (direction) |
| --- | --- | --- | --- | --- |
| Default authoring loop | `update/view` MVU | `Render` rebuild per frame | declarative element tree, explicit invalidation + models | GPUI-like rebuild + externalized state, but with explicit invalidation tooling |
| Message routing | typed `Message` everywhere | typed actions / callbacks | commands + effects; MVU helper exists in `fret-kit` | typed MVU/program surfaces in ecosystem, keep kernel minimal |
| UI tree model | immediate rebuild | immediate rebuild | long-term goal is rebuild; today has retained prototype + declarative elements | unify around declarative rebuild with stable identity |
| State ownership | user `State` | framework entities | app-owned models + element state scopes | keep app-owned models; improve sugar to make it 鈥渇eel like one way鈥?|
| Layout & styling | framework-provided widgets, theme | framework styling + components | mechanism in `fret-ui`, policy/taxonomy in ecosystem | keep policy in ecosystem; improve density + defaults |
| Ecosystem integration | widget crates, renderer backends | component crates, entities | ecosystem crates (`fret-ui-kit` / `fret-ui-shadcn` / `fret-kit`) | strengthen a single integration contract for third-party crates |
| Mixing other runtimes | possible but complex | not typical | supported via isolated surfaces only (ADR 0189) | keep this boundary explicit and ergonomic |

## Evidence (Current State)

### `Vec<AnyElement>` friction hotspots

- `ElementContext` builder-style helpers use `FnOnce -> Vec<AnyElement>` in many places.
  - Evidence: `crates/fret-ui/src/elements/cx.rs` has many `FnOnce(&mut Self) -> Vec<AnyElement>` occurrences.
- Many `fret-ui-shadcn` components accept `children: Vec<AnyElement>` in constructors and setters.
  - Evidence: `ecosystem/fret-ui-shadcn/src` contains many `children: Vec<AnyElement>` fields and `new(children: Vec<AnyElement>)` constructors.

This forces application code and third-party crates into a 鈥渧ector-first鈥?authoring style, reducing code density and composability with iterators/arrays.

### Unified builder surface exists (but must become the default)

`fret-ui-kit` provides a patch aggregator and a single authoring entrypoint:

- `UiExt::ui()` and `UiBuilder<T>::into_element(cx)`
  - Evidence: `ecosystem/fret-ui-kit/src/ui_builder.rs`

`fret-ui-shadcn` already wires many components into this builder surface via macros:

- Evidence: `ecosystem/fret-ui-shadcn/src/ui_ext/support.rs`

The remaining gap is adoption and completeness: examples and templates must make this the 鈥渙ne way鈥?

### Model observation ergonomics exist (component-layer sugar)

`ModelWatchExt` provides 鈥渙bserve + read鈥?sugar while preserving explicit invalidation semantics:

- Evidence: `ecosystem/fret-ui-kit/src/declarative/model_watch.rs`

## What鈥檚 Already Good (Keep It)

- `ElementContext::{scope,keyed,named}` provides deterministic identity without a diff engine.
- `for_each_keyed` is the correct direction for list identity and safety.
- Explicit invalidation strength (Paint/Layout/HitTest) is appropriate for editor-grade correctness, as long as:
  - the default path is ergonomic (`ModelWatchExt`, `UiExt`),
  - 鈥減ower user鈥?paths remain available (`observe_model`, direct runtime props).
- `fret-ui-shadcn` taxonomy + `fret-ui-kit` policy infrastructure is the right ecosystem layering.

## Main Ergonomics Risks

### R1) 鈥淰ector-first authoring鈥?reduces composability

Symptoms:

- Frequent `vec![...]` boilerplate.
- Harder to stream/compose child lists (iterators), conditionally append children, etc.
- Third-party crates tend to fork local helpers instead of sharing a universal pattern.

### R2) Two authoring dialects in app code

Symptoms:

- Apps mix:
  - direct runtime props (`ContainerProps`, `LayoutStyle`, manual `Theme::global(...)`),
  - per-component `refine_style/refine_layout`,
  - builder surface `.ui()...`.

This fragments teaching, makes examples harder to scan, and increases the cost of third-party component integration.

### R3) 鈥淔oreign UI mixing鈥?can collapse contracts

Attempting to mix other UI runtimes (Iced/Egui/etc) inside the same tree tends to require deep semantic bridges:

- layout, hit testing, event routing, focus/IME, a11y, invalidation.

This is high-risk and likely to bloat kernel contracts. The recommended approach is 鈥渋solated embedding鈥?

## Recommendations (Prioritized)

### P0 鈥?Make the golden path the default (docs + examples)

- Update representative examples to use:
  - `UiExt::ui()` (builder surface),
  - `ModelWatchExt` (observe+read sugar),
  - `stack::{hstack,vstack}` (avoid direct runtime props unless needed).
- Ensure `fret-kit` templates and `fretboard new` outputs follow this style.
  - Template shortcut: `fretboard new todo`.

Acceptance: new users can build a non-trivial UI without touching `LayoutStyle` directly.

### P1 鈥?Reduce `Vec<AnyElement>` requirements via `IntoIterator`

Change high-frequency APIs to accept `IntoIterator<Item = AnyElement>`:

- `ElementContext` subtree builders (`container`, `row`, `column`, etc).
- `fret-ui-shadcn` constructors/setters that currently take `Vec<AnyElement>`.

This should be source-compatible for callers passing `Vec`, while enabling:

- arrays/slices,
- iterators,
- incremental composition without intermediate allocations.

Practical rollout plan (to avoid a 鈥渇lag day鈥?:

- P1.1 (kernel-friendly, low risk): update `crates/fret-ui` authoring helpers so closures return `impl IntoIterator<Item = AnyElement>` consistently.
- P1.2 (ecosystem surface): update `fret-ui-shadcn` public constructors/setters to accept `impl IntoIterator<Item = AnyElement>` and internally `collect()` into stored `Vec` fields.
- Keep `AnyElementIterExt::elements()` as the escape hatch for iterator-heavy call sites until all APIs are migrated.

Acceptance:

- examples can compose children using iterators without `vec![...]` at call sites,
- third-party crates can expose `fn into_elements(...) -> impl Iterator<Item = AnyElement>` patterns naturally.

### P1.3 鈥?Elements ergonomics pass (in progress)

Tracked on branch/worktree: `refactor/elements-ergonomics-pass`.

Goal: remove remaining high-frequency 鈥渧ector-first鈥?authoring friction by preferring iterator-friendly inputs and
the `Elements` return wrapper where it improves composability.

Completed (so far):

- [x] Make `AnyElement::new` accept `children: impl IntoIterator<Item = AnyElement>`.
  - Evidence: `crates/fret-ui/src/element.rs` (`AnyElement::new`).
- [x] Make virtual list extension hooks iterator-friendly (avoid forcing `Vec<usize>` allocations).
  - Evidence: `crates/fret-ui/src/elements/cx.rs` (`virtual_list_*_ex` range extractor).
- [x] Migrate `fret-ui-kit` overlay primitives to accept iterable children (store internally as `Vec` when needed).
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/{dialog,alert_dialog,popover,tooltip,hover_card,menu/root,select}.rs`.
- [x] Migrate `fret-ui-shadcn` menu/command builder APIs to accept iterables for `entries/items`.
  - Evidence: `ecosystem/fret-ui-shadcn/src/{command,dropdown_menu,context_menu,menubar,navigation_menu,select}.rs`.
- [x] Return `Elements` from navigation menu spec builders where it improves composability.
  - Evidence: `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` (spec `children()`).
- [x] Make high-frequency row/cell render callbacks iterator-friendly (avoid forcing `Vec<AnyElement>`).
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/{list,table}.rs`,
    `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`.
- [x] Make `FormField::new` accept an iterable `control` list.
  - Evidence: `ecosystem/fret-ui-shadcn/src/form_field.rs`.
- [x] Make `NavigationMenuItem::new` accept iterable `content`.
  - Evidence: `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` (`NavigationMenuItem::new`).
- [x] Prefer `Elements` for provider-scoped subtree returns.
  - Evidence: `ecosystem/fret-ui-shadcn/src/tooltip.rs` (`TooltipProvider::with`).
- [x] Make overlay motion helpers accept iterable children (avoid forcing `Vec<AnyElement>`).
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs`.
- [x] Prefer `Elements` for Radix-like modal layer assembly helpers and make pressable builder helpers iterator-friendly.
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/{dialog,alert_dialog,popover,select,navigation_menu}.rs`.
- [x] Migrate Material3 tooltip/dialog surfaces away from `Vec<AnyElement>`-only contracts.
  - Evidence: `ecosystem/fret-ui-material3/src/tooltip.rs` (`TooltipProvider::with`),
    `ecosystem/fret-ui-material3/src/dialog.rs` (`Dialog::into_element`, `Dialog::actions`).
- [x] Make UI Gallery previews `Elements`-first (avoid `Vec<AnyElement>` glue at call sites).
  - Evidence: `apps/fret-ui-gallery/src/ui.rs` (`page_preview`, `material3_scoped_page`, `preview_*` builders).
- [x] Reduce `Vec<AnyElement>` return friction in ecosystem render helpers (where callers compose lists).
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (`render_menu_entries -> Elements`).
  - Evidence: `ecosystem/fret-markdown/src/pulldown_render.rs` (`render_pulldown_blocks -> Elements`).
  - Evidence: `ecosystem/fret-ui-shadcn/src/{menubar,context_menu,dropdown_menu}.rs` (`*_row_children` / `render_entries -> Elements`).
  - Evidence: `ecosystem/fret-ui-shadcn/src/table.rs` (`assign_grid_column_starts -> Elements`).

Next candidates (rolling):

- Migrate ecosystem APIs that currently accept `children: Vec<AnyElement>` to accept
  `children: impl IntoIterator<Item = AnyElement>` (store internally as `Vec`).
- Reduce callback churn where signatures are `FnOnce(...) -> Vec<AnyElement>` by allowing
  `FnOnce(...) -> Elements` or `FnOnce(...) -> impl IntoIterator<Item = AnyElement>` when feasible.
- Add lightweight test helpers for pointer events (reduce repeated `PointerId(0)` / `PointerType::Mouse` /
  `Modifiers::default()` boilerplate and prevent missing-field regressions).

Acceptance:

- common call sites no longer require `vec![...]` or `.collect::<Vec<_>>()` for simple composition,
- no new kernel-policy coupling (pure authoring ergonomics),
- examples and templates demonstrate the new patterns.

### P2 鈥?Standardize 鈥渢hird-party component integration contract鈥?
Define and document a small set of traits a third-party component should implement once:

- `UiPatchTarget` (+ optional `UiSupportsChrome/Layout`) for patch aggregation,
- `UiIntoElement` for rendering,
- optionally `RenderOnce` for deeper kernel-level composition (if needed).

Make this guidance part of:

- component author docs,
- a checklist in `docs/component-author-guide.md` or a dedicated ecosystem audit doc.

### P3 鈥?Formalize 鈥渇oreign UI embedding鈥?as isolated surfaces

ADR 0189 formalizes the policy (and the kernel already supports the mechanism). The recommended ecosystem entrypoint is `fret-kit`鈥檚 embedded viewport helper surface.

Supported surface (ecosystem-level):

- `EmbeddedViewportSurface` owned by window state
- `EmbeddedViewportForeignUi` (object-safe boundary) registered per window via `set_foreign_ui(...)`
- single-call driver wiring: `drive_embedded_viewport_foreign()`

This enables 鈥渞euse other ecosystems鈥?without collapsing focus/IME/a11y contracts.

Policy statement (keep):

- Supported: embed foreign UI as a render target surface + event forwarding.
- Not supported: mixing foreign widget trees as first-class Fret elements (no shared a11y/focus/IME semantics).

This keeps kernel contracts stable and prevents accidental scope creep.

## Checklist for New Ecosystem Components

**Authoring surface**

- Expose a `ui()` entrypoint via `UiPatchTarget` and `UiIntoElement`.
- Prefer accepting children as `impl IntoIterator<Item = AnyElement>`.
- Avoid leaking `Theme` resolution into app code; resolve tokens in `into_element(cx)`.

**State and invalidation**

- Use `ModelWatchExt` for 鈥渙bserve + read鈥?in component code.
- Use `Invalidation::Paint` by default; escalate to `Layout` only when layout changes.
- Ensure dynamic lists use `for_each_keyed` (or explicit keys) for stable identity.

**Boundaries**

- Keep policy (dismissal, focus trap/restore, hover intent) in `fret-ui-kit` / shadcn layer.
- Keep `crates/fret-ui` as mechanism-only.

## Follow-up Work Items

- Promote ADR 0175 from Proposed 鈫?Accepted once example migration proves the UX.
- Continue expanding P1 (`IntoIterator`) beyond the initial shadcn batch (and make kernel authoring helpers consistent).

## Comparative References (Iced, GPUI)

This section is a quick 鈥渄esign vocabulary alignment鈥?to make future ADR discussion concrete.

### Iced (Elm-style MVU, task/subscription driven)

Key authoring traits:

- State as a plain Rust value, updated by messages (`update(&mut State, Message) -> Task<Message>`).
- UI described as an element tree built each frame (`view(&State) -> Element<Message>`).
- Message routing is typed and implicit (widgets produce `Message` directly).
- No explicit invalidation model in user code; the runtime schedules redraws.

Evidence anchors (reference checkout):

- `F:\SourceCodes\Rust\fret\repo-ref\iced\src\application.rs` (`iced::application(...) -> Application`).
- `F:\SourceCodes\Rust\fret\repo-ref\iced\src\lib.rs` pocket guide (鈥渦pdate/view/message鈥?narrative).

Ergonomic takeaway for Fret: typed message routing and 鈥渟ingle default entry point鈥?can be borrowed
without adopting Iced鈥檚 widget model or layout approach.

### GPUI (hybrid immediate/retained; entities + render each frame)

Key authoring traits:

- 鈥淰iews鈥?are entities implementing `Render`, called once per frame to rebuild the element tree.
- Element state is owned by the framework (entity storage), not by the tree itself.
- Context objects are the primary interface surface (AppContext / WindowContext).

Evidence anchors (reference checkout):

- `F:\SourceCodes\Rust\fret\repo-ref\zed\crates\gpui\README.md` (鈥渢hree registers鈥?overview).

Ergonomic takeaway for Fret: Fret鈥檚 long-term direction already resembles GPUI; the main gap is
authoring surface polish (density, defaults, unified patterns), not core capability.

## Prototype: `fret_kit::mvu` (Typed Commands Without String Parsing)

To test how much of the Iced 鈥渢yped message鈥?ergonomics we can adopt without changing kernel
contracts, `fret-kit` provides a small ecosystem-level MVU helper:

- `ecosystem/fret-kit/src/mvu.rs`
  - `Program` trait (`init/update/view`) with `State` + `Message`.
  - `MessageRouter<Message>` that allocates per-frame `CommandId` and resolves it in the driver hook.
  - `fret_kit::mvu::app::<P>(...)` mirrors the existing `fret_kit::app(...)` golden path.

Why this shape:

- The golden-path driver uses `fn` pointers for hotpatch predictability, so a trait-based program is
  the simplest way to keep 鈥渘o captured closures in wiring鈥?while still improving ergonomics.

Limitations (current):

- This does not remove the model observation/invalidation responsibilities yet (it only eliminates
  stringly `CommandId` parsing and prefix handling in app code).
- `MessageRouter` is frame-local; it is intentionally not a stable command registry.

## Code Size Snapshot (TODO demo family)

Line counts (snapshot):

- `apps/fret-examples/src/todo_demo.rs`: 556 (single official baseline after consolidation)

Interpretation:

- The consolidated todo story now optimizes maintainability (single baseline) instead of comparing multiple todo paradigms by LOC.
- The largest ergonomic win still comes from using `fret-kit` golden-path hooks rather than writing a full custom driver.

Recent ergonomic improvement:

- `fret-kit` now includes `interop::embedded_viewport` to reduce 鈥渋solated embedding鈥?boilerplate
  (publish target id as models + global viewport input filtering + surface panel helper).
- `interop::embedded_viewport::{EmbeddedViewportUiAppDriverExt, EmbeddedViewportMvuUiAppDriverExt}`
  further reduce wiring boilerplate (install input + frame recording in one call).
- `fret-ui` exposes a policy hook for command availability (`command_on_command_availability_for`) so ecosystem
  components can participate in OS menu / command palette gating without introducing new kernel widget types.
