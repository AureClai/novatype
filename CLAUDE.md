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
```

## Key Features

1. **YAML Frontmatter**: Metadata in YAML format, auto-stripped before compilation
2. **Schema Validation**: JSON Schema validation for document metadata
3. **Modern CLI**: Unified tool for init/compile/validate/watch
4. **Citation Support**: BibTeX + CrossRef API integration (nova-cite)
5. **Data Visualization**: CSV/JSON to charts (nova-plot)

## Development

```bash
# Build all NovaType crates
cd typst
cargo build --package nova-cli --release

# Run tests
cargo test --package nova-core nova-schema nova-cite nova-plot nova-cli

# 94 tests total across all crates
```

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
