"""Check repository documentation paths without network access or new dependencies."""
import re
import unittest
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[2]


class DocumentationTests(unittest.TestCase):
    def test_relative_markdown_links_resolve(self):
        documents = set(ROOT.glob("*.md"))
        for pattern in ("docs/**/*.md", "sdk/*/README.md", "starters/*/README.md", ".github/**/*.md"):
            documents.update(ROOT.glob(pattern))
        failures = []
        for document in sorted(documents):
            text = re.sub(r"(?ms)^```[^\n]*\n.*?^```\s*$", "", document.read_text(encoding="utf-8"))
            for match in re.finditer(r"\[[^\]\n]*\]\(([^\s)]+)(?:\s+\"[^\"]*\")?\)", text):
                url = match.group(1)
                parts = urlsplit(url)
                if parts.scheme or parts.netloc or not parts.path:
                    continue
                target = (document.parent / unquote(parts.path)).resolve()
                if not target.is_relative_to(ROOT) or not target.exists():
                    failures.append(f"{document.relative_to(ROOT)}: {url}")
        self.assertFalse(failures, "Broken relative links:\n" + "\n".join(failures))


if __name__ == "__main__":
    unittest.main()
