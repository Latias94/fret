# ADR 0259: System Font Rescan + Injected Font Retention (Parley/fontique)

- Status: Proposed
- Date: 2026-02-11

## Context

Fret’s renderer shapes text via Parley and enumerates/selects fonts via fontique. Runners expose a best-effort font
catalog to settings UIs via runtime globals (ADR 0258), and the text system supports injecting font bytes at runtime
(`Effect::TextAddFonts`).

Two operational needs show up in editor-grade apps:

1) **System font changes**: users install/remove fonts while the app is running. We need a way to rescan system fonts
   without restarting.
2) **Injected font persistence across rescan**: if the app injected fonts (bundled UI fonts, icon fonts, etc), a system
   rescan must keep those injected fonts available; otherwise the font DB would “forget” them after the rescan.

Naively keeping every injected font blob forever risks unbounded memory growth (hot reloads, repeated injections).
Naively performing system rescans on the UI thread risks noticeable stalls during font enumeration.

## Goals

1) Provide a **native-only** “rescan system fonts” surface that can be invoked explicitly.
2) Prefer an **async-by-default** rescan on desktop to avoid blocking the UI thread.
3) Preserve **injected fonts** across a system rescan, with a bounded retention policy.
4) Keep wasm deterministic: no system font rescans; rely on injected/bundled fonts only.

## Non-goals

- Guarantee that system font enumeration is stable across machines.
- Provide a user-facing settings UI for rescan in v1 (runner surface is sufficient).
- Make injected font retention lossless under arbitrary memory budgets.

## Decision

### 1) Runner-owned explicit rescan effect (native-only)

The runtime exposes `Effect::TextRescanSystemFonts` (triggered by a command), and runners decide how to implement it:

- Desktop runner: run the rescan and update `FontCatalogMetadata` via ADR 0258’s no-op revisioning rules.
- Web runner: ignore (no system fonts).

### 2) Async-by-default desktop rescan with request coalescing

On desktop platforms, the runner performs system font rescans on a background thread by default and applies the result
on the main thread.

Repeated rescan requests while a rescan is in-flight are coalesced: at most one additional rescan is queued.

Set `FRET_TEXT_SYSTEM_FONT_RESCAN_ASYNC=0` to force a synchronous rescan path (debugging/triage).

### 3) Bounded injected font retention (dedupe + LRU eviction)

The renderer retains injected font bytes (from `Effect::TextAddFonts`) so it can re-register them during a system rescan.

This retention cache is:

- **deduped** (content equality),
- **LRU-evicted** under soft caps:
  - `FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT` (default: `256`, max: `4096`)
  - `FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES` (default: `268435456` = 256 MiB, max: 2 GiB)

If a font blob is evicted, it may not survive a subsequent system rescan.

## Consequences

- System font rescans do not have to block the UI thread on desktop (default).
- Injected fonts remain available across rescans as long as their blobs are retained.
- Under tight budgets, injected fonts may be dropped across a rescan boundary; this is an explicit tradeoff to prevent
  unbounded growth.
- Runners can treat catalog revision as “effective catalog changed” (ADR 0258), even when rescans occur defensively.

## Implementation anchors

- Desktop async rescan (coalesced + env gate):
  - `crates/fret-launch/src/runner/desktop/mod.rs` (`request_system_font_rescan`, `apply_pending_system_font_rescan_result`)
- Renderer seed/result split (background compute + main-thread apply):
  - `crates/fret-render-wgpu/src/text/mod.rs` (`SystemFontRescanSeed`, `SystemFontRescanResult`)
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs` (`run_system_font_rescan`)
- Injected font blob retention (dedupe + budgets):
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs` (`record_registered_font_blob`)

## Related docs

- ADR 0258: `docs/adr/0258-font-catalog-refresh-and-revisioning-v1.md`
- Workstream: `docs/workstreams/font-system-v1.md`
- Audit note: `docs/audits/font-system-parley-zed-xilem-2026-02.md`

