<img src="https://github.com/thrustlang/.github/blob/main/assets/logos/new%20logo/thrustlang-logo-banner-text-italic.png" alt="logo" style="width: 80%; height: 80%;"></img>

# The Thrust Website

<img src="https://github.com/thrustlang/.github/blob/main/assets/standard-text-separator.png" alt="standard-separator" style="width: 1hv;"></img>

There is a simple guide of standard conventions to follow in order to deliver a good Github commit for the Thrust Website (`website`).

### Title

It needs to be detailed. It can include technical slang. The base of a well designed Github commit title always needs a specific syntax such as:

#### Title - features

Following the syntax:

`feat(...)`

Valid locations:

- `site` Any location that usually involves the public pages of the website, including the landing pages and general HTML structure.
- `docs` Any location that usually involves the versioned documentation, editable documentation content, documentation assets, or documentation pages under `documentation/`.
- `assets` Any location that usually involves website shared CSS, Javascript, images, icons, and brand resources under `assets/`.
- `scripts` Any location that usually involves the Python tooling used to build, update, release, or archive documentation under `scripts/`.
- `project` Any location that usually involves repository level files, guides, metadata, or general project maintenance.

Example:

`feat(site)` Add a new downloads call to action on the homepage.

#### Title - fixes

Following the syntax:

`fix(...)`

Valid locations:

- `site` Any location that usually involves the public pages of the website, including the landing pages and general HTML structure.
- `docs` Any location that usually involves the versioned documentation, editable documentation content, documentation assets, or documentation pages under `documentation/`.
- `assets` Any location that usually involves website shared CSS, Javascript, images, icons, and brand resources under `assets/`.
- `scripts` Any location that usually involves the Python tooling used to build, update, release, or archive documentation under `scripts/`.
- `project` Any location that usually involves repository level files, guides, metadata, or general project maintenance.

Any consecutive location written next to another one needs to be followed by a COMMA character `,`.

Example:

`fix(docs)` Correct broken links in the language reference pages.

#### Title - Combinatory

In order to create a well designed combinatory title, you need to use the following syntax:

`(feat(...), fix(...))`

- It needs to be encapsulated by a pair of PAREN characters `()`.
- Each next feature or fix needs to be followed by a COMMA character `,`.

Example:

`(feat(site), fix(assets))` Add a homepage banner and correct its shared icon styles.

### Description

It needs to be concise, short, but detailed at the same time. It can include technical slang.
