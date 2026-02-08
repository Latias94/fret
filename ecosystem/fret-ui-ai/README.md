# fret-ui-ai

Opinionated AI/chat UI components for Fret, built on top of `fret-ui-shadcn`.

This crate is an ecosystem-layer port inspired by Vercel's `ai-elements`:

- Upstream reference: `repo-ref/ai-elements/packages/elements`
- UI baseline: `repo-ref/ai-elements/packages/shadcn-ui`

The goal is to provide a modular, policy-heavy component surface for "assistant / tooling" UIs
(messages, conversation views, prompts, tool calls, diffs, file trees, etc.) without pushing these
policies into `fret-ui` / `fret-ui-kit`.

## Notes on alignment

This crate tracks upstream outcomes rather than React APIs. For example, AI Elements `message.tsx`
includes:

- `Message` + `MessageContent` for role-aware message chrome
- `MessageActions` + `MessageAction` for per-message affordances (copy, retry, etc.)
- `MessageToolbar` for a “justify-between” row that typically hosts actions + branch selectors

In Fret:

- `MessageAction` is a thin wrapper over `fret-ui-shadcn::Button` + optional tooltip.
- `MessageActionTemplate` is a small convenience for apps/demos to predefine actions (id, label,
  tooltip, icon, handler, role filtering) while keeping effects app-owned.
