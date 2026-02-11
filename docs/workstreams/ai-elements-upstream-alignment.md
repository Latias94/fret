---
title: AI Elements upstream alignment (inventory + mapping)
status: active
date: 2026-02-11
scope: ecosystem/fret-ui-ai, repo-ref/ai-elements
---

# AI Elements upstream alignment (inventory + mapping)

This document is the mechanical source-of-truth for **what exists upstream** and how it maps into
`ecosystem/fret-ui-ai`.

Workstream narrative: `docs/workstreams/ai-elements-port.md`.
TODO tracker: `docs/workstreams/ai-elements-port-todo.md`.

## Upstream snapshot

Local checkout (developer machine asset, not part of this repo):

- `F:\SourceCodes\Rust\fret\repo-ref\ai-elements`

Version stamp:

- Repo: `vercel/ai-elements`
- Commit: `e2045329c8445ebd0523de6aa755a39d6193841f` (date `2026-02-06`)

## Inventory source

Upstream exports each `.tsx` file directly:

- `F:\SourceCodes\Rust\fret\repo-ref\ai-elements\packages\elements\src\*.tsx`

## Naming notes (TSX → Rust)

Upstream uses kebab-case filenames (e.g. `prompt-input.tsx`), while Rust modules use snake_case
(e.g. `prompt_input.rs`).

Some upstream "single surfaces" are intentionally split into multiple composable parts in Fret for
app-owned effects and better reuse. Example: upstream `sources.tsx` maps to `SourcesBlock` +
supporting types in `sources_block.rs`.

## Mapping table (current)

Legend:

- **Ported (prototype)**: exists in `fret-ui-ai`, but may still diverge from upstream composition or
  interaction details.
- **Not started**: no Fret surface yet (or intentionally deferred).

