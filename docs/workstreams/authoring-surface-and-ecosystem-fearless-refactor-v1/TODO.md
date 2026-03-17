# Authoring Surface + Ecosystem (Fearless Refactor v1) — TODO

This TODO list tracks the remaining closeout work described in `DESIGN.md`.

Because this is a pre-release reset, "done" means we actually delete the old surface rather than
carrying compatibility-only baggage.

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`

Execution note on 2026-03-12:

- treat this file as a closeout tracker,
- do not reopen broad surface redesign here,
- route remaining conversion-surface work to
  `docs/workstreams/into-element-surface-fearless-refactor-v1/`.

Status note:

- treat `MIGRATION_MATRIX.md` as the source of truth for lane/row status and delete-readiness,
- treat unchecked early bookkeeping items in this file as historical planning residue unless they
  still map to an active closeout task below,
- when this file and the matrix disagree, prefer the matrix plus the current source gates/tests.

Closeout note on 2026-03-15:

- this workstream should now be treated as a **targeted closeout lane**, not as a broad redesign
  backlog and not as a "maintenance only" archive,
- the app/component/advanced split itself does not need another broad redesign pass here,
- but three high-priority closeout tasks still belong here because they materially affect the
  public product surface:
  - narrowing `fret::app::prelude::*` so it is materially smaller than
    `fret::component::prelude::*` in both exports and autocomplete pressure,
  - reducing shadcn first-contact discovery to the curated facade lane rather than relying on
    source-policy tests to keep crate-root/facade/raw paths mentally sorted,
  - keeping `TARGET_INTERFACE_STATE.md` and its status matrix honest while
    `into-element-surface-fearless-refactor-v1` is still actively deleting surface families,
- remaining work is therefore a mix of docs cleanup, delete-ready follow-through, and explicit
  surface narrowing that is already implied by the target state but not yet fully reflected in the
  shipped exports.
- the next real product-surface pressure is no longer "how do we split app/component/advanced?",
  but rather:
  - finishing delete-ready cleanup on old root aliases and stale docs,
  - keeping the default app/component lane overlap closed and routing any remaining heavy helper
    families onto explicit secondary lanes instead of reopening wildcard-prelude growth,
  - keeping the conversion surface accurate in
    `docs/workstreams/into-element-surface-fearless-refactor-v1/`,
  - simplifying the shadcn discovery lane so `facade as shadcn` is the only first-contact story,
  - handling any future action-surface ergonomics in
    `docs/workstreams/action-first-authoring-fearless-refactor-v1/`.

Priority correction on 2026-03-15:

1. simplify shadcn first-contact discovery (`facade` first, `raw` explicit, crate root de-emphasized)
2. finish the conversion-surface reset under
   `docs/workstreams/into-element-surface-fearless-refactor-v1/`
3. only then add more small-app authoring sugar on top of the stabilized lane

Closeout note on 2026-03-16:

- the default app prelude is no longer the main open surface-reset blocker here; named overlap with
  `fret::component::prelude::*` is already down to `ui` and `Px`, and the remaining anonymous
  helper imports are the intentional app ergonomics budget rather than unresolved redesign debt,
- the advanced-import split is now effectively closed:
  `fret::advanced::prelude::*` no longer smuggles the component lane, first-party advanced code
  that still needs component authoring helpers imports `fret::component::prelude::*` explicitly,
  and both `cargo check -p fret-examples --all-targets` and
  `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app` now validate that
  posture,
- the `Component prelude` row is now also functionally closed at the `fret` facade level:
  env/responsive helpers, raw activation glue, and lower-level overlay nouns all live on explicit
  secondary lanes (`fret::env::{...}`, `fret::activate::{...}`, `fret::overlay::*`), and the
  component-author docs plus source-policy tests now lock that posture,
- the highest-value remaining closeout work in this folder is therefore:
  - continued shadcn discovery/doc tightening,
  - keeping this tracker honest while the conversion-surface follow-on workstream keeps deleting
    old vocabulary families.

Release-blocking closeout order on 2026-03-16:

1. reduce happy-path ceremony on the default app lane
2. keep the `AppActivateExt` bridge-empty rule source-gated as maintenance
3. resume ecosystem integration-trait budgeting against the frozen public lane story

Discovery-lane closeout note on 2026-03-16:

- `fret-ui-shadcn` component-family root modules are now crate-private rather than doc-hidden
  public residue,
- `raw::*` is now a real explicit wrapper lane rather than a re-export of public root modules,
- first-party gallery/example/source-policy gates now compile and pass against that posture,
- remaining shadcn work in this folder is therefore documentation/gate hygiene, not another
  discovery-lane redesign.

Root-budget freeze note on 2026-03-16:

- the `fret` root now keeps an explicit public-module allowlist gate and direct `pub use`
  allowlist gate,
- raw view-runtime helpers (`ViewWindowState`, `view_init_window`, `view_view`,
  `view_record_engine_frame`) now live on `fret::advanced::view::*` instead of `fret::view::*`,
- devloop helpers now live on `fret::advanced::dev::*` instead of `fret::dev::*`,
- remaining work in this folder is therefore no longer "what should still be on the root?", but
  "how much ceremony remains on the default app lane after the root is already closed".

Verification note on 2026-03-16:

- the closeout posture above has been revalidated after the latest docs/source updates with:
  - `cargo nextest run -p fret --lib authoring_surface_policy_tests:: --no-fail-fast`
  - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --no-fail-fast`
  - `cargo check -p fret-examples --all-targets`
- treat those commands as the minimum revalidation bundle before reopening any lane-budget or
  discovery-lane edits in this folder.

