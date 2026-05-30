#!/usr/bin/env python3
"""Generate a static Material 3 token inventory report."""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter, defaultdict
from datetime import date
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
TOKEN_RE = re.compile(r'"(md\.[^"]+)"')
INJECT_FN_RE = re.compile(r"^\s*(?:pub(?:\(crate\))?\s+)?fn\s+(inject_[A-Za-z0-9_]+)\s*\(")
MATERIAL_WEB_CALL_RE = re.compile(r"material_web_v30::(inject_[A-Za-z0-9_]+)\(")
INSERT_KEY_RE = re.compile(r'\.(?:insert|entry)\(\s*"([^"]+)".*?\.to_string\(\)', re.DOTALL)
COPY_KEY_RE = re.compile(r"\bcopy_(?:color|number)\(\s*cfg,\s*\"([^\"]+)\"", re.DOTALL)
PX_VALUE_RE = re.compile(r"Px\((-?\d+(?:\.\d+)?)\)")
UNWRAP_NUMBER_RE = re.compile(r"\.unwrap_or\((-?\d+(?:\.\d+)?)\)")
CONST_NUMBER_RE = re.compile(
    r"\b(?:pub(?:\(crate\))?\s+)?const\s+([A-Z0-9_]+)\s*:\s*[^=]+=\s*(?:Px\()?(-?\d+(?:\.\d+)?)"
)
COLOR_HEX_RE = re.compile(r"Color::from_srgb_hex_rgb\((0x[0-9a-fA-F_]+)\)")

SHARED_TOKEN_HELPER_MODULES = {"shape.rs", "typography.rs"}
TOKEN_MODULE_SKIP = {
    "material_web_v30.rs",
    "mod.rs",
    "v30.rs",
    "visual_fixtures.rs",
}.union(SHARED_TOKEN_HELPER_MODULES)
FALLBACK_MARKERS = {
    "component_to_system_color": "color_comp_or_sys(",
    "component_to_system_number": "number_comp_or_sys(",
    "system_color_resolver": "color_sys(",
    "system_number_resolver": "number_sys(",
    "theme_or_else_chain": ".or_else(",
    "literal_unwrap_or": ".unwrap_or(",
    "closure_unwrap_or_else": ".unwrap_or_else(",
    "default_unwrap": ".unwrap_or_default(",
}


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def production_lines(path: Path) -> list[tuple[int, str]]:
    lines: list[tuple[int, str]] = []
    for line_no, line in enumerate(read_text(path).splitlines(), 1):
        if re.match(r"\s*#\[cfg\(test\)\]", line):
            break
        lines.append((line_no, line))
    return lines


def extract_token_keys(path: Path) -> set[str]:
    return {match.group(1) for _, line in production_lines(path) for match in TOKEN_RE.finditer(line)}


def token_domain(key: str) -> str:
    if key.startswith("md.comp."):
        return "component"
    if key.startswith("md.sys.fret."):
        return "fret_sys_extension"
    if key.startswith("md.sys."):
        return "system"
    if key.startswith("md.ref."):
        return "reference"
    return "other"


def component_family(key: str) -> str:
    if not key.startswith("md.comp."):
        return token_domain(key)
    rest = key.removeprefix("md.comp.")
    return rest.split(".", 1)[0]


def line_sample(path: Path, line_no: int, line: str) -> dict[str, Any]:
    return {"path": rel(path), "line": line_no, "text": line.strip()}


def bounded_samples(samples: list[dict[str, Any]], limit: int = 8) -> list[dict[str, Any]]:
    return samples[:limit]


def scan_fallback_sites(path: Path) -> tuple[Counter[str], list[dict[str, Any]]]:
    counts: Counter[str] = Counter()
    samples: list[dict[str, Any]] = []
    for line_no, line in production_lines(path):
        for kind, marker in FALLBACK_MARKERS.items():
            if marker in line:
                counts[kind] += 1
                samples.append({"kind": kind, **line_sample(path, line_no, line)})
    return counts, samples


