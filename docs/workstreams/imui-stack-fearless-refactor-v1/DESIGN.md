# imui stack fearless refactor v1 - design

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Dear ImGui: `repo-ref/imgui`
- Zed / GPUI: `repo-ref/zed`, `repo-ref/gpui-component`
- shadcn/ui: `repo-ref/ui`
- Base UI: `repo-ref/base-ui`
- Radix primitives: `repo-ref/primitives`

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

Status: Proposed active workstream

Last updated: 2026-03-26

## Purpose

This workstream resets the in-tree `imui` stack as one unit:

- `ecosystem/fret-imui`
- `ecosystem/fret-ui-kit::imui`
- `ecosystem/fret-ui-editor::imui`

The goal is not another incremental facade extension.
The goal is one coherent, editor-grade, teachable immediate-style authoring story with:

- one canonical API per concept,
- one source of truth per widget/control,
- one clear boundary between mechanism and policy,
- and no compatibility layers carried forward "just in case".

Earlier `imui-authoring-facade-v*` and `imui-ecosystem-facade-v*` notes remain useful historical
references, but this directory is the execution surface for the next stack-level refactor.

## Current assessment

Compared with Dear ImGui, the current direction is broadly correct:

- Fret already treats `imui` as an authoring frontend rather than a second runtime.
- Identity is keyed through the existing runtime model rather than through a separate retained tree.
- Richer editor-facing helpers already exist for floating windows, popups, menus, response queries,
  and disabled scopes.
- The underlying retained substrate still owns focus, IME, overlays, routing, and multi-root
  correctness.

The current problem is not "wrong direction".
The current problem is stack coherence.

Today the stack still has the following structural drift:

- `ecosystem/fret-imui` is minimal in principle, but still carries backward-compatibility feature
  aliases and still concentrates too much code in one file.
- `ecosystem/fret-ui-kit::imui` mixes canonical surface, convenience wrappers, legacy entry points,
  popup/window helpers, response helpers, and layout helpers in one giant module.
- `ecosystem/fret-ui-editor::imui` only adapts a narrow subset of editor controls even though
  `fret-ui-editor` already contains a much broader declarative editor control inventory.
- Several entry points are aliases or near-aliases rather than distinct concepts
  (`begin_disabled` vs `disabled_scope`, `same_line` vs layout helpers, `floating_area_show` vs
  `floating_area_show_ex`, `window_ex` / `window_open_ex`, the legacy floating window path).

So the answer to "is the direction right?" is yes.
The answer to "is the current stack clean and complete enough?" is no.

## Why a reset is needed now

The repository already proved the hard part:

- immediate-style authoring can coexist with the declarative element runtime,
- editor-grade interactions can stay on the retained substrate,
- and declarative editor controls can be reused from an `imui` surface.

The remaining risk is now organizational rather than conceptual:

- duplicate names make the API hard to teach,
- monolithic files make review and ownership harder,
- thin adapters are incomplete, so call sites reach into lower layers directly,
- and compatibility shims keep old shapes alive long after the canonical shape is clear.

If this continues, Fret will not fail because `imui` is impossible.
It will fail because it becomes the kind of ecosystem surface that nobody can confidently simplify.

## Goals

### G1 - Delete compatibility debt instead of hiding it

This workstream explicitly allows a workspace-wide breaking migration.
We will not preserve deprecated aliases, feature aliases, or legacy helper paths if the replacement
surface is already understood.

### G2 - Leave one canonical immediate surface per concept

For each concept, there should be one obvious entry point:

- one disabled-scope API,
- one window API family,
- one floating-area API family,
- one layout vocabulary,
- one response story,
- one adapter story for editor controls.

### G3 - Keep `fret-imui` minimal and boring

`fret-imui` should stay a tiny authoring frontend around:

- `UiWriter`,
- identity scoping,
- output collection,
- minimal layout composition,
- and small convenience glue that is truly frontend-generic.

It must not become a second ecosystem widget layer.

### G4 - Make `fret-ui-editor::imui` the complete thin adapter layer

The editor crate already has the real control inventory.
Its `imui` module should expose a systematic thin adapter surface for that inventory, rather than a
small handful of hand-written helpers.

### G5 - Keep `crates/fret-ui` mechanism-only

This refactor must continue to honor ADR 0028 and ADR 0066:

