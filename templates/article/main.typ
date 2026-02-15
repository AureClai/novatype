// Academic Article Template for NovaType
// Version 1.0.0

#let article(
  title: none,
  authors: (),
  abstract: none,
  keywords: (),
  date: none,
  two_column: false,
  numbered: true,
  body,
) = {
  set document(
    title: title,
    author: authors.map(a => a.name),
  )

  set page(
    paper: "a4",
    margin: (top: 2.5cm, bottom: 2.5cm, left: 2.5cm, right: 2.5cm),
    numbering: "1",
    number-align: center,
  )

  set text(
    font: "New Computer Modern",
    size: 11pt,
    lang: "en",
  )

  set par(justify: true, leading: 0.65em)

  // Heading styles
  if numbered {
    set heading(numbering: "1.1.")
  }

  show heading.where(level: 1): it => {
    set text(size: 14pt, weight: "bold")
    block(above: 2em, below: 1em)[
      #if numbered { counter(heading).display("1.") }
      #it.body
    ]
  }

  show heading.where(level: 2): it => {
    set text(size: 12pt, weight: "bold")
    block(above: 1.5em, below: 0.8em)[
      #if numbered { counter(heading).display("1.1.") }
      #it.body
    ]
  }

  show heading.where(level: 3): it => {
    set text(size: 11pt, weight: "bold", style: "italic")
    block(above: 1.2em, below: 0.6em)[#it.body]
  }

  // Title block
  align(center)[
    #v(2em)
    #text(size: 18pt, weight: "bold")[#title]
    #v(1.2em)

    // Authors
    #for (i, author) in authors.enumerate() {
      if i > 0 { h(1em) }
      text(size: 11pt)[#author.name]
    }

    #v(0.6em)

    // Affiliations
    #set text(size: 9.5pt, fill: luma(100))
    #for author in authors {
      if author.at("affiliation", default: none) != none {
        block(spacing: 0.4em)[#author.affiliation]
      }
      if author.at("email", default: none) != none {
        block(spacing: 0.3em)[#link("mailto:" + author.email)[#author.email]]
      }
    }

    #if date != none {
      v(0.5em)
      text(size: 9.5pt, fill: luma(100))[#date]
    }

    #v(1.5em)
  ]

  // Abstract
  if abstract != none {
    block(width: 100%, inset: (x: 2em))[
      #set text(size: 10pt)
      #set par(justify: true)
      #text(weight: "bold", size: 10pt)[Abstract. ]
      #abstract
    ]
    v(0.8em)
  }

  // Keywords
  if keywords.len() > 0 {
    block(width: 100%, inset: (x: 2em))[
      #set text(size: 9.5pt)
      #text(weight: "bold")[Keywords: ]
      #keywords.join(", ")
    ]
    v(1.5em)
  }

  // Separator
  line(length: 100%, stroke: 0.5pt + luma(200))
  v(1em)

  // Main content
  if two_column {
    columns(2, gutter: 1.5em)[#body]
  } else {
    body
  }
}

// Export the template function
#let template = article
