# M2 Overlay Feature Boundary - 2026-05-12

Status: Boundary decision

This slice resolves the open P0 question for hover, completion, signature help, and code-action
surfaces. These features need editor-owned request data, but they must compose with Fret's existing
overlay/focus system instead of embedding Radix-like dismissal or popup policy inside
`fret-code-editor`.

## Decision

The code editor owns feature request facts, not overlay policy.

- `fret-code-editor-buffer` owns document identity, revisions, edits, transactions, and
  byte-indexed selections.
- `fret-code-editor-view` owns buffer/display coordinate vocabulary and deterministic projection
  helpers.
- `fret-code-editor` may expose request contexts, anchor facts, payload ids, command ids, and
  diagnostics/perf counters for editor features.
- Component and recipe layers (`fret-ui-kit`, `fret-ui-editor`, `fret-ui-shadcn`, or app-owned
  editor shells) own overlay lifecycle policy and visual composition.

This keeps the editor widget from becoming a second overlay runtime. It also keeps completion,
hover, and code actions compatible with multi-root z-order, command routing, focus restore, and
Radix/APG/Floating-aligned recipes.

## Request Data Vocabulary

Future feature request contexts should be data-first and revision-aware:

- document facts: `DocId`, optional `DocUri`, and `Revision`,
- cursor/selection facts: caret byte, `Selection`, and optional trigger range,
- display facts: `DisplayPoint`, logical line, display row, or materialized display row when a
  feature needs view projection,
- anchor facts: stable editor/row/caret element id or a surface-provided window-space rect after
  layout,
- trigger facts: explicit trigger kind such as keyboard invocation, typed character, pointer hover,
  gutter click, diagnostics affordance, or command palette invocation,
- payload ids: hover id, completion session id, code-action id, diagnostic id, or command id.

The editor should not require every request to have every coordinate form. Buffer byte ranges stay
the common storage contract; display coordinates and anchor geometry are projections.

## Feature Ownership

Completion:

- editor/view layer may expose request context, candidate payload shape, active candidate identity,
  and commit intent vocabulary,
- listbox navigation, pointer hover selection, scroll-into-view, focus handoff, outside dismissal,
  Escape behavior, and placement belong to the component layer,
- commit actions should route through text edit transactions or command/action ids rather than
  mutating arbitrary app state from overlay code.

Hover and signature help:

- editor/view layer may expose hovered range, request context, payload id, and anchor facts,
- the overlay should not steal text focus by default,
- hover delay, hover-intent safe areas, pointer dismissal, Escape dismissal, and rich content
  composition belong to `fret-ui-kit` / recipe layers.

Code actions:

- editor/view layer may expose range/context data, related diagnostic ids, and command ids,
- gutter lightbulbs, inline affordances, context menus, and popovers are recipe/app-owned UI,
- executing an action should go through command routing or an app-owned edit service, not through a
  hidden editor-global action stack.

## Policy That Must Stay Out of `fret-code-editor`

Do not put these policies in the code editor crate:

- choosing modal vs non-modal overlay roots,
- click-outside, focus-outside, Escape, window-focus-lost, and resize dismissal,
- focus trap, initial focus, focus restore, and branch-subtree dismissal exceptions,
- overlay placement, flip, shift, size, offset, arrow, and collision policy,
- hover intent, pointer safe polygons, open/close delays, and suppression after dismiss,
- combobox/listbox keyboard navigation and typeahead policy,
- shadcn/Radix default padding, row height, icon, color, or animation recipes.

`fret-code-editor` can expose enough facts for those policies to be correct, but the policy owner is
the ecosystem component or the app shell that builds the feature surface.

## IME, Read-Only, and Disabled Rules

- Read-only editors may still request hover, signature help, diagnostics, and code actions that do
  not mutate the buffer.
- Completion commit and code actions that edit the document must respect the same read-only and
  disabled gates as direct text edits.
- While inline preedit is active, completion and code-action UI may be visible, but mutation commits
  must route through the text-input/edit transaction boundary. Overlay code should not replace
  marked text directly.
- Disabled editors should not open new editor-owned feature requests; existing app-owned overlays
  may close through their normal overlay policy.

## Performance and Diagnostics

This boundary is not a license for broad paint-path rewrites. Before widening feature-heavy editor
surfaces, add counters and gates that explain:

- active feature sessions,
- candidate/item counts,
- visible overlay roots or requests,
- feature payload bytes/rows where relevant,
- whether decorations/semantic tokens/diagnostics changed the renderer payload.

The first UI proof should compare against the existing complex editor wheel, autoscroll, resize,
and renderer-payload contracts before changing thresholds.

## Evidence Anchors

- Multi-root overlay ownership: `docs/adr/0011-overlays-and-multi-root.md`
- Focus and command routing: `docs/adr/0020-focus-and-command-routing.md`
- UI behavior reference stack: `docs/reference-stack-ui-behavior.md`
- Overlay infrastructure: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- Existing anchored editor assist recipe:
  `ecosystem/fret-ui-editor/src/controls/text_assist_field.rs`
- Existing editor select overlay policy:
  `ecosystem/fret-ui-editor/src/controls/enum_select.rs`
- Existing Radix-like dismissal/focus gates:
  `ecosystem/fret-ui-shadcn/tests/*escape_dismiss_focus_restore.rs`

## Follow-Up

1. Turn the target vocabulary into concrete public structs only after the first combined editor UI
   proof needs them.
2. Add a UI Gallery or example proof with syntax, diagnostics/decorations, gutter markers,
   folds/inlays, soft wrap, selection, and at least one overlay-style feature hook.
3. Add diagnostics bundle counters for feature payloads before promoting feature-heavy performance
   baselines.
