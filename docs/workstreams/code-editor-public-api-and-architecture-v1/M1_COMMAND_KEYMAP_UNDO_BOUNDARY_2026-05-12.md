# M1 Command, Keymap, and Undo Boundary - 2026-05-12

Status: Public architecture boundary decision

This slice resolves the P0 boundary question for editor commands, keymaps, and undo grouping. It
does not change runtime behavior; it records the owner split that future code slices must follow.

## Current State

The code editor already has two paths:

- direct `KeyDown` handling for baseline navigation, selection, insertion, deletion, copy, and
  paste fallback,
- focused command handling for `edit.undo`, `edit.redo`, `text.select_all`, `text.copy`,
  `text.cut`, `text.paste`, `text.move_word_left`, `text.move_word_right`,
  `text.select_word_left`, and `text.select_word_right`.

The editor also keeps a local `UndoHistory<CodeEditorTx>` and records buffer transactions with
selection restore data. Diagnostics expose undo/redo counts and estimated stored text bytes through
the editor memory snapshot.

## Decision

### Command Vocabulary

- Baseline text editing intent uses ADR 0044 `text.*` command ids where semantics match.
- Window/document level undo keeps accepting `edit.undo` / `edit.redo` because ADR 0127 reserves
  those names for focused-surface or window/document routing.
- Future editor-only commands must use an `editor.*` namespace, for example comment toggles,
  folding actions, find references, or code-action entry points.
- New editor features must not add hidden shortcut-only behavior. If a behavior should appear in a
  menu, command palette, keymap, or automation script, it needs a command id and availability story.

### Keymap Boundary

- Default physical keys may remain a widget-local fallback while the command registry coverage is
  incomplete.
- The target direction is that key bindings resolve to command ids through the existing keymap and
  focus routing system, then the focused editor region handles the command.
- Key repeat semantics follow command metadata, not ad-hoc editor timers. Repeatable movement,
  selection, and deletion must stay aligned with ADR 0044 / ADR 0023.
- Editor-specific keybindings belong to app/ecosystem policy, not `crates/fret-ui`.

### Undo Boundary

- `fret-code-editor-buffer` owns byte-indexed edit transactions and selection restore vocabulary.
- `fret-code-editor` may keep a local default history for standalone editor widgets.
- App/document undo policy remains app-owned. Fret must not promote code-editor local history into a
  framework-global undo stack.
- Future external-history integration should expose or consume transaction-level data, document
  identity, and selection restore data instead of coupling app undo stacks to UI handles.
- Coalescing is a model/history concern. Continuous text insertion/deletion can coalesce locally;
  cross-domain coalescing remains app/editor policy.

### Availability and Read-Only Rules

- Disabled editors should not handle text commands.
- Read-only editors may handle focus, navigation, selection, and copy, but must block mutations,
  paste, cut, and local undo/redo.
- Availability should be queryable through command availability hooks for discoverable commands,
  not only enforced when a command executes.

## Compatibility

This is a boundary decision only. Current `edit.undo` / `edit.redo` and existing direct key handling
remain valid during the transition. Future slices can add more `text.*` command coverage without
breaking callers.

## Evidence

- ADR 0044 text command vocabulary: `docs/adr/0044-text-editing-state-and-commands.md`
- ADR 0020 focus/command routing: `docs/adr/0020-focus-and-command-routing.md`
- ADR 0023 command metadata/key repeat: `docs/adr/0023-command-metadata-menus-and-palette.md`
- ADR 0127 undo infrastructure boundary: `docs/adr/0127-undo-redo-infrastructure-boundary.md`
- ADR 0185 code editor command/undo direction: `docs/adr/0185-code-editor-ecosystem-v1.md`
- Current key path: `ecosystem/fret-code-editor/src/editor/input/mod.rs`
- Current command hook: `ecosystem/fret-code-editor/src/editor/mod.rs`

## Follow-ups

1. Add focused command coverage for the remaining ADR 0044 baseline commands currently handled only
   by direct `KeyDown`.
2. Decide whether the code editor should also accept `text.undo` / `text.redo` as local-history
   aliases while preserving `edit.undo` / `edit.redo` for window/document routing.
3. Extend command availability beyond `select_all` for undo, redo, cut, paste, copy, and movement
   commands.
4. Define the first `editor.*` command only when a real editor-only behavior is implemented.
