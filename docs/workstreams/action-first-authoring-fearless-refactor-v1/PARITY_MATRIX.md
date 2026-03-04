# Action-First Authoring + View Runtime (Fearless Refactor v1) — Parity Matrix

Last updated: 2026-03-04

This matrix is a checklist of **outcomes**, not 1:1 API parity.

Legend:

- `✅` supported / landed
- `⚠️` partial
- `❌` not yet
- `🧭` intentionally different (document rationale)

| Area | GPUI/Zed reference | Fret today | Target (v1) | Notes |
| --- | --- | --- | --- | --- |
| Typed unit actions | `gpui::action` | ✅ `ActionId` alias + typed unit action macro + handler table | ✅ | v1 landed; dispatch remains command-based at the mechanism layer, but authoring binds `ActionId` and handlers live in the view/app layer |
| Structured action payloads | Keymap JSON payload | ❌ | ❌ (v2) | Gate behind strict validation and determinism rules |
| Key contexts | `key_context("...")` | ✅ `UiBuilder::key_context` / `AnyElement::key_context` + `keyctx.*` `when` identifiers + shortcut routing trace includes `key_contexts[*]` | ✅ | Key contexts are derived from the focused node chain (or barrier root fallback) and are visible in `wait_shortcut_routing_trace` gates via `key_context` queries |
| Availability queries | `is_action_available` | ✅ action availability snapshot + diagnostics traces + scripted gates | ✅ | v1 landed (including modal-barrier gating gates) |
| Command palette integration | actions visible as commands | ✅ command palette dispatches via the same action/command pipeline | ✅ | metadata registry is unified (no duplication) |
| View-level dirty/notify loop | “notify → dirty views” | ✅ view runtime provides `notify()` dirty marking + reuse closure | ✅ | v1 landed (minimal); perf hardening continues in dedicated perf workstreams |
| View cache reuse semantics | `AnyView::cached` | ✅ cache-root reuse reasons are diagnosable + scripted reuse gate exists | ✅ | v1 landed (minimal); keep reuse reasons explicit and gateable |
| Immediate-mode frontend | n/a (GPUI is declarative-first) | ✅ (`fret-imui`) | ✅ | Must dispatch actions without string glue |
| Data-driven UI spec frontend | not first-class in GPUI | ✅ (GenUI) | ✅ | Align action IDs and metadata where possible |
