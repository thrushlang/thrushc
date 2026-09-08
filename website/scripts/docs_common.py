from __future__ import annotations

import html
import json
import re
import shutil
from pathlib import Path

from generate_std_social_cards import generate_std_social_cards, slug_filename


ROOT: Path = Path(__file__).resolve().parent
REPO_ROOT: Path = ROOT.parent
DOCS_ROOT: Path = REPO_ROOT / "documentation"
CONTENT_ROOT: Path = DOCS_ROOT / "content"
VERSION_STATE: Path = DOCS_ROOT / "versions.json"
STD_ROOT: Path = REPO_ROOT.parent / "thrustc" / "std"


def read_json(path: Path):
    return json.loads(path.read_text())


def write_json(path: Path, payload) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n")


def load_versions():
    return read_json(VERSION_STATE)


def save_versions(data) -> None:
    write_json(VERSION_STATE, data)


def load_pages(version: str):
    path = CONTENT_ROOT / version / "pages.json"

    if not path.exists():
        raise FileNotFoundError(f"missing content for {version}: {path}")

    return read_json(path)


def save_pages(version: str, data) -> None:
    write_json(CONTENT_ROOT / version / "pages.json", data)


def load_cli_reference(version: str):
    path = CONTENT_ROOT / version / "compiler-command-line-reference.json"

    if not path.exists():
        raise FileNotFoundError(f"missing compiler command line reference for {version}: {path}")

    return read_json(path)


def save_cli_reference(version: str, data) -> None:
    write_json(CONTENT_ROOT / version / "compiler-command-line-reference.json", data)


def copy_content(source_version: str, target_version: str) -> None:
    source = CONTENT_ROOT / source_version
    target = CONTENT_ROOT / target_version

    if not source.exists():
        raise FileNotFoundError(f"missing source content: {source}")

    if target.exists():
        raise FileExistsError(f"target content already exists: {target}")

    shutil.copytree(source, target)


def get_version_entry(versions_data, version: str):
    for entry in versions_data.get("versions", []):
        if entry.get("id") == version:
            return entry

    raise KeyError(f"unknown version: {version}")


def latest_version(versions_data) -> str:
    latest = versions_data.get("latest")

    if latest:
        return latest

    versions = versions_data.get("versions", [])

    if not versions:
        raise ValueError("versions.json has no versions")

    return versions[0]["id"]


def normalize_tail(path: str) -> str:
    if not path:
        return ""

    tail = path

    if tail.startswith("/"):
        tail = tail[1:]

    if tail in {"index.html", "search-index.json"}:
        return ""

    tail = re.sub(r"(^|/)index\.html$", r"\1", tail)

    if tail.endswith("search-index.json"):
        return ""

    if tail and not tail.endswith("/"):
        tail += "/"

    return tail


def current_version_and_tail(pathname: str):
    match = re.match(r"^/documentation/(v[^/]+)/?(.*)$", pathname)

    if not match:
        return {"version": None, "tail": ""}

    return {"version": match.group(1), "tail": normalize_tail(match.group(2))}


def doc_href(path: str) -> str:
    return path.rstrip("/") + "/index.html"


def extract_signatures(path: Path):
    lines = path.read_text().splitlines()
    out = []

    for line in lines:
        text = line.strip()

        if not text or text.startswith("/*") or text.startswith("//"):
            continue

        if re.match(r"^(fn|const|type|struct|static)\b", text) and "@public" in text:
            if text.startswith("fn ") and text.endswith("{"):
                text = text[:-1].rstrip() + ";"

            out.append(text)

    return out


def std_signatures_for_source(source_file: str, version: str):
    if source_file == "ffi/c/*.thrust":
        signatures = []

        for rel in [
            "ffi/c/primitives.thrust",
            "ffi/c/io.thrust",
            "ffi/c/mem.thrust",
            "ffi/c/math.thrust",
        ]:
            signatures.extend(extract_signatures(STD_ROOT / version / rel)[:45])

        return signatures

    return extract_signatures(STD_ROOT / version / source_file)


