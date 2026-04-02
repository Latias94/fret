# shadcn recipe focus and builder render closure v1 - design

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui v4: `repo-ref/ui`
- Zed / GPUI: `repo-ref/zed`, `repo-ref/gpui-component`
- Base UI: `repo-ref/base-ui`
- Radix primitives: `repo-ref/primitives`

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

Status: Active maintenance lane

Last updated: 2026-04-01

## Purpose

This workstream closes two recipe-layer drift classes that surfaced while aligning the shipped Todo
demo with the current shadcn reference:

1. text-entry chrome keyed too narrowly to `focus-visible`, which made pointer-focused inputs look
   inactive even though they were the active editing target, and
2. builder-based recipes probe-rendering the same child subtree twice in one frame, which can
   collide with local state callsites and emit `use_state called multiple times per frame`
   warnings.

The goal is not to paper over the Todo demo.
The goal is to lock the underlying recipe rules so future examples stop re-discovering the same
bugs.

## Problem statement

### Text-entry chrome drift

Fret's runtime `focus-visible` heuristic is correct as a mechanism-level concept: keyboard
navigation should decide when keyboard-only affordances appear.

The issue is narrower:

- text-entry controls (`Input`, `Textarea`, `InputGroup`-owned text fields) are not merely
  "focusable buttons",
- pointer focus is itself a meaningful active-editing state,
- and the shadcn outcomes we are matching keep the editing chrome visually active while the field is
  focused.

When recipe chrome keyed border/ring animation to `focus-visible` alone, pointer-focused text-entry
controls looked unfocused and the Todo demo regressed relative to the user's HTML/shadcn reference.

### Builder double-render drift

Some recipe surfaces accept child builders and then try to infer contextual state by probe-rendering
that builder before the real render.

That pattern is not safe in Fret's current authoring/runtime model:

- the same builder subtree can run twice in one frame,
- local state hooks inside the subtree can collide at the same callsite,
- and otherwise valid content starts emitting runtime warnings or duplicating work.

The `SidebarMenuItem::into_element_with_children(...)` path demonstrated the problem clearly.

## Goals

### G1 - Close the text-entry active chrome contract

For text-entry recipes that own outer border/ring chrome, the active border/ring state must follow
`focused`, not `focus_visible`.

### G2 - Keep keyboard-only affordances where they belong

Do not widen this rule into a blanket "all controls use focused".
Button-like, selector-like, and command-like controls still keep keyboard-only focus affordances on
`focus-visible`.

### G3 - Ban same-frame probe renders of the same builder subtree

Recipe authors should derive hover/focus/open context from the real rendered root, explicit state,
or API restructuring, not by rendering the same builder twice in one frame.

### G4 - Lock the outcome with small, durable gates

The contract should be proven on:

- focused unit tests for the affected recipe surfaces, and
- the Todo demo as a first-party proof surface, because it exposed the drift in a user-visible way.

## Non-goals

- Changing the runtime `focus-visible` heuristic in `crates/fret-ui`.
- Reinterpreting ADR 0061 as "never show active chrome on pointer-focused text fields".
- Widening the text-entry exception to `Select`, `NativeSelect`, `Checkbox`, `RadioGroup`,
  `Slider`, or other non-text-entry controls without fresh parity evidence.
- Adding Todo-demo-only hacks that bypass the underlying recipe issue.
- Introducing new mechanism knobs in `crates/fret-ui` just to compensate for recipe drift.

## Decision snapshot

### 1) Text-entry recipe chrome uses `focused`

For text-entry controls that own their outer active chrome, the recipe-level border/ring transition
follows `cx.is_focused_element(...)`.

Current landed surfaces:

- `ecosystem/fret-ui-shadcn/src/input.rs`
- `ecosystem/fret-ui-shadcn/src/textarea.rs`
- `ecosystem/fret-ui-shadcn/src/input_group.rs`

This keeps click-focused text fields visually active, including the active border and ring tween.

### 2) `focus-visible` remains the rule for button-like and selector-like controls

