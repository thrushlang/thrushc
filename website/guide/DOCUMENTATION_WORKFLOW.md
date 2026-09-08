<img src= "https://github.com/thrustlang/.github/blob/main/assets/logos/new%20logo/thrustlang-logo-banner-text-italic.png" alt= "logo" style= "width: 80%; height: 80%;"></img>

# Thrust Website Documentation Workflow

<img src= "https://github.com/thrustlang/.github/blob/main/assets/standard-text-separator.png" alt= "standard-separator" style= "width: 1hv;"> </img>

This guide explains how the website documentation system is organized, where the editable content lives, and how to update, release, and archive versioned documentation.

> [!IMPORTANT]
> The editable source of the website documentation lives under `documentation/content/`. The HTML files under `documentation/<version>/` are generated output.

## Overview

The website repository contains the official public pages and the official documentation for the Thrust Programming Language.

The documentation system is versioned. Each version has editable content, generated HTML output, a search index, and a public status in `documentation/versions.json`.

## Source Of Truth

The main inputs are:

- `documentation/content/<version>/pages.json`
- `documentation/content/<version>/compiler-command-line-reference.json`
- `documentation/versions.json`
- `thrustc/std/<version>/...`
- `thrustc/thrustc_cli/src/help.rs`

`pages.json` stores the editable text, examples, summaries, and structured page content.

`versions.json` stores the version list, the current latest version, and the public status shown in the documentation hub.

The `thrustc/std/<version>/...` tree is used to extract public signatures for standard library pages.

The compiler command line reference must be synchronized with `thrustc/thrustc_cli/src/help.rs`. Flags that exist only in parser internals are not public documentation until they are listed in the help output.

## Generated Output

The generated output lives under `documentation/<version>/`.

That output includes:

- `documentation/<version>/index.html`
- `documentation/<version>/std/...`
- `documentation/<version>/language-reference/...`
- `documentation/<version>/search-index.json`
- `documentation/<version>/social/std/*.png`

The documentation hub itself is regenerated at `documentation/index.html`.

Standard library pages also receive PNG Open Graph cards under `documentation/<version>/social/std/`. These images are used when `std::*` documentation URLs are shared.

## Maintenance Scripts

The repository contains documentation maintenance scripts under `scripts/`:

- `create_docs.py`
- `update_docs.py`
- `release_docs.py`
- `archive_docs.py`
- `generate_std_social_cards.py`
- `build_subpath.py`

These scripts rebuild the documentation after changing the editable state.

Install Python dependencies before rebuilding generated documentation:

```console
$ python -m pip install -r requirements.txt
```

The documentation build fails if `Pillow` is missing, because standard library pages reference generated PNG social cards.

## Recommended Workflow

1. Create a new page scaffold with `scripts/create_docs.py` when a section needs a new page.
2. Update existing content with `scripts/update_docs.py`.
3. Publish a new documentation version with `scripts/release_docs.py`.
4. Mark older versions as archived with `scripts/archive_docs.py`.

## Subpath Deployment Build

Use `scripts/build_subpath.py` when the website is deployed below a path instead of the domain root. For example, the compiler repository can publish this site at `https://thrustlang.github.io/website/` while keeping Rust code documentation at the root.

```console
$ python scripts/build_subpath.py --base-path /website --output /tmp/thrust-website-build
```

The script copies the website to the output directory and rewrites internal root-relative links for `/website/assets/`, `/website/documentation/`, `/website/en/`, and `/website/es/`. The source checkout is not changed.

## Related Guides

- [Scripts Reference](./DOCS_SCRIPTS_REFERENCE.md)
- [Update Docs](./UPDATE_DOCS.md)
- [Create Docs](./CREATE_DOCS.md)
- [Release Docs](./RELEASE_DOCS.md)
- [Archive Docs](./ARCHIVE_DOCS.md)

## Notes And Limitations

> [!NOTE]
> The documentation generator is intentionally simple. The editable content is stored in JSON and the generated HTML is treated as build output.

> [!NOTE]
> `scripts/update_docs.py` edits existing page fields. Use `scripts/create_docs.py` first when a page does not exist yet.

> [!WARNING]
> Releasing a new documentation version requires matching `std` sources in `thrustc/std/<new-version>/`.
