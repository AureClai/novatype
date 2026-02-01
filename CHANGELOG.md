# Changelog

All notable changes to NovaType will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CHANGELOG.md file for tracking changes

### Fixed
- Clippy warnings in `nova-wasm` (redundant closure, unnecessary `to_string()`)
- Test failures in `nova-plot` (missing `FromStr` import)
- Test failures in `nova-wasm` (unstable `as_str()` feature, `LazyHash` methods)
- Skip logic for `compile_creates_output` test when typst binary not available

## [0.1.0] - 2024-01-15

### Added

#### Core Features
- **nova-core**: Orchestration engine for document compilation
- **nova-cli**: Command-line interface with subcommands:
  - `nova init` - Initialize new projects with templates
  - `nova compile` - Compile documents to PDF/SVG/PNG
  - `nova watch` - Watch mode with auto-recompilation
  - `nova validate` - Validate document metadata against schemas
  - `nova template` - Template management (list, info, create)

#### Schema & Validation
- **nova-schema**: JSON Schema validation for document metadata
  - YAML and TOML frontmatter parsing
  - Built-in schemas for article, report, book templates
  - Custom schema support

#### Citations
- **nova-cite**: Citation and bibliography management
  - BibTeX file parsing
  - CrossRef API integration for DOI resolution
  - Multiple citation styles (IEEE, APA, Chicago, MLA, Vancouver)
  - Citation caching for performance

#### Data Visualization
- **nova-plot**: Chart generation from data
  - Line, bar, scatter, pie, area charts
  - CSV and JSON data source support
  - Customizable styling (colors, dimensions, labels)
  - SVG output

#### WebAssembly
- **nova-wasm**: Browser-based compilation
  - Full Typst compilation in browser
  - SVG page rendering
  - Virtual file system for embedded assets (bibliography, etc.)
  - Frontmatter parsing

#### Templates
- Built-in templates:
  - `article` - Simple academic article
  - `ieee-article` - IEEE conference format
  - `nature-article` - Nature journal style
  - `report` - Technical reports
  - `book` - Long-form documents
  - `presentation` - Slides
  - `cv` - Curriculum vitae
  - `letter` - Formal correspondence

#### Documentation
- Complete documentation website (`docs/`)
- Interactive playground with live WASM compilation
- Template gallery with live previews
- API documentation for all crates

#### CI/CD
- GitHub Actions workflow for releases
- Multi-platform builds (Linux x64/ARM64, Windows, macOS Intel/ARM)
- Automated crates.io publishing
- WASM build and deployment

### Technical Details
- Rust Edition 2021
- MSRV: 1.80
- Based on Typst 0.13
- Workspace with 6 crates
- 99 unit and integration tests

## [0.0.1] - 2024-01-01

### Added
- Initial project structure
- Basic Typst integration proof of concept

---

[Unreleased]: https://github.com/AureClai/novatype/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/AureClai/novatype/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/AureClai/novatype/releases/tag/v0.0.1