Ownership split on 2026-03-16:

- this workstream owns items 1 and 2 directly,
- item 3 is shared with
  `docs/workstreams/action-first-authoring-fearless-refactor-v1/` and
  `docs/workstreams/into-element-surface-fearless-refactor-v1/`,
- item 4 is shared with
  `docs/workstreams/action-first-authoring-fearless-refactor-v1/`.

Audit reconciliation note on 2026-03-16:

- fresh external audits are directionally correct that the remaining pre-release risk is no longer
  kernel/runtime design but public authoring-surface curation,
- specifically, the open pressure is now:
  - shadcn discovery-lane duplication,
  - `fret` root lane budgeting,
  - first-hour/default-path ceremony,
  - and `AppActivateExt` bridge growth pressure,
- the same audits should **not** be read as evidence that the app/component/advanced split itself
  needs another broad redesign pass here:
  - default app-prelude overlap is already down to intentional ergonomics budget plus the shared
    `ui` / `Px` vocabulary,
  - advanced/manual-assembly code that still needs reusable component authoring helpers now imports
    `fret::component::prelude::*` explicitly,
- ecosystem integration-trait follow-on work remains important, but it should resume only after the
  remaining lane-curation blockers below are stable so third-party extension seams align with the
  final public lane story rather than an interim discovery posture.

## Current release-blocking closeout checklist

- [x] Lock `fret-ui-shadcn` to one taught component-family discovery lane:
  `facade as shadcn`, with `raw::*` as the only explicit escape hatch and component-family root
  modules kept crate-private instead of public/doc-hidden.
  - Closed on 2026-03-16:
    - root component-family modules are no longer public,
    - `raw::*` now wraps private component-family modules explicitly,
    - first-party gallery/examples/tests compile and pass on the curated facade + explicit raw
      posture.
- [x] Freeze the `fret` root lane budget so new optional ecosystems only land as explicit
  secondary lanes and do not reopen default-lane autocomplete pressure for ordinary app authors.
  - Closed on 2026-03-16:
    - root-level public modules and direct `pub use` exports are now protected by explicit
      allowlist tests,
    - raw view-runtime helpers moved from `fret::view::*` to `fret::advanced::view::*`,
    - devloop helpers moved from `fret::dev::*` to `fret::advanced::dev::*`.
- [ ] Coordinate the next happy-path ceremony pass across `fret`, `fret-ui-kit`, and first-party
  docs/examples so the first-hour/default todo path gets materially shorter without reopening the
  kernel/mechanism split.
  - Priority targets once lane curation is stable:
    - tracked-value reads,
    - common local/payload write paths,
    - keyed/list/default child-collection ergonomics.
  - 2026-03-16 batch landed:
    - the canonical trio (`simple_todo`, `simple_todo_v2_target`, `todo_demo`) now prefers the
      shorter tracked-read surface `state.layout(cx).value_*` / `state.paint(cx).value_*` over
      `cx.state().watch(&state)...` for ordinary reads,
    - the same trio plus the `fretboard` todo/simple-todo templates now prefer
      `cx.actions().payload_local_update_if::<A, _>(...)` for the common keyed-row payload update
      path,
    - the default-path docs (`docs/examples/todo-app-golden-path.md`,
      `docs/authoring-golden-path-v2.md`) and the first-party source gates were updated to teach
      that shorter wording.
  - 2026-03-16 next batch scope:
    - audit the canonical trio plus the generated templates for remaining keyed/list/default
      child-collection noise (`ui::for_each_keyed(...)`, `ui::children![...]`, focused `*_build`
      seams),
    - prefer already-shipped helpers and tighter teaching copy before inventing any new surface,
    - if a genuinely new helper is still needed, keep it narrow, prove it on the canonical trio,
      and land it through the action-first / into-element follow-on workstreams instead of
      reopening this closeout lane with broad sugar design.
  - 2026-03-16 follow-up landed:
    - `ui::single(cx, child)` now covers the narrow "late-land one typed child" case,
    - canonical first-party roots/wrappers (`hello`, `hello_counter_demo`, `todo_demo`, generated
      todo/simple-todo templates, and their docs/gates) now use that helper instead of
      `ui::children![cx; child].into()`,
    - single-child wrapper closures such as `shadcn::card_content(|cx| ...)` on the canonical
      trio now also use `ui::single(...)` where they were only forwarding one typed subtree.
  - 2026-03-16 cookbook follow-up landed:
    - first-party cookbook examples now also move the obvious single-child
      `shadcn::card_content(|cx| ...)` wrappers onto `ui::single(...)`,
    - this batch intentionally stops at typed-child wrapper closures and does not treat shared
      scaffold page roots (`centered_page*`) as eligible `ui::single(...)` sites, because those
      helpers still return `Elements` rather than a typed child.
