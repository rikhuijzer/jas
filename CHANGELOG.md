# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-04-08

### Added

- Support multiple `--archive-filename`s ([#22](https://github.com/rikhuijzer/jas/pull/22))

### Changed

- Renamed `--binary-filename` to `--executable-filename` ([#20](https://github.com/rikhuijzer/jas/pull/20))
- Retry downloading on timeout ([#22](https://github.com/rikhuijzer/jas/pull/22))

### Fixed

- Fix Pandoc installation and a related path bug ([#20](https://github.com/rikhuijzer/jas/pull/20))

## [0.2.0] - 2025-04-04

### Added

- `--gh-token` flag to avoid rate limiting ([#14](https://github.com/rikhuijzer/jas/pull/14))
- Setup cargo audit ([#13](https://github.com/rikhuijzer/jas/pull/13))

### Changed

- Switched to ureq since it is a smaller dependency that reqwest ([#11](https://github.com/rikhuijzer/jas/pull/11))
- Move add to path into build ([#10](https://github.com/rikhuijzer/jas/pull/10))

## [0.1.0] - 2025-04-03

Initial release.

[0.2.0]: https://github.com/rikhuijzer/jas/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/rikhuijzer/jas/releases/tag/v0.1.0
