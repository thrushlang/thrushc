<img src= "https://github.com/thrustlang/.github/blob/main/assets/logos/new%20logo/thrustlang-logo-banner-text-italic.png" alt= "logo" style= "width: 80%; height: 80%;"></img>

# Thrust Website Documentation Archive

<img src= "https://github.com/thrustlang/.github/blob/main/assets/standard-text-separator.png" alt= "standard-separator" style= "width: 1hv;"> </img>

`scripts/archive_docs.py` marks an existing documentation version as archived and regenerates the documentation hub and generated output.

> [!WARNING]
> The latest version cannot be archived with this script.

## What The Script Does

The script:

1. Checks that the requested version exists.
2. Refuses to archive the latest version.
3. Marks the version as archived in `documentation/versions.json`.
4. Rebuilds the documentation hub and generated output.

## Command Syntax

```console
$ python scripts/archive_docs.py --version v0.2.0
```

## Parameters

- `--version`: the published documentation version to archive

## Example

```console
$ python scripts/archive_docs.py --version v0.2.0
```

## Files Touched

The script may update:

- `documentation/versions.json`
- `documentation/index.html`
- `documentation/<version>/...`

## What The Script Does Not Do

The script does not:

- delete the archived documentation
- move the content to another location
- promote another version automatically

## Common Errors

### Cannot Archive The Latest Version

The target version is still the current latest version.

### Unknown Version

The target version is not registered in `documentation/versions.json`.

## Notes

> [!NOTE]
> Archiving is a public status change. The versioned documentation remains available through its stable URL.
