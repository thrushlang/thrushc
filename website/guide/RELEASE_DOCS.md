<img src= "https://github.com/thrustlang/.github/blob/main/assets/logos/new%20logo/thrustlang-logo-banner-text-italic.png" alt= "logo" style= "width: 80%; height: 80%;"></img>

# Thrust Website Documentation Release

<img src= "https://github.com/thrustlang/.github/blob/main/assets/standard-text-separator.png" alt= "standard-separator" style= "width: 1hv;"> </img>

`scripts/release_docs.py` creates a new documentation version from an existing content version, promotes it to the latest public version, and regenerates the published output.

> [!IMPORTANT]
> The new version must already exist in `thrustc/std/<new-version>/` before this script is run.

## What The Script Does

The script:

1. Checks that the new version does not already exist.
2. Resolves the source version.
3. Verifies that the matching `std` sources exist.
4. Copies `documentation/content/<source-version>/` to the new version.
5. Builds the new generated documentation.
6. Updates `documentation/versions.json`.
7. Rebuilds the full documentation hub and output.

## Command Syntax

```console
$ python scripts/release_docs.py --new-version v0.2.2
```

## Parameters

- `--new-version`: new documentation version to publish
- `--from`: optional source version used as the base content

## Examples

### Release From The Current Latest Version

```console
$ python scripts/release_docs.py --new-version v0.2.2
```

### Release From A Specific Existing Version

```console
$ python scripts/release_docs.py --new-version v0.2.2 --from v0.2.1
```

## Files Touched

The script may update:

- `documentation/content/<new-version>/pages.json`
- `documentation/versions.json`
- `documentation/<new-version>/...`
- `documentation/index.html`

## Failure Behavior

> [!NOTE]
> The current implementation performs a partial rollback if the new version cannot be built after the content copy step.

That rollback restores:

- `documentation/versions.json`
- `documentation/content/<new-version>/`
- `documentation/<new-version>/`

## Common Errors

### Version Already Exists

The requested version is already present in `documentation/versions.json` or in the content tree.

### Unknown Source Version

The version passed through `--from` is not known by the documentation system.

### Missing Std Sources

The script could not find the matching directory in `thrustc/std/<new-version>/`.

## Notes

> [!WARNING]
> Release documentation only after the corresponding compiler-side `std` version is already present and stable enough to publish.
