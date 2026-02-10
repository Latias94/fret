---
title: AI Elements Port (`fret-ui-ai`)
status: draft
date: 2026-02-05
scope: ecosystem/fret-ui-ai, shadcn recipes reuse, diag gates
---

# AI Elements Port (`fret-ui-ai`) — Workstream

This workstream tracks the port of Vercel's **AI Elements** component taxonomy into Fret’s ecosystem:

- Upstream reference (local snapshot): `$FRET_REPO_REF_ROOT/ai-elements/packages/elements`
- Baseline UI vocabulary: `ecosystem/fret-ui-shadcn` (shadcn/ui v4-aligned taxonomy)
- Goal crate: `ecosystem/fret-ui-ai` (policy-heavy, AI-native surfaces)

This is an **outcomes-first** port: we align behavior and composition outcomes, not React/DOM APIs.

Milestone board (one-screen): `docs/workstreams/ai-elements-port-milestones.md`.

## Version stamp (upstream reference)

The upstream spec for this workstream is the pinned local checkout under
`$FRET_REPO_REF_ROOT/ai-elements` (default in this dev environment: `F:/SourceCodes/Rust/fret/repo-ref`).

Note: `repo-ref` is a local developer asset store and is not required to build the workspace.

- Upstream repo: `vercel/ai-elements`
- Pinned commit: `7401e828fdd893d4371d7e4f6c8ce9e3f423a52b` (commit date `2026-02-05`)

If the pinned upstream changes, update this stamp first so TODOs and behavior discussions remain
anchored.

## Why this workstream exists

AI-native applications share a recurring set of UI surfaces (conversations, streaming messages,
prompt inputs, tool calls, code blocks, file trees). In web land, AI Elements provides this as a
customizable component set built on shadcn. In Fret, we want the same: an ecosystem-layer surface
that lets app authors move fast without pushing policy into `crates/fret-ui`.

## Layering (non-negotiable)

Follow the same split used by `fret-ui-shadcn`:

- `crates/fret-ui`: mechanisms/contracts only (tree/layout/semantics/focus/overlay substrate).
- `ecosystem/fret-ui-kit` + `ecosystem/fret-ui-headless`: headless state machines and reusable infra.
- `ecosystem/fret-ui-shadcn`: shadcn v4 naming + recipes (composition + styling).
- `ecosystem/fret-ui-ai`: **AI-specific policy + composition** (chat/tooling surfaces).

Rule of thumb:

- If it is a **state machine** (selection, disclosure, hover intent, typeahead), it belongs in
  `fret-ui-headless` or `fret-ui-kit`.
- If it is **taxonomy + default styling**, it belongs in `fret-ui-shadcn`.
- If it is **AI/product policy** (tool call presentation, transcript affordances, markdown actions),
  it belongs in `fret-ui-ai`.

See also: `docs/reference-stack-ui-behavior.md`, `docs/radix-primitives-alignment.md`,
`docs/shadcn-declarative-progress.md`.

## Current state (2026-02)

`ecosystem/fret-ui-ai` exists and is wired into UI Gallery with both a long-transcript harness and
an interactive chat demo:

- `AiConversationTranscript` + `MessageParts`: parts-based transcript rendering with stable per-message
  `test_id` prefixes.
- `AiChat`: a default composition shell (transcript + scroll affordance + prompt input + optional
  empty/download parts), intended as a “good starting point” for apps.
- `MessageResponse`: markdown rendering backed by `ecosystem/fret-markdown` with streaming-friendly
  updates and code-block actions (copy + expand/collapse).
- `PromptInput`: textarea + send/stop + disabled/loading states, with keyboard-first selectors.
- `Suggestions` + `Suggestion`: horizontally scrollable suggestion pills row (prompt-adjacent utility).
- `ToolCallBlock` + `SourcesBlock` + `InlineCitation`: initial tooling surfaces (collapsible tool
  calls, sources list, citation highlight selection).
- `ConversationEmptyState` + `ConversationScrollButton` + `ConversationDownload` + `MessageToolbar`:
  conversation/message parts for app composition.