- [x] Keep `AppActivateExt` on a shrinking bridge-only path: no new first-party bridge impls for
  widgets that can instead ship native `.action(...)` / `.action_payload(...)` slots.
  - Exit pressure to remove:
    - facade-level bridge growth for widgets that already have a stable action meaning,
    - new first-party docs/snippets that normalize `AppActivateExt` as a default app-lane import,
    - any ambiguity about whether `.dispatch::<A>()` is the preferred path over native widget
      action slots.
  - 2026-03-16 shrink batch landed:
    - `fret::app::AppActivateSurface` no longer forwards shadcn `Badge`,
      `raw::extras::{BannerAction, BannerClose, Ticker}`, or
      `fret_ui_material3::{Card, DialogAction, TopAppBarAction}`,
    - those types already expose native `.action(...)` / `.action_payload(...)`, so they are no
      longer counted as intentional bridge residue.
  - 2026-03-16 `UiCx` grouped-actions follow-up:
    - `fret::app::UiCxActionsExt` now gives extracted helper functions the same grouped
      `cx.actions()` story as `View::render(&mut AppUi)`,
    - UI Gallery snippets for `confirmation_demo`, `conversation_demo`,
      `prompt_input_docs_demo`, and `web_preview_demo` now use `cx.actions().models::<A>(...)`
      plus native widget `.action(...)`,
    - `fret_ui_ai::{ConfirmationAction, ConversationDownload, PromptInputButton, WebPreviewNavigationButton}`
      were removed from the bridge table because those widgets already expose stable action slots.
  - 2026-03-16 widget-native hook clarification:
    - widget-owned `.on_activate(...)` also counts as a bridge-free surface when the component
      already exposes that hook directly,
    - the first-party `apps/fret-ui-gallery/src/ui/snippets/badge/link.rs` example now uses
      `Badge::on_activate(...)` to suppress link launch during diagnostics instead of importing
      `AppActivateExt` for a no-op listener.
  - 2026-03-16 next residue shortlist:
    - closed in the current batch:
      `WorkflowControlsButton`, `MessageAction`, `ArtifactAction`, `ArtifactClose`, and
      `CheckpointTrigger` were removed from the bridge table,
    - first-party UI Gallery snippets now use `UiCxActionsExt` plus widget-owned
      `.on_activate(...)` for those cases instead of importing `AppActivateExt`.
  - 2026-03-16 button/sidebar follow-up:
    - `fret_ui_shadcn::{facade::Button, facade::SidebarMenuButton}` were removed from the bridge
      table,
    - `SidebarMenuButton` now exposes native `.action_payload(...)`,
    - the remaining first-party button/sidebar listener snippets now stay on `UiCxActionsExt`
      plus widget-owned `.on_activate(...)` instead of importing `AppActivateExt`.
  - Closure note (2026-03-16):
    - the first-party default widget bridge table is now intentionally empty in
      `ecosystem/fret/src/view.rs`,
    - remaining bridge usage is the explicit custom/third-party activation-only seam, not an open
      first-party migration backlog.
  - Revalidation rule:
    - keep `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --no-fail-fast`
      green so future first-party bridge growth is caught immediately.
  - 2026-03-16 maintenance rule:
    - if a new first-party widget wants `AppActivateSurface`, treat that as a regression to
      justify against native `.action(...)` / `.action_payload(...)` or widget-owned
      `.on_activate(...)` first.

Post-closeout handoff order on 2026-03-16:

1. run the next happy-path ceremony pass across action-first and conversion follow-ons
2. resume ecosystem integration-trait budgeting once item 1 is stable and the canonical
   trio/templates/docs no longer need wording churn for the default path
3. keep the `AppActivateExt` bridge-empty rule green as a standing gate; reopen it only if a new
   first-party widget truly lacks a native action slot

## M0 — Freeze the target product surface

- [x] Finalize `TARGET_INTERFACE_STATE.md` as the single source of truth for the desired public surface.
- [x] Finalize `MIGRATION_MATRIX.md` as the single execution tracker for old surface removal.
- [x] Decide and lock the canonical names:
  - [x] `FretApp`
  - [x] `App`
  - [x] `KernelApp`
  - [x] `WindowId`
  - [x] `AppUi`
  - [x] `Ui`
- [x] Define the three public surface tiers:
  - [x] app surface
  - [x] component surface
  - [x] advanced surface
- [x] List every public-looking symbol that should be:
  - [x] kept on the app surface
  - [x] moved to the component surface
  - [x] moved to the advanced surface
  - [x] deleted entirely
- [x] Mark the initial status for every migration row:
  - [x] surface lanes
  - [x] ecosystem crates
  - [x] docs/templates/examples
  - [x] hard-delete rows

## M1 — Split the public preludes and imports

- [x] Make `fret::app::prelude::*` the only canonical app import and delete the broad `fret::prelude::*` bridge.
- [x] Add `fret::component::prelude::*`.
- [x] Add explicit advanced import modules under `fret::advanced::*`.
- [x] Remove broad transitive re-export of `fret_ui_kit::declarative::prelude::*` from the app surface.
- [x] Remove broad transitive re-export of `fret_ui_kit::prelude::*` from the advanced prelude convenience lane.
- [x] Stop forwarding the component prelude through `fret::advanced::prelude::*`.
  - 2026-03-16 closeout: advanced/manual-assembly code now imports `fret::component::prelude::*`
    explicitly when it genuinely needs component authoring helpers, instead of rediscovering that
    vocabulary through a hidden advanced-lane umbrella import.
- [x] Remove low-level mechanism types from the default app prelude:
  - [x] `AppWindowId`
  - [x] `Event`
  - [x] `ActionId`
  - [x] `TypedAction`
  - [x] `UiBuilder`
  - [x] `UiPatchTarget`
  - [x] `Length`
  - [x] `SemanticsProps`
  - [x] `HoverRegionProps`
  - [x] `ContainerQueryHysteresis`
  - [x] `ViewportQueryHysteresis`
  - [x] `ImageMetadata`
  - [x] `ImageMetadataStore`
  - [x] `ImageSamplingExt`
  - [x] `MarginEdge`
  - [x] `OverrideSlot`
  - [x] `WidgetState`
  - [x] `WidgetStateProperty`
  - [x] `WidgetStates`
  - [x] `merge_override_slot`
  - [x] `merge_slot`
  - [x] `resolve_override_slot`
  - [x] `resolve_override_slot_opt`
  - [x] `resolve_override_slot_opt_with`
  - [x] `resolve_override_slot_with`
  - [x] `resolve_slot`
  - [x] `ColorFallback`
  - [x] `SignedMetricRef`
  - [x] `Corners4`
  - [x] `Edges4`
  - [x] `ViewportOrientation`
  - [x] `ElementContext`
  - [x] `UiTree`
  - [x] `UiServices`
  - [x] `UiHost`
  - [x] `AnyElement`
  - [x] other runner/maintainer-only types
  - 2026-03-16 closeout: no known runner/maintainer-only types remain on the default
    `fret::app::prelude::*`; remaining prelude/export cleanup now lives on the component and
    advanced lanes rather than the app lane.
