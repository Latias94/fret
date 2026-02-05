---
title: AI Elements Port (`fret-ui-ai`)
status: draft
date: 2026-02-05
scope: ecosystem/fret-ui-ai, shadcn recipes reuse, diag gates
---

# AI Elements Port (`fret-ui-ai`) — Workstream

This workstream tracks the port of Vercel's **AI Elements** component taxonomy into Fret’s ecosystem:

- Upstream reference: `repo-ref/ai-elements/packages/elements`
- Baseline UI vocabulary: `ecosystem/fret-ui-shadcn` (shadcn/ui v4-aligned taxonomy)
- Goal crate: `ecosystem/fret-ui-ai` (policy-heavy, AI-native surfaces)

This is an **outcomes-first** port: we align behavior and composition outcomes, not React/DOM APIs.

## Version stamp (upstream reference)

The upstream spec for this workstream is the pinned local checkout under `repo-ref/ai-elements`.

- Upstream repo: `vercel/ai-elements`
- Pinned commit: `e7566cacc888d41cedd2a41510d2cf0df36928da` (commit date `2026-02-04`)

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

`ecosystem/fret-ui-ai` exists and is wired into UI Gallery as a long-transcript harness:

- `ConversationTranscript`: virtualized transcript + stick-to-bottom + scroll-to-bottom affordance.
- `Message`: minimal message bubble (role-based background).
- UI Gallery torture page: `AI Transcript / Long Conversation Harness` (virtualization + cache reuse).

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
   - Track the upstream component list (by file in `repo-ref/ai-elements/packages/elements/src`).
   - For each component: decide the owning layer and required dependencies (shadcn primitives vs new headless).

2. **Build thin, composable surfaces**
   - Prefer “small parts” (container/content/actions) over monolith widgets.
   - Favor controlled/uncontrolled models where it improves app integration (Radix-like patterns).

3. **Gate behavior early**
   - Add stable `test_id` on roots and rows.
   - Add `fretboard diag` scripts for interaction-heavy surfaces.
   - Add unit tests for invariants and state machines (avoid renderer dependencies when possible).

4. **Dogfood in UI Gallery**
   - Each new component family gets a UI Gallery page that can be used for perf/interaction regression.

## How to execute (workflow)

Recommended loop (mirrors how `fret-ui-shadcn` work is kept honest):

1. **Pick the upstream spec** for the component (docs + source):
   - Upstream component source: `repo-ref/ai-elements/packages/elements/src/<component>.tsx`
   - Upstream docs page (when helpful): `repo-ref/ai-elements/apps/docs/content/components/**/<component>.mdx`
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

### Message content model

Prefer a “parts” model rather than a single `text` field, so we can render rich assistant outputs
without retrofitting later.

Draft shape (not yet code):

- `Message` has `Vec<MessagePart>`.
- `MessagePart` can include:
  - `Markdown { text, stream_id? }`
  - `ToolCall { id, name, status, input, output, error }`
  - `Sources { items }`
  - `Attachment { id, kind, name, bytes? }` (bytes are app-owned; UI just displays metadata)
  - `CodeBlock { language?, text, block_id }` (usually produced by markdown rendering)

### Tool call lifecycle

Tool calls should have an explicit lifecycle state so the UI can show running/progress/error:

- `Pending` → `Running` → `Succeeded | Failed | Cancelled`

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

- append-only updates for markdown/text parts,
- stable block identifiers for code fences so per-block state (copy/expand) is preserved while
  text grows,
- a `finalize()` moment that enables heavier post-processing if needed (syntax highlight, linkify).

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

- Transcript layout:
  - `fret.ai.conversation.padding`
  - `fret.ai.conversation.gap`
  - `fret.ai.conversation.scroll_button.offset_bottom`
- Message chrome:
  - `fret.ai.message.user.bg` / `.fg`
  - `fret.ai.message.assistant.bg` / `.fg`
  - `fret.ai.message.system.bg` / `.fg`
  - `fret.ai.message.tool.bg` / `.fg`
- Prompt input:
  - `fret.ai.prompt_input.min_height`
  - `fret.ai.prompt_input.max_height`

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

## Component inventory (upstream → Fret mapping)

The upstream AI Elements repo groups components into (chatbot / code / workflow / voice / utilities).
We port in the same order, but we lean on existing Fret ecosystem crates.

Canonical list source: `repo-ref/ai-elements/packages/elements/src/*.tsx`.

Tracking and milestones live in: `docs/workstreams/ai-elements-port-todo.md`.

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
- `tools/diag-scripts/ui-gallery-ai-transcript-scroll.json` (or an equivalent script) gated by `fretboard diag`

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

- `tools/diag-scripts/ui-gallery-ai-prompt-input-keyboard.json` (or equivalent)

### M2 — Tooling surfaces MVP

Definition of done:

- Tool calls can be displayed in a transcript (input/output, running/success/error, collapse).
- Sources/citations can be displayed with deterministic selectors and “open url” intents.
- Interaction-heavy behavior is gated by at least one diag script + one Rust invariant test.

### M3 — Code artifacts MVP

Definition of done:

- Code blocks/snippets are backed by `fret-code-view` + `fret-syntax`, with:
  - copy actions,
  - long-code scrolling outcomes,
  - stable block IDs for per-block state.
- File-tree output is usable (virtualized) and stable under view-cache reuse.

Later milestones (workflow/voice) are explicitly optional and should not be started without a
concrete app consumer.
