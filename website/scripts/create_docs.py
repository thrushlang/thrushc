from __future__ import annotations

import argparse

from docs_common import (
    STD_ROOT,
    build_all,
    load_pages,
    load_versions,
    save_pages,
)

def std_template(title: str, summary: str, source: str) -> dict:
    module_name = title if title.startswith("std::") else f"std::{title}"
    return {
        "title": module_name,
        "summary": summary,
        "source": source,
        "overview": [
            f"{module_name} is a documented standard library module.",
            "Describe what the module provides, how it is meant to be used, and where it fits in the standard library.",
        ],
        "details": [
            "Describe the important behavior, constraints, and usage patterns for this module.",
        ],
        "examples": [
            f"import {module_name};\n\nfn main() s32 @public {{\n    return 0;\n}}",
        ],
        "notes": [
            "Add implementation notes, caveats, or migration details that readers should know.",
        ],
    }

def language_reference_template(title: str, summary: str, source: str) -> dict:
    return {
        "title": title,
        "summary": summary,
        "source": source,
        "overview": [
            f"{title} is a language reference topic.",
            "Describe what this syntax or feature is for and when it should be used.",
        ],
        "semantics": [
            "Describe the language rules, behavior, and important edge cases for this topic.",
        ],
        "guidance": [
            "Add practical guidance that helps readers use this feature correctly.",
        ],
        "signatures": "// Add representative syntax signatures for this topic.",
        "example": "fn main() s32 @public {\n    return 0;\n}",
    }


def create_documentation_page(
    version: str,
    section: str,
    slug: str,
    title: str,
    summary: str,
    source: str,
) -> None:
    versions = load_versions()
    known = {entry["id"] for entry in versions.get("versions", [])}

    if version not in known:
        raise SystemExit(f"unknown version: {version}")

    pages = load_pages(version)
    section_pages = pages.get(section)

    if section_pages is None:
        raise SystemExit(f"unknown section: {section}")

    if slug in section_pages:
        raise SystemExit(f"page already exists: {section}/{slug}")

    if section == "std":
        source_path = STD_ROOT / version / source

        if not source_path.exists():
            raise SystemExit(f"missing std source for {version}: {source_path}")

        page = std_template(title, summary, source)
    else:
        page = language_reference_template(title, summary, source)

    section_pages[slug] = page
    save_pages(version, pages)
    build_all()
    print(f"created {section}/{slug} in {version}")


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Create a new versioned documentation page scaffold.",
        epilog=(
            "Examples:\n"
            "  python scripts/create_docs.py --version v0.2.1 --section std --slug collections/hashmap --title 'std::collections::hashmap' --summary 'Hash map container.' --source collections/hashmap.thrust\n"
            "  python scripts/create_docs.py --version v0.2.1 --section language-reference --slug traits --title 'Traits' --summary 'Shared behavior contracts.' --source syntax/traits/README.md"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument("--version", required=True)
    parser.add_argument(
        "--section",
        required=True,
        choices=["std", "language-reference"],
    )
    parser.add_argument("--slug", required=True)
    parser.add_argument("--title", required=True)
    parser.add_argument("--summary", required=True)
    parser.add_argument("--source", required=True)

    args = parser.parse_args(argv)

    create_documentation_page(
        args.version,
        args.section,
        args.slug,
        args.title,
        args.summary,
        args.source,
    )


if __name__ == "__main__":
    main()
