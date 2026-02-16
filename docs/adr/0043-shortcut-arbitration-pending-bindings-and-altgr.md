# ADR 0043: Shortcut Arbitration, Pending Bindings, and AltGr


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Editor-grade applications require keyboard behavior that stays correct across:

- different keyboard layouts (US, DE, PL, etc.),
- IME pipelines,
- text editing widgets,
- multi-window focus and modal overlays,
- discoverable commands (menus + command palette).

Two classes of “late decisions” routinely cause large rewrites:

1) **Shortcut arbitration**: deciding when a keystroke is a shortcut versus character input.
2) **Multi-stroke bindings** (“pending bindings”): `Ctrl+K Ctrl+C` style sequences with timeout and replay.

AltGr is a special case: on many platforms it is represented as `Ctrl+Alt` at the event level, but it is
semantically “character input modifier” and should **not** accidentally trigger `Ctrl+Alt` shortcuts while typing.

References (proven behavior):

- Zed/GPUI pending input and replay:
  - `repo-ref/zed/crates/gpui/src/window.rs` (search `pending_input`, `needs_timeout`, `replay_pending_input`)
- Zed/GPUI “prefer character input” (AltGr case):
  - `repo-ref/zed/crates/gpui/src/interactive.rs` (see `KeyDownEvent.prefer_character_input`)
  - `repo-ref/zed/crates/gpui/src/window.rs` (search `prefer_character_input`, `AltGr`)

Related Fret ADRs:

- Keyboard/IME channels and TextInput contracts: `docs/adr/0012-keyboard-ime-and-text-input.md`
- Focus + command routing: `docs/adr/0020-focus-and-command-routing.md`
- Keymap file format v1: `docs/adr/0021-keymap-file-format.md`
- `when` expressions: `docs/adr/0022-when-expressions.md`
- Command metadata and palette: `docs/adr/0023-command-metadata-menus-and-palette.md`

## Decision

### 1) Shortcut arbitration is an explicit rule, not a widget-by-widget heuristic

For any keystroke that may produce text, the framework must decide (once, centrally) whether to:

- dispatch a command binding, or
- treat the input as character entry.

Locked P0 rules:

1. **If a keymap binding resolves to a command, the keystroke is consumed as a shortcut.**
2. **If a keystroke is consumed as a shortcut, any subsequent `Event::TextInput` originating from that same keystroke
   must be suppressed** so shortcuts never insert characters into focused text fields.
3. Character entry still flows via `Event::TextInput` and IME `Commit` (ADR 0012); editing/navigation keys flow via
   `KeyDown` + commands (ADR 0020/0023).

Notes:

- This is required because some platform backends emit both:
  - a `KeyDown` event and
  - a `text` field for the same physical keystroke.
- Fret must preserve the invariant that “shortcut resolution” and “text insertion” do not both happen for the same chord.

### 2) AltGr is a first-class modifier, not “Ctrl+Alt”

Locked P0 choice (avoid future breaking changes):

- Add `alt_gr: bool` to the core modifier representation.
- Treat AltGr as **semantically distinct** from `Ctrl+Alt` for shortcut matching and keymap persistence.

Matching semantics:

- When `alt_gr == true`, shortcut matching must **not** match bindings written as `ctrl+alt+...` unless the binding
  explicitly includes `altgr`.
- Keymap modifier token: `altgr` (lowercase) is reserved for `alt_gr`.

Rationale:

- Users must be able to type layout-dependent characters without accidentally triggering shortcuts.
- This is a hard-to-change contract because it affects:
  - `Modifiers` struct shape,
  - keymap tokens and format versioning,
  - shortcut matching logic and conflict resolution.

### 3) Multi-stroke bindings require a pending state machine with replay semantics

Keymap v1 supports only a single chord (`mods` + `key`). To support editor-grade shortcuts, Fret will add a
versioned keymap format that supports sequences.

Locked P0 design:

- Introduce **Keymap v2** with a sequence representation (multi-chord).
- Implement a window-scoped `PendingShortcut` state machine:
  - stores the matched prefix chords,
  - is tied to the current focus (cleared if focus changes),
  - is cleared by timeout,
  - consumes keystrokes while pending.

Replay semantics (to avoid “lost input”):

- If a pending sequence fails (next chord does not match any continuation), Fret must **replay** the captured chords
  as normal key events so text entry and widgets continue to behave as if no pending prefix existed.
- This mirrors Zed/GPUI’s `to_replay`/`flush_dispatch` approach (see references above).

Timeout policy:

- Default pending timeout: `1000ms` (configurable later).
- On timeout, replay the pending chords and clear the pending state.

### 4) Focus scopes apply to pending bindings exactly like they apply to shortcuts

- Pending state is window-scoped and is invalidated by:
  - focus moving to a different node,
  - active modal barrier changes (ADR 0011),
  - window losing focus.
- Multi-stroke sequences cannot “cross” focus scopes or modal barriers.

## Keymap v2 Sketch (File Format)

This ADR locks the *direction*; exact JSON shape is defined in a follow-up spec update to ADR 0021.

Proposed shape:

```json
{
  "keymap_version": 2,
  "bindings": [
    {
      "command": "editor.comment_line",
      "keys": [
        { "mods": ["ctrl"], "key": "KeyK" },
        { "mods": ["ctrl"], "key": "KeyC" }
      ],
      "when": "focus.is_text_input == false"
    }
  ]
}
```

Notes:

- v2 keeps “last-wins” semantics, but must account for sequences.
- v2 must reserve `altgr` modifier token.

## Alternatives Considered

### A) Treat AltGr as `ctrl+alt` forever (rejected)

Pros:

- No new modifier field.

Cons:

- Users can’t type layout-dependent characters safely if `ctrl+alt` bindings exist.
- Fixing later becomes a breaking change across keymaps and APIs.

### B) Support sequences without replay (rejected)

Pros:

- Simpler to implement.

Cons:

- Breaks typing in text-heavy apps: prefix chords can “eat” input and feel unreliable.
- Not acceptable for editor-grade UX.

## Consequences

- Shortcut vs character input becomes deterministic and centralized.
- AltGr behavior is safe for international keyboard layouts and avoids accidental shortcut triggers.
- Multi-stroke bindings become possible without compromising text entry (replay semantics).
- Some implementation complexity is required (pending state + timers + replay), but it avoids repeated rewrites.

## Implementation Notes (Current Workspace)

- TextInput suppression after shortcut resolution is already required by ADR 0012 and should remain true as we move to v2:
  - `docs/adr/0012-keyboard-ime-and-text-input.md`
- AltGr is tracked explicitly at the runner boundary:
  - `crates/fret-launch/src/runner/mod.rs` treats `NamedKey::AltGraph` as `Modifiers.alt_gr`,
    and normalizes away `ctrl/alt` while `alt_gr` is held.
- Keymap v2 parsing is supported (single-chord and sequences):
  - `crates/fret-app/src/keymap.rs`
- Pending bindings are implemented as a window-scoped state machine in the UI dispatcher:
  - `crates/fret-ui/src/tree/mod.rs` (`PendingShortcut`, timeout, replay via synthetic events).
- When implementing v2, prefer a single “key dispatcher” module that produces:
  - `Command` dispatch,
  - `Pending` updates,
  - `Replay` events,
  - and `TextInput suppression` decisions.
