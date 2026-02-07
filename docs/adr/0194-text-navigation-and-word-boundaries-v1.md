# ADR 0194: Text Navigation and Word Boundaries (v1)

- Status: Proposed
- Date: 2026-01-27

## Context

Fret already locks the “hard-to-change” foundations for editor-grade text interaction:

- UTF-8 byte indices for caret/selection and text editing commands (ADR 0044).
- Keyboard/IME split, preedit-first arbitration, and cursor-area feedback (ADR 0012 / ADR 0071).
- Text geometry queries as the stable UI/renderer boundary (ADR 0045 / ADR 0046).
- Read-only selection as a first-class primitive (`SelectableText`) (ADR 0152).
- Click count and double-click semantics in pointer events (ADR 0151).

However, “word” navigation and selection semantics are currently underspecified and risk ecosystem drift:

- Different widgets may implement incompatible “word” boundaries.
- Code editor surfaces require identifier-aware boundaries (e.g. `snake_case`, Unicode identifiers),
  while general UI text fields typically want Unicode word-break behavior.
- Double-click / triple-click selection outcomes must be stable across editable and read-only surfaces.

This ADR defines the v1 contract for “word boundaries” and their relationship to:

- `text.move_word_*` / `text.select_word_*` commands (ADR 0044),
- pointer double-click selection (ADR 0151),
- and future code editor ecosystem surfaces (ADR 0200).

## Goals

1) Make “word navigation” behavior stable across core text surfaces (`TextInput`, `TextArea`, `SelectableText`).
2) Keep the runtime mechanism-only (ADR 0066): allow ecosystem/editor policy to override boundaries without
   embedding language-specific behavior in `crates/fret-ui`.
3) Preserve the UTF-8 byte index contract (ADR 0044) and keep all indices clamped to UTF-8 char boundaries.
4) Provide deterministic double-click (select word) and triple-click (select line) outcomes.

## Non-goals (v1)

- Full editor-grade subword (camelCase) navigation policies.
- Rich locale-specific tokenization rules.
- Cross-element text selection (still out of scope; see ADR 0152).

## Decision

### 1) Standardize “word boundary modes” as a policy input

Fret defines a boundary *mode* used by “word” commands and pointer selection:

- `UnicodeWord`: Unicode word-break behavior intended for general UI text.
- `Identifier`: identifier/token behavior intended for code-like text editing surfaces.

The runtime MUST treat the mode as an explicit input (policy), not a hidden per-widget heuristic.

### 2) Provide a runtime mechanism to select the active boundary mode

The UI runtime MUST provide a mechanism-level seam so a focused text surface can determine which
boundary mode is active.

Recommended v1 mechanism (preferred):

- Add `InputContext.text_boundary_mode: TextBoundaryMode` as a **window-scoped snapshot input** (same spirit
  as command gating snapshots and other window-scoped arbitration state).
- Provide an app/driver-owned override stack (recommended: a small `WindowTextBoundaryModeService` with
  push/pop tokens, mirroring the “override stack” pattern used by window command gating).

Secondary mechanism (escape hatch):

- Allow a focused widget to override via a widget hook (e.g. `Widget::text_boundary_mode(...) -> Option<TextBoundaryMode>`)
  that participates in dispatch-path queries (ADR 1157). This is intentionally an override, not the primary policy path.

Layering rule:

- `crates/fret-ui` owns the mechanism to *query* the active mode.
- Ecosystem/app code owns the *policy* for selecting `UnicodeWord` vs `Identifier`.

Default (when no policy is provided):

- `TextInput` / `TextArea` / `SelectableText` MUST use `UnicodeWord`.

Recommended policy (v1):

- If the focused surface is a code-editor-grade widget (e.g. the future `fret-code-editor` surface, ADR 0200),
  set `Identifier` mode for that window while it is focused.
- Otherwise default to `UnicodeWord`.

### 3) Define baseline behavior for “word” commands

When executing the baseline word navigation commands (ADR 0044):

- `text.move_word_left` / `text.move_word_right`
- `text.select_word_left` / `text.select_word_right`

the widget MUST interpret “word” according to the active boundary mode:

