<img src= "https://github.com/thrustlang/.github/blob/main/assets/logos/new%20logo/thrustlang-logo-banner-text-italic.png" alt= "logo" style= "width: 80%; height: 80%;"></img>

# Thrust Website Documentation Update

<img src= "https://github.com/thrustlang/.github/blob/main/assets/standard-text-separator.png" alt= "standard-separator" style= "width: 1hv;"> </img>

`scripts/update_docs.py` updates an existing field inside a versioned documentation page and then rebuilds the documentation site output.

> [!NOTE]
> This script edits `documentation/content/<version>/pages.json`. It does not create a new documentation version.

## What The Script Does

The script:

1. Validates the requested version.
2. Validates the section and page slug.
3. Validates that the requested field already exists.
4. Replaces the field content.
5. Rebuilds the generated documentation output.

## Command Syntax

```console
$ python scripts/update_docs.py --version v0.2.1 --section std --slug io --field summary --value "New summary"
```

## Parameters

- `--version`: documentation version to modify
- `--section`: `std`, `language-reference`, or `compiler-command-line-reference`
- `--slug`: page identifier inside that section
- `--category`: CLI category slug when updating `compiler-command-line-reference`
- `--flag`: CLI flag name or alias when updating one flag inside a CLI category
- `--field`: existing field name to replace
- `--value`: plain inline text value
- `--input-file`: read the new value from a file
- `--json-value`: provide the replacement as raw JSON

## Examples

### Update A Summary

```console
$ python scripts/update_docs.py --version v0.2.1 --section std --slug io --field summary --value "Input and output APIs for streams, files, and formatted printing."
```

### Update A Code Example From A File

```console
$ python scripts/update_docs.py --version v0.2.1 --section language-reference --slug functions --field example --input-file example.thrust
```

### Update A List Field

```console
$ python scripts/update_docs.py --version v0.2.1 --section std --slug io --field notes --json-value "[\"Check return values.\", \"Close files after use.\"]"
```

### Update A CLI Flag Description

```console
$ python scripts/update_docs.py --version v0.2.1 --section compiler-command-line-reference --category compiler-flags --flag -emit --field description --value "Emit one selected compilation artifact."
```

## Files Touched

The script may update:

- `documentation/content/<version>/pages.json`
- `documentation/index.html`
- `documentation/<version>/...`
- `documentation/<version>/search-index.json`

## Common Errors

### Unknown Version

The requested version is not registered in `documentation/versions.json`.

### Unknown Page

The `--section` and `--slug` combination does not exist in the current content manifest.

### Unknown Field

The page exists, but that page does not expose the field you tried to replace.

### Invalid Value Type

The new value does not match the existing field type.

## Notes

> [!IMPORTANT]
> `examples` and other content-heavy fields should be updated carefully. Prefer `--input-file` for long code snippets.

> [!NOTE]
> The script rebuilds the generated output after the content change is saved.
