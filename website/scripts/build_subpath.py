from __future__ import annotations

import argparse
import re
import shutil
from pathlib import Path


ROOT: Path = Path(__file__).resolve().parent
REPO_ROOT: Path = ROOT.parent

EXCLUDED_NAMES = {
    ".git",
    ".github",
    ".venv",
    "__pycache__",
}

TEXT_SUFFIXES = {
    ".css",
    ".html",
    ".js",
    ".json",
    ".md",
    ".txt",
    ".xml",
}

ROOT_PATH_RE = re.compile(
    r"(?<![A-Za-z0-9_:.-])/(assets|documentation|en|es)(?=/|[\"'`),\s<>]|$)"
)
ESCAPED_ROOT_PATH_RE = re.compile(
    r"(?<![A-Za-z0-9_:.-])\\/(assets|documentation|en|es)(?=\\/|[\"'`),\s<>]|$)"
)


def normalize_base_path(base_path: str) -> str:
    base = base_path.strip()

    if not base:
        return ""

    if not base.startswith("/"):
        base = "/" + base

    return base.rstrip("/")


def should_ignore(_directory: str, names: list[str]) -> set[str]:
    ignored = set()

    for name in names:
        if name in EXCLUDED_NAMES or name.endswith(".pyc"):
            ignored.add(name)

    return ignored


def copy_site(source: Path, output: Path) -> None:
    if output.exists():
        shutil.rmtree(output)

    shutil.copytree(source, output, ignore=should_ignore)


def rewrite_text(text: str, base_path: str) -> str:
    if not base_path:
        return text

    escaped_base_path = "\\/" + base_path.strip("/").replace("/", "\\/")
    text = ROOT_PATH_RE.sub(rf"{base_path}/\1", text)

    return ESCAPED_ROOT_PATH_RE.sub(rf"{escaped_base_path}\/\1", text)


def rewrite_paths(output: Path, base_path: str) -> None:
    for path in output.rglob("*"):
        if not path.is_file() or path.suffix.lower() not in TEXT_SUFFIXES:
            continue

        text = path.read_text()
        rewritten = rewrite_text(text, base_path)

        if rewritten != text:
            path.write_text(rewritten)


def build_subpath(source: Path, output: Path, base_path: str) -> None:
    source = source.resolve()
    output = output.resolve()

    if source == output:
        raise SystemExit("output must be different from source")

    if source in output.parents:
        raise SystemExit("output must not be inside the source tree")

    normalized_base = normalize_base_path(base_path)
    copy_site(source, output)
    rewrite_paths(output, normalized_base)


def main(argv=None) -> None:
    parser = argparse.ArgumentParser(
        description="Build a copy of the website that can be served from a subpath.",
    )
    parser.add_argument(
        "--base-path",
        required=True,
        help="Base path where the copied site will be served, for example /website.",
    )
    parser.add_argument(
        "--output",
        required=True,
        type=Path,
        help="Destination directory for the rewritten site copy.",
    )
    parser.add_argument(
        "--source",
        type=Path,
        default=REPO_ROOT,
        help="Source website directory. Defaults to the repository root.",
    )

    args = parser.parse_args(argv)
    build_subpath(args.source, args.output, args.base_path)
    print(args.output.resolve())


if __name__ == "__main__":
    main()
