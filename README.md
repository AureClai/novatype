<p align="center">
  <img src="docs/assets/images/logo.svg" alt="NovaType Logo" width="120" height="120">
</p>

<h1 align="center">NovaType</h1>

<p align="center">
  <strong>Typesetting, reimagined.</strong>
</p>

<p align="center">
  <a href="https://aureclai.github.io/novatype/">Documentation</a> •
  <a href="https://aureclai.github.io/novatype/demos.html">Demos</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="https://marketplace.visualstudio.com/items?itemName=aureclai.novatype">VS Code Extension</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License MIT">
  <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust 1.70+">
  <img src="https://img.shields.io/badge/platform-windows%20%7C%20macos%20%7C%20linux-lightgrey.svg" alt="Platform">
  <a href="https://marketplace.visualstudio.com/items?itemName=aureclai.novatype">
    <img src="https://img.shields.io/visual-studio-marketplace/v/aureclai.novatype?label=VS%20Code&logo=visual-studio-code" alt="VS Code Extension">
  </a>
</p>

---

NovaType is a modern typesetting system built on [Typst](https://typst.app). It combines the quality of LaTeX with an intuitive syntax, instant compilation, and a project-oriented workflow powered by `nova.toml`.

## Why NovaType?

| Feature | NovaType | LaTeX | Word |
|---------|:--------:|:-----:|:----:|
| Instant compilation | ✅ | ❌ | ✅ |
| Mathematical equations | ✅ | ✅ | ❌ |
| Simple syntax | ✅ | ❌ | ✅ |
| Professional typography | ✅ | ✅ | ❌ |
| Project config (`nova.toml`) | ✅ | ❌ | ❌ |
| Python figure generation | ✅ | ❌ | ❌ |
| External data injection | ✅ | ❌ | ❌ |
| Data visualization | ✅ | ❌ | ✅ |
| Browser execution | ✅ | ❌ | ❌ |

## Features

- **Project configuration** — `nova.toml` as the single source of truth for your project
- **Zero-arg compile** — `nova init` then `nova compile` just works, no arguments needed
- **Python figures** — Generate publication-quality figures from Python scripts (matplotlib, plotly, etc.)
- **Data injection** — Load JSON, CSV, TOML, or YAML data files as Typst variables
- **Font management** — Configure fonts per-project in `nova.toml`
- **Instant compilation** — See your changes in real-time
- **Intuitive syntax** — Markdown-like, without LaTeX boilerplate
- **Smart citations** — BibTeX, CrossRef API, Zotero integration
- **Professional templates** — IEEE, Nature, reports, CV...
- **WebAssembly** — Compile in the browser

## Installation

```bash
# Via Cargo (recommended)
cargo install nova-cli

# Verify installation
nova --version
```

<details>
<summary>Other installation methods</summary>

### From source

```bash
git clone https://github.com/AureClai/novatype.git
cd novatype/typst
cargo build --package nova-cli --release
```

### Windows (binaries)

Download from [Releases](https://github.com/AureClai/novatype/releases).

</details>

## VS Code Extension

Get the full NovaType experience in Visual Studio Code:

```
ext install aureclai.novatype
```

**Features:**
- Live PDF preview with auto-refresh
- IntelliSense for references (`@`) and labels (`<`)
- Bibliography search via CrossRef API
- DOI to BibTeX import
- Syntax highlighting

[![VS Code Extension](https://img.shields.io/visual-studio-marketplace/v/aureclai.novatype?style=for-the-badge&logo=visual-studio-code&label=Install%20Extension)](https://marketplace.visualstudio.com/items?itemName=aureclai.novatype)

## Usage

```bash
# Create a new project
nova init my-article

# Compile (uses [document].main from nova.toml)
nova compile

# Or specify a file explicitly
nova compile main.typ --open

# Watch mode (auto-recompilation)
nova watch
```

## nova.toml

Every NovaType project is configured through `nova.toml`. After `nova init`, you get a ready-to-use config:

```toml
[project]
name = "my-article"
version = "0.1.0"
authors = ["Your Name"]

[document]
main = "main.typ"

[output]
format = "pdf"          # pdf, svg, png
directory = "build"

[bibliography]
style = "ieee"
bibliography = ["references.bib"]

[python]
python = "python"
figures_dir = "figures"
timeout = 60

[data]
results = "data/results.json"
params = "data/params.yaml"

# [fonts]
# main = "Inter"
# mono = "Fira Code"
# paths = ["./fonts"]

# [watch]
# debounce = 300
# clear = true
```

All settings have sensible defaults. CLI arguments always override `nova.toml` values.

## Example

```typ
---
title: "My Article"
author: "John Doe"
template: article
---

= Introduction

Welcome to *NovaType*! A _simple_ yet powerful syntax.

= Equations

Euler's formula: $ e^(i pi) + 1 = 0 $
```

## Architecture

```
novatype/
├── crates/
│   ├── nova-core/       # Compilation engine, config, data loading
│   ├── nova-cli/        # Command-line interface
│   ├── nova-python/     # Python figure integration
│   ├── nova-font/       # Font management
│   ├── nova-schema/     # Metadata validation
│   ├── nova-cite/       # Citation management
│   ├── nova-plot/       # Data visualization
│   ├── nova-template/   # Template management
│   └── nova-wasm/       # WebAssembly build
└── docs/                # Documentation website
```

## Contributing

Contributions are welcome! Check out the [Issues](https://github.com/AureClai/novatype/issues) to get started.

```bash
# Clone and build
git clone https://github.com/AureClai/novatype.git
cd novatype/typst
cargo build

# Run tests
cargo test --workspace
```

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <a href="https://aureclai.github.io/novatype/">Website</a> •
  <a href="https://github.com/AureClai/novatype/issues">Report a bug</a> •
  <a href="https://github.com/AureClai/novatype/discussions">Discussions</a>
</p>
