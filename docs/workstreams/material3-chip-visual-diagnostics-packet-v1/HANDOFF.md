# Material 3 Chip Visual Diagnostics Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

Chip visual diagnostics are closed for current representative gallery states.

What changed:

- Added `ui-gallery-material3-chip-visual-chrome.json`.
- Added root redirect and suite manifest entries so the diagnostics script is discoverable through
  the registry.
- Matrix rows for Chip, FilterChip, InputChip, and SuggestionChip now record visual diagnostics
  alignment instead of open conditional visual follow-ons.
- No component code, foundation policy, kit policy, or mechanism code changed.

Resume guidance:

- Use the new visual diagnostics script before changing chip spacing/elevation behavior.
- Keep ChipSet roving policy recipe-owned until another design system needs the same policy.