- no second runtime,
- no editor-policy migration into `crates/fret-ui`,
- no component-default compensation knobs added to runtime contracts to support facade cleanup.

## Non-goals

- Preserving source compatibility for existing in-tree `imui` call sites.
- Keeping deprecated aliases around for a grace period.
- Re-implementing editor control logic inside `fret-ui-editor::imui`.
- Turning `fret-imui` into a component library.
- Copying Dear ImGui's exact API grammar or ID syntax.
- Treating "more wrapper count" as the success metric.

## Non-negotiable boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| `crates/fret-ui` | runtime mechanisms, element tree hosting, layout, focus/capture, overlays, text/input, semantics | editor policy, immediate facade ergonomics, recipe defaults |
| `ecosystem/fret-imui` | minimal immediate-style authoring frontend, keyed identity helpers, output collection, small generic layout helpers | editor widgets, rich policy wrappers, style presets, compatibility baggage |
| `ecosystem/fret-ui-kit::imui` | richer generic facade for optional immediate-style helpers | canonical editor control ownership, legacy alias matrices, second implementations |
| `ecosystem/fret-ui-editor` | declarative editor primitives, controls, composites | runtime policy, docking/shell policy, duplicate immediate implementations |
| `ecosystem/fret-ui-editor::imui` | thin adapter layer over the same declarative editor controls | adapter-local widget logic, proof-only local state machines |

## Decision snapshot

### 1) `fret-imui` remains the minimal frontend

After the reset, `fret-imui` should contain only the smallest authoring contract and helper set that
is worth keeping generic:

- `ImUi`,
- `imui(...)`, `imui_build(...)`, and a small number of layout/identity helpers,
- `UiWriter`-based composition,
- the canonical `Response` re-export,
- feature names that reflect the real contract surface with no alias duplication.

The existing backward-compatible Cargo feature aliases (`query`, `selector`) should be deleted.

### 2) `fret-ui-kit::imui` stays richer, but loses duplicate entry points

`fret-ui-kit::imui` remains the place for richer immediate-style helpers, but the surface is reset to
one canonical entry point per concept.

Planned deletions/consolidations include:

- keep `disabled_scope`, delete `begin_disabled`,
- keep one canonical layout vocabulary and delete alias-only wrappers such as `same_line` and
  `items`,
- collapse `floating_area_show` / `floating_area_show_ex` into one typed canonical family,
- collapse `window_ex` / `window_open_ex` into one typed canonical family,
- delete `floating_window_impl_legacy`,
- delete helpers that only exist to preserve older naming instead of expressing a distinct concept.

The exact final names are less important than the rule:
there must be one obvious API, and the codebase should not carry a second synonym just because it
already exists.

### 3) `fret-ui-editor::imui` becomes systematic coverage, not ad hoc helpers

The editor crate should expose a thin `imui` adapter for the editor starter set, including the
controls that already exist declaratively:

- `ColorEdit`
- `NumericInput`
- `MiniSearchBox`
- `TextAssistField`
- `VecEdit`
- `TransformEdit`
- `AxisDragValue`
- `IconButton`
- plus the already-exposed `TextField`, `Checkbox`, `DragValue`, `Slider`, and `EnumSelect`

If an adapter cannot stay thin, that is a bug in the declarative ownership boundary, not a reason to
grow an adapter-local implementation.

### 4) No dual implementation paths

The refactor keeps the existing "single source of truth per widget/control" rule:

- declarative editor controls stay canonical,
- `imui` adapters delegate,
- facade wrappers may improve ergonomics but must not re-implement canonical state machines.

### 5) Flag-day migration is acceptable

The repository is allowed to perform a workspace-wide breaking migration.
If the new surface is correct, in-tree callers should be migrated atomically instead of forcing the
new design to coexist with the old one.

## Current quality bar vs Dear ImGui

Dear ImGui remains a useful outcome reference for:

- dense editor hand-feel,
- immediate control-flow ergonomics,
- floating window semantics vocabulary,
- popup/menu expectations,
- and "one obvious helper" API design.

This workstream does not aim for source-level Dear ImGui parity.
It aims for the same kind of clarity:

- one obvious way to author a concept,
- dense editor workflows without policy leaking into the runtime,
- and a starter surface that feels complete enough for editor tooling.

Current Fret assessment against that bar:

