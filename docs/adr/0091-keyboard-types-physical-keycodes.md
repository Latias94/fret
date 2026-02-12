# ADR 0091: Align Physical Key Codes with `keyboard-types::Code`

Status: Accepted

## Context

Fret targets editor-grade keyboard behavior:

- shortcuts must be stable across keyboard layouts (VSCode-style keybindings),
- text insertion must be IME-correct and must not be derived from key presses,
- keymap files must be stable and portable across platforms.

Earlier ADRs already locked the direction:

- physical key codes are canonical for shortcuts: `docs/adr/0018-key-codes-and-shortcuts.md`
- text input is a separate channel: `docs/adr/0012-keyboard-ime-and-text-input.md`
- AltGr semantics and pending bindings: `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
- keymap file format: `docs/adr/0021-keymap-file-format.md`

The remaining risk was “key code drift”: maintaining a custom `KeyCode` subset forces future breaking changes as soon
as apps need more keys (media keys, function keys beyond F12, international keys, etc.).

## Decision

### 1) `fret-core::KeyCode` is `keyboard-types::Code`

Fret’s physical key code type is aligned with the Web/Winit ecosystem:

- `fret_core::KeyCode` is a re-export of `keyboard_types::Code`.

This provides a stable, extensible key universe without having to “grow” a bespoke enum over time.

### 2) Runners forward physical key codes without translation

Winit’s physical key type is already based on `keyboard_types::Code`:

- `winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode)` is effectively `Code`.

Therefore runners should forward physical key codes directly:

- `PhysicalKey::Code(code) => code`
- `PhysicalKey::Unidentified(_) => Code::Unidentified`

### 3) Keymap tokens are `Code` variant names

Keymap `"key"` tokens map to `keyboard_types::Code` using `FromStr` on the variant name, e.g.:

- `"KeyP"`, `"ArrowUp"`, `"Escape"`, `"MetaLeft"`, `"NumpadEnter"`

## Consequences

- Shortcut bindings remain layout-independent and future-proof (no “add keys later” breaking changes).
- Runners do not need a mapping table and avoid fragile string-based conversions.
- Keymap configs converge on a single canonical token set (`keyboard-types::Code`).

## Alternatives Considered

### A) Maintain a custom `KeyCode` enum (rejected)

Pros:

- total control over naming.

Cons:

- guaranteed breaking changes as the framework grows,
- constant churn in runner mapping, keymap parsing, and shortcut display.

### B) Make keybindings based on logical keys/characters (rejected for default)

Pros:

- potentially more “what user sees” on non-US layouts.

Cons:

- breaks VSCode-style “muscle memory” bindings,
- interacts poorly with IME and dead keys,
- requires per-platform heuristics similar to GPUI’s `prefer_character_input`.

Fret’s default remains VSCode-style physical key bindings; text input continues to flow through `TextInput`/`Ime`.

## Notes (Zed/GPUI reference, non-normative)

- GPUI explicitly models “prefer character input” as part of key dispatch/replay to handle cases
  like AltGr and multi-stroke bindings:
  `repo-ref/zed/crates/gpui/src/window.rs` (`replay_pending_input`),
  `repo-ref/zed/crates/gpui/src/key_dispatch.rs`.