- `messages_to_markdown`: a pure helper used by “download/copy transcript” flows (effects are app-owned).
- `Artifact`: AI Elements-aligned artifact container surface (header + actions + scrollable content).
- `Shimmer`: AI Elements-aligned animated text shimmer surface (`duration` + `spread`).
- `Reasoning`: AI Elements-aligned reasoning disclosure surface (streaming-driven auto-open + timed auto-close + markdown content).
- `FileTree`: AI Elements-aligned nested file tree surface (small trees; per-row actions; no virtualization yet).
- `CodeBlock` + `Snippet`: AI Elements-aligned code artifact surfaces (copy feedback + header slots).
- `Commit`: AI Elements-aligned commit disclosure surface (copy button + file list rows).
- `StackTrace`: AI Elements-aligned stack trace disclosure surface (copy + parsed frames).
- `TestResults`: AI Elements-aligned test results surfaces (summary + suite disclosure + errors).
- `SchemaDisplay`: AI Elements-aligned schema viewer surface (parameters + request/response property trees).
- UI Gallery pages:
  - `AI transcript (torture harness)` (`ai_transcript_torture`): long-scroll virtualization + cache reuse.
  - `AI chat (demo)` (`ai_chat_demo`): interactive demo with `fretboard diag` gates:
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-prompt-input-keyboard.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-streaming-finalize.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-toolcall-collapse.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-sources-collapsible.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-inline-citation-hovercard.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-citation-highlight.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-codeblock-expand.json`
    - `tools/diag-scripts/ui-gallery-ai-chat-demo-export-markdown.json`
  - `AI artifact (demo)` (`ai_artifact_demo`): `Artifact` demo + gate:
    - `tools/diag-scripts/ui-gallery-ai-artifact-demo-close-toggle.json`
  - `AI shimmer (demo)` (`ai_shimmer_demo`): `Shimmer` demo + gate:
    - `tools/diag-scripts/ui-gallery-ai-shimmer-demo-pixels-changed.json`
  - `AI suggestions (demo)` (`ai_suggestions_demo`): `Suggestions`/`Suggestion` demo + gate:
    - `tools/diag-scripts/ui-gallery-ai-suggestions-demo-click.json`
  - `AI file tree (demo)` (`ai_file_tree_demo`): nested file tree demo + gate:
    - `tools/diag-scripts/ui-gallery-ai-file-tree-demo-toggle.json`
    - `tools/diag-scripts/ui-gallery-ai-file-tree-demo-actions.json`
  - `AI code block (demo)` (`ai_code_block_demo`): `CodeBlock` + `Snippet` demo + gate:
    - `tools/diag-scripts/ui-gallery-ai-code-block-demo-copy.json`
  - `AI commit (demo)` (`ai_commit_demo`): `Commit` demo + gate:
    - `tools/diag-scripts/ui-gallery-ai-commit-demo-copy.json`
  - `AI stack trace (demo)` (`ai_stack_trace_demo`): `StackTrace` demo + gate:
    - `tools/diag-scripts/ui-gallery-ai-stack-trace-demo-copy.json`
  - `AI test results (demo)` (`ai_test_results_demo`): `TestResults` demo + gate:
    - `tools/diag-scripts/ui-gallery-ai-test-results-demo-toggle.json`
  - `AI schema display (demo)` (`ai_schema_display_demo`): `SchemaDisplay` demo + gate:
    - `tools/diag-scripts/ui-gallery-ai-schema-display-demo.json`

This is a good foundation, but it is only a small subset of the upstream AI Elements surface.

## Workstream goals

P0 (MVP usability):

- Provide a **usable Chat UI kit**: `Conversation` + `Message` + `PromptInput` + basic tool call blocks.
- Support **streaming assistant output** (incremental append) with markdown/code rendering outcomes.
- Make performance characteristics explicit: long transcripts should stay stable under view-cache reuse.

P1 (tooling UIs):

- Provide reusable building blocks for “assistant/tooling” apps: sources, citations, file trees, code artifacts.

P2 (workflow + voice):

- Map workflow surfaces onto existing ecosystem crates (`fret-canvas`, `fret-node`, viewports).
- Voice surfaces only if/when there is a concrete app consumer.

## Non-goals (explicit)

- API compatibility with React AI Elements (no JSX/DOM prop mirroring).
- A CLI installer like `npx ai-elements` (not needed inside a Rust workspace).
- Pixel-perfect parity with upstream across all fonts/DPIs (we gate outcomes, not screenshots).

## Port strategy (use Fret’s existing strengths)

1. **Inventory + mapping**
   - Track the upstream component list (by file in `$FRET_REPO_REF_ROOT/ai-elements/packages/elements/src`).
   - For each component: decide the owning layer and required dependencies (shadcn primitives vs new headless).

2. **Build thin, composable surfaces**
   - Prefer “small parts” (container/content/actions) over monolith widgets.
   - Favor controlled/uncontrolled models where it improves app integration (Radix-like patterns).

3. **Gate behavior early**
   - Add stable `test_id` on roots and rows.
   - Add `fretboard diag` scripts for interaction-heavy surfaces.
   - Add unit tests for invariants and state machines (avoid renderer dependencies when possible).

### Running diag gates (recommended)

Prefer launching a prebuilt UI Gallery binary (instead of nesting `cargo run` under `--launch`) so
script startup is stable on slower machines:

```powershell
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag run tools/diag-scripts/<script>.json `
  --dir target/fret-diag-<name> --timeout-ms 300000 --poll-ms 200 --pack --include-all `
  --launch -- target/release/fret-ui-gallery.exe
