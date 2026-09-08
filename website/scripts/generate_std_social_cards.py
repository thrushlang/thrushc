from __future__ import annotations

import argparse
import json
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError as exc:
    raise SystemExit(
        "Missing dependency: Pillow. Install dependencies with: "
        "python -m pip install -r requirements.txt"
    ) from exc


ROOT: Path = Path(__file__).resolve().parent
REPO_ROOT: Path = ROOT.parent
DOCS_ROOT: Path = REPO_ROOT / "documentation"
CONTENT_ROOT: Path = DOCS_ROOT / "content"
VERSIONS_FILE: Path = DOCS_ROOT / "versions.json"
LOGO_FILE: Path = REPO_ROOT / "assets" / "brand" / "thrustlang-logo-128.png"
REGULAR_FONT_FILE: Path = REPO_ROOT / "assets" / "fonts" / "JetBrainsMonoNerdFont-Regular.ttf"
BOLD_FONT_FILE: Path = REPO_ROOT / "assets" / "fonts" / "JetBrainsMonoNerdFont-Bold.ttf"

WIDTH = 1200
HEIGHT = 630
PADDING = 72
TITLE_MIN_SIZE = 48
TITLE_MAX_SIZE = 128
TITLE_SUBTITLE_GAP = 64

BG = "#272727"
PANEL = "#2b2b2b"
LINE = "#5e5e5e"
TEXT = "#ffffff"
MUTED = "#c2c2c2"
ACCENT = "#8f8f8f"

def read_json(path: Path):
    return json.loads(path.read_text())


def slug_filename(slug: str) -> str:
    return slug.strip("/").replace("/", "-") + ".png"


def load_font(size: int, *, bold: bool = False):
    font_path = BOLD_FONT_FILE if bold else REGULAR_FONT_FILE

    if not font_path.exists():
        raise SystemExit(f"Missing bundled font: {font_path}")

    return ImageFont.truetype(str(font_path), size)


def title_line_candidates(title: str) -> list[list[str]]:
    parts = title.split("::")

    if len(parts) <= 2:
        return [[title]]

    candidates = [
        [title],
        ["::".join(parts[:-1]), f"::{parts[-1]}"],
        [f"{parts[0]}::", "::".join(parts[1:-1]), f"::{parts[-1]}"],
        [parts[0], *[f"::{part}" for part in parts[1:]]],
    ]

    unique = []

    for candidate in candidates:
        normalized = [line for line in candidate if line]

        if normalized not in unique:
            unique.append(normalized)

    return unique


def measure_multiline(draw: ImageDraw.ImageDraw, lines: list[str], font: ImageFont.ImageFont, line_gap: int) -> tuple[int, int]:
    widths = []
    heights = []

    for line in lines:
        bbox = draw.textbbox((0, 0), line, font=font)
        widths.append(bbox[2] - bbox[0])
        heights.append(bbox[3] - bbox[1])

    return max(widths), sum(heights) + (line_gap * (len(lines) - 1))


