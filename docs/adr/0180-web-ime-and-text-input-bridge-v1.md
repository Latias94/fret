# ADR 0180: Web IME and Text Input Bridge (wasm, v1)

- Status: Proposed
- Date: 2026-01-27

## Context

Fret targets desktop first, but explicitly plans for wasm/WebGPU and future mobile platforms.
Text input is a rewrite-prone area unless the platform boundary is decided early.

On native backends (winit), Fret already defines:

- The keyboard/IME split (`Event::TextInput` vs `Event::Ime`) and key suppression rules (ADR 0012).
- Inline preedit rendering and deterministic IME arbitration (ADR 0012 / ADR 0071).
- Candidate window positioning via `Effect::ImeSetCursorArea` (desktop runner integration) (ADR 0012).

On the web, the constraints differ:

- There is no direct equivalent of `Window::set_ime_cursor_area`.
- IME and composition events are delivered through DOM events whose indices and semantics are not
  identical to native APIs (often UTF-16 oriented).
- Mobile browsers frequently require a focused DOM input element for reliable keyboard/IME behavior.

This ADR defines the v1 architecture for a web text input bridge that:

- preserves Fret’s stable core contracts (UTF-8 byte indices, `ImeEvent` shapes),
- keeps policy out of `crates/fret-ui` (ADR 0066),
- and enables a code-editor-grade surface to run in wasm with acceptable IME fidelity.

## Goals

1) Provide reliable web text input and IME behavior for editable text surfaces (including future code editor surfaces).
2) Preserve Fret’s existing input model:
   - `Event::TextInput` carries committed insertion text without control characters (ADR 0012),
   - `Event::Ime` carries preedit/commit state (ADR 0012 / ADR 0071).
3) Provide best-effort caret anchoring for IME UI on platforms where DOM positioning is meaningful
   (especially mobile), without changing the core contract.
4) Keep the implementation portable within the wasm runner/platform crates and avoid leaking DOM
   policy into `crates/fret-ui`.

## Non-goals (v1)

- Perfect parity with native IME candidate window positioning.
- A full accessibility bridge for web equivalent to AccessKit (out of scope here).
- Cross-document clipboard formats beyond plain text.

## Decision

### 1) Introduce a DOM-backed “Text Input Bridge” in the web runner

The wasm runner MUST provide a bridge that owns a hidden DOM text input element and translates DOM events
into Fret events.

Recommended baseline implementation:

- A hidden `<textarea>` (or `<input>` for single-line surfaces) placed in an overlay layer.
- The element is positioned in window coordinates near the caret when available (best-effort).

The bridge is a runner/platform concern and MUST live outside `crates/fret-ui`, e.g.:

- `crates/fret-runner-web` (runner integration), and/or
- `crates/fret-platform-web` (DOM interaction helpers).

Recommended v1 implementation detail:

- Prefer `<textarea>` even for single-line as the default (IME behavior is often more consistent across browsers),
  but allow an optimization path for `<input>` if needed later.

### 2) Focus + enablement is driven by existing effects

The bridge MUST respond to the existing IME effects:

- `Effect::ImeAllow { window, enabled }`
  - `enabled = true` focuses the DOM textarea and enables composition.
  - `enabled = false` blurs the textarea and stops composition.

- `Effect::ImeSetCursorArea { window, rect }`
  - On web, this is treated as “best-effort caret anchoring”:
    - update the textarea’s CSS position so mobile IME UI tends to appear near the caret,
      when supported by the platform/browser.
  - If positioning is unsupported or unreliable, the effect MAY be a no-op.

This preserves ADR 0012’s contract while allowing web-specific behavior behind the runner.

### 3) Event translation rules (DOM → Fret)

The bridge MUST translate DOM events to Fret events as follows:

1) Composition/preedit:
   - `compositionstart` / `compositionupdate` produce `Event::Ime(ImeEvent::Preedit { text, cursor })`
     with best-effort cursor range.
   - `compositionend` produces `Event::Ime(ImeEvent::Commit { text })` when the DOM provides committed text,
     and clears preedit state.

2) Committed input:
   - `beforeinput` / `input` events produce `Event::TextInput(committed_text)` for committed insertions.
   - The bridge MUST filter control characters from `TextInput` (ADR 0012), so Enter/Tab/Backspace
     remain `KeyDown`/commands, not inserted characters.

3) Keyboard events:
   - `keydown` / `keyup` are translated to Fret key events using the existing physical-key strategy
     (ADR 0012 / ADR 0018 / ADR 0091).
   - When a keydown resolves to a command binding, any subsequent DOM text insertion for that keystroke
     MUST be suppressed (ADR 0012).

Recommended event strategy (v1):

- Use `beforeinput` as the primary “text mutation intent” signal when available:
  - `inputType` distinguishes `insertText`, `insertCompositionText`, `insertLineBreak`, `deleteContentBackward`, etc.
  - This helps preserve ADR 0012’s “control keys are not TextInput” rule on web.
- Use `composition*` events for preedit lifecycle, and treat `compositionend` as a commit boundary
  even if browsers also emit an `input` event.

Event ordering and suppression (v1):