def render_head(
    title: str,
    desc: str,
    canonical: str,
    *,
    og_title: str | None = None,
    og_description: str | None = None,
    og_image: str | None = None,
    og_url: str | None = None,
):
    safe_title = html.escape(title)
    safe_desc = html.escape(desc)
    safe_canonical = html.escape(canonical)
    social = ""

    if og_image is not None:
        safe_og_title = html.escape(og_title or title)
        safe_og_description = html.escape(og_description or desc)
        safe_og_image = html.escape(og_image)
        safe_og_url = html.escape(og_url or canonical)
        social = f"""
  <meta property=\"og:type\" content=\"website\">
  <meta property=\"og:title\" content=\"{safe_og_title}\">
  <meta property=\"og:description\" content=\"{safe_og_description}\">
  <meta property=\"og:image\" content=\"{safe_og_image}\">
  <meta property=\"og:url\" content=\"{safe_og_url}\">
  <meta name=\"twitter:card\" content=\"summary_large_image\">
  <meta name=\"twitter:title\" content=\"{safe_og_title}\">
  <meta name=\"twitter:description\" content=\"{safe_og_description}\">
  <meta name=\"twitter:image\" content=\"{safe_og_image}\">"""

    return f"""<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\">
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
  <title>{safe_title} | Thrust</title>
  <meta name=\"description\" content=\"{safe_desc}\">
  <link rel=\"icon\" type=\"image/png\" sizes=\"32x32\" href=\"/assets/brand/favicon-32.png\">
  <link rel=\"icon\" type=\"image/png\" sizes=\"16x16\" href=\"/assets/brand/favicon-16.png\">
  <link rel=\"canonical\" href=\"{safe_canonical}\">
{social}
  <link rel=\"stylesheet\" href=\"/assets/css/site.css\">
  <link rel=\"stylesheet\" href=\"/assets/css/gruvbox-code.css\">
</head>
<body class=\"docs-page\">
  <header class=\"site-header\">
    <div class=\"container header-row\">
      <a class=\"brand\" href=\"/en/\">
        <img class=\"brand-logo\" src=\"/assets/brand/thrustlang-logo-128.png\" srcset=\"/assets/brand/thrustlang-logo-128.png 128w, /assets/brand/thrustlang-logo-256.png 256w, /assets/brand/thrustlang-logo-512.png 512w\" sizes=\"44px\" alt=\"Thrust logo\">
        <span class=\"brand-title\">Thrust</span>
      </a>
      <nav class=\"nav-links\" aria-label=\"Main navigation\">
        <a href=\"/en/\">Home</a>
        <a href=\"/en/features/\">Features</a>
        <a href=\"/en/downloads/\">Downloads</a>
        <a class=\"current\" href=\"/documentation/index.html\">Documentation</a>
        <a class=\"lang-link\" href=\"/es/\">ES</a>
      </nav>
    </div>
  </header>
"""


FOOTER = """  <footer class=\"site-footer\">\n    <div class=\"container footer-row\">\n      <span class=\"muted\">Thrust Programming Language</span>\n      <div class=\"social\">\n        <a class=\"btn\" href=\"https://github.com/thrustlang\" target=\"_blank\" rel=\"noopener noreferrer\">\n          <svg class=\"icon icon-sm\" aria-hidden=\"true\"><use href=\"/assets/icons/brands.svg#icon-github\"></use></svg>\n          GitHub\n        </a>\n        <a class=\"btn\" href=\"https://discord.gg/MhVpCSxnhV\" target=\"_blank\" rel=\"noopener noreferrer\">\n          <svg class=\"icon icon-sm\" aria-hidden=\"true\"><use href=\"/assets/icons/brands.svg#icon-discord\"></use></svg>\n          Discord\n        </a>\n      </div>\n    </div>\n  </footer>\n  <script src=\"/assets/js/thrust-highlight.js\"></script>\n  <script src=\"/documentation/assets/docs-version-switcher.js\"></script>\n  <script src=\"/documentation/assets/docs-search.js\"></script>\n</body>\n</html>\n"""


def docs_hero(title: str, summary: str, version: str, badge: str):
    return f"""    <section class=\"container hero\">\n      <h1>{html.escape(title)} ({html.escape(version)})</h1>\n      <p>{html.escape(summary)}</p>\n      <div class=\"docs-version-row\">\n        <label for=\"docs-version-{html.escape(version)}\">Version</label>\n        <select id=\"docs-version-{html.escape(version)}\" class=\"docs-version-select\" data-docs-version-select></select>\n        <span class=\"doc-badge\">{html.escape(badge)}</span>\n      </div>\n    </section>\n"""


