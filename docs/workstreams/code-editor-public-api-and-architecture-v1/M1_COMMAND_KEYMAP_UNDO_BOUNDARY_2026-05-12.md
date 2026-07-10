# M1 Command, Keymap, and Undo Boundary - 2026-05-12

Status: Public architecture boundary decision

This slice resolves the P0 boundary question for editor commands, keymaps, and undo grouping. It
does not change runtime behavior; it records the owner split that future code slices must follow.

## Current State

The code editor already has two paths:

- direct `KeyDown` handling for baseline navigation, selection, insertion, deletion, copy, and
  paste fallback,
- focused command handling for `edit.undo`, `edit.redo`, `edit.select_all`, `edit.copy`, `edit.cut`,
  `edit.paste`, `text.move_word_left`,
  `text.move_word_right`,
  `text.select_word_left`, and `text.select_word_right`.

The editor also keeps a local `UndoHistory<CodeEditorTx>` and records buffer transactions with
selection restore data. Diagnostics expose undo/redo counts and estimated stored text bytes through
the editor memory snapshot.

## Decision

### Command Vocabulary

- Cross-surface editing intent uses ADR 0044 `edit.*` command ids.
- Text-buffer-specific movement, selection expansion, and deletion remain under `text.*`.
- Focused editor history and window/document fallback history share `edit.undo` / `edit.redo`;
  focus routing selects the owner.
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

## Current Contract

The temporary `text.undo/redo/copy/cut/paste/select_all` compatibility route is retired. New
commands may extend the text-specific vocabulary, but must not create a second identity for an
existing `edit.*` intent.

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
2. The temporary `text.undo` / `text.redo` alias experiment landed in
   `M1_TEXT_UNDO_REDO_ALIAS_2026-05-12.md` and was later superseded by the canonical `edit.*`
   command surface.
3. Extend command availability beyond `select_all` for undo, redo, cut, paste, copy, and movement
   commands. Completed in `M1_COMMAND_AVAILABILITY_COVERAGE_2026-05-12.md`.
4. Define the first `editor.*` command only when a real editor-only behavior is implemented.
