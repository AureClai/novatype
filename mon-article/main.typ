// ============================================
// STYLE MODERNE - NovaType
// ============================================

// Couleurs
#let primary = rgb("#2563eb")    // Bleu principal
#let secondary = rgb("#64748b")  // Gris
#let accent = rgb("#0ea5e9")     // Bleu clair
#let dark = rgb("#1e293b")       // Texte foncé

// Document metadata
#set document(
  title: "Introduction à NovaType",
  author: "Aurec",
  date: datetime(year: 2026, month: 1, day: 29),
)

// Page setup
#set page(
  paper: "a4",
  margin: (x: 3cm, y: 2.5cm),
  header: context {
    if counter(page).get().first() > 1 [
      #set text(size: 9pt, fill: secondary)
      #h(1fr) Introduction à NovaType #h(1fr)
    ]
  },
  footer: context {
    set text(size: 9pt, fill: secondary)
    h(1fr)
    counter(page).display("1 / 1", both: true)
    h(1fr)
  },
)

// Text settings - Police moderne sans-serif
#set text(
  font: "Arial",
  size: 11pt,
  fill: dark,
  lang: "fr",
)

// Paragraphes
#set par(
  justify: true,
  leading: 0.8em,
  first-line-indent: 0em,
)

// Espacement entre paragraphes
#set par(spacing: 1.2em)

// Style des titres
#set heading(numbering: "1.1")

#show heading.where(level: 1): it => {
  set text(size: 18pt, weight: "bold", fill: primary)
  v(1.5em)
  block(it)
  v(0.5em)
  line(length: 100%, stroke: 2pt + primary)
  v(1em)
}

#show heading.where(level: 2): it => {
  set text(size: 14pt, weight: "semibold", fill: dark)
  v(1em)
  block(it)
  v(0.5em)
}

// Style des liens
#show link: it => text(fill: primary, it)

// Style des listes
#set list(marker: text(fill: primary)[•])
#set enum(numbering: n => text(fill: primary, weight: "bold")[#n.])

// Style des équations (numérotation)
#set math.equation(numbering: "(1)")
#show math.equation: set text(size: 11pt)

// ============================================
// CONTENU
// ============================================

// Page de titre
#align(center)[
  #v(3cm)

  #rect(
    width: 100%,
    fill: primary,
    radius: 8pt,
    inset: 2em,
  )[
    #set text(fill: white)
    #text(size: 28pt, weight: "bold")[Introduction à NovaType]

    #v(0.5em)

    #text(size: 14pt)[Le futur de la composition documentaire]
  ]

  #v(2em)

  #text(size: 14pt, weight: "medium")[Aurec]

  #v(0.3em)

  #text(size: 11pt, fill: secondary)[Projet NovaType]

  #v(1em)

  #rect(
    stroke: 1pt + secondary,
    radius: 4pt,
    inset: 0.8em,
  )[
    #text(size: 10pt, fill: secondary)[29 janvier 2026]
  ]
]

#v(3em)

// Abstract
#rect(
  width: 100%,
  fill: rgb("#f1f5f9"),
  radius: 8pt,
  inset: 1.5em,
)[
  #text(weight: "bold", fill: primary)[Résumé]
  #v(0.5em)
  NovaType est un système de composition de documents moderne, conçu pour remplacer LaTeX avec une approche plus intuitive et des fonctionnalités adaptées aux besoins actuels.

  #v(0.8em)
  #text(weight: "bold", fill: primary)[Mots-clés]
  #h(0.5em)
  #box(fill: rgb("#e0f2fe"), radius: 3pt, inset: (x: 6pt, y: 3pt))[typesetting]
  #box(fill: rgb("#e0f2fe"), radius: 3pt, inset: (x: 6pt, y: 3pt))[documents]
  #box(fill: rgb("#e0f2fe"), radius: 3pt, inset: (x: 6pt, y: 3pt))[rust]
]

#pagebreak()

= Introduction

NovaType est né d'une frustration commune : LaTeX @lamport1994 est puissant mais archaïque. Inspiré par les travaux pionniers de Knuth sur la programmation lettrée @knuth1984, et par le nouveau système Typst @typst2023, nous avons besoin d'un outil moderne qui combine :

