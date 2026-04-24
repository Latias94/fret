# ImUi Item Behavior Kernel v1

Status: closed execution lane
Last updated: 2026-04-24

Status note (2026-04-24): this lane closed after the private full pressable item kernel migrated
button, checkbox/radio, selectable, and combo trigger behavior. Active-only switch/menu/tab cleanup
and slider value-editing cleanup should start as narrower follow-ons instead of reopening this
design.

## Why This Lane Exists

Fret's immediate-mode stack has already closed the broad stack reset, facade modularization, and
control-chrome rewrite lanes. The remaining Dear ImGui-class pressure is no longer "add more
helpers to `fret-imui`"; it is the repeated item behavior hidden inside individual control
families.

This lane owns a fearless internal rewrite in `ecosystem/fret-ui-kit::imui`: converge repeated
hover / active / pressed / disabled / focus-visible / response-mapping behavior for item-like
controls into one private behavior kernel, then delete the obsolete family-local duplicates.

The target is inspired by Dear ImGui's `ButtonBehavior`, `ItemAdd`, and item query vocabulary, but
Fret should not copy Dear ImGui's global mutable context model. The kernel must fit Fret's
declarative/runtime split and keep interaction policy in the ecosystem layer.

## Assumptions First

- Area: lane ownership
  - Assumption: this is a new narrow follow-on, not a reopening of the facade modularization or
    control-chrome lanes.
  - Evidence: `docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md`,
    `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`.
  - Confidence: Confident
  - Consequence if wrong: work could be attached to closed historical folders and blur ownership.

- Area: owning crate
  - Assumption: the first implementation owner is `ecosystem/fret-ui-kit::imui`, not
    `ecosystem/fret-imui`.
  - Evidence: the current gap is duplicated policy/helper behavior across kit control families;
    `fret-imui` remains the thin facade surface.
  - Confidence: Confident
  - Consequence if wrong: the public facade could become a Dear ImGui clone before the internal
    behavior model is coherent.

- Area: refactor posture
  - Assumption: compatibility fallback is not a goal inside this lane. If a family-local helper is
    obsolete after the kernel lands, delete it instead of keeping a parallel path.
  - Evidence: the lane is explicitly opened for fearless refactor and follows the repo's closed-lane
    pattern of narrow ownership plus gates.
  - Confidence: Confident
  - Consequence if wrong: the kernel would become one more abstraction above stale behavior instead
    of simplifying the stack.

- Area: hard contracts
  - Assumption: no `fret-ui`, runtime, or ADR-level hard contract needs to change for the first
    slice.
  - Evidence: current behavior pressure sits in ecosystem policy/control families.
  - Confidence: Likely
  - Consequence if wrong: stop the slice, update the relevant ADR/alignment note, and remove the old
    path rather than hiding the contract change behind compatibility glue.

- Area: adjacent feature gaps
  - Assumption: text input v2, table/tab v2, docking/multi-window parity, and debug draw should stay
    out of this lane unless they only consume the new private kernel.
  - Evidence: existing IMUI workstreams route those concerns through separate owner lanes.
  - Confidence: Likely
  - Consequence if wrong: this folder would become a broad parity bucket and lose reviewable exit
    criteria.

## Scope

In scope:

- Design a private `fret-ui-kit::imui` item-behavior owner for common item-like interactions.
- Migrate button-like controls first, then boolean/selectable, then overlay triggers and slider
  affordances only where the shared behavior is truly identical.
- Delete stale family-local duplicate behavior as each family migrates.
- Keep public API changes out of the first slice unless a current API is provably wrong and the
  correct fix is documented with an ADR/alignment update.
- Preserve or strengthen existing tests and demo proof surfaces while allowing internal rewrites.

Out of scope:

- Turning `fret-imui` into a fat Dear ImGui compatibility layer.
- Adding legacy compatibility aliases or dual behavior paths for old internal helpers.
- Changing `crates/fret-ui` mechanism contracts by default.
- Widening `fret-authoring::Response` by default.
- Shipping text input v2, table/tab v2, docking/multi-window behavior, or debug draw in this lane.
- Copying Dear ImGui label suffix, style stack, or global context semantics as a compatibility
  model.

## Target Shape

The target kernel should be private to `fret-ui-kit::imui` until a second proof surface demonstrates
that a public helper shape is genuinely needed.

Expected responsibilities:

- normalize enabled/disabled state and interactive hit behavior;
- drive hover, active, pressed, held, clicked, and focus-visible transitions through one vocabulary;
- map runtime interaction state into the existing response extension vocabulary;
- integrate with `control_chrome` without making chrome own behavior;
- expose narrow internal inputs so control families can keep their visual and layout policy;
- make deletion obvious: after a family migrates, duplicated family-local behavior should disappear.

Expected non-responsibilities:

- layout sizing, field width, padding, and recipe defaults;
- popup dismissal, focus trap/restore, hover intent, and menu/submenu policy beyond shared trigger
  item behavior;
- global ID allocation or Dear ImGui-compatible label parsing;
- retained tree/runtime mechanism changes.

## Layering Rule

This lane is ecosystem policy work:

- `crates/fret-ui`: mechanism contracts only; no new runtime policy for this lane by default.
- `ecosystem/fret-ui-kit`: private item-behavior kernel and migrated generic control families.
- `ecosystem/fret-ui-editor`: may consume the result later for editor controls, but should not be
  the first owner.
- `ecosystem/fret-docking`: out of scope except as future consumer pressure.
- `ecosystem/fret-imui`: thin facade; no broad API growth from this lane.

## Exit Criteria

- At least the button-like and one second control family use the same private behavior kernel.
- Obsolete family-local duplicate behavior is deleted, not kept as fallback.
- Existing focused `fret-ui-kit` and `fret-imui` tests pass.
- `imui_interaction_showcase_demo` and `imui_editor_proof_demo` still build.
- `tools/check_layering.py`, `git diff --check`, and `tools/check_workstream_catalog.py` pass.
- If a hard contract must change, the relevant ADR/alignment docs are updated in the same slice.
