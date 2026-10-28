# Changelog

## 0.1.1 - 2026-10-29

### Added
- Dynamic progress labels for single-file and batch conversions.
- GUI option to keep or replace existing output files.
- Prefilled filename in the output save dialog.
- `--binary` option for the Linux installer.

### Changed
- Updated toast notification palette.
- Linux installer copies an explicit binary to `~/.local/bin`.

## 0.1.0 - 2026-10-26

### Added
- Initial Rust workspace with `heic-converter-core` and `heic-converter-cli`.
- Optional Slint GUI behind the `gui` Cargo feature.
- CLI command to inspect a PDF.
- GitHub Actions for continuous integration and tagged releases.