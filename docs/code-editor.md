# Code Editor Authoring

Status: App-author guidance (MVP)
Last updated: 2026-05-12

This guide is the minimum public entry point for embedding the current Fret code editor surface
without reading `ecosystem/fret-code-editor/src/editor/*`.

The editor is still an ecosystem subsystem, not a `crates/fret-ui` runtime primitive. Use it when
you need an editor-grade text surface with large-document behavior, selections, soft wrap, IME,
syntax hooks, diagnostics/perf snapshots, and a path toward editor feature payloads.

## Crates

- `fret-code-editor`: UI surface, input handling, paint, a11y projection, cache/perf snapshots.
- `fret-code-editor-buffer`: document identity, revisions, byte-indexed edits, transactions, and
  `Selection`.
- `fret-code-editor-view`: display projection and feature payload contracts such as folds, inlays,
  diagnostics, decorations, gutter markers, semantic tokens, and wrap policy.
- `fret-syntax`: optional tree-sitter-backed syntax infrastructure.

Cargo features on `fret-code-editor` are opt-in:

- `syntax`: enables syntax integration.
- `syntax-rust`: enables Rust syntax support.
- `syntax-markdown`: enables Markdown syntax support.
- `syntax-all`: enables the bundled tree-sitter language set.

## Minimal Surface

Keep the `CodeEditorHandle` in view-owned or app-owned state. Do not construct it every render.
The handle owns the current buffer, selection, interaction mode, local undo state, view options,
and diagnostics snapshots for the widget instance.

```rust
use fret_code_editor::{CodeEditor, CodeEditorHandle};

let handle = cx.slot_state(
    || CodeEditorHandle::new("fn main() {}\n"),
    |handle| handle.clone(),
);

let editor = CodeEditor::new(handle.clone())
    .overscan(32)
    .soft_wrap_cols(Some(100))
    .viewport_test_id("my-code-editor")
    .into_element(cx);
```

The snippet assumes you are already inside a Fret render function with an element context such as
`AppComponentCx` or `ElementContext`.

## Interaction Modes

Use `CodeEditorInteractionOptions` to choose whether the surface is editable, read-only, or
disabled:

```rust
use fret_code_editor::CodeEditorInteractionOptions;

handle.set_interaction(CodeEditorInteractionOptions::editor());
handle.set_interaction(CodeEditorInteractionOptions::read_only());
handle.set_interaction(CodeEditorInteractionOptions::disabled());
```

Read-only editors still support focus, navigation, selection, copy, hover-style features, and
non-mutating diagnostics. Disabled editors should not accept input or open new editor-owned feature
requests.

## Text and Selection

Use `Selection` for byte-indexed caret and selection state:

```rust
use fret_code_editor::Selection;

handle.set_selection(Selection {
    anchor: 0,
    focus: 2,
});
```

Use `with_buffer(...)` for read-only buffer inspection. Use `set_text(...)` when render code is
re-publishing app-owned text; it is a no-op when the contents are unchanged. Use
`replace_buffer(...)` when the app intentionally installs a new `TextBuffer` / `DocId`. Workspace
/ document persistence remains app-owned. If the app needs global document undo/redo, keep that
history above the editor and bridge through explicit transactions; do not treat the editor-local
history as a framework-global undo stack.

Focused command routing uses the standard `edit.undo` / `edit.redo` command ids for editor-local
history. When the focused editor does not handle them, the same ids can fall back to the active
document or workspace owner.

## Syntax and Boundaries

Syntax highlighting is optional and feature-gated. Apply language settings from app state and avoid
re-applying setters from render unless the input actually changed:

```rust
use fret_code_editor::TextBoundaryMode;

handle.set_language(Some("rust"));
handle.set_text_boundary_mode(TextBoundaryMode::Identifier);
```

Use `TextBoundaryMode::Identifier` for code-like movement and
`TextBoundaryMode::UnicodeWord` for prose or Markdown-style editing.

## View Features

Folds, inlays, diagnostics, range decorations, gutter markers, and semantic tokens are wired through
the editor handle. Source-backed feature payloads use `TextBuffer` UTF-8 byte ranges; display-row
gutter markers are validated against the current `DisplayMap`.