| Upstream file | Fret owner | Rust module | Status | Notes |
| --- | --- | --- | --- | --- |
| `conversation.tsx` | `fret-ui-ai` | `conversation*.rs`, `ai_chat.rs`, `ai_conversation.rs` | Ported (prototype) | Parts-first transcript + default shell (`AiChat`). |
| `message.tsx` | `fret-ui-ai` | `message*.rs`, `message_response.rs` | Ported (prototype) | Split into message wrapper, toolbar/actions, and response rendering. |
| `prompt-input.tsx` | `fret-ui-ai` | `prompt_input.rs` | Ported (prototype) | Effects remain app-owned; interaction gated via diag. |
| `tool.tsx` | `fret-ui-ai` | `tool.rs`, `tool_call_block.rs` | Ported (prototype) | Collapsible tool call blocks + status outcomes. |
| `sources.tsx` | `fret-ui-ai` | `sources_block.rs`, `inline_citation.rs` | Ported (prototype) | Split into `SourcesBlock` + `InlineCitation` policy. |
| `inline-citation.tsx` | `fret-ui-ai` | `inline_citation.rs` | Ported (prototype) | HoverCard outcomes + selection seam. |
| `reasoning.tsx` | `fret-ui-ai` | `reasoning.rs` | Ported (prototype) | Streaming-driven auto-open/close policy. |
| `suggestion.tsx` | `fret-ui-ai` | `suggestions.rs` | Ported (prototype) | Fret uses plural module name; surfaces `Suggestions`/`Suggestion`. |
| `queue.tsx` | `fret-ui-ai` | `queue.rs` | Ported (prototype) | Policy-heavy; gated via diag. |
| `model-selector.tsx` | `fret-ui-ai` | `model_selector.rs` | Ported (prototype) | Provider icons are placeholders (no remote fetch). |
| `attachments.tsx` | `fret-ui-ai` | `attachments.rs` | Ported (prototype) | Chips grid; add/remove intents are app-owned. |
| `chain-of-thought.tsx` | `fret-ui-ai` | `chain_of_thought.rs` | Ported (prototype) | Step-list disclosure surface. |
| `checkpoint.tsx` | `fret-ui-ai` | `checkpoint.rs` | Ported (prototype) | Basic alignment. |
| `confirmation.tsx` | `fret-ui-ai` | `confirmation.rs` | Ported (prototype) | Approve/deny policy surface. |
| `context.tsx` | `fret-ui-ai` | `context.rs` | Ported (prototype) | Context usage hovercard (percent + progress + compact counts). |
| `plan.tsx` | `fret-ui-ai` | `plan.rs` | Ported (prototype) | Plan item list outcomes. |
| `shimmer.tsx` | `fret-ui-ai` | `shimmer.rs` | Ported (prototype) | Animated shimmer text surface. |
| `artifact.tsx` | `fret-ui-ai` | `artifact.rs` | Ported (prototype) | Artifact container chrome. |
| `code-block.tsx` | `fret-ui-ai` | `code_block.rs` | Ported (prototype) | Backed by `ecosystem/fret-code-view`. |
| `snippet.tsx` | `fret-ui-ai` | `snippet.rs` | Ported (prototype) | Copyable snippet outcomes. |
| `file-tree.tsx` | `fret-ui-ai` | `file_tree.rs` | Ported (prototype) | Flattens via UI Kit tree + `VirtualList`; small-tree focused. |
| `commit.tsx` | `fret-ui-ai` | `commit.rs` | Ported (prototype) | Disclosure + copy actions + file rows. |
| `stack-trace.tsx` | `fret-ui-ai` | `stack_trace.rs` | Ported (prototype) | Parsed frames + copy + seams. |
| `schema-display.tsx` | `fret-ui-ai` | `schema_display.rs` | Ported (prototype) | Schema viewer outcomes. |
| `test-results.tsx` | `fret-ui-ai` | `test_results.rs` | Ported (prototype) | Suite/test disclosures + optional activate seam. |
| `environment-variables.tsx` | `fret-ui-ai` | `environment_variables.rs` | Ported (prototype) | Table-like key/value outcomes. |
| `web-preview.tsx` | `fret-ui-ai` | `web_preview.rs` | Ported (prototype) | Chrome always available; native embed via `webview-wry` behind feature flags. |
| `image.tsx` | `fret-ui-ai` | `image.rs` | Ported (prototype) | Rendering only; decoding/upload is app-owned. |
| `terminal.tsx` | `fret-ui-ai` | `terminal.rs` | Ported (prototype) | Viewer-only (output + copy/clear + auto-scroll); no PTY/TTY in v1. |

## Known upstream files not yet ported

As of the snapshot above, these upstream surfaces do not exist as `fret-ui-ai` ports yet:

- `agent.tsx`
- `audio-player.tsx`
- `jsx-preview.tsx` (likely out of scope for Rust)
- `open-in-chat.tsx`
- `package-info.tsx`
- `persona.tsx`
- `sandbox.tsx`
- `task.tsx`
- Workflow wrappers: `canvas.tsx`, `node.tsx`, `edge.tsx`, `panel.tsx`, `toolbar.tsx`, `controls.tsx`, `connection.tsx`
- Voice surfaces: `mic-selector.tsx`, `speech-input.tsx`, `transcription.tsx`, `voice-selector.tsx`

## Regenerating this diff (developer note)

PowerShell snippet used for the inventory diff (update paths as needed):

```powershell
$up = 'F:\SourceCodes\Rust\fret\repo-ref\ai-elements\packages\elements\src'
$rs = 'F:\SourceCodes\Rust\fret-worktrees\ai-elements-port\ecosystem\fret-ui-ai\src\elements'

$upNorm = (Get-ChildItem $up -File -Filter '*.tsx').BaseName | ForEach-Object { $_ -replace '-', '_' }
$rsNames = (Get-ChildItem $rs -File -Filter '*.rs').BaseName | Where-Object { $_ -ne 'mod' }

'missing:'; $upNorm | Where-Object { $rsNames -notcontains $_ } | Sort-Object
```
