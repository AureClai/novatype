// Curriculum Vitae Template for NovaType
// Version 1.0.0

#let cv(
  name: none,
  title: none,
  email: none,
  phone: none,
  location: none,
  website: none,
  linkedin: none,
  body,
) = {
  set document(title: name + " — CV", author: name)

  let primary = rgb("#0f172a")   // slate-900
  let accent = rgb("#2563eb")    // blue-600
  let muted = rgb("#64748b")     // slate-500
  let border = rgb("#e2e8f0")    // slate-200
  let light = rgb("#f8fafc")     // slate-50

  set page(
    paper: "a4",
    margin: (top: 2cm, bottom: 2cm, left: 2cm, right: 2cm),
  )

  set text(font: "Inter", size: 9.5pt, fill: primary, lang: "en")
  set par(justify: false, leading: 0.65em)
  set heading(numbering: none)

  // Section headings
  show heading.where(level: 1): it => {
    v(16pt)
    block(below: 8pt)[
      #text(size: 11pt, weight: "bold", fill: primary, tracking: 1pt)[#upper(it.body)]
      #v(-3pt)
      #line(length: 100%, stroke: 1pt + border)
    ]
  }

  show heading.where(level: 2): it => {
    block(above: 10pt, below: 2pt)[
      #text(size: 10pt, weight: "semibold", fill: primary)[#it.body]
    ]
  }

  // --- Header ---
  block(width: 100%, inset: (bottom: 16pt))[
    #text(size: 24pt, weight: "bold", fill: primary)[#name]

    #if title != none {
      v(2pt)
      text(size: 12pt, fill: muted)[#title]
    }

    #v(10pt)

    // Contact row
    #set text(size: 8.5pt, fill: muted)
    #{
      let items = ()
      if email != none { items.push(link("mailto:" + email)[#email]) }
      if phone != none { items.push(phone) }
      if location != none { items.push(location) }
      if website != none { items.push(link(website)[#website]) }
      if linkedin != none { items.push(link(linkedin)[LinkedIn]) }
      items.join([ #h(6pt) | #h(6pt) ])
    }
  ]

  line(length: 100%, stroke: 1.5pt + primary)

  // Body
  body
}

// --- Helper functions ---

// Entry for experience / education
#let entry(
  dates: none,
  title: none,
  organization: none,
  location: none,
  description: none,
) = {
  let muted = rgb("#64748b")

  block(above: 8pt, below: 4pt)[
    #grid(
      columns: (1fr, auto),
      align: (left, right),
      [
        #text(weight: "semibold", size: 10pt)[#title]
        #if organization != none {
          text(fill: muted)[ — #organization]
        }
      ],
      text(size: 8.5pt, fill: muted)[#dates],
    )
    #if location != none {
      text(size: 8.5pt, fill: muted)[#location]
    }
    #if description != none {
      v(2pt)
      set text(size: 9.5pt)
      description
    }
  ]
}

// Skill category
#let skills(category, items) = {
  let muted = rgb("#64748b")
  block(above: 4pt, below: 4pt)[
    #grid(
      columns: (90pt, 1fr),
      text(weight: "semibold", size: 9pt)[#category],
      text(size: 9pt, fill: muted)[#items.join(" · ")],
    )
  ]
}

// Export the template function
#let template = cv
