# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1](https://github.com/jeertmans/filesfinder/compare/v0.3.0...v0.3.1) - 2022-10-28

### Chore

- GitHub action now checks that all versions match
- Docker image is smaller

## [0.3.0](https://github.com/jeertmans/filesfinder/compare/v0.2.0...v0.3.0) - 2022-08-05

### Chore

- GitHub action is checked against "itself"

### Added

- GitHub action and Docker image

## [0.2.0](https://github.com/jeertmans/filesfinder/compare/v0.1.0...v0.2.0) - 2022-08-03

### Chore

- Created CI test to check validity against `find`

### Added

- Improved performances
- Non failure on non-utf8 characters

## [0.1.0](https://github.com/jeertmans/filesfinder/commits/v0.1.0) - 2022-07-29

### Chore

- Add various GitHub actions.
- Create first README version.
- Create first CHANGELOG version.

### Added

- Created first CLI version with support for: globs, regex, hidden files, no gitignore, pattern inclusion / exclusion and directory selection.