```rust
use fret_code_editor::{
    DiagnosticSeverity, DiagnosticSpan, FoldSpan, GutterMarker, GutterMarkerKind, InlaySpan,
    RangeDecoration, SemanticToken,
};
use std::sync::Arc;

handle.set_line_folds(
    0,
    vec![FoldSpan {
        range: 3..12,
        placeholder: Arc::<str>::from("..."),
    }],
);

handle.set_line_inlays(
    0,
    vec![InlaySpan {
        byte: 3,
        text: Arc::<str>::from(": usize"),
    }],
);

handle.set_diagnostic_spans(vec![DiagnosticSpan::new(
    0..2,
    DiagnosticSeverity::Error,
    "expected expression",
)])?;

handle.set_range_decorations(vec![RangeDecoration::new(0..2, "search.match")])?;
handle.set_gutter_markers(vec![GutterMarker::logical_line(
    0,
    GutterMarkerKind::Diagnostic,
)])?;
handle.set_semantic_tokens(vec![SemanticToken::new(0..2, "keyword")])?;
```

The editor handle does not remap feature payloads across text edits. Buffer edits and explicit
buffer replacement clear source/display payloads; producers such as language services should
re-publish payloads for the new `buffer_revision()`. The `feature_payload_snapshot()` readout is
included in diagnostics bundles so tests can assert payload stability without parsing editor
internals.

Keep feature data expressed as buffer ranges, semantic classes, ids, and command ids rather than
paint colors or app-specific callbacks.

## Overlays and Code Actions

Hover, completion, signature help, and code actions should compose with Fret's overlay/focus
system. The editor should expose request facts such as document revision, caret/selection, display
point, anchor id/rect, payload ids, and command ids.

The first public data contract for this is available through the root editor facade:

```rust
use fret_code_editor::{
    CodeAction, CodeActionList, CompletionCandidate, CompletionList, DisplayMap, DisplayPoint,
    EditorAssistKind, EditorAssistRequest, HoverPayload, validate_code_action_list,
    validate_completion_list, validate_editor_assist_request, validate_hover_payload,
};

handle.with_buffer(|buffer| {
    let map = DisplayMap::new(buffer, None);
    let request = EditorAssistRequest::new(
        EditorAssistKind::Completion,
        buffer.revision(),
        handle.selection(),
        handle.selection().normalized(),
        DisplayPoint::new(0, 0),
    );
    validate_editor_assist_request(buffer, Some(&map), &request).expect("assist request");

    let mut list = CompletionList::new(
        "completion.request.1",
        buffer.revision(),
        vec![CompletionCandidate::new("candidate.println", "println!")],
    );
    list.active_id = Some("candidate.println".into());
    validate_completion_list(buffer, &list).expect("completion list");

    let hover = HoverPayload::new("hover.symbol.1", buffer.revision(), 0..2, "symbol docs");
    validate_hover_payload(buffer, &hover).expect("hover payload");

    let actions = CodeActionList::new(
        "code-action.request.1",
        buffer.revision(),
        0..2,
        vec![CodeAction::new(
            "action.extract",
            "Extract function",
            "editor.extract_function",
        )],
    );
    validate_code_action_list(buffer, &actions).expect("code action list");
});
```

Do not put overlay policy in `fret-code-editor`. These belong to ecosystem/app layers:

- dismissal rules,
- focus trap/restore,
- hover intent,
- placement/flip/shift/size,
- listbox/combobox keyboard navigation,
- popover/menu/lightbulb recipes.

Use `fret-ui-kit`, `fret-ui-editor`, or `fret-ui-shadcn` when building those surfaces.

## Diagnostics and Performance

The current public diagnostics hooks are:

- `cache_stats()`,
- `cache_size_snapshot()`,
- `memory_snapshot()`,
- `paint_perf_frame()`.

The first-party gallery also uses diagnostic helpers for bundle proofs:

- `diag_buffer_contains_str_cached()`,
- `diag_decorated_line_text()`.

For hot-path editor work, require p50/p95/max evidence and renderer payload evidence before
changing thresholds or starting a broad rewrite. Feature-heavy editor surfaces should add payload
counters before promoting new baselines.

## Examples

Current first-party examples:

- `apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/mvp.rs`
- `apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/torture.rs`
- `apps/fret-ui-gallery/src/ui/previews/pages/editors/markdown.rs`

Run the gallery locally with:

```powershell
cargo run -p fret-ui-gallery --features gallery-dev
```

Then open the editor pages in the gallery navigation.

## Current Gaps

- Concrete app-facing completion/hover/code-action structs now exist as experimental data
  contracts. They still need real LSP/workspace producers and recipe-level UI integrations.
- The current overlay proof is a UI Gallery anchored text-assist hook that keeps overlay lifecycle
  policy in the ecosystem/app layer.
- Linux performance is not validated by the current editor workstream; keep Windows/macOS/wasm
  evidence labeled by environment.
