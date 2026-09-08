<img src="https://github.com/thrustlang/.github/blob/main/assets/logos/new%20logo/thrustlang-logo-banner-text-italic.png" alt="logo" style="width: 80%; height: 80%;"></img>

# Thrust Website Documentation Scripts Reference

<img src="https://github.com/thrustlang/.github/blob/main/assets/standard-text-separator.png" alt="standard-separator" style="width: 1hv;"></img>

This guide is the operational reference for the documentation maintenance scripts under `scripts/`.

It explains what each script does, when to use it, what each flag means, which values each flag expects, and which flag combinations are valid in different situations.

> [!IMPORTANT]
> The editable documentation source lives under `documentation/content/`. The HTML files under `documentation/<version>/` are generated output.

> [!IMPORTANT]
> Standard library pages are checked against `thrustc/std/<version>/`. Compiler command line pages are checked against `thrustc/thrustc_cli/src/help.rs`; flags that only exist in parser internals are not public website documentation.

## Script Index

- `scripts/create_docs.py`: create a new page scaffold in `std` or `language-reference`
- `scripts/update_docs.py`: update existing content fields in an existing page or CLI reference entry
- `scripts/release_docs.py`: publish a new documentation version from an existing one
- `scripts/archive_docs.py`: mark an older documentation version as archived
- `scripts/generate_std_social_cards.py`: generate PNG Open Graph cards for standard library pages
- `scripts/build_subpath.py`: build a deployable site copy for a subpath such as `/website`

## When To Use Each Script

- Use `scripts/create_docs.py` when the page does not exist yet.
- Use `scripts/update_docs.py` when the page or CLI entry already exists and only its content should change.
- Use `scripts/release_docs.py` when a new public version must be published.
- Use `scripts/archive_docs.py` when an older published version should stay accessible but no longer be current.
- Use `scripts/build_subpath.py` when this website must be copied under a non-root URL path.

## Quick Situations Table

| Situation | Script | Core Flags |
| --- | --- | --- |
| Create a new `std` page | `scripts/create_docs.py` | `--version --section std --slug --title --summary --source` |
| Create a new `language-reference` page | `scripts/create_docs.py` | `--version --section language-reference --slug --title --summary --source` |
| Replace one short text field | `scripts/update_docs.py` | `--version --section --slug --field --value` |
| Replace a field from a file | `scripts/update_docs.py` | `--version --section --slug --field --input-file` |
| Replace a list or object field as JSON | `scripts/update_docs.py` | `--version --section --slug/--category/--flag --field --json-value` |
| Update one CLI flag description | `scripts/update_docs.py` | `--version --section compiler-command-line-reference --category --flag --field --value` |
| Create a new published docs version | `scripts/release_docs.py` | `--new-version` |
| Create a new published docs version from a specific base | `scripts/release_docs.py` | `--new-version --from` |
| Archive an old docs version | `scripts/archive_docs.py` | `--version` |
| Regenerate std social cards | `scripts/generate_std_social_cards.py` | optional `--version`, optional `--slug` |
| Build website for `/website` deployment | `scripts/build_subpath.py` | `--base-path /website --output <dir>` |

## `scripts/create_docs.py`

### What It Does

Creates a new page scaffold inside `documentation/content/<version>/pages.json` and rebuilds the generated documentation.

### When To Use It

- when a `std` page does not exist yet
- when a `language-reference` page does not exist yet
- when a nested slug such as `collections/hashmap` must be introduced for the first time

### Command Shape

```console
$ python scripts/create_docs.py --version <version> --section <section> --slug <slug> --title <title> --summary <summary> --source <source>
```

### Flags Index