def title_fits(draw: ImageDraw.ImageDraw, lines: list[str], max_width: int, max_height: int, size: int) -> bool:
    font = load_font(size, bold=True)
    line_gap = max(-48, -(size // 8))
    width, height = measure_multiline(draw, lines, font, line_gap)

    return width <= max_width and height <= max_height


def best_title_size(draw: ImageDraw.ImageDraw, lines: list[str], max_width: int, max_height: int) -> int | None:
    low = TITLE_MIN_SIZE
    high = TITLE_MAX_SIZE
    best = None

    while low <= high:
        mid = (low + high) // 2

        if title_fits(draw, lines, max_width, max_height, mid):
            best = mid
            low = mid + 1
        else:
            high = mid - 1

    return best


def fit_title(draw: ImageDraw.ImageDraw, title: str, max_width: int, max_height: int):
    best_lines = [title]
    title_size_cap = best_title_size(draw, best_lines, max_width, max_height)

    if title_size_cap is None:
        title_size_cap = TITLE_MIN_SIZE

    best_size = None

    for lines in title_line_candidates(title):
        size = best_title_size(draw, lines, max_width, max_height)

        if size is None:
            continue

        size = min(size, title_size_cap)

        if best_size is None or size > best_size or (size == best_size and len(lines) < len(best_lines)):
            best_lines = lines
            best_size = size

    if best_size is None:
        best_size = title_size_cap

    font = load_font(best_size, bold=True)
    line_gap = max(-48, -(best_size // 8))

    return best_lines, font, line_gap


def fit_font(draw: ImageDraw.ImageDraw, text: str, max_width: int, start_size: int, min_size: int = 74):
    size = start_size

    while size > min_size:
        font = load_font(size, bold=True)
        bbox = draw.textbbox((0, 0), text, font=font)

        if bbox[2] - bbox[0] <= max_width:
            return font

        size -= 4

    return load_font(min_size, bold=True)


def draw_card(version: str, title: str, destination: Path) -> None:
    image = Image.new("RGB", (WIDTH, HEIGHT), BG)
    draw = ImageDraw.Draw(image)

    if LOGO_FILE.exists():
        logo = Image.open(LOGO_FILE).convert("RGBA")
        logo.thumbnail((104, 104), Image.Resampling.LANCZOS)
        image.paste(logo, (PADDING, PADDING), logo)
    else:
        draw.rectangle((PADDING, PADDING, PADDING + 104, PADDING + 104), outline=LINE, width=2)

    label_font = load_font(34, bold=True)
    subtitle_font = load_font(42)
    subtitle_y = HEIGHT - 78
    title_bottom = subtitle_y - TITLE_SUBTITLE_GAP
    title_top_limit = 250
    title_lines, title_font, title_line_gap = fit_title(
        draw,
        title,
        WIDTH - (PADDING * 2),
        title_bottom - title_top_limit,
    )

    draw.text((PADDING + 128, PADDING + 16), "Thrust Programming Language", font=label_font, fill=TEXT)
    draw.text((PADDING + 128, PADDING + 62), "Standard Library", font=load_font(28), fill=MUTED)

    _, title_block_height = measure_multiline(draw, title_lines, title_font, title_line_gap)
    title_y = title_bottom - title_block_height
    line_y = max(title_top_limit - 24, title_y - 28)

    draw.line((PADDING, line_y, WIDTH - PADDING, line_y), fill=ACCENT, width=2)

    current_y = title_y
    for line in title_lines:
        bbox = draw.textbbox((0, 0), line, font=title_font)
        draw.text((PADDING, current_y), line, font=title_font, fill=TEXT)
        current_y += (bbox[3] - bbox[1]) + title_line_gap

    draw.text((PADDING, subtitle_y), f"Standard Library · {version}", font=subtitle_font, fill=MUTED)

    draw.rectangle((WIDTH - 244, PADDING, WIDTH - PADDING, PADDING + 56), fill=PANEL, outline=LINE, width=1)
    draw.text((WIDTH - 210, PADDING + 10), version, font=load_font(30, bold=True), fill=TEXT)

    destination.parent.mkdir(parents=True, exist_ok=True)
    image.save(destination, "PNG")


def generate_std_social_cards(version: str, pages: dict | None = None, slug: str | None = None) -> list[Path]:
    if pages is None:
        pages = read_json(CONTENT_ROOT / version / "pages.json").get("std", {})

    if slug is not None:
        if slug not in pages:
            raise SystemExit(f"unknown std page for {version}: {slug}")

        pages = {slug: pages[slug]}

    generated: list[Path] = []

    for page_slug, data in pages.items():
        destination = DOCS_ROOT / version / "social" / "std" / slug_filename(page_slug)
        draw_card(version, data["title"], destination)
        generated.append(destination)

    return generated


def versions_to_generate(version: str | None) -> list[str]:
    if version is not None:
        return [version]

    versions = read_json(VERSIONS_FILE)
    return [entry["id"] for entry in versions.get("versions", [])]


def main(argv=None) -> None:
    parser = argparse.ArgumentParser(
        description="Generate PNG Open Graph cards for standard library documentation pages.",
    )
    parser.add_argument("--version", help="Documentation version to generate, for example v0.2.1.")
    parser.add_argument("--slug", help="Standard library page slug to generate, for example mem or collections/vector.")

    args = parser.parse_args(argv)

    if args.slug and not args.version:
        raise SystemExit("--slug requires --version")

    generated: list[Path] = []

    for version in versions_to_generate(args.version):
        generated.extend(generate_std_social_cards(version, slug=args.slug))

    for path in generated:
        print(path.relative_to(REPO_ROOT))


if __name__ == "__main__":
    main()
