# VS Code Theme Import (Syntax Colors)

Fret uses Tree-sitter highlight tags (`fret-syntax`) and resolves them via theme tokens:

- `color.syntax.<tag>` (for example: `color.syntax.keyword.operator`)

This document explains how to import a VS Code theme JSON (`tokenColors`) into a Fret `ThemeConfig`
JSON so code views/editors can reuse the same palette.

## Quickstart

Merge VS Code syntax colors into an existing Fret theme:

```sh
cargo run -p fretboard -- theme import-vscode path/to/vscode-theme.json `
  --all-tags `
  --base themes/fret-default-dark.json `
  --out themes/fret-default-dark+vscode.json `
  --report .fret/theme-import-report.json
```

- `--all-tags`: tries to generate `color.syntax.<tag>` for every `fret-syntax` highlight tag.
- `--base`: merges the generated `color.syntax.*` tokens into an existing theme.
- `--report`: writes a JSON report describing how each token was matched.

To generate a patch-only theme (syntax colors only):

```sh
cargo run -p fretboard -- theme import-vscode path/to/vscode-theme.json --all-tags --out themes/vscode-syntax.json
```

## Why there is a mapping layer

VS Code themes are defined over TextMate scopes (`comment`, `keyword.operator`, `string.regexp`, ...),
while Fret highlights are defined over Tree-sitter capture tags (the `fret-syntax` tag set).

There is no universal, official, 1:1 mapping between these two worlds. The importer therefore:

1. Picks a small set of candidate TextMate scopes for a given `fret-syntax` tag (best-effort).
2. Searches VS Code `tokenColors` for the best matching rule (specificity + later rule wins).
3. Emits `color.syntax.<tag>` tokens for Fret to consume (with prefix fallback in UI components).

## Inspecting results

Use `--report` to understand what happened:

- which candidate scopes were considered
- which `tokenColors` rule matched (index + selector + specificity)
- whether a token was produced from a theme rule or forced via a mapping override

This makes the importer behavior reviewable even when the mapping is heuristic.

## Customizing the conversion

### 1) Provide a mapping file (`--map`)

`--map` lets you override candidate scopes or force a foreground color for:

- stable token keys (`tokens.*`, e.g. `color.syntax.comment`)
- per-highlight tags (`highlights.*`, e.g. `keyword.operator`)

Example `mapping.json`:

```json
{
  "version": 1,
  "tokens": {
    "color.syntax.comment": { "foreground": "#7c7c7c" },
    "color.syntax.keyword": { "scopes": ["keyword", "storage.type"] }
  },
  "highlights": {
    "string.regex": { "scopes": ["string.regexp"] },
    "keyword.operator": { "foreground": "rgb(255, 128, 0)" }
  }
}
```

Run:

```sh
cargo run -p fretboard -- theme import-vscode path/to/vscode-theme.json --all-tags --map mapping.json --out themes/out.json
```

### 2) One-off overrides (`--set`)

`--set` is for quick experiments without a mapping file.

- If the key starts with `color.syntax.`, it forces a token key.
- Otherwise, it is treated as a `fret-syntax` highlight tag.

Examples:

```sh
cargo run -p fretboard -- theme import-vscode path/to/vscode-theme.json --all-tags `
  --set color.syntax.comment=#7c7c7c `
  --set keyword.operator=#ff8000 `
  --out themes/out.json
```

## Notes

- The importer currently uses VS Code `tokenColors` only. It does not attempt to interpret
  TextMate grammars or VS Code semantic tokens.
- Not every VS Code theme will specify every scope; missing tokens are expected.
  Fret will fall back to default syntax color behavior when a `color.syntax.*` key is absent.

