---
title: AI Elements Port (`fret-ui-ai`) — Milestones
status: active
date: 2026-02-12
scope: ecosystem/fret-ui-ai, diag gates, upstream parity tracking
---

# AI Elements Port (`fret-ui-ai`) — Milestones

This document is a **one-screen milestone board** for the AI Elements port.

Source of truth for detailed TODOs: `docs/workstreams/ai-elements-port-todo.md`.
Narrative + contracts: `docs/workstreams/ai-elements-port.md`.
Upstream inventory + mapping: `docs/workstreams/ai-elements-upstream-alignment.md`.

## Definition of done (port-level)

We consider a component family “ported” when:

1. **Outcomes match upstream** for the documented behaviors (not API/props).
2. **Stable selectors** exist for automation (`test_id` on roots/rows/actions).
3. At least one **regression gate** exists:
   - a `fretboard diag` script for interactive/stateful surfaces, and/or
   - a targeted Rust invariant test for non-interactive geometry/contract rules.
4. The surface lives in the **correct layer** (mechanisms vs shadcn recipes vs AI policy).

## Milestones

### M0 — Foundations (contracts + crate shape)

Acceptance criteria:

- `ecosystem/fret-ui-ai` public surface area is shaped around **composition** (container/content/actions),
  not monolith widgets.
- Minimal **data model v0** exists (message role + parts + tool calls + sources + citations).
- Minimal **`fret.ai.*` token namespace** exists (keep it small; prefer shadcn tokens first).
- UI Gallery has at least one **AI demo page** wired (so we can dogfood quickly).

Status: In progress (foundation exists; keep tightening contracts).

### M1 — Chat MVP (conversation + message + prompt)

Scope:

- `conversation.tsx`, `message.tsx`, `prompt-input.tsx` parity pass.
- `tool.tsx` parity pass for “collapsible tool call” outcomes.
- Markdown streaming outcome parity (append + finalize) via `ecosystem/fret-markdown`.

Acceptance criteria:

- Keyboard-first prompt input gate passes (`tools/diag-scripts/ui-gallery-ai-chat-demo-prompt-input-keyboard.json`).
- Tool call disclosure gate passes (`tools/diag-scripts/ui-gallery-ai-chat-demo-toolcall-collapse.json`).
- Streaming finalize gate passes (`tools/diag-scripts/ui-gallery-ai-chat-demo-streaming-finalize.json`).

Status: In progress.

Notes:

- `Reasoning` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-reasoning-demo-auto-open-close.json`
- `Queue` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-queue-demo-section-scroll-action.json`
- `Attachments` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-attachments-demo-remove.json`
  - Verified PASS: 2026-02-10 (local).
- `MessageBranch` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-message-branch-demo-wrap.json`
  - Verified PASS: 2026-02-10 (local).
