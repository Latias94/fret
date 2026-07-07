#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path
from types import ModuleType


def load_matrix_module() -> ModuleType:
    path = (
        Path(__file__).parent
        / "parity-discovery"
        / "shadcn_component_harness_matrix.py"
    )
    spec = importlib.util.spec_from_file_location("shadcn_component_harness_matrix", path)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


MATRIX = load_matrix_module()


class ShadcnComponentHarnessMatrixTests(unittest.TestCase):
    def test_display_path_allows_repo_external_outputs(self) -> None:
        repo_path = MATRIX.ROOT / "docs/shadcn-declarative-progress.md"
        self.assertEqual(
            "docs/shadcn-declarative-progress.md",
            MATRIX._display_path(repo_path),
        )

        external_path = Path(tempfile.gettempdir()) / "fret-matrix-output.json"
        self.assertEqual(external_path.as_posix(), MATRIX._display_path(external_path))


if __name__ == "__main__":
    unittest.main()
