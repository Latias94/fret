import unittest

import diag_editor_paint_contract_preflight as preflight


class EditorPaintContractPreflightTests(unittest.TestCase):
    def test_required_contract_inputs_exist(self) -> None:
        root = preflight._workspace_root()
        required = [*preflight.EDITOR_PROBE_SCRIPTS, preflight.BASELINE_MATRIX]

        missing = [path for path in required if not (root / path).is_file()]

        self.assertEqual([], missing)


if __name__ == "__main__":
    unittest.main()