- [x] Remove component-author overlap from `fret::app::prelude::*`.
  - Goal: an ordinary app author should not discover the same style/layout/semantics helper
    families from both `fret::app::prelude::*` and `fret::component::prelude::*`.
  - [x] First batch on 2026-03-15: move overlap-heavy extension traits (`TrackedStateExt`,
    `StyledExt`, `UiExt`, `AnyElementSemanticsExt`, `ElementContextThemeExt`,
    `UiElementA11yExt`, `UiElementKeyContextExt`, `UiElementTestIdExt`) to anonymous app-prelude
    imports so their methods remain usable without turning the trait names into default app-lane
    vocabulary.
  - [x] First batch on 2026-03-15: remove raw `on_activate`, `on_activate_notify`,
    `on_activate_request_redraw`, and `on_activate_request_redraw_notify` free-function exports
    from `fret::app::prelude::*`; the default app lane now teaches widget-local
    `.action(...)` / `.action_payload(...)` / `.listen(...)` instead, with
    `.dispatch::<A>()` / `.dispatch_payload::<A>(...)` retained as explicit aliases.
  - [x] Second batch on 2026-03-15: move high-frequency icon/style nouns off
    `fret::app::prelude::*` into explicit `fret::icons::{icon, IconId}` and `fret::style::{Theme,
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, ShadowPreset, Size, Space}`
    lanes, then migrate first-party cookbook/examples/templates/readmes to those explicit imports.
  - [x] Third batch on 2026-03-15: move adaptive declarative helpers (`tailwind`,
    container/viewport breakpoint helpers, safe-area/pointer/media preference helpers, related
    inset probes) off `fret::app::prelude::*` into an explicit `fret::env::{...}` lane.
  - [x] Fourth batch on 2026-03-15: remove zero-usage anonymous overlap traits
    (`ElementCommandGatingExt`, `ElementContextThemeExt`, `UiElementKeyContextExt`) from
    `fret::app::prelude::*`; those capabilities now stay on component/advanced or explicit direct
    imports instead of the default app lane.
  - [x] Fifth batch on 2026-03-15: audit the remaining anonymous semantics/a11y/test-id helpers
    (`AnyElementSemanticsExt`, `UiElementA11yExt`, `UiElementTestIdExt`) and keep them on the
    default app lane intentionally. The trait names stay hidden, but `.role(...)`,
    `.a11y_role(...)`, and `.test_id(...)` remain high-frequency first-party app/gallery
    capabilities and are therefore treated as app-justified helper affordances rather than more
    prelude debt to delete.
  - [x] Sixth batch on 2026-03-15: remove `CommandId` from
    `fret::component::prelude::*` and point reusable component code at explicit
    `fret::actions::CommandId` / `fret-runtime` imports instead.
  - [x] Seventh batch on 2026-03-15: move `SemanticsRole` off `fret::app::prelude::*` into an
    explicit `fret::semantics::SemanticsRole` lane while keeping the `.role(...)` /
    `.a11y_role(...)` helper methods on the default app lane.
  - [x] Eighth batch on 2026-03-15: audit `Px` and keep it intentionally on both app and
    component preludes as the shared low-friction unit type for everyday Fret authoring.
  - [x] Ninth batch on 2026-03-15: remove `actions` / `workspace_menu` module re-exports from
    `fret::app::prelude::*`; app code now reaches those heavier module surfaces through explicit
    `fret::actions::*` / `fret::workspace_menu::*` lanes instead of first-contact wildcard imports.
  - [x] Tenth batch on 2026-03-15: remove `UiElementSinkExt as _` from
    `fret::app::prelude::*`; sink-style `*_build(|cx, out| ...)` composition now requires an
    explicit `use fret::children::UiElementSinkExt as _;` import in the small set of app examples
    that intentionally opt into manual child pipelines.
  - [x] Eleventh batch on 2026-03-15: keep command-availability reads off the default app
    prelude; code that intentionally calls `cx.action_is_enabled(...)` now imports
    `use fret::actions::ElementCommandGatingExt as _;` explicitly instead of rediscovering command
    gating through first-contact wildcard imports.
  - [x] Twelfth batch on 2026-03-15: move `CommandId`, `ThemeSnapshot`, and `LocalState` off
    `fret::app::prelude::*` into explicit `fret::actions::CommandId`,
    `fret::style::ThemeSnapshot`, and `fret::app::LocalState` lanes, and mirror the same
    autocomplete-tightening rule on `fret::component::prelude::*` by keeping overlap-heavy helper
    traits as anonymous `as _` imports instead of named exports.
  - [x] Thirteenth batch on 2026-03-15: keep `AppActivateExt` usable on the default app lane only
    as an anonymous helper import; code that intentionally names the activation-widget contract now
    goes through the explicit `fret::app::{AppActivateSurface, AppActivateExt}` lane instead of
    teaching `AppActivateExt` as first-contact autocomplete vocabulary.
  - [x] Fourteenth batch on 2026-03-15: close the remaining UI Gallery source-shape fallout from
    that prelude/activation cleanup. Material 3 dialog copyable roots now stay on the typed
    `UiCx`/`UiChild` lane, routed `render_doc_page(...)` pages keep an explicit final
    `vec![body.into_element(cx)]` landing line, and
    `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --no-fail-fast`
    now passes again after the default app surface tightening.
  - [x] Sixteenth batch on 2026-03-16: move `AppActivateExt` fully off
    `fret::app::prelude::*`. Activation-only bridge call sites now import
    `use fret::app::AppActivateExt as _;` explicitly, while ordinary action-capable snippets such
    as `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs` stay on native widget
    `.action(...)` slots without the bridge import.
  - [x] Fifteenth batch on 2026-03-15: narrow the component prelude's overlay vocabulary. The
    default component lane now keeps only `OverlayController`, `OverlayRequest`, and
    `OverlayPresence`; overlay anchoring helpers and stack/introspection nouns moved to explicit
    `fret::overlay::*` imports so reusable component authors do not meet runtime-ish overlay terms
    via wildcard autocomplete.
  - Minimum audit set:
    - semantics/test-id/key-context helper families that are still duplicated across app and
      component preludes without an app-specific justification,
    - raw `on_activate*` helper exports that now compete with the grouped app-facing
      `cx.actions().dispatch/listener` story.
    - any remaining app-prelude nouns that still belong better on explicit secondary lanes rather
      than the default autocomplete surface.
    - small-app authoring sugar that is better solved by action-first widget aliases than by more
      prelude surgery (for example command-shaped submit/cancel setters on default-facing text
      inputs).
  - Exit condition: the app prelude teaches the app nouns plus a small set of app-justified helper
    traits, while reusable component plumbing remains discoverable through the component lane.