def render_paragraphs(paragraphs):
    return "\n".join(f"      <p>{html.escape(polish_prose(p))}</p>" for p in paragraphs)


def render_items(items):
    return "\n".join(f"        <li>{html.escape(polish_prose(item))}</li>" for item in items)


def render_code_blocks(snippets):
    if isinstance(snippets, str):
        snippets = [snippets]

    blocks = []

    for snippet in snippets:
        blocks.append(
            f"      <pre class=\"code-thrust\"><code class=\"language-thrust\">{html.escape(snippet)}</code></pre>"
        )

    return "\n".join(blocks)


def render_cli_meta(label: str, value):
    if value is None or value == "" or value == []:
        return ""

    if isinstance(value, bool):
        text = "yes" if value else "no"
    elif isinstance(value, list):
        text = ", ".join(str(item) for item in value)
    else:
        text = str(value)

    return f"      <p><strong>{html.escape(label)}:</strong> <code>{html.escape(text)}</code></p>"


def render_attribute_sections(data: dict):
    if "attribute_catalog" not in data:
        return ""

    catalog = "\n".join(
        f"        <li><code>{html.escape(item.split(' - ')[0])}</code> - {html.escape(polish_prose(' - '.join(item.split(' - ')[1:])))}</li>"
        if " - " in item
        else f"        <li>{html.escape(polish_prose(item))}</li>"
        for item in data["attribute_catalog"]
    )

    applicability = "\n".join(
        f"      <p>{html.escape(polish_prose(text))}</p>" for text in data.get("applicability_notes", [])
    )

    return f"""
    <section class=\"container section panel docs-content\">\n      <h2>Complete Attribute Catalog</h2>\n      <p>This list is aligned with the compiler sources in <code>thrustc_attributes</code>, <code>thrustc_token_type</code>, and <code>thrustc_attribute_checker</code>.</p>\n      <ul>\n{catalog}\n      </ul>\n    </section>\n\n    <section class=\"container section panel docs-content\">\n      <h2>Where Each Attribute Applies</h2>\n{applicability}\n    </section>\n"""


def render_reference_sections(data: dict):
    if "reference_sections" not in data:
        return ""

    blocks = []

    for section in data["reference_sections"]:
        intro = section.get("intro", "")
        intro_html = f"      <p>{html.escape(polish_prose(intro))}</p>\n" if intro else ""
        items = "\n".join(
            f"        <li><code>{html.escape(item.split(' - ')[0])}</code> - {html.escape(polish_prose(' - '.join(item.split(' - ')[1:])))}</li>"
            if " - " in item
            else f"        <li>{html.escape(polish_prose(item))}</li>"
            for item in section.get("items", [])
        )
        blocks.append(
            f"""    <section class=\"container section panel docs-content\">\n      <h2>{html.escape(section['title'])}</h2>\n{intro_html}      <ul>\n{items}\n      </ul>\n    </section>"""
        )

    return "\n\n".join(blocks)


def polish_prose(text: str) -> str:
    polished = text
    polished = re.sub(r"\s+,", ",", polished)
    polished = re.sub(r"\s{2,}", " ", polished)

    return polished.strip()


def render_std_page(version: str, badge: str, slug: str, data: dict):
    signatures = std_signatures_for_source(data["source"], version)
    signature_block = "\n".join(signatures[:220])
    source_view = f"thrustc/std/{version}/{data['source']}"
    canonical = f"/documentation/{version}/std/{slug}/"
    social_image = f"/documentation/{version}/social/std/{slug_filename(slug)}"

    return f"""{render_head(data['title'] + ' ' + version, data['summary'], canonical, og_title=f"{data['title']} | Thrust", og_description=data['summary'], og_image=social_image, og_url=canonical)}
  <main class=\"site-main\">\n{docs_hero(data['title'], data['summary'], version, badge)}\n    <section class=\"container section panel docs-content\">\n      <h2>Overview</h2>\n{render_paragraphs(data['overview'])}\n      <p class=\"muted\">Source: <code>{html.escape(source_view)}</code></p>\n    </section>\n\n    <section class=\"container section panel docs-content\">\n      <h2>Public Signatures</h2>\n      <p class=\"muted\">Exported declarations for this module snapshot.</p>\n      <pre class=\"code-thrust\"><code class=\"language-thrust\">{html.escape(signature_block)}</code></pre>\n    </section>\n\n    <section class=\"container section panel docs-content\">\n      <h2>Behavior and Use</h2>\n{render_paragraphs(data['details'])}\n    </section>\n\n    <section class=\"container section panel docs-content\">\n      <h2>Examples</h2>\n{render_code_blocks(data['examples'])}\n    </section>\n\n    <section class=\"container section panel docs-content\">\n      <h2>Notes</h2>\n      <ul>\n{render_items(data['notes'])}\n      </ul>\n      <p><a class=\"btn\" href=\"{doc_href(f'/documentation/{version}/std/')}\">Back to std index</a></p>\n    </section>\n  </main>\n{FOOTER}"""


