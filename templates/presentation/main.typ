// Presentation Template for NovaType
// Version 1.0.0

#let presentation(
  title: none,
  subtitle: none,
  authors: (),
  date: none,
  aspect: "16-9",
  body,
) = {
  set document(
    title: title,
    author: authors.map(a => a.name),
  )

  let primary = rgb("#0f172a")   // slate-900
  let accent = rgb("#2563eb")    // blue-600
  let muted = rgb("#94a3b8")     // slate-400
  let border = rgb("#e2e8f0")    // slate-200
  let bg = rgb("#ffffff")

  // Slide dimensions
  let (w, h) = if aspect == "4-3" {
    (25.4cm, 19.05cm)
  } else {
    (25.4cm, 14.29cm)  // 16:9
  }

  set page(
    width: w,
    height: h,
    margin: (top: 2cm, bottom: 1.5cm, left: 2.5cm, right: 2.5cm),
    fill: bg,
  )

  set text(font: "Inter", size: 20pt, fill: primary, lang: "en")
  set par(justify: false, leading: 0.7em)
  set heading(numbering: none)

  // Each H1 = new slide with title
  show heading.where(level: 1): it => {
    pagebreak(weak: true)
    block(below: 0.8em)[
      #text(size: 32pt, weight: "bold", fill: primary)[#it.body]
      #v(-4pt)
      #line(length: 15%, stroke: 2pt + accent)
    ]
  }

  // H2 = subtitle within a slide
  show heading.where(level: 2): it => {
    block(above: 0.8em, below: 0.5em)[
      #text(size: 24pt, weight: "semibold", fill: primary)[#it.body]
    ]
  }

  // Larger list text for readability
  set list(indent: 12pt, body-indent: 8pt, spacing: 14pt)
  set enum(indent: 12pt, body-indent: 8pt, spacing: 14pt)

  // --- Title slide ---
  v(1fr)

  align(center)[
    #text(size: 40pt, weight: "bold", fill: primary)[#title]

    #if subtitle != none {
      v(0.4em)
      text(size: 22pt, fill: muted)[#subtitle]
    }

    #v(1.5em)
    #line(length: 10%, stroke: 2pt + accent)
    #v(1em)

    #set text(size: 18pt)
    #for (i, author) in authors.enumerate() {
      if i > 0 { [ · ] }
      [#author.name]
    }

    #if date != none {
      v(0.5em)
      text(size: 16pt, fill: muted)[#date]
    }
  ]

  v(1fr)

  // Slide number footer (starts from slide 2)
  set page(
    footer: context {
      set text(size: 12pt, fill: muted)
      h(1fr)
      counter(page).display("1")
    },
  )

  // --- Content slides ---
  body
}

// Export the template function
#let template = presentation