This workstream does not change the broader runtime or recipe rule that keyboard-only focus
affordances should stay on `focus-visible` for controls such as buttons, toggles, radios, checkboxes,
select triggers, and sliders.

The point is not "prefer focused everywhere".
The point is to recognize that text-entry is a different interaction category.

### 3) Same-frame builder probe renders are forbidden

Recipe code must not render the same builder subtree twice in a frame just to derive contextual
state such as `focus_within`.

Preferred alternatives:

- derive state from the actual rendered root (`cx.root_id()` / focus-within queries),
- pass explicit state through the API or context,
- or restructure the surface so the needed state is known without speculative rendering.

Current landed proof:

- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
  - `SidebarMenuItem::into_element_with_children(...)` now derives `focus_within` from the menu
    item root instead of probe-rendering the builder subtree twice.

## Layering and contract ownership

| Layer | Owns | Must not own |
| --- | --- | --- |
| `crates/fret-ui` | focus tree, focus-visible heuristic, ring painting primitives, hook/runtime safety invariants | shadcn policy for which surfaces treat pointer focus as active chrome |
| `ecosystem/fret-ui-kit` | reusable style/state primitives, recipe helpers, composition guardrails | Todo-demo-specific fixes, shadcn-only surface policy drift hidden as runtime behavior |
| `ecosystem/fret-ui-shadcn` | concrete shadcn recipe policy for text-entry chrome and builder composition outcomes | mechanism changes, duplicate builder execution hacks |
| `apps/fret-examples` | proof surfaces that reveal recipe drift early | recipe-only hacks that bypass shared components |

## Relationship to existing ADRs

- ADR 0061 still owns the runtime `focus-visible` heuristic and focus-ring primitive.
- ADR 0219 still correctly models `focused` and `focus_visible` as distinct states.
- This workstream records a recipe-level policy rule:
  - text-entry active chrome uses `focused`,
  - keyboard-only focus affordances for non-text-entry controls stay on `focus_visible`.

No new ADR is required for this lane because the mechanism/runtime contract did not change.
The change is a recipe classification and authoring-discipline closeout inside the ecosystem layer.

## Landed scope and evidence anchors

Implementation anchors:

- `ecosystem/fret-ui-shadcn/src/input.rs`
- `ecosystem/fret-ui-shadcn/src/textarea.rs`
- `ecosystem/fret-ui-shadcn/src/input_group.rs`
- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
- `apps/fret-examples/src/todo_demo.rs`

Focused regression gates:

- `cargo test -p fret-ui-shadcn input_focus_ring_tweens_in_and_out_like_a_transition --lib`
- `cargo test -p fret-ui-shadcn textarea_focus_ring_tweens_in_and_out_like_a_transition --lib`
- `cargo test -p fret-ui-shadcn input_group_focus_ring_tweens_in_and_out_like_a_transition --lib`
- `cargo test -p fret-ui-shadcn sidebar_menu_item_children_builder_runs_once_per_frame --lib`
- `cargo test -p fret-ui-shadcn sidebar_menu_action_show_on_hover_hides_until_item_hovered_on_desktop --lib`
- `cargo test -p fret-ui-shadcn sidebar_menu_action_show_on_hover_visible_when_menu_item_focus_within --lib`
- `cargo test -p fret-examples todo_demo_registers_vendor_icons_used_by_layout --lib`

Todo proof-surface gates:

- `tools/diag-scripts/tooling/todo/todo-baseline.json`
- `tools/diag-scripts/tooling/todo/todo-shortcuts-screenshot.json`

## What this workstream intentionally leaves open

- auditing any remaining text-entry wrappers that own outer chrome but were not part of this April
  2026 landing slice,
- documenting a short review checklist for recipe authors so builder double-render hazards are
  caught earlier in code review,
- and only adding new gates when a concrete parity regression appears.

This lane should remain narrow.
If future work reveals a broader state-resolution problem, open a separate contract or recipe lane
instead of silently widening these rules.