def render_lang_page(version: str, badge: str, slug: str, data: dict):
    extra_sections = render_attribute_sections(data) + render_reference_sections(data)

    return f"""{render_head(data['title'] + ' ' + version, data['summary'], f'/documentation/{version}/language-reference/{slug}/')}
  <main class=\"site-main\">\n{docs_hero(data['title'], data['summary'], version, badge)}\n    <section class=\"container section panel docs-content\">\n      <h2>Overview</h2>\n{render_paragraphs(data['overview'])}\n      <p class=\"muted\">Primary source: <code>{html.escape(data['source'])}</code></p>\n    </section>\n\n    <section class=\"container section panel docs-content\">\n      <h2>Syntax Signatures</h2>\n      <pre class=\"code-thrust\"><code class=\"language-thrust\">{html.escape(data['signatures'])}</code></pre>\n    </section>\n\n    <section class=\"container section panel docs-content\">\n      <h2>Behavior and Use</h2>\n{render_paragraphs(data['semantics'])}\n      <h3>Best Practices</h3>\n      <ul>\n{render_items(data['guidance'])}\n      </ul>\n    </section>\n\n{extra_sections}\n\n    <section class=\"container section panel docs-content\">\n      <h2>Example</h2>\n{render_code_blocks(data['example'])}\n      <p><a class=\"btn\" href=\"{doc_href(f'/documentation/{version}/language-reference/')}\">Back to language reference</a></p>\n    </section>\n  </main>\n{FOOTER}"""


def render_std_index(version: str, badge: str, pages: dict):
    cards = []

    for slug, data in pages.items():
        cards.append(
            f"""      <article class=\"feature-card\">\n        <h3>{html.escape(data['title'])}</h3>\n        <p>{html.escape(data['summary'])}</p>\n        <p><a class=\"btn\" href=\"{doc_href(f'/documentation/{version}/std/{slug}/')}\">Open module</a></p>\n      </article>"""
        )

    return f"""{render_head(version + ' Standard Library', 'Standard library module index.', f'/documentation/{version}/std/')}
  <main class=\"site-main\">\n{docs_hero('Standard Library', 'Standard modules and public APIs for this version.', version, badge)}\n{render_docs_search()}\n    <section class=\"container section panel docs-content\">\n      <p>The standard library contains the core modules that most Thrust programs use for everyday work: input and output, memory management, math helpers, dynamic arrays, and C bindings. It is intentionally small and explicit, so each module keeps close to the behavior it exposes.</p>\n      <p>These pages describe what each module provides, how its functions are meant to be used, and which parts map directly to C library behavior. Use this section as the starting point when you need reusable building blocks before writing lower-level bindings yourself.</p>\n    </section>\n    <section class=\"container section panel\">\n      <div class=\"features-grid docs-features-grid\">\n{chr(10).join(cards)}\n      </div>\n      <p><a class=\"btn\" href=\"/documentation/index.html\">Back to documentation hub</a></p>\n    </section>\n  </main>\n{FOOTER}"""


