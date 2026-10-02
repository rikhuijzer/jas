# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-10-02

### Added

- Support x64 and ARM64 release selection on Linux, macOS, and Windows Git Bash.
- Reject unsafe archive paths and links.
- Add offline Rust integration tests and CI coverage for six runner platforms;
  Rust is required only for testing.

### Changed

- Replace the compiled installer and platform-specific release binaries with one
  standalone Bash 3.2-compatible `jas` file and its checksum. Installation now
  requires Bash and system utilities instead of Rust; `cargo install jas` is no
  longer supported for this release.
- Add the install directory to GITHUB_PATH at runtime, including Windows paths.
- Clean up temporary archives after installation.

### Fixed

- Pair multiple archive/output filenames correctly.
- Honor output filename overrides for raw downloads.
- Locate Git Bash automatically in the Windows test harness.
- Handle Windows jq line endings and native paths passed to GNU tar.

### Removed

- Retire Rust-specific audit and Snap build workflows.

## [0.3.2] - 2025-05-24

### Fixed

- Fixed a bug in the snap release system and making a new tag to get this published.

## [0.3.1] - 2025-05-15

### Changed

- Updated dependencies

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

[0.4.0]: https://github.com/rikhuijzer/jas/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/rikhuijzer/jas/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/rikhuijzer/jas/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/rikhuijzer/jas/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/rikhuijzer/jas/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/rikhuijzer/jas/releases/tag/v0.1.0
