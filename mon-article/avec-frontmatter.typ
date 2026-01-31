---
title: "Document avec Frontmatter"
authors:
  - name: Aurec
    affiliation: NovaType
date: "2026-01-29"
keywords:
  - test
  - frontmatter
---

#set document(title: "Document avec Frontmatter")
#set page(paper: "a4", margin: 2cm)
#set text(size: 11pt)

= Test du Frontmatter

Ce document contient du frontmatter YAML qui doit etre retire avant compilation.

== Validation

Si vous voyez ce PDF, le preprocessing a fonctionne correctement !

Les metadonnees du frontmatter:
- Titre: Document avec Frontmatter
- Auteur: Aurec
- Date: 2026-01-29

== Contenu

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
