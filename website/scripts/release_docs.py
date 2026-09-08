from __future__ import annotations

import argparse
import shutil

from docs_common import (
    CONTENT_ROOT,
    DOCS_ROOT,
    STD_ROOT,
    build_all,
    build_version,
    copy_content,
    latest_version,
    load_versions,
    save_versions,
)

def release_documentation(new_version: str, source_version: str | None = None) -> None:
    versions = load_versions()

    original_versions = {
        "latest": versions.get("latest"),
        "versions": [dict(entry) for entry in versions.get("versions", [])],
        "defaultSection": versions.get("defaultSection"),
    }

    known = {entry["id"] for entry in versions.get("versions", [])}

    if new_version in known:
        raise SystemExit(f"version already exists: {new_version}")

    source = source_version or latest_version(versions)

    if source not in known:
        raise SystemExit(f"unknown source version: {source}")

    if not (STD_ROOT / new_version).exists():
        raise SystemExit(f"missing std sources for {new_version}: {STD_ROOT / new_version}")

    if not (CONTENT_ROOT / source / "compiler-command-line-reference.json").exists():
        raise SystemExit(
            f"missing compiler command line reference in source version: {source}"
        )

    copy_content(source, new_version)

    new_versions = []
    for entry in versions.get("versions", []):
        updated = dict(entry)
        updated["label"] = updated["id"]
        updated["status"] = "archived"
        new_versions.append(updated)

    new_versions.insert(
        0,
        {
            "id": new_version,
            "label": f"{new_version} (latest)",
            "status": "current",
        },
    )

    next_versions = dict(versions)
    next_versions["latest"] = new_version
    next_versions["versions"] = new_versions

    try:
        build_version(new_version, next_versions)
        save_versions(next_versions)
        build_all()
    except Exception:
        save_versions(original_versions)
        shutil.rmtree(CONTENT_ROOT / new_version, ignore_errors=True)
        shutil.rmtree(DOCS_ROOT / new_version, ignore_errors=True)
        raise

    print(f"released {new_version} from {source}")

def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Release a new documentation version.",
        epilog="Examples:\n  python scripts/release_docs.py --new-version v0.2.2\n  python scripts/release_docs.py --new-version v0.2.2 --from v0.2.1",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument("--new-version", required=True)
    parser.add_argument("--from", dest="from_version")

    args = parser.parse_args(argv)

    release_documentation(args.new_version, args.from_version)


if __name__ == "__main__":
    main()