| Flag | Required | What It Means | Valid Values | Notes |
| --- | --- | --- | --- | --- |
| `--version` | yes | Target documentation version to edit | existing version id like `v0.2.1` | Must already exist in `documentation/versions.json`. |
| `--section` | yes | Section where the page will be created | `std`, `language-reference` | No other top-level sections are supported by this script. |
| `--slug` | yes | Page identifier inside the section | simple or nested slug like `io`, `traits`, `collections/vector` | `/` is allowed and creates nested output paths. |
| `--title` | yes | Human-readable page title stored in the scaffold | any non-empty text | For `std`, use a module-style title such as `std::collections::hashmap`. |
| `--summary` | yes | Short summary shown in cards, metadata, and page intro | any non-empty text | Keep it short and descriptive. |
| `--source` | yes | Source reference saved in the page metadata | relative path string | For `std`, it must exist under `thrustc/std/<version>/`. |

### Valid Value Guidance

- `--version`
  - pass a version already listed in `documentation/versions.json`
  - example: `v0.2.1`
- `--section`
  - pass `std` for standard library modules
  - pass `language-reference` for syntax and language topics
- `--slug`
  - use short slugs for flat pages: `types`, `loops`, `io`
  - use nested slugs for hierarchy: `collections/hashmap`, `ffi/network`
- `--title`
  - `std`: prefer full module names such as `std::io`
  - `language-reference`: prefer topic titles such as `Traits`
- `--source`
  - `std`: use file paths like `io.thrust`, `collections/vector.thrust`
  - `language-reference`: use source references like `syntax/traits/README.md`

### Situational Examples

#### Create A New Standard Library Page

```console
$ python scripts/create_docs.py --version v0.2.1 --section std --slug collections/hashmap --title "std::collections::hashmap" --summary "Hash map container." --source collections/hashmap.thrust
```

Pass these values because:
- `--section std` tells the script to create a standard library template
- `--slug collections/hashmap` creates a nested page path
- `--source collections/hashmap.thrust` must exist in `thrustc/std/v0.2.1/`

#### Create A New Language Reference Topic

```console
$ python scripts/create_docs.py --version v0.2.1 --section language-reference --slug traits --title "Traits" --summary "Shared behavior contracts." --source syntax/traits/README.md
```

Pass these values because:
- `--section language-reference` selects the language topic template
- `--slug traits` becomes the public page path
- `--source` is only metadata here and does not need std validation

### Common Mistakes

- passing a version that is not registered
- reusing a slug that already exists
- using `--section std` with a missing std source file
- trying to create a CLI reference category with this script

### Files It Changes

- `documentation/content/<version>/pages.json`
- `documentation/index.html`
- `documentation/<version>/...`
- `documentation/<version>/search-index.json`

## `scripts/generate_std_social_cards.py`

### What It Does

Generates PNG Open Graph cards for standard library documentation pages. The generated images are used by `std::*` pages through their `og:image` and `twitter:image` metadata.

The script reads `documentation/content/<version>/pages.json` and uses each `std` page title as the large card text.

### Dependency

Install the Python dependencies before running documentation builds:

```console
$ python -m pip install -r requirements.txt
```

The build fails if `Pillow` is not installed.

### Command Shape

```console
$ python scripts/generate_std_social_cards.py [--version <version>] [--slug <std-slug>]
```

### Flags Index

| Flag | Required | What It Means | Valid Values | Notes |
| --- | --- | --- | --- | --- |
| `--version` | no | Documentation version to generate | existing version id like `v0.2.1` | If omitted, all versions are generated. |
| `--slug` | no | One std page slug to generate | existing std slug like `mem` or `collections/vector` | Requires `--version`. |

### Examples

Generate every standard library card for every version:

```console
$ python scripts/generate_std_social_cards.py
```

Generate one version:

```console
$ python scripts/generate_std_social_cards.py --version v0.2.1
```

Generate one module:

```console
$ python scripts/generate_std_social_cards.py --version v0.2.1 --slug mem
```

### Files It Changes

- `documentation/<version>/social/std/*.png`

### Notes