- The bridge MUST be robust to browser event ordering differences:
  - Some browsers can still emit an `input` event even if `beforeinput` was handled with
    `preventDefault()`.
  - Some browsers can emit `beforeinput`/`input` around `compositionend` in different orders.
- The bridge MUST avoid “double insert” outcomes by tracking suppression state across event turns:
  - If a keystroke resolved to a command binding (shortcut path), DOM mutation MUST be suppressed.
  - If `compositionend` emitted a commit, any immediate follow-up DOM `input` for the same commit
    MUST be ignored once.
  - If `beforeinput` produced a `TextInput` (and prevented default), any immediate follow-up DOM
    `input` MUST be ignored once.

### 4) Index representation and UTF-16 ↔ UTF-8 conversion

The bridge MUST preserve the public contract:

- all indices exposed to Fret widgets are UTF-8 byte offsets (ADR 0044 / ADR 0071).

When DOM APIs provide selection ranges in UTF-16 code units (typical):

- the bridge MUST convert DOM selection offsets to UTF-8 byte offsets for `ImeEvent` cursor ranges
  and any selection-related events.
- the conversion must clamp to UTF-8 char boundaries and remain deterministic for ill-formed input.

Implementation note (non-normative):

- Maintain a cached mapping table for the current DOM textarea value to convert offsets efficiently,
  and rebuild it only when the DOM value changes.

### 5) Selection synchronization (optional, v1)

The bridge SHOULD observe DOM selection changes and, when appropriate, emit:

- `Event::SetTextSelection { anchor, focus }`

so Fret’s semantics and selection state remain consistent (ADR 0071 / ADR 0033), especially for:

- user-driven selection handles on mobile,
- OS-level selection gestures in the DOM input element.

This is best-effort and may be deferred if the current widget does not rely on DOM-driven selection.

### 6) Large editor surfaces and virtualization

For a code editor surface that virtualizes visible rows (ADR 0175 / ADR 0177), the bridge MUST NOT
require mirroring the entire document into the DOM textarea.

Instead, the bridge SHOULD operate in a “focused editor proxy” mode:

- The DOM textarea holds only the minimal text needed for IME composition and committed insertions.
- The Fret editor remains the source of truth for the full document model.

This avoids O(n) DOM updates for large documents and keeps performance predictable on wasm/mobile.

Recommended proxy strategy (v1):

- Keep the DOM textarea value as a small, controlled buffer representing:
  - the current preedit string (when composing), plus
  - minimal surrounding context (optional; small fixed window) to improve some IME behaviors.
- Never mirror the full document into the DOM element.
- Treat the Fret editor buffer as the source of truth and reconcile DOM state after each commit/cancel.

## Observability (debug-only)

The web runner SHOULD provide lightweight debug counters to explain failures:

- last seen `inputType` and whether it was translated to `TextInput` or `ImeEvent`,
- whether a DOM event was suppressed due to command routing,
- current “bridge focused” state and last caret-rect anchor.

## Consequences

- Web IME support becomes an explicit runner responsibility, avoiding late rewrites inside `crates/fret-ui`.
- The existing core event model remains stable across native and web.
- Code-editor-grade text input becomes feasible on wasm without forcing a DOM-first authoring model.

## Alternatives Considered

1) Rely on `winit` wasm IME as-is without a DOM bridge
   - Rejected: web IME fidelity and mobile keyboard behavior are too inconsistent across browsers.

2) Make `crates/fret-ui` depend on `web-sys` to manage IME directly
   - Rejected: violates layering (ADR 0066) and complicates portability.

3) Use `contenteditable` instead of a textarea
   - Deferred: can offer richer selection behavior, but is significantly more complex and inconsistent.
     A hidden textarea is the v1 baseline; `contenteditable` may be explored as a v2 option.

## Open Questions

1) Mobile-first caret anchoring:
   - Do we want a stricter contract that the bridge must attempt to position the textarea at the caret rect
     on mobile (even if desktop browsers ignore it), or keep it fully best-effort?

2) Web clipboard + selection UX:
   - Some browsers provide better native selection handles when the DOM element visibly matches the text.
      We currently choose the “proxy textarea” approach; if selection UX becomes a priority, we may need a v2
      path using `contenteditable` for visible text surfaces.

## Evidence anchors (implementation)

- UTF-16 ↔ UTF-8 conversion + clamping: `crates/fret-core/src/utf.rs` (tests included).
- Hidden textarea bridge + effects mapping: `crates/fret-platform-web/src/wasm.rs` (`WebPlatformServices` handles `Effect::ImeAllow` / `Effect::ImeSetCursorArea`).
- Command-path suppression + composition/input ordering state: `crates/fret-platform-web/src/ime_dom_state.rs` (`WebImeDomState`).
- Web harness: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_WEB_IME_HARNESS`), `apps/fret-ui-gallery/src/ui.rs` (`preview_web_ime_harness`).

## References

- ADR 0012: Keyboard, IME, and text input model
- ADR 0071: Multiline text input + IME composition contract
- ADR 0044: Text editing state model and commands
- ADR 0066: Runtime contract surface (policy vs mechanism)
- ADR 0175 / ADR 0177: Windowed virtual surfaces and retained hosts (editor-scale virtualization)
- ADR 0092: Crate structure notes for wasm/web integration
