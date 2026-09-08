from __future__ import annotations

import argparse
import json
from pathlib import Path

from docs_common import (
    build_all,
    load_cli_reference,
    load_pages,
    load_versions,
    save_cli_reference,
    save_pages,
)

def require_known_version(version: str) -> None:
    versions = load_versions()
    known = {entry["id"] for entry in versions.get("versions", [])}

    if version not in known:
        raise SystemExit(f"unknown version: {version}")

def resolve_cli_target(
    version: str,
    section: str,
    category: str | None,
    flag_name: str | None,
):
    data = load_cli_reference(version)
    target = data
    location = section

    if flag_name is not None and category is None:
        raise SystemExit("--flag requires --category for compiler-command-line-reference")

    if category is not None:
        target = next(
            (
                item
                for item in data.get("categories", [])
                if item.get("slug") == category
            ),
            None,
        )

        if target is None:
            raise SystemExit(f"unknown category: {category}")

        location = f"{section}/{category}"

    if flag_name is not None:
        target = next(
            (
                item
                for item in target.get("flags", [])
                if item.get("name") == flag_name or flag_name in item.get("aliases", [])
            ),
            None,
        )

        if target is None:
            raise SystemExit(f"unknown flag: {flag_name}")

        location = f"{section}/{category}/{flag_name}"

    return data, target, location


def resolve_page_target(version: str, section: str, slug: str | None):
    if slug is None:
        raise SystemExit("--slug is required for std and language-reference updates")

    data = load_pages(version)
    section_data = data.get(section)

    if section_data is None:
        raise SystemExit(f"unknown section: {section}")

    target = section_data.get(slug)

    if target is None:
        raise SystemExit(f"unknown page: {section}/{slug}")

    location = f"{section}/{slug}"

    return data, target, location

def parse_list_text(field: str, text: str) -> list[str]:
    if field in {"examples", "usage"}:
        return [text]

    return [
        block.strip()
        for block in text.replace("\r\n", "\n").split("\n\n")
        if block.strip()
    ]

def resolve_next_value(
    field: str,
    current_value,
    value: str | None,
    input_file: str | None,
    json_value: str | None,
):
    if json_value is not None:
        return json.loads(json_value)

    text = Path(input_file).read_text() if input_file is not None else (value or "")

    if isinstance(current_value, list):
        return parse_list_text(field, text)

    if isinstance(current_value, dict):
        raise SystemExit(f"field {field} requires --json-value")

    return text

def validate_next_value(field: str, current_value, next_value) -> None:
    if type(next_value) is not type(current_value):
        raise SystemExit(
            f"invalid value type for {field}: expected {type(current_value).__name__}"
        )

    if isinstance(current_value, list) and any(
        not isinstance(item, str) for item in next_value
    ):
        raise SystemExit(f"invalid value for {field}: all items must be strings")

    if isinstance(current_value, dict) and any(
        not isinstance(key, str) for key in next_value
    ):
        raise SystemExit(f"invalid value for {field}: object keys must be strings")

def update_documentation(
    version: str,
    section: str,
    slug: str | None,
    category: str | None,
    flag_name: str | None,
    field: str,
    value: str | None,
    input_file: str | None,
    json_value: str | None,
) -> None:
    require_known_version(version)

    if section == "compiler-command-line-reference":
        data, target, location = resolve_cli_target(
            version,
            section,
            category,
            flag_name,
        )
    else:
        data, target, location = resolve_page_target(version, section, slug)

    if field not in target:
        raise SystemExit(f"unknown field for target: {field}")

    current_value = target[field]
    next_value = resolve_next_value(
        field,
        current_value,
        value,
        input_file,
        json_value,
    )

    validate_next_value(field, current_value, next_value)

    target[field] = next_value

    if section == "compiler-command-line-reference":
        save_cli_reference(version, data)
    else:
        save_pages(version, data)

    build_all()
    print(f"updated {location} in {version}")

def main(argv= None):
    parser = argparse.ArgumentParser(
        description="Update versioned documentation content.",
        epilog="Examples:\n  python scripts/update_docs.py --version v0.2.1 --section std --slug io --field summary --value 'New summary'\n  python scripts/update_docs.py --version v0.2.1 --section language-reference --slug functions --field example --input-file example.thrust\n  python scripts/update_docs.py --version v0.2.1 --section compiler-command-line-reference --category compiler-flags --flag=-emit --field description --value 'Updated description'",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument("--version", required=True)
    parser.add_argument(
        "--section",
        required=True,
        choices=["std", "language-reference", "compiler-command-line-reference"],
    )
    parser.add_argument("--slug")
    parser.add_argument("--category")
    parser.add_argument("--flag")
    parser.add_argument("--field", required=True)

    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--value")
    group.add_argument("--input-file")
    group.add_argument("--json-value")

    args = parser.parse_args(argv)

    update_documentation(
        args.version,
        args.section,
        args.slug,
        args.category,
        args.flag,
        args.field,
        args.value,
        args.input_file,
        args.json_value,
    )


if __name__ == "__main__":
    main()
