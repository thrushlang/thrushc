<img src="https://github.com/thrustlang/.github/blob/main/assets/logos/new%20logo/thrustlang-logo-banner-text-italic.png" alt="logo" style="width: 80%; height: 80%;"></img>

# Thrust Website Documentation Creation

<img src="https://github.com/thrustlang/.github/blob/main/assets/standard-text-separator.png" alt="standard-separator" style="width: 1hv;"></img>

`scripts/create_docs.py` creates a new page scaffold inside an existing versioned documentation section and then rebuilds the generated documentation output.

> [!IMPORTANT]
> This script creates new page entries in `documentation/content/<version>/pages.json`. It does not create a new top-level documentation section.

## What The Script Does

The script:

1. Validates the requested version.
2. Validates the target section.
3. Refuses to overwrite an existing page slug.
4. Creates a complete page scaffold with the required fields for that section.
5. For `std`, validates that the referenced source file exists in `thrustc/std/<version>/`.
6. Rebuilds the generated documentation output.

## Supported Sections

- `std`
- `language-reference`

## Command Syntax

```console
$ python scripts/create_docs.py --version v0.2.1 --section std --slug collections/hashmap --title "std::collections::hashmap" --summary "Hash map container." --source collections/hashmap.thrust
```

## Parameters

- `--version`: documentation version to modify
- `--section`: either `std` or `language-reference`
- `--slug`: page identifier inside that section
- `--title`: page title written into the scaffold
- `--summary`: short page summary written into the scaffold
- `--source`: source reference stored in the page metadata

## Examples

### Create A Standard Library Page

```console
$ python scripts/create_docs.py --version v0.2.1 --section std --slug collections/hashmap --title "std::collections::hashmap" --summary "Hash map container." --source collections/hashmap.thrust
```

### Create A Language Reference Topic

```console
$ python scripts/create_docs.py --version v0.2.1 --section language-reference --slug traits --title "Traits" --summary "Shared behavior contracts." --source syntax/traits/README.md
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

### Unknown Section

The `--section` value is not supported by the scaffold script.

### Page Already Exists

The requested `--slug` already exists in the selected section.

### Missing Std Source

The `--source` path for a `std` page does not exist in `thrustc/std/<version>/`.

## Notes

> [!NOTE]
> Slugs may contain `/`. This is how nested output paths such as `collections/vector` are represented.

> [!NOTE]
> The generated scaffold is only a starting point. After creation, use `scripts/update_docs.py` to refine individual fields.
