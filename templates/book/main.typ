// Book / Thesis Template for NovaType
// Version 1.0.0

#let book(
  title: none,
  subtitle: none,
  authors: (),
  publisher: none,
  date: none,
  abstract: none,
  paper: "a4",
  toc: true,
  body,
) = {
  set document(
    title: title,
    author: authors.map(a => a.name),
  )

  let primary = rgb("#1e293b")
  let muted = rgb("#64748b")
  let border = rgb("#e2e8f0")

  set page(
    paper: paper,
    margin: (top: 2.5cm, bottom: 2.5cm, inside: 3cm, outside: 2.2cm),
    numbering: "1",
    number-align: center + bottom,
    header: context {
      let page-num = counter(page).get().first()
      if page-num > 2 {
        set text(size: 8pt, fill: muted)
        // Even pages: title on left; odd pages: chapter on right
        if calc.rem(page-num, 2) == 0 {
          emph(title)
          h(1fr)
        } else {
          h(1fr)
          context {
            let headings = query(heading.where(level: 1).before(here()))
            if headings.len() > 0 {
              emph(headings.last().body)
            }
          }
        }
        v(-2pt)
        line(length: 100%, stroke: 0.4pt + border)
      }
    },
  )

  set text(
    font: "New Computer Modern",
    size: 11pt,
    fill: primary,
    lang: "en",
  )

  set par(justify: true, leading: 0.7em, first-line-indent: 1em)
  set heading(numbering: "1.1.")

  // Chapter headings (level 1) — start on recto page
  show heading.where(level: 1): it => {
    pagebreak(weak: true, to: "odd")
    v(4cm)
    block(below: 2em)[
      #text(size: 14pt, fill: muted, weight: "regular")[Chapter #counter(heading).display("1")]
      #v(0.5em)
      #text(size: 24pt, weight: "bold", fill: primary)[#it.body]
      #v(0.3em)
      #line(length: 40%, stroke: 1pt + border)
    ]
  }

  show heading.where(level: 2): it => {
    block(above: 2em, below: 1em)[
      #text(size: 14pt, weight: "semibold", fill: primary)[
        #counter(heading).display("1.1.") #it.body
      ]
    ]
  }

  show heading.where(level: 3): it => {
    block(above: 1.5em, below: 0.7em)[
      #text(size: 12pt, weight: "semibold", fill: primary)[#it.body]
    ]
  }

  // --- Half-title page ---
  v(1fr)
  align(center)[
    #text(size: 20pt, weight: "bold", fill: primary)[#title]
  ]
  v(1fr)
  pagebreak()

  // --- Title page ---
  v(3cm)
  align(center)[
    #text(size: 28pt, weight: "bold", fill: primary)[#title]

    #if subtitle != none {
      v(0.8em)
      text(size: 16pt, fill: muted)[#subtitle]
    }

    #v(3cm)

    #for author in authors {
      text(size: 14pt)[#author.name]
      v(0.3em)
    }

    #v(1fr)

    #if publisher != none {
      text(size: 11pt, fill: muted)[#publisher]
      v(0.5em)
    }

    #if date != none {
      text(size: 11pt, fill: muted)[#date]
    }
  ]
  v(2cm)
  pagebreak()

  // --- Copyright page (blank verso) ---
  pagebreak()

  // --- Preface / abstract ---
  if abstract != none {
    heading(level: 1, numbering: none)[Preface]
    set par(first-line-indent: 0pt)
    abstract
    pagebreak()
  }

  // --- Table of contents ---
  if toc {
    heading(level: 1, numbering: none)[Contents]
    outline(title: none, indent: 1.5em, depth: 3)
    pagebreak()
  }

  // --- Main body ---
  counter(page).update(1)
  body
}

// Export the template function
#let template = book
