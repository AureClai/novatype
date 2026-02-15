// Professional Report Template for NovaType
// Version 1.0.0

#let report(
  title: none,
  subtitle: none,
  authors: (),
  organization: none,
  date: none,
  version: none,
  abstract: none,
  toc: true,
  cover: true,
  body,
) = {
  set document(
    title: title,
    author: authors.map(a => a.name),
  )

  // Colors
  let primary = rgb("#1e293b")   // slate-800
  let accent = rgb("#2563eb")    // blue-600
  let muted = rgb("#64748b")     // slate-500
  let border = rgb("#e2e8f0")    // slate-200

  set page(
    paper: "a4",
    margin: (top: 2.5cm, bottom: 2cm, left: 2.5cm, right: 2cm),
    numbering: "1",
    number-align: center,
    header: context {
      if counter(page).get().first() > 1 {
        set text(size: 8pt, fill: muted)
        if title != none { title } else { "" }
        h(1fr)
        if date != none { date }
        v(-2pt)
        line(length: 100%, stroke: 0.5pt + border)
      }
    },
    footer: context {
      set text(size: 8pt, fill: muted)
      if organization != none { organization } else { "" }
      h(1fr)
      counter(page).display("1 / 1", both: true)
    },
  )

  set text(font: "Inter", size: 10pt, fill: primary, lang: "en")
  set par(justify: true, leading: 0.7em)
  set heading(numbering: "1.1.")

  show heading.where(level: 1): it => {
    pagebreak(weak: true)
    v(8pt)
    block(below: 16pt)[
      #text(size: 18pt, weight: "bold", fill: primary)[
        #counter(heading).display("1.") #it.body
      ]
      #v(-2pt)
      #line(length: 100%, stroke: 1pt + border)
    ]
  }

  show heading.where(level: 2): it => {
    block(above: 22pt, below: 8pt)[
      #text(size: 13pt, weight: "semibold", fill: primary)[
        #counter(heading).display("1.1.") #it.body
      ]
    ]
  }

  show heading.where(level: 3): it => {
    block(above: 14pt, below: 6pt)[
      #text(size: 11pt, weight: "semibold", fill: primary)[#it.body]
    ]
  }

  // Cover page
  if cover {
    // Accent line
    place(
      left + top,
      dx: -2.5cm,
      dy: -2.5cm,
      rect(width: 1pt, height: 100% + 5cm, fill: accent),
    )

    v(6cm)

    if organization != none {
      text(size: 11pt, weight: "medium", fill: muted, tracking: 2pt)[#upper(organization)]
      v(1.5em)
    }

    text(size: 28pt, weight: "bold", fill: primary)[#title]

    if subtitle != none {
      v(0.5em)
      text(size: 14pt, fill: muted)[#subtitle]
    }

    v(3cm)
    line(length: 25%, stroke: 1pt + border)
    v(1cm)

    set text(size: 9.5pt)
    grid(
      columns: (100pt, auto),
      row-gutter: 10pt,
      ..if authors.len() > 0 {
        (text(fill: muted)[Authors], authors.map(a => a.name).join(", "))
      },
      ..if date != none {
        (text(fill: muted)[Date], text(fill: primary)[#date])
      },
      ..if version != none {
        (text(fill: muted)[Version], text(fill: primary)[#version])
      },
    )

    v(1fr)
    pagebreak()
  }

  // Executive summary
  if abstract != none {
    text(size: 18pt, weight: "bold", fill: primary)[Executive Summary]
    v(-2pt)
    line(length: 25%, stroke: 1pt + border)
    v(12pt)
    set text(size: 10pt)
    abstract
    pagebreak()
  }

  // Table of contents
  if toc {
    text(size: 18pt, weight: "bold", fill: primary)[Table of Contents]
    v(-2pt)
    line(length: 25%, stroke: 1pt + border)
    v(14pt)
    outline(title: none, indent: 1.5em, depth: 3)
    pagebreak()
  }

  // Body
  body
}

// Export the template function
#let template = report