- [x] Update crate-level docs to teach the new split.
  - README todo taste example now imports `Space` explicitly from `fret::style`.
  - `ecosystem/fret/README.md` calls out `fret::style::{...}`, `fret::icons::{icon, IconId}`, and
    `fret::env::{...}` as the explicit secondary app lanes.
  - `docs/crate-usage-guide.md` now teaches style/icon nouns as explicit imports rather than part
    of `fret::app::prelude::*`, and points adaptive helpers at `fret::env::{...}`.
  - `docs/crate-usage-guide.md` now also teaches reusable component authors to import
    `fret::actions::CommandId` explicitly instead of expecting it from
    `fret::component::prelude::*`.
  - `docs/crate-usage-guide.md` and `docs/component-author-guide.md` now also teach
    environment/responsive helpers as an explicit `fret::env::{...}` lane instead of part of
    `fret::component::prelude::*`.
  - `docs/crate-usage-guide.md` and `docs/component-author-guide.md` now also teach raw
    activation helper glue as an explicit `fret::activate::{...}` lane instead of part of
    `fret::component::prelude::*`.
  - `ecosystem/fret/README.md` and `docs/crate-usage-guide.md` now teach
    `fret::semantics::SemanticsRole` as the explicit app-facing semantic-role lane.
  - `TARGET_INTERFACE_STATE.md` now records `Px` as an intentional shared primitive instead of
    treating it as unresolved prelude overlap debt.

## M2 — Reset the app authoring API

- [x] Introduce grouped app-facing context namespaces:
  - [x] `state()`
  - [x] `actions()`
  - [x] `data()`
  - [x] `effects()`
- [x] Add the new default operations:
  - [x] local state creation/init
  - [x] local state watch/read
  - [x] default local transactions
  - [x] payload-local handlers
  - [x] transient action helpers
  - [x] selector/query integration points
- [x] Rename or replace flat helpers that are no longer part of the blessed path.
- [x] Remove redundant first-contact aliases from the app surface.
- 2026-03-16 closeout: M2 is functionally closed.
  The grouped app model (`state/actions/data/effects`) is the shipped first-party posture, and
  remaining authoring ergonomics work should be tracked as follow-on sugar/doc cleanup rather than
  as unfinished app-surface reset work.

## M3 — Migrate first-party ecosystems to the new surface