- Cards are generated only for `std::*` documentation pages.
- Language reference pages and compiler command line pages do not receive std social cards.
- `documentation/assets/build_docs.py` also generates these cards as part of the normal documentation build.

## `scripts/build_subpath.py`

### What It Does

Builds a copy of the website that can be served from a non-root path, such as `https://thrustlang.github.io/website/`.

The source website keeps root-relative paths such as `/website/assets/`, `/website/documentation/`, `/website/en/`, and `/website/es/`. This script copies the site to an output directory and rewrites those internal paths with the requested base path.

### When To Use It

- when another repository deploys this website under a subdirectory
- when testing the website from a GitHub Pages path like `/website`
- when the source website should remain usable at the domain root later

### Command Shape

```console
$ python scripts/build_subpath.py --base-path /website --output /tmp/thrust-website-build
```

### Flags Index

| Flag | Required | What It Means | Valid Values | Notes |
| --- | --- | --- | --- | --- |
| `--base-path` | yes | URL prefix where the copied site will be served | path like `/website` | Leading slash is optional. Trailing slash is removed. |
| `--output` | yes | Destination directory for the copied and rewritten site | path outside the repo | Existing output is replaced. |
| `--source` | no | Source website directory | path to a website checkout | Defaults to the current repository root. |

### Rewritten Paths

- `/website/assets/...`
- `/website/documentation/...`
- `/website/en/...`
- `/website/es/...`

External URLs such as `https://github.com/...` are not rewritten.

### Files It Changes

- the directory passed through `--output`

The source checkout is not modified.

## `scripts/update_docs.py`

### What It Does

Updates an existing field in an existing page or CLI reference entry, saves the editable JSON, and rebuilds the generated documentation.

### When To Use It

- when a page already exists and only one field should change
- when a long example should be loaded from a file
- when a CLI category or CLI flag description needs correction
- when an existing list field such as `notes`, `overview`, or `usage` should be replaced

### Command Shape

```console
$ python scripts/update_docs.py --version <version> --section <section> [--slug <slug>] [--category <category>] [--flag <flag>] --field <field> (--value <value> | --input-file <path> | --json-value <json>)
```

### Flags Index

| Flag | Required | What It Means | Valid Values | Notes |
| --- | --- | --- | --- | --- |
| `--version` | yes | Target documentation version to edit | existing version id like `v0.2.1` | Must already exist. |
| `--section` | yes | Which editable document family to target | `std`, `language-reference`, `compiler-command-line-reference` | Controls which of the other locator flags are required. |
| `--slug` | conditional | Page identifier inside `std` or `language-reference` | existing page slug | Required for `std` and `language-reference`. |
| `--category` | conditional | CLI category slug inside `compiler-command-line-reference` | existing category slug like `compiler-flags` | Required when using `--flag`. Optional when editing the category itself. |
| `--flag` | conditional | Specific CLI flag name or alias | existing flag like `-emit`, `--help` | Only valid with `--section compiler-command-line-reference` and `--category`. |
| `--field` | yes | Existing field name to replace | depends on the selected target | The field must already exist. |
| `--value` | one of three | Inline text replacement | any string | Mutually exclusive with `--input-file` and `--json-value`. |
| `--input-file` | one of three | Read replacement text from a file | readable file path | Good for long examples or signatures. |
| `--json-value` | one of three | Provide the replacement as raw JSON | valid JSON string | Use for object fields and for exact list control. |

### Locator Rules

- For `--section std`
  - `--slug` is required
  - `--category` and `--flag` are not used
- For `--section language-reference`
  - `--slug` is required
  - `--category` and `--flag` are not used
- For `--section compiler-command-line-reference`
  - omit `--slug`
  - use no locator flags to edit the root document
  - use `--category` to edit one category
  - use `--category` and `--flag` together to edit one flag in that category
  - `--flag` without `--category` is invalid

### Value Mode Rules

- `--value`
  - use for short text replacement
  - good for `summary`, `title`, `description`, `intro`
