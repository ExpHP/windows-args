# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2019-06-18
### Added
- This CHANGELOG file.
- Support for Unix.  This is limited to the `str`-based APIs;
  the `OsStr`-based APIs remain exclusive to Windows.

### Changed
- Changed the behavior for the input `""` to produce `[""]` rather than
  `["TEST.EXE"]`. The new output matches the behavior for an input like `" "`.

### Removed
- `Args::parse_args_os`, `Args::parse_cmd_os`, and `NonUtf8ArgError`.
  These don't pull their weight. Use `ArgsOs` instead, and handle conversion
  errors yourself if you need `String`s.

## [0.1.0] - 2019-06-16
### Added
- `Args` and `ArgsOs`, with `parse_args` and `parse_cmd` methods.

[Unreleased]: https://github.com/ExpHP/windows-args/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ExpHP/windows-args/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ExpHP/windows-args/releases/tag/v0.1.0