- [x] Migrate `fret-ui-shadcn` to the component surface + explicit optional app integration seams.
  - [x] Move app integration helpers under `shadcn::app::*` instead of the recipe root.
  - [x] Move environment / `UiServices` hooks off the default app lane and keep them explicit via
    `fret_ui_shadcn::advanced::*` (or `fret::shadcn::raw::advanced::*` from the `fret` facade).
  - [x] Move first-party advanced cookbook examples to `shadcn::app::install`.
  - [x] Replace the broad `fret::shadcn` whole-crate re-export with a curated facade
    (`shadcn::{..., app, themes, raw}`).
  - [x] Migrate first-party direct-crate examples to `fret_ui_shadcn::{facade as shadcn, prelude::*}`
    and require raw-only helpers to flow through `shadcn::raw::*`.
  - [x] Classify the first-party raw escape hatches and gate them to the documented set
    (`typography`, `extras`, breadcrumb primitives, low-level icon helpers, advanced/raw prelude
    seams where explicitly justified).
  - [x] Add a source gate that forbids first-party curated examples from drifting back to
    `use fret_ui_shadcn as shadcn;`, `shadcn::shadcn_themes::*`, or root
    `shadcn::typography::*`.
  - [x] Audit remaining first-party docs/examples for root-level shadcn app-install teaching.
  - [x] Reduce first-contact shadcn discovery to one taught lane.
    - Goal: `use fret_ui_shadcn::{facade as shadcn, prelude::*};` is the only default first-contact
      story, while component-family crate-root exports stay deleted and `shadcn::raw::*` remains
      explicit.
    - This is not just a docs issue: if first-party tests must keep forbidding alternative import
      paths, the public surface still needs more self-constraint.
    - Exit condition: docs and status docs stop talking about crate root / facade as peer teaching
      lanes, and the remaining root-level exposure is explicitly classified as raw or narrow glue
      residue.
    - 2026-03-15 progress: the curated `prelude`, raw seam doctests, crate-internal recipe/helper
      glue, crate-local tests, and first-party workspace call sites are now off hidden flat root
      component exports; the component-family and direction-utility delete passes have landed and
      the remaining root lane is limited to non-component glue residue.
    - 2026-03-15 follow-up: first-party UI Gallery snippet/page surfaces no longer use
      `fret_ui_shadcn::icon::*`, `fret_ui_shadcn::empty::*`, `fret_ui_shadcn::select::*`,
      `fret_ui_shadcn::tabs::*`, or similar flat root/module lanes; gallery authoring now flows
      through `shadcn::*`, `shadcn::raw::*`, or prelude glue only.
    - 2026-03-15 follow-up: after continuing through `fret-ui-ai`, `fret-bootstrap`, and
      `ecosystem/fret`, non-test first-party workspace code no longer contains
      `fret_ui_shadcn::*` flat root/component calls outside explicit `facade::*` / `raw::*` /
      `advanced::*` seams.
    - 2026-03-15 progress: `ecosystem/fret-ui-ai/src/elements/**` now imports shadcn components
      from `fret_ui_shadcn::facade::*` (with `raw::*` retained only for documented escape hatches).
    - 2026-03-15 progress: `ecosystem/fret-ui-ai/tests/shadcn_import_surface.rs` records the
      crate-local source policy, and `tools/gate_fret_ui_ai_curated_shadcn_surfaces.py` now keeps
      the wider first-party workspace on the same curated-lane rule.
    - 2026-03-15 implementation follow-up: `fret-ui-shadcn` root component modules are now
      `#[doc(hidden)]` compatibility residue rather than a peer public discovery lane. The
      intended authoring surface is therefore self-constraining in rustdoc in addition to the
      first-party source-policy tests.
    - 2026-03-16 progress: the historical direct-crate root theme lane has also been removed from
      the public `fret-ui-shadcn` surface; first-party integration tests now use
      `fret_ui_shadcn::facade::themes::*` (or `shadcn::themes::*` through the alias) rather than
      normalizing `fret_ui_shadcn::shadcn_themes::*`.
    - 2026-03-16 progress: the remaining direct-crate root authoring glue is no longer public
      either. `icon`, `decl_style`, `ui`, styling glue, and ui-builder extension traits are now
      available only through explicit `prelude` / `raw` lanes, and crate-internal recipe code no
      longer depends on a root `icon` shim either. First-party gallery/tests forbid drifting back
      to root glue paths such as `fret_ui_shadcn::decl_style`, `fret_ui_shadcn::icon::*`, or root
      `*UiBuilderExt`.
    - 2026-03-15 progress: the repo-level gate is wired into `tools/pre_release.py`, and
      `cargo check -p fret-ui-ai --lib` is green again after fixing the local `mic_selector.rs` /
      `voice_selector.rs` `Vec::new()` inference residue that was masking the import-lane
      closeout.
    - Remaining bounded cleanup after the gallery pass: non-gallery first-party consumers
      is now reduced to selected internal tests/docs strings plus any future crates that reintroduce
      flat root drift.
- [x] Migrate `fret-docking` to the component/advanced split without redefining the app authoring model.
  - [x] Add an explicit `fret::docking` facade module behind a `fret/docking` feature.
  - [x] Move the cookbook docking example to the `fret::docking::*` seam.
  - [x] Move app-facing `fret-examples` docking demos (`docking_demo`, `container_queries_docking_demo`) to the `fret::docking::*` seam.
  - [x] Audit remaining advanced/component call sites and keep direct `fret-docking` imports explicit.
- [x] Migrate `fret-selector` to the grouped app data surface.
  - [x] Initial migration on 2026-03-15: keep `cx.data().selector(...)` as the grouped default
    app story.
  - [x] Closeout on 2026-03-15: move `DepsBuilder` / `DepsSignature` back off
    `fret::app::prelude::*` into the explicit `fret::selector::*` lane so default first-contact
    imports stay smaller.
  - [x] Move default docs/templates/examples to `cx.data().selector(...)`.
  - [x] Follow-on on 2026-03-17 under
    `dataflow-authoring-surface-fearless-refactor-v1`: narrow the LocalState-first happy path to
    `cx.data().selector_layout(...)` while keeping raw `cx.data().selector(...)` explicit for
    shared-model/global-signature work.
  - [x] Audit remaining advanced/component call sites and keep them explicit.
- [x] Migrate `fret-query` to the grouped app data surface.
  - [x] Move default docs/examples to `cx.data().query(...)` / `cx.data().query_async(...)`.
  - [x] Closeout on 2026-03-15: move explicit `QueryKey` / `QueryPolicy` / `QueryState`-style
    nouns onto `fret::query::*` so grouped app data remains the default story without widening
    `fret::app::prelude::*`.
  - [x] Add the grouped `data()` namespace to extracted `UiCx` helpers so helper-heavy examples no
    longer fall back to raw `use_query*`.
  - [x] Add source/doc gates that forbid default teaching text from drifting back to flat query hooks.
  - [x] Audit remaining advanced/component call sites and keep them explicit.
