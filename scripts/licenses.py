"""Bundle verbatim dependency notices plus the installed Rust toolchain notices."""
import argparse
import json
from pathlib import Path
import subprocess


def collect(packages):
    sections = ["Dependency license/notice texts (resolved target, including build dependencies).\n"]
    for pkg in sorted(packages, key=lambda p: (p["name"], p["version"])):
        if pkg.get("source") is None:
            continue
        root = Path(pkg["manifest_path"]).parent
        files = {p for p in root.rglob("*") if p.is_file() and p.name.upper().startswith(("LICENSE", "LICENCE", "COPYRIGHT", "NOTICE"))}
        if pkg.get("license_file"):
            files.add(root / pkg["license_file"])
        if not files:
            raise ValueError(f"No license text found for {pkg['name']} {pkg['version']}")
        sections.append(f"\n## {pkg['name']} {pkg['version']} ({pkg.get('license')})\n")
        for path in sorted(files):
            sections.append(f"\n### {path.relative_to(root)}\n" + path.read_text(encoding="utf-8") + "\n")
    return "".join(sections)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--target", required=True)
    args = parser.parse_args()
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", args.target], text=True, encoding="utf-8"))
    active = {node["id"] for node in metadata["resolve"]["nodes"]}
    packages = [pkg for pkg in metadata["packages"] if pkg["id"] in active]
    text = collect(packages)
    version = subprocess.check_output(["rustc", "--version"], text=True).split()[1]
    sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip())
    # Shipped with rustc itself; binds notices to the actual installed toolchain.
    notices = (sysroot / "share/doc/rust/COPYRIGHT-library.html").read_bytes()
    if b"copyright" not in notices.lower() or len(notices) < 1000:
        raise ValueError("Rust copyright bundle was empty or invalid")
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "THIRD_PARTY_LICENSES.txt").write_text(text, encoding="utf-8")
    (args.output / "RUST_COPYRIGHT.html").write_bytes(notices)
    print(f"Collected license texts from {len(packages)} package metadata entries and Rust {version}")
