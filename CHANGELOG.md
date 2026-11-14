# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.5] - 2026-11-14

### Security
- Updated dependencies to fix every advisory reported by `cargo audit`:
  - `h2`: unbounded empty DATA frames (RUSTSEC-2026-0258).
  - `rustls` 0.23.46 and `rustls-webpki` 0.103.17 (RUSTSEC-2026-0285,
    RUSTSEC-2026-0346, RUSTSEC-2026-0348, RUSTSEC-2026-0349,
    RUSTSEC-2026-0382). These were listed in `Cargo.lock` only and never
    compiled into the binary, which uses native-tls.
  - `tokio` 1.54.1 and `rustix`: unsound code (RUSTSEC-2026-0386,
    RUSTSEC-2026-0486).

### Changed
- Upgraded `ratatui` 0.29 → 0.30 and `crossterm` 0.28 → 0.29. This removes
  the unsound `lru` 0.12 crate (RUSTSEC-2026-0002, RUSTSEC-2026-0253) and the
  unmaintained `paste` crate (RUSTSEC-2024-0436).

## [1.1.4] - 2026-06-28

### Changed
- README: added a version badge and removed the screenshots.

## [1.1.3] - 2026-06-28

### Added
- Status text scrolls in rainbow colors while a station is playing.

## [1.1.2] - 2026-06-28

### Removed
- Pause/resume.

### Changed
- Added doc comments to all public types and functions.
- README: added Mermaid flow diagrams.

## [1.1.0] - 2026-06-27

### Added
- The Now Playing field updates with live ICY metadata from the stream.

## [1.0.0] - 2026-06-27

### Added
- First release: a terminal UI player for SomaFM internet radio, with
  playback through `mpv`.
- Volume control with a visual volume bar in the status line.
- Search and filter for the station list.
- MIT license.

[Unreleased]: https://github.com/sgtnasty/somars/compare/v1.1.5...HEAD
[1.1.5]: https://github.com/sgtnasty/somars/compare/v1.1.4...v1.1.5
[1.1.4]: https://github.com/sgtnasty/somars/compare/v1.1.3...v1.1.4
[1.1.3]: https://github.com/sgtnasty/somars/compare/v1.1.2...v1.1.3
[1.1.2]: https://github.com/sgtnasty/somars/compare/v1.1.0...v1.1.2
[1.1.0]: https://github.com/sgtnasty/somars/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/sgtnasty/somars/releases/tag/v1.0.0