- [x] Migrate `fret-router` to the new explicit app/advanced extension seams.
  - [x] Add an explicit `fret::router` facade module behind a `fret/router` feature.
  - [x] Move the cookbook router example to the `fret::router::*` extension seam.
  - [x] Keep `fret-router-ui` thin and app-owned instead of turning it into a competing default runtime.
  - [x] Audit remaining direct imports of `fret-router` / `fret-router-ui` in first-party app-facing examples and docs.
- [x] Audit first-party ecosystem crates for private or accidental shortcuts that bypass the new public contracts.
  - [x] Explicit app/advanced split crates (`fret-ui-assets`, `fret-icons-lucide`,
    `fret-icons-radix`, `fret-node`, `fret-router-ui`) now gate against root-level shortcut
    re-exports or install helpers that would bypass their documented seams.

## M4 — Migrate docs, templates, and examples

- [x] Update `README.md`.
- [x] Update `docs/README.md`.
- [x] Update `docs/first-hour.md`.
- [x] Update the golden-path todo docs.
- [x] Update scaffold templates in `apps/fretboard`.
- [x] Update official cookbook examples to use the new app surface.
- [x] Move advanced examples to explicit advanced imports when needed.
- [x] Migrate first-party extracted helper teaching snippets to `UiCx` unless they intentionally
  stay generic over `H: UiHost` or define an explicit advanced entry seam.
- [x] Normalize the first-party UI Gallery routed page surface to `UiCx` and add source gates for
  the default app-facing teaching surface.
- [x] Keep the follow-on `into-element-surface-fearless-refactor-v1` tracker linked from repo
  indexes and active workstream docs so conversion-surface cleanup has an explicit owner after the
  app/component/advanced split lands.
- [x] Record that future default-authoring ergonomics work belongs to
  `action-first-authoring-fearless-refactor-v1` rather than reopening this split workstream.
- [x] Finish migrating the remaining first-party UI Gallery internal preview surface to `UiCx`
  before deleting the old `ElementContext<'_, App>` teaching seam.
  - Current bounded remainder on 2026-03-11 after the editor/torture batch: `0 / 92`
    preview-surface files in
    `apps/fret-ui-gallery/src/ui/previews/**`.
  - The remaining cleanup work is deletion/compaction of legacy helpers, not interface migration.
  - 2026-03-11 follow-up cleanup removed the first dead legacy helpers from the gallery
    atoms/components buckets and deleted orphan `gallery/data/table*.rs` preview bridge files.
  - 2026-03-11 follow-up cleanup also started feature-boundary alignment for UI Gallery dev-only
    teaching surfaces (`harness.rs`, `content.rs`, routed dev pages), restoring a green
    `cargo check -p fret-ui-gallery --lib --features gallery-full`.
- [ ] Remove or rewrite examples that still teach superseded patterns.
- [ ] Keep first-party docs/examples/UI Gallery copy aligned with the next-phase target:
  app-facing lanes teach `Ui` / `UiChild`, while reusable generic helpers move to the unified
  component conversion trait once that workstream lands it.
  - 2026-03-15: UI Gallery code examples and helper snippets were normalized away from
    `fret_ui_shadcn::*` flat root/module paths; remaining stale references are bounded to a small
    set of narrative copy strings and non-gallery first-party crates.
  - 2026-03-16: the live app-entry docs no longer describe removed `run_view*` builder methods as
    part of `FretApp`, the golden-path todo example no longer uses the legacy-looking
    `install_app` helper name on the default builder lane, and the async integration guides now
    lead with grouped `cx.data().query_*` helpers. The remaining stale references are now mostly
    historical workstream prose rather than first-contact docs/examples.

## M5 — Delete the old surface

- [x] Remove `run_view::<V>()` / `run_view_with_hooks::<V>(...)` from the default app surface once
  docs/templates/examples and gates all prefer `view::<V>()?.run()`.
- [ ] Remove old default-path names that are no longer canonical.
- [ ] Remove root-level low-level aliases that are no longer part of the default facade vocabulary.
  - [x] 2026-03-12: removed `fret::ActionMeta` / `fret::ActionRegistry`; low-level registry
    access remains explicit under `fret::actions::*`.
  - [x] 2026-03-12: removed `fret::IconRegistry`; raw icon registry access now stays explicit via
    `fret-icons` / `fret-bootstrap` while app-facing icon packs install through `.setup(...::app::install)`.
  - [x] 2026-03-12: removed root `workspace_shell_model*` shortcuts; editor-style workspace shell
    assembly now stays explicit under `fret::workspace_shell::*`.
- [x] Remove flat `AppUi` data/effects helpers that duplicate the grouped `cx.data()` /
  `cx.effects()` surface.
- [x] Remove public flat `AppUi::use_local*` helpers that duplicate the grouped `cx.state()`
  surface while keeping raw `use_state*` as an explicit advanced seam for now.
- [x] Move raw `use_state*` off the default `AppUi` inherent surface and keep it only as an
  explicit advanced trait seam.
- [x] Remove flat `AppUi` action mutation helpers that duplicate the grouped `cx.actions()` surface
  while keeping raw handler registration as an explicit advanced seam.