def constant_role(line: str) -> str:
    lowered = line.lower()
    if "opacity" in lowered or "alpha" in lowered:
        return "opacity"
    if "shape" in lowered or "corner" in lowered or "corners" in lowered or "radius" in lowered:
        return "shape"
    if "elevation" in lowered or "shadow" in lowered:
        return "elevation"
    if "duration" in lowered or "motion" in lowered or "spring" in lowered:
        return "motion"
    if "color" in lowered:
        return "color"
    return "metric"


def scan_magic_constants(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line_no, line in production_lines(path):
        stripped = line.strip()
        if not stripped or stripped.startswith("//") or "assert" in stripped:
            continue
        contexts = []
        if ".unwrap_or(" in line or "fallback" in line.lower():
            contexts.append("fallback")
        if "const " in line:
            contexts.append("const")
        if "Color::from_srgb_hex_rgb" in line:
            contexts.append("color_literal")
        if not contexts and "Px(" not in line:
            continue
        for match in PX_VALUE_RE.finditer(line):
            rows.append(
                {
                    "kind": "px_literal",
                    "role": constant_role(line),
                    "value": match.group(1),
                    "contexts": contexts or ["px_literal"],
                    **line_sample(path, line_no, line),
                }
            )
        for match in UNWRAP_NUMBER_RE.finditer(line):
            rows.append(
                {
                    "kind": "number_fallback",
                    "role": constant_role(line),
                    "value": match.group(1),
                    "contexts": contexts or ["literal_unwrap_or"],
                    **line_sample(path, line_no, line),
                }
            )
        for match in CONST_NUMBER_RE.finditer(line):
            rows.append(
                {
                    "kind": "named_const",
                    "role": constant_role(line),
                    "name": match.group(1),
                    "value": match.group(2),
                    "contexts": contexts or ["const"],
                    **line_sample(path, line_no, line),
                }
            )
        for match in COLOR_HEX_RE.finditer(line):
            rows.append(
                {
                    "kind": "color_literal",
                    "role": "color",
                    "value": match.group(1),
                    "contexts": contexts or ["color_literal"],
                    **line_sample(path, line_no, line),
                }
            )
    return rows


def count_by(items: list[str]) -> dict[str, int]:
    return dict(sorted(Counter(items).items()))


def scan_injection_surface(path: Path, material_web_keys: set[str]) -> dict[str, Any]:
    text = read_text(path)
    production_text = "\n".join(line for _, line in production_lines(path))
    injection_functions = [
        match.group(1)
        for _, line in production_lines(path)
        for match in [INJECT_FN_RE.match(line)]
        if match is not None
    ]
    material_web_delegates = sorted(set(MATERIAL_WEB_CALL_RE.findall(production_text)))
    manual_write_keys = sorted(
        set(INSERT_KEY_RE.findall(production_text)) | set(COPY_KEY_RE.findall(production_text))
    )
    non_generated_manual_write_keys = sorted(key for key in manual_write_keys if key not in material_web_keys)
    referenced_keys = sorted(extract_token_keys(path))
    non_generated_referenced_keys = sorted(key for key in referenced_keys if key not in material_web_keys)

    return {
        "path": rel(path),
        "injection_function_count": len(injection_functions),
        "injection_functions": injection_functions,
        "material_web_delegate_count": len(material_web_delegates),
        "material_web_delegates": material_web_delegates,
        "referenced_token_key_count": len(referenced_keys),
        "referenced_token_domain_counts": count_by([token_domain(key) for key in referenced_keys]),
        "manual_write_key_count": len(manual_write_keys),
        "non_generated_manual_write_key_count": len(non_generated_manual_write_keys),
        "non_generated_manual_write_domain_counts": count_by(
            [token_domain(key) for key in non_generated_manual_write_keys]
        ),
        "non_generated_manual_write_component_family_counts": count_by(
            [component_family(key) for key in non_generated_manual_write_keys]
        ),
        "non_generated_referenced_key_count": len(non_generated_referenced_keys),
        "non_generated_referenced_domain_counts": count_by(
            [token_domain(key) for key in non_generated_referenced_keys]
        ),
        "non_generated_manual_write_keys": non_generated_manual_write_keys,
    }


def load_matrix_components(matrix_path: Path) -> tuple[list[dict[str, Any]], dict[str, list[str]]]:
    matrix = json.loads(read_text(matrix_path))
    by_module: dict[str, list[str]] = defaultdict(list)
    for component in matrix.get("components", []):
        for token_module in component.get("token_modules", []):
            module = Path(token_module).stem
            by_module[module].append(component["id"])
    return matrix.get("components", []), dict(sorted(by_module.items()))


def scan_component_modules(
    tokens_dir: Path, matrix_module_map: dict[str, list[str]]
) -> tuple[list[dict[str, Any]], Counter[str], list[dict[str, Any]], list[dict[str, Any]]]:
    modules: list[dict[str, Any]] = []
    aggregate_fallback_counts: Counter[str] = Counter()
    all_fallback_samples: list[dict[str, Any]] = []
    all_magic_constants: list[dict[str, Any]] = []

    for path in sorted(tokens_dir.glob("*.rs")):
        if path.name in TOKEN_MODULE_SKIP:
            continue
        keys = sorted(extract_token_keys(path))
        fallback_counts, fallback_samples = scan_fallback_sites(path)
        magic_constants = scan_magic_constants(path)
        aggregate_fallback_counts.update(fallback_counts)
        all_fallback_samples.extend(fallback_samples)
        all_magic_constants.extend(magic_constants)
        modules.append(
            {
                "module": path.stem,
                "path": rel(path),
                "matrix_components": matrix_module_map.get(path.stem, []),
                "token_key_count": len(keys),
                "token_domain_counts": count_by([token_domain(key) for key in keys]),
                "component_family_counts": count_by([component_family(key) for key in keys]),
                "fallback_site_count": sum(fallback_counts.values()),
                "fallback_pattern_counts": dict(sorted(fallback_counts.items())),
                "fallback_samples": bounded_samples(fallback_samples, 6),
                "magic_visual_constant_count": len(magic_constants),
                "magic_visual_constant_role_counts": count_by([row["role"] for row in magic_constants]),
                "magic_visual_constant_samples": bounded_samples(magic_constants, 6),
            }
        )
    return modules, aggregate_fallback_counts, all_fallback_samples, all_magic_constants


def scan_shared_token_helper_modules(tokens_dir: Path) -> list[dict[str, Any]]:
    modules: list[dict[str, Any]] = []
    for name in sorted(SHARED_TOKEN_HELPER_MODULES):
        path = tokens_dir / name
        if not path.exists():
            continue
        keys = sorted(extract_token_keys(path))
        fallback_counts, fallback_samples = scan_fallback_sites(path)
        magic_constants = scan_magic_constants(path)
        modules.append(
            {
                "module": path.stem,
                "path": rel(path),
                "token_key_count": len(keys),
                "token_domain_counts": count_by([token_domain(key) for key in keys]),
                "component_family_counts": count_by([component_family(key) for key in keys]),
                "fallback_site_count": sum(fallback_counts.values()),
                "fallback_pattern_counts": dict(sorted(fallback_counts.items())),
                "fallback_samples": bounded_samples(fallback_samples, 6),
                "magic_visual_constant_count": len(magic_constants),
                "magic_visual_constant_role_counts": count_by([row["role"] for row in magic_constants]),
                "magic_visual_constant_samples": bounded_samples(magic_constants, 6),
            }
        )
    return modules


def scan_foundation(paths: list[Path]) -> dict[str, Any]:
    modules: list[dict[str, Any]] = []
    aggregate_fallback_counts: Counter[str] = Counter()
    all_magic_constants: list[dict[str, Any]] = []
    for directory in paths:
        for path in sorted(directory.glob("*.rs")):
            fallback_counts, fallback_samples = scan_fallback_sites(path)
            magic_constants = scan_magic_constants(path)
            aggregate_fallback_counts.update(fallback_counts)
            all_magic_constants.extend(magic_constants)
            modules.append(
                {
                    "module": path.stem,
                    "path": rel(path),
                    "token_key_count": len(extract_token_keys(path)),
                    "fallback_site_count": sum(fallback_counts.values()),
                    "fallback_pattern_counts": dict(sorted(fallback_counts.items())),
                    "fallback_samples": bounded_samples(fallback_samples, 6),
                    "magic_visual_constant_count": len(magic_constants),
                    "magic_visual_constant_role_counts": count_by([row["role"] for row in magic_constants]),
                    "magic_visual_constant_samples": bounded_samples(magic_constants, 6),
                }
            )
    return {
        "module_count": len(modules),
        "fallback_site_count": sum(aggregate_fallback_counts.values()),
        "fallback_pattern_counts": dict(sorted(aggregate_fallback_counts.items())),
        "magic_visual_constant_count": len(all_magic_constants),
        "magic_visual_constant_role_counts": count_by([row["role"] for row in all_magic_constants]),
        "modules": modules,
    }


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    tokens_dir = ROOT / "ecosystem/fret-ui-material3/src/tokens"
    foundation_dir = ROOT / "ecosystem/fret-ui-material3/src/foundation"
    interaction_dir = ROOT / "ecosystem/fret-ui-material3/src/interaction"
    material_web_path = tokens_dir / "material_web_v30.rs"
    v30_path = tokens_dir / "v30.rs"
    matrix_path = ROOT / args.matrix

    matrix_components, matrix_module_map = load_matrix_components(matrix_path)
    material_web_keys = sorted(extract_token_keys(material_web_path))
    injection_surface = scan_injection_surface(v30_path, set(material_web_keys))
    component_modules, fallback_counts, fallback_samples, magic_constants = scan_component_modules(
        tokens_dir, matrix_module_map
    )
    shared_helper_modules = scan_shared_token_helper_modules(tokens_dir)
    foundation = scan_foundation([foundation_dir, interaction_dir])
    actual_modules = {module["module"] for module in component_modules}
    matrix_modules = set(matrix_module_map)

    report = {
        "schema_version": 1,
        "generated_date": args.generated_date,
        "generated_by": "tools/parity-discovery/material3_token_inventory.py",
        "source_precedence": {
            "token_inventory": "Material Web v30 generated snapshot, then manual Fret v30 aliases",
            "state_naming": "Compose Material3 interaction/state vocabulary, then Fret pressable state",
            "fallback_ownership": "Material foundation resolver first, component token modules second, recipe code only for composition",
        },
        "inputs": {
            "matrix": args.matrix,
            "generated_snapshot": rel(material_web_path),
            "injection_surface": rel(v30_path),
            "component_token_dir": rel(tokens_dir),
            "foundation_dir": rel(foundation_dir),
            "interaction_dir": rel(interaction_dir),
        },
        "summary": {
            "matrix_component_count": len(matrix_components),
            "component_token_module_count": len(component_modules),
            "matrix_token_module_count": len(matrix_modules),
            "shared_token_helper_module_count": len(shared_helper_modules),
            "shared_token_helper_modules": [module["module"] for module in shared_helper_modules],
            "unmapped_component_token_modules": sorted(actual_modules - matrix_modules),
            "matrix_modules_without_file": sorted(matrix_modules - actual_modules),
            "material_web_generated_key_count": len(material_web_keys),
            "v30_injection_function_count": injection_surface["injection_function_count"],
            "v30_non_generated_manual_write_key_count": injection_surface[
                "non_generated_manual_write_key_count"
            ],
            "component_token_fallback_site_count": sum(fallback_counts.values()),
            "component_token_fallback_pattern_counts": dict(sorted(fallback_counts.items())),
            "component_token_magic_visual_constant_count": len(magic_constants),
            "component_token_magic_visual_constant_role_counts": count_by(
                [row["role"] for row in magic_constants]
            ),
            "foundation_fallback_site_count": foundation["fallback_site_count"],
            "foundation_magic_visual_constant_count": foundation["magic_visual_constant_count"],
        },
        "material_web_generated_snapshot": {
            "path": rel(material_web_path),
            "token_key_count": len(material_web_keys),
            "token_domain_counts": count_by([token_domain(key) for key in material_web_keys]),
            "component_family_counts": count_by([component_family(key) for key in material_web_keys]),
        },
        "v30_injection_surface": injection_surface,
        "shared_token_helper_modules": shared_helper_modules,
        "component_token_modules": component_modules,
        "foundation_and_interaction": foundation,
        "fallback_sample_index": bounded_samples(all_sorted_samples(fallback_samples), 40),
        "magic_visual_constant_sample_index": bounded_samples(all_sorted_samples(magic_constants), 40),
        "findings": derive_findings(
            component_modules, foundation, injection_surface, shared_helper_modules
        ),
    }
    return report


def all_sorted_samples(samples: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return sorted(samples, key=lambda row: (row.get("path", ""), row.get("line", 0), row.get("kind", "")))


def derive_findings(
    component_modules: list[dict[str, Any]],
    foundation: dict[str, Any],
    injection_surface: dict[str, Any],
    shared_helper_modules: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    heavy_fallback_modules = [
        module
        for module in component_modules
        if module["fallback_site_count"] >= 20 or module["magic_visual_constant_count"] >= 20
    ]
    unmapped_modules = [module for module in component_modules if not module["matrix_components"]]
    matrix_mapping_finding = {
        "finding": "Shared token helper modules are tracked separately from component token rows; unmapped component modules still require matrix/schema updates.",
        "evidence": {
            "shared_helper_modules": [module["module"] for module in shared_helper_modules],
            "unmapped_component_token_modules": [module["module"] for module in unmapped_modules],
        },
    }

    return [
        {
            "id": "M3TVM-F01",
            "level": "high",
            "layer": "component_token_modules",
            "finding": "Several component token modules own large fallback chains or literal defaults; family packets should converge repeated state-layer, disabled-opacity, shape, and metric fallbacks into typed token outcomes before recipe refactors.",
            "evidence": [
                {
                    "module": module["module"],
                    "fallback_site_count": module["fallback_site_count"],
                    "magic_visual_constant_count": module["magic_visual_constant_count"],
                }
                for module in heavy_fallback_modules[:12]
            ],
            "next_task": "M3TVM-030 then M3TVM-040/M3TVM-050/M3TVM-060/M3TVM-070",
        },
        {
            "id": "M3TVM-F02",
            "level": "medium",
            "layer": "v30_injection_surface",
            "finding": "The v30 injection surface writes non-generated Fret aliases and extensions on top of the Material Web snapshot; those keys need to stay documented as Fret-owned compatibility aliases.",
            "evidence": {
                "non_generated_manual_write_key_count": injection_surface[
                    "non_generated_manual_write_key_count"
                ],
                "domain_counts": injection_surface["non_generated_manual_write_domain_counts"],
                "family_counts": injection_surface["non_generated_manual_write_component_family_counts"],
            },
            "next_task": "M3TVM-080",
        },
        {
            "id": "M3TVM-F03",
            "level": "medium",
            "layer": "foundation",
            "finding": "Foundation and interaction modules already centralize some shared constants, but token resolver fallback colors remain a hard-coded emergency path that should not become component truth.",
            "evidence": {
                "foundation_fallback_site_count": foundation["fallback_site_count"],
                "foundation_magic_visual_constant_count": foundation["magic_visual_constant_count"],
            },
            "next_task": "M3TVM-030",
        },
        {
            "id": "M3TVM-F04",
            "level": "low",
            "layer": "matrix_mapping",
            **matrix_mapping_finding,
            "next_task": "M3TVM-080",
        },
    ]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--matrix",
        default="docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json",
    )
    parser.add_argument("--output", required=True)
    parser.add_argument("--generated-date", default=date.today().isoformat())
    args = parser.parse_args()

    report = build_report(args)
    output = ROOT / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n"
    )


if __name__ == "__main__":
    main()
