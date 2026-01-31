# Contributing to NovaType

Merci de votre interet pour contribuer a NovaType ! Ce guide vous aidera a demarrer.

## Table des matieres

- [Code de Conduite](#code-de-conduite)
- [Comment Contribuer](#comment-contribuer)
- [Configuration de l'Environnement](#configuration-de-lenvironnement)
- [Workflow de Developpement](#workflow-de-developpement)
- [Standards de Code](#standards-de-code)
- [Processus de Pull Request](#processus-de-pull-request)
- [Architecture du Projet](#architecture-du-projet)

## Code de Conduite

Ce projet adhère au [Code de Conduite](CODE_OF_CONDUCT.md). En participant, vous vous engagez a respecter ses termes.

## Comment Contribuer

### Signaler un Bug

1. Verifiez que le bug n'a pas deja ete signale dans les [Issues](https://github.com/AureClai/novatype/issues)
2. Creez une nouvelle issue en utilisant le template "Bug Report"
3. Incluez un exemple minimal reproductible si possible

### Proposer une Fonctionnalite

1. Verifiez que la fonctionnalite n'a pas deja ete proposee
2. Ouvrez une [Discussion](https://github.com/AureClai/novatype/discussions) pour en discuter
3. Une fois validee, creez une issue "Feature Request"

### Contribuer au Code

1. Choisissez une issue avec le label `good first issue` pour commencer
2. Commentez l'issue pour indiquer que vous travaillez dessus
3. Forkez le repository et creez une branche
4. Suivez le workflow de developpement ci-dessous

## Configuration de l'Environnement

### Prerequis

- [Rust](https://rustup.rs/) 1.70 ou superieur
- [Git](https://git-scm.com/)
- Un editeur avec support Rust (VS Code + rust-analyzer recommande)

### Installation

```bash
# Cloner le repository
git clone https://github.com/AureClai/novatype.git
cd novatype

# Compiler le projet
cd typst
cargo build

# Verifier que tout fonctionne
cargo test --workspace
```

### Structure du Repository

```
novatype/
├── typst/                    # Fork Typst avec crates NovaType
│   ├── crates/
│   │   ├── nova-core/        # Orchestration et compilation
│   │   ├── nova-schema/      # Validation JSON Schema
│   │   ├── nova-cite/        # Gestion des citations
│   │   ├── nova-plot/        # Visualisation de donnees
│   │   ├── nova-cli/         # Interface ligne de commande
│   │   └── nova-wasm/        # Build WebAssembly
│   └── Cargo.toml
├── docs/                     # Site de documentation
├── .github/                  # Templates et workflows
└── README.md
```

## Workflow de Developpement

### Branches

- `master` - Branche principale, toujours stable
- `feature/*` - Nouvelles fonctionnalites
- `fix/*` - Corrections de bugs
- `docs/*` - Documentation

### Creer une Branche

```bash
# Mettre a jour master
git checkout master
git pull origin master

# Creer une branche de feature
git checkout -b feature/ma-fonctionnalite

# Ou pour un fix
git checkout -b fix/mon-fix
```

### Commits

Nous utilisons [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[body optionnel]

[footer optionnel]
```

**Types:**
- `feat` - Nouvelle fonctionnalite
- `fix` - Correction de bug
- `docs` - Documentation
- `style` - Formatage (pas de changement de code)
- `refactor` - Refactoring
- `test` - Ajout ou modification de tests
- `chore` - Maintenance

**Exemples:**
```bash
git commit -m "feat(nova-cite): add CrossRef API support"
git commit -m "fix(nova-cli): handle spaces in file paths"
git commit -m "docs: update installation guide"
```

## Standards de Code

### Formatage

```bash
# Formater le code
cargo fmt

# Verifier le formatage
cargo fmt --check
```

### Linting

```bash
# Lancer clippy
cargo clippy --all-targets --all-features

# Avec warnings comme erreurs
cargo clippy --all-targets --all-features -- -D warnings
```

### Tests

```bash
# Lancer tous les tests
cargo test --workspace

# Tests d'un crate specifique
cargo test --package nova-core

# Tests avec output
cargo test --workspace -- --nocapture
```

### Documentation

```bash
# Generer la documentation
cargo doc --workspace --no-deps

# Ouvrir dans le navigateur
cargo doc --workspace --no-deps --open
```

### Checklist avant PR

- [ ] `cargo fmt` passe sans erreur
- [ ] `cargo clippy` passe sans warning
- [ ] `cargo test --workspace` passe
- [ ] La documentation est a jour
- [ ] Les commits suivent Conventional Commits

## Processus de Pull Request

### 1. Preparer la PR

```bash
# Rebaser sur master
git fetch origin
git rebase origin/master

# Pousser la branche
git push origin feature/ma-fonctionnalite
```

### 2. Creer la PR

1. Allez sur GitHub et creez une Pull Request
2. Remplissez le template PR
3. Liez les issues concernees (`Fixes #123`)
4. Demandez une review

### 3. Review

- Repondez aux commentaires
- Faites les modifications demandees
- Re-demandez une review si necessaire

### 4. Merge

Une fois approuvee, la PR sera mergee par un mainteneur.

## Architecture du Projet

### nova-core

Moteur principal d'orchestration. Gere le pipeline de compilation.

```rust
// Exemple d'utilisation interne
use nova_core::NovaCompiler;

let compiler = NovaCompiler::new();
let result = compiler.compile("main.typ")?;
```

### nova-schema

Validation des metadonnees YAML avec JSON Schema.

### nova-cite

Gestion des citations: BibTeX, CrossRef API, styles de citation.

### nova-plot

Generation de graphiques SVG depuis CSV/JSON.

### nova-cli

Interface utilisateur: `nova init`, `nova compile`, `nova watch`, etc.

### nova-wasm

Bindings WebAssembly pour execution dans le navigateur.

## Besoin d'Aide ?

- [Discussions GitHub](https://github.com/AureClai/novatype/discussions) - Questions generales
- [Issues](https://github.com/AureClai/novatype/issues) - Bugs et features
- [Documentation](https://aureclai.github.io/novatype/) - Guides et references

---

Merci de contribuer a NovaType !
