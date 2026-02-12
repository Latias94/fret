# ADR 0261: Platform Text Input Client Interop (v1)

Status: Proposed

## Context

ADR 0012 defines Fret’s **input event model** (`KeyDown` vs `TextInput` vs `ImeEvent`) and the
preedit-first arbitration rules required for editor-grade IME behavior.

However, mobile platforms (and some desktop integrations such as macOS `NSTextInputClient`) require
an additional integration seam:

- the platform asks the focused text input for **selection / marked ranges**,
- requests **text excerpts**,
- requests **bounds for ranges** (caret / selection geometry),
- and applies edits by requesting **replacement** in a specified range.

If this seam is not defined early, higher layers will accumulate ad-hoc platform branches that are
hard to unwind later.

This ADR defines a **data-only** and **query-driven** runtime contract that lets runners implement
platform-native “text input client” APIs without leaking platform details into `crates/fret-ui`
(ADR 0066).

## Goals

1. Provide a stable, portable “platform text input client” seam for:
   - IME (composition + candidate placement),
   - selection handles / OS-driven selection (future),
   - accessibility text fields (aligned with ADR 0033).
2. Keep `crates/fret-ui` mechanism-only: widgets expose state/geometry, runners do platform glue.
3. Use UTF-16 code units for platform-facing indices (to match iOS/Android/web norms) while keeping
   existing widget editing indices UTF-8 (ADR 0071).
4. Make keyboard show/hide and caret anchoring expressible via existing effects (ADR 0012) with
   mobile-friendly semantics.

## Non-goals (v1)

- A full mobile selection-handles UI/gesture policy (policy belongs in ecosystem crates).
- A complete “autofill” contract surface.
- A guarantee that all platforms can position the IME candidate window perfectly.

## Decision

### D1 — Define a platform-facing snapshot and query surface (runtime-owned)

The runtime publishes a per-window `WindowTextInputSnapshot` after paint, and runners may read it
after render to drive platform APIs that need final geometry.

The snapshot contract is defined by:

- `crates/fret-runtime/src/window_text_input_snapshot.rs` (`WindowTextInputSnapshot`)

Normative fields (v1):

- `focus_is_text_input: bool`
- `is_composing: bool`
- `text_len_utf16: u32` (length of the **composed view**, in UTF-16 code units)
- `selection_utf16: Option<(u32, u32)>` (anchor/focus, UTF-16)
- `marked_utf16: Option<(u32, u32)>` (marked/preedit range, UTF-16)
- `ime_cursor_area: Option<Rect>` (window logical coordinates; ADR 0012)

The runtime also exposes a focused-widget query surface for platform-style requests:

- `crates/fret-runtime/src/platform_text_input.rs` (`PlatformTextInputQuery`)

The query surface is **window-scoped** and **focus-scoped**:

- Queries target the currently focused text input widget (if any).
- If no text input is focused, all queries return `None` (or their `Option`-wrapped equivalent).

### D2 — Index model: UTF-16 over the composed view

All platform-facing indices in this ADR are UTF-16 code units over the widget’s **composed view**:

- base buffer text, with active IME preedit spliced at the caret (and rendered inline).

This matches existing runtime types:

- `Utf16Range` in `crates/fret-runtime/src/platform_text_input.rs`

Widget-internal editing indices remain UTF-8 byte indices (ADR 0071). Widgets are responsible for
deterministic conversions and clamping at UTF-8 boundaries as needed.

### D3 — Coordinate model for geometry queries

All rectangles returned by the platform text input seam are expressed in **window logical
coordinates** (DIP / logical px), consistent with ADR 0017 and ADR 0012.

Specifically:

- `WindowTextInputSnapshot.ime_cursor_area` is in window logical coordinates.
- `PlatformTextInputQuery::BoundsForRange` returns `Rect` in window logical coordinates.

Runners may convert to physical pixels for OS APIs.

### D4 — Effects: IME allow + caret anchoring are the portable baseline

The portable baseline for platform glue remains the effects in ADR 0012:

- `Effect::ImeAllow { window, enabled }`
- `Effect::ImeSetCursorArea { window, rect }`

Mobile-friendly semantics (v1):

- `ImeAllow(enabled=true)` means:
  - the focused widget is a text input,
  - the runner SHOULD make best effort to show the virtual keyboard when required by the platform
    (Android/iOS), and MUST enable IME/composition when supported.
- `ImeAllow(enabled=false)` means:
  - the runner SHOULD hide the virtual keyboard when appropriate (best-effort),
  - and MUST disable IME/composition routing to the focused text surface.

Rationale: many platforms tie keyboard visibility and composition enablement to the same “text input
client active” concept, even if they expose separate APIs.

### D5 — Platform-driven edits: replace-by-range (UTF-16)

To support platform-native edit application (e.g. Android `InputConnection`, macOS text services),
the runtime provides **range replacement** entry points scoped to the focused text widget:

- `platform_text_input_replace_text_in_range_utf16(range, text) -> bool`
- `platform_text_input_replace_and_mark_text_in_range_utf16(range, text, marked) -> bool`

Contract:

- Replacements are specified in UTF-16 over the composed view.
- The widget MUST clamp and normalize inputs deterministically.
- `replace_and_mark` is used for “set composing text” style operations:
  - the widget updates its preedit state to match `marked`,
  - without violating ADR 0071’s “preedit does not permanently mutate the base buffer until commit”.

### D6 — Update timing: publish after paint, consume after render

For candidate window placement and geometry correctness:

- The runtime publishes `WindowTextInputSnapshot` **after paint** (same-frame caret rect).
- The runner consumes it **after render** so platform calls see the final caret geometry for the
  presented frame.

This avoids stale-caret positioning during scroll/wrap/layout changes.

## Consequences

- Mobile and desktop “text input client” APIs have a stable seam without coupling platform code to
  `crates/fret-ui`.
- The composed-view + UTF-16 model is locked early, reducing future rewrite risk for IME,
  accessibility, and selection geometry features.
- Runners may gradually implement richer platform interop (selection handles, marked text rects,
  bounds queries) without changing the widget authoring model.

## Implementation notes (current)

Evidence anchors (current code paths that already match this ADR’s intent):

- Snapshot publishing (after paint): `crates/fret-ui/src/tree/paint.rs`
- Runner consumption (after render): `crates/fret-launch/src/runner/desktop/app_handler.rs`
- Query + replace entry points: `crates/fret-ui/src/tree/mod.rs`
- Query types: `crates/fret-runtime/src/platform_text_input.rs`

## Open questions

1. Do we want a dedicated effect to request “show keyboard now” that can be emitted only inside a
   user gesture (Android constraint), or do we treat `ImeAllow(true)` as sufficient and let runners
   use gesture-origin heuristics?
2. Should `PointerCancelReason` grow more variants so mobile cancellations (system interruption,
   gesture arena loss) are distinguishable for higher-level policies?

