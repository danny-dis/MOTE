"""Keep declared SDK minimum runtimes in CI without breaking release tooling."""
import json
from pathlib import Path
import re
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]


class SDKMinimumTests(unittest.TestCase):
    def test_ci_uses_declared_minimums_after_release_tooling(self):
        workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        node = json.loads((ROOT / "sdk/typescript/package.json").read_text())["engines"]["node"].removeprefix(">=")
        python = tomllib.loads((ROOT / "sdk/python/pyproject.toml").read_text())["project"]["requires-python"].removeprefix(">=")
        go = re.search(r"(?m)^go ([\d.]+)$", (ROOT / "sdk/go/go.mod").read_text()).group(1)
        for key, version in (("node-version", node), ("python-version", python), ("go-version", go)):
            with self.subTest(runtime=key):
                self.assertIn(f"{key}: '{version}'", workflow)
        # Release scripts import tomllib; their tests must run on Python 3.11+.
        tooling_setup = workflow.index("name: Install Python for release tooling")
        tooling = workflow.index("name: Release tooling tests")
        sdk_setup = workflow.index("name: Install minimum supported Python for SDK tests")
        sdk_tests = workflow.index("name: Python SDK and real bridge integration")
        self.assertIn("python-version: '3.11'", workflow[tooling_setup:tooling])
        self.assertLess(tooling, sdk_setup)
        self.assertLess(sdk_setup, sdk_tests)
        self.assertIn(f"python-version: '{python}'", workflow[sdk_setup:sdk_tests])


if __name__ == "__main__":
    unittest.main()
