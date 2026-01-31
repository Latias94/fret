# shadcn/ui v4 Audit - Command (cmdk)

This audit compares Fret's shadcn-aligned `Command` surface (backed by cmdk-style behavior) against
the upstream shadcn/ui v4 docs and registry implementations, focusing on **behavioral outcomes**
instead of API compatibility.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/command.mdx`
- shadcn registry (new-york-v4): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/command.tsx`
- cmdk repo: `repo-ref/cmdk`

Key upstream semantics:

- `Command` is a composite (input + results list) where focus typically stays in the input.
- Highlight moves via `aria-activedescendant` (active descendant); `Enter` activates the highlighted item.
- Filtering/ranking is fuzzy-scored; `value`/`keywords` participate in matching.
- Structure primitives exist: `CommandGroup` (optional heading), `CommandSeparator`, `CommandEmpty`.
- Visual conventions commonly used by shadcn: optional check indicator and right-side shortcut text.

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/command.rs`
- Headless scoring/selection: `ecosystem/fret-ui-kit/src/headless/cmdk_score.rs`,
  `ecosystem/fret-ui-kit/src/headless/cmdk_selection.rs`
- Active descendant contract: `docs/adr/0073-active-descendant-and-composite-widget-semantics.md`

## Audit checklist

### Composition & navigation (cmdk-style)

- Pass: `CommandPalette` is a unified implementation (input + list) keeping focus in the input and driving highlight via active-descendant.
- Pass: `ArrowUp/Down`, `Home/End`, `PageUp/PageDown` update highlight; `Enter` triggers `on_select` / `command`.
- Pass: Hover moves highlight without stealing focus (matches cmdk expectations).

### Filtering / ranking semantics

- Pass: `CommandPalette` provides built-in filtering/ranking via `fret-ui-kit::headless::cmdk_score`.
- Pass: `CommandItem.value` participates as an alias when `value != label`.
- Pass: `CommandItem.keywords([...])` aligns with cmdk's `keywords` semantics (matching can hit non-label strings).

### Structure primitives (Group / Separator / Empty)

- Pass: Supports `CommandGroup` (optional heading) and `CommandSeparator`, and trims leading/trailing/consecutive separators after filtering.
- Pass: Shows `empty_text` when no results; provides `CommandEmpty` as a convenience (`CommandPalette::empty(...)`).

### Layout & geometry (new-york-v4)

- Pass: `CommandDemo` matches the upstream split sizing: input wrapper uses `h-9` while the input uses `h-10`
  (the input overflows the wrapper slightly in the web golden).
- Pass: `CommandDialog` matches the upstream overrides (`h-12` wrapper + `h-12` input, and `pt-0` for sibling groups).
- Pass: `CommandPalette` defaults to a `w-full` root layout to avoid cmdk listbox width collapse when embedded in recipes (e.g. `Combobox`).

### Visual/content conventions (shadcn)

- Pass: `CommandItem.checkmark(bool)` + `CommandShortcut` support the common "left check + right shortcut" row layout.
- Pass: `CommandItem.children(...)` allows rich custom row content.
- Pass: Default `CommandPalette` rows can render cmdk-style match highlighting (matched characters use `foreground`; non-matched characters use `muted-foreground`).

### CommandDialog

- Pass: `CommandDialog` supports `entries(...)`, so `CommandGroup/Separator/...` can be used inside a dialog.

## Validation

- `cargo test -p fret-ui-shadcn --lib command::tests`
- shadcn-web golden + gates:
  - Golden: `goldens/shadcn-web/v4/new-york-v4/command-demo.json`
  - Layout gates:
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout -- web_vs_fret_layout_command_demo_input_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout -- web_vs_fret_layout_command_demo_listbox_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout -- web_vs_fret_layout_command_demo_listbox_option_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout -- web_vs_fret_layout_command_demo_listbox_option_insets_match`
  - Golden: `goldens/shadcn-web/v4/new-york-v4/command-dialog.open.json`
  - Chrome gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome -- web_vs_fret_command_dialog_panel_chrome_matches`
  - Placement gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement -- web_vs_fret_command_dialog_overlay_center_matches`
  - List metrics gates (v4 dialog surface overrides):
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement -- web_vs_fret_command_dialog_input_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement -- web_vs_fret_command_dialog_listbox_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement -- web_vs_fret_command_dialog_listbox_option_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement -- web_vs_fret_command_dialog_listbox_option_insets_match`
    - Tiny viewport variants: `*_tiny_viewport` (ensures the web-style “centered even when overflowing” behavior stays aligned)

## Follow-ups (non-P0)

- Composability: if split authoring (`CommandInput`/`CommandList`/`CommandItem`) becomes a goal, introduce a shared context model (query + active + selection) with an explicit contract/ADR first.