- *Simplicité* d'utilisation
- *Puissance* de mise en page
- *Intégration* des données

== Objectifs du projet

Le projet vise à offrir :

+ Une syntaxe claire et intuitive
+ Des templates professionnels (IEEE, APA, Springer)
+ L'intégration native de graphiques depuis CSV/JSON
+ Un rendu web via WebAssembly

= Architecture

Le système repose sur plusieurs composants :

#figure(
  table(
    columns: (1fr, 2fr),
    inset: 12pt,
    align: (left, left),
    stroke: none,
    fill: (x, y) => if y == 0 { primary } else if calc.odd(y) { rgb("#f8fafc") } else { white },
    table.header(
      text(fill: white, weight: "bold")[Composant],
      text(fill: white, weight: "bold")[Rôle],
    ),
    [nova-schema], [Validation des métadonnées],
    [nova-cite], [Gestion des citations],
    [nova-plot], [Visualisation de données],
    [nova-core], [Orchestration],
  ),
  caption: [Composants de NovaType],
)

== Pipeline de compilation

Le document passe par plusieurs étapes :

#v(1em)
#align(center)[
  #let step(name) = box(
    fill: white,
    stroke: 2pt + primary,
    radius: 6pt,
    inset: (x: 12pt, y: 8pt),
  )[#text(weight: "medium")[#name]]

  #let arrow = text(fill: primary, size: 16pt)[→]

  #step[Source] #arrow #step[Parse] #arrow #step[Validate] #arrow #step[Render] #arrow #step[PDF]
]
#v(1em)

= Utilisation des formules

NovaType permet l'insertion d'équations mathématiques avec une syntaxe proche de LaTeX.

== Équations inline

La célèbre formule d'Einstein $E = m c^2$ @einstein1905 révolutionna la physique. On peut aussi écrire des expressions comme $integral_0^infinity e^(-x^2) dif x = sqrt(pi)/2$.

== Équations bloc numérotées

La somme des inverses des carrés converge vers :

$ sum_(n=1)^infinity 1/n^2 = pi^2/6 $ <eq:basel>

L'équation de Schrödinger @schrodinger1926 s'écrit :

$ i planck.reduce (diff Psi) / (diff t) = -(planck.reduce^2) / (2m) nabla^2 Psi + V Psi $ <eq:schrodinger>

La transformée de Fourier @fourier1822 est définie par :

$ hat(f)(xi) = integral_(-infinity)^(+infinity) f(x) e^(-2 pi i x xi) dif x $ <eq:fourier>

== Références aux équations

Comme démontré dans l'@eq:basel (problème de Bâle), les séries convergent de manière élégante. L'@eq:schrodinger décrit l'évolution quantique, tandis que l'@eq:fourier est fondamentale en traitement du signal.

== Comparaison LaTeX / Typst

#figure(
  table(
    columns: (1fr, 1fr),
    inset: 10pt,
    align: (left, left),
    stroke: 0.5pt + secondary,
    [*LaTeX*], [*Typst*],
    [`\frac{a}{b}`], [`a/b`],
    [`\sum_{i=1}^{n}`], [`sum_(i=1)^n`],
    [`\int_0^\infty`], [`integral_0^infinity`],
    [`\sqrt{x}`], [`sqrt(x)`],
    [`\alpha, \beta`], [`alpha, beta`],
    [`\partial`], [`diff` ou `partial`],
  ),
  caption: [Équivalences LaTeX → Typst],
)

= Conclusion

NovaType représente une nouvelle approche de la composition documentaire, alliant modernité et puissance. Développé en Rust @rust2024, le projet est open-source et disponible pour contribuer.

#v(2em)
#align(center)[
  #rect(
    fill: rgb("#f0fdf4"),
    stroke: 1pt + rgb("#22c55e"),
    radius: 6pt,
    inset: 1em,
  )[
    #text(fill: rgb("#16a34a"), weight: "medium")[✓ Prêt pour la production]
  ]
]

// ============================================
// BIBLIOGRAPHIE
// ============================================

#pagebreak()

#heading(numbering: none)[Références]

#bibliography("references.bib", style: "ieee")
