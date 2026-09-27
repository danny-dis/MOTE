"""Execute the release workflow's changelog extraction without publishing."""
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[2]


class ReleaseNotesTests(unittest.TestCase):
    def test_workflow_extracts_only_tagged_version_and_rejects_missing_notes(self):
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        step = re.search(
            r"(?ms)^      - name: Extract release notes from changelog\n        run: \|\n(.*?)(?=^      - |\Z)",
            workflow,
        )
        self.assertIsNotNone(step, "Release notes must come from the changelog")
        shell = textwrap.dedent(step.group(1))
        match = re.fullmatch(r"python - <<'PY'\n(.*?)\nPY\s*", shell, re.S)
        self.assertIsNotNone(match, "Unexpected release-note extraction command")
        code = match.group(1)
        cases = [
            ("# Changelog\n\n## Unreleased\nFUTURE\n\n## 1.2.3\n\n### Fixed\n- CORRECT\n\n## 1.2.2\nOLD\n", True),
            ("# Changelog\n\n## 1.2.2\nWRONG VERSION\n", False),
            ("# Changelog\n\n## 1.2.3\n\n## 1.2.2\nOLD\n", False),
        ]
        for changelog, succeeds in cases:
            with self.subTest(changelog=changelog), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / "CHANGELOG.md").write_text(changelog, encoding="utf-8")
                result = subprocess.run(
                    [sys.executable, "-c", code], cwd=root,
                    env={**os.environ, "RELEASE_TAG": "v1.2.3", "RUNNER_TEMP": directory},
                    capture_output=True, text=True, timeout=10,
                )
                self.assertEqual(result.returncode == 0, succeeds, result.stderr)
                notes = root / "release-notes.md"
                if succeeds:
                    text = notes.read_text(encoding="utf-8")
                    self.assertIn("CORRECT", text)
                    self.assertIn("MOTE 1.2.3", text)
                    self.assertNotIn("FUTURE", text)
                    self.assertNotIn("OLD", text)
                else:
                    self.assertFalse(notes.exists())
        self.assertIn('--notes-file "$RUNNER_TEMP/release-notes.md"', workflow)
        self.assertNotIn("docs/RELEASE_NOTES.md", workflow)


if __name__ == "__main__":
    unittest.main()