```

4. **Dogfood in UI Gallery**
   - Each new component family gets a UI Gallery page that can be used for perf/interaction regression.

## Priority roadmap (next)

This list is intentionally short. The rule is: **close one user-visible gap, then add/repair a gate**.

P0 (Chat usability + correctness):

- **PromptInput attachments parity** (`prompt-input.tsx`) (done; gated):
  - attachments chips row (`add/remove/clear`) + add-attachments action (plus button emits an app-owned intent) with app-owned effects (file picker / open URL).
  - keyboard parity: `Enter` submits (IME-safe; `Shift+Enter` inserts newline), `Backspace` on empty input removes the last attachment.
  - height constraints: align the textarea “grows but clamps” behavior (min/max height outcome).
  - gate: `tools/diag-scripts/ui-gallery-ai-chat-demo-prompt-attachments-backspace-enter.json`
- Blocker: **clipboard file/image paste** needs a runtime/platform capability (Fret currently has clipboard text effects only).
- **Regression gates**:
  - add a `fretboard diag` script that covers the attachment keyboard behaviors (add → backspace remove → send).
  - keep stable selectors on the prompt textarea + attachment row.

P1 (AI Elements “extras” surfaces):

- **FileTree audit + extension seams**:
  - re-check `FileTree / Folder / File / Icon / Name` decomposition against upstream `file-tree.tsx` and local reference apps.
  - document extension points (custom row actions, selection model, large-tree strategy).

P2 (platform capability parity, if required by upstream behavior):

- **Clipboard file/image paste**:
  - option A (preferred long-term): add `ClipboardGetFiles`/`ClipboardGetImage` style effects + platform completions (native + web).
  - option B (short-term fallback): support “drop files onto the prompt input” as the primary ergonomic path.

## Upstream parity focus (2026-02)

The pinned upstream (`repo-ref/ai-elements/packages/elements/src`) is the spec. For the next phase,
we focus on closing parity gaps for the two most user-visible “AI Elements” surfaces:

### `sources.tsx` (Collapsible list)

Upstream behavior:

- `Sources` is a `Collapsible` root with a trigger that reads “Used N sources”.
- The content is hidden by default and expands/collapses with lightweight motion.
- Each `Source` is a link (`target="_blank"`) with an icon + title.

Fret mapping:

- `SourcesBlock` becomes a Collapsible-based surface (apps still own effects; link activation emits
  `on_open_url` intents).
- Stable selectors remain mandatory:
  - root: `${msg_prefix}sources-{part_index}`
  - trigger: `${root}-trigger`
  - content: `${root}-content`
  - rows: `${msg_prefix}source-row-{part_index}-{row_index}`

### `message.tsx` (message container + content + actions)

Upstream behavior:

- `Message` is a role-aware layout wrapper (user messages align right and use a different chrome).
- `MessageContent` defines the “bubble” styling for user messages and a plain flow for assistant messages.
- `MessageActions` / `MessageAction` standardize per-message icon buttons and optional tooltips.
- Optional “branching” surfaces exist (`MessageBranch*`) for multi-variant assistant messages.

Fret mapping (planned):

- Keep `MessageParts` as the “content router” for markdown/tool/sources/citations.
- Add a `Message` composition surface that matches upstream decomposition:
  - `Message` (role-aware row wrapper),
  - `MessageContent` (bubble chrome),
  - `MessageActions` + `MessageAction` (tooltip-ready shadcn icon buttons).
- Branch selector: only implement once there is an app consumer; keep the contract in docs first.

Implementation notes (current):

- `Message`, `MessageContent`, `MessageActions`, and `MessageAction` exist in `fret-ui-ai`.
- `MessageParts` now renders via `Message` + `MessageContent` so transcripts align with the same
  decomposition.
- UI Gallery includes a tooltip-gated example action (`ui-gallery-ai-chat-action-copy`).

### `tool.tsx` (Tool call disclosure)

Upstream behavior:

- A tool call is a `Collapsible` with:
  - a header trigger (`ToolHeader`) showing a wrench icon, a derived title, and a status badge,
  - a content panel (`ToolContent`) with “Parameters” and “Result/Error” sections,
  - JSON-like payloads rendered as code blocks.

Fret mapping:

- `Tool` / `ToolHeader` / `ToolContent` / `ToolInput` / `ToolOutput` live in `fret-ui-ai` as
  shadcn-aligned building blocks.
- `ToolCallBlock` remains the model-specific convenience wrapper rendered by `MessageParts`.
- Stable selectors remain mandatory:
  - root: `${msg_prefix}toolcall-{part_index}`
  - trigger: `${msg_prefix}toolcall-trigger-{part_index}`

### `inline-citation.tsx` (HoverCard + pager)

Upstream behavior:

- Citation trigger is a rounded “badge” that shows `hostname` for the first source and `+N` for
  additional sources.
- Hovering opens a `HoverCard` (open/close delay 0) with a small pager:
  - prev/next arrows,
  - index label (`current/count`),
  - body content per source.

Fret mapping:

- `InlineCitation` evolves into a HoverCard-based surface built from shadcn primitives:
  - `HoverCard` + `HoverCardContent`,
  - rounded-full outline buttons for pager arrows,
  - message-local selection model for “highlight selected source” (optional extra).
- Data model requirement: a single citation may reference **multiple sources**.

## How to execute (workflow)

Recommended loop (mirrors how `fret-ui-shadcn` work is kept honest):

1. **Pick the upstream spec** for the component (docs + source):
   - Upstream component source: `$FRET_REPO_REF_ROOT/ai-elements/packages/elements/src/<component>.tsx`
   - Upstream docs page (when helpful): `$FRET_REPO_REF_ROOT/ai-elements/apps/docs/content/components/**/<component>.mdx`
2. **Map to the correct layer** (mechanism vs headless vs shadcn recipe vs ai policy).
3. **Implement the smallest composable surface** that matches outcomes (split container/content/actions).
4. **Add gates immediately**:
   - stable `test_id` targets,
   - at least one unit/invariant test for the most fragile rule,
   - a `fretboard diag` script if interaction/state machines are involved.

If the work involves aligning to shadcn/Radix behavior, follow the same rules used by shadcn work:

- Layering + upstream-as-spec: `docs/reference-stack-ui-behavior.md`
- Mapping guide: `docs/radix-primitives-alignment.md`
- Declarative-only + parity gating mindset: `docs/shadcn-declarative-progress.md`

## Public API rules (v0)

These are the authoring/API constraints that keep `fret-ui-ai` scalable and consistent with the
shadcn ecosystem. Treat them as “should” rules unless a TODO explicitly says otherwise.

### 1) Prefer “parts” over monolith widgets

Upstream AI Elements is composed of small parts (e.g. `ConversationContent`, `MessageActions`). In
Rust, mirror this as composable element types rather than a single mega-struct with many options.

Targets:

- `Conversation` (root) + `ConversationContent` + `ConversationEmptyState` + `ConversationScrollButton` + `ConversationDownload`
- `Message` (root) + `MessageContent` + `MessageActions` + `MessageToolbar` + `MessageResponse`
- `PromptInput` (root) + typed slots (textarea, send/stop, attachments, suggestions)

Why: parts make it possible to reuse the same policy surface in different apps without adding a
combinatorial explosion of parameters.

### 2) Controlled/uncontrolled models (Radix-like)

When a component has meaningful state (open/closed, selected tab, input text), prefer a Radix-like
surface:

- Controlled: app provides a `Model<T>` (and optionally a callback/action hook).
- Uncontrolled: component owns state via `cx.with_state(...)` with sensible defaults.

Do not require `fret-app` as a dependency for the crate; use the portable model types already
available via `fret-ui`/`fret-runtime` where possible.

### 3) Effects are app-owned; components emit intents

AI components should not perform host effects directly (file picker, open-url, clipboard, network).

Instead:

- emit intents via action hooks (`OnActivate`, dismiss hooks, “submit” callbacks),
- accept app-provided handlers (e.g. `on_open_url`, `on_copy`) where needed,
- keep any helper functions pure (e.g. `messages_to_markdown`).

### 4) Stable selectors are mandatory

Every interactive/virtualized surface must expose stable selectors:

- `test_id` on roots and rows (virtual list items),
- `test_id` on key actions (send/stop/collapse/copy),
- semantics roles when applicable (list/listitem, button, textbox).

This is required for `fretboard diag` scripts and for long-term refactors.

Convention (v0; subject to refinement):

- Messages (transcript): `ui-ai-msg-{message_id}-...`
- Prompt input (example app prefix): `<prefix>prompt-textarea`, `<prefix>prompt-send`, `<prefix>prompt-stop`
- Tool call blocks: `<msg_prefix>toolcall-{part_index}`, trigger `<msg_prefix>toolcall-trigger-{part_index}`
- Sources blocks: `<msg_prefix>sources-{part_index}`, rows `<msg_prefix>source-row-{part_index}-{row_index}`
- Inline citations: `<msg_prefix>citation-{part_index}-{citation_index}` (and optional highlight: `<source_row_test_id>-active`)
- Markdown code actions: `<msg_prefix>code-expand-{ordinal}`

### 5) Keep policy out of `crates/fret-ui`

If implementing a feature tempts you to add a runtime public API, stop and justify it against ADR
0066. The default is to:

- add headless state in `fret-ui-headless` / `fret-ui-kit`,
- wire it from `fret-ui-shadcn` (generic recipe) or `fret-ui-ai` (AI-specific policy).

## Data model contract (v0 draft)

The upstream repo uses `ai` package types (e.g. `UIMessage`) and React-specific composition.
`fret-ui-ai` needs a minimal, portable data model that supports the same outcomes without locking
us into a single LLM SDK.

This section defines the **minimum** contract we should converge on early.

### Identifiers and roles

- `MessageId`: **`u64`** (aligned with `crates/fret-ui::ItemKey = u64`).
- Optional: `external_id: Arc<str>` for interoperability with upstream/SDK IDs (UUID/nanoid/string).
- `MessageRole`: `{ user, assistant, system, tool }` (plus optional `data` if needed later).

#### Why `MessageId = u64` (decision rationale)

Fret’s virtualization and windowed surfaces key off `ItemKey`, which is `u64` today (`crates/fret-ui`).
`ConversationTranscript` already depends on this for stable virtual list identity.

Requiring a `u64` message ID gives us:

- deterministic keys across builds/platforms (no randomized hash seeds),
- no hidden collision risk,
- trivial performance characteristics (copyable, compact),
- straightforward integration with view-cache + retained virtualization.

Interoperability with upstream/string IDs is handled by storing `external_id` (for app logic) and
deriving `MessageId` deterministically at the app boundary.

Recommended derivation strategy:

- If your upstream gives you an ordered stream, use a **monotonic counter** (best).
- If you need to key by a string ID, use a **stable hash with a fixed seed** at the app boundary and
  **detect duplicates** during message list construction (fail fast in debug builds).
  - `fret-ui-ai` provides helpers: `message_id_from_external_id(external_id: &str)` and
    `message_id_from_salted_external_id(salt: u64, external_id: &str)`.
  - Reminder: hash collisions are possible; if you cannot tolerate collisions, keep a per-transcript
    mapping table and assign monotonic `MessageId`s.
- Avoid using `Vec` indices as IDs if you ever insert/remove in the middle of the transcript (it
  breaks keyed identity and can cause per-row state to “jump”).

### Message content model

Prefer a “parts” model rather than a single `text` field, so we can render rich assistant outputs
without retrofitting later.

Implemented in `ecosystem/fret-ui-ai/src/model.rs` as:

- `AiMessage { id, external_id?, role, parts: Arc<[MessagePart]> }`
- `MessagePart` variants:
  - `Text(Arc<str>)` (typically user input)
  - `Markdown(MarkdownPart { text: Arc<str>, finalized: bool })` (assistant output; supports streaming)
  - `ToolCall(ToolCall)` (structured input/output + lifecycle)
  - `Sources(Arc<[SourceItem]>)`
  - `Citations(Arc<[CitationItem]>)` (inline citations referencing a `Sources` part)

Notes:

- Use `Arc<[T]>` for `parts` to keep `AiMessage` cheap to clone while still allowing apps to own the
  source-of-truth message list.
- “Attachments parts” and richer “code block parts” are deferred until we have a stable authoring
  contract for file pickers / bytes ownership / per-block state. The standalone attachments UI
  surfaces (`Attachments` / `Attachment*`) can still be used in prompt composers and tool panels.

### Tool call lifecycle

Tool calls should have an explicit lifecycle state so the UI can show running/progress/error:

- AI Elements-aligned states:
  - `ApprovalRequested` / `ApprovalResponded`
  - `InputStreaming` / `InputAvailable`
  - `OutputAvailable` / `OutputDenied` / `OutputError`

Represent `input`/`output` as structured values (e.g. JSON string or `serde_json::Value`) so apps
can choose how to present them.

### Sources and citations

Minimum data for sources:

- stable `source_id`
- `title`
- optional `url`
- optional `snippet` / `excerpt`

Inline citations should reference `source_id` and a stable anchor key (message id + part index +
optional range), so scripts can select and highlight deterministically.

### Streaming updates

We need a contract for “assistant is streaming” without locking into a networking stack:

- append-only updates for `MessagePart::Markdown` via `MarkdownPart.text` while `finalized == false`,
- stable block identifiers for code fences so per-block state (copy/expand) is preserved while
  text grows,
- a `finalized = true` moment that flushes pending blocks (e.g. unterminated code fences) and enables
  heavier post-processing if needed (syntax highlight, linkify).

## Theme tokens (v0 strategy)

Goal: keep AI components visually consistent with the shadcn preset, while allowing AI-specific
policies to introduce new tokens without polluting global shadcn namespaces.

Rules:

1. Prefer existing shadcn tokens (`background`, `card`, `muted`, `border`, `primary`, typography
   metrics) before adding new ones.
2. When new tokens are required, add them under a Fret-owned namespace:
   - `fret.ai.*` (canonical)
3. Keep token usage outcome-driven (spacing, radii, row height, focus ring), not implementation-driven.

Initial token candidates (names tentative; keep the list small):

Token list (v0):

| Token key | Kind | Default (fallback) | Notes |
| --- | --- | --- | --- |
| `fret.ai.conversation.padding` | metric (px) | `Space::N4` | Padding for transcript content area. |
| `fret.ai.conversation.gap` | metric (px) | `Space::N8` | Gap between transcript rows. |
| `fret.ai.conversation.scroll_button.offset_bottom` | metric (px) | `Space::N4` | Offset for “scroll to bottom” affordance. |
| `fret.ai.message.user.bg` | color | `primary` | User message background. |
| `fret.ai.message.user.fg` | color | `primary-foreground` | User message foreground. |
| `fret.ai.message.assistant.bg` | color | `card` | Assistant message background. |
| `fret.ai.message.assistant.fg` | color | `foreground` | Assistant message foreground. |
| `fret.ai.message.system.bg` | color | `muted` | System message background. |
| `fret.ai.message.system.fg` | color | `foreground` | System message foreground. |
| `fret.ai.message.tool.bg` | color | `secondary` | Tool message background. |
| `fret.ai.message.tool.fg` | color | `foreground` | Tool message foreground. |
| `fret.ai.prompt_input.min_height` | metric (px) | `Space::N10` | Minimum height for multi-line prompt input (if/when enabled). |
| `fret.ai.prompt_input.max_height` | metric (px) | `Px(240.0)` | Maximum height for multi-line prompt input (if/when enabled). |

Implemented today:

- `fret.ai.message.*.bg` / `fret.ai.message.*.fg` are read by `Message` and `MessageParts` with
  shadcn fallback tokens when not configured.
- `fret.ai.conversation.padding` / `fret.ai.conversation.gap` and
  `fret.ai.conversation.scroll_button.offset_bottom` are used as defaults by transcript surfaces.
- `fret.ai.prompt_input.min_height` is used as a default by `PromptInput`.

Avoid hard-coded string keys like `theme.color_required("primary")` in `fret-ui-ai` where a
tokenized mapping is feasible.

## Reuse map (existing building blocks)

This is a pragmatic “what do we already have?” map to avoid reinventing infrastructure.

- Markdown + streaming:
  - `ecosystem/fret-markdown` (including `MarkdownStreamState` and `MarkdownComponents` policies)
- Code blocks + highlighting:
  - `ecosystem/fret-code-view` + `ecosystem/fret-syntax`
- Prompt input primitives:
  - `ecosystem/fret-ui-shadcn::{InputGroup, Textarea, Button, Spinner, CommandPalette, Select, DropdownMenu, Tooltip}`
- Virtualization:
  - `crates/fret-ui` VirtualList + scroll handles (already used by `ConversationTranscript`)
- Overlays + dismissal/focus policy:
  - `ecosystem/fret-ui-kit` overlay request orchestration + headless intent state machines
- Diagnostics + gates:
  - `apps/fretboard` (`diag`) + `tools/diag-scripts/*` + stable `test_id` selectors

If a ported AI component needs something not listed here, the default path is:

1. add a headless primitive/state machine in `fret-ui-headless` (unit-tested),
2. wire it through `fret-ui-kit` helpers if it is reusable infra,
3. expose a recipe surface in `fret-ui-shadcn` if it is generally useful,
4. keep AI-specific composition/policy in `fret-ui-ai`.

## Regression gates (v1)

This workstream should reuse the existing diagnostics + scripting infrastructure (ADR 0174). A
minimum v1 gate set:

1. Long transcript:
   - wheel scroll for N frames,
   - capture bundle,
   - check stale paint,
   - check view-cache reuse stability (when enabled).
2. Prompt input (keyboard-only):
   - focus input, type, submit, stop/cancel,
   - capture bundle for deterministic selectors.
3. Tool call collapse/expand:
   - toggle disclosure, ensure semantics/test_id remain stable.

Concrete script names and gating checklists live in the TODO tracker:
`docs/workstreams/ai-elements-port-todo.md`.

## Known pitfalls (v1)

### Diag automation: scroll container selection and semantics wrappers

If multiple semantics nodes accidentally share the same `test_id` (e.g. a layout-transparent
semantics decorator plus an inner leaf), a naive “pick deepest match” strategy can select a tiny
wrapper node. For scroll-driven automation, this matters because wheel routing depends on the
hit-test target.

Current mitigation (diagnostics):

- `scroll_into_view` container selection prefers a container candidate that is an ancestor of the
  target semantics node and has the largest bounds (more likely to be the true scroll surface).
- `fret-ui-ai` parts use layout-transparent semantics wrappers for `test_id` anchors so adding
  selectors does not change layout or hit-test routing.

If these gates regress again:

- Capture a bundle at the failure step and inspect `debug.semantics` for duplicate `test_id` nodes.
- If a selector matches multiple candidates, prefer a selector that is path-qualified (role + path)
  or add a container/root `test_id` that is unique.

### Sticky bottom chrome and safe insets

Even with correct container selection, targets can land too close to sticky prompt chrome (the
prompt panel). Keep `scroll_into_view.padding_insets_px.bottom_px` conservative in scripts so click
targets land above the sticky region.

Diagnostics gotcha: Fret element identity uses callsite + key hashing. If you render the same
"logical list" multiple times from the same callsite (e.g. grid/inline/list variants), key by a
tuple like `(variant, id)` or wrap each variant section in a dedicated `cx.keyed(...)` scope to
avoid collisions (missing semantics anchors / state jumping).

## Component inventory (upstream → Fret mapping)

The upstream AI Elements repo groups components into (chatbot / code / workflow / voice / utilities).
We port in the same order, but we lean on existing Fret ecosystem crates.

Canonical list source: `$FRET_REPO_REF_ROOT/ai-elements/packages/elements/src/*.tsx`.

Tracking and milestones live in: `docs/workstreams/ai-elements-port-todo.md`.

Legend:

- **Owner**: which crate should own the surface (not necessarily where every helper lives).
- **Status**: `Done` | `Partial` | `Planned (M1/M2/...)` | `Defer` (no consumer) | `N/A` (not applicable in Fret).

| Upstream (AI Elements) | Fret owner (candidate) | Fret module/path (candidate) | Status | Notes |
| --- | --- | --- | --- | --- |
| `conversation.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/conversation.rs` | Partial | Transcript exists; parts for empty/download/scroll button exist as separate surfaces (see `conversation_*` files). |
| `message.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/message.rs` | Partial | Role chrome exists; parts-based composition is available via `MessageParts` + `MessageToolbar` (action policies remain app-owned). Message branching surfaces are ported (`ecosystem/fret-ui-ai/src/elements/message_branch.rs`) + UI Gallery demo + diag gate pass (`tools/diag-scripts/ui-gallery-ai-message-branch-demo-wrap.json`). |
| (subset) | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/message_response.rs` | Partial | Markdown rendering exists; streaming append + finalize supported; richer per-block actions (copy/download) are TODO. |
| `prompt-input.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/prompt_input.rs` | Done | MVP: textarea + send/stop + disabled/loading + attachments row + add-attachments intent + stable selectors. |
| `tool.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/tool_call_block.rs` | Partial | Tool call block exists (collapsible + state chrome); richer payload views are pending. |
| `sources.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/sources_block.rs` | Partial | Sources list exists; v0 highlight contract supports “select citation → highlight source row”. |
| `inline-citation.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/inline_citation.rs` | Partial | Citation chrome exists; v0 select/highlight contract is implemented via a shared model. |
| `attachments.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/attachments.rs` | Prototype | UI surfaces exist (grid/inline/list) + UI Gallery demo + diag gate pass (`tools/diag-scripts/ui-gallery-ai-attachments-demo-remove.json`); hover uses `HoverRegion` to keep remove affordance interactive while pointer is over the nested button. File pick/open effects remain app-owned. |
| `code-block.tsx` | `fret-ui-ai` + `fret-code-view` | `ecosystem/fret-markdown` + `ecosystem/fret-code-view` | Partial | Code fences render via markdown; copy/expand/download actions need slots. |
| `snippet.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/snippet.rs` | Done | Inline copyable surface + copy feedback + UI Gallery demo + diag gate. |
| `file-tree.tsx` | `fret-ui-ai` + `fret-ui-kit` | `ecosystem/fret-ui-ai/src/elements/file_tree.rs` | Done | Nested file tree surface (small trees) + per-row actions + UI Gallery demo + diag gates; future: virtualized outline path via UI Kit tree recipes. |
| `artifact.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/artifact.rs` | Done | Artifact container surface (header + actions + scrollable content) + UI Gallery demo + diag gate. |
| `image.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/image.rs` | Prototype | Renders an `ImageId` (decode/upload policy remains app-owned). |
| `audio-player.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/audio_player.rs` | Defer | Depends on audio backend + buffering policy. |
| `shimmer.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/shimmer.rs` | Done | Animated text shimmer surface (`duration` + `spread`) + UI Gallery demo + diag gate. |
| `toolbar.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/message_toolbar.rs` | Done | Message toolbar part; composes shadcn buttons + menus (policy app-owned). |
| `suggestion.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/suggestions.rs` | Done | Suggestions row + pill surfaces + UI Gallery demo + diag gate (`tools/diag-scripts/ui-gallery-ai-suggestions-demo-click.json`). |
| `reasoning.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/reasoning.rs` | Prototype | Auto-open while streaming (unless `default_open=false`), auto-close once (1s after stream end), duration accounting, markdown content. |
| `chain-of-thought.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/chain_of_thought.rs` | N/A | Avoid baking “CoT UI” as a default surface without a consumer. |
| `plan.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/plan.rs` | Defer | Could be a markdown-like block with disclosure; wait for consumer. |
| `stack-trace.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/stack_trace.rs` | Done | Stack trace disclosure surface + parsed frames + copy feedback + UI Gallery demo + diag gate. |
| `terminal.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/terminal.rs` | Defer | Prefer tying to existing terminal viewport/runner if present. |
| `schema-display.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/schema_display.rs` | Done | Schema viewer surface + UI Gallery demo + diag gate. |
| `jsx-preview.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/jsx_preview.rs` | Defer | Would need a sandboxed renderer/preview system. |
| `web-preview.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/web_preview.rs` | Defer | Needs webview/viewport integration. |
| `sandbox.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/sandbox.rs` | Defer | Depends on execution sandbox and policies. |
| `test-results.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/test_results.rs` | Done | Test results surfaces + suite disclosure + UI Gallery demo + diag gate. |
| `checkpoint.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/checkpoint.rs` | Defer | Workflow-specific; not core chat UI. |
| `queue.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/queue.rs` | Prototype | Queue surfaces + UI Gallery demo + diag gate pass (`tools/diag-scripts/ui-gallery-ai-queue-demo-section-scroll-action.json`); hover uses `HoverRegion` (group-hover parity), list cap uses `ScrollArea` root `max_h`. |
| `task.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/task.rs` | Defer | Workflow-specific. |
| `agent.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/agent.rs` | Defer | Likely app-specific persona chrome. |
| `persona.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/persona.rs` | Defer | Same. |
| `model-selector.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/model_selector.rs` | Defer | Only if app needs it; depends on overlay/select recipes. |
| `mic-selector.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/mic_selector.rs` | Defer | Voice surfaces are optional. |
| `voice-selector.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/voice_selector.rs` | Defer | Voice surfaces are optional. |
| `speech-input.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/speech_input.rs` | Defer | Depends on audio/ASR stack. |
| `transcription.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/transcription.rs` | Defer | Voice pipeline dependent. |
| `controls.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/controls.rs` | Defer | Only if it maps to app-level transport controls. |
| `confirmation.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/confirmation.rs` | Defer | Likely a dialog/sheet recipe (shadcn owner), not AI-specific by default. |
| `context.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/context.rs` | Defer | Needs a “context items” data model + file references. |
| `open-in-chat.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/open_in_chat.rs` | Defer | App-specific affordance. |
| `panel.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/panel.rs` | Defer | Workspace shell/panels belong in docking/viewports workstreams. |
| `canvas.tsx` | `fret-canvas` + `fret-ui-ai` | `ecosystem/fret-canvas` (core) + `fret-ui-ai` chrome | Defer | Only when chat embeds interactive canvases. |
| `node.tsx` / `edge.tsx` | `fret-node` + `fret-ui-ai` | `ecosystem/fret-node` (core) + `fret-ui-ai` chrome | Defer | Same. |
| `commit.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/commit.rs` | Done | Commit disclosure surface + copy feedback + UI Gallery demo + diag gate. |
| `connection.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/connection.rs` | Defer | Workflow-specific. |
| `environment-variables.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/environment_variables.rs` | Defer | Workflow-specific. |
| `package-info.tsx` | `fret-ui-ai` | `ecosystem/fret-ui-ai/src/elements/package_info.rs` | Defer | Workflow-specific. |

## Risks & design constraints (track proactively)

- **Virtualization with variable row heights**: transcript rows will become “rich” (markdown, tool blocks).
  The virtualization contract must remain correct without forcing cache-root rerender loops.
- **Selection/copy** in transcript: define the contract early (single message vs multi-message selection).
- **Accessibility semantics**: transcripts and message actions must expose stable semantics roles/labels.
- **Host effects** (file pickers, open-url, clipboard): components should emit intents, apps perform effects.
- **Theme token ownership**: AI components should use shadcn tokens where possible and introduce new tokens
  under a Fret-owned namespace when needed (avoid ad-hoc `Theme::color_required(...)` strings).

## Deliverables

- A coherent `fret-ui-ai` public surface (documented in crate README and UI Gallery usage docs).
- A TODO tracker with milestones and regression gates: `docs/workstreams/ai-elements-port-todo.md`.
- At least one scripted regression suite for long transcripts and prompt input interactions.

## Milestones (definition of done)

This section is intentionally “contract-adjacent”: it defines what “we can ship and rely on”
without prescribing exact implementation details.

### M0 — Foundations

Definition of done:

- `fret-ui-ai` has a stable public module layout and naming conventions.
- Every shipped surface has stable `test_id` anchors on:
  - root element,
  - key interactive affordances (buttons/inputs),
  - any virtualized rows/items.
- At least one `fretboard diag` script covers the long-transcript harness.

Evidence (expected):

- `docs/workstreams/ai-elements-port-todo.md` M0 items marked `[x]`
- `tools/diag-scripts/ui-gallery-ai-transcript-torture-scroll.json` (or equivalent) gated by `fretboard diag`

### M1 — Chat surfaces MVP

Definition of done:

- There is a composable, app-usable surface for:
  - `Conversation` (including empty state + scroll affordances),
  - `Message` (container/content/actions slots),
  - `MessageResponse` (markdown/code rendering, streaming-friendly),
  - `PromptInput` (text + send/stop + disabled/loading outcomes).
- A UI Gallery page demonstrates “chat demo” (not only torture).
- A keyboard-only diag script can drive prompt input and submit/cancel.

Evidence (expected):

- `tools/diag-scripts/ui-gallery-ai-chat-demo-prompt-input-keyboard.json` (or equivalent)
- `tools/diag-scripts/ui-gallery-ai-chat-demo-streaming-finalize.json`
- `tools/diag-scripts/ui-gallery-ai-chat-demo-export-markdown.json`

### M2 — Tooling surfaces MVP

Definition of done:

- Tool calls can be displayed in a transcript (input/output, running/success/error, collapse).
- Sources/citations can be displayed with deterministic selectors and “open url” intents.
- Interaction-heavy behavior is gated by at least one diag script + one Rust invariant test.

Evidence (expected):

- `tools/diag-scripts/ui-gallery-ai-chat-demo-toolcall-collapse.json`
- `tools/diag-scripts/ui-gallery-ai-chat-demo-citation-highlight.json`

### M3 — Code artifacts MVP

Definition of done:

- Code blocks/snippets are backed by `fret-code-view` + `fret-syntax`, with:
  - copy actions,
  - long-code scrolling outcomes,
  - stable block IDs for per-block state.
- File-tree output is usable for small trees (AI Elements parity), and there is a clear large-tree
  strategy (prefer `fret-ui-kit` virtualization/retained helpers to avoid new engines in `fret-ui-ai`).

Later milestones (workflow/voice) are explicitly optional and should not be started without a
concrete app consumer.