- [ ] Remove duplicate or ambiguous exports from the app prelude.
- [ ] Remove compatibility-only aliases that survive only for internal inertia.
  - [x] 2026-03-12: removed the `fret/icons-lucide` compatibility feature alias; the canonical
    feature name for the default Lucide pack is now just `icons`.
  - [x] 2026-03-12: removed `FretApp::register_icon_pack(...)`,
    `UiAppBuilder::register_icon_pack(...)`, and `UiAppBuilder::with_lucide_icons()` from the
    default `fret` facade; explicit pack setup now flows through `setup(...::app::install)`.
  - [x] 2026-03-12: removed the root `fret::router::install_app(...)` exception; router setup on
    the default app lane now follows the same `fret::router::app::install(...)` pattern as the
    other ecosystem app installers.
  - [x] 2026-03-12: removed the naked root `fret::run_native_with_compat_driver(...)` entry;
    retained low-level interop now stays on the explicit
    `fret::advanced::interop::run_native_with_compat_driver(...)` path.
  - [x] 2026-03-12: removed the naked root `fret::run_native_with_fn_driver*` helpers; advanced
    runner escape hatches now stay on the explicit `fret::advanced::*` path.
  - [x] 2026-03-12: removed root `fret::kernel::*` / `fret::interop::*` module exports; low-level
    runtime/render/viewport seams now stay on the explicit `fret::advanced::{kernel, interop}`
    lane.
  - [x] 2026-03-17: removed the raw `fret::query::ui` and `fret::router::ui` passthrough lanes
    from the `fret` facade and narrowed `fret::selector::ui` to the explicitly documented
    `DepsBuilder` export; reusable/advanced code that needs raw UI adoption traits keeps using the
    direct crates (`fret-query`, `fret-selector`, `fret-router-ui`) instead of routing those seams
    back through `fret`.
- [ ] Remove dead docs and stale guidance after the migration is complete.
  - 2026-03-16 bounded remainder: default-facing docs/examples are now largely aligned; the main
    remaining stale guidance is historical workstream prose and closeout bookkeeping, not the
    shipped first-contact surface.

## M6 — Add gates so the surface stays clean

- [x] Add a gate that checks the app prelude stays app-only.
- [x] Add a gate that checks low-level mechanism types do not leak into the app prelude.
- [x] Add a gate that templates only use blessed app-surface APIs.
- [x] Add source gates that keep default docs/examples/templates on `view::<V>()?.run()`.
- [x] Add a gate that README/docs/first-hour agree on the default action model.
- [x] Add source gates that keep default selector/query teaching on grouped `cx.data()` helpers.
- [x] Add a gate that keeps `.setup(...)` on named installers/tuples/bundles and reserves inline
  closures for `setup_with(...)`.
  - Landed on 2026-03-12 in `ecosystem/fret` authoring-surface policy tests and the first-party
    source-policy tests for `apps/fret-examples` and `apps/fret-cookbook`.
- [x] Add a source gate that keeps default extracted helper teaching on `UiCx` instead of raw
  `ElementContext`.
- [x] Add focused UI Gallery source gates for the first migrated teaching surfaces:
  routed pages, gallery shell helpers, the retired Material 3 surface, Magic previews, and
  component preview modules.
- [x] Extend the internal preview gates to cover the first harness-shell batch.
- [x] Extend the internal preview gates to cover gallery atoms/forms/data/overlays.
- [x] Extend the internal preview gates to cover the remaining harness/editor/torture preview buckets.
  - On 2026-03-11 these UI Gallery gates were moved out of `apps/fret-ui-gallery/src/lib.rs` into
    dedicated integration tests under `apps/fret-ui-gallery/tests/ui_authoring_surface_*.rs` to
    reduce merge conflicts on the crate entry file.
- [x] Add a gate that first-party ecosystem crates use documented extension seams.
  - [x] Shadcn docs/examples now gate the curated `shadcn::app::*` seam, explicit advanced hooks,
    and the documented raw escape-hatch lanes.
  - [x] `fret-ui-ai` now also gates curated shadcn imports through both
    `ecosystem/fret-ui-ai/tests/shadcn_import_surface.rs` and the repo-level
    `tools/gate_fret_ui_ai_curated_shadcn_surfaces.py` check that runs in `tools/pre_release.py`.
  - [x] Router cookbook/docs now gate the `fret::router::*` seam.
  - [x] `fret-router-ui` now gates its thin adoption-layer posture and forbids growing a second
    app runtime surface.
  - [x] Docking cookbook/docs now gate the `fret::docking::*` seam.
  - [x] Selector/query docs, templates, and helper-heavy examples now gate grouped
    `cx.data().selector/query*` teaching while keeping raw hook entry explicit to advanced or
    component surfaces.
  - [x] Optional split ecosystem crates (`fret-ui-assets`, icon packs, `fret-node`) now gate
    against root-level app/advanced shortcut re-exports that would bypass their explicit seams.
- [ ] Keep layering checks green.
  - 2026-03-12: `python3 tools/check_layering.py` passed after the split-ecosystem shortcut audit
    guards landed.
  - 2026-03-12: keep the `fret` root facade free of low-level action registry aliases; the source
    gate now requires those names to stay under `fret::actions::*` instead.
  - 2026-03-12: keep icon registry / icon-pack builder helpers off the default `fret` facade; the
    source gates now require app-facing icon setup to stay on `.setup(...::app::install)`.
  - 2026-03-12: keep router app wiring on `fret::router::app::install(...)`; the source/docs
    gates now forbid `fret::router::install_app(...)` from returning on the default lane.
  - 2026-03-12: keep the `fret` feature surface on canonical names only; the source gate now
    forbids the old `icons-lucide = ["icons"]` alias from returning.
  - 2026-03-12: keep workspace-shell helpers module-scoped; the source gate now forbids root
    `workspace_shell_model*` shortcuts from returning.

## Exit Criteria

- [ ] A new user can write a small app without seeing low-level mechanism types.
- [ ] A component author can identify the reusable surface quickly and confidently.
- [ ] First-party ecosystems share one authoring vocabulary.
- [ ] The old broad surface is actually removed.
- [ ] The new surface is guarded by tests/scripts, not just prose.