- `--input-file`
  - use for long code examples or long signatures
  - the file content is loaded as text
- `--json-value`
  - use when you need exact JSON structure control
  - use for object values
  - use for precise list replacement when item splitting should not be inferred

### Field Behavior Notes

- if the current field is a string, `--value` or `--input-file` stores one string
- if the current field is a list
  - `examples` and `usage` become a single-item list when using `--value` or `--input-file`
  - other list fields are split by blank lines into multiple items when using `--value` or `--input-file`
- if the current field is an object, use `--json-value`

### Common Target Fields By Section

- `std`
  - `title`
  - `summary`
  - `source`
  - `overview`
  - `details`
  - `examples`
  - `notes`
- `language-reference`
  - `title`
  - `summary`
  - `source`
  - `overview`
  - `semantics`
  - `guidance`
  - `signatures`
  - `example`
  - some pages also include `attribute_catalog`, `applicability_notes`, or `reference_sections`
- `compiler-command-line-reference` root
  - `title`
  - `summary`
  - `source`
  - `overview`
  - `usage`
- `compiler-command-line-reference` category
  - `slug`
  - `title`
  - `summary`
  - `intro`
  - `flags`
- `compiler-command-line-reference` flag
  - `name`
  - `aliases`
  - `takes_value`
  - `value_syntax`
  - `allowed_values`
  - `description`
  - `details`
  - `examples`
  - `notes`

### Situational Examples

#### Replace One Summary In `std`

```console
$ python scripts/update_docs.py --version v0.2.1 --section std --slug io --field summary --value "Input and output APIs for streams, files, and formatted printing."
```

Use this shape when the target is one existing `std` page and the replacement is short text.

#### Replace One Long Example From A File

```console
$ python scripts/update_docs.py --version v0.2.1 --section language-reference --slug functions --field example --input-file example.thrust
```

Use this shape when the replacement is too large to manage safely inline.

#### Replace A List Field Exactly

```console
$ python scripts/update_docs.py --version v0.2.1 --section std --slug io --field notes --json-value "[\"Check return values.\", \"Close files after use.\"]"
```

Use `--json-value` when exact item boundaries matter.

#### Update The CLI Reference Root Usage Block

```console
$ python scripts/update_docs.py --version v0.2.1 --section compiler-command-line-reference --field usage --json-value "[\"thrustc main.thrust -opt O2\", \"thrustc main.thrust -emit llvm-ir\"]"
```

Use no `--slug`, no `--category`, and no `--flag` because the target is the root CLI reference document.

#### Update One CLI Category Intro

```console
$ python scripts/update_docs.py --version v0.2.1 --section compiler-command-line-reference --category compiler-flags --field intro --value "These flags shape the compilation pipeline itself."
```

Use `--category` because the target is one category, not the root and not one flag.

#### Update One CLI Flag Description

```console
$ python scripts/update_docs.py --version v0.2.1 --section compiler-command-line-reference --category compiler-flags --flag=-emit --field description --value "Emit one selected compilation artifact."
```

Use `--category` and `--flag` together because the target is one specific flag.

### Common Mistakes

- forgetting `--slug` for `std` or `language-reference`
- passing `--flag` without `--category`
- trying to update a field that does not exist on that target
- passing plain text to a field that expects JSON structure
- using `--value`, `--input-file`, and `--json-value` together

### Files It Changes

- `documentation/content/<version>/pages.json`
- `documentation/content/<version>/compiler-command-line-reference.json`
- `documentation/index.html`
- `documentation/<version>/...`
- `documentation/<version>/search-index.json`

## `scripts/release_docs.py`

### What It Does

Creates a new documentation version from an existing content version, marks it as the latest version, updates `documentation/versions.json`, and rebuilds the published output.

### When To Use It

- when a new documentation release should become public
- when the new compiler or std snapshot already exists and the site needs a matching docs version
- when the latest content is a good base for the next release