- `Tool` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-tool-demo-toggle.json`
- `ModelSelector` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-model-selector-demo-open-filter-select.json`
- `ChainOfThought` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-chain-of-thought-demo-toggle.json`
- `PromptInput` attachments + keyboard behavior (Enter/Backspace + add-attachments action + file drop + provider mode) is aligned with upstream `prompt-input.tsx` and gated:
  - `tools/diag-scripts/ui-gallery-ai-chat-demo-prompt-attachments-backspace-enter.json`
  - Verified PASS: 2026-02-10 (local).
- `PromptInput` attachments constraints (`accept` / `multiple` / `maxFiles` / `maxFileSize` / `onError`) are implemented for external drops and gated with unit tests:
  - `ecosystem/fret-ui-ai/src/elements/prompt_input.rs` (`prompt_input_drop_respects_max_files_and_emits_error`, `prompt_input_drop_accept_and_size_errors_do_not_add_attachments`)
  - Verified PASS: 2026-02-10 (local).
- `PromptInput` action menu parts are implemented and gated:
  - `tools/diag-scripts/ui-gallery-ai-prompt-input-action-menu-demo.json`
- `PromptInput` referenced sources (local to prompt input) are implemented and gated:
  - `tools/diag-scripts/ui-gallery-ai-prompt-input-referenced-sources-demo.json`
- Next parity focus inside M1 is PromptInput “parts-first” decomposition so apps can compose prompt chrome without forking:
  - Target: `PromptInputBody` / `PromptInputTextarea` / `PromptInputHeader` / `PromptInputFooter` / `PromptInputTools` /
    `PromptInputButton` / `PromptInputSubmit` / `PromptInputActionMenu*` / `PromptInputActionAddAttachments`.
  - Add a UI Gallery provider-mode composition demo + diag gate:
    - `tools/diag-scripts/ui-gallery-ai-prompt-input-provider-demo.json`
- Next parity focus inside M1 is clipboard file/image paste (blocked on a runtime/platform capability; clipboard effects are text-only today).

### M2 — Tooling UIs (sources + citations)

Scope:

- `sources.tsx` Collapsible outcomes.
- `inline-citation.tsx` HoverCard + pager outcomes.

Acceptance criteria:

- Sources Collapsible gate passes (`tools/diag-scripts/ui-gallery-ai-chat-demo-sources-collapsible.json`).
- Inline citation HoverCard gate passes (`tools/diag-scripts/ui-gallery-ai-chat-demo-inline-citation-hovercard.json`).
- Citation highlight gate passes (`tools/diag-scripts/ui-gallery-ai-chat-demo-citation-highlight.json`).

Status: Done (gates passing).

Notes:

- `Checkpoint` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-checkpoint-demo-tooltip.json`
- `Confirmation` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-confirmation-demo-approve.json`
- `EnvironmentVariables` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-environment-variables-demo-toggle-copy.json`
- `Plan` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-plan-demo-toggle.json`
- `Task` v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-task-demo-toggle.json`

### M3 — Code artifacts (developer-facing outputs)

Scope:

- `code-block.tsx`, `snippet.tsx`, `file-tree.tsx` (and supporting utilities).

Acceptance criteria:

- Code fences render with stable per-block actions (copy / expand / download) and preserve per-block
  state during streaming growth.
- `CodeBlock` / `Snippet` are backed by `ecosystem/fret-code-view` (no new engines).
- `FileTree` preserves keyed identity (no state jumping) and has a clear large-tree strategy
  (prefer `fret-ui-kit` virtualization/retained helpers).

Status: In progress (core artifacts are gated; parity polish pending).
Notes:

- `FileTree` is ported with UI Kit flatten + `VirtualList` (virtualized under height constraints) and gated:
  - `tools/diag-scripts/ui-gallery-ai-file-tree-demo-toggle.json`
  - `tools/diag-scripts/ui-gallery-ai-file-tree-demo-actions.json`
  - `tools/diag-scripts/ui-gallery-ai-file-tree-large-scroll.json`
- `CodeBlock` / `Snippet` v0 are ported with a UI Gallery demo + diag gate (`ui-gallery-ai-code-block-demo-copy.json`).
- `Artifact` v0 is ported with a UI Gallery demo + diag gate (`ui-gallery-ai-artifact-demo-close-toggle.json`).
- `Shimmer` v0 is ported with a UI Gallery demo + diag gate (`ui-gallery-ai-shimmer-demo-pixels-changed.json`).
- `Commit` v0 is ported with a UI Gallery demo + diag gate (`ui-gallery-ai-commit-demo-copy.json`).
- `Commit` large-list demo + scroll/click seam gate exists (`ui-gallery-ai-commit-large-scroll.json`).
- `StackTrace` v0 is ported with a UI Gallery demo + diag gate (`ui-gallery-ai-stack-trace-demo-copy.json`).
- `StackTrace` large-list demo + scroll/click seam gate exists (`ui-gallery-ai-stack-trace-large-scroll.json`).
- `Terminal` v1 is ported as a viewer-only output surface (monospace output + copy/clear + auto-scroll) and gated:
  - `tools/diag-scripts/ui-gallery-ai-terminal-demo-copy-clear.json`
