<p align="center">
  <img src="docs/assets/images/logo.svg" alt="NovaType Logo" width="120" height="120">
</p>

<h1 align="center">NovaType</h1>

<p align="center">
  <strong>La composition typographique, reimaginee.</strong>
</p>

<p align="center">
  <a href="https://aureclai.github.io/novatype/">Documentation</a> •
  <a href="https://aureclai.github.io/novatype/demos.html">Demos</a> •
  <a href="#installation">Installation</a> •
  <a href="#utilisation">Utilisation</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License MIT">
  <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust 1.70+">
  <img src="https://img.shields.io/badge/platform-windows%20%7C%20macos%20%7C%20linux-lightgrey.svg" alt="Platform">
</p>

---

NovaType est un systeme moderne de composition typographique construit sur [Typst](https://typst.app). Il combine la qualite de LaTeX avec une syntaxe intuitive et une compilation instantanee.

## Pourquoi NovaType ?

| Fonctionnalite | NovaType | LaTeX | Word |
|----------------|:--------:|:-----:|:----:|
| Compilation instantanee | ✅ | ❌ | ✅ |
| Equations mathematiques | ✅ | ✅ | ❌ |
| Syntaxe simple | ✅ | ❌ | ✅ |
| Qualite typographique | ✅ | ✅ | ❌ |
| Visualisation de donnees | ✅ | ❌ | ✅ |
| Execution navigateur | ✅ | ❌ | ❌ |

## Caracteristiques

- **Compilation instantanee** — Visualisez vos modifications en temps reel
- **Syntaxe intuitive** — Proche du Markdown, sans boilerplate LaTeX
- **Citations intelligentes** — BibTeX, CrossRef API, Zotero integres
- **Visualisation de donnees** — Graphiques depuis CSV/JSON
- **Templates professionnels** — IEEE, Nature, rapports, CV...
- **WebAssembly** — Compilez dans le navigateur

## Installation

```bash
# Via Cargo (recommande)
cargo install nova-cli

# Verifier l'installation
nova --version
```

<details>
<summary>Autres methodes d'installation</summary>

### Depuis les sources

```bash
git clone https://github.com/AureClai/novatype.git
cd novatype/typst
cargo build --package nova-cli --release
```

### Windows (binaires)

Telechargez depuis [Releases](https://github.com/AureClai/novatype/releases).

</details>

## Utilisation

```bash
# Creer un nouveau projet
nova init mon-article

# Avec un template specifique
nova init mon-article --template ieee-article

# Compiler
nova compile main.typ --open

# Mode watch (recompilation automatique)
nova watch main.typ
```

## Exemple

```typ
---
title: "Mon Article"
author: "Jean Dupont"
template: article
---

= Introduction

Bienvenue dans *NovaType* ! Une syntaxe _simple_ et puissante.

= Equations

La formule d'Euler : $ e^(i pi) + 1 = 0 $

= Visualisation

#nova-plot(
  data: "results.csv",
  type: "bar",
  title: "Resultats"
)
```

## Architecture

```
novatype/
├── typst/crates/
│   ├── nova-core/      # Moteur de compilation
│   ├── nova-schema/    # Validation des metadonnees
│   ├── nova-cite/      # Gestion des citations
│   ├── nova-plot/      # Visualisation de donnees
│   ├── nova-cli/       # Interface ligne de commande
│   └── nova-wasm/      # Build WebAssembly
└── docs/               # Site de documentation
```

## Contribuer

Les contributions sont les bienvenues ! Consultez les [Issues](https://github.com/AureClai/novatype/issues) pour commencer.

```bash
# Cloner et compiler
git clone https://github.com/AureClai/novatype.git
cd novatype/typst
cargo build

# Lancer les tests
cargo test --workspace
```

## Licence

MIT License - voir [LICENSE](LICENSE) pour plus de details.

---

<p align="center">
  <a href="https://aureclai.github.io/novatype/">Site web</a> •
  <a href="https://github.com/AureClai/novatype/issues">Signaler un bug</a> •
  <a href="https://github.com/AureClai/novatype/discussions">Discussions</a>
</p>