### Command Shape

```console
$ python scripts/release_docs.py --new-version <new-version> [--from <source-version>]
```

### Flags Index

| Flag | Required | What It Means | Valid Values | Notes |
| --- | --- | --- | --- | --- |
| `--new-version` | yes | New documentation version to publish | new version id like `v0.2.2` | Must not already exist. |
| `--from` | no | Existing version to copy as the starting point | existing version id like `v0.2.1` | If omitted, the current latest version is used. |

### Valid Value Guidance

- `--new-version`
  - choose a version not already present in `documentation/versions.json`
  - the matching `thrustc/std/<new-version>/` directory must already exist
- `--from`
  - choose an existing version already registered in the docs system
  - omit it when the latest version is the intended base

### Situational Examples

#### Release From The Current Latest Version

```console
$ python scripts/release_docs.py --new-version v0.2.2
```

Use this when `v0.2.1` is already the best content base and `thrustc/std/v0.2.2/` already exists.

#### Release From A Specific Existing Version

```console
$ python scripts/release_docs.py --new-version v0.2.2 --from v0.2.0
```

Use this when the latest version is not the base you want to copy.

### Common Mistakes

- choosing a new version that already exists
- passing `--from` with an unknown version
- releasing before `thrustc/std/<new-version>/` exists
- assuming this script edits one field instead of publishing a new full version

### Files It Changes

- `documentation/content/<new-version>/pages.json`
- `documentation/content/<new-version>/compiler-command-line-reference.json`
- `documentation/versions.json`
- `documentation/<new-version>/...`
- `documentation/index.html`

## `scripts/archive_docs.py`

### What It Does

Marks one existing documentation version as archived in `documentation/versions.json` and rebuilds the generated documentation hub and version output.

### When To Use It

- when an older version should stay online but should no longer appear as current
- when a newer version is already public and an older one should be marked archived

### Command Shape

```console
$ python scripts/archive_docs.py --version <version>
```

### Flags Index

| Flag | Required | What It Means | Valid Values | Notes |
| --- | --- | --- | --- | --- |
| `--version` | yes | Published documentation version to archive | existing non-latest version like `v0.2.0` | The current latest version cannot be archived. |

### Valid Value Guidance

- pass a version already listed in `documentation/versions.json`
- do not pass the current latest version

### Situational Examples

#### Archive An Older Version

```console
$ python scripts/archive_docs.py --version v0.2.0
```

Use this when `v0.2.0` should remain accessible but should no longer be presented as current.

### Common Mistakes

- trying to archive the latest version
- passing a version that is not registered
- expecting the archived files to be deleted

### Files It Changes

- `documentation/versions.json`
- `documentation/index.html`
- `documentation/<version>/...`

## Error Reference

### `unknown version`

The version is not registered in `documentation/versions.json`.

### `unknown section`

The selected section is not supported for that script.

### `page already exists`

The target slug already exists, so `scripts/create_docs.py` refuses to overwrite it.

### `unknown page`

The page slug does not exist, so use `scripts/create_docs.py` first.

### `unknown category`

The CLI category slug does not exist in `compiler-command-line-reference.json`.

### `unknown flag`

The CLI flag name or alias does not exist inside the selected category.

### `unknown field for target`

The selected target exists, but that field is not present on it.

### `invalid value type`

The replacement type does not match the current field type.

### `missing std source`

The requested `std` source path does not exist under `thrustc/std/<version>/`.

### `cannot archive the latest version`

`scripts/archive_docs.py` only supports archiving non-latest versions.

### `version already exists`

`scripts/release_docs.py` requires a brand new version id.

## Recommended Workflow

1. Create a missing page with `scripts/create_docs.py`.
2. Refine the generated content with `scripts/update_docs.py`.
3. Publish a new release with `scripts/release_docs.py` when a new docs version is ready.
4. Archive older versions with `scripts/archive_docs.py` after the new version is public.
