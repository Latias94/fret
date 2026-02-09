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
| `tool` | Prototype | `fret-ui-ai` | `Tool` + `ToolCallBlock` follow `tool.tsx` header/content outcomes (wrench + status badge + parameter/result sections); keep iterating on payload views and styling parity. |
| `sources` | Prototype | `fret-ui-ai` | Collapsible parity (`Used N sources`) is implemented; keep iterating on styling and payload richness (excerpt/link affordances). |
| `inline-citation` | Prototype | `fret-ui-ai` | HoverCard pager + multi-source citations are implemented; keep iterating on styling polish and interaction parity. |
| `reasoning` | Prototype | `fret-ui-ai` | `Reasoning` + `ReasoningTrigger` + `ReasoningContent` exist with streaming-driven auto-open + timed auto-close; UI Gallery demo + diag gate added. |
| `suggestion` | Prototype | `fret-ui-ai` | `Suggestions` + `Suggestion` surfaces exist; UI Gallery demo + diag gate exist. |
| `queue` | Not started | `fret-ui-ai` | Optional. |
| `model-selector` | Not started | `fret-ui-ai` | Optional; prefer app composition. |
| `persona` | Not started | `fret-ui-ai` | Optional; prefer app composition. |
| `attachments` | Not started | `fret-ui-ai` | Requires host effects (file pick); keep policy-only in components. |
| `chain-of-thought` | Not started | `fret-ui-ai` | Consider mapping to `reasoning`/disclosure patterns. |
| `checkpoint` | Not started | `fret-ui-ai` | Likely a styling recipe. |
| `confirmation` | Not started | `fret-ui-ai` | Likely maps to shadcn alert/dialog. |
| `context` | Not started | `fret-ui-ai` | Decide if needed; likely app-level. |
| `plan` | Not started | `fret-ui-ai` | Optional; depends on product needs. |
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
| `web-preview` | Not started | n/a | Needs embedded webview; out of scope unless there is a host. |
| `agent` / `sandbox` / `package-info` / `environment-variables` | Not started | `fret-ui-ai` | Only if there is a concrete app consumer. |

### Utilities

| Upstream | Status | Planned owner | Notes |
| --- | --- | --- | --- |
| `image` | Not started | `fret-ui-ai` or `fret-ui-shadcn` | Decide whether it is generic enough to live in shadcn. |
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
- [ ] AIEL-MVP1-chat-007 Port `MessageBranch` surfaces (branch content + selector controls).
- [x] AIEL-MVP1-chat-003 Add `MessageResponse` (markdown/code rendering + initial code actions).
- [x] AIEL-MVP1-chat-004 Port `PromptInput` MVP (text input + send/stop + disabled/loading states).
- [x] AIEL-MVP1-chat-005 UI Gallery page(s): chat demo with streaming append + tool calls (not just torture).
- [x] AIEL-MVP1-chat-006 Define and implement the v0 streaming contract for markdown parts (append-only + finalize).

### M2 — Tooling surfaces (assistant/tooling apps)

- [~] AIEL-MVP2-tools-001 Port `Tool` (input/output blocks, running/success/error states, collapse).
- [~] AIEL-MVP2-tools-002 Port `Sources` and `InlineCitation` (stable linking and display).
- [ ] AIEL-MVP2-tools-003 Port `Suggestion` and `Queue` (optional; only if apps need them).

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

Existing gates (UI Gallery `ai_suggestions_demo`):

- `tools/diag-scripts/ui-gallery-ai-suggestions-demo-click.json`

Existing gates (UI Gallery `ai_code_block_demo`):

- `tools/diag-scripts/ui-gallery-ai-code-block-demo-copy.json`

Existing gates (UI Gallery `ai_commit_demo`):

- `tools/diag-scripts/ui-gallery-ai-commit-demo-copy.json`

Existing gates (UI Gallery `ai_stack_trace_demo`):

- `tools/diag-scripts/ui-gallery-ai-stack-trace-demo-copy.json`

Existing gates (UI Gallery `ai_test_results_demo`):

- `tools/diag-scripts/ui-gallery-ai-test-results-demo-toggle.json`

Existing gates (UI Gallery `ai_transcript_torture`):

- `tools/diag-scripts/ui-gallery-ai-transcript-torture-scroll.json`
- `tools/diag-scripts/ui-gallery-ai-transcript-scroll-button.json`

Existing gates (UI Gallery `ai_file_tree_demo`):

- `tools/diag-scripts/ui-gallery-ai-file-tree-demo-toggle.json`
- `tools/diag-scripts/ui-gallery-ai-file-tree-demo-actions.json`

Existing gates (UI Gallery `ai_schema_display_demo`):

- `tools/diag-scripts/ui-gallery-ai-schema-display-demo.json`

### M3 — Code artifacts (developer-facing outputs)

Prioritize thin adapters over new engines:

- [~] AIEL-MVP3-code-001 `CodeBlock` / `Snippet` backed by `ecosystem/fret-code-view` + `ecosystem/fret-syntax`.
- [ ] AIEL-MVP3-code-002 `FileTree` backed by `fret-ui-kit` tree primitives (virtualized).
- [~] AIEL-MVP3-code-003 `Commit` / `StackTrace` / `TestResults` surfaces (only after CodeBlock is solid).
  - `Commit` v0 is implemented with a UI Gallery demo + diag gate.
  - `Artifact` v0 is implemented with a UI Gallery demo + diag gate.
  - `StackTrace` v0 is implemented with a UI Gallery demo + diag gate.
  - `TestResults` v0 is implemented with a UI Gallery demo + diag gate.
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
- [ ] AIEL-MVP1-chat-081 Optional attachments chips: add/remove/clear (app performs file picker effects).
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

- [ ] AIEL-MVP2-tools-100 Tool call block: request/response sections, collapse, error state.
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
- [ ] AIEL-MVP1-gates-020 Add at least one unit test per shipped component family asserting a fragile invariant
  (e.g. stick-to-bottom eligibility rules, stable key mapping, overlay dismiss outcomes).
