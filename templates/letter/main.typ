// Formal Letter Template for NovaType
// Version 1.0.0

#let letter(
  sender_name: none,
  sender_address: none,
  sender_email: none,
  sender_phone: none,
  recipient_name: none,
  recipient_address: none,
  date: none,
  subject: none,
  salutation: "Dear Sir or Madam",
  closing: "Sincerely",
  body,
) = {
  set document(title: "Letter to " + recipient_name, author: sender_name)

  let primary = rgb("#1e293b")
  let muted = rgb("#64748b")
  let border = rgb("#e2e8f0")

  set page(
    paper: "a4",
    margin: (top: 2.5cm, bottom: 2.5cm, left: 2.5cm, right: 2.5cm),
  )

  set text(font: "New Computer Modern", size: 11pt, fill: primary, lang: "en")
  set par(justify: true, leading: 0.7em)

  // --- Sender block (top right) ---
  align(right)[
    #set text(size: 10pt)
    #text(weight: "semibold")[#sender_name]

    #if sender_address != none {
      v(2pt)
      set text(size: 9.5pt, fill: muted)
      sender_address
    }

    #set text(size: 9pt, fill: muted)
    #if sender_email != none {
      v(2pt)
      link("mailto:" + sender_email)[#sender_email]
    }
    #if sender_phone != none {
      v(1pt)
      sender_phone
    }
  ]

  v(1.5cm)

  // --- Recipient block (left) ---
  block[
    #set text(size: 10pt)
    #text(weight: "semibold")[#recipient_name]
    #if recipient_address != none {
      v(2pt)
      set text(size: 9.5pt, fill: muted)
      recipient_address
    }
  ]

  v(1cm)

  // --- Date ---
  if date != none {
    align(right)[
      #text(size: 10pt, fill: muted)[#date]
    ]
    v(0.8cm)
  }

  // --- Subject line ---
  if subject != none {
    text(weight: "bold", size: 11pt)[Re: #subject]
    v(0.8cm)
  }

  // --- Salutation ---
  [#salutation,]
  v(0.5cm)

  // --- Body ---
  body

  // --- Closing ---
  v(1cm)
  [#closing,]
  v(2cm)
  text(weight: "semibold")[#sender_name]
}

// Export the template function
#let template = letter
