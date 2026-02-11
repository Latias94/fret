# AI Elements Port (`fret-ui-ai`) — TODO Tracker

Status: Active (workstream tracker)

This document tracks executable TODOs for porting AI Elements into Fret’s ecosystem.

Upstream reference:

- Components: `repo-ref/ai-elements/packages/elements/src/*.tsx`
- Docs: `repo-ref/ai-elements/apps/docs/content/components/*/*.mdx`

Workstream narrative: `docs/workstreams/ai-elements-port.md`
Milestone board (one-screen): `docs/workstreams/ai-elements-port-milestones.md`

## Tracking format

Each TODO is labeled:

- ID: `AIEL-MVP{n}-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Milestones (make progress measurable)

### M0 — Foundations (composition + gates)

- [x] AIEL-MVP0-foundation-001 Define `fret-ui-ai` public module layout (elements + model).
- [x] AIEL-MVP0-foundation-002 Add crate-level docs and a small “usage” section for each exported surface.
- [x] AIEL-MVP0-foundation-003 Add baseline `test_id` conventions (roots/rows/actions) for diag automation.
- [x] AIEL-MVP0-foundation-004 Add at least one `fretboard diag` script that targets the transcript torture page (`ai_transcript_torture`).
- [x] AIEL-MVP0-foundation-005 Define the `fret-ui-ai` data model v0 (message parts, tool calls, sources, citations).
- [x] AIEL-MVP0-foundation-006 Define the `fret.ai.*` theme token v0 list (keep small; document defaults + usage rules).

## Component inventory (upstream baseline)

Source of truth: `repo-ref/ai-elements/packages/elements/src/*.tsx`.

Status legend:

- `Prototype`: exists in `fret-ui-ai`, but not yet aligned with upstream decomposition.
- `Not started`: no Fret port yet.

### Chatbot

| Upstream | Status | Planned owner | Notes |
| --- | --- | --- | --- |
| `conversation` | Prototype | `fret-ui-ai` | Parts-based transcript exists (`AiConversationTranscript`) + empty state + download button + scroll affordance; `AiChat` provides a default composition shell. |
| `message` | Prototype | `fret-ui-ai` | `MessageParts` + `MessageResponse` exist (markdown + tool calls + sources/citations); richer action slots pending. |
| `prompt-input` | Prototype | `fret-ui-ai` | MVP exists (send/stop/disabled/loading + `test_id`). |
| `tool` | Prototype | `fret-ui-ai` | `Tool` + `ToolCallBlock` follow `tool.tsx` header/content outcomes (wrench + status badge + parameter/result sections); UI Gallery demo + diag gate: `ai_tool_demo`, `tools/diag-scripts/ui-gallery-ai-tool-demo-toggle.json`. |
| `sources` | Prototype | `fret-ui-ai` | Collapsible parity (`Used N sources`) is implemented; keep iterating on styling and payload richness (excerpt/link affordances). |
| `inline-citation` | Prototype | `fret-ui-ai` | HoverCard pager + multi-source citations are implemented; keep iterating on styling polish and interaction parity. |
| `reasoning` | Prototype | `fret-ui-ai` | `Reasoning` + `ReasoningTrigger` + `ReasoningContent` exist with streaming-driven auto-open + timed auto-close; UI Gallery demo + diag gate added. |
| `suggestion` | Prototype | `fret-ui-ai` | `Suggestions` + `Suggestion` surfaces exist; UI Gallery demo + diag gate exist. |
| `queue` | Prototype | `fret-ui-ai` | Queue surfaces + UI Gallery demo + diag gate exist; keep iterating on styling parity. |
| `model-selector` | Prototype | `fret-ui-ai` | Thin wrappers + demo exist (`apps/fret-ui-gallery` `ai_model_selector_demo`) and gated via `tools/diag-scripts/ui-gallery-ai-model-selector-demo-open-filter-select.json`. Provider logos are placeholders (no remote fetch). |
| `persona` | Not started | `fret-ui-ai` | Optional; prefer app composition. |
| `attachments` | Prototype | `fret-ui-ai` | `Attachments` + `Attachment*` surfaces exist; UI Gallery demo + diag gate exist. File pick/open effects remain app-owned. |
| `chain-of-thought` | Prototype | `fret-ui-ai` | Ported as a “step list disclosure” surface (`ChainOfThought*`) with a UI Gallery demo + diag gate: `tools/diag-scripts/ui-gallery-ai-chain-of-thought-demo-toggle.json`. |
| `checkpoint` | Prototype | `fret-ui-ai` | Ported as `Checkpoint*` surfaces + UI Gallery demo + diag gate (`tools/diag-scripts/ui-gallery-ai-checkpoint-demo-tooltip.json`). |
| `confirmation` | Prototype | `fret-ui-ai` | Ported as `Confirmation*` surfaces + UI Gallery demo + diag gate (`tools/diag-scripts/ui-gallery-ai-confirmation-demo-approve.json`). |
| `context` | Not started | `fret-ui-ai` | Decide if needed; likely app-level. |
| `plan` | Prototype | `fret-ui-ai` | Ported as `Plan*` surfaces + UI Gallery demo + diag gate (`tools/diag-scripts/ui-gallery-ai-plan-demo-toggle.json`). |
| `shimmer` | Prototype | `fret-ui-ai` | Animated text shimmer surface (`Shimmer`) + UI Gallery demo + diag gate exist. |
| `task` | Not started | `fret-ui-ai` | Optional; depends on “agent/task” UIs. |

### Code

| Upstream | Status | Planned owner | Notes |
| --- | --- | --- | --- |
| `code-block` | Prototype | `fret-ui-ai` | Backed by `ecosystem/fret-code-view`; demo + diag gate exist. |
| `snippet` | Prototype | `fret-ui-ai` | Inline copyable surface; demo + diag gate exist. |
| `artifact` | Prototype | `fret-ui-ai` | Artifact container surfaces (`Artifact*`) + UI Gallery demo + diag gate exist. |
| `file-tree` | Prototype | `fret-ui-ai` | AI Elements-aligned nested `FileTree` surface (small trees) with per-row actions (`FileTreeAction`) + future path to UI Kit virtualization for large outlines. |
| `commit` | Prototype | `fret-ui-ai` | Commit disclosure surface (`Commit*` parts) + copy feedback + file rows; demo + diag gate exist. |
| `stack-trace` | Prototype | `fret-ui-ai` | Stack trace disclosure surface (`StackTrace`) + parsed frames + copy feedback; demo + diag gate exist. |
| `schema-display` | Prototype | `fret-ui-ai` | Schema viewer surface (`SchemaDisplay*`) + UI Gallery demo + diag gate exist. |
| `terminal` | Not started | `fret-ui-ai` | Depends on whether we want ANSI rendering; may become a separate crate. |
| `test-results` | Prototype | `fret-ui-ai` | Test results surface (`TestResults*`) + suite disclosure (`TestSuite`) + UI Gallery demo + diag gate exist. |
| `jsx-preview` | Not started | n/a | Likely out of scope for Rust. |
| `web-preview` | Prototype | `fret-ui-ai` | Chrome port exists (`WebPreview*`) with UI Gallery demo + diag gate: `ai_web_preview_demo`, `tools/diag-scripts/ui-gallery-ai-web-preview-demo-commit-console.json`. Embedded native webview backend is host-owned and available behind `fret-launch/webview-wry` + `fret-ui-gallery/webview-wry` (see `docs/workstreams/webview-wry-v1.md`). |
| `agent` | Not started | `fret-ui-ai` | Only if there is a concrete app consumer. |
| `sandbox` | Not started | `fret-ui-ai` | Only if there is a concrete app consumer. |
| `package-info` | Not started | `fret-ui-ai` | Only if there is a concrete app consumer. |
| `environment-variables` | Prototype | `fret-ui-ai` | Ported as `EnvironmentVariables*` surfaces + UI Gallery demo + diag gate (`tools/diag-scripts/ui-gallery-ai-environment-variables-demo-toggle-copy.json`). |

### Utilities

| Upstream | Status | Planned owner | Notes |
| --- | --- | --- | --- |
| `image` | Prototype | `fret-ui-ai` or `fret-ui-shadcn` | `Image` surface exists (renders `ImageId`; decoding/upload is app-owned). Decide later if it belongs in shadcn. |
| `open-in-chat` | Not started | `fret-ui-ai` | Likely app-level wiring; component is just chrome. |

### Workflow

| Upstream | Status | Planned owner | Notes |
| --- | --- | --- | --- |
| `canvas` / `node` / `edge` / `panel` / `toolbar` / `controls` / `connection` | Not started | `fret-ui-ai` (wrappers) | Prefer “styling recipes” over new engines; reuse `fret-node`/`fret-canvas`. |

### Voice

| Upstream | Status | Planned owner | Notes |
| --- | --- | --- | --- |
| `audio-player` / `mic-selector` / `speech-input` / `transcription` / `voice-selector` | Not started | TBD | Defer until there is a concrete consumer. |

### M1 — Chat surfaces (usable app kit)

- [x] AIEL-MVP1-chat-001 Port `Conversation` parts: content, empty state, scroll button, download.
- [x] AIEL-MVP1-chat-002 Port `Message` parts: content wrapper, actions, toolbar slots.
- [x] AIEL-MVP1-chat-007 Port `MessageBranch` surfaces (branch content + selector controls).
  - Implemented: `ecosystem/fret-ui-ai/src/elements/message_branch.rs`
  - Demo: `apps/fret-ui-gallery` page `ai_message_branch_demo`
  - Gate: `tools/diag-scripts/ui-gallery-ai-message-branch-demo-wrap.json`
- [x] AIEL-MVP1-chat-003 Add `MessageResponse` (markdown/code rendering + initial code actions).
- [x] AIEL-MVP1-chat-004 Port `PromptInput` MVP (text input + send/stop + disabled/loading states).
- [x] AIEL-MVP1-chat-005 UI Gallery page(s): chat demo with streaming append + tool calls (not just torture).
- [x] AIEL-MVP1-chat-006 Define and implement the v0 streaming contract for markdown parts (append-only + finalize).

### M2 — Tooling surfaces (assistant/tooling apps)

- [~] AIEL-MVP2-tools-001 Port `Tool` (input/output blocks, running/success/error states, collapse).
- [~] AIEL-MVP2-tools-002 Port `Sources` and `InlineCitation` (stable linking and display).
- [ ] AIEL-MVP2-tools-003 Port `Suggestion` (optional; only if apps need it).

## Regression gates (scripts)

Existing gates (UI Gallery `ai_chat_demo`):

- `tools/diag-scripts/ui-gallery-ai-chat-demo-prompt-input-keyboard.json`
- `tools/diag-scripts/ui-gallery-ai-chat-demo-toolcall-collapse.json`
- `tools/diag-scripts/ui-gallery-ai-chat-demo-codeblock-expand.json`
- `tools/diag-scripts/ui-gallery-ai-chat-demo-codeblock-copy.json`
- `tools/diag-scripts/ui-gallery-ai-chat-demo-streaming-finalize.json`
- `tools/diag-scripts/ui-gallery-ai-chat-demo-citation-highlight.json`
- `tools/diag-scripts/ui-gallery-ai-chat-demo-sources-collapsible.json` (`[!]` if click targeting is occluded by prompt chrome)
- `tools/diag-scripts/ui-gallery-ai-chat-demo-inline-citation-hovercard.json` (`[!]` if hover/click targeting is occluded by prompt chrome)

Existing gates (UI Gallery `ai_artifact_demo`):

- `tools/diag-scripts/ui-gallery-ai-artifact-demo-close-toggle.json`

Existing gates (UI Gallery `ai_shimmer_demo`):

- `tools/diag-scripts/ui-gallery-ai-shimmer-demo-pixels-changed.json`

Existing gates (UI Gallery `ai_attachments_demo`):

- `tools/diag-scripts/ui-gallery-ai-attachments-demo-remove.json`

Existing gates (UI Gallery `ai_suggestions_demo`):

- `tools/diag-scripts/ui-gallery-ai-suggestions-demo-click.json`

Existing gates (UI Gallery `ai_code_block_demo`):

- `tools/diag-scripts/ui-gallery-ai-code-block-demo-copy.json`

Existing gates (UI Gallery `ai_commit_demo`):

- `tools/diag-scripts/ui-gallery-ai-commit-demo-copy.json`

Existing gates (UI Gallery `ai_commit_large_demo`):

- `tools/diag-scripts/ui-gallery-ai-commit-large-scroll.json`

Existing gates (UI Gallery `ai_stack_trace_demo`):

- `tools/diag-scripts/ui-gallery-ai-stack-trace-demo-copy.json`

Existing gates (UI Gallery `ai_stack_trace_large_demo`):

- `tools/diag-scripts/ui-gallery-ai-stack-trace-large-scroll.json`

Existing gates (UI Gallery `ai_test_results_demo`):

- `tools/diag-scripts/ui-gallery-ai-test-results-demo-toggle.json`

Existing gates (UI Gallery `ai_test_results_large_demo`):

- `tools/diag-scripts/ui-gallery-ai-test-results-large-scroll.json`

Existing gates (UI Gallery `ai_checkpoint_demo`):

- `tools/diag-scripts/ui-gallery-ai-checkpoint-demo-tooltip.json`

Existing gates (UI Gallery `ai_confirmation_demo`):

- `tools/diag-scripts/ui-gallery-ai-confirmation-demo-approve.json`

Existing gates (UI Gallery `ai_environment_variables_demo`):

- `tools/diag-scripts/ui-gallery-ai-environment-variables-demo-toggle-copy.json`

Existing gates (UI Gallery `ai_plan_demo`):

- `tools/diag-scripts/ui-gallery-ai-plan-demo-toggle.json`

Existing gates (UI Gallery `ai_tool_demo`):

- `tools/diag-scripts/ui-gallery-ai-tool-demo-toggle.json`

Existing gates (UI Gallery `ai_model_selector_demo`):

- `tools/diag-scripts/ui-gallery-ai-model-selector-demo-open-filter-select.json`

Existing gates (UI Gallery `ai_web_preview_demo`):

- `tools/diag-scripts/ui-gallery-ai-web-preview-demo-commit-console.json`

Existing gates (UI Gallery `ai_chain_of_thought_demo`):

- `tools/diag-scripts/ui-gallery-ai-chain-of-thought-demo-toggle.json`

Existing gates (UI Gallery `ai_transcript_torture`):

- `tools/diag-scripts/ui-gallery-ai-transcript-torture-scroll.json`
- `tools/diag-scripts/ui-gallery-ai-transcript-scroll-button.json`

Existing gates (UI Gallery `ai_file_tree_demo`):

- `tools/diag-scripts/ui-gallery-ai-file-tree-demo-toggle.json`
- `tools/diag-scripts/ui-gallery-ai-file-tree-demo-actions.json`
- `tools/diag-scripts/ui-gallery-ai-file-tree-large-scroll.json`

Existing gates (UI Gallery `ai_schema_display_demo`):

- `tools/diag-scripts/ui-gallery-ai-schema-display-demo.json`

### M3 — Code artifacts (developer-facing outputs)

Prioritize thin adapters over new engines:

- [x] AIEL-MVP3-code-001 `CodeBlock` / `Snippet` backed by `ecosystem/fret-code-view` + `ecosystem/fret-syntax`.
  - Gate: `tools/diag-scripts/ui-gallery-ai-code-block-demo-copy.json` (CodeBlock copy + Snippet copy).
- [x] AIEL-MVP3-code-002 `FileTree` backed by `fret-ui-kit` tree primitives (virtualized).
  - Notes: `fret-ui-ai::FileTree` now flattens items via UI Kit `TreeItem` + `flatten_tree`, and renders via `VirtualList` when the host provides a height constraint.
  - Gate: `tools/diag-scripts/ui-gallery-ai-file-tree-large-scroll.json` (expand → scroll to row → click select).
- [x] AIEL-MVP3-code-003 `Commit` / `StackTrace` / `TestResults` surfaces (only after CodeBlock is solid).
  - `Commit` v0 is implemented with a UI Gallery demo + diag gate.
  - `Artifact` v0 is implemented with a UI Gallery demo + diag gate.
  - `StackTrace` v0 is implemented with a UI Gallery demo + diag gate.
  - `TestResults` v0 is implemented with a UI Gallery demo + diag gate.
  - Parity polish checklist (make it measurable):
    - [~] Match upstream copy semantics per-surface (some buttons suppress re-copy while `copied` is active, others do not); add `on_copy` hooks where missing.
      - Done: `CommitCopyButton` suppresses re-copy while `copied` is active + `on_copy` hook exists.
      - Done: `StackTraceCopyButton` `on_copy` hook exists (repeat-copy semantics remain allowed).
    - [~] Add/confirm stable `test_id` selectors for per-row actions (commit files, stack frames, test suites/tests).
      - Done: commit file rows + file paths have `test_id` and a large-list demo uses them.
      - Done: per-frame `test_id` for stack trace rows + file path buttons (used by the large-list gate).
      - Done: test rows can be tagged with `test_id` and a large-list demo uses them.
    - [~] Confirm long-list behavior stays stable (scroll + selection) and add one gate if needed:
      - commit: many files; stack trace: many frames; test results: many suites.
      - Done: commit many-files scroll gate (`tools/diag-scripts/ui-gallery-ai-commit-large-scroll.json`).
      - Done: stack trace many-frames scroll gate (`tools/diag-scripts/ui-gallery-ai-stack-trace-large-scroll.json`).
      - Done: test results many-tests scroll/activate gate (`tools/diag-scripts/ui-gallery-ai-test-results-large-scroll.json`).
    - [~] Confirm extension hooks exist for app-owned effects:
      - commit: file row click / open file
      - stack trace: file path click / open file
      - test results: test click / open test output
      - Done: `Test::on_activate(OnTestActivate)` exists and is used by the large demo gate.
    - [x] Re-audit upstream TS (`commit.tsx`, `stack-trace.tsx`, `test-results.tsx`) and document known deltas in `docs/workstreams/ai-elements-port.md`.
- [x] AIEL-MVP3-code-004 `SchemaDisplay` (portable JSON schema-ish viewer) if needed for tool outputs.

### M4 — Workflow surfaces (optional; reuse existing ecosystem)

Only implement if it is mostly “styling recipes over existing crates”:

- [ ] AIEL-MVP4-workflow-001 Map `Canvas/Node/Edge/Panel/Toolbar` to `ecosystem/fret-node` + `ecosystem/fret-canvas`.
- [ ] AIEL-MVP4-workflow-002 Provide shadcn-aligned chrome wrappers (panels, toolbars, controls).

### M5 — Voice surfaces (defer until there is a concrete consumer)

- [ ] AIEL-MVP5-voice-001 Decide whether voice UI belongs in `fret-ui-ai` or a sibling crate (`fret-ui-voice`).
- [ ] AIEL-MVP5-voice-002 If in scope: `AudioPlayer`, `MicSelector`, `Transcription`, `VoiceSelector`.

## Upstream inventory (keep honest)

These are the upstream component entrypoints in the pinned checkout:
`repo-ref/ai-elements/packages/elements/src/*.tsx`.

Keep this list in sync with the pinned upstream commit recorded in
`docs/workstreams/ai-elements-port.md`.

- `agent.tsx`
- `artifact.tsx`
- `attachments.tsx`
- `audio-player.tsx`
- `canvas.tsx`
- `chain-of-thought.tsx`
- `checkpoint.tsx`
- `code-block.tsx`
- `commit.tsx`
- `confirmation.tsx`
- `connection.tsx`
- `context.tsx`
- `controls.tsx`
- `conversation.tsx`
- `edge.tsx`
- `environment-variables.tsx`
- `file-tree.tsx`
- `image.tsx`
- `inline-citation.tsx`
- `jsx-preview.tsx`
- `message.tsx`
- `mic-selector.tsx`
- `model-selector.tsx`
- `node.tsx`
- `open-in-chat.tsx`
- `package-info.tsx`
- `panel.tsx`
- `persona.tsx`
- `plan.tsx`
- `prompt-input.tsx`
- `queue.tsx`
- `reasoning.tsx`
- `sandbox.tsx`
- `schema-display.tsx`
- `shimmer.tsx`
- `snippet.tsx`
- `sources.tsx`
- `speech-input.tsx`
- `stack-trace.tsx`
- `suggestion.tsx`
- `task.tsx`
- `terminal.tsx`
- `test-results.tsx`
- `tool.tsx`
- `toolbar.tsx`
- `transcription.tsx`
- `voice-selector.tsx`
- `web-preview.tsx`

## TODOs by subsystem

### Foundation

- [x] AIEL-MVP0-foundation-010 Add a component inventory table (upstream file → Fret module → status).
- [ ] AIEL-MVP0-foundation-011 Ensure all public surfaces are declarative-only (no retained widget authoring).
- [ ] AIEL-MVP0-foundation-012 Add “where should this code live?” rules of thumb (copy from shadcn workstream patterns).
- [x] AIEL-MVP0-foundation-013 Add a short “Public API rules” section to the workstream and keep it updated.
  - Target: `docs/workstreams/ai-elements-port.md` (parts, controlled/uncontrolled, intents not effects, stable selectors).
- [x] AIEL-MVP0-foundation-014 Add “Version stamp” update rules (pinned upstream commit must be updated first).
  - Target: `docs/workstreams/ai-elements-port.md`

### WebPreview backend (optional, feature-gated)

- [x] AIEL-MVP0-webpreview-001 Port `WebPreview` chrome-only surfaces + UI Gallery demo + diag gate.
- [~] AIEL-MVP0-webpreview-002 Close the v1 backend loop (navigation state + URL reflection) and document limitations.
  - Source of truth: `docs/workstreams/webview-wry-v1.md` + TODO tracker.
- [x] AIEL-MVP0-webpreview-002 Define WebView integration plan and crate boundaries (contract + wry backend).
  - Workstream: `docs/workstreams/webview-wry-v1.md`
  - TODOs: `docs/workstreams/webview-wry-v1-todo.md`

### Data model (v0)

- [x] AIEL-MVP0-model-001 Confirm `MessageId = u64` (align with `crates/fret-ui::ItemKey`) and document stability rules.
  - Include interop guidance: keep optional `external_id: Arc<str>` and derive `u64` key at the app boundary.
- [x] AIEL-MVP0-model-002 Define `MessageRole` + `MessagePart` enums (markdown/tool/sources/attachments).
- [x] AIEL-MVP0-model-003 Define `ToolCall` lifecycle state model (Pending/Running/Succeeded/Failed/Cancelled).
- [x] AIEL-MVP0-model-004 Define `SourceItem` + inline citation referencing (stable anchor keys for scripts).
- [x] AIEL-MVP0-model-005 Define streaming update contract for markdown/text parts (append + finalize).
- [x] AIEL-MVP0-model-006 Allow a single citation to reference multiple sources (upstream `InlineCitationCardTrigger.sources: string[]`).
  - Implemented as `CitationItem.source_ids: Arc<[Arc<str>]>` (keeps single-source `CitationItem::new` for convenience).

### Theme tokens (v0)

- [x] AIEL-MVP0-theme-001 Create a minimal token list under `fret.ai.*` (padding/gaps/chrome basics).
- [ ] AIEL-MVP0-theme-002 Add default token values to the baseline shadcn theme config (or document required overrides).
- [~] AIEL-MVP0-theme-003 Replace hard-coded theme string keys in `fret-ui-ai` where tokenized mapping is feasible.

### Conversation / Transcript

- [x] AIEL-MVP1-chat-020 Split transcript into composable parts (Conversation root vs transcript body).
- [x] AIEL-MVP1-chat-021 Add “empty state” surface (title/description/icon).
- [x] AIEL-MVP1-chat-022 Add “download transcript” helper (format function hook; effect performed by app).
- [~] AIEL-MVP1-chat-023 Add “scroll-to-bottom” button styling parity (rounded, outline, dark-mode background).
- [ ] AIEL-MVP1-chat-024 Define selection/search contracts for long transcripts (defer implementation if needed, but write the contract).

### Tool calls

- [x] AIEL-MVP1-tool-100 Align `tool.tsx` disclosure outcomes (header + status badge + sections).
  - Targets: `Tool`, `ToolHeader`, `ToolContent`, `ToolInput`, `ToolOutput`, `ToolCallBlock`.

### Message

- [x] AIEL-MVP1-chat-040 Replace `Message(text)` with a composition surface (container + content + actions slots).
- [x] AIEL-MVP1-chat-043 Port `message.tsx` action surfaces (`MessageActions`, `MessageAction` with optional tooltip).
- [ ] AIEL-MVP1-chat-041 Implement branch selector outcomes (optional; only if there is a consumer).
- [ ] AIEL-MVP1-chat-042 Add per-role chrome tokens (avoid hard-coded theme string keys in component code).

### MessageResponse (markdown + streaming)

- [x] AIEL-MVP1-chat-060 Integrate `ecosystem/fret-markdown` for markdown rendering.
- [x] AIEL-MVP1-chat-061 Define streaming update contract (append chunks, finalize, stable block IDs for code fences).
- [~] AIEL-MVP1-chat-062 Add code fence actions slot (copy / expand / download) using `MarkdownComponents`.

### PromptInput

- [x] AIEL-MVP1-chat-080 Prompt input MVP: text input + send + stop + loading spinner.
- [x] AIEL-MVP1-chat-081 Optional attachments chips: add/remove/clear (app performs file picker effects).
  - Outcomes: render chips row, allow remove per item, expose `clear` intent at the app boundary.
  - Gate: `tools/diag-scripts/ui-gallery-ai-chat-demo-prompt-attachments-backspace-enter.json` (add → remove via Backspace → send).
- [x] AIEL-MVP1-chat-084 Textarea keyboard parity: `Enter` submits, `Shift+Enter` inserts newline (IME-safe).
- [x] AIEL-MVP1-chat-085 Backspace parity: when textarea is empty, `Backspace` removes the last attachment.
- [!] AIEL-MVP1-chat-086 Clipboard file paste parity: `paste` adds file/image attachments.
  - Blocker: Fret runtime currently supports clipboard *text* effects only; file/image clipboard needs a new platform capability.
- [x] AIEL-MVP1-chat-087 PromptInput add-attachments action parity: plus-button action emits an app-owned “open file dialog” intent.
  - Surface: `PromptInput::on_add_attachments` + `PromptInput::test_id_add_attachments` (also exposed via `AiChat::on_add_attachments`).
  - Gate: `tools/diag-scripts/ui-gallery-ai-chat-demo-prompt-attachments-backspace-enter.json` (click plus → chips → Backspace pop → send).
- [x] AIEL-MVP1-chat-088 PromptInput drag-and-drop fallback: accept external file drops as attachments (primary path while clipboard files are unsupported).
  - Surface: `PromptInput` wraps its root with `ExternalDragRegion` and handles `ExternalDragKind::DropFiles` by appending file-name-based attachment chips.
  - Notes: this is metadata-only (no bytes); the component releases the `ExternalDropToken` via `Effect::ExternalDropRelease` after it has updated the attachments model.
  - Gate: unit test `crates/fret-ui/src/declarative/tests/interactions.rs` (`declarative_external_drag_region_can_handle_external_drag_events`).
- [x] AIEL-MVP1-chat-089 PromptInput provider mode parity: allow lifting text + attachments models outside the PromptInput surface.
  - Surface: `PromptInputProvider` + `use_prompt_input_controller` + `PromptInput::new_uncontrolled` (provider/local/uncontrolled resolution).
  - Gate: unit test `ecosystem/fret-ui-ai/src/elements/prompt_input.rs` (`prompt_input_provider_text_model_receives_text_input`).
- [x] AIEL-MVP1-chat-090 PromptInput parts decomposition parity (align upstream composition surface).
  - Goal: mirror the upstream “parts-first” API so apps can rearrange prompt chrome without forking.
  - Target parts (initial set):
    - `PromptInputBody`, `PromptInputTextarea`, `PromptInputHeader`, `PromptInputFooter`
    - `PromptInputTools`, `PromptInputButton`, `PromptInputSubmit`
    - `PromptInputActionMenu*` and an intent-driven `PromptInputActionAddAttachments`
  - Keep `PromptInput` as a default recipe wrapper for backward compatibility.
  - Surface (Fret): `PromptInputRoot` + `PromptInputSlots` + `PromptInputHeader`/`Footer`/`Tools`/`Button`/`Submit`/`ActionAddAttachments` (+ `PromptInputProvider`, `use_prompt_input_controller`, `use_prompt_input_config`).
- [x] AIEL-MVP1-chat-092 PromptInput action menu parity: `PromptInputActionMenu*` surfaces (dropdown menu trigger + items).
  - Surface: `PromptInputActionMenu` / `PromptInputActionMenuTrigger` / `PromptInputActionMenuContent` / `PromptInputActionMenuItem` + `PromptInputActionAddAttachments`.
  - Gate: `tools/diag-scripts/ui-gallery-ai-prompt-input-action-menu-demo.json`.
- [x] AIEL-MVP1-chat-093 PromptInput attachments constraints parity: accept/multiple/maxFiles/maxFileSize/onError.
  - Upstream reference: `prompt-input.tsx` (`accept`, `multiple`, `maxFiles`, `maxFileSize`, `onError`).
  - Surface (Fret): `PromptInputConfig::{accept,multiple,max_files,max_file_size_bytes,on_error}` (also on `PromptInputRoot` + `PromptInput`).
  - Behavior: validates external file drops against `accept` + `max_files` + `max_file_size_bytes`, filters rejected items, and emits `on_error` with a typed `PromptInputErrorCode`.
  - Notes:
    - Constraints are enforced on `ExternalDragKind::DropFiles` (metadata-only attachments). App-owned file dialogs can reuse the same config, but programmatic attachment insertion is app-owned by design.
    - `max_file_size_bytes` is enforced only when `size_bytes` is known.
    - On native, `media_type` may be unavailable; `accept` matching falls back to file extensions for common `image/*` formats.
  - Gate: unit tests in `ecosystem/fret-ui-ai/src/elements/prompt_input.rs` (`prompt_input_drop_respects_max_files_and_emits_error`, `prompt_input_drop_accept_and_size_errors_do_not_add_attachments`).
- [x] AIEL-MVP1-chat-094 PromptInput referenced sources parity: local referenced sources model + chips row.
  - Upstream reference: `prompt-input.tsx` (`ReferencedSourcesContext` local to PromptInput).
  - Note: keep this local even in provider mode (matches upstream: attachments can be provider-owned, referenced sources remain local).
  - Gate: `tools/diag-scripts/ui-gallery-ai-prompt-input-referenced-sources-demo.json`.
- [ ] AIEL-MVP1-chat-095 PromptInput hidden input sync parity (`syncHiddenInput`).
  - Upstream reference: `prompt-input.tsx` (`syncHiddenInput`).
  - Note: implement only if we have a concrete app need; otherwise defer (native form posts are not a primary Fret flow).
- [x] AIEL-MVP1-chat-091 Provider-mode composition demo + gate.
  - Add a UI Gallery page that composes `PromptInputProvider` + parts and demonstrates “external actions”
    (e.g. a toolbar button/menu item that triggers `on_add_attachments`).
  - Add a diag script gate:
    - `tools/diag-scripts/ui-gallery-ai-prompt-input-provider-demo.json`
  - UI Gallery page: `ai_prompt_input_provider_demo`.
- [ ] AIEL-MVP1-chat-082 Optional model selector and persona surfaces only if used by apps (avoid porting for completeness).
- [x] AIEL-MVP1-chat-083 Add a diag script for keyboard-only operation (type, submit, cancel/stop).

### Sources / Citations (parity pass)

- [x] AIEL-MVP2-tooling-010 Align `Sources` to upstream Collapsible behavior (`Used N sources` trigger, hidden-by-default content).
- [x] AIEL-MVP2-tooling-011 Align `InlineCitation` to upstream HoverCard behavior (delay 0, pager with prev/next + `current/count`).
- [x] AIEL-MVP2-tooling-012 Gate hover + pager with `fretboard diag` (open hover card, next/prev).
  - Script: `tools/diag-scripts/ui-gallery-ai-chat-demo-inline-citation-hovercard.json`
- [x] AIEL-MVP2-tooling-013 Gate sources Collapsible with `fretboard diag` (open, verify rows).
  - Script: `tools/diag-scripts/ui-gallery-ai-chat-demo-sources-collapsible.json`
- [x] AIEL-MVP2-tooling-014 Fix/mitigate pointer targeting issues for these gates.
  - Implemented: diagnostics `scroll_into_view` container selection prefers an ancestor container with the largest bounds (avoids selecting tiny semantics wrappers).
  - Implemented: parts-level `test_id` semantics wrappers are layout-transparent (avoid accidental wheel/click routing changes).

### Tool calls / Sources / Citations

- [x] AIEL-MVP2-tools-100 Tool call block: request/response sections, collapse, error state.
  - UI Gallery page: `ai_tool_demo`
  - Diag gate: `tools/diag-scripts/ui-gallery-ai-tool-demo-toggle.json`
- [ ] AIEL-MVP2-tools-101 Sources list: title/url, open-url intent hook, truncation rules.
- [ ] AIEL-MVP2-tools-102 Inline citation: stable anchor behavior (jump/highlight) within a transcript.

## Regression gates (default requirement)

- [x] AIEL-MVP0-gates-001 Gate transcript torture scrolling.
  - Scripts:
    - `tools/diag-scripts/ui-gallery-ai-transcript-torture-scroll.json`
    - `tools/diag-scripts/ui-gallery-ai-transcript-scroll-button.json`
  - Scenario: open UI Gallery `ai_transcript_torture`, wheel-scroll, verify scroll affordances, capture bundle.
  - Env baseline:
    - `FRET_UI_GALLERY_START_PAGE=ai_transcript_torture`
    - `FRET_UI_GALLERY_AI_TRANSCRIPT_LEN=5000`
    - optional: `FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`
  - Checks (choose at least one): stale paint, view-cache reuse stability, top-frame perf snapshot.
- [ ] AIEL-MVP0-gates-002 Add `tools/diag-scripts/ui-gallery-ai-transcript-append.json`.
  - Scenario: append messages (or bump revision) while scrolled near-bottom and away-from-bottom; verify stick-to-bottom eligibility.
- [x] AIEL-MVP1-gates-010 Gate “chat demo” interactions via diag.
  - Scripts:
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-prompt-input-keyboard.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-streaming-finalize.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-toolcall-collapse.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-codeblock-expand.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-export-markdown.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-message-action-tooltip.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-sources-collapsible.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-inline-citation-hovercard.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-citation-highlight.json`
- [x] AIEL-MVP1-gates-011 Gate reasoning auto-open/auto-close via diag.
  - Script: `tools/diag-scripts/ui-gallery-ai-reasoning-demo-auto-open-close.json`
- [x] AIEL-MVP1-gates-012 Gate queue sections + scroll + hover actions via diag.
  - Script: `tools/diag-scripts/ui-gallery-ai-queue-demo-section-scroll-action.json`
  - Note: exercises scrolling via `wheel`; does not assert post-scroll item hit-testing (scroll offset is HitTest-only).
- [x] AIEL-MVP1-gates-013 Gate attachments hover-remove via diag.
  - Script: `tools/diag-scripts/ui-gallery-ai-attachments-demo-remove.json`
  - Verified PASS: 2026-02-09 (local).
- [x] AIEL-MVP1-gates-014 Gate message branch wrap-around via diag.
  - Script: `tools/diag-scripts/ui-gallery-ai-message-branch-demo-wrap.json`
  - Verified PASS: 2026-02-09 (local).
- [x] AIEL-MVP2-gates-030 Gate tool disclosure demo via diag.
  - Script: `tools/diag-scripts/ui-gallery-ai-tool-demo-toggle.json`
- [ ] AIEL-MVP1-gates-020 Add at least one unit test per shipped component family asserting a fragile invariant
  (e.g. stick-to-bottom eligibility rules, stable key mapping, overlay dismiss outcomes).
