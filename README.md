# NovaType

**La nouvelle frontiere de la composition de documents**

NovaType est un systeme moderne de composition de documents, construit sur le moteur Typst, concu pour remplacer LaTeX avec une solution plus performante, accessible et extensible.

## Caracteristiques

- **Performance Rust** : Compilation incrementale en millisecondes
- **Syntaxe accessible** : Proche du Markdown, sans boilerplate
- **Templates universels** : Separation contenu/style avec switch zero-friction
- **Data-science native** : Visualisation de donnees integree (CSV/JSON vers graphes)
- **Cloud-native** : Distribution WebAssembly pour execution navigateur

## Installation

```bash
# Via Cargo
cargo install nova-cli

# Ou telecharger les binaires pre-compiles
# https://github.com/novatype/novatype/releases
```

## Utilisation rapide

```bash
# Creer un nouveau projet
nova init mon-article --template ieee-article

# Compiler un document
nova compile main.typ

# Mode watch (recompilation automatique)
nova watch main.typ

# Valider les metadonnees
nova validate main.typ
```

## Structure du projet

```
novatype/
├── crates/
│   ├── nova-core/      # Moteur de compilation
│   ├── nova-schema/    # Validation schemas/metadonnees
│   ├── nova-cite/      # Gestion des citations
│   ├── nova-plot/      # Visualisation de donnees
│   └── nova-cli/       # Interface ligne de commande
├── nova-wasm/          # Build WebAssembly
├── templates/          # Templates officiels
└── schemas/            # Schemas de validation JSON
```

## Exemple de document

```typ
---
title: "Mon Article Scientifique"
authors:
  - name: Jean Dupont
    email: jean@universite.fr
    affiliation: Universite de Paris
template: ieee-article
citation_style: ieee
bibliography:
  - references.bib
---

= Introduction

Ceci est un exemple de document NovaType.

= Methodes

Description des methodes utilisees.

= Resultats

Presentation des resultats avec visualisation:

#nova-plot(
  data: "data.csv",
  type: "line",
  title: "Evolution temporelle"
)

= Conclusion

Resume des conclusions.
```

## Developpement

```bash
# Cloner le depot
git clone https://github.com/novatype/novatype.git
cd novatype

# Compiler
cargo build

# Executer les tests
cargo test --workspace

# Verifier le formatage et les lints
cargo fmt --check
cargo clippy --all-targets

# Generer la documentation
cargo doc --workspace --open
```

## Architecture

Le projet suit une architecture modulaire avec des crates independants:

| Crate | Description |
|-------|-------------|
| `nova-core` | Orchestration de la compilation |
| `nova-schema` | Validation JSON Schema des metadonnees |
| `nova-cite` | Citations (BibTeX, CrossRef, Zotero) |
| `nova-plot` | Rendu SVG de graphiques |
| `nova-cli` | Interface utilisateur CLI |
| `nova-wasm` | Bindings WebAssembly |

## Contribuer

Les contributions sont les bienvenues! Consultez [CONTRIBUTING.md](CONTRIBUTING.md) pour les directives.

## Licence

Distribue sous licence MIT ou Apache-2.0, au choix.
