# NovaType Document Creation

Create professional documents using NovaType with a collaborative, design-first approach.

## Arguments
- `$ARGUMENTS` - Project name, action, or empty for guided creation

## Core Principle: Seamless Visual Integration

**All graphic elements must integrate organically into the document:**
- Unified color palette shared across Typst, Python plots, SVG diagrams
- Consistent typography and spacing
- No visual "foreignness" - every element feels native to the document

---

## Instructions

### Phase 1: Discovery Dialogue

Before creating anything, engage the user in a structured conversation:

**1. Document Purpose & Type**
Ask: "What kind of document are you creating?"
- Academic paper / thesis / dissertation
- Technical report / documentation
- Business proposal / presentation
- Creative / editorial content
- Other (describe)

**2. Format & Layout Preferences**
Ask about:
- Paper size (A4, letter, custom)
- Single or two-column layout
- Header/footer content
- Page numbering style
- Margin preferences

**3. Structure & Plan**
Discuss:
- Proposed outline / table of contents
- Number of sections expected
- Abstract / executive summary needs
- Appendices requirements

**4. Visual Elements Inventory**
Ask: "What graphic elements will you need?"
- Mathematical equations (inline, numbered, referenced)
- Data visualizations (charts, plots, graphs)
- Diagrams (flowcharts, architecture, schemas)
- Images (photos, screenshots)
- Tables (data, comparison, reference)
- Code listings

**5. Visual Identity**
Discuss:
- Color preferences (provide 2-3 options if unsure)
- Font preferences (serif, sans-serif, monospace for code)
- Visual tone (minimal, colorful, formal, modern)
- Existing brand guidelines to follow?

---

### Phase 2: Design System Creation

Based on the dialogue, create a cohesive design system:

**theme.typ** (extracted for reuse):
```typst
// NovaType Design System
// Generated from user preferences

// === COLOR PALETTE ===
#let colors = (
  primary: rgb("#2563eb"),     // Main accent
  secondary: rgb("#64748b"),   // Supporting color
  dark: rgb("#1e293b"),        // Text color
  light: rgb("#f8fafc"),       // Background accent
  success: rgb("#22c55e"),     // Positive indicators
  warning: rgb("#f59e0b"),     // Warnings
  error: rgb("#ef4444"),       // Errors/important
)

// === TYPOGRAPHY ===
#let fonts = (
  main: "Inter",               // Body text
  heading: "Inter",            // Headings
  mono: "JetBrains Mono",      // Code
)

// === SPACING SYSTEM ===
#let spacing = (
  xs: 0.25em, sm: 0.5em, md: 1em, lg: 1.5em, xl: 2em
)

// === FIGURE STYLING ===
#let fig-style(content, caption: none, label: none) = {
  figure(
    content,
    caption: if caption != none { [#text(size: 10pt, fill: colors.secondary)[#caption]] },
  )
}
```

**python_theme.py** (for matplotlib/seaborn consistency):
```python
"""NovaType-consistent plotting theme"""

COLORS = {
    'primary': '#2563eb',
    'secondary': '#64748b',
    'dark': '#1e293b',
    'light': '#f8fafc',
    'palette': ['#2563eb', '#64748b', '#22c55e', '#f59e0b', '#ef4444']
}

def apply_novatype_style():
    import matplotlib.pyplot as plt
    plt.rcParams.update({
        'font.family': 'sans-serif',
        'font.sans-serif': ['Inter', 'Arial'],
        'font.size': 10,
        'axes.titlesize': 12,
        'axes.labelsize': 10,
        'axes.edgecolor': COLORS['dark'],
        'axes.labelcolor': COLORS['dark'],
        'axes.prop_cycle': plt.cycler(color=COLORS['palette']),
        'figure.facecolor': 'white',
        'figure.dpi': 300,
        'savefig.dpi': 300,
        'savefig.transparent': True,
    })
```

**diagram_theme.svg** (base style for SVG diagrams):
```xml
<!-- NovaType SVG Style Reference -->
<style>
  .nt-primary { fill: #2563eb; stroke: #2563eb; }
  .nt-secondary { fill: #64748b; stroke: #64748b; }
  .nt-dark { fill: #1e293b; stroke: #1e293b; }
  .nt-text { font-family: 'Inter', sans-serif; fill: #1e293b; }
  .nt-box { fill: #f8fafc; stroke: #2563eb; stroke-width: 2; }
  .nt-arrow { stroke: #64748b; stroke-width: 2; marker-end: url(#arrowhead); }
</style>
```

