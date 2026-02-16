#!/usr/bin/env python3
"""
Add an "Upstream references" section to docs that reference `repo-ref/<name>` but don't contain
any GitHub links.

Policy (open source):
- `repo-ref/` is optional local state (ignored by git).
- Docs may keep `repo-ref/...` file anchors as a *reading aid*, but should also provide upstream
  GitHub links so readers without local checkouts can follow along.

This script is intentionally conservative:
- Only touches `docs/**/*.md`.
- Only modifies docs that (a) reference `repo-ref/` and (b) contain no `github.com/...` link.
- Inserts a standard section near the top (after front matter or after the first H1).
"""

from __future__ import annotations

import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


def _repo_root() -> Path:
    out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"])
    return Path(out.decode("utf-8").strip()).resolve()


RE_REPO_REF = re.compile(r"repo-ref/([A-Za-z0-9_.-]+)")
RE_GITHUB = re.compile(r"https?://github\.com/[^\s)>\"]+")

SECTION_TITLE = "## Upstream references (non-normative)"


@dataclass(frozen=True)
class Upstream:
    label: str
    url: str | None


UPSTREAM: dict[str, Upstream] = {
    "agentskills": Upstream("AgentSkills (skills-ref)", "https://github.com/agentskills/agentskills"),
    "ai-elements": Upstream("Vercel AI Elements", "https://github.com/vercel/ai-elements"),
    "animata": Upstream("Animata", "https://github.com/codse/animata"),
    "aria-practices": Upstream("WAI-ARIA Authoring Practices", "https://github.com/w3c/aria-practices"),
    "base-ui": Upstream("MUI Base UI", "https://github.com/mui/base-ui"),
    "bevy": Upstream("Bevy", "https://github.com/bevyengine/bevy"),
    "bevy_material_ui": Upstream("bevy_material_ui", "https://github.com/edgarhsanchez/bevy_material_ui"),
    "charming": Upstream("Charming (ECharts helper)", "https://github.com/ecomfe/charming"),
    "cmdk": Upstream("cmdk", "https://github.com/pacocoursey/cmdk"),
    "compose-multiplatform-core": Upstream("Compose Multiplatform (core)", "https://github.com/JetBrains/compose-multiplatform-core"),
    "dear-imgui-rs": Upstream("imgui-rs", "https://github.com/imgui-rs/imgui-rs"),
    "dioxus": Upstream("Dioxus", "https://github.com/DioxusLabs/dioxus"),
    "dockview": Upstream("dockview", "https://github.com/mathuo/dockview"),
    "echarts": Upstream("Apache ECharts", "https://github.com/apache/echarts"),
    "egui": Upstream("egui", "https://github.com/emilk/egui"),
    "egui-snarl": Upstream("egui-snarl", "https://github.com/zakarumych/egui-snarl"),
    "egui_plot": Upstream("egui_plot", "https://github.com/emilk/egui_plot"),
    "egui_tiles": Upstream("egui_tiles", "https://github.com/rerun-io/egui_tiles"),
    "floating-ui": Upstream("Floating UI", "https://github.com/floating-ui/floating-ui"),
    "flutter": Upstream("Flutter", "https://github.com/flutter/flutter"),
    "glide-data-grid": Upstream("Glide Data Grid", "https://github.com/glideapps/glide-data-grid"),
    "godot": Upstream("Godot", "https://github.com/godotengine/godot"),
    "gpui-component": Upstream("gpui-component", "https://github.com/longbridge/gpui-component"),
    "Graphics": Upstream("Unity Graphics (ShaderGraph)", "https://github.com/Unity-Technologies/Graphics"),
    "imgui": Upstream("Dear ImGui", "https://github.com/ocornut/imgui"),
    "imgui-node-editor": Upstream("imgui-node-editor", "https://github.com/thedmd/imgui-node-editor"),
    "ImGuizmo": Upstream("ImGuizmo", "https://github.com/CedricGuillemet/ImGuizmo"),
    "implot": Upstream("ImPlot", "https://github.com/epezent/implot"),
    "implot3d": Upstream("ImPlot3D", "https://github.com/brenocq/implot3d"),
    "json-render": Upstream("json-render", "https://github.com/vercel-labs/json-render"),
    "kibo": Upstream("kibo", "https://github.com/haydenbleasel/kibo"),
    "lucide": Upstream("Lucide", "https://github.com/lucide-icons/lucide"),
    "magicui": Upstream("Magic UI", "https://github.com/magicuidesign/magicui"),
    "makepad": Upstream("Makepad", "https://github.com/makepad/makepad"),
    "material-web": Upstream("Material Web", "https://github.com/material-components/material-web"),
    "monaco-editor": Upstream("Monaco Editor", "https://github.com/microsoft/monaco-editor"),
    "motion": Upstream("Motion", "https://github.com/motiondivision/motion"),
    "parley": Upstream("Parley", "https://github.com/linebender/parley"),
    "primitives": Upstream("Radix UI Primitives", "https://github.com/radix-ui/primitives"),
    "react-bits": Upstream("React Bits", "https://github.com/DavidHDev/react-bits"),
    "react-day-picker": Upstream("react-day-picker", "https://github.com/gpbl/react-day-picker"),
    "react-hook-form": Upstream("react-hook-form", "https://github.com/react-hook-form/react-hook-form"),
    "react-virtualized": Upstream("react-virtualized", "https://github.com/bvaughn/react-virtualized"),
    "react-virtuoso": Upstream("react-virtuoso", "https://github.com/petyosi/react-virtuoso"),
    "router": Upstream("TanStack Router", "https://github.com/TanStack/router"),
    "table": Upstream("TanStack Table", "https://github.com/TanStack/table"),
    "tailwindcss": Upstream("Tailwind CSS", "https://github.com/tailwindlabs/tailwindcss"),
    "tanstack-table": Upstream("TanStack Table", "https://github.com/TanStack/table"),
    "tanstack-virtual": Upstream("TanStack Virtual", "https://github.com/TanStack/virtual"),
    "transform-gizmo": Upstream("transform-gizmo", "https://github.com/urholaukkarinen/transform-gizmo"),
    "ui": Upstream("shadcn/ui", "https://github.com/shadcn-ui/ui"),
    "vello": Upstream("Vello", "https://github.com/linebender/vello"),
    "virtualizer": Upstream("virtualizer (Rust)", "https://github.com/Latias94/virtualizer"),
    "winit": Upstream("winit", "https://github.com/rust-windowing/winit"),
    "xilem": Upstream("Xilem", "https://github.com/linebender/xilem"),
    "xyflow": Upstream("XYFlow", "https://github.com/xyflow/xyflow"),
    "zed": Upstream("Zed", "https://github.com/zed-industries/zed"),
    "zod": Upstream("zod", "https://github.com/colinhacks/zod"),
    # Internal / not publicly available (keep repo-ref anchors for maintainers; do not invent URLs).
    "fret-ui-precision": Upstream("fret-ui-precision", None),
    # Rarely used / informational only; mapping can be added later if needed.
    "icons": Upstream("icons", None),
}