- In `UnicodeWord` mode: boundaries follow Unicode segmentation rules (UAX #29 word boundaries),
  with deterministic behavior for whitespace runs.
- In `Identifier` mode: boundaries follow identifier/token rules suitable for code editing.

The exact identifier rule set is not locked at the character-class level in v1, but the behavior
MUST be deterministic and documented by the policy provider. Recommended direction:

- ASCII fast path (`[A-Za-z0-9_]`) plus Unicode identifier categories (XID_Continue),
  implemented in an ecosystem crate (not in `crates/fret-ui`).

Recommended implementation placement (v1):

- Keep Unicode segmentation and identifier tokenization in an ecosystem helper module/crate
  (e.g. `ecosystem/fret-text-segmentation` or inside `fret-code-editor-view`), using
  `unicode-segmentation` (UAX #29) under a feature flag to control wasm size.

### 4) Double-click selects a word; triple-click selects a logical line

When a text surface receives pointer activation with a click count (ADR 0151):

- Click count = 2 (“double click”) selects the word at the pointer position.
  - Word selection uses the active boundary mode (Section 1–3).
  - The selected range MUST expand to UTF-8 char boundaries and be expressed as byte indices
    into the surface’s current display text (ADR 0071 for composing cases).

- Click count = 3 (“triple click”) selects the **logical line** containing the pointer position.
  - Logical line is delimited by newline characters in the underlying text model, not by soft-wrapped
    visual rows.
  - The selection MUST include the trailing newline when present, matching common editor behavior.

### 5) Interaction with IME preedit (composing)

During IME composition (ADR 0012 / ADR 0071), word/line selection MUST operate on the **display text**
when the surface is exposing inline preedit (the same principle as semantics in ADR 0071):

- A double-click selects a word in the display text (base text + inline preedit).
- The resulting selection indices MUST be expressed in byte offsets into that display text.

Widgets MAY choose to clear preedit before applying selection changes, but must do so deterministically.

### 6) Testing and conformance expectations

At minimum, conformance harnesses SHOULD cover:

- Unicode word boundaries: mixed Latin/CJK/emoji sequences.
- Identifier boundaries: underscores, digits, mixed scripts, and punctuation.
- Double/triple click selection correctness under scroll offsets and transforms (ADR 1156 / ADR 0083).
- Composing behavior: selection indices remain valid byte offsets in the display text (ADR 0071).

## Open Questions (for follow-up ADRs)

1) Subword navigation (camelCase / snake_case segments):
   - Keep out of v1 baseline; decide whether it belongs under `TextBoundaryMode::Identifier` as an additional
     command family (e.g. `text.move_subword_*`) vs editor-only `editor.*` commands.

## Consequences

- Ecosystem components and future code editor surfaces can share baseline command IDs (`text.*`)
  without forking behavior.
- The runtime stays policy-free while still enabling code-editor-grade identifier navigation via
  an explicit seam.
- Double-click selection becomes stable across editable and read-only surfaces.

## Alternatives Considered

1) Hardcode identifier-style word rules in `crates/fret-ui`
   - Rejected: violates ADR 0066 (policy leakage) and is hard to keep correct across languages.

2) Keep “word” semantics widget-specific
   - Rejected: causes drift and breaks user expectations across surfaces.

3) Make `text.move_word_*` always mean “identifier”
   - Rejected: surprises users in general UI text fields and harms non-code workflows.

## M0 Review Checklist (Non-Normative)

The workstream blocks on explicitly confirming these v1 decisions:

1) Seam: the preferred mechanism is window-scoped `InputContext.text_boundary_mode` plus an
   override-stack service (not a per-widget heuristic).
2) Defaults: core text widgets default to `UnicodeWord`; code-editor-grade surfaces opt into
   `Identifier` while focused.
3) Determinism around whitespace:
   - clicking whitespace selects a whitespace run,
   - clicking whitespace just after a word selects the previous word.
4) Fallback behavior:
   - when no “word” exists at the pointer (emoji/punctuation), selection falls back to a single
     grapheme cluster (never split ZWJ emoji sequences).
5) Identifier character class (v1 baseline): `_` plus Unicode XID_Continue.
6) Triple-click line selection includes the trailing newline when present.
7) Conformance strategy: keep a shared, testable implementation (currently `crates/fret-text-nav`)
   so `TextInput`/`TextArea` and the code editor cannot drift.

## Evidence anchors (implementation)

- Runtime seam + override stack: `crates/fret-runtime/src/input.rs` (`TextBoundaryMode`, `InputContext.text_boundary_mode`), `crates/fret-runtime/src/window_text_boundary_mode.rs` (`WindowTextBoundaryModeService`).
- Shared word/line boundary implementation + tests: `crates/fret-text-nav/src/lib.rs` (`select_word_range`, `select_line_range`, `move_word_left`, `move_word_right`).
- Core surface integration + tests: `crates/fret-ui/src/text_edit.rs` (delegates word/line navigation to `fret-text-nav`), `crates/fret-ui/src/declarative/tests/interactions.rs` (double-click selection under scroll/transform for `TextInput` / `TextArea`).
- Ecosystem consumer (code editor): `ecosystem/fret-code-editor/src/editor/mod.rs` (double/triple click selection; `TextInputRegionProps.text_boundary_mode_override`), `ecosystem/fret-code-editor-view/src/lib.rs` (`select_word_range_in_buffer`, `move_word_*_in_buffer`, delegates to `fret-text-nav`).

## References

- ADR 0044: Text editing state model and commands
- ADR 0071: Multiline text input + IME composition contract
- ADR 0151: Pointer click count and double-click semantics
- ADR 0152: Read-only text selection and clipboard commands
- ADR 0066: Runtime contract surface (policy vs mechanism)
- ADR 0200: Code editor ecosystem v1 (ecosystem layering)