- mechanism direction: correct,
- immediate/declarative runtime split: correct,
- editor control single-source rule: correct,
- API coherence: not yet acceptable,
- editor `imui` starter-set closure: not yet acceptable.

## Target architecture

### `ecosystem/fret-imui`

Representative target split:

- `src/lib.rs` as a small entry/re-export surface,
- `src/core.rs` for `ImUi` + builder entry points,
- `src/layout.rs` for minimal frontend layout helpers,
- `src/identity.rs` or equivalent for keyed/unkeyed iteration helpers,
- `src/tests/*` for focused behavior coverage instead of a giant mixed file.

The exact filenames can change, but the public outcome should be:

- small crate root,
- minimal surface,
- no test monolith,
- no compatibility aliases.

### `ecosystem/fret-ui-kit::imui`

Representative target split:

- `src/imui/mod.rs`
- `src/imui/response.rs`
- `src/imui/layout.rs`
- `src/imui/floating.rs`
- `src/imui/popup.rs`
- `src/imui/widgets.rs`
- `src/imui/store.rs`
- `src/imui/adapters.rs`

The exact file map is not normative.
The normative outcome is that popup/window/floating/layout/response concerns stop living in one
giant source file.

### `ecosystem/fret-ui-editor::imui`

Representative target split:

- `src/imui/mod.rs`
- `src/imui/fields.rs`
- `src/imui/numeric.rs`
- `src/imui/composites.rs`
- or a similarly small set of adapter-focused modules

The public goal is simple:
editor adapters should be easy to review as a coverage layer over existing declarative controls.

## Deletion policy

This workstream is intentionally incompatible with a "soft migration" mindset.

Rules:

- do not keep deprecated aliases,
- do not keep `_legacy` implementation paths,
- do not keep Cargo feature aliases once canonical names exist,
- do not keep two public APIs that describe the same concept,
- do not keep adapter-local logic that only exists to preserve an older surface.

If a caller breaks, the caller is migrated.
If a helper becomes redundant, it is deleted.

## Regression and proof requirements

The refactor still needs to leave behind evidence, not only cleaner code.

Minimum closure expected from implementation:

- focused tests for identity and response semantics where the surface is renamed or consolidated,
- focused tests for popup/floating/window behavior when legacy paths are deleted,
- focused tests for editor `imui` adapters covering the promoted starter set,
- proof/demo coverage that keeps editor-grade authoring reviewable,
- updated workstream/TODO evidence anchors when the stack shape changes materially.

If `UiWriter` or `Response` contract shape changes materially rather than cosmetically, the relevant
ADR/workstream alignment must be updated instead of being silently folded into refactor code.

## Relationship to earlier workstreams

This workstream supersedes earlier incremental `imui` facade expansion as the execution baseline for
the next refactor pass.

Those earlier notes still matter as references for:

- why `imui` must remain an authoring frontend,
- why `UiWriter` exists,
- where Dear ImGui parity lessons were already captured,
- and why richer facade helpers belong in `fret-ui-kit` instead of `fret-imui`.

But the next code-moving phase should follow this directory's principles:

- delete compatibility debt,
- simplify the stack as one system,
- close the editor adapter story,
- and keep the runtime boundary boring.

## Closeout snapshot (2026-03-27)

This section records what actually survived the fearless refactor, what was deleted outright, and
which naming decisions became the canonical authoring surface.

### What survived

`ecosystem/fret-imui` survived as the minimal authoring frontend:

- `ImUi`
- `imui(...)`
- `imui_build(...)`
- `imui_vstack(...)`
- `Response` re-export
- feature names `state-query`, `state-selector`, and `state`

Evidence:

- `ecosystem/fret-imui/src/lib.rs`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-imui/Cargo.toml`

`ecosystem/fret-ui-kit::imui` survived as the richer generic facade layer, but with the
implementation split by concern:

- `response.rs`
- `options.rs`
- `containers.rs`
- `interaction_runtime.rs`
- `popup_store.rs`
- `popup_overlay.rs`
- `floating_surface.rs`
- `floating_window.rs`
- `text_controls.rs`
- `button_controls.rs`
- `menu_controls.rs`
- `boolean_controls.rs`
- `slider_controls.rs`
- `select_controls.rs`

The surviving surface remains centered on:

- response helpers,
- popup/menu/modal helpers,
- floating/window helpers,
- and generic control wrappers.

Evidence:

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/`

