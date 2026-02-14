# NovaType - Document Composition System

## Overview

NovaType is a modern document composition system built on Typst, designed to replace LaTeX with a more intuitive approach.

## Project Structure

```
novatype/
├── typst/                    # Typst fork with NovaType crates
│   ├── crates/
│   │   ├── nova-core/        # Orchestration engine
│   │   ├── nova-schema/      # JSON Schema validation
│   │   ├── nova-cite/        # Citation management
│   │   ├── nova-plot/        # Data visualization
│   │   ├── nova-cli/         # Command-line interface
│   │   ├── nova-font/        # Font management (Google Fonts)
│   │   └── nova-wasm/        # WebAssembly build
│   └── target/
│       └── release/
│           ├── nova.exe      # NovaType CLI
│           └── typst.exe     # Typst compiler
└── mon-article/              # Example project
```

## Commands

```bash
# Initialize new project
nova init <project-name>

# Compile document
nova compile main.typ

# Compile and open
nova compile main.typ --open

# Watch mode (auto-recompile)
nova watch main.typ

# Validate frontmatter
nova validate main.typ

# List templates
nova template list

# Font management
nova font search "mono"           # Search Google Fonts
nova font install "JetBrains Mono" # Install a font
nova font list                     # List installed fonts
nova font bundle minimal           # Install a bundle
nova font cache info               # Show cache info
```

## Key Features

1. **YAML Frontmatter**: Metadata in YAML format, auto-stripped before compilation
2. **Schema Validation**: JSON Schema validation for document metadata
3. **Modern CLI**: Unified tool for init/compile/validate/watch
4. **Citation Support**: BibTeX + CrossRef API integration (nova-cite)
5. **Data Visualization**: CSV/JSON to charts (nova-plot)
6. **Font Management**: Google Fonts integration with local caching (nova-font)

## Development

```bash
# Build in debug mode (faster, use during development)
cargo build --package novatype-cli

# Build in release mode (optimized, for final testing)
cargo build --package novatype-cli --release

# Run tests
cargo test --package novatype-core nova-schema nova-cite nova-plot novatype-cli

# Debug binary location
target/debug/nova.exe

# Release binary location
target/release/nova.exe
```

**Note**: Prefer debug builds during development for faster iteration. Release builds take 4-6 minutes.

## Typst Syntax Quick Reference

### Text Formatting
- `*bold*`, `_italic_`
- `= H1`, `== H2`, `=== H3`
- `- bullet`, `+ numbered`

### Math
- Inline: `$x^2$`
- Block: `$ sum_(i=1)^n i^2 $`
- Labeled: `$ E = mc^2 $ <eq:einstein>`
- Reference: `@eq:einstein`

### Figures & Tables
```typst
#figure(
  image("photo.png", width: 50%),
  caption: [Description],
) <fig:label>

#figure(
  table(columns: 2, [A], [B], [1], [2]),
  caption: [Table caption],
) <tab:label>
```

### Bibliography
```typst
@citation_key          // In text
#bibliography("refs.bib", style: "ieee")
```

## Skill Usage

Use `/novatype` skill to:
- Create new documents: `/novatype new mon-projet`
- Add equations: `/novatype add equation`
- Add figures: `/novatype add figure`
- Compile: `/novatype compile`