def render_lang_index(version: str, badge: str, pages: dict):
    cards = []

    for slug, data in pages.items():
        cards.append(
            f"""      <article class=\"feature-card\">\n        <h3>{html.escape(data['title'])}</h3>\n        <p>{html.escape(data['summary'])}</p>\n        <p><a class=\"btn\" href=\"{doc_href(f'/documentation/{version}/language-reference/{slug}/')}\">Open topic</a></p>\n      </article>"""
        )

    return f"""{render_head(version + ' Language Reference', 'Language reference index.', f'/documentation/{version}/language-reference/')}
  <main class=\"site-main\">\n{docs_hero('Language Reference', 'Stable syntax topics with behavior notes and practical guidance.', version, badge)}\n{render_docs_search()}\n    <section class=\"container section panel docs-content\">\n      <p>The language reference maps syntax materials into topic pages with plain explanations. Each topic includes a short overview, representative signatures, notes about behavior, and practical guidance.</p>\n      <p>Pages can be read in order or used independently. They prioritize stable constructs and clear behavior over shorthand descriptions.</p>\n    </section>\n    <section class=\"container section panel\">\n      <div class=\"features-grid docs-features-grid\">\n{chr(10).join(cards)}\n      </div>\n      <p><a class=\"btn\" href=\"/documentation/index.html\">Back to documentation hub</a></p>\n    </section>\n  </main>\n{FOOTER}"""


def render_cli_flag_section(flag: dict):
    details = flag.get("details", [])
    notes = flag.get("notes", [])
    examples = flag.get("examples", [])
    details_block = (
        render_paragraphs(details)
        if details
        else "      <p class=\"muted\">No additional details for this flag.</p>"
    )
    notes_block = (
        f"      <ul>\n{render_items(notes)}\n      </ul>"
        if notes
        else "      <p class=\"muted\">No extra notes for this flag.</p>"
    )
    examples_block = (
        render_code_blocks(examples)
        if examples
        else "      <p class=\"muted\">No examples for this flag yet.</p>"
    )

    return f"""    <section class=\"container section panel docs-content\">\n      <h2><code>{html.escape(flag['name'])}</code></h2>\n      <p>{html.escape(polish_prose(flag['description']))}</p>\n{render_cli_meta('Aliases', flag.get('aliases'))}\n{render_cli_meta('Accepts value', flag.get('takes_value'))}\n{render_cli_meta('Value syntax', flag.get('value_syntax'))}\n{render_cli_meta('Allowed values', flag.get('allowed_values'))}\n      <h3>Details</h3>\n{details_block}\n      <h3>Examples</h3>\n{examples_block}\n      <h3>Notes</h3>\n{notes_block}\n    </section>"""


def render_cli_category_page(version: str, badge: str, cli_reference: dict, category: dict):
    slug = category["slug"]
    blocks = [render_cli_flag_section(flag) for flag in category.get("flags", [])]

    return f"""{render_head(category['title'] + ' ' + version, category['summary'], f'/documentation/{version}/compiler-command-line-reference/{slug}/')}
  <main class=\"site-main\">\n{docs_hero(category['title'], category['summary'], version, badge)}\n    <section class=\"container section panel docs-content\">\n      <h2>Overview</h2>\n      <p>{html.escape(polish_prose(category['intro']))}</p>\n      <p class=\"muted\">Reference source: <code>{html.escape(cli_reference['source'])}</code></p>\n      <p><a class=\"btn\" href=\"{doc_href(f'/documentation/{version}/compiler-command-line-reference/')}\">Back to command line reference</a></p>\n    </section>\n\n{chr(10).join(blocks)}\n  </main>\n{FOOTER}"""


def render_cli_index(version: str, badge: str, cli_reference: dict):
    cards = []

    for category in cli_reference.get("categories", []):
        slug = category["slug"]
        cards.append(
            f"""      <article class=\"feature-card\">\n        <h3>{html.escape(category['title'])}</h3>\n        <p>{html.escape(category['summary'])}</p>\n        <p><a class=\"btn\" href=\"{doc_href(f'/documentation/{version}/compiler-command-line-reference/{slug}/')}\">Open category</a></p>\n      </article>"""
        )

    return f"""{render_head(cli_reference['title'] + ' ' + version, cli_reference['summary'], f'/documentation/{version}/compiler-command-line-reference/')}
  <main class=\"site-main\">\n{docs_hero(cli_reference['title'], cli_reference['summary'], version, badge)}\n{render_docs_search()}\n    <section class=\"container section panel docs-content\">\n      <h2>Overview</h2>\n{render_paragraphs(cli_reference.get('overview', []))}\n      <p class=\"muted\">Reference source: <code>{html.escape(cli_reference['source'])}</code></p>\n    </section>\n\n    <section class=\"container section panel docs-content\">\n      <h2>Usage</h2>\n{render_code_blocks(cli_reference.get('usage', []))}\n    </section>\n\n    <section class=\"container section panel\">\n      <div class=\"features-grid docs-features-grid\">\n{chr(10).join(cards)}\n      </div>\n      <p><a class=\"btn\" href=\"/documentation/index.html\">Back to documentation hub</a></p>\n    </section>\n  </main>\n{FOOTER}"""


