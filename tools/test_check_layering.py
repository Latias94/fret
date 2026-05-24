#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from types import ModuleType
from typing import Any


def load_layering_module() -> ModuleType:
    path = Path(__file__).with_name("check_layering.py")
    spec = importlib.util.spec_from_file_location("check_layering", path)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


LAYERING = load_layering_module()


def package(
    name: str,
    *,
    features: dict[str, list[str]] | None = None,
    dependency_features: list[str] | None = None,
) -> dict[str, Any]:
    dependencies: list[dict[str, Any]] = []
    if dependency_features is not None:
        dependencies.append(
            {
                "name": "fret-ui",
                "features": dependency_features,
            }
        )

    return {
        "id": f"path+file:///workspace/{name}#0.1.0",
        "name": name,
        "features": features or {},
        "dependencies": dependencies,
    }


def metadata(*packages: dict[str, Any]) -> dict[str, Any]:
    return {
        "workspace_members": [p["id"] for p in packages],
        "packages": list(packages),
    }


class RetainedBridgeLayeringTests(unittest.TestCase):
    def test_deleted_bridge_feature_mapping_is_rejected(self) -> None:
        violations: list[object] = []

        LAYERING._check_unstable_retained_bridge_feature_mapping_allowlist(
            metadata(
                package(
                    "fret-node",
                    features={
                        "compat-retained-canvas": ["fret-ui/unstable-retained-bridge"],
                    },
                )
            ),
            allowlist={},
            violations=violations,
        )

        self.assertEqual(1, len(violations))
        self.assertEqual(
            "unstable-retained-bridge-feature-mapping-allowlist",
            violations[0].rule,
        )
        self.assertIn("fret-node/compat-retained-canvas", violations[0].message)

    def test_compat_feature_without_deleted_bridge_mapping_passes(self) -> None:
        violations: list[object] = []

        LAYERING._check_unstable_retained_bridge_feature_mapping_allowlist(
            metadata(
                package(
                    "fret-node",
                    features={
                        "compat-retained-canvas": ["fret-ui"],
                    },
                )
            ),
            allowlist={},
            violations=violations,
        )

        self.assertEqual([], violations)

    def test_default_feature_mapping_to_deleted_bridge_is_rejected(self) -> None:
        violations: list[object] = []

        LAYERING._check_unstable_retained_bridge_feature_mapping_allowlist(
            metadata(
                package(
                    "fret-node",
                    features={
                        "default": ["fret-ui/unstable-retained-bridge"],
                    },
                )
            ),
            allowlist={},
            violations=violations,
        )

        self.assertEqual(1, len(violations))
        self.assertIn("fret-node/default", violations[0].message)

    def test_direct_dependency_feature_is_rejected_by_default(self) -> None:
        violations: list[object] = []

        LAYERING._check_unstable_retained_bridge_dependency_allowlist(
            metadata(
                package(
                    "fret-chart",
                    dependency_features=["unstable-retained-bridge"],
                )
            ),
            allowlist=set(),
            violations=violations,
        )

        self.assertEqual(1, len(violations))
        self.assertEqual(
            "unstable-retained-bridge-dependency-allowlist",
            violations[0].rule,
        )


if __name__ == "__main__":
    unittest.main()
