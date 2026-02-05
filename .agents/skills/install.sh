#!/usr/bin/env bash
#
# Fret skills installer (repo-local source -> project-local agent directory)
#
# This script copies skill folders (each containing `SKILL.md`) from this repo's `.agents/skills/`
# directory into a target project's agent skills directory:
#
# - Claude Code: <project>/.claude/skills/
# - Codex CLI:   <project>/.agents/skills/
# - Gemini CLI:  <project>/.gemini/skills/
#
# Examples:
#   ./.agents/skills/install.sh --agent claude-code
#   ./.agents/skills/install.sh --agent codex --target /path/to/project --force
#   ./.agents/skills/install.sh --agent claude-code --skills fret-diag-workflow,fret-shadcn-app-recipes

set -euo pipefail

AGENT="claude-code"
TARGET="."
SKILLS=""
FORCE=false
LIST=false
DRY_RUN=false

usage() {
  cat <<'EOF'
Usage: install.sh [OPTIONS]

Options:
  --agent AGENT       Agent: claude-code|codex|gemini (default: claude-code)
  --target DIR        Target project directory (default: .)
  --skills LIST       Comma-separated skill names (default: all fret-*)
  --force             Overwrite existing installed skills
  --list              List available skills and exit
  --dry-run           Print actions without changing files
  --help              Show this help and exit
EOF
}

info()  { printf "[INFO] %s\n" "$1"; }
warn()  { printf "[WARN] %s\n" "$1" >&2; }
ok()    { printf "[OK]   %s\n" "$1"; }
die()   { printf "[ERROR] %s\n" "$1" >&2; exit 1; }

while [[ $# -gt 0 ]]; do
  case "$1" in
    --agent) AGENT="${2:-}"; shift 2 ;;
    --target) TARGET="${2:-}"; shift 2 ;;
    --skills) SKILLS="${2:-}"; shift 2 ;;
    --force) FORCE=true; shift ;;
    --list) LIST=true; shift ;;
    --dry-run) DRY_RUN=true; shift ;;
    --help) usage; exit 0 ;;
    *) die "Unknown option: $1" ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_ABS="$(cd "$TARGET" 2>/dev/null && pwd)" || die "Target does not exist: $TARGET"

normalize_agent() {
  case "$1" in
    claude|claude-code) echo "claude-code" ;;
    codex) echo "codex" ;;
    gemini) echo "gemini" ;;
    *) die "Unknown agent: $1 (expected claude-code, codex, gemini)" ;;
  esac
}

skills_dest_dir() {
  case "$1" in
    codex) echo "$TARGET_ABS/.agents/skills" ;;
    gemini) echo "$TARGET_ABS/.gemini/skills" ;;
    claude-code) echo "$TARGET_ABS/.claude/skills" ;;
    *) die "Unknown agent: $1" ;;
  esac
}

AGENT="$(normalize_agent "$AGENT")"
DEST_DIR="$(skills_dest_dir "$AGENT")"

mapfile -t AVAILABLE < <(find "$SCRIPT_DIR" -maxdepth 1 -mindepth 1 -type d -name 'fret-*' -print | sort)

if [[ ${#AVAILABLE[@]} -eq 0 ]]; then
  die "No skills found under $SCRIPT_DIR (expected folders like fret-*/SKILL.md)"
fi

if $LIST; then
  info "Available skills in $SCRIPT_DIR:"
  for p in "${AVAILABLE[@]}"; do
    name="$(basename "$p")"
    if [[ -f "$p/SKILL.md" ]]; then
      printf "  - %s\n" "$name"
    fi
  done
  exit 0
fi

declare -a REQUESTED=()
if [[ -n "$SKILLS" ]]; then
  IFS=',' read -r -a REQUESTED <<<"$SKILLS"
else
  for p in "${AVAILABLE[@]}"; do
    if [[ -f "$p/SKILL.md" ]]; then
      REQUESTED+=("$(basename "$p")")
    fi
  done
fi

if [[ ${#REQUESTED[@]} -eq 0 ]]; then
  die "No skills selected"
fi

info "Source: $SCRIPT_DIR"
info "Target: $TARGET_ABS"
info "Agent:  $AGENT"
info "Dest:   $DEST_DIR"
info "Skills: ${REQUESTED[*]}"

if $DRY_RUN; then
  info "Dry run: no files will be changed"
fi

if [[ ! -d "$DEST_DIR" ]]; then
  if $DRY_RUN; then
    info "Dry run: would create $DEST_DIR"
  else
    mkdir -p "$DEST_DIR"
  fi
fi

for name in "${REQUESTED[@]}"; do
  src="$SCRIPT_DIR/$name"
  dst="$DEST_DIR/$name"

  [[ -d "$src" ]] || die "Missing source skill folder: $src"
  [[ -f "$src/SKILL.md" ]] || die "Missing SKILL.md in: $src"

  if [[ -e "$dst" ]]; then
    if $FORCE; then
      if $DRY_RUN; then
        warn "Dry run: would remove existing $dst"
      else
        rm -rf "$dst"
      fi
    else
      warn "Skip (already exists): $dst (use --force to overwrite)"
      continue
    fi
  fi

  if $DRY_RUN; then
    info "Dry run: would copy $src -> $dst"
  else
    cp -R "$src" "$dst"
    ok "Installed: $name"
  fi
done

ok "Done."