def render_docs_search():
    return """    <section class="container section panel docs-search" data-docs-search>
      <h2>Search Documentation</h2>
      <p class="muted">Search the standard library, language reference, and compiler command line reference for this version.</p>
      <label class="docs-search-label" for="docs-search-input">Query</label>
      <input id="docs-search-input" class="docs-search-input" type="search" placeholder="Try vector, deref, @extern, sizeOf, -emit..." data-docs-search-input>
      <div class="docs-search-meta muted" data-docs-search-meta>Type at least two characters.</div>
      <div class="docs-search-results" data-docs-search-results></div>
    </section>
"""


def render_version_redirect(version: str):
    return f"""<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\">
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
  <title>Redirecting | Thrust</title>
  <meta http-equiv=\"refresh\" content=\"0; url=/documentation/index.html\">
  <link rel=\"canonical\" href=\"/documentation/\">
</head>
<body>
  <p>Redirecting to <a href=\"/documentation/index.html\">Documentation</a>.</p>
  <script>window.location.replace('/documentation/index.html');</script>
</body>
</html>
"""


def page_search_text(data: dict, extra: str = ""):
    parts = [data.get("title", ""), data.get("summary", "")]

    for key in ["overview", "details", "semantics", "guidance", "notes", "attribute_catalog", "applicability_notes", "usage"]:
        value = data.get(key)

        if isinstance(value, list):
            parts.extend(value)
        elif isinstance(value, str):
            parts.append(value)

    for key in ["signatures", "example", "examples"]:
        value = data.get(key)

        if isinstance(value, list):
            parts.extend(value)
        elif isinstance(value, str):
            parts.append(value)

    for section in data.get("reference_sections", []):
        parts.append(section.get("title", ""))

        if section.get("intro"):
            parts.append(section["intro"])

        parts.extend(section.get("items", []))

    if extra:
        parts.append(extra)

    return "\n".join(parts)


def cli_category_search_text(cli_reference: dict, category: dict):
    parts = [
        cli_reference.get("title", ""),
        cli_reference.get("summary", ""),
        category.get("title", ""),
        category.get("summary", ""),
        category.get("intro", ""),
    ]

    for flag in category.get("flags", []):
        parts.append(flag.get("name", ""))
        parts.extend(flag.get("aliases", []))

        if flag.get("value_syntax"):
            parts.append(flag["value_syntax"])

        parts.extend(flag.get("allowed_values", []))
        parts.append(flag.get("description", ""))
        parts.extend(flag.get("details", []))
        parts.extend(flag.get("examples", []))
        parts.extend(flag.get("notes", []))

    return "\n".join(part for part in parts if part)


def write_search_index(version: str, pages: dict, cli_reference: dict):
    entries = []

    for slug, data in pages.get("std", {}).items():
        signatures = "\n".join(std_signatures_for_source(data["source"], version)[:220])

        entries.append(
            {
                "title": data["title"],
                "section": "Standard Library",
                "url": doc_href(f"/documentation/{version}/std/{slug}/"),
                "summary": data["summary"],
                "text": page_search_text(data, signatures),
            }
        )

    for slug, data in pages.get("language-reference", {}).items():

        entries.append(
            {
                "title": data["title"],
                "section": "Language Reference",
                "url": doc_href(f"/documentation/{version}/language-reference/{slug}/"),
                "summary": data["summary"],
                "text": page_search_text(data),
            }
        )

    entries.append(
        {
            "title": cli_reference["title"],
            "section": "Compiler Command Line Reference",
            "url": doc_href(f"/documentation/{version}/compiler-command-line-reference/"),
            "summary": cli_reference["summary"],
            "text": page_search_text(cli_reference),
        }
    )

    for category in cli_reference.get("categories", []):
        slug = category["slug"]

        entries.append(
            {
                "title": category["title"],
                "section": "Compiler Command Line Reference",
                "url": doc_href(
                    f"/documentation/{version}/compiler-command-line-reference/{slug}/"
                ),
                "summary": category["summary"],
                "text": cli_category_search_text(cli_reference, category),
            }
        )

    write_json(
        DOCS_ROOT / version / "search-index.json",
        {"version": version, "entries": entries},
    )


