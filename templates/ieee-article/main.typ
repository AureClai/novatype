// IEEE Article Template for NovaType
// Version 1.0.0

#let ieee-article(
  title: none,
  authors: (),
  abstract: none,
  keywords: (),
  date: none,
  two_column: true,
  conference: false,
  body,
) = {
  // Document metadata
  set document(
    title: title,
    author: authors.map(a => a.name),
  )

  // Page setup
  set page(
    paper: "us-letter",
    margin: (
      top: 0.75in,
      bottom: 1in,
      left: 0.625in,
      right: 0.625in,
    ),
  )

  // Typography
  set text(
    font: "Times New Roman",
    size: 10pt,
  )

  set par(
    justify: true,
    leading: 0.5em,
  )

  // Heading styles
  set heading(numbering: "I.A.1.")

  show heading.where(level: 1): it => {
    set text(size: 10pt, weight: "bold")
    set align(center)
    block(above: 1.5em, below: 1em)[
      #counter(heading).display("I.")
      #smallcaps(it.body)
    ]
  }

  show heading.where(level: 2): it => {
    set text(size: 10pt, style: "italic")
    block(above: 1em, below: 0.5em)[
      #counter(heading).display("I.A.")
      #it.body
    ]
  }

  // Title
  align(center)[
    #block(text(size: 24pt, weight: "bold")[#title])
    #v(1em)

    // Authors
    #for (i, author) in authors.enumerate() {
      if i > 0 { [, ] }
      [#author.name]
    }

    #v(0.5em)

    // Affiliations
    #set text(size: 9pt)
    #for author in authors {
      if author.at("affiliation", default: none) != none {
        block[#author.affiliation]
      }
      if author.at("email", default: none) != none {
        block[#author.email]
      }
    }

    #v(1em)
  ]

  // Abstract
  if abstract != none {
    block(width: 100%, inset: (x: 0.5in))[
      #set text(size: 9pt)
      #set par(first-line-indent: 0pt)
      *Abstract*—#abstract
    ]
    v(1em)
  }

  // Keywords
  if keywords.len() > 0 {
    block(width: 100%, inset: (x: 0.5in))[
      #set text(size: 9pt, style: "italic")
      *Index Terms*—#keywords.join(", ")
    ]
    v(1em)
  }

  // Main content
  if two_column {
    columns(2, gutter: 0.25in)[
      #body
    ]
  } else {
    body
  }
}

// Export the template function
#let template = ieee-article