`ecosystem/fret-ui-editor::imui` survived as a thin coverage layer over declarative editor
controls:

- `text_field`
- `checkbox`
- `color_edit`
- `drag_value`
- `axis_drag_value`
- `numeric_input`
- `slider`
- `enum_select`
- `mini_search_box`
- `text_assist_field`
- `icon_button`
- `vec2_edit`
- `vec3_edit`
- `vec4_edit`
- `transform_edit`

Evidence:

- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`

### What was deleted

The refactor deliberately removed compatibility baggage instead of preserving it:

- Cargo feature aliases `query` and `selector`
- `begin_disabled`
- alias-only layout helpers such as `same_line` and `items`
- compatibility entry points `floating_area_show` / `floating_area_show_ex`
- compatibility entry points `window_ex` / `window_open_ex`
- `floating_window_impl_legacy`
- older thin aliases such as `area`

These names should be treated as historical only.

### What became canonical

The intended post-reset ownership story is now:

- `fret-imui` for minimal frontend authoring,
- `fret-ui-kit::imui` for richer generic immediate-style helpers,
- `fret-ui-editor::imui` for thin editor-control adapters.

The clearest canonical outcomes already landed are:

- `disabled_scope(...)` is the surviving disabled-scope helper,
- editor `imui` authoring goes through the thin adapter layer instead of ad hoc lower-level calls,
- `fret-imui` feature names match the real contract surface,
- proof/demo coverage is anchored on `imui_editor_proof_demo`,
- editor adapter coverage is anchored on `imui_adapter_smoke`,
- explicit-options authoring helpers now consistently use `*_with_options(...)`,
- the last non-legacy overlay-root `_ex` contract in `fret-ui` now uses
  `OverlayRootOptions` + `push_overlay_root_with_options(...)`,
- live Rust code in `crates/`, `ecosystem/`, and `apps/` no longer keeps any `*_ex` helper or
  public surface names,
- earlier `imui` workstreams now open with explicit archive framing, current-source-of-truth
  redirects, and canonical name mapping notes,
- top-level navigation docs now route readers to the stack-reset workstream before the archived
  `imui` notes,
- and the public authoring surface no longer mixes generic surviving helpers between
  `*_with_options(...)` and non-legacy `*_ex(...)`.

### Verified closeout outcomes

The following M5 claims are now backed by direct evidence:

- `fret-imui` remains policy-light and dependency-light.
  - It depends on `fret-authoring` and `fret-ui` only, and keeps policy-heavy helpers out of the
    crate.
- `fret-ui-editor::imui` reads as a thin coverage layer.
  - The module is a small set of `add_editor_element(...)` wrappers over existing declarative
    controls.
- The refactor now has a concise proof/gate package.
  - `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke`
  - `cargo check -p fret-demo --bin imui_editor_proof_demo`
  - `cargo nextest run -p fret-imui --lib`

### Closeout status

The public alias debt that motivated this closeout is now closed:

- `window(...)` and `window_with_options(...)` are the surviving public floating-window families,
- `floating_area_with_options(...)` is the surviving explicit-options floating-area entry point,
- `floating_area_drag_surface(...)` is the surviving drag-surface helper,
- `select_model_with_options(...)` is the surviving explicit-options select helper,
- layout/container helpers now use `horizontal_with_options(...)`, `vertical_with_options(...)`,
  `grid_with_options(...)`, and `scroll_with_options(...)`,
- control helpers now use `button_with_options(...)`, `action_button_with_options(...)`,
  `menu_item_with_options(...)`, `menu_item_checkbox_with_options(...)`,
  `menu_item_radio_with_options(...)`, `menu_item_action_with_options(...)`,
- model-backed form helpers now use `input_text_model_with_options(...)`,
  `textarea_model_with_options(...)`, `switch_model_with_options(...)`,
  and `slider_f32_model_with_options(...)`,
- and popup helpers now use `begin_popup_menu_with_options(...)`,
  `begin_popup_modal_with_options(...)`, and `begin_popup_context_menu_with_options(...)`.
- overlay-root installation now uses `push_overlay_root_with_options(...)` when non-default
  hit-test behavior is required, with `OverlayRootOptions` making the mechanism contract explicit.

Older names now fall into one of two categories only:

- deleted public compatibility names kept only in historical workstream notes,
- or private/internal helper names inside split implementation modules.