def _extract_repo_refs(text: str) -> list[str]:
    return sorted(set(RE_REPO_REF.findall(text)), key=str.lower)


def _has_upstream_section(text: str) -> bool:
    return SECTION_TITLE in text


def _insert_after_front_matter(text: str, insert: str) -> str | None:
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---\n", 4)
    if end == -1:
        return None
    end += len("\n---\n")
    return text[:end] + "\n" + insert + "\n" + text[end:]


def _insert_after_h1(text: str, insert: str) -> str | None:
    lines = text.splitlines(keepends=True)
    for i, line in enumerate(lines):
        if line.startswith("# "):
            # Insert after the H1 line and any following blank line(s).
            j = i + 1
            while j < len(lines) and lines[j].strip() == "":
                j += 1
            return "".join(lines[:j]) + "\n" + insert + "\n" + "".join(lines[j:])
    return None


def _insert_at_top(text: str, insert: str) -> str:
    return insert + "\n\n" + text


def _build_insert_block(repo_names: list[str]) -> str:
    bullets: list[str] = []
    for name in repo_names:
        up = UPSTREAM.get(name)
        if up is None:
            # Unknown repo-ref entry: still provide a GitHub search link as a last resort.
            bullets.append(f"- `{name}`: https://github.com/search?q={name}&type=repositories")
            continue
        if up.url:
            bullets.append(f"- {up.label}: {up.url}")
        else:
            bullets.append(f"- {up.label}: (internal reference; no public upstream link)")

    return "\n".join(
        [
            SECTION_TITLE,
            "",
            "This document references optional local checkouts under `repo-ref/` for convenience.",
            "Upstream sources:",
            "",
            *bullets,
            "",
            "See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.",
        ]
    )


def main(argv: list[str]) -> int:
    _ = argv
    root = _repo_root()
    docs_dir = root / "docs"
    if not docs_dir.is_dir():
        print("error: missing docs/ directory", file=sys.stderr)
        return 2

    changed = 0
    scanned = 0
    skipped_with_github = 0

    for path in sorted(docs_dir.rglob("*.md")):
        scanned += 1
        text = path.read_text(encoding="utf-8", errors="replace")
        repo_names = _extract_repo_refs(text)
        if not repo_names:
            continue

        if RE_GITHUB.search(text):
            skipped_with_github += 1
            continue

        if _has_upstream_section(text):
            continue

        insert = _build_insert_block(repo_names)

        new_text = _insert_after_front_matter(text, insert)
        if new_text is None:
            new_text = _insert_after_h1(text, insert)
        if new_text is None:
            # Some docs start directly with content (e.g. "Status:" or "## ..."). Insert at top.
            new_text = _insert_at_top(text, insert)

        if new_text != text:
            path.write_text(new_text, encoding="utf-8")
            changed += 1

    print(f"scanned={scanned} changed={changed} skipped_with_github={skipped_with_github}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