- `TestResults` v0 is ported with a UI Gallery demo + diag gate (`ui-gallery-ai-test-results-demo-toggle.json`).
- `TestResults` large-list demo + scroll/activate seam gate exists (`ui-gallery-ai-test-results-large-scroll.json`).
- `SchemaDisplay` v0 is ported with a UI Gallery demo + diag gate (`ui-gallery-ai-schema-display-demo.json`).
- `WebPreview` chrome v0 is ported with a UI Gallery demo + diag gate:
  - `tools/diag-scripts/ui-gallery-ai-web-preview-demo-commit-console.json`
  - Optional native embedding is available behind `wry` (see `docs/workstreams/webview-wry-v1.md`) with gates:
    - `tools/diag-scripts/ui-gallery-ai-web-preview-demo-webview-wry-nav.json`
    - `tools/diag-scripts/ui-gallery-ai-web-preview-demo-webview-wry-console.json`
    - `tools/diag-scripts/ui-gallery-ai-web-preview-demo-webview-wry-console-clear.json`
Remaining work (current focus):

- Tighten parity for `Commit` / `StackTrace` / `TestResults` (layout/styling + copy + extension hooks).
- Keep gates stable and add missing selectors for edge cases (long lists / deep disclosures).

### M4 — Workflow surfaces (optional; only if we can reuse existing crates)

Scope:

- Minimal chrome recipes over existing ecosystem crates (`fret-node`, `fret-canvas`, docking/viewports).

Acceptance criteria:

- No “new engines” in `fret-ui-ai`; only composition/policy wrappers.

Status: In progress (chrome-only; feature-gate heavy integrations).

Progress:

- `WorkflowPanel` / `WorkflowToolbar` / `WorkflowControls` chrome are ported and demoed on
  `PAGE_AI_WORKFLOW_CHROME_DEMO` with gates:
  - `tools/diag-scripts/ui-gallery-ai-workflow-chrome-demo.json`
  - `tools/diag-scripts/ui-gallery-ai-workflow-controls-demo.json`
- `WorkflowNode` chrome is ported and demoed on the same page with gate:
  - `tools/diag-scripts/ui-gallery-ai-workflow-node-demo.json`

### M5 — Voice surfaces (defer until there is a concrete consumer)

Scope:

- `audio-player` (UI-only chrome is ported; playback backend is app-owned).
- `transcription` (segment surface is ported; playback timing remains app-owned).
- `mic-selector`, `speech-input`, `voice-selector` (UI-only chrome is ported; capture/ASR/preview are app-owned).

Acceptance criteria:

- Backends/policies are explicit; UI remains intent-driven (apps own side effects).

Status: Done (UI-only voice chrome ported; UI Gallery demos + diag gates exist; backends are app-owned).

### M6 — Upstream coverage closure (all files accounted for)

Scope:

- Port workflow wrapper surfaces as chrome-only wrappers (no new engines inside `fret-ui-ai`).
- Explicitly mark upstream-only web surfaces as `N/A` where appropriate (e.g. `jsx-preview.tsx`), with rationale.

Acceptance criteria:

- Every upstream `.tsx` has a corresponding Rust module (or is explicitly marked `N/A` with rationale).
- Each newly ported surface has stable selectors (`test_id`) and at least one regression gate
  (diag script preferred for interactive surfaces).
- Heavy dependencies and backends are feature-gated; UI surfaces remain intent-driven.

Status: In progress (remaining workflow wrappers: `canvas.tsx`, `edge.tsx`, `connection.tsx`).

## Next-step checklist (recommended weekly cadence)

- Update the upstream “Version stamp” first when `repo-ref/ai-elements` changes.
- Pick 1–2 upstream components max, align outcomes, then add/repair gates immediately.
- Keep `docs/workstreams/ai-elements-port-todo.md` honest (mark blockers with a concrete bundle + step id).
