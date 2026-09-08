from __future__ import annotations
import argparse
from docs_common import build_all, latest_version, load_versions, save_versions

def archive_documentation(version: str) -> None:
    versions = load_versions()
    current_latest = latest_version(versions)

    if version == current_latest:
        raise SystemExit("cannot archive the latest version")

    found = False

    for entry in versions.get("versions", []):
        if entry.get("id") == version:
            entry["status"] = "archived"
            entry["label"] = entry["id"]
            found = True
            break

    if not found:
        raise SystemExit(f"unknown version: {version}")

    save_versions(versions)
    build_all()
    print(f"archived {version}")


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Archive a documentation version.",
        epilog="Examples:\n  python scripts/archive_docs.py --version v0.2.0",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument("--version", required=True)

    args = parser.parse_args(argv)

    archive_documentation(args.version)


if __name__ == "__main__":
    main()