def render_docs_hub(versions_data):
    versions = versions_data.get("versions", [])

    if not versions:
        raise ValueError("versions.json has no versions")

    latest = latest_version(versions_data)
    archived = [entry for entry in versions if entry.get("status") == "archived"]
    archived_cards = []

    for entry in archived:
        archived_cards.append(
            f"      <p><a class=\"btn\" href=\"/documentation/{html.escape(entry['id'])}/index.html\">Open {html.escape(entry['id'])}</a></p>"
        )

    archived_html = "".join(archived_cards) if archived_cards else '      <p class="muted">No archived versions yet.</p>'

    return f"""<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\">
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
  <title>Documentation | Thrust</title>
  <meta name=\"description\" content=\"Versioned Thrust documentation hub.\">
  <link rel=\"icon\" type=\"image/png\" sizes=\"32x32\" href=\"/assets/brand/favicon-32.png\">
  <link rel=\"icon\" type=\"image/png\" sizes=\"16x16\" href=\"/assets/brand/favicon-16.png\">
  <link rel=\"canonical\" href=\"/documentation/\">
  <link rel=\"stylesheet\" href=\"/assets/css/site.css\">
  <link rel=\"stylesheet\" href=\"/assets/css/gruvbox-code.css\">
</head>
<body class=\"docs-page docs-home\">
  <header class=\"site-header\">
    <div class=\"container header-row\">
      <a class=\"brand\" href=\"/en/\">
        <img class=\"brand-logo\" src=\"/assets/brand/thrustlang-logo-128.png\" srcset=\"/assets/brand/thrustlang-logo-128.png 128w, /assets/brand/thrustlang-logo-256.png 256w, /assets/brand/thrustlang-logo-512.png 512w\" sizes=\"44px\" alt=\"Thrust logo\">
        <span class=\"brand-title\">Thrust</span>
      </a>
      <nav class=\"nav-links\" aria-label=\"Main navigation\">
        <a href=\"/en/\">Home</a>
        <a href=\"/en/features/\">Features</a>
        <a href=\"/en/downloads/\">Downloads</a>
        <a class=\"current\" href=\"/documentation/index.html\">Documentation</a>
        <a class=\"lang-link\" href=\"/es/\">ES</a>
      </nav>
    </div>
  </header>

  <main class=\"site-main\">\n    <section class=\"container hero\">\n      <h1>Documentation</h1>\n      <p>Versioned documentation for Thrust, with current and archived references.</p>\n      <div class=\"docs-version-row\">\n        <label for=\"docs-version-root\">Version</label>\n        <select id=\"docs-version-root\" class=\"docs-version-select\" data-docs-version-select></select>\n      </div>\n    </section>\n\n    <section class=\"container section panel docs-content\">\n      <h2>Early Stage Notice</h2>\n      <p>Thrust and its standard library are still very young for a systems programming language. Expect future changes in the standard library, along with smaller adjustments in the language reference, including areas such as builtins.</p>\n    </section>\n\n    <section class=\"container section panel docs-search\" data-docs-search>\n      <h2>Search Documentation</h2>\n      <p class=\"muted\">Search the latest standard library, language reference, and compiler command line reference.</p>\n      <label class=\"docs-search-label\" for=\"docs-search-input\">Query</label>\n      <input id=\"docs-search-input\" class=\"docs-search-input\" type=\"search\" placeholder=\"Try vector, deref, @extern, sizeOf, -emit...\" data-docs-search-input>\n      <div class=\"docs-search-meta muted\" data-docs-search-meta>Type at least two characters.</div>\n      <div class=\"docs-search-results\" data-docs-search-results></div>\n    </section>\n\n    <section class=\"container section docs-grid docs-grid-large\">\n      <article class=\"panel doc-card\">\n        <h2>Standard Library</h2>\n        <p class=\"muted\">Core modules and APIs provided by the standard library.</p>\n        <p><a class=\"btn\" href=\"/documentation/{html.escape(latest)}/std/index.html\">View std index</a></p>\n      </article>\n      <article class=\"panel doc-card\">\n        <h2>Language Reference</h2>\n        <p class=\"muted\">Stable syntax, semantics, and language constructs.</p>\n        <p><a class=\"btn\" href=\"/documentation/{html.escape(latest)}/language-reference/index.html\">View language reference</a></p>\n      </article>\n      <article class=\"panel doc-card\">\n        <h2>Compiler Command Line Reference</h2>\n        <p class=\"muted\">Detailed reference for compiler flags, categories, values, and usage.</p>\n        <p><a class=\"btn\" href=\"/documentation/{html.escape(latest)}/compiler-command-line-reference/index.html\">View CLI reference</a></p>\n      </article>\n    </section>\n\n    <section class=\"container section panel\">\n      <h2>Archived Versions</h2>\n      <p class=\"muted\">Archived snapshots remain available at fixed versioned URLs.</p>\n{archived_html}\n    </section>\n  </main>\n  <footer class=\"site-footer\">\n    <div class=\"container footer-row\">\n      <span class=\"muted\">Thrust Programming Language</span>\n      <div class=\"social\">\n        <a class=\"btn\" href=\"https://github.com/thrustlang\" target=\"_blank\" rel=\"noopener noreferrer\">\n          <svg class=\"icon icon-sm\" aria-hidden=\"true\"><use href=\"/assets/icons/brands.svg#icon-github\"></use></svg>\n          GitHub\n        </a>\n        <a class=\"btn\" href=\"https://discord.gg/MhVpCSxnhV\" target=\"_blank\" rel=\"noopener noreferrer\">\n          <svg class=\"icon icon-sm\" aria-hidden=\"true\"><use href=\"/assets/icons/brands.svg#icon-discord\"></use></svg>\n          Discord\n        </a>\n      </div>\n    </div>\n  </footer>\n  <script src=\"/assets/js/thrust-highlight.js\"></script>\n  <script src=\"/documentation/assets/docs-version-switcher.js\"></script>\n  <script src=\"/documentation/assets/docs-search.js\"></script>\n</body>\n</html>\n"""