---

### Phase 3: Document Scaffolding

Create the main document with user-validated structure:

**main.typ**:
```typst
#import "theme.typ": colors, fonts, spacing, fig-style

#set document(title: "[TITLE]", author: "[AUTHOR]")
#set page(paper: "a4", margin: 2.5cm)
#set text(font: fonts.main, size: 11pt, fill: colors.dark, lang: "fr")
#set par(justify: true, leading: 0.65em)
#set heading(numbering: "1.1")
#set math.equation(numbering: "(1)")

// Heading styles
#show heading.where(level: 1): it => {
  set text(size: 18pt, weight: "bold", fill: colors.primary)
  v(spacing.lg); block(it); v(spacing.sm)
  line(length: 100%, stroke: 2pt + colors.primary)
  v(spacing.md)
}

#show heading.where(level: 2): it => {
  set text(size: 14pt, weight: "semibold", fill: colors.dark)
  v(spacing.md); block(it); v(spacing.sm)
}

// Code blocks styling
#show raw.where(block: true): it => {
  set text(font: fonts.mono, size: 9pt)
  block(fill: colors.light, inset: spacing.md, radius: 4pt, width: 100%, it)
}

// === TITLE PAGE ===
#align(center)[
  #v(3cm)
  #text(size: 28pt, weight: "bold", fill: colors.primary)[
    [DOCUMENT TITLE]
  ]
  #v(1cm)
  #text(size: 14pt, fill: colors.secondary)[
    [SUBTITLE IF ANY]
  ]
  #v(2cm)
  #text(size: 12pt)[
    [AUTHOR NAME]
  ]
  #v(0.5cm)
  #text(size: 11pt, fill: colors.secondary)[
    #datetime.today().display("[day] [month repr:long] [year]")
  ]
]

#pagebreak()

// === TABLE OF CONTENTS ===
#outline(indent: auto, depth: 3)

#pagebreak()

// === CONTENT SECTIONS ===
// [Generated based on user-defined plan]

= Introduction

// [User content placeholder]

= [Section from Plan]

== [Subsection from Plan]

// [User content placeholder]

= Conclusion

// [User content placeholder]

// === BIBLIOGRAPHY ===
// #bibliography("references.bib", style: "ieee")
```

---

### Phase 4: Element Integration Guide

When adding elements, always ensure seamless integration:

**Equations** (use theme colors for annotations):
```typst
$ underbrace(E = m c^2, text(fill: colors.secondary, size: 9pt)[relativité restreinte]) $ <eq:einstein>
```

**Figures from Python** (export with theme):
```python
apply_novatype_style()
fig, ax = plt.subplots(figsize=(6, 4))
# ... plot code ...
fig.savefig('figures/plot.pdf', bbox_inches='tight')
```

Then in Typst:
```typst
#figure(
  image("figures/plot.pdf", width: 85%),
  caption: [Description de la figure avec référence à @eq:einstein.],
) <fig:my-plot>
```

**SVG Diagrams** (use theme classes):
```typst
#figure(
  image("diagrams/schema.svg", width: 70%),
  caption: [Architecture du système.],
) <fig:architecture>
```

**Tables** (styled consistently):
```typst
#figure(
  table(
    columns: (1fr, 2fr, 1fr),
    fill: (_, y) => if y == 0 { colors.light } else { white },
    stroke: 0.5pt + colors.secondary,
    [*Paramètre*], [*Description*], [*Valeur*],
    [Alpha], [Taux d'apprentissage], [0.001],
    [Beta], [Momentum], [0.9],
  ),
  caption: [Hyperparamètres du modèle.],
) <tab:params>
```

---

### Phase 5: Iterative Refinement

After initial creation, offer to:
1. Review and adjust the color palette
2. Modify the document structure
3. Add/remove sections
4. Generate sample visualizations to validate the theme
5. Export style guides for external collaborators

---

## Quick Actions

If `$ARGUMENTS` specifies a direct action:

- `new <name>` - Start Phase 1 dialogue, then create project
- `compile` - Run `nova compile main.typ --open`
- `add equation|figure|table|diagram` - Show themed snippet
- `theme` - Display/edit current design system
- `export-theme` - Export Python/SVG theme files

## Compilation

```bash
cd $WORKING_DIRECTORY/../typst/target/release
./nova.exe compile main.typ --open
```
