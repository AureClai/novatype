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
  <a href="#usage">Usage</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License MIT">
  <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust 1.70+">
  <img src="https://img.shields.io/badge/platform-windows%20%7C%20macos%20%7C%20linux-lightgrey.svg" alt="Platform">
</p>

---

NovaType is a modern typesetting system built on [Typst](https://typst.app). It combines the quality of LaTeX with an intuitive syntax and instant compilation.

## Why NovaType?

| Feature | NovaType | LaTeX | Word |
|---------|:--------:|:-----:|:----:|
| Instant compilation | ✅ | ❌ | ✅ |
| Mathematical equations | ✅ | ✅ | ❌ |
| Simple syntax | ✅ | ❌ | ✅ |
| Professional typography | ✅ | ✅ | ❌ |
| Data visualization | ✅ | ❌ | ✅ |
| Browser execution | ✅ | ❌ | ❌ |

## Features

- **Instant compilation** — See your changes in real-time
- **Intuitive syntax** — Markdown-like, without LaTeX boilerplate
- **Smart citations** — BibTeX, CrossRef API, Zotero integration
- **Data visualization** — Charts from CSV/JSON
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

## Usage

```bash
# Create a new project
nova init my-article

# With a specific template
nova init my-article --template ieee-article

# Compile
nova compile main.typ --open

# Watch mode (auto-recompilation)
nova watch main.typ
```

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

= Visualization

#nova-plot(
  data: "results.csv",
  type: "bar",
  title: "Results"
)
```

## Architecture

```
novatype/
├── typst/crates/
│   ├── nova-core/      # Compilation engine
│   ├── nova-schema/    # Metadata validation
│   ├── nova-cite/      # Citation management
│   ├── nova-plot/      # Data visualization
│   ├── nova-cli/       # Command-line interface
│   └── nova-wasm/      # WebAssembly build
└── docs/               # Documentation website
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