def build_version(version: str, versions_data=None) -> None:
    if versions_data is None:
        versions_data = load_versions()

    pages = load_pages(version)
    cli_reference = load_cli_reference(version)
    badge = "Latest" if version == latest_version(versions_data) else "Archived"

    base = DOCS_ROOT / version
    (base / "std").mkdir(parents=True, exist_ok=True)
    (base / "language-reference").mkdir(parents=True, exist_ok=True)
    (base / "compiler-command-line-reference").mkdir(parents=True, exist_ok=True)

    (base / "index.html").write_text(render_version_redirect(version))
    (base / "std" / "index.html").write_text(render_std_index(version, badge, pages.get("std", {})))
    (base / "language-reference" / "index.html").write_text(
        render_lang_index(version, badge, pages.get("language-reference", {}))
    )
    (base / "compiler-command-line-reference" / "index.html").write_text(
        render_cli_index(version, badge, cli_reference)
    )
    write_search_index(version, pages, cli_reference)

    for slug, data in pages.get("std", {}).items():
        page = base / "std" / slug / "index.html"
        page.parent.mkdir(parents=True, exist_ok=True)
        page.write_text(render_std_page(version, badge, slug, data))

    generate_std_social_cards(version, pages.get("std", {}))

    for slug, data in pages.get("language-reference", {}).items():
        page = base / "language-reference" / slug / "index.html"
        page.parent.mkdir(parents=True, exist_ok=True)
        page.write_text(render_lang_page(version, badge, slug, data))

    for category in cli_reference.get("categories", []):
        page = base / "compiler-command-line-reference" / category["slug"] / "index.html"
        page.parent.mkdir(parents=True, exist_ok=True)
        page.write_text(render_cli_category_page(version, badge, cli_reference, category))


def build_all() -> None:
    versions_data = load_versions()

    for entry in versions_data.get("versions", []):
        build_version(entry["id"], versions_data)

    write_docs_hub(versions_data)


def write_docs_hub(versions_data=None) -> None:
    if versions_data is None:
        versions_data = load_versions()

    (DOCS_ROOT / "index.html").write_text(render_docs_hub(versions_data))
